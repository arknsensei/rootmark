//! Data model (SPEC §1, §2, §3, §6).

use serde::{Deserialize, Serialize};

/// blake3 of `data`, first 32 hex characters (SPEC: "hex ilk 32 karakter").
pub fn hash32(data: &str) -> String {
    let mut h = blake3::hash(data.as_bytes()).to_hex().to_string();
    h.truncate(32);
    h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockType {
    Heading,
    Paragraph,
    ListItem,
    Code,
    Table,
    Blockquote,
    Other,
}

impl BlockType {
    pub fn as_str(self) -> &'static str {
        match self {
            BlockType::Heading => "Heading",
            BlockType::Paragraph => "Paragraph",
            BlockType::ListItem => "ListItem",
            BlockType::Code => "Code",
            BlockType::Table => "Table",
            BlockType::Blockquote => "Blockquote",
            BlockType::Other => "Other",
        }
    }
}

/// `version_id = blake3(block_type + "\n" + normalized_text)` (SPEC §1).
pub fn version_id(block_type: BlockType, normalized_text: &str) -> String {
    hash32(&format!("{}\n{}", block_type.as_str(), normalized_text))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorKind {
    Human,
    Agent,
    Transcript,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub author_kind: AuthorKind,
    pub source: String,
}

impl Default for Provenance {
    fn default() -> Self {
        Provenance {
            author_kind: AuthorKind::Human,
            source: String::new(),
        }
    }
}

/// Parser output: a block before lineage assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockDraft {
    pub version_id: String,
    pub block_type: BlockType,
    pub text: String,
    pub normalized_text: String,
    pub heading_path: Vec<String>,
    pub heading_level: Option<u8>,
    /// first and last source line (1-based, inclusive, counted in the whole
    /// document including frontmatter)
    pub line_start: u32,
    pub line_end: u32,
}

/// A block occurrence with its lineage (SPEC §1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub version_id: String,
    pub lineage_id: String,
    pub block_type: BlockType,
    pub text: String,
    pub normalized_text: String,
    pub heading_path: Vec<String>,
    pub heading_level: Option<u8>,
    pub parents: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WikiLink {
    pub target: String,
    pub heading: Option<String>,
}
