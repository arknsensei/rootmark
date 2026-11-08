//! P2-Q.2/R1 (PQ1-1; `docs/rounds/P2-Q.2/progress/TESTS.md` M1–M11,
//! DESIGN §3): the MCP compatibility matrix of `rootmark serve --mcp stdio`.
//! Each cell is a client — protocol version of `initialize`, the `_meta` its
//! requests carry, and for Claude Code the exact shape of the requests it
//! was recorded sending (`progress/r0_requests.json`: a `server/discover`
//! with the 2026-07-28 request metadata, then `initialize` 2025-11-25) — and
//! a working cell gets: `tools/list` with the 11 tools, byte-equal to 0.3.1's
//! list (`fixtures/p2q2/`), a `search`, and a `propose` whose blocks are in
//! the log with the agent's provenance. The Claude Code cell is PQ1-1:
//! rmcp 3.5.0 locked such a session to the inline lifecycle and refused its
//! `tools/list`. Runs on macOS and Linux (`make test`, CI). Needs the model
//! files (`make models`).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(120);
const PV: &str = "io.modelcontextprotocol/protocolVersion";
const CI: &str = "io.modelcontextprotocol/clientInfo";
const CC: &str = "io.modelcontextprotocol/clientCapabilities";

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/p2q2")
        .join(name)
}

/// mini_vault copied and ingested into a fresh directory.
fn vault(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("rootmark-p2q2c-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    let mini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault");
    for e in std::fs::read_dir(&mini).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "md") {
            std::fs::copy(&p, v.join(p.file_name().unwrap())).unwrap();
        }
    }
    let o = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    v
}

/// A `rootmark serve --mcp stdio` process and its stdout, line by line.
struct Mcp {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
}

impl Mcp {
    fn start(v: &Path) -> Mcp {
        let mut child = Command::new(rootmark())
            .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for l in BufReader::new(out).lines() {
                let Ok(l) = l else { break };
                if tx.send(l).is_err() {
                    break;
                }
            }
        });
        Mcp {
            stdin: child.stdin.take(),
            child,
            lines,
        }
    }

    fn send(&mut self, m: &Value) {
        let s = self.stdin.as_mut().expect("stream open");
        writeln!(s, "{m}").unwrap();
        s.flush().unwrap();
    }

    /// The answer to request `id`: its line as sent and parsed (server
    /// notifications and requests in between are skipped).
    fn answer(&mut self, id: &Value) -> (String, Value) {
        loop {
            let l = self
                .lines
                .recv_timeout(LIMIT)
                .unwrap_or_else(|e| panic!("no answer to request {id}: {e}"));
            let m: Value = serde_json::from_str(&l).unwrap();
            if m.get("id") == Some(id) && m.get("method").is_none() {
                return (l, m);
            }
        }
    }

    fn request(&mut self, id: Value, method: &str, params: Option<Value>) -> (String, Value) {
        let mut m = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if let Some(p) = params {
            m["params"] = p;
        }
        self.send(&m);
        self.answer(&id)
    }

    /// `initialize` and `notifications/initialized`; the negotiated version.
    fn init(&mut self, proto: &str, name: &str, version: &str, caps: Value) -> String {
        let (_, r) = self.request(
            json!(1),
            "initialize",
            Some(json!({
                "protocolVersion": proto,
                "capabilities": caps,
                "clientInfo": {"name": name, "version": version},
            })),
        );
        assert!(r.get("error").is_none(), "initialize {proto}: {r}");
        self.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        r["result"]["protocolVersion"].as_str().unwrap().to_string()
    }

    /// Closes the stream; the process must exit 0.
    fn close(mut self) {
        drop(self.stdin.take());
        for _ in 0..1200 {
            if let Some(st) = self.child.try_wait().unwrap() {
                assert!(st.success(), "rootmark serve exited {st}");
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = self.child.kill();
        panic!("rootmark serve did not exit after the end of the stream");
    }
}

/// The error code of an answer (none: `None`).
fn error_code(r: &Value) -> Option<i64> {
    r.get("error").and_then(|e| e["code"].as_i64())
}

/// The request metadata of 2026-07-28, complete, naming `proto`.
fn meta(proto: &str, name: &str, version: &str, caps: Value) -> Value {
    json!({PV: proto, CI: {"name": name, "version": version}, CC: caps})
}

fn with_meta(mut params: Value, m: &Option<Value>) -> Value {
    if let Some(m) = m {
        params["_meta"] = m.clone();
    }
    params
}

/// A working cell, after its handshake: `tools/list` (0.3.1's list, byte
/// for byte when `raw`), `search`, `propose` of `doc` with the agent's
/// provenance `<name>/<version>`; then the stream closes.
fn works(mut c: Mcp, v: &Path, m: Option<Value>, raw: bool, name: &str, version: &str, doc: &str) {
    let params = m.as_ref().map(|_| with_meta(json!({}), &m));
    let (line, r) = c.request(json!(2), "tools/list", params);
    assert!(r.get("error").is_none(), "tools/list: {r}");
    let want: Value =
        serde_json::from_slice(&std::fs::read(fixture("tools_list_0.3.1.json")).unwrap()).unwrap();
    assert_eq!(r["result"], want, "tools/list differs from 0.3.1's");
    let names: Vec<&str> = r["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    let specs: Vec<&str> = rootmarkserver::tools::specs().iter().map(|s| s.name).collect();
    assert_eq!(names.len(), 11);
    assert_eq!(names, specs, "M11: the server's tools, in order");
    if raw {
        let want_raw = std::fs::read_to_string(fixture("tools_list_0.3.1.raw.jsonl")).unwrap();
        assert_eq!(format!("{line}\n"), want_raw, "tools/list bytes differ from 0.3.1's");
    }
    let (_, r) = c.request(
        json!(3),
        "tools/call",
        Some(with_meta(
            json!({"name": "search", "arguments": {"query": "espresso", "k": 3}}),
            &m,
        )),
    );
    assert!(r.get("error").is_none(), "search: {r}");
    assert_ne!(r["result"]["isError"], true, "search: {r}");
    assert!(r["result"]["structuredContent"]["hits"].is_array(), "search: {r}");
    let (_, r) = c.request(
        json!(4),
        "tools/call",
        Some(with_meta(
            json!({"name": "propose", "arguments": {
                "doc_path": doc,
                "mode": "create",
                "content": "# Agent note\n\nProposed over MCP by the compatibility test.\n",
                "reason": "P2-Q.2 MCP compatibility",
            }}),
            &m,
        )),
    );
    assert!(r.get("error").is_none(), "propose: {r}");
    assert_ne!(r["result"]["isError"], true, "propose: {r}");
    c.close();
    let log = std::fs::read_to_string(v.join(".rootmark/events.jsonl")).unwrap();
    let created: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .filter(|e| e["kind"] == "BlockCreated" && e["doc_path"] == doc)
        .collect();
    assert!(!created.is_empty(), "{doc}: no BlockCreated");
    for e in &created {
        assert_eq!(e["meta"]["provenance"]["author_kind"], "Agent", "{e}");
        assert_eq!(
            e["meta"]["provenance"]["source"],
            format!("{name}/{version}"),
            "{e}"
        );
    }
}

/// M1: `initialize` 2025-06-18, no `_meta` (and the raw bytes of 0.3.1's list).
#[test]
fn m1_2025_06_18_without_meta() {
    let v = vault("m1");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2025-06-18", "generic", "1.0", json!({})), "2025-06-18");
    works(c, &v, None, true, "generic", "1.0", "agent/m1.md");
}

/// M2: `initialize` 2025-11-25, no `_meta`.
#[test]
fn m2_2025_11_25_without_meta() {
    let v = vault("m2");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2025-11-25", "generic", "1.0", json!({})), "2025-11-25");
    works(c, &v, None, true, "generic", "1.0", "agent/m2.md");
}

/// M3: a client asking for 2026-07-28 in `initialize` gets 2025-11-25.
#[test]
fn m3_2026_07_28_asked_negotiates_2025_11_25() {
    let v = vault("m3");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2026-07-28", "generic", "1.0", json!({})), "2025-11-25");
    works(c, &v, None, true, "generic", "1.0", "agent/m3.md");
}

/// M4: complete request metadata naming 2025-11-25 on every request.
#[test]
fn m4_complete_meta_naming_2025_11_25() {
    let v = vault("m4");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2025-11-25", "generic", "1.0", json!({})), "2025-11-25");
    let m = meta("2025-11-25", "generic", "1.0", json!({}));
    works(c, &v, Some(m), false, "generic", "1.0", "agent/m4.md");
}

/// M5: an unrelated `_meta` key (`progressToken`).
#[test]
fn m5_unrelated_meta() {
    let v = vault("m5");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2025-06-18", "generic", "1.0", json!({})), "2025-06-18");
    works(
        c,
        &v,
        Some(json!({"progressToken": 7})),
        false,
        "generic",
        "1.0",
        "agent/m5.md",
    );
}

/// M6: a request naming 2026-07-28 in its `_meta` after `initialize` is an
/// explicit error (the server does not offer 2026-07-28); the session is not
/// locked: the next request without `_meta` works.
#[test]
fn m6_request_naming_2026_07_28_is_refused_session_stays() {
    let v = vault("m6");
    let mut c = Mcp::start(&v);
    assert_eq!(c.init("2025-11-25", "generic", "1.0", json!({})), "2025-11-25");
    let (_, r) = c.request(
        json!(9),
        "tools/list",
        Some(json!({"_meta": meta("2026-07-28", "generic", "1.0", json!({}))})),
    );
    assert_eq!(error_code(&r), Some(-32022), "{r}");
    works(c, &v, None, true, "generic", "1.0", "agent/m6.md");
}

/// M7: an inline-lifecycle attempt (no handshake, complete 2026-07-28
/// metadata) is refused, and the client can still `initialize`.
#[test]
fn m7_inline_attempt_then_initialize() {
    let v = vault("m7");
    let mut c = Mcp::start(&v);
    let (_, r) = c.request(
        json!("pre"),
        "tools/list",
        Some(json!({"_meta": meta("2026-07-28", "generic", "1.0", json!({}))})),
    );
    assert_eq!(error_code(&r), Some(-32022), "{r}");
    assert_eq!(c.init("2025-11-25", "generic", "1.0", json!({})), "2025-11-25");
    works(c, &v, None, true, "generic", "1.0", "agent/m7.md");
}

/// Claude Code 2.1.319's requests (`progress/r0_requests.json`): a
/// `server/discover` with a string id and the complete 2026-07-28 request
/// metadata (refused: unsupported version), then `initialize` 2025-11-25.
fn claude_code_open(v: &Path) -> Mcp {
    let mut c = Mcp::start(v);
    let caps = json!({"elicitation": {}, "extensions": {}, "roots": {}});
    let (_, r) = c.request(
        json!("0"),
        "server/discover",
        Some(json!({"_meta": meta("2026-07-28", "claude-code", "2.1.319", caps)})),
    );
    assert_eq!(error_code(&r), Some(-32022), "server/discover: {r}");
    let negotiated = c.init(
        "2025-11-25",
        "claude-code",
        "2.1.319",
        json!({"elicitation": {}, "roots": {}}),
    );
    assert_eq!(negotiated, "2025-11-25");
    c
}

/// M8: Claude Code's recorded sequence works (PQ1-1).
#[test]
fn m8_claude_code_recorded_sequence() {
    let v = vault("m8");
    let c = claude_code_open(&v);
    works(c, &v, None, true, "claude-code", "2.1.319", "agent/m8.md");
}

/// M9: a discovery probe with incomplete metadata is refused and the client
/// can still `initialize`.
#[test]
fn m9_incomplete_discover_then_initialize() {
    let v = vault("m9");
    let mut c = Mcp::start(&v);
    let (_, r) = c.request(
        json!("0"),
        "server/discover",
        Some(json!({"_meta": {PV: "2026-07-28"}})),
    );
    assert_eq!(error_code(&r), Some(-32602), "server/discover: {r}");
    assert_eq!(c.init("2025-11-25", "claude-code", "2.1.319", json!({})), "2025-11-25");
    works(c, &v, None, true, "claude-code", "2.1.319", "agent/m9.md");
}

/// M10: the same Claude Code session in two fresh vaults gives the same log.
#[test]
fn m10_same_session_same_log() {
    let mut logs = Vec::new();
    for n in ["m10a", "m10b"] {
        let v = vault(n);
        let c = claude_code_open(&v);
        works(c, &v, None, false, "claude-code", "2.1.319", "agent/m10.md");
        logs.push(std::fs::read(v.join(".rootmark/events.jsonl")).unwrap());
    }
    assert!(logs[0] == logs[1], "the two event logs differ");
}
