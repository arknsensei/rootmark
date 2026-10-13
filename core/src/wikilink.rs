//! Wikilink extraction (SPEC §1.3).

use crate::model::WikiLink;
use regex::Regex;
use std::sync::OnceLock;

fn link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\[\[([^\[\]|#]*)(?:#([^\[\]|]*))?(?:\|([^\[\]]*))?\]\]").expect("valid regex")
    })
}

/// `[[t]]`, `[[t|label]]`, `[[t#h]]`, `[[t#h|label]]`. The target is the file
/// name: path-independent, without `.md`, trimmed. Empty targets are skipped.
pub fn extract_wikilinks(text: &str) -> Vec<WikiLink> {
    link_re()
        .captures_iter(text)
        .filter_map(|c| {
            let raw = c.get(1).map_or("", |m| m.as_str()).trim();
            let name = raw.rsplit('/').next().unwrap_or(raw);
            let name = name.strip_suffix(".md").unwrap_or(name).trim();
            if name.is_empty() {
                return None;
            }
            let heading = c
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .filter(|h| !h.is_empty());
            Some(WikiLink {
                target: name.to_string(),
                heading,
            })
        })
        .collect()
}
