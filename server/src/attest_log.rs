//! The server's attestation diagnostic log (P2-A/A0 c; P2-0.2 verification
//! Ö1 c, F7): `rootmark serve --attest-log <file>` appends one JSON line per
//! attestation call and per observed external change, so that a refused
//! attestation (its reason) and an accepted one that matched no observation
//! (P2-0.2 FINAL note 3 (B)) can be told apart afterwards. The file must lie
//! outside the vault (nothing is written into the vault for it) and is
//! diagnostic only: no decision reads it back.
//!
//! Lines (`t_ms`: milliseconds since the server started):
//! - `{"event": "attest", t_ms, doc_path, content_hash, kind, result, …}` —
//!   `kind` `plain` | `rename` | `link_update`; `result` `accepted` with
//!   `attestation_id`, `at_seq`, `recorded`, `raised` (count), `matched`
//!   (`false` when the log line carries `matched: false`), `partial`; or
//!   `refused` with `code` and `reason` (the error message);
//! - `{"event": "observe", t_ms, doc_path, content_hash, seq, ingest, …}` —
//!   the watcher's (or `propose`'s sync) ingest of an external change:
//!   `ingest` `attested` | `partial` (with `attestation_id` and
//!   `seq_distance` = `seq` − the attestation's `at_seq`), `unverified` or
//!   `canonical` (the policy). An `unverified` observation lists the
//!   attestations of the same document accepted within twice the window
//!   before it (`accepted_before`): each with its `content_hash`, `at_seq`,
//!   `seq_distance`, `age_ms` and why it did not match (`other_bytes`: it
//!   vouched for other bytes; `expired`: same bytes, the window had passed;
//!   `consumed`: same bytes, already matched by an earlier observation).
//!   An empty list is "no accepted attestation" (refused or never sent).

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

/// Accepted attestations remembered for `accepted_before` (the oldest go).
const MAX_ACCEPTED: usize = 4096;

struct Accepted {
    id: String,
    doc: String,
    content_hash: String,
    at_seq: u64,
    t_ms: u64,
    consumed: bool,
}

pub struct AttestLog {
    path: PathBuf,
    start: Instant,
    inner: Mutex<(std::fs::File, Vec<Accepted>)>,
}

impl AttestLog {
    /// Opens (appends to) `path`; refused when it lies inside `vault`.
    pub fn open(path: &Path, vault: &Path) -> Result<AttestLog> {
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let dir = parent
            .canonicalize()
            .with_context(|| format!("--attest-log: folder {} not found", parent.display()))?;
        let v = vault
            .canonicalize()
            .with_context(|| format!("vault {}", vault.display()))?;
        // the file itself too, when it exists (a link into the vault: A0
        // review K2)
        let target = match path.file_name() {
            Some(n) => dir.join(n),
            None => bail!("--attest-log {}: not a file name", path.display()),
        };
        let inside = target.starts_with(&v)
            || std::fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink())
                && target.canonicalize().map_or(true, |t| t.starts_with(&v));
        if inside || dir.starts_with(&v) {
            bail!(
                "--attest-log {} lies inside the vault; the diagnostic log is written outside it",
                path.display()
            );
        }
        let f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("--attest-log: open {}", path.display()))?;
        Ok(AttestLog {
            path: path.to_path_buf(),
            start: Instant::now(),
            inner: Mutex::new((f, Vec::new())),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn write(f: &mut std::fs::File, line: &Value) -> Result<()> {
        let mut s = serde_json::to_string(line)?;
        s.push('\n');
        f.write_all(s.as_bytes()).context("--attest-log: write")?;
        Ok(())
    }

    /// An attestation call: `kind`, and its response or its error.
    pub fn attest(
        &self,
        doc: &str,
        content_hash: &str,
        kind: &str,
        res: std::result::Result<Value, &anyhow::Error>,
    ) -> Result<()> {
        let t = self.now_ms();
        let mut g = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("attest log lock poisoned"))?;
        let mut line = json!({
            "event": "attest", "t_ms": t, "doc_path": doc,
            "content_hash": content_hash, "kind": kind,
        });
        match res {
            Ok(r) => {
                line["result"] = json!("accepted");
                for k in ["attestation_id", "at_seq", "recorded", "matched", "partial"] {
                    if let Some(x) = r.get(k) {
                        line[k] = x.clone();
                    }
                }
                let raised = r["raised"].as_array().map_or(0, Vec::len);
                line["raised"] = json!(raised);
                if let (Some(id), Some(at)) = (r["attestation_id"].as_str(), r["at_seq"].as_u64()) {
                    let (_, acc) = &mut *g;
                    acc.push(Accepted {
                        id: id.to_string(),
                        doc: doc.to_string(),
                        content_hash: content_hash.to_string(),
                        at_seq: at,
                        t_ms: t,
                        // a late match raised what it vouched for
                        consumed: raised > 0,
                    });
                    if acc.len() > MAX_ACCEPTED {
                        acc.remove(0);
                    }
                }
            }
            Err(e) => {
                line["result"] = json!("refused");
                line["code"] = json!(crate::error::code_of(e).as_str());
                line["reason"] = json!(format!("{e:#}"));
            }
        }
        Self::write(&mut g.0, &line)
    }

    /// An observed external change ingested at `seq` (`ingest`: attested,
    /// partial, unverified, canonical; `attestation_id` for the first two).
    pub fn observe(
        &self,
        doc: &str,
        content_hash: &str,
        seq: u64,
        ingest: &str,
        attestation_id: Option<&str>,
        window_ms: u64,
    ) -> Result<()> {
        let t = self.now_ms();
        let mut g = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("attest log lock poisoned"))?;
        let (f, acc) = &mut *g;
        let mut line = json!({
            "event": "observe", "t_ms": t, "doc_path": doc,
            "content_hash": content_hash, "seq": seq, "ingest": ingest,
        });
        if let Some(id) = attestation_id {
            line["attestation_id"] = json!(id);
            if let Some(a) = acc.iter_mut().rev().find(|a| a.id == id) {
                line["seq_distance"] = json!(seq as i64 - a.at_seq as i64);
                a.consumed = true;
            }
        }
        if ingest == "unverified" {
            let before: Vec<Value> = acc
                .iter()
                .filter(|a| a.doc == doc && t.saturating_sub(a.t_ms) <= 2 * window_ms)
                .map(|a| {
                    let age = t.saturating_sub(a.t_ms);
                    let why = if a.content_hash != content_hash {
                        "other_bytes"
                    } else if a.consumed {
                        "consumed"
                    } else if age > window_ms {
                        "expired"
                    } else {
                        // same bytes, in the window, unused: refused by a
                        // later rule (a rename's caps, a voiding change)
                        "not_applied"
                    };
                    json!({
                        "attestation_id": a.id, "content_hash": a.content_hash,
                        "at_seq": a.at_seq, "seq_distance": seq as i64 - a.at_seq as i64,
                        "age_ms": age, "why": why,
                    })
                })
                .collect();
            line["accepted_before"] = json!(before);
        }
        Self::write(f, &line)
    }
}
