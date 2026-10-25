//! Block locations in responses (P2-B/B1; docs/CLIENT_CONTRACT.md §2.5 `loc`).
//!
//! `blame`, `read_block` and `search` hits carry per block `loc {line_start,
//! line_end, byte_start, byte_end, content_hash}`: where the block stands in
//! the bytes of the version the response describes (`content_hash`: those
//! bytes' hash). The response's `loc_stale` says whether the file on disk
//! holds other bytes:
//! - the file parses into the version's blocks (same block versions, same
//!   order): the locations are the file's own, `loc_stale: false`;
//! - otherwise (changed and not yet ingested, removed, or the response is of
//!   an earlier checkpoint): `loc_stale: true`, and the locations are those
//!   recorded when that version was ingested (`rootmarkcore::loc`) — never the
//!   old offsets applied to the new file. A version ingested before P2-B has
//!   no record: its blocks carry no `loc`.
//!
//! A location is derived: nothing is written (no event, id, `events_hash`,
//! trace or markdown byte changes).

use rootmarkcore::{BlockLoc, Snapshot, Vault};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// The locations of one version of a document.
pub struct DocLocs {
    pub stale: bool,
    /// `content_hash` of the bytes the locations are taken from (`None`:
    /// no location known)
    pub content_hash: Option<String>,
    /// per block of the snapshot, in its order
    pub blocks: Option<Vec<BlockLoc>>,
}

impl DocLocs {
    /// The `loc` of the snapshot's `i`-th block, if known.
    pub fn loc(&self, i: usize) -> Option<Value> {
        let (b, h) = (self.blocks.as_ref()?.get(i)?, self.content_hash.as_ref()?);
        Some(json!({
            "line_start": b.line_start,
            "line_end": b.line_end,
            "byte_start": b.byte_start,
            "byte_end": b.byte_end,
            "content_hash": h,
        }))
    }
}

/// The locations of `snap` (a version of its document; at the head or at a
/// checkpoint) given the file on disk.
pub fn doc_locs(vault: &Path, v: &Vault, snap: &Snapshot) -> anyhow::Result<DocLocs> {
    let vids: Vec<&str> = snap.blocks.iter().map(|(_, x)| x.as_str()).collect();
    // the file as it is now: its locations, if it holds this version
    if let Ok(p) = crate::paths::doc_file(vault, &snap.doc_path) {
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                let locs = rootmarkcore::locate_blocks(text);
                if locs
                    .iter()
                    .map(|b| b.version_id.as_str())
                    .eq(vids.iter().copied())
                {
                    return Ok(DocLocs {
                        stale: false,
                        content_hash: Some(rootmarkcore::loc::content_hash(text)),
                        blocks: Some(locs),
                    });
                }
            }
        }
    }
    Ok(match v.loc_record(snap)? {
        Some(r) => DocLocs {
            stale: true,
            content_hash: Some(r.content_hash),
            blocks: Some(r.blocks),
        },
        None => DocLocs {
            stale: true,
            content_hash: None,
            blocks: None,
        },
    })
}

/// Adds `loc` to each `blocks[]` row (snapshot order) and `loc_stale` to the
/// object (`blame`).
pub fn annotate_rows(out: &mut Value, rows: &str, locs: &DocLocs) {
    if let Some(a) = out[rows].as_array_mut() {
        for (i, row) in a.iter_mut().enumerate() {
            if let Some(l) = locs.loc(i) {
                row["loc"] = l;
            }
        }
    }
    out["loc_stale"] = json!(locs.stale);
}

/// Adds to each search hit `loc` (one entry per `lineage_ids` entry, `null`
/// where unknown) and `loc_stale`, each document's locations as of the
/// search's checkpoint.
pub fn annotate_hits(vault: &Path, v: &Vault, out: &mut Value) -> anyhow::Result<()> {
    let Some(cp) = out["checkpoint_id"].as_str().map(str::to_string) else {
        return Ok(());
    };
    let mut cache: HashMap<String, (DocLocs, HashMap<String, usize>)> = HashMap::new();
    let Some(hits) = out["hits"].as_array_mut() else {
        return Ok(());
    };
    for h in hits {
        let doc = h["doc_path"].as_str().unwrap_or_default().to_string();
        if !cache.contains_key(&doc) {
            let snap = v
                .snapshot_at(&doc, &cp)?
                .ok_or_else(|| anyhow::anyhow!("{doc}: no snapshot at checkpoint {cp}"))?;
            let pos = snap
                .blocks
                .iter()
                .enumerate()
                .map(|(i, (l, _))| (l.clone(), i))
                .collect();
            cache.insert(doc.clone(), (doc_locs(vault, v, &snap)?, pos));
        }
        let (locs, pos) = &cache[&doc];
        let loc: Vec<Value> = h["lineage_ids"]
            .as_array()
            .map(|ls| {
                ls.iter()
                    .map(|l| {
                        l.as_str()
                            .and_then(|l| pos.get(l))
                            .and_then(|&i| locs.loc(i))
                            .unwrap_or(Value::Null)
                    })
                    .collect()
            })
            .unwrap_or_default();
        h["loc"] = json!(loc);
        h["loc_stale"] = json!(locs.stale);
    }
    Ok(())
}
