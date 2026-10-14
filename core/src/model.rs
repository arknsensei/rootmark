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

/// Governance status of a version (spec §7). Ordered by trust:
/// `Derived < Proposal < Canonical`. Ingest never raises a lineage's layer
/// (only a `Promoted` event does); see `Vault` for the transition rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Derived,
    Proposal,
    Canonical,
}

impl Layer {
    /// Default layer on ingest (spec §7): `human` → canonical; `agent`,
    /// `transcript` → proposal; `synthetic` → as declared, canonical when
    /// undeclared (spec 1.0.1, K-B1.4; a declaration is `meta.layer`).
    pub fn default_for(kind: AuthorKind) -> Layer {
        match kind {
            AuthorKind::Human | AuthorKind::Synthetic => Layer::Canonical,
            AuthorKind::Agent | AuthorKind::Transcript => Layer::Proposal,
        }
    }

    pub fn parse(s: &str) -> anyhow::Result<Layer> {
        Ok(match s {
            "canonical" => Layer::Canonical,
            "proposal" => Layer::Proposal,
            "derived" => Layer::Derived,
            _ => anyhow::bail!("unknown layer {s:?} (canonical|proposal|derived)"),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Layer::Canonical => "canonical",
            Layer::Proposal => "proposal",
            Layer::Derived => "derived",
        }
    }
}

/// Where a lineage's current layer comes from (`blame`; K-B1.1): `event` —
/// an event in the log set it (the block event's author, `LayerChanged`,
/// `Promoted`, `Demoted`); `derived` — replay applied a drop the layer rule
/// implies but a v3-final-format log (no layer information) does not record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerSource {
    Event,
    Derived,
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
    /// Layer of the lineage at this snapshot (replayed from the event log;
    /// `Layer::default_for(author_kind)` for lineages without layer history).
    pub layer: Layer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WikiLink {
    pub target: String,
    pub heading: Option<String>,
}
