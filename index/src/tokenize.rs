//! Lexical tokenization (GOAL P1-C "Sözcüksel"): lower-case, then the maximal
//! runs of `[a-z0-9]` — the prototype's `re.findall(r"[a-z0-9]+",
//! text.lower())` (`research/rootmark/index.py`), checked against it by
//! `index/tests/lexical.rs` (`research/data/tok_ref.json`). The same function
//! feeds tantivy (`RootmarkTokenizer`), the query and the hot tier.

use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

/// Registered name of `RootmarkTokenizer` in the tantivy index. A data
/// identifier written into every index (manifest `lexical.tokenizer`,
/// tantivy `meta.json`) and compared when one is opened: it kept its value
/// through the rename (K-GH.1, GH-0 ASSUMPTIONS G0-2), so a migrated index
/// is read as it is.
pub const TOKENIZER_NAME: &str = "mdk_lower_alnum";

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

#[derive(Clone, Default)]
pub struct RootmarkTokenizer;

pub struct RootmarkTokenStream {
    tokens: Vec<Token>,
    next: usize,
}

impl Tokenizer for RootmarkTokenizer {
    type TokenStream<'a> = RootmarkTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> RootmarkTokenStream {
        // offsets are not used (no positions, no highlighting)
        let tokens = tokens(text)
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
