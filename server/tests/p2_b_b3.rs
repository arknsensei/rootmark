//! P2-B/B3 (GOAL P2-B 4; B-5, B-6): the event stream and the search time
//! filter.
//! - (a) `events`: over HTTP the body holds each event as the bytes of its
//!   `events.jsonl` line, in seq order (MCP's text content likewise); `kinds`
//!   and `path_prefix` only filter (a subsequence, order kept); `since_seq`
//!   at 0, at the head and beyond it; `limit` bounds the page and a larger
//!   one is refused; a prefix outside the vault is refused; after the calls
//!   `events_hash`, the log and the traces are unchanged;
//! - (b) `date_from`/`date_to`: dates from `[index] date_from_path` and from
//!   `[index] date_field`, never from the file time (changing mtime changes
//!   nothing); the trace carries the filter; an unfiltered search's
//!   `query_hash` is `H(query)` as before; an invalid recipe is refused at
//!   load naming the field; documents without a date are left out and
//!   counted.
//!
//! Needs the model files (`make models`).

use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(rootmark()).args(args).output().unwrap()
}

fn root(name: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("rootmark-p2b3-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    r
}

fn recipe(index: &str) -> String {
    let base = rootmarkserver::config::default_recipe();
    let base = if index.is_empty() {
        base
    } else {
        base.replace("[index]\n", &format!("[index]\n{index}\n"))
    };
    format!("{base}\n[identity]\nhuman_id = \"ada\"\n")
}

fn vault(name: &str, index: &str, files: &[(&str, &str)]) -> PathBuf {
    let v = root(name).join("vault");
    std::fs::create_dir_all(&v).unwrap();
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(run(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(v.join(".rootmark/policy.toml"), recipe(index)).unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-b-b3")
}

const FILES: &[(&str, &str)] = &[
    ("journal/2026/10/24.md", "# Dün\n\nÇay demlendi.\n"),
    ("journal/2026/10/25.md", "# Bugün\n\nÇay yine demlendi.\n"),
    (
        "notes/çay.md",
        "---\ndate: 2026-09-01\n---\n# Çay\n\nDemleme notları.\n",
    ),
    (
        "notes/kahve.md",
        "---\ndate: \"2026-10-25T08:00\"\n---\n# Kahve\n\nDemleme değil.\n",
    ),
    ("notes/undated.md", "# Tarihsiz\n\nDemleme hakkında.\n"),
];

fn snapshot(v: &Path) -> (Vec<u8>, Option<Vec<u8>>, String) {
    (
        std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
        std::fs::read(v.join(".rootmark/traces.jsonl")).ok(),
        rootmarkcore::Vault::open(v).unwrap().events_hash().unwrap(),
    )
}

// ---- (a) events ----------------------------------------------------------------------

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
    let mut child = Command::new(rootmark())
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
    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    let start = Instant::now();
    let port = loop {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "serve did not start"
        );
        let l = lines.next().expect("serve exited").unwrap();
        if let Some(p) = l.strip_prefix("rootmark serve: http listening on 127.0.0.1:") {
            break p.trim().parse().unwrap();
        }
    };
    std::thread::spawn(move || for _ in lines {});
    let token = std::fs::read_to_string(v.join(".rootmark/server.token"))
        .unwrap()
        .trim()
        .to_string();
    Served { child, port, token }
}

/// `POST /v1/events`: the status and the raw body.
fn http_events(s: &Served, body: &Value) -> (u16, String) {
    let body = body.to_string();
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    write!(
        c,
        "POST /v1/events HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nX-Consumer: p2-b-b3\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        s.token,
        body.len()
    )
    .unwrap();
    let mut resp = String::new();
    c.read_to_string(&mut resp).unwrap();
    let status: u16 = resp[9..12].parse().unwrap();
    (
        status,
        resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string(),
    )
}

/// The event objects of a raw body, as text (each between the array's
/// separators): the log lines themselves.
fn raw_events(body: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(body).unwrap();
    let n = v["events"].as_array().unwrap().len();
    let start = body.find("\"events\":[").unwrap() + "\"events\":[".len();
    let mut out = Vec::new();
    let (mut depth, mut from, mut in_str, mut esc) = (0i32, start, false, false);
    for (i, ch) in body[start..].char_indices() {
        let i = start + i;
        if in_str {
            match (esc, ch) {
                (true, _) => esc = false,
                (false, '\\') => esc = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '{' => {
                if depth == 0 {
                    from = i;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(body[from..=i].to_string());
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }
    assert_eq!(out.len(), n);
    out
}

#[test]
fn events_are_the_log_lines_and_filters_only_filter() {
    let v = vault("events", "", FILES);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    // a search writes a trace; then nothing else may change
    k.call(
        "search",
        &json!({"query": "çay", "mode": "lexical"}),
        human(),
    )
    .unwrap();
    let before = snapshot(&v);
    let log = String::from_utf8(before.0.clone()).unwrap();
    let lines: Vec<&str> = log.lines().collect();
    let head = lines.len() as u64;
    let s = serve_http(&v);
    // every event, byte for byte, in pages of 7
    let mut got: Vec<String> = Vec::new();
    let mut since = 0;
    loop {
        let (st, body) = http_events(&s, &json!({"since_seq": since, "limit": 7}));
        assert_eq!(st, 200, "{body}");
        let page: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(page["head_seq"], head);
        let evs = raw_events(&body);
        got.extend(evs.iter().cloned());
        let next = page["next_seq"].as_u64().unwrap();
        if evs.len() < 7 {
            assert_eq!(next, head);
            break;
        }
        since = next;
    }
    assert_eq!(got, lines);
    // a filter only filters: a subsequence of the lines, in order
    for f in [
        json!({"kinds": ["SnapshotCommitted", "BlockCreated"]}),
        json!({"path_prefix": ["journal"]}),
        json!({"path_prefix": ["notes/c\u{327}ay.md"], "kinds": ["BlockCreated"]}),
    ] {
        let (st, body) = http_events(&s, &f);
        assert_eq!(st, 200, "{body}");
        let page: Value = serde_json::from_str(&body).unwrap();
        let evs = raw_events(&body);
        assert!(!evs.is_empty(), "{f}");
        let want: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|l| {
                let e: Value = serde_json::from_str(l).unwrap();
                let kinds_ok = f
                    .get("kinds")
                    .is_none_or(|k| k.as_array().unwrap().iter().any(|x| *x == e["kind"]));
                let doc = e["doc_path"].as_str().unwrap();
                let pref_ok = f.get("path_prefix").is_none_or(|p| {
                    let p = p[0].as_str().unwrap();
                    let p = if p.contains('\u{327}') {
                        "notes/çay.md"
                    } else {
                        p
                    };
                    doc == p || doc.starts_with(&format!("{p}/"))
                });
                kinds_ok && pref_ok
            })
            .collect();
        assert_eq!(evs, want, "{f}");
        assert!(page.get("filter").is_some());
    }
    // the NFD prefix is shown in NFC
    let (_, body) = http_events(&s, &json!({"path_prefix": ["notes/c\u{327}ay.md"]}));
    let page: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(page["filter"]["path_prefix"], json!(["notes/çay.md"]));
    // since_seq at 0, at the head, beyond it
    let (_, body) = http_events(&s, &json!({"since_seq": 0, "limit": 1}));
    let p0: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(raw_events(&body), vec![lines[0].to_string()]);
    assert_eq!(p0["next_seq"], 1);
    let (st, body) = http_events(&s, &json!({"since_seq": head}));
    let ph: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(st, 200);
    assert_eq!(ph["events"], json!([]));
    assert_eq!(ph["next_seq"], head);
    let (st, body) = http_events(&s, &json!({"since_seq": head + 1}));
    assert_eq!(st, 400, "{body}");
    assert!(body.contains("invalid_arguments") && body.contains("beyond the log"));
    // limit: above the bound is refused (never cut), so is 0; unknown kind,
    // a prefix outside the vault, an unknown field
    for bad in [
        json!({"limit": 1001}),
        json!({"limit": 0}),
        json!({"kinds": ["Nope"]}),
        json!({"kinds": []}),
        json!({"path_prefix": ["../etc"]}),
        json!({"path_prefix": ["/etc"]}),
        json!({"path_prefix": ["notes/../.."]}),
        json!({"since": 1}),
    ] {
        let (st, body) = http_events(&s, &bad);
        assert_eq!(st, 400, "{bad}: {body}");
        assert!(body.contains("invalid_arguments"), "{bad}: {body}");
    }
    let (st, body) = http_events(&s, &json!({"limit": 1000}));
    assert_eq!(st, 200, "{body}");
    drop(s);
    // nothing written: the log, the traces, events_hash
    assert_eq!(snapshot(&v), before);
    // MCP's structured content is the same events (Kernel::call)
    let r = k
        .call(
            "events",
            &json!({"limit": 3}),
            Caller::mcp(Some("reader/1")),
        )
        .unwrap();
    let want: Vec<Value> = lines[..3]
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(r["events"], json!(want));
    assert_eq!(snapshot(&v), before);
}

/// MCP: the tool is listed, its text content holds the log lines byte for
/// byte, nothing is written.
#[test]
fn events_over_mcp_keep_the_line_bytes() {
    let v = vault("events-mcp", "", FILES);
    let before = snapshot(&v);
    let log = String::from_utf8(before.0.clone()).unwrap();
    let mut child = Command::new(rootmark())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut send = |m: Value| writeln!(stdin, "{m}").unwrap();
    send(
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"p2b3","version":"1"}}}),
    );
    let _ = out.next();
    send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    send(json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
    let list: Value = serde_json::from_str(&out.next().unwrap().unwrap()).unwrap();
    assert!(list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "events"));
    send(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"events","arguments":{"since_seq":2,"limit":4}}}),
    );
    let r: Value = serde_json::from_str(&out.next().unwrap().unwrap()).unwrap();
    let text = r["result"]["content"][0]["text"].as_str().unwrap();
    let want: Vec<&str> = log.lines().skip(2).take(4).collect();
    assert_eq!(raw_events(text), want);
    assert_eq!(r["result"]["structuredContent"]["next_seq"], 6);
    drop(stdin);
    let _ = child.wait();
    assert_eq!(snapshot(&v).0, before.0);
    assert_eq!(snapshot(&v).2, before.2);
}

// ---- (b) the time filter ---------------------------------------------------------------

fn search(k: &Kernel, a: Value) -> anyhow::Result<Value> {
    k.call("search", &a, human())
}

fn docs(r: &Value) -> Vec<String> {
    let mut d: Vec<String> = r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["doc_path"].as_str().unwrap().to_string())
        .collect();
    d.sort();
    d.dedup();
    d
}

#[test]
fn dates_from_the_path() {
    let v = vault("path", "date_from_path = \"{yyyy}/{mm}/{dd}\"", FILES);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let q = json!({"query": "demlendi demleme çay", "mode": "lexical", "k": 20});
    let all = search(&k, q.clone()).unwrap();
    let mut a = q.clone();
    a["date_from"] = json!("2026-10-25");
    let r = search(&k, a).unwrap();
    assert_eq!(docs(&r), ["journal/2026/10/25.md"]);
    // three notes have no date from the path
    assert_eq!(r["undated_excluded"], 3);
    assert_eq!(r["filter"], json!({"date_from": "2026-10-25"}));
    let mut a = q.clone();
    a["date_from"] = json!("2026-10-24");
    a["date_to"] = json!("2026-10-24");
    a["path_prefix"] = json!(["journal"]);
    let r = search(&k, a).unwrap();
    assert_eq!(docs(&r), ["journal/2026/10/24.md"]);
    assert_eq!(
        r["filter"],
        json!({"date_from": "2026-10-24", "date_to": "2026-10-24", "path_prefix": ["journal"]})
    );
    // the trace carries the filter; the query hash covers it
    let t = k.call("trace", &json!({}), human()).unwrap();
    let last = t["traces"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["filter"], r["filter"]);
    assert_eq!(last["query_hash"], r["query_hash"]);
    assert_ne!(r["query_hash"], all["query_hash"]);
    // an unfiltered search: H(query), unchanged, no new field
    assert_eq!(
        all["query_hash"],
        rootmarkcore::hash32("demlendi demleme çay")
    );
    assert!(all.get("undated_excluded").is_none());
    assert!(all.get("filter").is_none());
}

#[test]
fn dates_from_the_front_matter_never_from_the_file_time() {
    let v = vault("field", "date_field = \"date\"", FILES);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let q = json!({"query": "demleme", "mode": "lexical", "k": 20, "date_from": "2026-09-01", "date_to": "2026-10-31"});
    let r = search(&k, q.clone()).unwrap();
    assert_eq!(docs(&r), ["notes/kahve.md", "notes/çay.md"]);
    assert_eq!(r["undated_excluded"], 3);
    // the file times change: nothing changes (the hits; the trace's
    // returned blocks; only the trace id and seq differ)
    for p in [
        "notes/çay.md",
        "notes/kahve.md",
        "notes/undated.md",
        "journal/2026/10/24.md",
    ] {
        let f = std::fs::File::options()
            .write(true)
            .open(v.join(p))
            .unwrap();
        f.set_modified(std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(86_400 * 365))
            .unwrap();
    }
    let r2 = search(&k, q.clone()).unwrap();
    assert_eq!(r2["hits"], r["hits"]);
    assert_eq!(r2["query_hash"], r["query_hash"]);
    assert_eq!(r2["undated_excluded"], r["undated_excluded"]);
    let mut q = q;
    q["date_from"] = json!("2026-10-01");
    assert_eq!(docs(&search(&k, q).unwrap()), ["notes/kahve.md"]);
}

#[test]
fn invalid_time_filters_and_recipes_are_refused() {
    let v = vault("invalid", "", FILES);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let before = snapshot(&v);
    // no date source in the recipe
    let e = search(&k, json!({"query": "çay", "date_from": "2026-10-01"})).unwrap_err();
    assert!(format!("{e:#}").contains("date_from_path"), "{e:#}");
    assert_eq!(snapshot(&v), before);
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        recipe("date_field = \"date\""),
    )
    .unwrap();
    for bad in [
        json!({"query": "çay", "date_from": "2026-1-01"}),
        json!({"query": "çay", "date_from": "2026-02-30"}),
        json!({"query": "çay", "date_to": "25.10.2026"}),
        json!({"query": "çay", "date_from": "2026-10-25", "date_to": "2026-10-24"}),
    ] {
        let e = search(&k, bad.clone()).unwrap_err();
        let code = rootmarkserver::error::code_of(&e);
        assert_eq!(code.as_str(), "invalid_arguments", "{bad}: {e:#}");
    }
    assert_eq!(snapshot(&v), before);
    // invalid recipes: refused at load, naming the field
    for (idx, field) in [
        (
            "date_from_path = \"{yyyy}/{mm}\"\ndate_field = \"date\"",
            "not both",
        ),
        ("date_from_path = \"{yyyy}/{mm}\"", "date_from_path"),
        (
            "date_from_path = \"{yyyy}/{mm}/{dd}/{hh}\"",
            "date_from_path",
        ),
        ("date_from_path = \"/{yyyy}/{mm}/{dd}\"", "date_from_path"),
        ("date_from_path = 2026", "date_from_path"),
        ("date_field = \"\"", "date_field"),
    ] {
        let text = recipe(idx);
        let p = rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::Builtin).unwrap();
        let e = rootmarkserver::config::ServerConfig::from_policy(&p).unwrap_err();
        assert!(format!("{e:#}").contains(field), "{idx}: {e:#}");
        std::fs::write(v.join(".rootmark/policy.toml"), &text).unwrap();
        let o = run(&[
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--http",
            "127.0.0.1:0",
        ]);
        assert!(!o.status.success(), "{idx}");
    }
}
