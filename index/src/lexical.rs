//! Lexical index on tantivy (GOAL P1-C "Sözcüksel"; spec §9).
//!
//! tantivy holds the inverted index of the cold records: one document per
//! record, written by one writer thread in storage order (`(doc_path,
//! chunk_id)`, then `record_id`), committed once and merged into a single
//! segment; the fast field `ord` is the record's position in
//! `records.jsonl`. Text is indexed with term frequencies only, through
//! `MdkTokenizer` (`crate::tokenize`).
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

use crate::record::ChunkRecord;
use crate::tokenize::{MdkTokenizer, TOKENIZER_NAME};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use tantivy::columnar::Column;
use tantivy::postings::Postings;
use tantivy::schema::{Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST};
use tantivy::{DocSet, Index, IndexReader, ReloadPolicy, TantivyDocument, Term, TERMINATED};

const TEXT: &str = "text";
const ORD: &str = "ord";
/// Writer memory budget: large enough for one segment at the measured scale;
/// more segments are merged into one anyway.
const WRITER_BUDGET: usize = 200_000_000;

fn schema() -> (Schema, Field, Field) {
    let mut sb = Schema::builder();
    let opts = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(TOKENIZER_NAME)
            .set_index_option(IndexRecordOption::WithFreqs)
            .set_fieldnorms(false),
    );
    let text = sb.add_text_field(TEXT, opts);
    let ord = sb.add_u64_field(ORD, FAST);
    (sb.build(), text, ord)
}

fn register(index: &Index) {
    index.tokenizers().register(
        TOKENIZER_NAME,
        tantivy::tokenizer::TextAnalyzer::from(MdkTokenizer),
    );
}

/// Writes the tantivy index of `records` (storage order) into the empty or
/// missing directory `dir`, then canonicalizes it.
pub fn write(dir: &Path, records: &[ChunkRecord]) -> Result<()> {
    if dir.exists() && fs::read_dir(dir)?.next().is_some() {
        bail!("{} is not empty", dir.display());
    }
    fs::create_dir_all(dir)?;
    let (schema, text, ord) = schema();
    let index = Index::create_in_dir(dir, schema)?;
    register(&index);
    let mut w = index.writer_with_num_threads::<TantivyDocument>(1, WRITER_BUDGET)?;
    for (i, r) in records.iter().enumerate() {
        let mut d = TantivyDocument::default();
        d.add_text(text, &r.text);
        d.add_u64(ord, u64::try_from(i)?);
        w.add_document(d)?;
    }
    w.commit()?;
    let ids = index.searchable_segment_ids()?;
    if ids.len() > 1 {
        w.merge(&ids).wait()?;
    }
    w.wait_merging_threads()?;
    let n = index.searchable_segment_ids()?.len();
    if n > usize::from(!records.is_empty()) {
        bail!("lexical index has {n} segments after merge");
    }
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

/// Read side: `(ord, tf)` postings of a term.
pub struct Lexical {
    reader: IndexReader,
    text: Field,
    ords: Vec<Column<u64>>,
    num_docs: u64,
}

impl Lexical {
    pub fn open(dir: &Path) -> Result<Lexical> {
        let index = Index::open_in_dir(dir)
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
