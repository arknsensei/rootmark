//! mdkernel-core: block-level lineage, provenance and derived structures over
//! plain Markdown (spec/SPEC.md). Matcher v3 (default) and v2 (flag).

pub mod chunk;
pub mod events;
pub mod files;
pub mod governance;
pub mod lineage;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod store;
pub mod vault;
pub mod wikilink;

pub use chunk::{chunk, Chunk, Strategy};
pub use events::{Event, EventKind};
pub use governance::{
    checkpoint_id, trace_id, Actor, ActorKind, Checkpoint, Decision, Deny, HumanOnly,
    PromotionPolicy, PromotionRequest, Trace, TraceEntry, TransitionKind,
};
pub use lineage::{MatchParams, MatcherVersion};
pub use model::{
    hash32, version_id, AuthorKind, Block, BlockDraft, BlockType, Layer, Provenance, WikiLink,
};
pub use normalize::normalize;
pub use parse::parse_document;
pub use store::BlockObject;
pub use vault::{snapshot_id, BlameLine, IngestResult, ReadBlock, Snapshot, Vault};
pub use wikilink::extract_wikilinks;
