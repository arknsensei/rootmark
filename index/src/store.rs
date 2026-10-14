//! The on-disk index of a vault, `<vault>/.mdkernel/index/` (GOAL P1-C):
//!
//! - `manifest.json` — format, counts, tokenizer and BM25 parameters, blake3
//!   of `records.jsonl` and `snapshots.jsonl`
//! - `records.jsonl` — every chunk record the index has seen, in storage
//!   order `(doc_path, chunk_id, record_id)`
//! - `snapshots.jsonl` — `(doc_path, snapshot_id)` → its records in document
//!   order; a query at a checkpoint reads exactly the snapshots it names
//! - `lexical/` — tantivy (`crate::lexical`), fast field `ord` = line of
//!   `records.jsonl`
//!
//! A build covers every document head and every snapshot named by a
//! checkpoint (`record::build_snapshots`), writes into
//! `.mdkernel/index.build/` and replaces the index by renaming it. Every
//! file is a function of the vault: no time, no path, no randomness.

use crate::lexical::{self, Lexical};
use crate::record::{build_snapshots, snapshot_records, ChunkRecord, SnapshotChunks};
use anyhow::{bail, Context, Result};
use mdkcore::Vault;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "mdkernel-index/1";
/// BM25 parameters: tantivy's defaults (GOAL "BM25 varsayılan parametreler").
pub const BM25_K1: f64 = 1.2;
pub const BM25_B: f64 = 0.75;
pub const TOKENIZER: &str = "lowercase, [a-z0-9]+";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub chunking: String,
    pub tokenizer: String,
    pub bm25_k1: f64,
    pub bm25_b: f64,
    pub records: usize,
    pub snapshots: usize,
    pub records_blake3: String,
    pub snapshots_blake3: String,
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

/// Rebuilds `<vault>/.mdkernel/index/` from the vault.
pub fn build(vault: &Vault) -> Result<BuildReport> {
    let (records, snaps) = collect(vault)?;
    let dir = index_dir(vault);
    let tmp = vault.store_dir().join("index.build");
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }
    fs::create_dir_all(&tmp)?;
    let rec_bytes = jsonl(&records)?;
    let snap_bytes = jsonl(&snaps)?;
    fs::write(tmp.join("records.jsonl"), &rec_bytes)?;
    fs::write(tmp.join("snapshots.jsonl"), &snap_bytes)?;
    lexical::write(&tmp.join("lexical"), &records)?;
    let manifest = Manifest {
        format: FORMAT.into(),
        chunking: "structural".into(),
        tokenizer: TOKENIZER.into(),
        bm25_k1: BM25_K1,
        bm25_b: BM25_B,
        records: records.len(),
        snapshots: snaps.len(),
        records_blake3: blake3::hash(&rec_bytes).to_hex().to_string(),
        snapshots_blake3: blake3::hash(&snap_bytes).to_hex().to_string(),
    };
    write_pretty(&tmp.join("manifest.json"), &manifest)?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::rename(&tmp, &dir)?;
    Ok(BuildReport {
        records: records.len(),
        snapshots: snaps.len(),
    })
}

/// An opened index: records, snapshot map, lexical postings.
pub struct Store {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub records: Vec<ChunkRecord>,
    by_id: HashMap<String, usize>,
    snapshots: HashMap<(String, String), Vec<usize>>,
    pub lexical: Lexical,
}

impl Store {
    pub fn open(vault: &Vault) -> Result<Store> {
        let dir = index_dir(vault);
        if !dir.join("manifest.json").is_file() {
            bail!("no index at {} (run `mdk index build`)", dir.display());
        }
        let manifest: Manifest =
            serde_json::from_str(&fs::read_to_string(dir.join("manifest.json"))?)
                .context("index manifest.json")?;
        if manifest.format != FORMAT {
            bail!("index format {:?} != {FORMAT:?}", manifest.format);
        }
        if manifest.bm25_k1 != BM25_K1
            || manifest.bm25_b != BM25_B
            || manifest.tokenizer != TOKENIZER
        {
            bail!("index was built with other lexical parameters; rebuild it");
        }
        let (records, rh): (Vec<ChunkRecord>, String) = read_jsonl(&dir.join("records.jsonl"))?;
        let (snaps, sh): (Vec<SnapshotChunks>, String) = read_jsonl(&dir.join("snapshots.jsonl"))?;
        if rh != manifest.records_blake3 || sh != manifest.snapshots_blake3 {
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
        let by_id: HashMap<String, usize> = records
            .iter()
            .enumerate()
            .map(|(i, r)| (r.record_id.clone(), i))
            .collect();
        let mut snapshots = HashMap::new();
        for s in snaps {
            let ix = s
                .records
                .iter()
                .map(|id| {
                    by_id
                        .get(id)
                        .copied()
                        .with_context(|| format!("snapshots.jsonl: unknown record {id}"))
                })
                .collect::<Result<Vec<_>>>()?;
            if snapshots
                .insert((s.doc_path.clone(), s.snapshot_id.clone()), ix)
                .is_some()
            {
                bail!("snapshots.jsonl: {}@{} twice", s.doc_path, s.snapshot_id);
            }
        }
        let lexical = Lexical::open(&dir.join("lexical"))?;
        if lexical.num_docs() != records.len() as u64 {
            bail!(
                "lexical index holds {} documents, records.jsonl {}",
                lexical.num_docs(),
                records.len()
            );
        }
        Ok(Store {
            dir,
            manifest,
            records,
            by_id,
            snapshots,
            lexical,
        })
    }

    pub fn record_index(&self, record_id: &str) -> Option<usize> {
        self.by_id.get(record_id).copied()
    }

    /// Records (storage positions) of one document snapshot; a snapshot the
    /// index never saw is an error, never an empty result.
    pub fn snapshot(&self, doc_path: &str, snapshot_id: &str) -> Result<&[usize]> {
        self.snapshots
            .get(&(doc_path.to_string(), snapshot_id.to_string()))
            .map(Vec::as_slice)
            .with_context(|| {
                format!(
                    "snapshot {snapshot_id} of {doc_path} is not indexed (run `mdk index build`)"
                )
            })
    }
}
