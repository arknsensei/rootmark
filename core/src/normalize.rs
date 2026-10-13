//! Text normalization (SPEC §1.1).

use regex::Regex;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

fn ws_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s+").expect("valid regex"))
}

/// Unicode NFC, `\r\n`/`\r` → `\n`; for non-code text additionally trim and
/// collapse every whitespace run to one space. Emphasis markers are kept.
pub fn normalize(text: &str, is_code: bool) -> String {
    let nfc: String = text.nfc().collect();
    let lf = nfc.replace("\r\n", "\n").replace('\r', "\n");
    if is_code {
        return lf;
    }
    ws_re().replace_all(lf.trim(), " ").into_owned()
}
