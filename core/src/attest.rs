//! Attested writes (spec §7.1, §8; 2.0.0, P2-0).
//!
//! - The attestation log `.rootmark/attestations.jsonl`: append-only lines
//!   `{attestation_id, doc_path, content_hash, at_seq, saved_at_seq?}` with
//!   `attestation_id = H(doc_path + "\n" + content_hash + "\n" + at_seq)`.
//!   No wall-clock time is written (spec §7.1).
//! - The window matcher (`Window`): which external change an attestation
//!   vouches for. It works on times the caller gives (milliseconds of a
//!   monotonic clock of its choice), so it is a pure function of its inputs:
//!   the server feeds it its clock, conformance (C2-12) explicit times.

use crate::model::hash32;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Reference default of the window `W` (spec §7.1), milliseconds.
pub const DEFAULT_WINDOW_MS: u64 = 2_000;

/// One line of `attestations.jsonl` (spec §8, 2.0.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attestation {
    pub attestation_id: String,
    pub doc_path: String,
    pub content_hash: String,
    pub at_seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saved_at_seq: Option<u64>,
}

/// `attestation_id = H(doc_path + "\n" + content_hash + "\n" + at_seq)`,
/// `at_seq` in decimal.
pub fn attestation_id(doc_path: &str, content_hash: &str, at_seq: u64) -> String {
    hash32(&format!("{doc_path}\n{content_hash}\n{at_seq}"))
}

/// `content_hash` of file bytes: the first 32 lowercase hex characters of
/// their BLAKE3 hash (spec §8).
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex()[..32].to_string()
}

fn is_hash32(s: &str) -> bool {
    s.len() == 32 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A vault-relative Markdown path as the log names documents: not empty,
/// `/`-separated, no empty, `.` or `..` component, no leading `/` or `\`,
/// no hidden component, ending in `.md`.
pub fn check_doc_path(p: &str) -> Result<()> {
    let ok = !p.is_empty()
        && p.ends_with(".md")
        && !p.contains('\\')
        && !p.contains('\n')
        && p.split('/')
            .all(|c| !c.is_empty() && c != "." && c != ".." && !c.starts_with('.'));
    if !ok {
        bail!("doc_path {p:?} is not a vault-relative Markdown path");
    }
    Ok(())
}

impl Attestation {
    /// A new attestation of `content_hash` for `doc_path`, recorded when the
    /// log's next event has `seq = at_seq`.
    pub fn new(
        doc_path: &str,
        content_hash: &str,
        at_seq: u64,
        saved_at_seq: Option<u64>,
    ) -> Result<Attestation> {
        let a = Attestation {
            attestation_id: attestation_id(doc_path, content_hash, at_seq),
            doc_path: doc_path.to_string(),
            content_hash: content_hash.to_string(),
            at_seq,
            saved_at_seq,
        };
        a.check(at_seq)?;
        Ok(a)
    }

    /// Spec §8: the id formula, a 32-hex `content_hash`, a Markdown
    /// `doc_path`, `at_seq` not beyond `next_seq` (the seq the log's next
    /// event takes).
    pub fn check(&self, next_seq: u64) -> Result<()> {
        check_doc_path(&self.doc_path)?;
        if !is_hash32(&self.content_hash) {
            bail!(
                "attestation {}: content_hash {:?} is not 32 lowercase hexadecimal characters",
                self.attestation_id,
                self.content_hash
            );
        }
        if self.at_seq > next_seq {
            bail!(
                "attestation {}: at_seq {} is beyond the log (next seq {next_seq})",
                self.attestation_id,
                self.at_seq
            );
        }
        let id = attestation_id(&self.doc_path, &self.content_hash, self.at_seq);
        if self.attestation_id != id {
            bail!(
                "attestation {}: id does not match its fields (expected {id})",
                self.attestation_id
            );
        }
        Ok(())
    }
}

/// Reads and checks `attestations.jsonl` (spec §8): every line valid, no id
/// twice. A missing file is an empty log.
pub fn read_attestations(path: &Path, next_seq: u64) -> Result<Vec<Attestation>> {
    if std::fs::symlink_metadata(path).is_err() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (i, line) in text.lines().enumerate() {
        let a: Attestation = serde_json::from_str(line)
            .with_context(|| format!("{} line {}: malformed attestation", path.display(), i + 1))?;
        a.check(next_seq)
            .with_context(|| format!("{} line {}", path.display(), i + 1))?;
        if !seen.insert(a.attestation_id.clone()) {
            bail!(
                "{} line {}: attestation {} recorded twice",
                path.display(),
                i + 1,
                a.attestation_id
            );
        }
        if let Some(prev) = out.last() {
            let prev: &Attestation = prev;
            if a.at_seq < prev.at_seq {
                bail!(
                    "{} line {}: at_seq {} before the previous line's {} (append-only)",
                    path.display(),
                    i + 1,
                    a.at_seq,
                    prev.at_seq
                );
            }
        }
        out.push(a);
    }
    Ok(out)
}

/// An unattested external change waiting for a late attestation (spec
/// §7.1): the bytes it ingested, when it was observed, and the lineages it
/// left `unverified` — each with the version that change gave it and the
/// layer an attested ingest would have given it (`canonical`, or
/// `proposal` for a proposal it touched).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unattested {
    pub doc_path: String,
    pub content_hash: String,
    pub observed_ms: u64,
    pub lineages: Vec<Raise>,
}

/// One lineage a late attestation may raise: only while it is still at
/// `version_id` (a later change of it is not what was attested).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raise {
    pub lineage_id: String,
    pub version_id: String,
    pub to: crate::model::Layer,
}

/// The window matcher of spec §7.1. Holds, in memory only, the times at
/// which attestations were recorded and unattested changes observed; the
/// logs never see them.
#[derive(Debug, Clone)]
pub struct Window {
    pub window_ms: u64,
    /// recorded attestations: (doc_path, content_hash, attestation_id, ms)
    attested: Vec<(String, String, String, u64)>,
    unattested: Vec<Unattested>,
}

impl Window {
    pub fn new(window_ms: u64) -> Window {
        Window {
            window_ms,
            attested: Vec::new(),
            unattested: Vec::new(),
        }
    }

    fn expire(&mut self, now_ms: u64) {
        let w = self.window_ms;
        self.attested.retain(|a| now_ms.saturating_sub(a.3) <= w);
        self.unattested
            .retain(|u| now_ms.saturating_sub(u.observed_ms) <= w);
    }

    /// An external change of `doc_path` observed at `now_ms` with bytes
    /// hashing to `content_hash`: the id of the newest attestation of the
    /// same `(doc_path, content_hash)` recorded at most `W` before, if any.
    pub fn observe(&mut self, doc_path: &str, content_hash: &str, now_ms: u64) -> Option<String> {
        self.expire(now_ms);
        self.attested
            .iter()
            .rev()
            .find(|a| a.0 == doc_path && a.1 == content_hash && a.3 <= now_ms)
            .map(|a| a.2.clone())
    }

    /// An unattested change was ingested (its `unverified` lineages): a late
    /// attestation within `W` may still raise them.
    pub fn unattested(&mut self, u: Unattested) {
        self.expire(u.observed_ms);
        if !u.lineages.is_empty() {
            self.unattested.push(u);
        }
    }

    /// An attestation recorded at `now_ms`: remembered for later
    /// observations, and the unattested changes of the same
    /// `(doc_path, content_hash)` observed at most `W` before it are
    /// returned (and forgotten) — the late matches, oldest first.
    pub fn attest(&mut self, a: &Attestation, now_ms: u64) -> Vec<Unattested> {
        self.attest_with(a, now_ms, true)
    }

    /// `attest`; with `late = false` (a bulk attestation, `rootmark attest
    /// --all`) the window is left as it is: the attestation raises nothing
    /// and is not remembered for a later observation either — a bulk
    /// attestation reads the files as they are, possibly with an agent's
    /// bytes not observed yet, so it must never vouch for a change
    /// (security reviews of P2-0/Z2). It is only written to the log.
    pub fn attest_with(&mut self, a: &Attestation, now_ms: u64, late: bool) -> Vec<Unattested> {
        if !late {
            return Vec::new();
        }
        self.expire(now_ms);
        let matches = |u: &Unattested| {
            u.doc_path == a.doc_path && u.content_hash == a.content_hash && u.observed_ms <= now_ms
        };
        self.attested.push((
            a.doc_path.clone(),
            a.content_hash.clone(),
            a.attestation_id.clone(),
            now_ms,
        ));
        let (late, keep): (Vec<Unattested>, Vec<Unattested>) = std::mem::take(&mut self.unattested)
            .into_iter()
            .partition(matches);
        self.unattested = keep;
        late
    }

    /// Unattested changes still inside their window (status).
    pub fn pending(&self) -> usize {
        self.unattested.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_formula_and_checks() {
        let a = Attestation::new("notes/a.md", &content_hash(b"x"), 3, Some(2)).unwrap();
        assert_eq!(
            a.attestation_id,
            hash32(&format!("notes/a.md\n{}\n3", a.content_hash))
        );
        assert!(Attestation::new("a.md", "ABC", 0, None).is_err());
        assert!(Attestation::new("../a.md", &content_hash(b"x"), 0, None).is_err());
        assert!(Attestation::new(".rootmark/a.md", &content_hash(b"x"), 0, None).is_err());
        assert!(Attestation::new("a.txt", &content_hash(b"x"), 0, None).is_err());
        assert!(a.check(2).is_err(), "at_seq beyond the log");
        let mut b = a.clone();
        b.at_seq = 2;
        assert!(b.check(9).is_err(), "id no longer matches");
    }

    #[test]
    fn window_before_after_and_outside() {
        let h = content_hash(b"human");
        let mut w = Window::new(2_000);
        let a = Attestation::new("a.md", &h, 0, None).unwrap();
        // before the observation, within W
        assert!(w.attest(&a, 1_000).is_empty());
        assert_eq!(w.observe("a.md", &h, 2_500), Some(a.attestation_id.clone()));
        // another hash, another document: no match
        assert_eq!(w.observe("a.md", &content_hash(b"agent"), 2_500), None);
        assert_eq!(w.observe("b.md", &h, 2_500), None);
        // beyond W
        assert_eq!(w.observe("a.md", &h, 3_001), None);
        // after the observation: a late match within W
        let mut w = Window::new(2_000);
        let u = Unattested {
            doc_path: "a.md".into(),
            content_hash: h.clone(),
            observed_ms: 10_000,
            lineages: vec![Raise {
                lineage_id: "l1".into(),
                version_id: "v1".into(),
                to: crate::model::Layer::Canonical,
            }],
        };
        w.unattested(u.clone());
        assert_eq!(w.pending(), 1);
        assert_eq!(w.attest(&a, 12_000), vec![u.clone()]);
        assert_eq!(w.pending(), 0);
        // a bulk attestation (late = false): no raise, the pending change
        // stays, and it never matches a later observation
        let mut w = Window::new(2_000);
        let p = Unattested {
            doc_path: "a.md".into(),
            content_hash: h.clone(),
            observed_ms: 10_000,
            lineages: Vec::new(),
        };
        w.unattested.push(p);
        assert!(w.attest_with(&a, 10_500, false).is_empty());
        assert_eq!(w.pending(), 1);
        assert_eq!(w.observe("a.md", &h, 10_600), None);
        let mut w = Window::new(2_000);
        assert!(w.attest_with(&a, 10_500, false).is_empty());
        assert_eq!(w.observe("a.md", &h, 10_600), None);
        // late beyond W: nothing
        let mut w = Window::new(2_000);
        w.unattested(u);
        assert!(w.attest(&a, 12_001).is_empty());
    }
}
