//! mdkernel-core: markdown-native versioned block kernel (v2).

pub mod events;
pub mod lineage;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod store;
pub mod vault;
pub mod wikilink;

pub use events::{Event, EventKind};
pub use lineage::MatchParams;
pub use model::{
    hash32, version_id, AuthorKind, Block, BlockDraft, BlockType, Provenance, WikiLink,
};
pub use normalize::normalize;
pub use parse::parse_document;
pub use store::BlockObject;
pub use vault::{snapshot_id, IngestResult, Snapshot, Vault};
pub use wikilink::extract_wikilinks;
