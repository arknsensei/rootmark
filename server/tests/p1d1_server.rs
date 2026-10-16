//! P1-D.1/D1.2: the server's channels and governance —
//! - channel → actor (K-D1.3): MCP is an agent (promote denied), HTTP the
//!   vault's human (`[identity] human_id`; without it HTTP writes are 403);
//! - stable error codes (K-D1.5), in `Kernel::call`, MCP and HTTP;
//! - the edit policy in `propose` (K-D1.1) and attributed writes (K-D1.2);
//! - checkpoint naming (K-D1.6);
//! - the writer queue (K-D1.4): a pipelining MCP client gets the same log.
//!
//! Needs the model files (`make models`).

use mdkserver::error::{code_of, Code};
use mdkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn mdk() -> &'static str {
    env!("CARGO_BIN_EXE_mdk")
}

/// mini_vault copied and ingested (human content), index built.
fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("mdk-p1d1s-{}-{name}", std::process::id()));
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
    let st = Command::new(mdk())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn policy(v: &Path, extra: &str) {
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        format!("{}{extra}", mdkserver::config::default_recipe()),
    )
    .unwrap();
}

fn agent() -> Caller<'static> {
    Caller::mcp(Some("agent/1"))
}

fn log_of(v: &Path) -> String {
    std::fs::read_to_string(v.join(".mdkernel/events.jsonl")).unwrap()
}

fn files_of(v: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = mdkcore::files::markdown_files(v)
        .unwrap()
        .into_iter()
        .map(|(d, p)| (d, std::fs::read(p).unwrap()))
        .collect();
    out.sort();
    out
}

fn propose(k: &Kernel, a: Value) -> anyhow::Result<Value> {
    let mut a = a;
    a["reason"] = json!("test");
    k.call("propose", &a, agent())
}

const SAFETY: &str = "# Safety\n\nIntro written by a human.\n\n## Fire\n\nKeep a class D extinguisher next to the roaster.\n\nNever leave a roast unattended.\n\n### Drills\n\nA fire drill every quarter.\n\n## Water\n\nDescale the boiler monthly.\n";

#[test]
fn append_only_refuses_every_canonical_replace_and_allows_proposal_sections() {
    let v = vault("edit");
    std::fs::write(v.join("safety.md"), SAFETY).unwrap();
    let k = Kernel::open(&v).unwrap();
    // the human document is ingested as human content (the watcher's rule)
    k.watch_doc("safety.md", "mdk-serve-watch").unwrap();
    let before_log = log_of(&v);
    let before_files = files_of(&v);
    // D1-1: six replace_section requests that touch canonical blocks
    let cases = [
        (
            "full section",
            json!({"heading": "Water", "content": "Agents descale weekly."}),
        ),
        (
            "partial",
            json!({"heading": "Fire", "content": "Keep a class D extinguisher next to the roaster.\n\nAgents add a sprinkler."}),
        ),
        (
            "heading only body empty",
            json!({"heading": "Drills", "content": ""}),
        ),
        (
            "nested heading",
            json!({"heading": "Fire", "content": "### Drills\n\nA fire drill every quarter."}),
        ),
        (
            "whole human document",
            json!({"heading": "Safety", "content": "Replaced by an agent."}),
        ),
        (
            "mixed",
            json!({"heading": "Water", "content": "Descale the boiler monthly.\n\nAnd weekly."}),
        ),
    ];
    for (name, c) in &cases {
        let mut a = c.clone();
        a["doc_path"] = json!("safety.md");
        a["mode"] = json!("replace_section");
        let e = propose(&k, a).unwrap_err();
        assert_eq!(code_of(&e), Code::Denied, "{name}: {e:#}");
    }
    assert_eq!(log_of(&v), before_log);
    assert_eq!(files_of(&v), before_files);
    // an append without a heading of its own joins the human chunk: denied
    let e = propose(
        &k,
        json!({"doc_path": "safety.md", "mode": "append", "content": "An agent line."}),
    )
    .unwrap_err();
    assert_eq!(code_of(&e), Code::Denied, "{e:#}");
    // with its own heading: accepted, proposal
    propose(&k, json!({"doc_path": "safety.md", "mode": "append", "content": "## Agent notes\n\nAn agent line."})).unwrap();
    // D1-2: replace_section of a section made only of proposals
    let r = propose(&k, json!({"doc_path": "safety.md", "mode": "replace_section", "heading": "Agent notes", "content": "A corrected agent line."})).unwrap();
    let blocks = r["blocks"].as_array().unwrap();
    let n = blocks.len();
    assert_eq!(blocks[n - 2]["layer"], "proposal");
    assert_eq!(blocks[n - 1]["layer"], "proposal");
    assert!(std::fs::read_to_string(v.join("safety.md"))
        .unwrap()
        .ends_with("## Agent notes\n\nA corrected agent line.\n"));
    // the human blocks are all still canonical
    let b = k
        .call("blame", &json!({"doc_path": "safety.md"}), agent())
        .unwrap();
    assert!(b["blocks"].as_array().unwrap()[..n - 2]
        .iter()
        .all(|x| x["layer"] == "canonical"));
}

#[test]
fn unrestricted_deletion_names_the_actor() {
    let v = vault("unrestricted");
    std::fs::write(v.join("safety.md"), SAFETY).unwrap();
    let k = Kernel::open(&v).unwrap();
    k.watch_doc("safety.md", "mdk-serve-watch").unwrap();
    policy(&v, "");
    let recipe = std::fs::read_to_string(v.join(".mdkernel/policy.toml")).unwrap();
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        recipe.replace("policy = \"append_only\"", "policy = \"unrestricted\""),
    )
    .unwrap();
    let r = propose(&k, json!({"doc_path": "safety.md", "mode": "replace_section", "heading": "Water", "content": ""})).unwrap();
    assert_eq!(r["edit_policy"], "unrestricted");
    let log = log_of(&v);
    let del: Value = log
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .rfind(|e| e["kind"] == "BlockDeleted")
        .unwrap();
    assert_eq!(
        del["meta"]["actor"],
        json!({"kind": "agent", "id": "agent/1"})
    );
    assert_eq!(del["meta"]["source"], "agent/1");
    let h = k
        .call(
            "history",
            &json!({"lineage_id": del["lineage_ids"][0]}),
            agent(),
        )
        .unwrap();
    let last = h["events"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["kind"], "BlockDeleted");
    assert_eq!(last["meta"]["actor"]["id"], "agent/1");
}

#[test]
fn watched_edits_are_attributed_to_the_vault_human() {
    let v = vault("watch");
    policy(&v, "\n[identity]\nhuman_id = \"ada\"\n");
    let k = Kernel::open(&v).unwrap();
    let p = v.join("espresso.md");
    let text = std::fs::read_to_string(&p).unwrap();
    // drop the last paragraph
    let cut = text.trim_end().rfind("\n\n").unwrap();
    std::fs::write(&p, &text[..cut + 1]).unwrap();
    k.watch_doc("espresso.md", "mdk-serve-watch").unwrap();
    let del: Value = log_of(&v)
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|e| e["kind"] == "BlockDeleted")
        .unwrap();
    assert_eq!(del["meta"]["actor"], json!({"kind": "human", "id": "ada"}));
    assert_eq!(del["meta"]["source"], "mdk-serve-watch");
}

#[test]
fn error_codes_through_the_kernel() {
    let v = vault("codes");
    let k = Kernel::open(&v).unwrap();
    std::os::unix::fs::symlink(std::env::temp_dir(), v.join("out")).unwrap();
    let big = "x".repeat(1_048_577);
    let cases: Vec<(&str, Value, Code)> = vec![
        (
            "search",
            json!({"query": "x", "mode": "bogus"}),
            Code::InvalidArguments,
        ),
        (
            "read_block",
            json!({"doc_path": "espresso.md"}),
            Code::InvalidArguments,
        ),
        (
            "history",
            json!({"lineage_id": "x", "extra": 1}),
            Code::InvalidArguments,
        ),
        (
            "search",
            json!({"query": "x", "k": "abc"}),
            Code::InvalidArguments,
        ),
        (
            "search",
            json!({"query": "x", "k": -1}),
            Code::InvalidArguments,
        ),
        (
            "search",
            json!({"query": "x", "k": 0}),
            Code::InvalidArguments,
        ),
        ("search", json!({"query": "  "}), Code::EmptyQuery),
        (
            "propose",
            json!({"doc_path": "../escape.md", "content": "x", "reason": "r"}),
            Code::PathOutsideVault,
        ),
        (
            "propose",
            json!({"doc_path": "/tmp/abs.md", "content": "x", "reason": "r"}),
            Code::PathOutsideVault,
        ),
        (
            "propose",
            json!({"doc_path": "out/x.md", "content": "x", "reason": "r"}),
            Code::PathOutsideVault,
        ),
        (
            "propose",
            json!({"doc_path": "big.md", "content": big, "reason": "r"}),
            Code::TooLarge,
        ),
        (
            "propose",
            json!({"doc_path": "espresso.md", "content": "x", "reason": "r", "mode": "replace_section", "heading": "Nope"}),
            Code::HeadingNotFound,
        ),
        (
            "propose",
            json!({"doc_path": "espresso.md", "content": "x", "reason": "r"}),
            Code::Exists,
        ),
        (
            "propose",
            json!({"doc_path": "x.txt", "content": "x", "reason": "r"}),
            Code::InvalidArguments,
        ),
        (
            "propose",
            json!({"doc_path": "x.md", "content": "x", "reason": ""}),
            Code::InvalidArguments,
        ),
        ("history", json!({"lineage_id": "0123"}), Code::NotFound),
        ("blame", json!({"doc_path": "missing.md"}), Code::NotFound),
        (
            "search",
            json!({"query": "x", "at": "no-such-checkpoint"}),
            Code::NotFound,
        ),
        ("nope", json!({}), Code::NotFound),
    ];
    let before = log_of(&v);
    for (tool, a, want) in cases {
        let e = k.call(tool, &a, agent()).unwrap_err();
        assert_eq!(code_of(&e), want, "{tool} {a}: {e:#}");
    }
    assert_eq!(log_of(&v), before);
    assert!(!v.join(".mdkernel/traces.jsonl").exists());
    std::fs::write(v.join(".mdkernel/policy.toml"), "promotion = 7\n").unwrap();
    let e = k.call("status", &json!({}), agent()).unwrap_err();
    assert_eq!(code_of(&e), Code::PolicyViolation, "{e:#}");
}

#[test]
fn checkpoint_names_the_unnamed_state_checkpoint() {
    let v = vault("cp");
    let k = Kernel::open(&v).unwrap();
    let s = k
        .call("search", &json!({"query": "espresso"}), agent())
        .unwrap();
    let cid = s["checkpoint_id"].as_str().unwrap().to_string();
    let c = k
        .call("checkpoint", &json!({"name": "morning"}), agent())
        .unwrap();
    assert_eq!(c["checkpoint_id"], cid);
    assert_eq!(c["name"], "morning");
    // the same again: the same checkpoint
    assert_eq!(
        k.call("checkpoint", &json!({"name": "morning"}), agent())
            .unwrap(),
        c
    );
    // another name for the named state: exists
    let e = k
        .call("checkpoint", &json!({"name": "noon"}), agent())
        .unwrap_err();
    assert_eq!(code_of(&e), Code::Exists, "{e:#}");
    // a name another checkpoint holds: exists
    propose(
        &k,
        json!({"doc_path": "agent/n.md", "content": "# N\n\nnew"}),
    )
    .unwrap();
    let e = k
        .call("checkpoint", &json!({"name": "morning"}), agent())
        .unwrap_err();
    assert_eq!(code_of(&e), Code::Exists, "{e:#}");
    let s = k
        .call(
            "search",
            &json!({"query": "espresso", "at": "morning"}),
            agent(),
        )
        .unwrap();
    assert_eq!(s["checkpoint_id"], cid);
}

#[test]
fn cli_propose_goes_through_the_edit_policy() {
    let v = vault("cli");
    let draft = v.parent().unwrap().join("d.md");
    std::fs::write(&draft, "Replaced by an agent.\n").unwrap();
    let out = Command::new(mdk())
        .args([
            "propose",
            v.to_str().unwrap(),
            "espresso.md",
            draft.to_str().unwrap(),
            "--source",
            "cli-agent",
            "--mode",
            "append",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("denied by edit policy append_only"), "{err}");
    let out = Command::new(mdk())
        .args([
            "propose",
            v.to_str().unwrap(),
            "agent/cli.md",
            draft.to_str().unwrap(),
            "--source",
            "cli-agent",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let r: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["actor"], json!({"kind": "agent", "id": "cli-agent"}));
    assert_eq!(r["layers"][0]["layer"], "proposal");
}

// ---- HTTP -----------------------------------------------------------------

struct Served {
    child: Child,
    port: u16,
    token: String,
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn serve_http(v: &Path) -> Served {
    let mut child = Command::new(mdk())
        .args([
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--http",
            "127.0.0.1:0",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let err = child.stderr.take().unwrap();
    let mut lines = BufReader::new(err).lines();
    let start = Instant::now();
    let port = loop {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "serve did not start"
        );
        let l = lines.next().expect("serve exited").unwrap();
        if let Some(p) = l.strip_prefix("mdk serve: http listening on 127.0.0.1:") {
            break p.trim().parse().unwrap();
        }
    };
    std::thread::spawn(move || for _ in lines {});
    let token = std::fs::read_to_string(v.join(".mdkernel/server.token"))
        .unwrap()
        .trim()
        .to_string();
    Served { child, port, token }
}

fn http(s: &Served, tool: &str, body: &Value, consumer: &str) -> (u16, Value) {
    let body = body.to_string();
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    write!(
        c,
        "POST /v1/{tool} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nX-Consumer: {consumer}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        s.token,
        body.len()
    )
    .unwrap();
    let mut resp = String::new();
    c.read_to_string(&mut resp).unwrap();
    let status: u16 = resp[9..12].parse().unwrap();
    let json = resp.split("\r\n\r\n").nth(1).unwrap_or("");
    // chunked bodies are not used by the server (Content-Length replies)
    (status, serde_json::from_str(json).unwrap_or(Value::Null))
}

#[test]
fn http_is_the_human_channel() {
    let v = vault("http");
    // an agent proposal to promote, written before the server starts
    let k = Kernel::open(&v).unwrap();
    let r = propose(
        &k,
        json!({"doc_path": "agent/p.md", "content": "# P\n\nAgent text."}),
    )
    .unwrap();
    let lineage = r["blocks"][1]["lineage_id"].as_str().unwrap().to_string();
    drop(k);
    let s = serve_http(&v);
    // without [identity] human_id: writes are 403 denied, searches work
    for (tool, body) in [
        (
            "propose",
            json!({"doc_path": "h.md", "content": "x", "reason": "r"}),
        ),
        ("promote", json!({"lineage_id": lineage, "reason": "r"})),
        ("checkpoint", json!({"name": "x"})),
    ] {
        let (st, b) = http(&s, tool, &body, "human:me");
        assert_eq!(st, 403, "{tool}: {b}");
        assert_eq!(b["error"]["code"], "denied", "{tool}: {b}");
    }
    let (st, _) = http(&s, "search", &json!({"query": "espresso"}), "obsidian");
    assert_eq!(st, 200);
    // with it: HTTP promote is the human's and is allowed (HumanOnly)
    let recipe = std::fs::read_to_string(v.join(".mdkernel/policy.toml")).unwrap();
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        format!("{recipe}\n[identity]\nhuman_id = \"ada\"\n"),
    )
    .unwrap();
    let (st, b) = http(
        &s,
        "promote",
        &json!({"lineage_id": lineage, "reason": "ok"}),
        "agent:x",
    );
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["decision"], "promoted", "{b}");
    assert_eq!(
        b["event"]["meta"]["actor"],
        json!({"kind": "human", "id": "ada"})
    );
    // codes and statuses
    let (st, b) = http(&s, "search", &json!({"query": ""}), "c");
    assert_eq!(
        (st, b["error"]["code"].clone()),
        (400, json!("empty_query"))
    );
    let (st, b) = http(&s, "history", &json!({"lineage_id": "0123"}), "c");
    assert_eq!((st, b["error"]["code"].clone()), (404, json!("not_found")));
    let (st, b) = http(
        &s,
        "propose",
        &json!({"doc_path": "espresso.md", "content": "x", "reason": "r"}),
        "c",
    );
    assert_eq!((st, b["error"]["code"].clone()), (409, json!("exists")));
    let (st, b) = http(&s, "nope", &json!({}), "c");
    assert_eq!((st, b["error"]["code"].clone()), (404, json!("not_found")));
    // promoting a lineage that is already canonical: invalid_arguments
    let (st, b) = http(
        &s,
        "promote",
        &json!({"lineage_id": lineage, "reason": "again"}),
        "c",
    );
    assert_eq!(
        (st, b["error"]["code"].clone()),
        (400, json!("invalid_arguments")),
        "{b}"
    );
}

// ---- MCP pipelining (K-D1.4) ------------------------------------------------

/// Sends every message at once (no waiting), then reads the responses.
fn mcp_pipelined(v: &Path, calls: &[Value]) -> Vec<Value> {
    let mut child = Command::new(mdk())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut msgs = vec![
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "pipe", "version": "1.0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    ];
    for (i, c) in calls.iter().enumerate() {
        msgs.push(json!({"jsonrpc": "2.0", "id": i + 1, "method": "tools/call", "params": c}));
    }
    let mut buf = String::new();
    for m in &msgs {
        buf.push_str(&m.to_string());
        buf.push('\n');
    }
    stdin.write_all(buf.as_bytes()).unwrap();
    let out = BufReader::new(child.stdout.take().unwrap());
    let mut resps = Vec::new();
    for l in out.lines() {
        let r: Value = serde_json::from_str(&l.unwrap()).unwrap();
        if r.get("id").is_some() {
            resps.push(r);
        }
        if resps.len() == calls.len() + 1 {
            break;
        }
    }
    drop(stdin);
    assert!(child.wait().unwrap().success());
    resps.sort_by_key(|r| r["id"].as_u64().unwrap());
    resps
}

#[test]
fn pipelined_calls_give_the_same_log() {
    let a = vault("pipe-a");
    let b = vault("pipe-b");
    let mut calls = Vec::new();
    for i in 0..12 {
        calls.push(json!({"name": "propose", "arguments": {
            "doc_path": format!("agent/p{}.md", i % 4),
            "content": format!("## Note {i}\n\nAgent note number {i} about grinders."),
            "reason": "pipe", "mode": if i < 4 { "create" } else { "append" }}}));
        calls.push(json!({"name": "search", "arguments": {"query": format!("grinders {i}"), "include_proposal": true}}));
        calls.push(json!({"name": "history", "arguments": {"lineage_id": "0123"}}));
    }
    let ra = mcp_pipelined(&a, &calls);
    let rb = mcp_pipelined(&b, &calls);
    assert_eq!(ra.len(), calls.len() + 1);
    // the same responses, too (writes and traces in the same order)
    assert_eq!(ra[1..], rb[1..]); // the initialize reply names the vault path
    for name in ["events.jsonl", "traces.jsonl", "proposals.jsonl"] {
        assert_eq!(
            std::fs::read(a.join(".mdkernel").join(name)).unwrap(),
            std::fs::read(b.join(".mdkernel").join(name)).unwrap(),
            "{name}"
        );
    }
    assert_eq!(files_of(&a), files_of(&b));
    // the appends landed in the order sent
    let p0 = std::fs::read_to_string(a.join("agent/p0.md")).unwrap();
    let (i4, i8) = (p0.find("Note 4").unwrap(), p0.find("Note 8").unwrap());
    assert!(i4 < i8);
    // the traces are in the order the searches were sent
    let traces = std::fs::read_to_string(a.join(".mdkernel/traces.jsonl")).unwrap();
    assert_eq!(traces.lines().count(), 12);
    // tool errors carry a code in structuredContent
    let h = &ra[3];
    assert_eq!(h["result"]["isError"], true);
    assert_eq!(
        h["result"]["structuredContent"]["error"]["code"],
        "not_found"
    );
}

/// D1.2 review: a cancelled call still runs (the SDK does not abort it) and
/// keeps its arrival place, so cancellations change nothing in the log.
fn mcp_with_cancels(v: &Path, n: usize, cancel: bool) {
    let mut child = Command::new(mdk())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut buf = String::new();
    let mut push = |m: Value| {
        buf.push_str(&m.to_string());
        buf.push('\n');
    };
    push(
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "cancel", "version": "1.0"}}}),
    );
    push(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    for i in 1..=n {
        push(
            json!({"jsonrpc": "2.0", "id": i, "method": "tools/call", "params": {"name": "propose",
            "arguments": {"doc_path": "agent/c.md", "mode": if i == 1 { "create" } else { "append" },
                "content": format!("## Part {i}\n\nText {i}."), "reason": "cancel"}}}),
        );
        if cancel && i % 2 == 0 {
            push(
                json!({"jsonrpc": "2.0", "method": "notifications/cancelled",
                "params": {"requestId": i, "reason": "test"}}),
            );
        }
    }
    // a final writer: it runs after every earlier ticket
    let last = n + 1;
    push(json!({"jsonrpc": "2.0", "id": last, "method": "tools/call",
        "params": {"name": "search", "arguments": {"query": "text"}}}));
    stdin.write_all(buf.as_bytes()).unwrap();
    stdin.flush().unwrap();
    for l in BufReader::new(child.stdout.take().unwrap()).lines() {
        let r: Value = serde_json::from_str(&l.unwrap()).unwrap();
        if r["id"] == json!(last) {
            break;
        }
    }
    drop(stdin);
    assert!(child.wait().unwrap().success());
}

#[test]
fn cancelled_calls_keep_their_place() {
    let a = vault("cancel-a");
    let b = vault("cancel-b");
    mcp_with_cancels(&a, 20, true);
    mcp_with_cancels(&b, 20, false);
    assert_eq!(log_of(&a), log_of(&b));
    assert_eq!(files_of(&a), files_of(&b));
    let text = std::fs::read_to_string(a.join("agent/c.md")).unwrap();
    let pos: Vec<usize> = (1..=20)
        .map(|i| text.find(&format!("## Part {i}\n")).unwrap())
        .collect();
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "{text}");
}

/// D1.2 review: when the ingest fails after the file was written, the file
/// is restored and the directories the call created are removed.
#[cfg(unix)]
#[test]
fn failed_commit_restores_the_file() {
    use std::os::unix::fs::PermissionsExt;
    let v = vault("restore");
    let k = Kernel::open(&v).unwrap();
    let objects = v.join(".mdkernel/objects");
    let mode = std::fs::metadata(&objects).unwrap().permissions().mode();
    std::fs::set_permissions(&objects, std::fs::Permissions::from_mode(0o555)).unwrap();
    let before = log_of(&v);
    let r = propose(
        &k,
        json!({"doc_path": "newdir/sub/r.md", "content": "# R\n\nA unique zebu sentence."}),
    );
    std::fs::set_permissions(&objects, std::fs::Permissions::from_mode(mode)).unwrap();
    let e = r.unwrap_err();
    assert!(format!("{e:#}").contains("the file was restored"), "{e:#}");
    assert!(!v.join("newdir").exists());
    assert_eq!(log_of(&v), before);
    assert!(!v.join(".mdkernel/proposals.jsonl").exists());
}

/// Sends `msgs` at once and returns the responses to `want` ids (all of
/// them, or panics after 60 s).
fn mcp_raw(v: &Path, msgs: &[Value], want: &[u64]) -> Vec<Value> {
    let mut child = Command::new(mdk())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut buf = String::new();
    for m in msgs {
        buf.push_str(&m.to_string());
        buf.push('\n');
    }
    stdin.write_all(buf.as_bytes()).unwrap();
    stdin.flush().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    std::thread::spawn(move || {
        for l in BufReader::new(stdout).lines() {
            let Ok(l) = l else { break };
            if tx.send(l).is_err() {
                break;
            }
        }
    });
    let mut got = Vec::new();
    while got.len() < want.len() {
        let l = rx
            .recv_timeout(Duration::from_secs(60))
            .expect("no response within 60 s: the writer queue stalled");
        let r: Value = serde_json::from_str(&l).unwrap();
        if r["id"].as_u64().is_some_and(|i| want.contains(&i)) {
            got.push(r);
        }
    }
    drop(stdin);
    let _ = child.wait();
    got.sort_by_key(|r| r["id"].as_u64().unwrap());
    got
}

fn init(name: &str) -> Vec<Value> {
    vec![
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": name, "version": "1.0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    ]
}

/// D1.2 review 2 (A): a write the SDK fails before its tool runs, and whose
/// response the SDK drops because it was cancelled, must not stall the queue.
#[test]
fn failed_and_cancelled_writes_release_their_ticket() {
    let v = vault("leak");
    let mut msgs = init("leak");
    msgs.push(
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
        "name": "propose", "arguments": {"doc_path": "a.md", "content": "# A\n\nx", "reason": "r"},
        "_meta": {"io.modelcontextprotocol/protocolVersion": "1999-01-01"}}}),
    );
    msgs.push(
        json!({"jsonrpc": "2.0", "method": "notifications/cancelled",
        "params": {"requestId": 1, "reason": "test"}}),
    );
    msgs.push(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "propose", "arguments": {"doc_path": "b.md", "content": "# B\n\ny", "reason": "r"}}}));
    let r = mcp_raw(&v, &msgs, &[2]);
    assert_eq!(r[0]["result"]["isError"], false, "{}", r[0]);
    assert!(v.join("b.md").exists());
}

/// D1.2 review 2 (B): over MCP a promotion request is the agent's and the
/// policy denies it — also for a lineage that is already canonical.
#[test]
fn mcp_promote_of_a_canonical_lineage_is_denied() {
    let v = vault("promote-canon");
    let k = Kernel::open(&v).unwrap();
    let b = k
        .call("blame", &json!({"doc_path": "espresso.md"}), agent())
        .unwrap();
    let lineage = b["blocks"][0]["lineage_id"].clone();
    assert_eq!(b["blocks"][0]["layer"], "canonical");
    drop(k);
    let mut msgs = init("promote");
    msgs.push(
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
        "name": "promote", "arguments": {"lineage_id": lineage, "reason": "r"}}}),
    );
    let r = mcp_raw(&v, &msgs, &[1]);
    assert_eq!(r[0]["result"]["isError"], false, "{}", r[0]);
    assert_eq!(r[0]["result"]["structuredContent"]["decision"], "denied");
}
