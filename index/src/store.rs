//! The on-disk index of a vault, `<vault>/.rootmark/index/` (GOAL P1-C):
//!
//! - `manifest.json` — format, counts, the lexical scoring record
//!   (`lexical.{variant, idf, k1, b, tokenizer, dl}`, K-C1.1, K-D1.7), embedder, blake3
//!   of every other file (`store::load` checks them all; a missing or
//!   different lexical record is an error)
//! - `records.jsonl` — every chunk record the index has seen, in storage
//!   order `(doc_path, chunk_id, record_id)`
//! - `snapshots.jsonl` — `(doc_path, snapshot_id)` → its records in document
//!   order; a query at a checkpoint reads exactly the snapshots it names
//! - `lexical/` — tantivy (`crate::lexical`), fast field `ord` = line of
//!   `records.jsonl`
//! - `vectors/` — one vector per record and its `embedder_id`
//!   (`crate::vectors`); one embedder per index
//!
//! A build covers every document head and every snapshot named by a
//! checkpoint (`record::build_snapshots`), writes into
//! `.rootmark/index.build/` and replaces the index by renaming it. Every
//! file is a function of the vault and the embedder: no time, no path, no
//! randomness. A rebuild reuses the vector of a record the previous index
//! already holds under the same `embedder_id` (embedding is deterministic,
//! C1), so only new records are embedded.

use crate::embed::Embedder;
use crate::lexical::{self, Lexical};
use crate::record::{build_snapshots, snapshot_records, ChunkRecord, SnapshotChunks};
use crate::vectors::{self, VectorMeta};
use anyhow::{bail, Context, Result};
use rootmarkcore::Vault;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// `/2` (P1-C.1): the lexical scoring record replaced the `/1` fields
/// `tokenizer`, `bm25_k1`, `bm25_b`. `/3` (P1-D.1, K-D1.7): the record
/// declares the document length (`lexical.dl`). A data identifier written
/// into every manifest and compared when an index is opened: it kept its
/// value through the rename (K-GH.1, GH-0 ASSUMPTIONS G0-2), so the index
/// bytes did not change and a migrated vault's index is read as it is.
pub const FORMAT: &str = "mdkernel-index/3";
/// BM25 parameters: tantivy's defaults (GOAL P1-C "BM25 varsayılan
/// parametreler"); the variant is declared in the manifest (K-C1.1).
pub const BM25_VARIANT: &str = "lucene-bm25";
pub const BM25_IDF: &str = "ln(1 + (N - n + 0.5) / (n + 0.5))";
pub const BM25_K1: f64 = 1.2;
pub const BM25_B: f64 = 0.75;
/// The document length BM25 uses (K-D1.7): a chunk's exact token count as a
/// 64-bit float, the average not quantized (no one-byte field norms).
pub const BM25_DL: &str = "exact token count, f64, no fieldnorm quantization";

/// How the lexical scores of this index are computed (K-C1.1; spec §9
/// informative note). `search::bm25_with` implements exactly this record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalParams {
    pub variant: String,
    pub idf: String,
    pub k1: f64,
    pub b: f64,
    /// tokenizer id (`crate::tokenize::TOKENIZER_NAME`: lower case, maximal
    /// runs of `[a-z0-9]`)
    pub tokenizer: String,
    /// document length (`BM25_DL`; K-D1.7)
    pub dl: String,
}

impl LexicalParams {
    /// The record of this build of the index.
    pub fn current() -> LexicalParams {
        LexicalParams {
            variant: BM25_VARIANT.into(),
            idf: BM25_IDF.into(),
            k1: BM25_K1,
            b: BM25_B,
            tokenizer: crate::tokenize::TOKENIZER_NAME.into(),
            dl: BM25_DL.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub chunking: String,
    pub lexical: LexicalParams,
    pub embedder_id: String,
    pub dim: usize,
    pub records: usize,
    pub snapshots: usize,
    pub records_blake3: String,
    pub snapshots_blake3: String,
    pub vectors_blake3: String,
    pub embedders_blake3: String,
}

pub fn index_dir(vault: &Vault) -> PathBuf {
    vault.store_dir().join("index")
}

fn jsonl<T: Serialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for x in items {
        serde_json::to_writer(&mut out, x)?;
        out.push(b'\n');
    }
    Ok(out)
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(p: &Path) -> Result<(Vec<T>, String)> {
    let bytes = fs::read(p).with_context(|| format!("read {}", p.display()))?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    let text = std::str::from_utf8(&bytes)?;
    let items = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            serde_json::from_str(l).with_context(|| format!("{} line {}", p.display(), i + 1))
        })
        .collect::<Result<Vec<T>>>()?;
    Ok((items, hash))
}

fn write_pretty(p: &Path, v: &impl Serialize) -> Result<()> {
    let mut text = serde_json::to_string_pretty(v)?;
    text.push('\n');
    fs::write(p, text)?;
    Ok(())
}

/// Result of `build`.
#[derive(Debug, Clone, Serialize)]
pub struct BuildReport {
    pub records: usize,
    pub snapshots: usize,
    pub embedder_id: String,
    /// records embedded by this build (the others reused their vector)
    pub embedded: usize,
    /// where reused vectors came from, or why none were reused
    pub reuse: String,
}

/// Chunk records of every snapshot an index build covers, deduplicated and
/// in storage order, plus the snapshot → records map.
pub fn collect(vault: &Vault) -> Result<(Vec<ChunkRecord>, Vec<SnapshotChunks>)> {
    let mut records: BTreeMap<String, ChunkRecord> = BTreeMap::new();
    let mut snaps = Vec::new();
    for ((doc, sid), snap) in build_snapshots(vault)? {
        let recs = snapshot_records(vault, &snap)?;
        let mut seen = std::collections::HashSet::new();
        for r in &recs {
            if !seen.insert(r.chunk_id.clone()) {
                bail!("{doc}@{sid}: chunk {} occurs twice", r.chunk_id);
            }
        }
        snaps.push(SnapshotChunks {
            doc_path: doc,
            snapshot_id: sid,
            records: recs.iter().map(|r| r.record_id.clone()).collect(),
        });
        for r in recs {
            records.entry(r.record_id.clone()).or_insert(r);
        }
    }
    let mut records: Vec<ChunkRecord> = records.into_values().collect();
    records.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok((records, snaps))
}

/// A whole index directory read and checked: manifest, every file against
/// its manifest hash and count, storage order, vectors aligned with records
/// under one `embedder_id`. The only reader of an index directory (`Store`
/// and vector reuse), so nothing is used from an index that fails a check.
struct Loaded {
    manifest: Manifest,
    records: Vec<ChunkRecord>,
    snaps: Vec<SnapshotChunks>,
    vecs: Vec<Vec<f32>>,
}

/// The index is missing, of another format or of another lexical record:
/// nothing in it is wrong, it is only not this implementation's, and a
/// rebuild (`rootmark index build`) makes it current (K-E0.3). The message is the
/// reason. Every other failure of `load` (a file that does not match the
/// manifest, broken order, misaligned vectors) is damage, not staleness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexStale(pub String);

impl std::fmt::Display for IndexStale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for IndexStale {}

/// Why the index of `vault` is stale (`IndexStale`), or `None` when its
/// manifest is of this format and lexical record. Reads `manifest.json`
/// only; the full check is `Store::open`.
pub fn staleness(vault: &Vault) -> Result<Option<IndexStale>> {
    match manifest_of(&index_dir(vault)) {
        Ok(_) => Ok(None),
        Err(e) => match e.downcast::<IndexStale>() {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(e),
        },
    }
}

/// `manifest.json` of `dir`, checked for format and lexical record first
/// (an index of another format says so: `IndexStale`).
fn manifest_of(dir: &Path) -> Result<Manifest> {
    let stale = |m: String| anyhow::Error::new(IndexStale(m));
    if !dir.join("manifest.json").is_file() {
        return Err(stale(format!(
            "no index at {} (run `rootmark index build`)",
            dir.display()
        )));
    }
    let text = fs::read_to_string(dir.join("manifest.json"))?;
    // the format first, so that an index of another format says so
    let raw: serde_json::Value = serde_json::from_str(&text).context("index manifest.json")?;
    if raw["format"] != FORMAT {
        return Err(stale(format!(
            "index format {} != {FORMAT:?} (rebuild it with `rootmark index build`)",
            raw["format"]
        )));
    }
    let manifest: Manifest = serde_json::from_value(raw).context("index manifest.json")?;
    if manifest.lexical != LexicalParams::current() {
        return Err(stale(format!(
            "index lexical record {:?} != this implementation's {:?}; rebuild it",
            manifest.lexical,
            LexicalParams::current()
        )));
    }
    Ok(manifest)
}

fn load(dir: &Path) -> Result<Loaded> {
    let manifest = manifest_of(dir)?;
    let (records, rh): (Vec<ChunkRecord>, String) = read_jsonl(&dir.join("records.jsonl"))?;
    let (snaps, sh): (Vec<SnapshotChunks>, String) = read_jsonl(&dir.join("snapshots.jsonl"))?;
    let (metas, eh): (Vec<VectorMeta>, String) = read_jsonl(&dir.join("vectors/embedders.jsonl"))?;
    let vb = fs::read(dir.join("vectors/vectors.f32")).context("vectors.f32")?;
    if rh != manifest.records_blake3
        || sh != manifest.snapshots_blake3
        || eh != manifest.embedders_blake3
        || blake3::hash(&vb).to_hex().as_str() != manifest.vectors_blake3
    {
        bail!("index files do not match manifest.json (blake3)");
    }
    if records.len() != manifest.records || snaps.len() != manifest.snapshots {
        bail!("index files do not match manifest.json (counts)");
    }
    if records
        .windows(2)
        .any(|w| w[0].sort_key() >= w[1].sort_key())
    {
        bail!("records.jsonl is not in storage order");
    }
    let vecs = vectors::decode(&vb, manifest.dim)?;
    if vecs.len() != records.len()
        || metas.len() != records.len()
        || metas
            .iter()
            .zip(&records)
            .any(|(m, r)| m.record_id != r.record_id)
    {
        bail!("vectors do not line up with records.jsonl");
    }
    let id = vectors::single_embedder(&metas)?;
    if !records.is_empty() && id.as_deref() != Some(manifest.embedder_id.as_str()) {
        bail!(
            "vectors carry embedder_id {id:?}, manifest {:?}",
            manifest.embedder_id
        );
    }
    Ok(Loaded {
        manifest,
        records,
        snaps,
        vecs,
    })
}

/// Vectors of the current index by `record_id` and why they are (not)
/// reused. Only an index that passes every `load` check and was built with
/// the same `embedder_id` and dimension lends vectors; otherwise everything
/// is embedded again (the result bytes are the same, embedding is
/// deterministic) and the reason is reported.
fn reusable(dir: &Path, embedder_id: &str, dim: usize) -> (HashMap<String, Vec<f32>>, String) {
    if !dir.exists() {
        return (HashMap::new(), "none: no previous index".into());
    }
    match load(dir) {
        Err(e) => (
            HashMap::new(),
            format!("none: previous index invalid: {e:#}"),
        ),
        Ok(l) if l.manifest.embedder_id != embedder_id || l.manifest.dim != dim => (
            HashMap::new(),
            format!(
                "none: previous index embedder {} (dim {})",
                l.manifest.embedder_id, l.manifest.dim
            ),
        ),
        Ok(l) => {
            let mut have: HashMap<String, Vec<f32>> = HashMap::new();
            let mut why = String::from("previous index");
            match load_hot(dir, &l.manifest) {
                Ok(Some(h)) => {
                    have.extend(h.records.into_iter().map(|r| r.record_id).zip(h.vecs));
                    why.push_str(" (cold + hot)");
                }
                Ok(None) => {}
                Err(e) => why.push_str(&format!(" (cold only; hot tier invalid: {e:#})")),
            }
            have.extend(l.records.into_iter().map(|r| r.record_id).zip(l.vecs));
            (have, why)
        }
    }
}

/// Writes a complete cold index (records in storage order) into the new
/// directory `tmp`.
fn write_cold(
    tmp: &Path,
    records: &[ChunkRecord],
    snaps: &[SnapshotChunks],
    vecs: &[Vec<f32>],
    embedder_id: &str,
    dim: usize,
) -> Result<()> {
    if records
        .windows(2)
        .any(|w| w[0].sort_key() >= w[1].sort_key())
    {
        bail!("records are not in storage order");
    }
    if vecs.len() != records.len() {
        bail!("{} vectors for {} records", vecs.len(), records.len());
    }
    if tmp.exists() {
        fs::remove_dir_all(tmp)?;
    }
    fs::create_dir_all(tmp)?;
    let metas: Vec<VectorMeta> = records
        .iter()
        .map(|r| VectorMeta {
            record_id: r.record_id.clone(),
            embedder_id: embedder_id.to_string(),
        })
        .collect();
    let rec_bytes = jsonl(records)?;
    let snap_bytes = jsonl(snaps)?;
    fs::write(tmp.join("records.jsonl"), &rec_bytes)?;
    fs::write(tmp.join("snapshots.jsonl"), &snap_bytes)?;
    lexical::write(&tmp.join("lexical"), records)?;
    let vec_bytes = vectors::encode(vecs, dim)?;
    fs::create_dir_all(tmp.join("vectors"))?;
    fs::write(tmp.join("vectors/vectors.f32"), &vec_bytes)?;
    let emb_bytes = jsonl(&metas)?;
    fs::write(tmp.join("vectors/embedders.jsonl"), &emb_bytes)?;
    let manifest = Manifest {
        format: FORMAT.into(),
        chunking: "structural".into(),
        lexical: LexicalParams::current(),
        embedder_id: embedder_id.to_string(),
        dim,
        records: records.len(),
        snapshots: snaps.len(),
        records_blake3: blake3::hash(&rec_bytes).to_hex().to_string(),
        snapshots_blake3: blake3::hash(&snap_bytes).to_hex().to_string(),
        vectors_blake3: blake3::hash(&vec_bytes).to_hex().to_string(),
        embedders_blake3: blake3::hash(&emb_bytes).to_hex().to_string(),
    };
    write_pretty(&tmp.join("manifest.json"), &manifest)
}

/// Replaces `<vault>/.rootmark/index/` (hot tier included) by `tmp`.
fn install(vault: &Vault, tmp: &Path) -> Result<()> {
    let dir = index_dir(vault);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::rename(tmp, &dir)?;
    Ok(())
}

/// Rebuilds `<vault>/.rootmark/index/` from the vault (the hot tier is
/// dropped: every head and checkpoint snapshot is in the new cold index).
pub fn build(vault: &Vault, embedder: &dyn Embedder) -> Result<BuildReport> {
    let (records, snaps) = collect(vault)?;
    let dir = index_dir(vault);
    let (mut have, reuse) = reusable(&dir, embedder.id(), embedder.dim());
    let missing: Vec<&ChunkRecord> = records
        .iter()
        .filter(|r| !have.contains_key(&r.record_id))
        .collect();
    let texts: Vec<&str> = missing.iter().map(|r| r.text.as_str()).collect();
    for (r, v) in missing.iter().zip(embedder.embed(&texts)?) {
        have.insert(r.record_id.clone(), v);
    }
    let vecs: Vec<Vec<f32>> = records
        .iter()
        .map(|r| have.remove(&r.record_id).context("vector"))
        .collect::<Result<_>>()?;
    let tmp = vault.store_dir().join("index.build");
    write_cold(&tmp, &records, &snaps, &vecs, embedder.id(), embedder.dim())?;
    install(vault, &tmp)?;
    Ok(BuildReport {
        records: records.len(),
        snapshots: snaps.len(),
        embedder_id: embedder.id().to_string(),
        embedded: missing.len(),
        reuse,
    })
}

// ---- hot tier (GOAL P1-C "Sıcak katman")

/// `<index>/hot/`: append-only files of the records a `watch` added after
/// the cold index was written — `records.jsonl`, `snapshots.jsonl`,
/// `vectors.f32`, `embedders.jsonl` (same formats as cold), and `base.json`
/// naming the cold index they extend (its `records_blake3`). A hot tier of
/// another cold index, misaligned files, or a record also in cold is an
/// error. `compact` moves everything to a new cold index.
pub const HOT_DIR: &str = "hot";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct HotBase {
    cold_records_blake3: String,
}

struct Hot {
    records: Vec<ChunkRecord>,
    snaps: Vec<SnapshotChunks>,
    vecs: Vec<Vec<f32>>,
}

fn load_hot(dir: &Path, cold: &Manifest) -> Result<Option<Hot>> {
    let h = dir.join(HOT_DIR);
    if !h.exists() {
        return Ok(None);
    }
    let base: HotBase =
        serde_json::from_str(&fs::read_to_string(h.join("base.json"))?).context("hot/base.json")?;
    if base.cold_records_blake3 != cold.records_blake3 {
        bail!("hot tier extends another cold index");
    }
    let (records, _): (Vec<ChunkRecord>, String) = read_jsonl(&h.join("records.jsonl"))?;
    let (snaps, _): (Vec<SnapshotChunks>, String) = read_jsonl(&h.join("snapshots.jsonl"))?;
    let (metas, _): (Vec<VectorMeta>, String) = read_jsonl(&h.join("embedders.jsonl"))?;
    let vecs = vectors::decode(&fs::read(h.join("vectors.f32"))?, cold.dim)?;
    if vecs.len() != records.len()
        || metas.len() != records.len()
        || metas
            .iter()
            .zip(&records)
            .any(|(m, r)| m.record_id != r.record_id || m.embedder_id != cold.embedder_id)
    {
        bail!("hot tier vectors do not line up with its records (or mix embedder ids)");
    }
    Ok(Some(Hot {
        records,
        snaps,
        vecs,
    }))
}

/// Result of `Store::watch`.
#[derive(Debug, Clone, Serialize)]
pub struct WatchReport {
    pub doc_path: String,
    pub snapshot_id: String,
    pub events: usize,
    /// chunks of the new snapshot
    pub chunks: usize,
    /// chunks new to the index, embedded and appended to the hot tier
    pub embedded: usize,
}

/// Result of `compact`.
#[derive(Debug, Clone, Serialize)]
pub struct CompactReport {
    pub records: usize,
    pub snapshots: usize,
    pub moved_from_hot: usize,
}

/// `index status`.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub embedder_id: String,
    pub cold_records: usize,
    pub cold_snapshots: usize,
    pub hot_records: usize,
    pub hot_snapshots: usize,
    /// documents whose head snapshot the index does not hold
    pub stale_docs: Vec<String>,
}

/// An opened index: records (cold, then hot), snapshot map, postings,
/// vectors.
pub struct Store {
    pub dir: PathBuf,
    pub manifest: Manifest,
    /// cold records in storage order, then hot records in append order
    pub records: Vec<ChunkRecord>,
    /// one vector per record
    pub vectors: Vec<Vec<f32>>,
    pub cold_len: usize,
    hot_snapshots: usize,
    by_id: HashMap<String, usize>,
    snapshots: HashMap<(String, String), Vec<usize>>,
    lexical: Lexical,
    /// hot postings: token → (position, term frequency), position order
    hot_postings: HashMap<String, Vec<(usize, u32)>>,
    /// a hot append failed part-way: the files may be misaligned, no
    /// further append (reopen to get the error, rebuild to repair)
    poisoned: bool,
}

impl Store {
    /// Opens the index of `vault` (cold and hot) for queries embedded by
    /// `embedder`: the index must hold exactly that `embedder_id`.
    pub fn open(vault: &Vault, embedder: &dyn Embedder) -> Result<Store> {
        let dir = index_dir(vault);
        let Loaded {
            manifest,
            mut records,
            snaps,
            mut vecs,
        } = load(&dir)?;
        if manifest.embedder_id != embedder.id() || manifest.dim != embedder.dim() {
            bail!(
                "index embedder_id {} (dim {}) != query embedder {} (dim {}): rebuild the index",
                manifest.embedder_id,
                manifest.dim,
                embedder.id(),
                embedder.dim()
            );
        }
        let lexical = Lexical::open(&dir.join("lexical"))?;
        if lexical.num_docs() != records.len() as u64 {
            bail!(
                "lexical index holds {} documents, records.jsonl {}",
                lexical.num_docs(),
                records.len()
            );
        }
        let cold_len = records.len();
        let mut all_snaps = snaps;
        let mut hot_snapshots = 0;
        if let Some(hot) = load_hot(&dir, &manifest)
            .context("hot tier invalid (rebuild with `rootmark index build`)")?
        {
            hot_snapshots = hot.snaps.len();
            records.extend(hot.records);
            vecs.extend(hot.vecs);
            all_snaps.extend(hot.snaps);
        }
        let mut store = Store {
            dir,
            manifest,
            records: Vec::new(),
            vectors: Vec::new(),
            cold_len,
            hot_snapshots,
            by_id: HashMap::new(),
            snapshots: HashMap::new(),
            lexical,
            hot_postings: HashMap::new(),
            poisoned: false,
        };
        for (r, v) in records.into_iter().zip(vecs) {
            store.add_record(r, v)?;
        }
        for s in all_snaps {
            store.add_snapshot(s)?;
        }
        Ok(store)
    }

    fn add_record(&mut self, r: ChunkRecord, v: Vec<f32>) -> Result<usize> {
        let i = self.records.len();
        if self.by_id.insert(r.record_id.clone(), i).is_some() {
            bail!("record {} is indexed twice", r.record_id);
        }
        if i >= self.cold_len {
            let mut tf: BTreeMap<String, u32> = BTreeMap::new();
            for t in crate::tokenize::tokens(&r.text) {
                *tf.entry(t).or_insert(0) += 1;
            }
            for (t, n) in tf {
                self.hot_postings.entry(t).or_default().push((i, n));
            }
        }
        self.records.push(r);
        self.vectors.push(v);
        Ok(i)
    }

    fn add_snapshot(&mut self, s: SnapshotChunks) -> Result<()> {
        let ix = s
            .records
            .iter()
            .map(|id| {
                self.by_id
                    .get(id)
                    .copied()
                    .with_context(|| format!("snapshot entry: unknown record {id}"))
            })
            .collect::<Result<Vec<_>>>()?;
        if self
            .snapshots
            .insert((s.doc_path.clone(), s.snapshot_id.clone()), ix)
            .is_some()
        {
            bail!("snapshot {}@{} indexed twice", s.doc_path, s.snapshot_id);
        }
        Ok(())
    }

    pub fn record_index(&self, record_id: &str) -> Option<usize> {
        self.by_id.get(record_id).copied()
    }

    pub fn hot_len(&self) -> usize {
        self.records.len() - self.cold_len
    }

    /// `(position, term frequency)` of `token` in every record, cold
    /// (tantivy) then hot (memory).
    pub fn postings(&self, token: &str) -> Result<Vec<(usize, u32)>> {
        let mut out: Vec<(usize, u32)> = self
            .lexical
            .postings(token)?
            .into_iter()
            .map(|(o, tf)| Ok((usize::try_from(o)?, tf)))
            .collect::<Result<_>>()?;
        if let Some(h) = self.hot_postings.get(token) {
            out.extend_from_slice(h);
        }
        Ok(out)
    }

    /// Records (positions) of one document snapshot; a snapshot the index
    /// never saw is an error, never an empty result.
    pub fn snapshot(&self, doc_path: &str, snapshot_id: &str) -> Result<&[usize]> {
        self.snapshots
            .get(&(doc_path.to_string(), snapshot_id.to_string()))
            .map(Vec::as_slice)
            .with_context(|| {
                format!(
                    "snapshot {snapshot_id} of {doc_path} is not indexed (run `rootmark index build` or `rootmark index watch`)"
                )
            })
    }

    pub fn has_snapshot(&self, doc_path: &str, snapshot_id: &str) -> bool {
        self.snapshots
            .contains_key(&(doc_path.to_string(), snapshot_id.to_string()))
    }

    /// GOAL "watch(doc)": ingests `text` as `doc_path` (`core::ingest`), then
    /// the chunks of the new head that the index does not hold are embedded
    /// and appended to the hot tier (memory and `hot/`). Unchanged chunks
    /// are not embedded again (`WatchReport.embedded`).
    pub fn watch(
        &mut self,
        vault: &mut Vault,
        embedder: &dyn Embedder,
        doc_path: &str,
        text: &str,
        prov: rootmarkcore::Provenance,
    ) -> Result<WatchReport> {
        let matcher = vault
            .matcher_lock()
            .unwrap_or(rootmarkcore::MatcherVersion::V3);
        self.watch_with(vault, embedder, doc_path, text, prov, matcher)
    }

    /// `watch` with an explicit matcher (a vault locked to another matcher
    /// refuses the ingest, `Vault::ingest`).
    pub fn watch_with(
        &mut self,
        vault: &mut Vault,
        embedder: &dyn Embedder,
        doc_path: &str,
        text: &str,
        prov: rootmarkcore::Provenance,
        matcher: rootmarkcore::MatcherVersion,
    ) -> Result<WatchReport> {
        if self.poisoned {
            bail!("an earlier hot append failed; reopen the index (`rootmark index build` repairs it)");
        }
        if embedder.id() != self.manifest.embedder_id {
            bail!(
                "watch embedder {} != index embedder {}",
                embedder.id(),
                self.manifest.embedder_id
            );
        }
        let res = vault.ingest(doc_path, text, prov, matcher)?;
        self.index_head(vault, embedder, doc_path, res.events.len())
    }

    /// `watch_with` for an ingest planned (and checked) beforehand
    /// (`Vault::plan_ingest`; the server's attributed writes, K-D1.1/K-D1.2):
    /// commits the plan, then indexes the new head like `watch_with`.
    pub fn watch_plan(
        &mut self,
        vault: &mut Vault,
        embedder: &dyn Embedder,
        plan: rootmarkcore::IngestPlan,
    ) -> Result<WatchReport> {
        self.check_watch(embedder)?;
        let doc_path = plan.doc_path.clone();
        let res = vault.commit_plan(plan)?;
        self.index_head(vault, embedder, &doc_path, res.events.len())
    }

    fn check_watch(&self, embedder: &dyn Embedder) -> Result<()> {
        if self.poisoned {
            bail!("an earlier hot append failed; reopen the index (`rootmark index build` repairs it)");
        }
        if embedder.id() != self.manifest.embedder_id {
            bail!(
                "watch embedder {} != index embedder {}",
                embedder.id(),
                self.manifest.embedder_id
            );
        }
        Ok(())
    }

    /// Indexes the head of `doc_path` after an ingest of `events` events.
    fn index_head(
        &mut self,
        vault: &mut Vault,
        embedder: &dyn Embedder,
        doc_path: &str,
        events: usize,
    ) -> Result<WatchReport> {
        let snap = vault
            .snapshot(doc_path)?
            .with_context(|| format!("{doc_path}: no head after ingest"))?;
        let mut report = WatchReport {
            doc_path: doc_path.to_string(),
            snapshot_id: snap.snapshot_id.clone(),
            events,
            chunks: 0,
            embedded: 0,
        };
        if self.has_snapshot(doc_path, &snap.snapshot_id) {
            report.chunks = self.snapshot(doc_path, &snap.snapshot_id)?.len();
            return Ok(report);
        }
        let recs = snapshot_records(vault, &snap)?;
        report.chunks = recs.len();
        let new: Vec<&ChunkRecord> = recs
            .iter()
            .filter(|r| !self.by_id.contains_key(&r.record_id))
            .collect();
        let texts: Vec<&str> = new.iter().map(|r| r.text.as_str()).collect();
        let vecs = embedder.embed(&texts)?;
        let entry = SnapshotChunks {
            doc_path: doc_path.to_string(),
            snapshot_id: snap.snapshot_id.clone(),
            records: recs.iter().map(|r| r.record_id.clone()).collect(),
        };
        // disk first (append), then memory
        let h = self.dir.join(HOT_DIR);
        if !h.exists() {
            fs::create_dir_all(&h)?;
            write_pretty(
                &h.join("base.json"),
                &HotBase {
                    cold_records_blake3: self.manifest.records_blake3.clone(),
                },
            )?;
        }
        let owned: Vec<ChunkRecord> = new.iter().map(|r| (*r).clone()).collect();
        let metas: Vec<VectorMeta> = owned
            .iter()
            .map(|r| VectorMeta {
                record_id: r.record_id.clone(),
                embedder_id: embedder.id().to_string(),
            })
            .collect();
        let (rb, eb, vb, sb) = (
            jsonl(&owned)?,
            jsonl(&metas)?,
            vectors::encode(&vecs, self.manifest.dim)?,
            jsonl(std::slice::from_ref(&entry))?,
        );
        self.poisoned = true;
        append(&h.join("records.jsonl"), &rb)?;
        append(&h.join("embedders.jsonl"), &eb)?;
        append(&h.join("vectors.f32"), &vb)?;
        append(&h.join("snapshots.jsonl"), &sb)?;
        self.poisoned = false;
        report.embedded = owned.len();
        for (r, v) in owned.into_iter().zip(vecs) {
            self.add_record(r, v)?;
        }
        self.add_snapshot(entry)?;
        self.hot_snapshots += 1;
        Ok(report)
    }

    pub fn status(&self, vault: &Vault) -> Result<Status> {
        let mut stale = Vec::new();
        for d in vault.doc_paths() {
            let h = vault.snapshot(&d)?.context("head")?;
            if !self.has_snapshot(&d, &h.snapshot_id) {
                stale.push(d);
            }
        }
        Ok(Status {
            embedder_id: self.manifest.embedder_id.clone(),
            cold_records: self.cold_len,
            cold_snapshots: self.manifest.snapshots,
            hot_records: self.hot_len(),
            hot_snapshots: self.hot_snapshots,
            stale_docs: stale,
        })
    }
}

fn append(p: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new().create(true).append(true).open(p)?;
    f.write_all(bytes)?;
    f.sync_data()?;
    Ok(())
}

/// GOAL "compact()": moves the hot tier into a new cold index — every
/// record and snapshot the index holds, nothing embedded again — and drops
/// `hot/`. Rankings do not change (statistics are per checkpoint, C2-1).
pub fn compact(vault: &Vault, embedder: &dyn Embedder) -> Result<CompactReport> {
    let s = Store::open(vault, embedder)?;
    let moved = s.hot_len();
    let mut order: Vec<usize> = (0..s.records.len()).collect();
    order.sort_by(|&a, &b| s.records[a].sort_key().cmp(&s.records[b].sort_key()));
    let records: Vec<ChunkRecord> = order.iter().map(|&i| s.records[i].clone()).collect();
    let vecs: Vec<Vec<f32>> = order.iter().map(|&i| s.vectors[i].clone()).collect();
    let mut keys: Vec<&(String, String)> = s.snapshots.keys().collect();
    keys.sort();
    let snaps: Vec<SnapshotChunks> = keys
        .into_iter()
        .map(|k| SnapshotChunks {
            doc_path: k.0.clone(),
            snapshot_id: k.1.clone(),
            records: s.snapshots[k]
                .iter()
                .map(|&i| s.records[i].record_id.clone())
                .collect(),
        })
        .collect();
    let tmp = vault.store_dir().join("index.build");
    write_cold(
        &tmp,
        &records,
        &snaps,
        &vecs,
        &s.manifest.embedder_id,
        s.manifest.dim,
    )?;
    drop(s);
    install(vault, &tmp)?;
    Ok(CompactReport {
        records: records.len(),
        snapshots: snaps.len(),
        moved_from_hot: moved,
    })
}
