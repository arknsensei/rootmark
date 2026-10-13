//! Governance types (spec §8, Draft). Round P1-A defines the shapes only:
//! no constructor, id formula, store or log is implemented here (P1-B).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Immutable vault-wide map `doc_path → snapshot_id` at an event-log offset
/// (spec §1, §8). `checkpoint_id` follows spec §3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub checkpoint_id: String,
    pub name: Option<String>,
    pub event_seq: u64,
    pub docs: BTreeMap<String, String>,
}

/// One block version a consumer received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEntry {
    pub doc_path: String,
    pub lineage_id: String,
    pub version_id: String,
}

/// Consumption trace line (spec §8): which block versions a consumer
/// received for a query at a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub checkpoint_id: String,
    pub consumer: String,
    pub query_hash: String,
    pub returned: Vec<TraceEntry>,
    pub at_seq: u64,
}
