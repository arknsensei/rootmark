//! Lexical index on tantivy (GOAL P1-C "Sözcüksel"; spec §9).
//!
//! tantivy holds the inverted index of the cold records: one document per
//! record, written by one writer thread in storage order (`(doc_path,
//! chunk_id)`, then `record_id`) and committed once as a single segment (an
//! intermediate segment is an error, see `write_with_budget`); the fast field `ord` is the record's position in
//! `records.jsonl`. Text is indexed with term frequencies only, through
//! `RootmarkTokenizer` (`crate::tokenize`).
//!
//! Byte-identical directories (GOAL C-6): segment contents are a function of
//! the documents, but tantivy names the segment with a random UUID and lists
//! its files in hash order. After the writer is closed the segment is renamed
//! to `blake3(file names + bytes)[..32]`, `meta.json` is rewritten with that
//! id, `.managed.json` lists the files sorted, and the writer's lock files are
//! removed. Nothing else is touched.
//!
//! Scoring is not tantivy's: BM25 statistics must be those of the queried
//! checkpoint's records (an index that also holds other chunk versions must
//! rank exactly as one rebuilt from that checkpoint, GOAL C-5), so the
//! postings give `(ord, tf)` and `crate::search` computes BM25 with tantivy's
//! default parameters and IDF formula.
//!
//! The read side never writes (K-C1.4): `Lexical::open` opens the directory
//! through `ReadOnlyDir`, which refuses every write, delete and the writer
//! lock, and takes the reader's meta lock without creating a lock file (the
//! index directory is replaced as a whole by a build, never modified in
//! place, so a reader needs no lock against a writer).

use crate::record::ChunkRecord;
use crate::tokenize::{Variant, VariantTokenizer};
use anyhow::{bail, Context, Result};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use tantivy::columnar::Column;
use tantivy::directory::error::{DeleteError, LockError, OpenReadError, OpenWriteError};
use tantivy::directory::{
    Directory, DirectoryLock, FileHandle, Lock, MmapDirectory, WatchCallback, WatchHandle,
    WritePtr, META_LOCK,
};
use tantivy::postings::Postings;
use tantivy::schema::{Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST};
use tantivy::{DocSet, Index, IndexReader, ReloadPolicy, TantivyDocument, Term, TERMINATED};

const TEXT: &str = "text";
const ORD: &str = "ord";
/// Writer memory budget per byte of indexed text. The writer must never
/// flush an intermediate segment: tantivy merges segments of equal size in
/// an order set by their random ids, so the merged document order — and the
/// bytes — would vary between builds (C2 review). The budget is sized from
/// the corpus, a corpus that cannot fit is an error, and `write` verifies
/// one segment whose document order is storage order (`ord == doc`).
const BUDGET_PER_TEXT_BYTE: usize = 32;
const BUDGET_FLOOR: usize = 200_000_000;
/// tantivy 0.26 per-thread writer budget bounds (`index_writer.rs`, not
/// exported): `MARGIN_IN_BYTES` 1 MB; min 15 MB, max `u32::MAX` − 1 MB.
pub const WRITER_BUDGET_MIN: usize = 15_000_000;
pub const WRITER_BUDGET_MAX: usize = u32::MAX as usize - 1_000_000;

fn schema(tok: Variant) -> (Schema, Field, Field) {
    let mut sb = Schema::builder();
    let opts = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(tok.name())
            .set_index_option(IndexRecordOption::WithFreqs)
            .set_fieldnorms(false),
    );
    let text = sb.add_text_field(TEXT, opts);
    let ord = sb.add_u64_field(ORD, FAST);
    (sb.build(), text, ord)
}

/// Every variant under its name: the index's schema names the one it uses.
fn register(index: &Index) {
    for v in Variant::ALL {
        index.tokenizers().register(
            v.name(),
            tantivy::tokenizer::TextAnalyzer::from(VariantTokenizer(v)),
        );
    }
}

/// Writer budget for `records`: `BUDGET_PER_TEXT_BYTE` × text bytes, at
/// least `BUDGET_FLOOR`; beyond tantivy's maximum → error.
pub fn budget_for(records: &[ChunkRecord]) -> Result<usize> {
    let text: usize = records.iter().map(|r| r.text.len()).sum();
    let want = text
        .checked_mul(BUDGET_PER_TEXT_BYTE)
        .context("lexical budget overflow")?
        .max(BUDGET_FLOOR);
    if want >= WRITER_BUDGET_MAX {
        bail!(
            "{text} bytes of chunk text exceed the single-segment lexical index (writer budget {want} > {})",
            WRITER_BUDGET_MAX
        );
    }
    Ok(want)
}

/// Writes the tantivy index of `records` (storage order) into the empty or
/// missing directory `dir`, then canonicalizes it (the default variant).
pub fn write(dir: &Path, records: &[ChunkRecord]) -> Result<()> {
    write_variant(dir, records, Variant::default())
}

/// `write` with the lexical variant `tok` (P2-L/L1).
pub fn write_variant(dir: &Path, records: &[ChunkRecord], tok: Variant) -> Result<()> {
    write_with(dir, records, budget_for(records)?, tok)
}

/// `write` with an explicit writer budget (tests force intermediate
/// segments with it): anything but one segment in storage order is an error.
pub fn write_with_budget(dir: &Path, records: &[ChunkRecord], budget: usize) -> Result<()> {
    write_with(dir, records, budget, Variant::default())
}

fn write_with(dir: &Path, records: &[ChunkRecord], budget: usize, tok: Variant) -> Result<()> {
    if dir.exists() && fs::read_dir(dir)?.next().is_some() {
        bail!("{} is not empty", dir.display());
    }
    fs::create_dir_all(dir)?;
    let (schema, text, ord) = schema(tok);
    let index = Index::create_in_dir(dir, schema)?;
    register(&index);
    let mut w = index.writer_with_num_threads::<TantivyDocument>(1, budget)?;
    for (i, r) in records.iter().enumerate() {
        let mut d = TantivyDocument::default();
        d.add_text(text, &r.text);
        d.add_u64(ord, u64::try_from(i)?);
        w.add_document(d)?;
    }
    w.commit()?;
    w.wait_merging_threads()?;
    let n = index.searchable_segment_ids()?.len();
    if n > 1 {
        bail!(
            "lexical writer flushed {n} segments (budget {budget} bytes): their merge order would not be deterministic"
        );
    }
    // document order must be storage order
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;
    let searcher: tantivy::Searcher = reader.searcher();
    for seg in searcher.segment_readers() {
        let col = seg.fast_fields().u64(ORD)?;
        for doc in 0..seg.max_doc() {
            if col.first(doc) != Some(u64::from(doc)) {
                bail!("lexical document {doc} is not record {doc}");
            }
        }
    }
    if searcher.num_docs() != records.len() as u64 {
        bail!(
            "lexical index holds {} of {} records",
            searcher.num_docs(),
            records.len()
        );
    }
    drop(searcher);
    drop(reader);
    canonicalize(dir)
}

fn hyphenated(simple: &str) -> String {
    format!(
        "{}-{}-{}-{}-{}",
        &simple[0..8],
        &simple[8..12],
        &simple[12..16],
        &simple[16..20],
        &simple[20..32]
    )
}

/// Deterministic segment name, sorted `.managed.json`, no lock files.
fn canonicalize(dir: &Path) -> Result<()> {
    let meta_p = dir.join("meta.json");
    let mut meta = fs::read_to_string(&meta_p).context("lexical meta.json")?;
    let v: serde_json::Value = serde_json::from_str(&meta)?;
    let segs = v["segments"].as_array().context("meta.json: segments")?;
    if segs.len() > 1 {
        bail!("lexical index has {} segments", segs.len());
    }
    if let Some(seg) = segs.first() {
        let old_h = seg["segment_id"]
            .as_str()
            .context("segment_id")?
            .to_string();
        let old = old_h.replace('-', "");
        let mut parts: Vec<(String, std::path::PathBuf)> = Vec::new();
        for e in fs::read_dir(dir)? {
            let e = e?;
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(ext) = name.strip_prefix(&format!("{old}.")) {
                parts.push((ext.to_string(), e.path()));
            }
        }
        parts.sort();
        if parts.is_empty() {
            bail!("segment {old} has no files");
        }
        let mut h = blake3::Hasher::new();
        for (ext, p) in &parts {
            h.update(ext.as_bytes());
            h.update(b"\n");
            h.update(&fs::read(p)?);
        }
        let new = h.finalize().to_hex()[..32].to_string();
        for (ext, p) in &parts {
            fs::rename(p, dir.join(format!("{new}.{ext}")))?;
        }
        meta = meta.replace(&old_h, &hyphenated(&new));
        if meta.contains(&old_h) || meta.contains(&old) {
            bail!("meta.json still names segment {old}");
        }
        fs::write(&meta_p, &meta)?;
    }
    for lock in [".tantivy-meta.lock", ".tantivy-writer.lock"] {
        let p = dir.join(lock);
        if p.exists() {
            fs::remove_file(p)?;
        }
    }
    let mut files: Vec<String> = fs::read_dir(dir)?
        .map(|e| Ok(e?.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>>>()?;
    files.retain(|f| f != ".managed.json");
    files.sort();
    fs::write(dir.join(".managed.json"), serde_json::to_string(&files)?)?;
    Ok(())
}

/// A tantivy directory that cannot be written (K-C1.4): reads go to the
/// memory-mapped directory; writes, deletes, `sync_directory` and any lock
/// but the reader's meta lock are errors; the meta lock is taken without a
/// lock file.
#[derive(Clone, Debug)]
struct ReadOnlyDir(MmapDirectory);

fn read_only(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!(
            "{}: the lexical index is opened read-only (a query never writes)",
            path.display()
        ),
    )
}

impl Directory for ReadOnlyDir {
    fn get_file_handle(&self, path: &Path) -> Result<Arc<dyn FileHandle>, OpenReadError> {
        self.0.get_file_handle(path)
    }

    fn delete(&self, path: &Path) -> Result<(), DeleteError> {
        Err(DeleteError::IoError {
            io_error: Arc::new(read_only(path)),
            filepath: path.to_path_buf(),
        })
    }

    fn exists(&self, path: &Path) -> Result<bool, OpenReadError> {
        self.0.exists(path)
    }

    fn open_write(&self, path: &Path) -> Result<WritePtr, OpenWriteError> {
        Err(OpenWriteError::IoError {
            io_error: Arc::new(read_only(path)),
            filepath: path.to_path_buf(),
        })
    }

    fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
        self.0.atomic_read(path)
    }

    fn atomic_write(&self, path: &Path, _data: &[u8]) -> io::Result<()> {
        Err(read_only(path))
    }

    fn sync_directory(&self) -> io::Result<()> {
        Err(read_only(Path::new(".")))
    }

    fn acquire_lock(&self, lock: &Lock) -> Result<DirectoryLock, LockError> {
        if lock.filepath == META_LOCK.filepath {
            Ok(DirectoryLock::from(Box::new(())))
        } else {
            Err(LockError::IoError(Arc::new(read_only(&lock.filepath))))
        }
    }

    fn watch(&self, cb: WatchCallback) -> tantivy::Result<WatchHandle> {
        self.0.watch(cb)
    }
}

/// Read side: `(ord, tf)` postings of a term.
pub struct Lexical {
    reader: IndexReader,
    text: Field,
    ords: Vec<Column<u64>>,
    num_docs: u64,
}

impl Lexical {
    /// Opens `dir` read-only (K-C1.4): nothing in it is created or changed.
    pub fn open(dir: &Path) -> Result<Lexical> {
        let mmap = MmapDirectory::open(dir)
            .with_context(|| format!("open lexical index {}", dir.display()))?;
        let index = Index::open(ReadOnlyDir(mmap))
            .with_context(|| format!("open lexical index {}", dir.display()))?;
        register(&index);
        let text = index.schema().get_field(TEXT)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        let searcher = reader.searcher();
        let mut ords = Vec::new();
        for seg in searcher.segment_readers() {
            if seg.alive_bitset().is_some() {
                bail!("lexical index has deleted documents");
            }
            ords.push(seg.fast_fields().u64(ORD)?);
        }
        let num_docs = searcher.num_docs();
        Ok(Lexical {
            reader,
            text,
            ords,
            num_docs,
        })
    }

    pub fn num_docs(&self) -> u64 {
        self.num_docs
    }

    /// Every `(ord, term frequency)` of `token`, in ord order.
    pub fn postings(&self, token: &str) -> Result<Vec<(u64, u32)>> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.text, token);
        let mut out = Vec::new();
        for (seg, ords) in searcher.segment_readers().iter().zip(&self.ords) {
            let inv = seg.inverted_index(self.text)?;
            if let Some(mut p) = inv.read_postings(&term, IndexRecordOption::WithFreqs)? {
                let mut doc = p.doc();
                while doc != TERMINATED {
                    let ord = ords
                        .first(doc)
                        .with_context(|| format!("lexical doc {doc} without ord"))?;
                    out.push((ord, p.term_freq()));
                    doc = p.advance();
                }
            }
        }
        out.sort_unstable();
        Ok(out)
    }
}
