//! Block locations of ingested document versions (P2-B/B1; docs/CLIENT_CONTRACT.md
//! §2.5 `loc`).
//!
//! A location is derived from a document's bytes (`parse::locate_blocks`):
//! it never enters an event, an id or `events_hash`, and nothing is written
//! to the markdown file. The bytes of an ingested version are not kept, so
//! the locations of each version are recorded when it is ingested, under
//! `.rootmark/loc/<H(doc_path)>/<snapshot_id>.json`: the version's blocks in
//! snapshot order with their lines and bytes, and the `content_hash` of the
//! bytes they were taken from. A server answers with them when the file on
//! disk no longer holds that version (`loc_stale`). A record is a cache of
//! derived data: rewritten by the next ingest of the same snapshot (the
//! same blocks, possibly other bytes between them), absent in a vault
//! ingested before P2-B — then no location is given, never a guessed one.

use crate::model::hash32;
use crate::parse::{locate_blocks, BlockLoc};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const LOC_FORMAT: &str = "rootmark-loc/1";

/// `content_hash` of a document's bytes: the first 32 hex characters of
/// their BLAKE3 (the attestation's `content_hash`, docs/CLIENT_CONTRACT.md §2.6).
pub fn content_hash(text: &str) -> String {
    hash32(text)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocRecord {
    pub format: String,
    pub doc_path: String,
    pub snapshot_id: String,
    pub content_hash: String,
    pub blocks: Vec<BlockLoc>,
}

impl LocRecord {
    /// The locations of `text` (a version of `doc_path`).
    pub fn of(doc_path: &str, snapshot_id: &str, text: &str) -> LocRecord {
        LocRecord {
            format: LOC_FORMAT.into(),
            doc_path: doc_path.into(),
            snapshot_id: snapshot_id.into(),
            content_hash: content_hash(text),
            blocks: locate_blocks(text),
        }
    }

    /// The record must describe the version whose block versions are
    /// `vids` (snapshot order); anything else is an error, never a location.
    pub fn check(&self, doc_path: &str, snapshot_id: &str, vids: &[&str]) -> Result<()> {
        if self.format != LOC_FORMAT || self.doc_path != doc_path || self.snapshot_id != snapshot_id
        {
            bail!("location record of {doc_path} / {snapshot_id} names another version or format");
        }
        let have: Vec<&str> = self.blocks.iter().map(|b| b.version_id.as_str()).collect();
        if have != vids {
            bail!("location record of {doc_path} / {snapshot_id} does not match the snapshot's blocks");
        }
        Ok(())
    }
}

pub fn record_path(meta_dir: &Path, doc_path: &str, snapshot_id: &str) -> PathBuf {
    meta_dir
        .join("loc")
        .join(hash32(doc_path))
        .join(format!("{snapshot_id}.json"))
}

/// Writes `r` (replacing an earlier record of the same version).
pub fn write(meta_dir: &Path, r: &LocRecord) -> Result<()> {
    let p = record_path(meta_dir, &r.doc_path, &r.snapshot_id);
    let dir = p.parent().expect("record path has a parent");
    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    let mut bytes = serde_json::to_vec(r)?;
    bytes.push(b'\n');
    let tmp = dir.join(format!(".{}.tmp", r.snapshot_id));
    fs::write(&tmp, &bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &p).with_context(|| format!("rename to {}", p.display()))?;
    Ok(())
}

/// The record of a version, `None` when none was written; an unreadable
/// record is an error.
pub fn read(meta_dir: &Path, doc_path: &str, snapshot_id: &str) -> Result<Option<LocRecord>> {
    let p = record_path(meta_dir, doc_path, snapshot_id);
    if !p.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&p).with_context(|| format!("read {}", p.display()))?;
    let r: LocRecord = serde_json::from_slice(&bytes)
        .with_context(|| format!("{}: not a location record", p.display()))?;
    Ok(Some(r))
}
