//! Shared helpers of the P2-Q.1 watcher tests (`watch_newdir.rs`,
//! `watch_remove.rs`): a vault served with `rootmark serve --http
//! 127.0.0.1:0 --watch` under a chosen `[attestation]` policy, its event log
//! read as JSON lines, HTTP calls with the token, and waits with a deadline
//! (a deadline passed is a failure; no latency is measured).

#![allow(dead_code)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

/// A fresh, empty directory under the system's temporary directory.
pub fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-p2q1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub struct Served {
    pub vault: PathBuf,
    pub port: u16,
    pub token: String,
    child: Child,
    rest: Option<std::thread::JoinHandle<String>>,
    pub head: String,
}

/// `unattested`: `"canonical"` or `"unverified"`; `window_s`: the
/// attestation window.
pub fn serve(vault: &Path, unattested: &str, window_s: f64) -> Served {
    std::fs::create_dir_all(vault.join(".rootmark")).unwrap();
    std::fs::write(
        vault.join(".rootmark/policy.toml"),
        rootmarkserver::config::base_recipe()
            + &format!(
                "\n[attestation]\nunattested = \"{unattested}\"\nwindow_s = {window_s:.1}\n"
            ),
    )
    .unwrap();
    let vs = vault.to_str().unwrap();
    let mut child = Command::new(rootmark())
        .args(["serve", "--vault", vs, "--http", "127.0.0.1:0", "--watch"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut err = BufReader::new(child.stderr.take().unwrap());
    let mut head = String::new();
    let mut port = 0u16;
    loop {
        let mut l = String::new();
        assert!(err.read_line(&mut l).unwrap() > 0, "server exited: {head}");
        head.push_str(&l);
        if let Some(p) = l
            .trim()
            .strip_prefix("rootmark serve: http listening on 127.0.0.1:")
        {
            port = p.parse().unwrap();
        }
        if l.contains("rootmark serve: watching") {
            break;
        }
    }
    assert!(port > 0, "{head}");
    let rest = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    let token = std::fs::read_to_string(vault.join(".rootmark/server.token"))
        .unwrap()
        .trim()
        .to_string();
    Served {
        vault: vault.to_path_buf(),
        port,
        token,
        child,
        rest: Some(rest),
        head,
    }
}

impl Served {
    /// SIGTERM; the exit status and the server's log after start.
    pub fn stop(mut self) -> (bool, String) {
        let st = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(st.success());
        let code = self.child.wait().unwrap();
        let tail = self.rest.take().unwrap().join().unwrap();
        (code.success(), format!("{}{}", self.head, tail))
    }

    pub fn http(&self, method: &str, path: &str, body: &Value) -> (u16, Value) {
        let body = body.to_string();
        let mut s = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            s,
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\n\
             X-Consumer: p2q1-test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            self.token,
            body.len()
        )
        .unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        let text = String::from_utf8(resp).unwrap();
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        let status: u16 = head.split(' ').nth(1).unwrap().parse().unwrap();
        (status, serde_json::from_str(body).unwrap_or(Value::Null))
    }

    /// The live blocks of `doc` (`blame`).
    pub fn blame(&self, doc: &str) -> Vec<Value> {
        let (st, v) = self.http("POST", "/v1/blame", &json!({"doc_path": doc}));
        assert_eq!(st, 200, "{v}");
        v["blocks"].as_array().cloned().unwrap_or_default()
    }

    /// The documents of the lexical hits for `word` (`search`, hot tier),
    /// proposals and `unverified` content included.
    pub fn hits(&self, word: &str) -> Vec<String> {
        let (st, v) = self.http(
            "POST",
            "/v1/search",
            &json!({"query": word, "mode": "lexical", "k": 20, "include_proposal": true}),
        );
        assert_eq!(st, 200, "{v}");
        v["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["doc_path"].as_str().unwrap().to_string())
            .collect()
    }

    pub fn events(&self) -> Vec<Value> {
        events(&self.vault)
    }

    /// `rootmark attest <vault> <doc> [--renamed-from <from>]`: the JSON
    /// answer and whether the command succeeded.
    pub fn attest(&self, doc: &str, renamed_from: Option<&str>) -> (bool, String) {
        let mut c = Command::new(rootmark());
        c.args(["attest", self.vault.to_str().unwrap(), doc]);
        if let Some(f) = renamed_from {
            c.args(["--renamed-from", f]);
        }
        let o = c.output().unwrap();
        (
            o.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            ),
        )
    }
}

/// The event log's lines (a line being written is left out).
pub fn events(vault: &Path) -> Vec<Value> {
    let t = std::fs::read_to_string(vault.join(".rootmark/events.jsonl")).unwrap_or_default();
    t.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

pub fn of_kind<'a>(evs: &'a [Value], kind: &str, doc: &str) -> Vec<&'a Value> {
    evs.iter()
        .filter(|e| e["kind"] == kind && e["doc_path"] == doc)
        .collect()
}

/// `SnapshotCommitted` events of `doc` with at least one block.
pub fn filled_snapshots(evs: &[Value], doc: &str) -> usize {
    // a snapshot's blocks are not in its event: a snapshot follows the
    // block events of its ingest, so count the snapshots after which the
    // document had live blocks, by replaying the block events
    let mut live: std::collections::BTreeSet<String> = Default::default();
    let mut n = 0;
    for e in evs.iter().filter(|e| e["doc_path"] == doc) {
        let lin: Vec<String> = e["lineage_ids"]
            .as_array()
            .map(|a| a.iter().map(|x| x.as_str().unwrap().to_string()).collect())
            .unwrap_or_default();
        match e["kind"].as_str().unwrap() {
            "BlockDeleted" => {
                for l in &lin {
                    live.remove(l);
                }
            }
            "BlockCreated" | "BlockUpdated" | "BlockSplit" | "BlockMerged" | "BlockMoved" => {
                live.extend(lin)
            }
            "SnapshotCommitted" if !live.is_empty() => n += 1,
            _ => {}
        }
    }
    n
}

/// Waits until `f` holds; panics with `what` after `limit`.
pub fn wait_for(limit: Duration, what: &str, mut f: impl FnMut() -> bool) {
    let t0 = Instant::now();
    while !f() {
        assert!(t0.elapsed() < limit, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Waits until the event log has not grown for `settle`.
pub fn settle(vault: &Path, settle: Duration, limit: Duration) {
    let p = vault.join(".rootmark/events.jsonl");
    let t0 = Instant::now();
    let mut last = (u64::MAX, Instant::now());
    loop {
        let n = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        if n != last.0 {
            last = (n, Instant::now());
        } else if last.1.elapsed() >= settle {
            return;
        }
        assert!(t0.elapsed() < limit, "the vault did not settle");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// One `write` of the whole text (as an editor tool does).
pub fn write(p: &Path, text: &str) {
    let mut f = std::fs::File::create(p).unwrap();
    f.write_all(text.as_bytes()).unwrap();
    f.sync_all().unwrap();
}
