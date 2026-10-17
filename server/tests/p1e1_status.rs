//! P1-E.1/E1.1 (K-E1.2): `status` does not depend on the index.
//! - E1-1: a real `mdkernel-index/2` index (written by the `p1d-final` `mdk`,
//!   `server/tests/fixtures/index_v2`, `scripts/gen_old_index.sh`) with
//!   `[index] auto_rebuild = false`: `status` over HTTP GET, HTTP POST and
//!   MCP answers 200 with `index.state = "stale"`; `search` 503
//!   `index_stale`; `blame`, `checkpoints`, `trace` 200. With
//!   `auto_rebuild = true` every one of them answers 200 (8 cases).
//! - E1-2: an unreadable manifest (broken JSON): `status` 200 with
//!   `index.state = "damaged"`; `search` 503 (2 cases).
//!
//! Each criterion writes its record to `$MDK_P1E1_OUT/<name>.json` when set
//! (before its threshold is asserted). Needs the model files (`make models`).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn mdk() -> &'static str {
    env!("CARGO_BIN_EXE_mdk")
}

/// A copy of the old-index fixture vault with the default policy recipe
/// (`extra` in its `[index]` table) and a human id.
fn old_vault(name: &str, extra: &str) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_v2/vault");
    let root = std::env::temp_dir().join(format!("mdk-p1e1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let st = Command::new("cp")
        .args(["-R", src.to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(st.success());
    let v = root.join("vault");
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        format!(
            "{}\n[identity]\nhuman_id = \"ada\"\n",
            mdkserver::config::default_recipe().replace("[index]\n", &format!("[index]\n{extra}"))
        ),
    )
    .unwrap();
    v
}

fn out(name: &str, v: &Value) {
    if let Ok(d) = std::env::var("MDK_P1E1_OUT") {
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            Path::new(&d).join(format!("{name}.json")),
            serde_json::to_string_pretty(v).unwrap() + "\n",
        )
        .unwrap();
    }
}

fn index_format(v: &Path) -> Value {
    let p = v.join(".mdkernel/index/manifest.json");
    match serde_json::from_str::<Value>(&std::fs::read_to_string(p).unwrap()) {
        Ok(m) => m["format"].clone(),
        Err(_) => Value::Null,
    }
}

/// One `mdk serve --mcp stdio --http 127.0.0.1:0` process (the contract §1
/// form): HTTP for the human, MCP over stdio for the agent.
struct Served {
    child: Child,
    port: u16,
    token: String,
    start_log: Vec<String>,
    stdin: std::process::ChildStdin,
    stdout: std::io::Lines<BufReader<std::process::ChildStdout>>,
    next_id: u64,
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn serve(v: &Path) -> Served {
    let mut child = Command::new(mdk())
        .args([
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--mcp",
            "stdio",
            "--http",
            "127.0.0.1:0",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
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
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut s = Served {
        child,
        port,
        token,
        start_log,
        stdin,
        stdout,
        next_id: 1,
    };
    s.send(
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "p1e1", "version": "1.0"}}}),
    );
    s.reply(0);
    s.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    s
}

impl Served {
    fn send(&mut self, m: Value) {
        writeln!(self.stdin, "{m}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn reply(&mut self, id: u64) -> Value {
        loop {
            let l = self.stdout.next().expect("mcp closed").unwrap();
            let r: Value = serde_json::from_str(&l).unwrap();
            if r["id"] == json!(id) {
                return r;
            }
        }
    }

    /// MCP `tools/call` → its `result`
    fn mcp(&mut self, tool: &str, a: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": tool, "arguments": a}}));
        self.reply(id)["result"].clone()
    }

    fn http(&self, method: &str, tool: &str, body: Option<&Value>) -> (u16, Value) {
        let body = body.map(|b| b.to_string()).unwrap_or_default();
        let mut c = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            c,
            "{method} /v1/{tool} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nX-Consumer: p1e1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.token,
            body.len()
        )
        .unwrap();
        let mut resp = String::new();
        c.read_to_string(&mut resp).unwrap();
        let status: u16 = resp[9..12].parse().unwrap();
        let json = resp.split("\r\n\r\n").nth(1).unwrap_or("");
        (status, serde_json::from_str(json).unwrap_or(Value::Null))
    }
}

/// Every call of E1-1 against one served vault: (status code, body) per case.
fn e1_1_calls(s: &mut Served) -> Value {
    let q = json!({"query": "extinguisher roaster"});
    let (get_st, get) = s.http("GET", "status", None);
    let (post_st, post) = s.http("POST", "status", Some(&json!({})));
    let m = s.mcp("status", json!({}));
    let (search_st, search) = s.http("POST", "search", Some(&q));
    let (blame_st, blame) = s.http("POST", "blame", Some(&json!({"doc_path": "safety.md"})));
    let (cps_st, _) = s.http("POST", "checkpoints", Some(&json!({})));
    let (trace_st, _) = s.http("POST", "trace", Some(&json!({})));
    json!({
        "status_get": {"http": get_st, "index": get["index"]},
        "status_post": {"http": post_st, "index": post["index"]},
        "status_mcp": {"is_error": m["isError"], "index": m["structuredContent"]["index"]},
        "status_get_equals_post": get == post,
        "status_mcp_equals_http": m["structuredContent"] == get,
        "search": {"http": search_st, "code": search["error"]["code"],
            "hits": search["hits"].as_array().map(|h| h.len())},
        "blame": {"http": blame_st, "blocks": blame["blocks"].as_array().map(|b| b.len())},
        "checkpoints": {"http": cps_st},
        "trace": {"http": trace_st},
    })
}

fn status_ok(r: &Value, state: &str) -> bool {
    r["status_get"]["http"] == 200
        && r["status_post"]["http"] == 200
        && r["status_mcp"]["is_error"] != true
        && r["status_get"]["index"]["state"] == state
        && r["status_post"]["index"]["state"] == state
        && r["status_mcp"]["index"]["state"] == state
        && r["status_get_equals_post"] == true
        && r["status_mcp_equals_http"] == true
}

// ---- E1-1 ---------------------------------------------------------------------

#[test]
fn e1_1_real_old_index_status_and_reads() {
    // auto_rebuild = false: the p1d-final index stays as it is
    let v = old_vault("e11-off", "auto_rebuild = false\n");
    assert_eq!(index_format(&v), "mdkernel-index/2", "fixture");
    let ev0 = std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap();
    let (off, off_log) = {
        let mut s = serve(&v);
        (e1_1_calls(&mut s), s.start_log.clone())
    };
    let idx = &off["status_get"]["index"];
    let cases_off = [
        (
            "status GET 200 stale",
            off["status_get"]["http"] == 200 && idx["state"] == "stale",
        ),
        (
            "status POST 200 stale",
            off["status_post"]["http"] == 200 && off["status_post"]["index"]["state"] == "stale",
        ),
        (
            "status MCP stale",
            off["status_mcp"]["is_error"] != true
                && off["status_mcp"]["index"]["state"] == "stale"
                && off["status_mcp_equals_http"] == true
                && off["status_get_equals_post"] == true,
        ),
        (
            "search 503 index_stale",
            off["search"]["http"] == 503 && off["search"]["code"] == "index_stale",
        ),
        ("blame 200", off["blame"]["http"] == 200),
        ("checkpoints 200", off["checkpoints"]["http"] == 200),
        ("trace 200", off["trace"]["http"] == 200),
    ];
    let fields_ok = idx["format"] == "mdkernel-index/2"
        && idx["expected_format"] == "mdkernel-index/3"
        && idx["auto_rebuild"] == false;
    let untouched = index_format(&v) == "mdkernel-index/2"
        && std::fs::read(v.join(".mdkernel/events.jsonl")).unwrap() == ev0;
    // auto_rebuild = true (the default): rebuilt at start, everything 200
    let v = old_vault("e11-on", "");
    let (on, on_log) = {
        let mut s = serve(&v);
        (e1_1_calls(&mut s), s.start_log.clone())
    };
    let on_pass = status_ok(&on, "ok")
        && on["status_get"]["index"]["format"] == "mdkernel-index/3"
        && on["status_get"]["index"]["auto_rebuild"] == true
        && on["search"]["http"] == 200
        && on["search"]["hits"].as_u64().unwrap_or(0) > 0
        && on["blame"]["http"] == 200
        && on["checkpoints"]["http"] == 200
        && on["trace"]["http"] == 200
        && on_log
            .iter()
            .any(|l| l.contains("index stale") && l.contains("rebuilt"));
    let mut cases = serde_json::Map::new();
    for (name, ok) in cases_off {
        cases.insert(name.to_string(), json!(ok && fields_ok && untouched));
    }
    cases.insert("auto_rebuild=true: all 200".into(), json!(on_pass));
    let passed = cases.values().filter(|b| **b == json!(true)).count();
    let res = json!({"criterion": "E1-1",
        "fixture": "server/tests/fixtures/index_v2 (p1d-final mdk, mdkernel-index/2)",
        "auto_rebuild_false": off, "auto_rebuild_false_start_log_says_stale":
            off_log.iter().any(|l| l.contains("index stale") && l.contains("auto_rebuild = false")),
        "auto_rebuild_false_index_fields_ok": fields_ok, "auto_rebuild_false_untouched": untouched,
        "auto_rebuild_true": on,
        "cases": cases, "passed": format!("{passed}/8"), "pass": passed == 8});
    out("e1_1", &res);
    assert_eq!(res["pass"], true, "{res:#}");
}

// ---- E1-2 ---------------------------------------------------------------------

#[test]
fn e1_2_unreadable_manifest_status_damaged_search_503() {
    let mut runs = serde_json::Map::new();
    let mut passed = 0;
    for (name, extra) in [
        ("auto_rebuild_true", ""),
        ("auto_rebuild_false", "auto_rebuild = false\n"),
    ] {
        let v = old_vault(&format!("e12-{name}"), extra);
        let m = v.join(".mdkernel/index/manifest.json");
        std::fs::write(&m, "{ not json").unwrap();
        let mut s = serve(&v);
        let (st, status) = s.http("GET", "status", None);
        let mcp = s.mcp("status", json!({}));
        let (search_st, search) = s.http("POST", "search", Some(&json!({"query": "extinguisher"})));
        let untouched = std::fs::read_to_string(&m).unwrap() == "{ not json";
        let r = json!({"status_http": st, "index": status["index"],
            "status_mcp_state": mcp["structuredContent"]["index"]["state"],
            "search_http": search_st, "search_code": search["error"]["code"],
            "manifest_untouched": untouched});
        let status_pass = st == 200
            && status["index"]["state"] == "damaged"
            && status["index"]["format"].is_null()
            && status["embedder"].is_null()
            && r["status_mcp_state"] == "damaged";
        let search_pass = search_st == 503 && search["error"]["code"] == "index_stale" && untouched;
        if name == "auto_rebuild_false" {
            // the criterion's two cases on the vault that keeps its index
            passed = status_pass as u8 + search_pass as u8;
        }
        runs.insert(
            name.into(),
            json!({"calls": r, "status_200_damaged": status_pass, "search_503": search_pass}),
        );
    }
    let both = runs
        .values()
        .all(|r| r["status_200_damaged"] == true && r["search_503"] == true);
    let res = json!({"criterion": "E1-2", "runs": runs,
        "passed": format!("{passed}/2"), "pass": passed == 2 && both});
    out("e1_2", &res);
    assert_eq!(res["pass"], true, "{res:#}");
}
