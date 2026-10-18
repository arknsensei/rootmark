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
}

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
        }
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
    ) -> Result<(Ingest, u64)> {
        let now = self.now_ms();
        if !policy.attestation.unverified {
            return Ok((Ingest::Canonical, now));
        }
        let i = match self.window(policy)?.observe(doc_path, content_hash, now) {
            Some(id) => Ingest::Attested(id),
            None => Ingest::Unverified,
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
    ) -> Result<Vec<Unattested>> {
        let now = self.now_ms();
        Ok(self.window(policy)?.attest(a, now))
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
/// run it right after saving (e.g. from the editor's save hook).
pub fn cli_attest(vault: &std::path::Path, docs: &[String]) -> Result<serde_json::Value> {
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
        out.push(post(
            port,
            &token,
            "attest",
            &serde_json::json!({"doc_path": d, "content_hash": h, "session_id": sid,
                "mac": mac(&key, &mac_message(d, &h, None))}),
        )?);
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
