//! Attested writes on the server side (spec §7.1; GOAL P2-0 "Tasarım").
//!
//! - **Sessions** (stage 1): a human-channel client opens a session over
//!   HTTP with the token (`POST /v1/attest_session`) and receives a
//!   short-lived random key; every attestation it sends
//!   (`POST /v1/attest`) carries `mac = HMAC-SHA256(key, message)` with
//!   `message` the lines `rootmark-attest/1`, `doc_path`, `content_hash`
//!   and `saved_at_seq` (decimal, empty when absent) joined by `\n`
//!   (`mac_message`). A token holder can open a
//!   session: stage 1 validates the design, it does not close the threat of
//!   an agent that reads the token (THREAT_MODEL). Keys live in memory only.
//! - **Matching**: `rootmarkcore::attest::Window` on this process's
//!   monotonic clock (milliseconds since the service started); the window
//!   is `[attestation] window_s`.
//! - **External changes** (the watcher, and `propose`'s sync of an edit made
//!   outside the server): `Attesting::decide` says how the bytes are
//!   ingested; an unattested ingest is remembered for a late attestation.
//!
//! MCP has no attestation tool: an agent's channel can never attest (GOAL
//! P2-0 "Yapılmayacaklar").

use anyhow::{bail, Result};
use rootmarkcore::attest::{Raise, Unattested, Window};
use rootmarkcore::{Actor, ActorKind, Policy};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Lifetime of a session key (stage 1): the client opens a new session when
/// the server answers `denied` for an expired or unknown one.
pub const SESSION_TTL: Duration = Duration::from_secs(12 * 3600);
/// Open sessions kept at most; opening one more drops the oldest (a client
/// whose session was dropped opens a new one on `denied`).
pub const MAX_SESSIONS: usize = 64;

/// The MAC message of one attestation (see the module comment).
pub fn mac_message(doc_path: &str, content_hash: &str, saved_at_seq: Option<u64>) -> String {
    format!(
        "rootmark-attest/1\n{doc_path}\n{content_hash}\n{}",
        saved_at_seq.map(|s| s.to_string()).unwrap_or_default()
    )
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// `HMAC-SHA256(key, message)` as 64 lowercase hex characters.
pub fn mac(key: &[u8], message: &str) -> String {
    hex(&hmac_sha256::HMAC::mac(message.as_bytes(), key))
}

/// Byte comparison whose time does not depend on where the inputs differ.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// How an external change is ingested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingest {
    /// `[attestation] unattested = "canonical"`: as in 1.0.3 (human)
    Canonical,
    /// attested (the attestation's id): human, canonical
    Attested(String),
    /// not attested at its observation: `unverified`, actor `unattested_fs`
    Unverified,
    /// a partial match (P2-0.1 T2): the attestation's id and the bytes it
    /// vouched for (as text); those are ingested first as an attested change,
    /// then the observed bytes as an unattested one
    Partial(String, String),
}

/// The actor of an unattested external change (spec §7.1).
pub fn unattested_actor() -> Actor {
    Actor::new(ActorKind::UnattestedFs, "fs").expect("non-empty id")
}

/// The source an unattested ingest records (`blame`, change events).
pub const UNATTESTED_SOURCE: &str = "unattested_fs";

struct Session {
    key: Vec<u8>,
    expires: Instant,
}

/// The served vault's attestation state (in memory).
pub struct Attesting {
    start: Instant,
    window: Mutex<Window>,
    sessions: Mutex<HashMap<String, Session>>,
    /// accepted rename attestations, newest last: (new path, old path)
    /// (P2-0.1 T1: the base of a link update of a note the log does not
    /// know yet)
    renames: Mutex<Vec<(String, String)>>,
    /// link-update attestations accepted (P2-0.1 T1; `status`)
    link_updates: std::sync::atomic::AtomicU64,
    /// unattested ingests still in their window: (doc, content_hash,
    /// observed ms, the document's head before the ingest) — a late link
    /// update is checked against the state before the change it would
    /// raise (P2-0.1 Y1 review 1)
    /// and the observed blocks' versions (a late partial match, T2)
    bases: Mutex<Vec<Pending>>,
}

/// An unattested ingest in its window, as the server remembers it beside
/// the matcher (P2-0.1): the document's head before it and the observed
/// blocks' versions.
#[derive(Debug, Clone)]
pub struct Pending {
    pub doc: String,
    pub content_hash: String,
    pub observed_ms: u64,
    pub base: rootmarkcore::attest::LinkBase,
    pub observed: Vec<String>,
}

/// Rename attestations remembered at most (the oldest is dropped).
pub const MAX_RENAMES: usize = 256;

impl Default for Attesting {
    fn default() -> Self {
        Self::new()
    }
}

impl Attesting {
    pub fn new() -> Attesting {
        Attesting {
            start: Instant::now(),
            window: Mutex::new(Window::new(rootmarkcore::attest::DEFAULT_WINDOW_MS)),
            sessions: Mutex::new(HashMap::new()),
            renames: Mutex::new(Vec::new()),
            link_updates: std::sync::atomic::AtomicU64::new(0),
            bases: Mutex::new(Vec::new()),
        }
    }

    /// An unattested ingest of `doc` (bytes `content_hash`) observed at
    /// `observed_ms`, and the document's head before it (`None`: a new
    /// document).
    pub fn unattested_base(
        &self,
        policy: &Policy,
        doc: &str,
        content_hash: &str,
        observed_ms: u64,
        base: Option<rootmarkcore::attest::LinkBase>,
        observed: Vec<String>,
    ) -> Result<()> {
        let w = policy.attestation.window_ms;
        let mut b = self
            .bases
            .lock()
            .map_err(|_| anyhow::anyhow!("base lock poisoned"))?;
        b.retain(|x| observed_ms.saturating_sub(x.observed_ms) <= w);
        // a new document: an empty base (every block is new, so no link
        // update of it can raise anything)
        let base = base.unwrap_or(rootmarkcore::attest::LinkBase {
            doc_path: doc.to_string(),
            frontmatter: None,
            blocks: Vec::new(),
        });
        b.push(Pending {
            doc: doc.to_string(),
            content_hash: content_hash.to_string(),
            observed_ms,
            base,
            observed,
        });
        Ok(())
    }

    /// The head of `doc` before the oldest unattested ingest of the same
    /// bytes still in its window, if any (the state a late link update is
    /// checked against).
    pub fn pending_base(
        &self,
        policy: &Policy,
        doc: &str,
        content_hash: &str,
    ) -> Result<Option<rootmarkcore::attest::LinkBase>> {
        let now = self.now_ms();
        let w = policy.attestation.window_ms;
        let b = self
            .bases
            .lock()
            .map_err(|_| anyhow::anyhow!("base lock poisoned"))?;
        let m: Vec<_> = b
            .iter()
            .filter(|x| {
                x.doc == doc
                    && x.content_hash == content_hash
                    && now.saturating_sub(x.observed_ms) <= w
            })
            .collect();
        if m.len() > 1 {
            // the same bytes observed unattested more than once: which
            // change a late attestation would vouch for is ambiguous (the
            // window raises none, Y1 security review B5)
            bail!("{doc}: these bytes were observed unattested more than once in the window");
        }
        Ok(m.first().map(|x| x.base.clone()))
    }

    /// Forgets the bases of `doc`'s unattested ingests of `content_hash`
    /// (an attestation of those bytes was recorded: matched or not, they
    /// serve no later check).
    pub fn drop_bases(&self, doc: &str, content_hash: &str) -> Result<()> {
        let mut b = self
            .bases
            .lock()
            .map_err(|_| anyhow::anyhow!("base lock poisoned"))?;
        b.retain(|x| !(x.doc == doc && x.content_hash == content_hash));
        Ok(())
    }

    /// A late partial match (P2-0.1 T2): the one unattested ingest of `doc`
    /// in its window, of other bytes than `content_hash`, whose observed
    /// blocks contain the attested blocks `attested` (versions) and that
    /// they can precede (`late_partial_raises`: the saved bytes hold every
    /// block the change left as it was) — taken out of the window, with the
    /// lineages to raise. More than one candidate is ambiguous, and a match
    /// that would raise nothing is none (the attestation then stays for its
    /// own observation; Y2 review 1 b, c).
    pub fn late_partial(
        &self,
        policy: &Policy,
        doc: &str,
        content_hash: &str,
        attested: &[String],
    ) -> Result<Option<Vec<rootmarkcore::attest::Raise>>> {
        let now = self.now_ms();
        let w = policy.attestation.window_ms;
        let mut b = self
            .bases
            .lock()
            .map_err(|_| anyhow::anyhow!("base lock poisoned"))?;
        let m: Vec<usize> = b
            .iter()
            .enumerate()
            .filter(|(_, x)| {
                x.doc == doc
                    && x.content_hash != content_hash
                    && now.saturating_sub(x.observed_ms) <= w
                    && rootmarkcore::attest::covers(&x.observed, attested)
            })
            .map(|(i, _)| i)
            .collect();
        if m.len() != 1 {
            return Ok(None);
        }
        let mut win = self.window(policy)?;
        let p = &b[m[0]];
        let Some(lineages) = win.pending_lineages(&p.doc, &p.content_hash, p.observed_ms) else {
            return Ok(None);
        };
        let raises =
            match rootmarkcore::attest::late_partial_raises(&p.observed, attested, &lineages) {
                Some(r) if !r.is_empty() => r,
                _ => return Ok(None),
            };
        win.take_unattested(&p.doc, &p.content_hash, p.observed_ms);
        b.remove(m[0]);
        Ok(Some(raises))
    }

    /// Forgets a recorded attestation in the window (it matched late,
    /// partially).
    pub fn forget(&self, policy: &Policy, attestation_id: &str) -> Result<()> {
        self.window(policy)?.forget(attestation_id);
        Ok(())
    }

    /// The old path of the newest accepted rename to `to`, if any.
    pub fn renamed_from_of(&self, to: &str) -> Result<Option<String>> {
        let r = self
            .renames
            .lock()
            .map_err(|_| anyhow::anyhow!("rename lock poisoned"))?;
        Ok(r.iter()
            .rev()
            .find(|(t, _)| t == to)
            .map(|(_, f)| f.clone()))
    }

    /// An accepted rename attestation `from` → `to` (P2-0.1 T1).
    pub fn renamed(&self, from: &str, to: &str) -> Result<()> {
        let mut r = self
            .renames
            .lock()
            .map_err(|_| anyhow::anyhow!("rename lock poisoned"))?;
        r.push((to.to_string(), from.to_string()));
        if r.len() > MAX_RENAMES {
            r.remove(0);
        }
        Ok(())
    }

    /// The document whose head a link update of `doc` is checked against:
    /// `doc` itself when the log knows it (`known`), else the path an
    /// accepted rename moved it from, followed back (a chain of renames
    /// the watcher has not observed yet). An error when neither is known.
    pub fn base_of(&self, doc: &str, known: impl Fn(&str) -> bool) -> Result<String> {
        let r = self
            .renames
            .lock()
            .map_err(|_| anyhow::anyhow!("rename lock poisoned"))?;
        let mut d = doc.to_string();
        for _ in 0..=MAX_RENAMES {
            if known(&d) {
                return Ok(d);
            }
            match r.iter().rev().find(|(to, _)| *to == d) {
                Some((_, from)) => d = from.clone(),
                None => break,
            }
        }
        bail!("{doc} is not a document of the log")
    }

    /// A link-update attestation was accepted.
    pub fn link_update(&self) {
        self.link_updates
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Link-update attestations accepted since the server started.
    pub fn link_updates(&self) -> u64 {
        self.link_updates.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Milliseconds since the service started (the matcher's clock).
    pub fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn window(&self, policy: &Policy) -> Result<std::sync::MutexGuard<'_, Window>> {
        let mut w = self
            .window
            .lock()
            .map_err(|_| anyhow::anyhow!("attestation window lock poisoned"))?;
        // the window follows the recipe (it may change while serving)
        w.window_ms = policy.attestation.window_ms;
        Ok(w)
    }

    /// How an external change of `doc_path` with bytes hashing to
    /// `content_hash`, observed now, is ingested under `policy`; with the
    /// observation time (the late window starts there, not at the commit).
    pub fn decide(
        &self,
        policy: &Policy,
        doc_path: &str,
        content_hash: &str,
        text: &str,
    ) -> Result<(Ingest, u64)> {
        let now = self.now_ms();
        if !policy.attestation.unverified {
            return Ok((Ingest::Canonical, now));
        }
        let mut w = self.window(policy)?;
        let i = match w.observe(doc_path, content_hash, now) {
            Some(id) => Ingest::Attested(id),
            None => match w.observe_partial(doc_path, text, now)? {
                Some((id, saved)) => Ingest::Partial(id, saved),
                None => Ingest::Unverified,
            },
        };
        Ok((i, now))
    }

    /// An unattested ingest observed at `observed_ms` left `lineages`
    /// `unverified`: a late attestation within the window may raise them.
    pub fn unattested(
        &self,
        policy: &Policy,
        doc_path: &str,
        content_hash: &str,
        observed_ms: u64,
        lineages: Vec<Raise>,
    ) -> Result<()> {
        self.window(policy)?.unattested(Unattested {
            doc_path: doc_path.to_string(),
            content_hash: content_hash.to_string(),
            observed_ms,
            lineages,
        });
        Ok(())
    }

    /// An attestation was recorded now: the unattested ingests it matches
    /// late (spec §7.1).
    pub fn attested(
        &self,
        policy: &Policy,
        a: &rootmarkcore::attest::Attestation,
        late: bool,
        content: Option<String>,
    ) -> Result<(Vec<Unattested>, bool)> {
        let now = self.now_ms();
        Ok(self.window(policy)?.attest_content(a, now, late, content))
    }

    /// Unattested ingests still inside their window.
    pub fn pending(&self) -> Result<usize> {
        let w = self
            .window
            .lock()
            .map_err(|_| anyhow::anyhow!("attestation window lock poisoned"))?;
        Ok(w.pending())
    }

    /// A new session (stage 1): `(session_id, key hex, ttl seconds)`.
    pub fn open_session(&self) -> Result<(String, String, u64)> {
        let mut id = [0u8; 16];
        let mut key = [0u8; 32];
        getrandom::fill(&mut id).map_err(|e| anyhow::anyhow!("random session id: {e}"))?;
        getrandom::fill(&mut key).map_err(|e| anyhow::anyhow!("random session key: {e}"))?;
        let mut s = self
            .sessions
            .lock()
            .map_err(|_| anyhow::anyhow!("session lock poisoned"))?;
        let now = Instant::now();
        s.retain(|_, v| v.expires > now);
        while s.len() >= MAX_SESSIONS {
            let Some(oldest) = s
                .iter()
                .min_by_key(|(_, v)| v.expires)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            s.remove(&oldest);
        }
        let sid = hex(&id);
        s.insert(
            sid.clone(),
            Session {
                key: key.to_vec(),
                expires: now + SESSION_TTL,
            },
        );
        Ok((sid, hex(&key), SESSION_TTL.as_secs()))
    }

    /// Checks an attestation's MAC against its session; an unknown or
    /// expired session, or a wrong MAC, is an error.
    pub fn verify(
        &self,
        session_id: &str,
        mac_hex: &str,
        doc_path: &str,
        content_hash: &str,
        saved_at_seq: Option<u64>,
    ) -> Result<()> {
        let s = self
            .sessions
            .lock()
            .map_err(|_| anyhow::anyhow!("session lock poisoned"))?;
        let Some(sess) = s.get(session_id) else {
            bail!("unknown attestation session (open one with POST /v1/attest_session)");
        };
        if sess.expires <= Instant::now() {
            bail!("attestation session expired (open a new one with POST /v1/attest_session)");
        }
        let want = mac(
            &sess.key,
            &mac_message(doc_path, content_hash, saved_at_seq),
        );
        if !same(want.as_bytes(), mac_hex.as_bytes()) {
            bail!("attestation MAC does not match its session key");
        }
        Ok(())
    }
}

/// One `POST /v1/<endpoint>` to the vault's running `rootmark serve`
/// (`127.0.0.1:<[server] http_port>`, the token); the JSON answer, or an
/// error with the answer's code and message.
fn post(
    port: u16,
    token: &str,
    endpoint: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value> {
    use std::io::{Read, Write};
    let body = body.to_string();
    let mut c = std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| {
        anyhow::anyhow!(
            "no rootmark serve answers on 127.0.0.1:{port} ({e}); `rootmark attest` sends to the running server (`rootmark serve --vault <vault> --http 127.0.0.1`)"
        )
    })?;
    write!(
        c,
        "POST /v1/{endpoint} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nX-Consumer: rootmark-attest-cli\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    let mut resp = String::new();
    c.read_to_string(&mut resp)?;
    let status: u16 = resp
        .get(9..12)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("{endpoint}: not an HTTP answer"))?;
    let json = resp.split("\r\n\r\n").nth(1).unwrap_or("");
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| anyhow::anyhow!("{endpoint}: HTTP {status}, body is not JSON ({e})"))?;
    if status != 200 {
        bail!(
            "{endpoint}: HTTP {status} {}: {}",
            v["error"]["code"].as_str().unwrap_or("?"),
            v["error"]["message"].as_str().unwrap_or("?")
        );
    }
    Ok(v)
}

/// `rootmark attest <vault> <doc>…` / `--all` (GOAL P2-0, for editors
/// without a client): attests the current bytes of each document to the
/// vault's running server over HTTP with the token, as a client does after a
/// save (stage 1: one session, one MAC per document). The window applies:
/// run it right after saving (e.g. from the editor's save hook). With
/// `late = false` (`--all`) nothing already ingested is raised: a bulk
/// attestation must not launder an unattested change it did not look at
/// (security review of P2-0/Z2); it counts for the next observations only.
///
/// Each attestation carries the bytes read (`content`, UTF-8; P2-0.1 T2:
/// a partial match when another writer's bytes reach the file in the same
/// observation), except for `--all`. `renamed_from` (one document, `mv`):
/// the server accepts it only as a rename of canonical content (contract
/// §2.6; P2-0.1 T3).
pub fn cli_attest(
    vault: &std::path::Path,
    docs: &[String],
    late: bool,
    renamed_from: Option<&str>,
) -> Result<serde_json::Value> {
    let policy = Policy::load(vault)?;
    let cfg = crate::config::ServerConfig::from_policy(&policy)?;
    let Some(port) = cfg.http_port else {
        bail!("policy.toml has no [server] http_port: `rootmark attest` needs the running server's port");
    };
    let token = std::fs::read_to_string(crate::token::token_path(vault))
        .map_err(|e| {
            anyhow::anyhow!("read the server token ({e}): start `rootmark serve --http` once")
        })?
        .trim()
        .to_string();
    let s = post(port, &token, "attest_session", &serde_json::json!({}))?;
    let sid = s["session_id"].as_str().unwrap_or_default().to_string();
    let key_hex = s["key"].as_str().unwrap_or_default();
    if sid.is_empty() || key_hex.len() != 64 {
        bail!("attest_session: answer outside the contract: {s}");
    }
    let key: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&key_hex[2 * i..2 * i + 2], 16))
        .collect::<std::result::Result<_, _>>()?;
    let mut out = Vec::new();
    for d in docs {
        let path = crate::paths::doc_file(vault, d)?;
        let bytes =
            std::fs::read(&path).map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
        let h = rootmarkcore::attest::content_hash(&bytes);
        let mut body = serde_json::json!({"doc_path": d, "content_hash": h, "session_id": sid,
            "mac": mac(&key, &mac_message(d, &h, None)), "late": late});
        if late {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                body["content"] = serde_json::json!(text);
            }
        }
        if let Some(from) = renamed_from {
            body["renamed_from"] = serde_json::json!(from);
        }
        out.push(post(port, &token, "attest", &body)?);
    }
    Ok(serde_json::json!({"attested": out.len(), "attestations": out}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_sha256_vector() {
        // RFC 4231 test case 2
        assert_eq!(
            mac(b"Jefe", "what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn sessions_verify_macs() {
        let a = Attesting::new();
        let (sid, key, _) = a.open_session().unwrap();
        let key = (0..key.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&key[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let h = "0123456789abcdef0123456789abcdef";
        let m = mac(&key, &mac_message("a.md", h, Some(4)));
        a.verify(&sid, &m, "a.md", h, Some(4)).unwrap();
        assert!(a.verify(&sid, &m, "a.md", h, None).is_err());
        assert!(a.verify(&sid, &m, "b.md", h, Some(4)).is_err());
        assert!(a.verify("nope", &m, "a.md", h, Some(4)).is_err());
    }
}
