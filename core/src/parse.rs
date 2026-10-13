//! Markdown → block drafts (SPEC §1.2).

use crate::model::{version_id, BlockDraft, BlockType};
use crate::normalize::normalize;
use anyhow::Result;
use pulldown_cmark::{Event, Options, Parser, Tag};
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

/// Parses a document into block drafts and its raw frontmatter (SPEC §1.2).
pub fn parse_document(text: &str, _doc_path: &str) -> Result<(Vec<BlockDraft>, Option<String>)> {
    let (frontmatter, body) = split_frontmatter(text);
    let mut items: Vec<(BlockType, Option<u8>, String, String)> = Vec::new();
    for (bt, level, range) in top_level_ranges(body) {
        let raw = body[range].trim_end_matches(['\n', '\r']).to_string();
        let norm = normalize(&raw, bt == BlockType::Code);
        if norm.trim().is_empty() {
            continue;
        }
        items.push((bt, level, raw, norm));
    }
    let paths = heading_paths(items.iter().map(|(bt, l, _, n)| (*bt, *l, n.as_str())));
    let drafts = items
        .into_iter()
        .zip(paths)
        .map(|((bt, level, raw, norm), heading_path)| BlockDraft {
            version_id: version_id(bt, &norm),
            block_type: bt,
            text: raw,
            normalized_text: norm,
            heading_path,
            heading_level: level,
        })
        .collect();
    Ok((drafts, frontmatter))
}
