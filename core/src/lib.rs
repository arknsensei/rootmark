//! mdkernel-core: markdown-native versioned block kernel (v2).

pub mod model;
pub mod normalize;
pub mod parse;
pub mod wikilink;

pub use model::{
    hash32, version_id, AuthorKind, Block, BlockDraft, BlockType, Provenance, WikiLink,
};
pub use normalize::normalize;
pub use parse::parse_document;
pub use wikilink::extract_wikilinks;
