//! Lexical tokenization (GOAL P1-C "Sözcüksel"): lower-case, then the maximal
//! runs of `[a-z0-9]` — the prototype's `re.findall(r"[a-z0-9]+",
//! text.lower())` (`research/rootmark/index.py`), checked against it by
//! `index/tests/lexical.rs` (`research/data/tok_ref.json`). The same function
//! feeds tantivy (`RootmarkTokenizer`), the query and the hot tier.
//!
//! P2-L/L1 (GOAL P2-L 2): a second variant, `uni_fold_tr`, chosen per vault
//! (`[index] tokenizer`, `crate::config`) and declared in the index manifest
//! (`lexical.tokenizer`, K-C1.1): the text in Unicode NFC, Turkish case
//! folding (`İ` → `i`, `I` → `ı`, then Unicode lower case), then the maximal
//! runs of characters that are Unicode `Alphabetic` or `Numeric`. The first
//! variant, `mdk_lower_alnum`, is unchanged and stays the default.

use tantivy::tokenizer::{Token, TokenStream, Tokenizer};
use unicode_normalization::UnicodeNormalization;

/// Registered name of `RootmarkTokenizer` in the tantivy index. A data
/// identifier written into every index (manifest `lexical.tokenizer`,
/// tantivy `meta.json`) and compared when one is opened: it kept its value
/// through the rename (K-GH.1, GH-0 ASSUMPTIONS G0-2), so a migrated index
/// is read as it is.
pub const TOKENIZER_NAME: &str = "mdk_lower_alnum";
/// Registered name of the Unicode, Turkish-folding variant (P2-L/L1).
pub const UNI_FOLD_TR: &str = "uni_fold_tr";

/// A lexical variant: the tokenizer an index is built and queried with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    /// `mdk_lower_alnum`: lower case, maximal runs of `[a-z0-9]` (`tokens`)
    #[default]
    LowerAlnum,
    /// `uni_fold_tr`: NFC, Turkish folding, `Alphabetic`/`Numeric` runs
    /// (`uni_fold_tr`)
    UniFoldTr,
}

impl Variant {
    pub const ALL: [Variant; 2] = [Variant::LowerAlnum, Variant::UniFoldTr];

    /// Its registered name (manifest `lexical.tokenizer`, `[index] tokenizer`).
    pub fn name(self) -> &'static str {
        match self {
            Variant::LowerAlnum => TOKENIZER_NAME,
            Variant::UniFoldTr => UNI_FOLD_TR,
        }
    }

    /// The variant of a registered name; `None` for any other.
    pub fn of_name(name: &str) -> Option<Variant> {
        Variant::ALL.into_iter().find(|v| v.name() == name)
    }

    pub fn tokens(self, text: &str) -> Vec<String> {
        match self {
            Variant::LowerAlnum => tokens(text),
            Variant::UniFoldTr => uni_fold_tr(text),
        }
    }
}

/// `uni_fold_tr` (P2-L/L1): NFC, then `İ` → `i` and `I` → `ı` (Turkish case
/// folding: the dotted capital to the dotted small letter, the dotless
/// capital to the dotless small letter), then every other character to its
/// Unicode lower case (`char::to_lowercase`), then the maximal runs of
/// characters that are `Alphabetic` or `Numeric` (`char::is_alphanumeric`).
/// Lower-casing never splits a run here: `İ`, the one character whose lower
/// case adds a combining mark, is folded before.
pub fn uni_fold_tr(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.nfc() {
        let mut push = |l: char| {
            if l.is_alphanumeric() {
                cur.push(l);
            } else if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        };
        match c {
            '\u{130}' => push('i'),
            'I' => push('\u{131}'),
            _ => c.to_lowercase().for_each(&mut push),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The tantivy tokenizer of `mdk_lower_alnum` (`tokens`).
#[derive(Clone, Default)]
pub struct RootmarkTokenizer;

/// The tantivy tokenizer of a variant (registered under its name; P2-L/L1).
#[derive(Clone, Default)]
pub struct VariantTokenizer(pub Variant);

fn stream(tokens: Vec<String>) -> RootmarkTokenStream {
    // offsets are not used (no positions, no highlighting)
    let tokens = tokens
        .into_iter()
        .enumerate()
        .map(|(position, text)| Token {
            offset_from: 0,
            offset_to: 0,
            position,
            text,
            position_length: 1,
        })
        .collect();
    RootmarkTokenStream { tokens, next: 0 }
}

impl Tokenizer for VariantTokenizer {
    type TokenStream<'a> = RootmarkTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> RootmarkTokenStream {
        stream(self.0.tokens(text))
    }
}

pub struct RootmarkTokenStream {
    tokens: Vec<Token>,
    next: usize,
}

impl Tokenizer for RootmarkTokenizer {
    type TokenStream<'a> = RootmarkTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> RootmarkTokenStream {
        stream(tokens(text))
    }
}

impl TokenStream for RootmarkTokenStream {
    fn advance(&mut self) -> bool {
        self.next += 1;
        self.next <= self.tokens.len()
    }

    fn token(&self) -> &Token {
        &self.tokens[self.next - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.next - 1]
    }
}
