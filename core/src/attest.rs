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
    /// `true` (P2-0.1 T2): the attestation vouched for only part of an
    /// observed change — the attested blocks — the rest stayed unattested
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial: Option<bool>,
    /// `false` (P2-0.1 T4): when it was recorded, the file held other bytes
    /// and no observed change contained them (informative)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched: Option<bool>,
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
            partial: None,
            matched: None,
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
        if self.partial == Some(false) {
            bail!(
                "attestation {}: partial is written only as true",
                self.attestation_id
            );
        }
        if self.matched == Some(true) {
            bail!(
                "attestation {}: matched is written only as false",
                self.attestation_id
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

fn wikilink_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    // `[[target#heading|label]]` (and the embed `![[…]]`): the target is
    // everything before `#` or `|`
    RE.get_or_init(|| {
        regex::Regex::new(r"\[\[([^\[\]|#]*)((?:#[^\[\]|]*)?(?:\|[^\[\]]*)?)\]\]")
            .expect("valid regex")
    })
}

fn mdlink_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    // `[label](dest "title")` / `![alt](dest)`: the destination's path, before
    // any `#fragment`, `<…>` kept as written
    RE.get_or_init(|| {
        regex::Regex::new(
            r#"\]\(\s*(<[^<>\n#]*|[^\s()<>#]*)((?:#[^\s()>]*)?>?(?:\s+"[^"\n]*")?\s*)\)"#,
        )
        .expect("valid regex")
    })
}

/// One link target of a text: its kind (`w` wikilink, `m` Markdown link)
/// and its target text as written.
type Target = (char, String);

/// `text` with the target of every wikilink and Markdown link replaced by a
/// marker, and the targets in order (P2-0.1 T1).
fn link_skeleton(text: &str) -> (String, Vec<Target>) {
    let mut targets = Vec::new();
    let w = wikilink_re().replace_all(text, |c: &regex::Captures| {
        targets.push(('w', c[1].to_string()));
        format!("[[\u{0}{}]]", &c[2])
    });
    let mut md = Vec::new();
    let m = mdlink_re().replace_all(&w, |c: &regex::Captures| {
        md.push(('m', c[1].to_string()));
        format!("](\u{0}{})", &c[2])
    });
    // wikilinks first, then Markdown links: both lists keep their own order
    targets.extend(md);
    (m.into_owned(), targets)
}

/// The file a link target names, as a `/`-separated path without `.md`:
/// percent-decoded (Markdown links), without `<…>`, `./` and `../`
/// segments.
fn target_file(kind: char, t: &str) -> String {
    let t = t.trim().trim_start_matches('<').trim_end_matches('>');
    let t = if kind == 'm' {
        percent_decode(t)
    } else {
        t.to_string()
    };
    let parts: Vec<&str> = t
        .split('/')
        .filter(|p| !p.is_empty() && *p != "." && *p != "..")
        .collect();
    let t = parts.join("/");
    t.strip_suffix(".md").unwrap_or(&t).trim().to_string()
}

fn percent_decode(s: &str) -> String {
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Does the link target text `t` (kind `w`/`m`) name the note at
/// `doc_path`? Its file path must be `doc_path` without `.md`, or a
/// trailing run of its `/`-separated components (`bee` and `sub/bee` name
/// `sub/bee.md`).
pub fn link_names(kind: char, t: &str, doc_path: &str) -> bool {
    let f = target_file(kind, t);
    let d = doc_path.strip_suffix(".md").unwrap_or(doc_path);
    !f.is_empty() && (f == d || d.ends_with(&format!("/{f}")))
}

/// A link update (P2-0.1 T1, contract §2.6 `link_update_of`): `new` differs
/// from `old` only in the targets of links, and every changed target names
/// the note `renamed` (its path now) — and, when the renamed note's old path
/// `was` is known, its old target named `was` — the text around the links,
/// the number and kinds of links, labels and `#heading` parts are the same.
/// `Ok(n)` with the number of changed targets; `Err` says what else
/// differs.
pub fn link_update_only(old: &str, new: &str, renamed: &str, was: Option<&str>) -> Result<usize> {
    let (so, to) = link_skeleton(old);
    let (sn, tn) = link_skeleton(new);
    if so != sn || to.len() != tn.len() {
        bail!("the text differs outside link targets");
    }
    let mut changed = 0;
    for ((ko, a), (kn, b)) in to.iter().zip(&tn) {
        if ko != kn {
            bail!("a link changed its kind");
        }
        if a == b {
            continue;
        }
        if !link_names(*kn, b, renamed) {
            bail!("the link target {b:?} does not name {renamed}");
        }
        if let Some(w) = was {
            if !link_names(*ko, a, w) {
                bail!("the old link target {a:?} does not name {w}");
            }
        }
        changed += 1;
    }
    Ok(changed)
}

/// One block of the state a link update is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseBlock {
    pub version_id: String,
    pub block_type: crate::model::BlockType,
    pub text: String,
    /// a live (not pruned) `canonical` lineage
    pub canonical: bool,
}

/// The state of a document a link update is checked against (P2-0.1 T1):
/// its blocks and front matter, either the head now or the head before an
/// unattested change was ingested (a late link update must be checked
/// against what preceded the change it would raise, Y1 review 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkBase {
    pub doc_path: String,
    pub frontmatter: Option<String>,
    pub blocks: Vec<BaseBlock>,
}

impl LinkBase {
    /// The head of `doc` in `v` (`None` for a document the log does not know).
    pub fn head(v: &crate::Vault, doc: &str) -> Result<Option<LinkBase>> {
        let Some(head) = v.snapshot(doc)? else {
            return Ok(None);
        };
        let st = v
            .doc_state(doc)
            .with_context(|| format!("{doc}: no document state"))?;
        let blocks = v
            .snapshot_blocks(&head)?
            .into_iter()
            .map(|b| BaseBlock {
                canonical: st.layer.get(&b.lineage_id) == Some(&crate::Layer::Canonical)
                    && !st.inactive.contains(&b.lineage_id),
                version_id: b.version_id,
                block_type: b.block_type,
                text: b.text,
            })
            .collect();
        Ok(Some(LinkBase {
            doc_path: doc.to_string(),
            frontmatter: head.frontmatter,
            blocks,
        }))
    }
}

/// A link update of a document (P2-0.1 T1; spec §7.1 "Link update"): the
/// bytes `new_text` are `base` with, at most, the targets of links changed
/// to name the note now at `renamed` (and, when known, its old path `was`).
/// Block for block (same count, order and kinds; the front matter
/// likewise): a block is either the base's version, or differs from it
/// only in link targets (`link_update_only`) and is then a live `canonical`
/// lineage — the other blocks may hold any layer, they stay as they are.
/// Content an agent wrote that is not in the base, or any other change,
/// makes it an error: a rename's link update never vouches for anything
/// but itself. `Ok` with the number of changed link targets.
pub fn check_link_update(
    base: &LinkBase,
    new_text: &str,
    renamed: &str,
    was: Option<&str>,
) -> Result<usize> {
    let (drafts, fm) = crate::parse_document(new_text, &base.doc_path)?;
    if drafts.len() != base.blocks.len() {
        bail!(
            "{} blocks against {} in {}",
            drafts.len(),
            base.blocks.len(),
            base.doc_path
        );
    }
    let mut changed = match (&base.frontmatter, &fm) {
        (a, b) if a == b => 0,
        (Some(a), Some(b)) => {
            link_update_only(a, b, renamed, was).context("the front matter differs")?
        }
        _ => bail!("the front matter differs"),
    };
    for (i, (d, b)) in drafts.iter().zip(&base.blocks).enumerate() {
        if d.version_id == b.version_id {
            continue;
        }
        if d.block_type != b.block_type {
            bail!("block {i} changed its kind");
        }
        changed += link_update_only(&b.text, &d.text, renamed, was)
            .with_context(|| format!("block {i}"))?;
        if !b.canonical {
            bail!("block {i} is not a live canonical lineage");
        }
    }
    Ok(changed)
}

/// The block versions of a document text, in order (P2-0.1 T2).
pub fn block_versions(text: &str, doc_path: &str) -> Result<Vec<String>> {
    Ok(crate::parse_document(text, doc_path)?
        .0
        .into_iter()
        .map(|d| d.version_id)
        .collect())
}

/// `attested` is a non-empty sub-multiset of `observed` (block versions).
pub fn covers(observed: &[String], attested: &[String]) -> bool {
    if attested.is_empty() {
        return false;
    }
    let mut left: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for v in observed {
        *left.entry(v).or_default() += 1;
    }
    attested.iter().all(|v| match left.get_mut(v.as_str()) {
        Some(n) if *n > 0 => {
            *n -= 1;
            true
        }
        _ => false,
    })
}

/// The lineages a late partial match raises (P2-0.1 T2): of the unattested
/// change's `lineages` (each at the version the change gave it), those
/// whose versions are attested blocks the document did not already hold
/// unchanged — `observed` are the observed blocks' versions; the versions
/// the change gave no event (the blocks it left as they were) are taken out
/// of `attested` first, so an agent's copy of an existing human block is
/// never raised. Multiplicities count; order follows `lineages`.
pub fn partial_raises(observed: &[String], attested: &[String], lineages: &[Raise]) -> Vec<Raise> {
    late_partial_raises(observed, attested, lineages).unwrap_or_default()
}

/// `partial_raises`, `None` when the saved bytes cannot precede the change
/// (Y2 review 1 c): they lack a block the change left as it was — so they
/// were saved after it (the human removed something the change kept).
pub fn late_partial_raises(
    observed: &[String],
    attested: &[String],
    lineages: &[Raise],
) -> Option<Vec<Raise>> {
    use std::collections::HashMap;
    let mut changed: HashMap<&str, usize> = HashMap::new();
    for r in lineages {
        *changed.entry(r.version_id.as_str()).or_default() += 1;
    }
    // versions the change left unchanged: observed minus the changed ones
    let mut unchanged: HashMap<&str, usize> = HashMap::new();
    for v in observed {
        let c = changed.entry(v.as_str()).or_default();
        if *c > 0 {
            *c -= 1;
        } else {
            *unchanged.entry(v.as_str()).or_default() += 1;
        }
    }
    let mut want: HashMap<&str, usize> = HashMap::new();
    for v in attested {
        let u = unchanged.entry(v.as_str()).or_default();
        if *u > 0 {
            *u -= 1;
        } else {
            *want.entry(v.as_str()).or_default() += 1;
        }
    }
    if unchanged.values().any(|n| *n > 0) {
        return None;
    }
    let raised = lineages
        .iter()
        .filter(|r| match want.get_mut(r.version_id.as_str()) {
            Some(n) if *n > 0 => {
                *n -= 1;
                true
            }
            _ => false,
        })
        .cloned()
        .collect();
    Some(raised)
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
    /// recorded attestations: (doc_path, content_hash, attestation_id, ms,
    /// the saved bytes as text when the client sent them — memory only)
    attested: Vec<(String, String, String, u64, Option<String>)>,
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
    ///
    /// An attestation matches at most one change: it is consumed when it
    /// matches (P2-0.1 Y1 security review B1: bytes written again within
    /// `W` — after the agent removed and restored its block — must not
    /// match again).
    pub fn observe(&mut self, doc_path: &str, content_hash: &str, now_ms: u64) -> Option<String> {
        self.expire(now_ms);
        let i = self
            .attested
            .iter()
            .rposition(|a| a.0 == doc_path && a.1 == content_hash && a.3 <= now_ms)?;
        Some(self.attested.remove(i).2)
    }

    /// An external change of `doc_path` observed at `now_ms` with bytes
    /// `observed` that no attestation matches exactly (P2-0.1 T2, spec
    /// §7.1 "Partial match"): the newest attestation of the document
    /// recorded at most `W` before that carried its saved bytes, whose blocks
    /// are a non-empty sub-multiset (by version) of the observed blocks —
    /// `(attestation_id, saved text)`, consumed.
    pub fn observe_partial(
        &mut self,
        doc_path: &str,
        observed: &str,
        now_ms: u64,
    ) -> Result<Option<(String, String)>> {
        self.expire(now_ms);
        let obs = block_versions(observed, doc_path)?;
        let mut hit = None;
        for (i, a) in self.attested.iter().enumerate().rev() {
            let (Some(text), true) = (&a.4, a.0 == doc_path && a.3 <= now_ms) else {
                continue;
            };
            if covers(&obs, &block_versions(text, doc_path)?) {
                hit = Some(i);
                break;
            }
        }
        Ok(hit.map(|i| {
            let a = self.attested.remove(i);
            (a.2, a.4.unwrap_or_default())
        }))
    }

    /// Takes the unattested change of `doc_path` with bytes `content_hash`
    /// observed at `observed_ms` out of the window (a late partial match
    /// the caller decided, P2-0.1 T2).
    pub fn take_unattested(
        &mut self,
        doc_path: &str,
        content_hash: &str,
        observed_ms: u64,
    ) -> Option<Unattested> {
        let i = self.unattested.iter().position(|u| {
            u.doc_path == doc_path && u.content_hash == content_hash && u.observed_ms == observed_ms
        })?;
        Some(self.unattested.remove(i))
    }

    /// The lineages of the unattested change of `doc_path` with bytes
    /// `content_hash` observed at `observed_ms`, if it is still in the
    /// window (read only).
    pub fn pending_lineages(
        &self,
        doc_path: &str,
        content_hash: &str,
        observed_ms: u64,
    ) -> Option<Vec<Raise>> {
        self.unattested
            .iter()
            .find(|u| {
                u.doc_path == doc_path
                    && u.content_hash == content_hash
                    && u.observed_ms == observed_ms
            })
            .map(|u| u.lineages.clone())
    }

    /// Forgets a recorded attestation (it matched partially, late).
    pub fn forget(&mut self, attestation_id: &str) {
        self.attested.retain(|a| a.2 != attestation_id);
    }

    /// An unattested change was ingested (its `unverified` lineages): a late
    /// attestation within `W` may still raise them.
    ///
    /// The document's attestations recorded before it no longer match a
    /// later observation (P2-0.1 Y1 security review B1): bytes observed
    /// after a change nobody vouched for are not vouched for by an earlier
    /// statement, even when they are the same bytes again.
    pub fn unattested(&mut self, u: Unattested) {
        self.expire(u.observed_ms);
        self.attested
            .retain(|a| !(a.0 == u.doc_path && a.3 <= u.observed_ms));
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
        self.attest_content(a, now_ms, late, None).0
    }

    /// `attest_with`, remembering the saved bytes `content` (as text) for a
    /// partial match (P2-0.1 T2; memory only); with whether the late match
    /// was ambiguous (then it is no partial match either, Y2 review 3).
    pub fn attest_content(
        &mut self,
        a: &Attestation,
        now_ms: u64,
        late: bool,
        content: Option<String>,
    ) -> (Vec<Unattested>, bool) {
        if !late {
            return (Vec::new(), false);
        }
        self.expire(now_ms);
        let matches = |u: &Unattested| {
            u.doc_path == a.doc_path && u.content_hash == a.content_hash && u.observed_ms <= now_ms
        };
        let (mut late, keep): (Vec<Unattested>, Vec<Unattested>) =
            std::mem::take(&mut self.unattested)
                .into_iter()
                .partition(matches);
        self.unattested = keep;
        // one match per attestation: when the same bytes were observed
        // unattested more than once in the window, which change the
        // statement vouches for is ambiguous, so it raises none of them
        // (they are dropped; Y1 security review B5)
        if late.len() > 1 {
            late.clear();
            return (late, true);
        }
        // an attestation that matched late is consumed (Y1 security review)
        if late.is_empty() {
            self.attested.push((
                a.doc_path.clone(),
                a.content_hash.clone(),
                a.attestation_id.clone(),
                now_ms,
                content,
            ));
        }
        (late, false)
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
        // P2-0.1: `partial` only as true, `matched` only as false
        let mut p = a.clone();
        p.partial = Some(false);
        assert!(p.check(3).is_err());
        p.partial = Some(true);
        p.check(3).unwrap();
        p.matched = Some(true);
        assert!(p.check(3).is_err());
        p.matched = Some(false);
        p.check(3).unwrap();
        let mut b = a.clone();
        b.at_seq = 2;
        assert!(b.check(9).is_err(), "id no longer matches");
    }

    #[test]
    fn link_update_only_targets() {
        // Obsidian's rewrite of `b.md` → `sub/bee.md` (measured, P2-0.1/Y1)
        let old = "See [[b]] and [[b|bee]] and [md](b.md) and ![[b#B]].";
        let new = "See [[bee]] and [[bee|bee]] and [md](bee.md) and ![[sub/bee#B]].";
        assert_eq!(link_update_only(old, new, "sub/bee.md", None).unwrap(), 4);
        assert_eq!(link_update_only(old, old, "sub/bee.md", None).unwrap(), 0);
        let ok = |o: &str, n: &str| link_update_only(o, n, "sub/bee.md", Some("b.md")).is_ok();
        // percent-encoded and angle-bracket destinations, a title, a fragment
        assert!(ok("[x](b.md \"t\")", "[x](sub/bee.md \"t\")"));
        assert!(ok("[x](<b.md>)", "[x](<sub/bee.md>)"));
        assert!(ok("[x](b.md#h)", "[x](../sub/bee.md#h)"));
        assert!(link_update_only("[x](b.md)", "[x](my%20bee.md)", "my bee.md", None).is_ok());
        // the old target must name the renamed note's old path, when known
        assert!(
            !ok("See [[c]].", "See [[bee]]."),
            "a link to another note redirected"
        );
        assert!(link_update_only("See [[c]].", "See [[bee]].", "sub/bee.md", None).is_ok());
        // anything else is not a link update
        assert!(!ok("See [[b]].", "See [[bee]]!"), "text outside the link");
        assert!(!ok("See [[b|x]].", "See [[bee|y]]."), "a label");
        assert!(!ok("See [[b#A]].", "See [[bee#B]]."), "a heading part");
        assert!(!ok("See [[b]].", "See [[bee]] [[bee]]."), "a new link");
        assert!(
            !ok("See [[b]].", "See [[other]]."),
            "a target naming another note"
        );
        assert!(
            !ok("See [[b]].", "See [md](bee.md)."),
            "another kind of link"
        );
        assert!(
            !ok("[[b]]", "[[bee]]\n\nAgent text."),
            "an appended paragraph"
        );
        assert!(
            !ok("See [[b]].", "See [[ee]]."),
            "a suffix inside a component"
        );
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
        // consumed at the first match; voided by a later unattested change
        let mut w = Window::new(2_000);
        assert!(w.attest(&a, 1_000).is_empty());
        assert_eq!(w.observe("a.md", &h, 1_100), Some(a.attestation_id.clone()));
        assert_eq!(w.observe("a.md", &h, 1_200), None, "consumed");
        assert!(w.attest(&a, 2_000).is_empty());
        w.unattested(Unattested {
            doc_path: "a.md".into(),
            content_hash: content_hash(b"agent"),
            observed_ms: 2_100,
            lineages: Vec::new(),
        });
        assert_eq!(
            w.observe("a.md", &h, 2_200),
            None,
            "voided by the unattested change"
        );
        // the same bytes observed unattested twice: a late attestation is
        // ambiguous and raises neither
        let mut w = Window::new(2_000);
        w.unattested(u.clone());
        w.unattested(Unattested {
            observed_ms: 10_500,
            ..u.clone()
        });
        assert!(w.attest(&a, 11_000).is_empty());
        assert_eq!(w.pending(), 0);
        // late beyond W: nothing
        let mut w = Window::new(2_000);
        w.unattested(u);
        assert!(w.attest(&a, 12_001).is_empty());
    }
}
