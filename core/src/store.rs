//! Content-addressed object and snapshot store (SPEC §4).
//!
//! Stored objects are pure functions of their key so that "same key, different
//! bytes" is always a real error: a block object holds only the fields that
//! `version_id` determines, a snapshot object only the ordered `version_id`s
//! that `snapshot_id` hashes. Occurrence data (lineage, raw text, heading path,
//! frontmatter, parent) lives in the event log and is rebuilt by `replay`.

use crate::model::BlockType;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockObject {
    pub version_id: String,
    pub block_type: BlockType,
    pub normalized_text: String,
    pub heading_level: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotObject {
    pub snapshot_id: String,
    pub version_ids: Vec<String>,
}

pub fn object_path(root: &Path, vid: &str) -> PathBuf {
    root.join("objects")
        .join(&vid[..2])
        .join(format!("{vid}.json"))
}

pub fn snapshot_path(root: &Path, sid: &str) -> PathBuf {
    root.join("snapshots").join(format!("{sid}.json"))
}

/// Writes `bytes` to `path` unless it exists. Existing identical bytes → Ok
/// (file untouched); existing different bytes → Err.
pub fn write_immutable(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        let cur = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        if cur != bytes {
            bail!(
                "object conflict: {} exists with different content",
                path.display()
            );
        }
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn put_object(root: &Path, obj: &BlockObject) -> Result<()> {
    let expect = crate::model::version_id(obj.block_type, &obj.normalized_text);
    if obj.version_id != expect {
        bail!(
            "object {} does not hash to its content ({expect})",
            obj.version_id
        );
    }
    write_immutable(
        &object_path(root, &obj.version_id),
        &serde_json::to_vec(obj)?,
    )
}

pub fn get_object(root: &Path, vid: &str) -> Result<BlockObject> {
    if vid.len() < 2 {
        bail!("invalid version_id {vid:?}");
    }
    let p = object_path(root, vid);
    let bytes = fs::read(&p).with_context(|| format!("object {vid} not found"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn put_snapshot(root: &Path, obj: &SnapshotObject) -> Result<()> {
    write_immutable(
        &snapshot_path(root, &obj.snapshot_id),
        &serde_json::to_vec(obj)?,
    )
}

pub fn get_snapshot(root: &Path, sid: &str) -> Result<SnapshotObject> {
    let bytes =
        fs::read(snapshot_path(root, sid)).with_context(|| format!("snapshot {sid} not found"))?;
    Ok(serde_json::from_slice(&bytes)?)
}
