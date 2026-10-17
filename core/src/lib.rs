//! rootmark-core: block-level lineage, provenance and derived structures over
//! plain Markdown (spec/SPEC.md). Matcher v3 (default) and v2 (flag).

pub mod chunk;
pub mod events;
pub mod files;
pub mod governance;
pub mod journal;
pub mod lineage;
pub mod meta;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod policy;
pub mod store;
pub mod vault;
pub mod wikilink;

pub use chunk::{chunk, Chunk, Strategy};
pub use events::{Event, EventKind};
pub use governance::{
    check_trace_input, checkpoint_id, trace_id, Actor, ActorKind, AppendOnly, Checkpoint, Decision,
    Deny, EditEffect, EditMode, EditPolicy, EditRequest, HumanOnly, PromotionPolicy,
    PromotionRequest, TouchedLineage, Trace, TraceEntry, TransitionKind, Unrestricted,
};
pub use journal::{
    IdleOrDistance, JournalRecord, JournalRetention, PruneReport, TriggerInput, VersionTrigger,
    MIN_RETENTION_HOURS,
};
pub use lineage::{MatchParams, MatcherVersion};
pub use model::{
    hash32, version_id, AuthorKind, Block, BlockDraft, BlockType, Layer, LayerSource, Provenance,
    WikiLink,
};
pub use normalize::normalize;
pub use parse::parse_document;
pub use policy::{Edit, Policy, PolicySource, Promotion};
pub use store::BlockObject;
pub use vault::{snapshot_id, BlameLine, IngestPlan, IngestResult, ReadBlock, Snapshot, Vault};
pub use wikilink::extract_wikilinks;
