//! P1-E/E0 (ADDENDUM): the two contract additions the P1-D.1 verification
//! asked for before the client —
//! - E-14 `index_stale` (K-E0.3): an index of another format answers HTTP
//!   503 `index_stale` (not `internal`) when `[index] auto_rebuild = false`;
//!   with `auto_rebuild` (the default) `mdk serve` rebuilds it at start and
//!   the search answers 200;
//! - E-15 an agent's `append` to human blocks (K-E0.4): without a heading of
//!   its own → `policy_violation` with `hint` "start the appended text with
//!   a heading", nothing written; with one → accepted, `proposal`.
//!
//! Each criterion writes its record to `$MDK_P1E_OUT/<name>.json` when set.
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

const SAFETY: &str = "# Safety\n\nIntro written by a human.\n\n## Fire\n\nKeep a class D extinguisher next to the roaster.\n\n## Water\n\nDescale the boiler monthly.\n";

/// One human document, ingested, index built; `extra` goes into the
/// default policy recipe's `[index]` table.
fn vault(name: &str, extra: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("mdk-p1e0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("safety.md"), SAFETY).unwrap();
    let st = Command::new(mdk())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        format!(
            "{}\n[identity]\nhuman_id = \"ada\"\n",
            mdkserver::config::default_recipe().replace("[index]\n", &format!("[index]\n{extra}"))
        ),
    )
    .unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn out(name: &str, v: &Value) {
    if let Ok(d) = std::env::var("MDK_P1E_OUT") {
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            Path::new(&d).join(format!("{name}.json")),
            serde_json::to_string_pretty(v).unwrap() + "\n",
        )
        .unwrap();
    }
}

/// The index manifest set to the P1-D (`mdkernel-index/2`) format: what an
/// index written by an older `mdk` looks like to this one.
fn age_index(v: &Path) {
    let p = v.join(".mdkernel/index/manifest.json");
    let mut m: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    m["format"] = json!("mdkernel-index/2");
    std::fs::write(&p, serde_json::to_string(&m).unwrap()).unwrap();
}

fn index_format(v: &Path) -> Value {
    let p = v.join(".mdkernel/index/manifest.json");
    let m: Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    m["format"].clone()
}

struct Served {
    child: Child,
    port: u16,
    token: String,
    /// the start lines before "http listening"
    start_log: Vec<String>,
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
    let mut start_log = Vec::new();
    let port = loop {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "serve did not start"
        );
        let l = lines.next().expect("serve exited").unwrap();
        if let Some(p) = l.strip_prefix("mdk serve: http listening on 127.0.0.1:") {
            break p.trim().parse().unwrap();
        }
        start_log.push(l);
    };
    std::thread::spawn(move || for _ in lines {});
    let token = std::fs::read_to_string(v.join(".mdkernel/server.token"))
        .unwrap()
        .trim()
        .to_string();
    Served {
        child,
        port,
        token,
        start_log,
    }
}

fn http(s: &Served, tool: &str, body: &Value) -> (u16, Value) {
    let body = body.to_string();
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    write!(
        c,
        "POST /v1/{tool} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nX-Consumer: p1e-e0\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        s.token,
        body.len()
    )
    .unwrap();
    let mut resp = String::new();
    c.read_to_string(&mut resp).unwrap();
    let status: u16 = resp[9..12].parse().unwrap();
    let json = resp.split("\r\n\r\n").nth(1).unwrap_or("");
    (status, serde_json::from_str(json).unwrap_or(Value::Null))
}

// ---- E-14 ---------------------------------------------------------------------

#[test]
fn e14_stale_index_answers_index_stale_or_is_rebuilt() {
    let q = json!({"query": "extinguisher roaster"});
    // auto_rebuild = false: the old index stays, search → 503 index_stale
    let v = vault("e14-off", "auto_rebuild = false\n");
    age_index(&v);
    let ev0 = std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap();
    let off = {
        let s = serve_http(&v);
        let (st, body) = http(&s, "search", &q);
        // read-only tools without the index still answer
        let (st_blame, _) = http(&s, "blame", &json!({"doc_path": "safety.md"}));
        json!({"status": st, "code": body["error"]["code"], "message": body["error"]["message"],
            "start_log_says_stale": s.start_log.iter().any(|l| l.contains("index stale") && l.contains("auto_rebuild = false")),
            "blame_status": st_blame, "index_format_after": index_format(&v),
            "events_unchanged": std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap() == ev0})
    };
    let off_pass = off["status"] == 503
        && off["code"] == "index_stale"
        && off["start_log_says_stale"] == true
        && off["blame_status"] == 200
        && off["index_format_after"] == "mdkernel-index/2"
        && off["events_unchanged"] == true;
    // the default (auto_rebuild = true): rebuilt at start, search → 200
    let v = vault("e14-on", "");
    age_index(&v);
    let on = {
        let s = serve_http(&v);
        let (st, body) = http(&s, "search", &q);
        json!({"status": st, "hits": body["hits"].as_array().map(|h| h.len()),
            "first_hit_doc": body["hits"][0]["doc_path"],
            "start_log_says_rebuilt": s.start_log.iter().any(|l| l.contains("index stale") && l.contains("rebuilt")),
            "index_format_after": index_format(&v)})
    };
    let on_pass = on["status"] == 200
        && on["first_hit_doc"] == "safety.md"
        && on["start_log_says_rebuilt"] == true
        && on["index_format_after"] == "mdkernel-index/3";
    // a missing index outside `mdk serve` (the kernel alone) is index_stale too
    let v = vault("e14-missing", "");
    std::fs::remove_dir_all(v.join(".mdkernel/index")).unwrap();
    let k = Kernel::open(&v).unwrap();
    let e = k.call("search", &q, Caller::http("p1e-e0")).unwrap_err();
    let missing = json!({"code": code_of(&e).as_str()});
    let res = json!({"criterion": "E-14", "auto_rebuild_false": off, "auto_rebuild_true": on,
        "missing_index_kernel": missing,
        "cases": 2, "passed": (off_pass as u8 + on_pass as u8),
        "pass": off_pass && on_pass && code_of(&e) == Code::IndexStale});
    out("e14", &res);
    assert_eq!(res["pass"], true, "{res:#}");
}

/// A damaged index manifest: `mdk serve` starts as in P1-D.1 (the index is
/// neither rebuilt nor touched); since K-E1.2 (GOAL P1-E.1 E1-2) `search`
/// answers 503 `index_stale` — only a rebuild recovers it — where P1-E/E0
/// answered 500 `internal` (contract §3).
#[test]
fn damaged_manifest_still_serves_and_search_is_index_stale() {
    for (name, extra) in [("dmg-on", ""), ("dmg-off", "auto_rebuild = false\n")] {
        let v = vault(name, extra);
        let m = v.join(".mdkernel/index/manifest.json");
        std::fs::write(&m, "{ not json").unwrap();
        let s = serve_http(&v);
        let (st, body) = http(&s, "search", &json!({"query": "extinguisher"}));
        assert_eq!(st, 503, "{name}: {body}");
        assert_eq!(body["error"]["code"], "index_stale", "{name}");
        // propose in a vault with an index needs it too (K-E1.2); nothing written
        let ev = std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap();
        let (st, body) = http(
            &s,
            "propose",
            &json!({"doc_path": "safety.md", "mode": "append", "content": "## Note\n\nA human note.", "reason": "dmg"}),
        );
        assert_eq!(st, 503, "{name}: propose {body}");
        assert_eq!(body["error"]["code"], "index_stale", "{name}");
        assert_eq!(
            std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap(),
            ev,
            "{name}"
        );
        assert!(
            s.start_log
                .iter()
                .any(|l| l.contains("index damaged") && l.contains("index_stale")),
            "{name}: {:?}",
            s.start_log
        );
        assert_eq!(
            std::fs::read_to_string(&m).unwrap(),
            "{ not json",
            "{name}: untouched"
        );
    }
}

// ---- E-15 ---------------------------------------------------------------------

#[test]
fn e15_append_to_human_blocks_needs_a_heading() {
    let v = vault("e15", "");
    let k = Kernel::open(&v).unwrap();
    let agent = Caller::mcp(Some("agent/1"));
    let snap = |v: &Path| {
        (
            std::fs::read(v.join("safety.md")).unwrap(),
            std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap(),
            std::fs::read(v.join(".mdkernel/proposals.jsonl")).ok(),
        )
    };
    let before = snap(&v);
    // without a heading: joins the human `Water` chunk → policy_violation + hint
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "safety.md", "mode": "append", "content": "An agent line.", "reason": "e15"}),
            agent,
        )
        .unwrap_err();
    let j = mdkserver::error::error_json(&e);
    let no_heading = json!({"code": j["error"]["code"], "hint": j["error"]["hint"],
        "message": j["error"]["message"], "nothing_written": snap(&v) == before});
    let no_heading_pass = code_of(&e) == Code::PolicyViolation
        && j["error"]["hint"] == "start the appended text with a heading"
        && no_heading["nothing_written"] == true;
    // the same over HTTP (the human channel is held to the same edit policy)
    let http_case = {
        let s = serve_http(&v);
        let (st, body) = http(
            &s,
            "propose",
            &json!({"doc_path": "safety.md", "mode": "append", "content": "A human-channel line.", "reason": "e15"}),
        );
        json!({"status": st, "code": body["error"]["code"], "hint": body["error"]["hint"],
            "nothing_written": snap(&v) == before})
    };
    let http_pass = http_case["status"] == 422
        && http_case["code"] == "policy_violation"
        && http_case["hint"] == "start the appended text with a heading"
        && http_case["nothing_written"] == true;
    // with its own heading: accepted, the new blocks are proposals
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "safety.md", "mode": "append", "content": "## Öneri: Fire blanket\n\nAn agent line.", "reason": "e15"}),
            agent,
        )
        .unwrap();
    let blocks = r["blocks"].as_array().unwrap();
    let n = blocks.len();
    let heading = json!({"accepted": true, "edit_policy": r["edit_policy"],
        "new_layers": [blocks[n - 2]["layer"], blocks[n - 1]["layer"]],
        "human_layers_canonical": blocks[..n - 2].iter().all(|b| b["layer"] == "canonical")});
    let heading_pass = r["edit_policy"] == "append_only"
        && heading["new_layers"] == json!(["proposal", "proposal"])
        && heading["human_layers_canonical"] == true;
    let res = json!({"criterion": "E-15", "no_heading_mcp_kernel": no_heading, "no_heading_http": http_case,
        "with_heading": heading,
        "cases": 2, "passed": ((no_heading_pass && http_pass) as u8 + heading_pass as u8),
        "pass": no_heading_pass && http_pass && heading_pass});
    out("e15", &res);
    assert_eq!(res["pass"], true, "{res:#}");
}
