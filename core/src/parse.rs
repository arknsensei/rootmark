//! Markdown → block drafts (SPEC §1.2).

use crate::model::{version_id, BlockDraft, BlockType};
use crate::normalize::normalize;
use anyhow::Result;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::ops::Range;

fn options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_STRIKETHROUGH
}

/// Splits a leading `---`-delimited YAML frontmatter from the body.
/// Returns `(frontmatter, body)`; the frontmatter is the raw text between the fences.
pub fn split_frontmatter(text: &str) -> (Option<String>, &str) {
    let first_end = match text.find('\n') {
        Some(i) => i,
        None => return (None, text),
    };
    if text[..first_end].trim_end_matches('\r') != "---" {
        return (None, text);
    }
    let mut pos = first_end + 1;
    while pos <= text.len() {
        let line_end = text[pos..].find('\n').map_or(text.len(), |i| pos + i);
        let line = text[pos..line_end].trim_end_matches('\r');
        if line == "---" || line == "..." {
            let fm = text[first_end + 1..pos].to_string();
            let body_start = (line_end + 1).min(text.len());
            return (Some(fm), &text[body_start..]);
        }
        if line_end >= text.len() {
            break;
        }
        pos = line_end + 1;
    }
    (None, text)
}

/// Heading title derived from the normalized heading text: ATX markers
/// (`## t ##`) or a setext underline (`t ===`) are stripped. Used both by the
/// parser and when `heading_path` is rebuilt from stored blocks.
pub fn heading_title(normalized: &str) -> String {
    let s = normalized.trim();
    if s.starts_with('#') {
        let s = s.trim_start_matches('#').trim();
        let t = s.trim_end_matches('#');
        // A closing sequence must be preceded by a space (CommonMark).
        let s = if t.len() != s.len() && (t.is_empty() || t.ends_with(' ')) {
            t
        } else {
            s
        };
        return s.trim().to_string();
    }
    if let Some((title, underline)) = s.rsplit_once(' ') {
        if !underline.is_empty() && underline.chars().all(|c| c == '=' || c == '-') {
            return title.trim().to_string();
        }
    }
    s.to_string()
}

/// Rebuilds `heading_path` for a block sequence given `(block_type, heading_level, normalized_text)`.
pub fn heading_paths<'a, I>(blocks: I) -> Vec<Vec<String>>
where
    I: IntoIterator<Item = (BlockType, Option<u8>, &'a str)>,
{
    let mut stack: Vec<(u8, String)> = Vec::new();
    let mut out = Vec::new();
    for (bt, level, norm) in blocks {
        if bt == BlockType::Heading {
            let level = level.unwrap_or(1);
            while stack.last().is_some_and(|(l, _)| *l >= level) {
                stack.pop();
            }
            out.push(stack.iter().map(|(_, t)| t.clone()).collect());
            stack.push((level, heading_title(norm)));
        } else {
            out.push(stack.iter().map(|(_, t)| t.clone()).collect());
        }
    }
    out
}

/// Top-level block boundaries: `(type, heading level, byte range in body)`.
fn top_level_ranges(body: &str) -> Vec<(BlockType, Option<u8>, Range<usize>)> {
    let mut out = Vec::new();
    // `stack` mirrors open tags; `capture` marks tags whose range became a block.
    let mut stack: Vec<bool> = Vec::new();
    let mut in_top_list: Vec<bool> = Vec::new();
    for (ev, range) in Parser::new_ext(body, options()).into_offset_iter() {
        match ev {
            Event::Start(tag) => {
                let depth = stack.len();
                let parent_is_top_list = depth == 1 && in_top_list.first() == Some(&true);
                let mut is_list = false;
                if depth == 0 {
                    let kind = match &tag {
                        Tag::Heading { level, .. } => {
                            Some((BlockType::Heading, Some(*level as u8)))
                        }
                        Tag::Paragraph => Some((BlockType::Paragraph, None)),
                        Tag::CodeBlock(_) => Some((BlockType::Code, None)),
                        Tag::Table(_) => Some((BlockType::Table, None)),
                        Tag::BlockQuote(_) => Some((BlockType::Blockquote, None)),
                        Tag::List(_) => {
                            is_list = true;
                            None
                        }
                        _ => Some((BlockType::Other, None)),
                    };
                    if let Some((bt, lvl)) = kind {
                        out.push((bt, lvl, range.clone()));
                    }
                } else if parent_is_top_list && matches!(tag, Tag::Item) {
                    out.push((BlockType::ListItem, None, range.clone()));
                }
                stack.push(true);
                in_top_list.push(is_list);
            }
            Event::End(_) => {
                stack.pop();
                in_top_list.pop();
            }
            Event::Rule if stack.is_empty() => out.push((BlockType::Other, None, range)),
            Event::Html(_) | Event::Text(_) if stack.is_empty() => {
                out.push((BlockType::Other, None, range))
            }
            _ => {}
        }
    }
    out
}

/// One block's place in the document's bytes (P2-B/B1, docs/CLIENT_CONTRACT.md
/// §2.5 `loc`): derived from the bytes, never part of an event, an id or
/// `events_hash`. Lines are 1-based and inclusive, counted in the whole
/// document including the front matter (as `BlockDraft`); bytes are UTF-8
/// offsets into the document, half open, the block's raw text without its
/// trailing line ending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockLoc {
    pub version_id: String,
    pub line_start: u32,
    pub line_end: u32,
    pub byte_start: u64,
    pub byte_end: u64,
}

/// A block before lineage assignment, with its byte range in the document.
struct Item {
    bt: BlockType,
    level: Option<u8>,
    raw: String,
    norm: String,
    line_start: u32,
    line_end: u32,
    byte_start: usize,
}

fn parse_items(text: &str) -> (Vec<Item>, Option<String>) {
    let (frontmatter, body) = split_frontmatter(text);
    let body_off = text.len() - body.len();
    // line number (1-based) of a byte offset in `text`
    let line_of = |off: usize| {
        1 + text.as_bytes()[..off]
            .iter()
            .filter(|&&b| b == b'\n')
            .count() as u32
    };
    let mut items = Vec::new();
    for (bt, level, range) in top_level_ranges(body) {
        let start = range.start;
        let raw = body[range].trim_end_matches(['\n', '\r']).to_string();
        let norm = normalize(&raw, bt == BlockType::Code);
        if norm.trim().is_empty() {
            continue;
        }
        let (line_start, line_end) = (
            line_of(body_off + start),
            line_of(body_off + start + raw.len().max(1) - 1),
        );
        items.push(Item {
            bt,
            level,
            raw,
            norm,
            line_start,
            line_end,
            byte_start: body_off + start,
        });
    }
    (items, frontmatter)
}

/// The blocks of a document with their place in its bytes (P2-B/B1), in
/// document order: the same blocks, in the same order, as `parse_document`.
pub fn locate_blocks(text: &str) -> Vec<BlockLoc> {
    parse_items(text)
        .0
        .into_iter()
        .map(|it| BlockLoc {
            version_id: version_id(it.bt, &it.norm),
            line_start: it.line_start,
            line_end: it.line_end,
            byte_start: it.byte_start as u64,
            byte_end: (it.byte_start + it.raw.len()) as u64,
        })
        .collect()
}

/// Parses a document into block drafts and its raw frontmatter (SPEC §1.2).
pub fn parse_document(text: &str, _doc_path: &str) -> Result<(Vec<BlockDraft>, Option<String>)> {
    let (items, frontmatter) = parse_items(text);
    let paths = heading_paths(items.iter().map(|it| (it.bt, it.level, it.norm.as_str())));
    let drafts = items
        .into_iter()
        .zip(paths)
        .map(|(it, heading_path)| BlockDraft {
            version_id: version_id(it.bt, &it.norm),
            block_type: it.bt,
            text: it.raw,
            normalized_text: it.norm,
            heading_path,
            heading_level: it.level,
            line_start: it.line_start,
            line_end: it.line_end,
        })
        .collect();
    Ok((drafts, frontmatter))
}
