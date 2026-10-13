//! Derived chunks (SPEC §6). Chunking never touches the log or lineages.

use crate::model::{hash32, Block, BlockType, WikiLink};
use crate::vault::{Snapshot, Vault};
use crate::wikilink::extract_wikilinks;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiktoken_rs::CoreBPE;

pub const MAX_TOKENS: usize = 512;
pub const FIXED_OVERLAP: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    Fixed,
    Structural,
}

impl Strategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Strategy::Fixed => "fixed",
            Strategy::Structural => "structural",
        }
    }

    pub fn parse(s: &str) -> Result<Strategy> {
        match s {
            "fixed" => Ok(Strategy::Fixed),
            "structural" => Ok(Strategy::Structural),
            _ => bail!("unknown chunk strategy {s:?} (fixed|structural)"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub doc_path: String,
    pub lineage_ids: Vec<String>,
    pub version_ids: Vec<String>,
    pub heading_path: Vec<String>,
    pub text: String,
    pub wikilinks: Vec<WikiLink>,
    pub strategy: String,
    pub token_count: u32,
}

fn bpe() -> &'static CoreBPE {
    tiktoken_rs::cl100k_base_singleton()
}

/// cl100k_base token count, ordinary encoding (no special tokens); equals
/// Python `tiktoken.get_encoding("cl100k_base").encode_ordinary(text)`.
pub fn count_tokens(text: &str) -> u32 {
    bpe().encode_ordinary(text).len() as u32
}

/// `chunk_id = blake3(version_ids joined by "\n")`. A repeat of the same
/// version list inside one document gets `"\n#<k>"` appended (k ≥ 1) so that
/// `(doc_path, chunk_id)` stays a unique index key.
fn assign_ids(chunks: &mut [Chunk]) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for c in chunks.iter_mut() {
        let base = c.version_ids.join("\n");
        let k = seen.entry(base.clone()).or_insert(0);
        c.chunk_id = if *k == 0 {
            hash32(&base)
        } else {
            hash32(&format!("{base}\n#{k}"))
        };
        *k += 1;
    }
}

fn links(blocks: &[&Block]) -> Vec<WikiLink> {
    blocks
        .iter()
        .flat_map(|b| extract_wikilinks(&b.normalized_text))
        .collect()
}

fn dedup_keep_order(v: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    v.into_iter().filter(|x| seen.insert(x.clone())).collect()
}

/// Section key of a block: a heading opens its own section, so it is grouped
/// with the blocks under it rather than with the preceding section.
fn section_of(b: &Block) -> Vec<String> {
    let mut p = b.heading_path.clone();
    if b.block_type == BlockType::Heading {
        p.push(crate::parse::heading_title(&b.normalized_text));
    }
    p
}

fn structural_text(path: &[String], blocks: &[&Block]) -> String {
    let body = blocks
        .iter()
        .map(|b| b.normalized_text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    if path.is_empty() {
        body
    } else {
        format!("{}\n\n{}", path.join(" > "), body)
    }
}

fn make_chunk(
    doc: &str,
    strategy: Strategy,
    path: Vec<String>,
    blocks: &[&Block],
    text: String,
) -> Chunk {
    Chunk {
        chunk_id: String::new(),
        doc_path: doc.to_string(),
        lineage_ids: dedup_keep_order(blocks.iter().map(|b| b.lineage_id.clone())),
        version_ids: blocks.iter().map(|b| b.version_id.clone()).collect(),
        heading_path: path,
        token_count: count_tokens(&text),
        text,
        wikilinks: links(blocks),
        strategy: strategy.as_str().to_string(),
    }
}

/// Consecutive blocks of one section (every heading opens a new chunk); a
/// block that would push the chunk over
/// 512 tokens starts a new chunk; a single block over 512 is its own chunk.
pub fn structural(doc: &str, blocks: &[Block]) -> Vec<Chunk> {
    let mut out = Vec::new();
    let mut cur: Vec<&Block> = Vec::new();
    let mut cur_path: Vec<String> = Vec::new();
    for b in blocks {
        let path = section_of(b);
        if !cur.is_empty() {
            let fits = {
                let mut trial = cur.clone();
                trial.push(b);
                count_tokens(&structural_text(&cur_path, &trial)) as usize <= MAX_TOKENS
            };
            if b.block_type == BlockType::Heading || path != cur_path || !fits {
                let text = structural_text(&cur_path, &cur);
                out.push(make_chunk(
                    doc,
                    Strategy::Structural,
                    cur_path.clone(),
                    &cur,
                    text,
                ));
                cur.clear();
            }
        }
        if cur.is_empty() {
            cur_path = path;
        }
        cur.push(b);
    }
    if !cur.is_empty() {
        let text = structural_text(&cur_path, &cur);
        out.push(make_chunk(doc, Strategy::Structural, cur_path, &cur, text));
    }
    assign_ids(&mut out);
    out
}

/// Whole-document text in 512-token windows with 64 overlap, ignoring block
/// boundaries; `lineage_ids` are the blocks that overlap the window.
pub fn fixed(doc: &str, blocks: &[Block]) -> Vec<Chunk> {
    let mut tokens: Vec<u32> = Vec::new();
    let mut owner: Vec<usize> = Vec::new(); // block index per token (separators: next block)
    for (i, b) in blocks.iter().enumerate() {
        if i > 0 {
            let sep = bpe().encode_ordinary("\n\n");
            owner.extend(std::iter::repeat_n(i, sep.len()));
            tokens.extend(sep);
        }
        let t = bpe().encode_ordinary(&b.normalized_text);
        owner.extend(std::iter::repeat_n(i, t.len()));
        tokens.extend(t);
    }
    let mut out = Vec::new();
    let step = MAX_TOKENS - FIXED_OVERLAP;
    let mut start = 0;
    while start < tokens.len() {
        let end = (start + MAX_TOKENS).min(tokens.len());
        let bytes = bpe().decode_bytes(&tokens[start..end]).unwrap_or_default();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let (first, last) = (owner[start], owner[end - 1]);
        let win: Vec<&Block> = blocks[first..=last].iter().collect();
        let path = win[0].heading_path.clone();
        out.push(make_chunk(doc, Strategy::Fixed, path, &win, text));
        if end == tokens.len() {
            break;
        }
        start += step;
    }
    assign_ids(&mut out);
    out
}

/// SPEC §6 / §7 `chunk(snapshot, vault, strategy)`.
pub fn chunk(snapshot: &Snapshot, vault: &Vault, strategy: Strategy) -> Result<Vec<Chunk>> {
    let blocks = vault.snapshot_blocks(snapshot)?;
    Ok(match strategy {
        Strategy::Fixed => fixed(&snapshot.doc_path, &blocks),
        Strategy::Structural => structural(&snapshot.doc_path, &blocks),
    })
}
