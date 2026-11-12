//! Shared helpers of the P2-Q.3 restart tests (`watch_restart.rs`,
//! `ingest_dir_remove.rs`; `docs/rounds/P2-Q.3/progress/TESTS.md`): a
//! synthetic vault, `rootmark serve --http 127.0.0.1:<port> --watch` as a
//! child process stopped by real process signals (`kill -<SIG> <pid>`), its
//! stderr kept while it runs, the event log read as JSON lines, and waits
//! with a deadline (a deadline passed is a failure). No content leaves the
//! temporary directory; nothing is timed but the deadlines.

#![allow(dead_code)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const LIMIT: Duration = Duration::from_secs(90);
/// `[attestation] window_s` of the test recipe; the watcher holds a removal
/// `window_s` + 0.5 s
pub const WINDOW_S: f64 = 1.0;
pub const HOLD: Duration = Duration::from_millis(1500);
/// between a change made while the server is down and the next start
/// (D0F-6: FSEvents may hand a change made just before a start to the new
/// watcher)
pub const GAP: Duration = Duration::from_millis(1200);

pub fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

/// The number of repetitions of each case: `ROOTMARK_RESTART_REPS`, 20 by
/// default (TESTS.md); a value that is not a positive number is an error.
pub fn reps() -> usize {
    match std::env::var("ROOTMARK_RESTART_REPS") {
        Err(std::env::VarError::NotPresent) => 20,
        Err(e) => panic!("ROOTMARK_RESTART_REPS: {e}"),
        Ok(s) => match s.trim().parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => panic!("ROOTMARK_RESTART_REPS={s:?} is not a positive number"),
        },
    }
}

/// The R1 reproduction on the tree without the reconciliation
/// (`ROOTMARK_RESTART_LEGACY=1`): starts do not wait for the
/// reconciliation's summary line, which that tree never writes.
pub fn legacy() -> bool {
    std::env::var("ROOTMARK_RESTART_LEGACY").is_ok_and(|s| s == "1")
}

/// A fresh, empty directory under the system's temporary directory.
pub fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-p2q3-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A synthetic note with a unique word (`zqr<i>`) in its first paragraph.
pub fn note(i: usize) -> String {
    format!(
        "# Note {i}\n\nThis synthetic note carries the word zqr{i} once.\n\nA second paragraph about topic {} and item {i}.\n",
        i % 3
    )
}

/// One `write` of the whole text, synced.
pub fn write(p: &Path, text: &str) {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).unwrap();
    }
    let mut f = std::fs::File::create(p).unwrap();
    f.write_all(text.as_bytes()).unwrap();
    f.sync_all().unwrap();
}

/// The recipe: the server's base recipe, the human `tester`, the policy
/// `unattested` and `window_s` = `WINDOW_S`.
pub fn recipe(unattested: &str) -> String {
    rootmarkserver::config::base_recipe()
        + &format!(
            "\n[identity]\nhuman_id = \"tester\"\n\n[attestation]\nunattested = \"{unattested}\"\nwindow_s = {WINDOW_S:.1}\n"
        )
}

pub fn run_ok(args: &[&str]) -> String {
    let o = Command::new(rootmark()).args(args).output().unwrap();
    assert!(
        o.status.success(),
        "rootmark {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout).unwrap()
}

/// `rootmark <args>`: (success, stdout, stderr).
pub fn run(args: &[&str]) -> (bool, String, String) {
    let o = Command::new(rootmark()).args(args).output().unwrap();
    (
        o.status.success(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// V0 (TESTS.md): `n0.md`…`n4.md` and `sub/k.md`, the recipe, ingested
/// with the server down, and an index built.
pub fn base_vault(dir: &Path, unattested: &str) {
    for i in 0..5 {
        write(&dir.join(format!("n{i}.md")), &note(i));
    }
    write(
        &dir.join("sub/k.md"),
        "# Keep\n\nA note that keeps the sub directory, word zqrkeep.\n",
    );
    let vs = dir.to_str().unwrap();
    run_ok(&["ingest-dir", vs, vs]);
    write(&dir.join(".rootmark/policy.toml"), &recipe(unattested));
    run_ok(&["index", "build", vs]);
}

/// A copy of `src` (files and `.rootmark/`) at `dst`.
pub fn copy_tree(src: &Path, dst: &Path) {
    let _ = std::fs::remove_dir_all(dst);
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), &to).unwrap();
        }
    }
}

pub struct Served {
    pub vault: PathBuf,
    pub port: u16,
    pub token: String,
    child: Child,
    err: Arc<Mutex<String>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// `rootmark serve --vault <v> --http 127.0.0.1:<port> --watch`; returns
/// once it listens, says "watching" and (unless `legacy()`) has written
/// its reconciliation summary.
pub fn serve(vault: &Path) -> Served {
    let port = free_port();
    let mut child = Command::new(rootmark())
        .args([
            "serve",
            "--vault",
            vault.to_str().unwrap(),
            "--http",
            &format!("127.0.0.1:{port}"),
            "--watch",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let err = Arc::new(Mutex::new(String::new()));
    let e2 = err.clone();
    let stderr = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut r = BufReader::new(stderr);
        loop {
            let mut l = String::new();
            match r.read_line(&mut l) {
                Ok(0) | Err(_) => break,
                Ok(_) => e2.lock().unwrap().push_str(&l),
            }
        }
    });
    let mut sv = Served {
        vault: vault.to_path_buf(),
        port,
        token: String::new(),
        child,
        err,
        reader: Some(reader),
    };
    let t0 = Instant::now();
    loop {
        if let Some(st) = sv.child.try_wait().unwrap() {
            panic!("serve exited ({st}): {}", sv.log());
        }
        if sv.log().contains("rootmark serve: watching") {
            break;
        }
        assert!(t0.elapsed() < LIMIT, "serve did not start: {}", sv.log());
        std::thread::sleep(Duration::from_millis(20));
    }
    sv.token = std::fs::read_to_string(vault.join(".rootmark/server.token"))
        .unwrap()
        .trim()
        .to_string();
    if !legacy() {
        wait_for(LIMIT, "the start's reconciliation", || {
            sv.reconciled().is_some()
        });
    }
    sv
}

/// The reconciliation's summary (`changed`, `removals held`, `errors`),
/// once written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub files: usize,
    pub changed: usize,
    pub held: usize,
    pub errors: usize,
}

fn num_before(s: &str, word: &str) -> Option<usize> {
    let i = s.find(word)?;
    s[..i]
        .split_whitespace()
        .last()?
        .trim_start_matches('(')
        .parse()
        .ok()
}

impl Served {
    /// The server's stderr so far.
    pub fn log(&self) -> String {
        self.err.lock().unwrap().clone()
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn reconciled(&self) -> Option<Summary> {
        let log = self.log();
        let l = log
            .lines()
            .find(|l| l.contains("rootmark serve: watch: start: reconciled"))?;
        Some(Summary {
            files: num_before(l, "files with the log")?,
            changed: num_before(l, "changed")?,
            held: num_before(l, "removals held")?,
            errors: num_before(l, "errors")?,
        })
    }

    /// Sends `sig` (`TERM`, `INT`, `KILL`) with `kill`.
    pub fn signal(&self, sig: &str) {
        let st = Command::new("kill")
            .args([&format!("-{sig}"), &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(st.success(), "kill -{sig}");
    }

    pub fn alive(&mut self) -> bool {
        self.child.try_wait().unwrap().is_none()
    }

    /// Waits for the process to end; (exited with 0, killed by a signal,
    /// the whole stderr).
    pub fn wait(mut self) -> (bool, bool, String) {
        let st = self.child.wait().unwrap();
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
        #[cfg(unix)]
        let by_signal = {
            use std::os::unix::process::ExitStatusExt;
            st.signal().is_some()
        };
        #[cfg(not(unix))]
        let by_signal = false;
        (st.success(), by_signal, self.log())
    }

    /// SIGTERM, then the exit (must be 0).
    pub fn stop(self) -> String {
        self.signal("TERM");
        let (ok, _, log) = self.wait();
        assert!(ok, "serve did not exit 0 on SIGTERM: {log}");
        log
    }

    /// SIGKILL, then the end.
    pub fn kill(self) -> String {
        self.signal("KILL");
        let (_, by_signal, log) = self.wait();
        assert!(by_signal, "serve was not killed: {log}");
        log
    }

    pub fn http(&self, method: &str, path: &str, body: &Value) -> (u16, Value) {
        let body = body.to_string();
        let mut s = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            s,
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\n\
             X-Consumer: p2q3-test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
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

    /// The live blocks of `doc` (`blame`); none for an unknown document.
    pub fn blame(&self, doc: &str) -> Vec<Value> {
        let (st, v) = self.http("POST", "/v1/blame", &json!({"doc_path": doc}));
        if st == 404 && v["error"]["code"] == "not_found" {
            return Vec::new();
        }
        assert_eq!(st, 200, "{v}");
        v["blocks"].as_array().cloned().unwrap_or_default()
    }

    /// The documents of the lexical hits for `word`, proposals and
    /// `unverified` content included.
    pub fn hits(&self, word: &str) -> Vec<String> {
        let (st, v) = self.http(
            "POST",
            "/v1/search",
            &json!({"query": word, "mode": "lexical", "k": 20, "include_proposal": true}),
        );
        assert_eq!(st, 200, "{v}");
        let mut out: Vec<String> = v["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["doc_path"].as_str().unwrap().to_string())
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The event log's lines (a line being written is left out).
pub fn events(vault: &Path) -> Vec<Value> {
    let t = std::fs::read_to_string(vault.join(".rootmark/events.jsonl")).unwrap_or_default();
    t.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

/// The event log's text (compared as text, so a difference reads as lines).
pub fn log_bytes(vault: &Path) -> String {
    std::fs::read_to_string(vault.join(".rootmark/events.jsonl")).unwrap_or_default()
}

pub fn of_kind<'a>(evs: &'a [Value], kind: &str, doc: &str) -> Vec<&'a Value> {
    evs.iter()
        .filter(|e| e["kind"] == kind && e["doc_path"] == doc)
        .collect()
}

/// The lineages `BlockDeleted` closed in `doc`, in log order.
pub fn deleted(evs: &[Value], doc: &str) -> Vec<String> {
    of_kind(evs, "BlockDeleted", doc)
        .iter()
        .flat_map(|e| e["lineage_ids"].as_array().unwrap().iter())
        .map(|l| l.as_str().unwrap().to_string())
        .collect()
}

/// The live lineages of every document, by replaying the block events
/// (independent of the server).
pub fn live(
    evs: &[Value],
) -> std::collections::BTreeMap<String, std::collections::BTreeSet<String>> {
    let mut out: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        Default::default();
    for e in evs {
        let Some(doc) = e["doc_path"].as_str() else {
            continue;
        };
        let lin: Vec<String> = e["lineage_ids"]
            .as_array()
            .map(|a| a.iter().map(|x| x.as_str().unwrap().to_string()).collect())
            .unwrap_or_default();
        let set = out.entry(doc.to_string()).or_default();
        match e["kind"].as_str().unwrap() {
            "BlockDeleted" => {
                for l in &lin {
                    set.remove(l);
                }
            }
            "BlockCreated" | "BlockUpdated" | "BlockSplit" | "BlockMerged" | "BlockMoved" => {
                set.extend(lin)
            }
            _ => {}
        }
    }
    out.retain(|_, s| !s.is_empty());
    out
}

/// The block texts of `doc`'s head (`rootmark replay --loc`).
pub fn head_texts(vault: &Path, doc: &str) -> Vec<String> {
    let out = run_ok(&["replay", vault.to_str().unwrap(), doc, "--loc"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    v["blocks_detail"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|b| b["normalized_text"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// The vault's `*.md` files (no hidden component), vault-relative, sorted.
pub fn md_files(vault: &Path) -> Vec<String> {
    fn walk(root: &Path, d: &Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(d).unwrap() {
            let e = e.unwrap();
            let name = e.file_name().to_str().unwrap().to_string();
            if name.starts_with('.') {
                continue;
            }
            let t = e.file_type().unwrap();
            if t.is_dir() {
                walk(root, &e.path(), out);
            } else if t.is_file() && name.ends_with(".md") {
                let rel = e
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
    }
    let mut out = Vec::new();
    walk(vault, vault, &mut out);
    out.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    out
}

/// The disk and the log agree: the documents with live blocks are the
/// `*.md` files, and each document's head has one block per paragraph of
/// its file with that paragraph's text.
pub fn assert_log_matches_disk(vault: &Path) {
    let evs = events(vault);
    let lv = live(&evs);
    let docs: Vec<String> = lv.keys().cloned().collect();
    assert_eq!(docs, md_files(vault), "live documents != files");
    for d in docs {
        let file = std::fs::read_to_string(vault.join(&d)).unwrap();
        let paras: Vec<&str> = file
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        let texts = head_texts(vault, &d);
        assert_eq!(
            texts.len(),
            paras.len(),
            "{d}: blocks {texts:?} vs {paras:?}"
        );
        for (t, p) in texts.iter().zip(&paras) {
            let last = p.split_whitespace().last().unwrap();
            assert!(t.contains(last), "{d}: block {t:?} is not paragraph {p:?}");
        }
    }
}

/// Waits until `f` holds; panics with `what` after `limit`.
pub fn wait_for(limit: Duration, what: &str, mut f: impl FnMut() -> bool) {
    let t0 = Instant::now();
    while !f() {
        assert!(t0.elapsed() < limit, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Waits until the event log has not grown for `quiet`.
pub fn settle(vault: &Path, quiet: Duration) {
    let p = vault.join(".rootmark/events.jsonl");
    let t0 = Instant::now();
    let mut last = (u64::MAX, Instant::now());
    loop {
        let n = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        if n != last.0 {
            last = (n, Instant::now());
        } else if last.1.elapsed() >= quiet {
            return;
        }
        assert!(t0.elapsed() < LIMIT, "the vault did not settle");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// A case's tally, written as one JSON line to `ROOTMARK_RESTART_OUT` (a
/// file) when set: ran and passed are counted apart.
pub fn record(case: &str, ran: usize, passed: usize, extra: Value) {
    if let Ok(p) = std::env::var("ROOTMARK_RESTART_OUT") {
        let line = json!({
            "case": case,
            "platform": std::env::consts::OS,
            "ran": ran,
            "passed": passed,
            "extra": extra,
        });
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .unwrap();
        // one write of the whole line: cases run in parallel threads
        f.write_all(format!("{line}\n").as_bytes()).unwrap();
    }
}

/// Runs `rep` `reps()` times (each repetition's panic is caught and
/// counted), records the tally and fails if any repetition failed, with
/// the first failure's message.
pub fn repeat(case: &str, mut rep: impl FnMut(usize) -> Value) {
    let n = reps();
    let mut passed = 0;
    let mut first: Option<String> = None;
    let mut extras = Vec::new();
    for i in 0..n {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rep(i))) {
            Ok(x) => {
                passed += 1;
                extras.push(x);
            }
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic".to_string());
                extras.push(json!({"failed": msg.chars().take(300).collect::<String>()}));
                first.get_or_insert(format!("rep {i}: {msg}"));
            }
        }
    }
    record(case, n, passed, Value::Array(extras));
    assert!(
        first.is_none(),
        "{case}: {passed}/{n} passed; first failure {}",
        first.unwrap_or_default()
    );
}
