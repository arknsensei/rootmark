//! Chunk records (GOAL P1-C "Chunk kaynağı"): the retrieval units of the
//! index, from `core`'s structural chunker. A record is one chunk occurrence
//! `(doc_path, chunk_id)` with the blocks it covers; `record_id` hashes all of
//! it, because one `(doc_path, chunk_id)` may occur with a different heading
//! path or lineages in another snapshot (the chunk id hashes version ids
//! only). Within one snapshot `(doc_path, chunk_id)` is unique.

use anyhow::{bail, Context, Result};
use mdkcore::chunk::structural;
use mdkcore::events::read_events;
use mdkcore::vault::{Replayed, Snapshot};
use mdkcore::{hash32, Block, EventKind, Vault};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockRef {
    pub lineage_id: String,
    pub version_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub record_id: String,
    pub doc_path: String,
    pub chunk_id: String,
    pub heading_path: Vec<String>,
    /// covered blocks in document order
    pub blocks: Vec<BlockRef>,
    /// distinct lineages of `blocks`, first occurrence order
    pub lineage_ids: Vec<String>,
    /// chunk text (`heading_path` prefix, spec §6)
    pub text: String,
    /// number of lexical tokens of `text` (BM25 document length)
    pub lexical_len: u32,
}

impl ChunkRecord {
    /// Tie-break and storage order: `(doc_path, chunk_id)`, then `record_id`.
    pub fn sort_key(&self) -> (&str, &str, &str) {
        (&self.doc_path, &self.chunk_id, &self.record_id)
    }
}

/// The chunks of one document snapshot, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotChunks {
    pub doc_path: String,
    pub snapshot_id: String,
    pub records: Vec<String>,
}

fn record_id(
    doc_path: &str,
    chunk_id: &str,
    heading_path: &[String],
    blocks: &[BlockRef],
    text: &str,
) -> Result<String> {
    Ok(hash32(&serde_json::to_string(&(
        doc_path,
        chunk_id,
        heading_path,
        blocks,
        text,
    ))?))
}

/// Structural chunks of `blocks` (one snapshot of `doc`) as records.
pub fn chunk_records(doc: &str, blocks: &[Block]) -> Result<Vec<ChunkRecord>> {
    let mut out = Vec::new();
    let mut i = 0;
    for c in structural(doc, blocks) {
        let n = c.version_ids.len();
        let span = blocks
            .get(i..i + n)
            .with_context(|| format!("{doc}: chunk {} runs past the snapshot", c.chunk_id))?;
        if span.iter().map(|b| &b.version_id).ne(c.version_ids.iter()) {
            bail!(
                "{doc}: chunk {} is not a run of consecutive blocks",
                c.chunk_id
            );
        }
        i += n;
        let refs: Vec<BlockRef> = span
            .iter()
            .map(|b| BlockRef {
                lineage_id: b.lineage_id.clone(),
                version_id: b.version_id.clone(),
            })
            .collect();
        let lexical_len = u32::try_from(crate::tokenize::tokens(&c.text).len())?;
        out.push(ChunkRecord {
            record_id: record_id(doc, &c.chunk_id, &c.heading_path, &refs, &c.text)?,
            doc_path: doc.to_string(),
            chunk_id: c.chunk_id,
            heading_path: c.heading_path,
            blocks: refs,
            lineage_ids: c.lineage_ids,
            text: c.text,
            lexical_len,
        });
    }
    if i != blocks.len() {
        bail!("{doc}: chunks cover {i} of {} blocks", blocks.len());
    }
    Ok(out)
}

/// Snapshots an index build covers: every document head and every document
/// snapshot named by a checkpoint, `(doc_path, snapshot_id)` → snapshot
/// (one replay of the log).
pub fn build_snapshots(vault: &Vault) -> Result<BTreeMap<(String, String), Snapshot>> {
    let mut st = Replayed::default();
    let mut out = BTreeMap::new();
    let take = |st: &Replayed, out: &mut BTreeMap<(String, String), Snapshot>| {
        for (d, ds) in &st.docs {
            if let Some(h) = &ds.head {
                out.insert((d.clone(), h.snapshot_id.clone()), h.clone());
            }
        }
    };
    for ev in read_events(&vault.events_path())? {
        st.apply(&ev)?;
        if ev.kind == EventKind::CheckpointCreated {
            take(&st, &mut out);
        }
    }
    take(&st, &mut out);
    for d in vault.doc_paths() {
        let h = vault.snapshot(&d)?.context("head")?;
        if !out.contains_key(&(d.clone(), h.snapshot_id.clone())) {
            bail!("{d}: head {} differs from the replayed log", h.snapshot_id);
        }
    }
    Ok(out)
}

/// Records of one snapshot.
pub fn snapshot_records(vault: &Vault, snap: &Snapshot) -> Result<Vec<ChunkRecord>> {
    chunk_records(&snap.doc_path, &vault.snapshot_blocks(snap)?)
}
