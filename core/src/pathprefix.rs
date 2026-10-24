//! Vault-relative path prefixes (P2-L/L3 search filter, P2-P/P1 promotion
//! rules): one normalization and one rejection rule for both.
//!
//! A prefix is NFC-normalized and a trailing `/` dropped; it must be
//! non-empty (the whole vault is no prefix: at least one path component),
//! at most `MAX_PREFIX_BYTES` bytes, `/`-separated and relative (no leading
//! `/`, no `\`, no drive letter), without control characters, and without
//! empty, `.` or `..` components. A document matches a prefix when its path
//! is the prefix or lies under it (`<prefix>/…`).

use anyhow::{bail, Result};

pub const MAX_PREFIX_BYTES: usize = 1024;

/// The normalized prefix of `raw`, or an error naming it
/// (`path_prefix "<raw>": …`).
pub fn normalize(raw: &str) -> Result<String> {
    use unicode_normalization::UnicodeNormalization;
    let p: String = raw.nfc().collect();
    let p = p.strip_suffix('/').unwrap_or(&p).to_string();
    if p.is_empty() {
        bail!("path_prefix {raw:?}: empty");
    }
    if p.len() > MAX_PREFIX_BYTES {
        bail!(
            "path_prefix: a prefix of {} bytes (at most {MAX_PREFIX_BYTES})",
            p.len()
        );
    }
    if p.starts_with('/') || p.contains('\\') || p.chars().nth(1) == Some(':') {
        bail!("path_prefix {raw:?}: not a vault-relative path ('/' separated, not absolute)");
    }
    if p.chars().any(char::is_control) {
        bail!("path_prefix {raw:?}: a control character");
    }
    if p.split('/').any(|c| c.is_empty() || c == "." || c == "..") {
        bail!("path_prefix {raw:?}: an empty, '.' or '..' component");
    }
    Ok(p)
}

/// Whether `doc_path` is the (normalized) `prefix` or lies under it.
pub fn matches(prefix: &str, doc_path: &str) -> bool {
    doc_path == prefix
        || (doc_path.len() > prefix.len()
            && doc_path.starts_with(prefix)
            && doc_path.as_bytes()[prefix.len()] == b'/')
}
