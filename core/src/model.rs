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

/// Governance status of a version (spec §7, Draft). Round P1-A: only the
/// default by `author_kind` is assigned; there are no transitions and the
/// layer is not written to the event log (layer events arrive in P1-B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Canonical,
    Proposal,
    Derived,
}

impl Layer {
    /// Default layer on ingest (spec §7): `human` → canonical; `agent`,
    /// `transcript` → proposal; `synthetic` → "as declared" — there is no
    /// declaration mechanism before P1-B, so synthetic content (evaluation
    /// data standing in for human edits) is canonical
    /// (docs/rounds/P1-A/progress/ASSUMPTIONS.md, A2).
    pub fn default_for(kind: AuthorKind) -> Layer {
        match kind {
            AuthorKind::Human | AuthorKind::Synthetic => Layer::Canonical,
            AuthorKind::Agent | AuthorKind::Transcript => Layer::Proposal,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Layer::Canonical => "canonical",
            Layer::Proposal => "proposal",
            Layer::Derived => "derived",
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
    /// Derived from `provenance.author_kind` (`Layer::default_for`); P1-A has
    /// no layer transitions.
    pub layer: Layer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WikiLink {
    pub target: String,
    pub heading: Option<String>,
}
