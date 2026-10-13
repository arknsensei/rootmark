//! Append-only JSONL event log (SPEC §3).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    BlockCreated,
    BlockUpdated,
    BlockMoved,
    BlockSplit,
    BlockMerged,
    BlockDeleted,
    SnapshotCommitted,
}

/// One log line. Field semantics per kind:
/// - `BlockCreated`: `lineage_ids=[new]`, `new_version_ids=[v]`.
/// - `BlockUpdated`: `lineage_ids=[l]`, `old_version_ids=[v_old]`, `new_version_ids=[v_new]`.
/// - `BlockMoved`: `lineage_ids=[l]`, old = new = `[v]`.
/// - `BlockSplit`: `lineage_ids` = new lineages (document order), `old_version_ids=[v_old]`,
///   `new_version_ids` = parts; `meta.from_lineages=[l_old]`.
/// - `BlockMerged`: `lineage_ids=[new]`, `old_version_ids` = merged olds,
///   `new_version_ids=[v]`; `meta.from_lineages` = old lineages.
/// - `BlockDeleted`: `lineage_ids=[l]`, `old_version_ids=[v]`.
/// - `SnapshotCommitted`: `lineage_ids`/`new_version_ids` = the full ordered snapshot,
///   `snapshot_id` set; `meta.frontmatter`, `meta.parent_snapshot`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub seq: u64,
    pub doc_path: String,
    pub kind: EventKind,
    pub lineage_ids: Vec<String>,
    pub old_version_ids: Vec<String>,
    pub new_version_ids: Vec<String>,
    pub snapshot_id: Option<String>,
    pub meta: serde_json::Value,
}

/// Reads all events; a malformed line or a non-contiguous `seq` is an error.
pub fn read_events(path: &Path) -> Result<Vec<Event>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line?;
        let ev: Event =
            serde_json::from_str(&line).with_context(|| format!("events.jsonl line {}", i + 1))?;
        if ev.seq != i as u64 {
            bail!("events.jsonl line {}: seq {} != {}", i + 1, ev.seq, i);
        }
        out.push(ev);
    }
    Ok(out)
}

/// Appends events as JSON lines; never rewrites existing bytes.
pub fn append_events(path: &Path, events: &[Event]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let mut buf = Vec::new();
    for ev in events {
        serde_json::to_writer(&mut buf, ev)?;
        buf.push(b'\n');
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(&buf)?;
    f.sync_data()?;
    Ok(())
}
