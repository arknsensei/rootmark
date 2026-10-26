//! P2-R/R2 (GOAL P2-R 3; R-5, R-8, R-4's HTTP path): retention and the
//! explicit prune.
//! - traces behind the bound move to `traces/archive/` byte for byte: the
//!   archive and the live file joined are the former `traces.jsonl`; the live lines keep
//!   their `at_seq` and `trace_id`; a new trace continues the count; a
//!   filtered search's filter line moves with its trace;
//!   `since_seq < base` → `archived_before`;
//! - checkpoints behind the bound, unnamed and read by no live trace move to
//!   `checkpoints/archive/`: listed `archived: true`, a query `at` one is
//!   `archived` (naming the file); a named checkpoint and one a live trace
//!   reads never move;
//! - location records: each head's and the live checkpoints' versions' are
//!   kept, the others deleted;
//! - `dry_run` changes no file (CLI and HTTP); a recipe without
//!   `[retention]` archives nothing and says so;
//! - a broken base, a missing or extra archive file, a deleted checkpoint
//!   file: the vault does not open, the error names the file;
//! - only an explicit call prunes: `POST /v1/prune` needs `human_id` (403),
//!   MCP has no `prune`, a long `rootmark serve` session archives and
//!   deletes nothing; the same vault and call prune alike.
//!
//! Needs the model files (`make models`).

use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

fn ok_json(args: &[&str]) -> Value {
    let o = Command::new(rootmark()).args(args).output().unwrap();
    assert!(
        o.status.success(),
        "rootmark {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

fn err_text(args: &[&str]) -> String {
    let o = Command::new(rootmark()).args(args).output().unwrap();
    assert!(!o.status.success(), "{args:?} succeeded");
    String::from_utf8_lossy(&o.stderr).to_string()
}

const RETENTION: &str = "\n[retention]\nkeep_checkpoints = 2\n";

fn recipe(identity: bool, extra: &str) -> String {
    format!(
        "{}{}{extra}",
        rootmarkserver::config::default_recipe(),
        if identity {
            "\n[identity]\nhuman_id = \"ada\"\n"
        } else {
            ""
        }
    )
}

fn set_recipe(v: &Path, identity: bool, extra: &str) {
    std::fs::write(v.join(".rootmark/policy.toml"), recipe(identity, extra)).unwrap();
}

fn human() -> Caller<'static> {
    Caller::http("p2-r-r2")
}

/// A vault with `a.md`, `b/c.md`; `rounds` rounds of: an edit of `a.md`
/// (watched: the hot tier follows), a search (filtered by `b` on odd
/// rounds) and a `read_block`, each at the new state's checkpoint; after
/// round 1 that checkpoint is named `keep`.
fn scene(name: &str, rounds: usize) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2r2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(v.join("b")).unwrap();
    std::fs::write(v.join("a.md"), "# Alpha\n\nalpha round start\n").unwrap();
    std::fs::write(v.join("b/c.md"), "# Beta\n\nalpha beta notes\n").unwrap();
    let vs = v.to_str().unwrap();
    ok_json(&["ingest-dir", vs, vs]);
    set_recipe(&v, true, "");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    for r in 0..rounds {
        std::fs::write(
            v.join("a.md"),
            format!("# Alpha\n\nalpha round {r} text\n\nsecond {r}\n"),
        )
        .unwrap();
        k.watch_doc("a.md", "p2-r-r2").unwrap();
        let mut q = json!({"query": "alpha", "include_proposal": true});
        if r % 2 == 1 {
            q["path_prefix"] = json!(["b"]);
        }
        k.call("search", &q, human()).unwrap();
        let l = ok_json(&["blame", vs, "a.md"])["blocks"][1]["lineage_id"].clone();
        k.call(
            "read_block",
            &json!({"doc_path": "a.md", "lineage_id": l}),
            human(),
        )
        .unwrap();
        if r == 1 {
            k.call("checkpoint", &json!({"name": "keep"}), human())
                .unwrap();
        }
    }
    v
}

fn meta(v: &Path) -> PathBuf {
    v.join(".rootmark")
}

/// Every file under `.rootmark/` (relative path → bytes); the server token
/// excluded.
fn files(v: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(d: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                let r = p.strip_prefix(base).unwrap().display().to_string();
                if r != "server.token" {
                    out.insert(r, std::fs::read(&p).unwrap());
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(&meta(v), &meta(v), &mut out);
    out
}

fn lines(b: &[u8]) -> Vec<Value> {
    std::str::from_utf8(b)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The archive files of a kind joined in name order (`<first>-<last>`).
fn archive(v: &Path, suffix: &str) -> Vec<u8> {
    let d = meta(v).join("traces/archive");
    let mut fs: Vec<(u64, PathBuf)> = std::fs::read_dir(&d)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            n.ends_with(suffix) && (suffix != ".jsonl" || !n.ends_with(".filters.jsonl"))
        })
        .map(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            (n.split('-').next().unwrap().parse().unwrap(), p)
        })
        .collect();
    fs.sort();
    fs.iter()
        .flat_map(|(_, p)| std::fs::read(p).unwrap())
        .collect()
}

#[test]
fn traces_move_to_the_archive_byte_for_byte_and_keep_their_ids() {
    let v = scene("traces", 6);
    let vs = v.to_str().unwrap();
    set_recipe(&v, true, RETENTION);
    let before = std::fs::read(meta(&v).join("traces.jsonl")).unwrap();
    let filters_before = std::fs::read(meta(&v).join("trace_filters.jsonl")).unwrap();
    let old = lines(&before);
    let out = ok_json(&["prune", vs, "--now-ms", "0"]);
    let t = &out["retention"]["traces"];
    let base = t["base"].as_u64().unwrap();
    assert!(base > 0 && base < old.len() as u64, "{out}");
    assert_eq!(t["lines"], base);
    assert_eq!(
        std::fs::read_to_string(meta(&v).join("traces/base")).unwrap(),
        format!("{base}\n")
    );
    // live + archive = the former file, byte for byte; filters too
    let live = std::fs::read(meta(&v).join("traces.jsonl")).unwrap();
    assert_eq!([archive(&v, ".jsonl"), live.clone()].concat(), before);
    let flive = std::fs::read(meta(&v).join("trace_filters.jsonl")).unwrap();
    assert_eq!(
        [archive(&v, ".filters.jsonl"), flive].concat(),
        filters_before
    );
    assert!(t["filter_lines"].as_u64().unwrap() > 0, "{out}");
    // the live lines are the old lines from the base on: same at_seq, ids
    assert_eq!(lines(&live), old[base as usize..].to_vec());
    // the trace tool says what was archived; the CLI reads the archive
    let k = Kernel::open(&v).unwrap();
    let tr = k.call("trace", &json!({}), human()).unwrap();
    assert_eq!(tr["archived_before"], base);
    assert_eq!(tr["traces"][0]["at_seq"], base);
    let tr = k
        .call("trace", &json!({"since_seq": base}), human())
        .unwrap();
    assert!(tr.get("archived_before").is_none());
    let a = ok_json(&["trace", "list", vs, "--archived"]);
    let ats: Vec<u64> = a["traces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["at_seq"].as_u64().unwrap())
        .collect();
    assert_eq!(ats, (0..base).collect::<Vec<_>>());
    // a new trace continues the count (K18-T)
    let s = k
        .call("search", &json!({"query": "alpha"}), human())
        .unwrap();
    let after = lines(&std::fs::read(meta(&v).join("traces.jsonl")).unwrap());
    let last = after.last().unwrap();
    assert_eq!(last["at_seq"], old.len() as u64);
    assert_eq!(last["trace_id"], s["trace_id"]);
    // pruning again moves nothing new (the bound did not move)
    let again = ok_json(&["prune", vs, "--now-ms", "0"]);
    assert_eq!(again["retention"]["traces"]["lines"], 0);
}

#[test]
fn checkpoints_behind_the_bound_are_archived_unless_named_or_read() {
    let v = scene("checkpoints", 6);
    let vs = v.to_str().unwrap();
    let cps: Vec<Value> = ok_json(&["checkpoint", "list", vs])["checkpoints"]
        .as_array()
        .unwrap()
        .clone();
    assert!(cps.len() >= 6);
    let k = Kernel::open(&v).unwrap();
    // a live trace (the newest line) reads the oldest unnamed checkpoint
    let oldest = cps.iter().find(|c| c["name"].is_null()).unwrap()["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();
    k.call("search", &json!({"query": "alpha", "at": oldest}), human())
        .unwrap();
    set_recipe(&v, true, RETENTION);
    let out = ok_json(&["prune", vs, "--now-ms", "0"]);
    let archived: Vec<String> = out["retention"]["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    assert!(!archived.is_empty(), "{out}");
    let named = cps.iter().find(|c| c["name"] == "keep").unwrap()["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        !archived.contains(&named),
        "a named checkpoint was archived"
    );
    assert!(
        !archived.contains(&oldest),
        "a checkpoint a live trace reads was archived"
    );
    for c in &archived {
        assert!(meta(&v)
            .join(format!("checkpoints/archive/{c}.json"))
            .exists());
        assert!(!meta(&v).join(format!("checkpoints/{c}.json")).exists());
    }
    // the listing marks them; the others are as before
    let list = k.call("checkpoints", &json!({}), human()).unwrap();
    for c in list["checkpoints"].as_array().unwrap() {
        let id = c["checkpoint_id"].as_str().unwrap().to_string();
        assert_eq!(c.get("archived").is_some(), archived.contains(&id), "{c}");
    }
    let cli = ok_json(&["checkpoint", "list", vs]);
    assert_eq!(cli["checkpoints"], list["checkpoints"]);
    // a query as of an archived checkpoint: `archived`, naming the file
    let at = &archived[0];
    let l = ok_json(&["blame", vs, "a.md"])["blocks"][0]["lineage_id"].clone();
    let traces = std::fs::read(meta(&v).join("traces.jsonl")).unwrap();
    for (tool, args) in [
        ("search", json!({"query": "alpha", "at": at})),
        (
            "read_block",
            json!({"doc_path": "a.md", "lineage_id": l, "at": at}),
        ),
        ("blame", json!({"doc_path": "a.md", "at": at})),
    ] {
        let e = k.call(tool, &args, human()).unwrap_err();
        assert_eq!(code_of(&e), Code::Archived, "{tool}: {e:#}");
        assert!(
            format!("{e:#}").contains(&format!(".rootmark/checkpoints/archive/{at}.json")),
            "{e:#}"
        );
    }
    assert_eq!(
        std::fs::read(meta(&v).join("traces.jsonl")).unwrap(),
        traces,
        "a refused query wrote a trace"
    );
    let e = err_text(&[
        "read",
        vs,
        "--at",
        at,
        "--consumer",
        "x",
        l.as_str().unwrap(),
    ]);
    assert!(e.contains("is archived"), "{e}");
    // the named checkpoint still answers
    k.call("search", &json!({"query": "alpha", "at": "keep"}), human())
        .unwrap();
}

#[test]
fn location_records_of_heads_and_live_checkpoints_are_kept() {
    let v = scene("loc", 6);
    let vs = v.to_str().unwrap();
    set_recipe(&v, true, RETENTION);
    let before: Vec<String> = files(&v)
        .into_keys()
        .filter(|p| p.starts_with("loc/"))
        .collect();
    let out = ok_json(&["prune", vs, "--now-ms", "0"]);
    let removed = out["retention"]["loc"]["removed"].as_array().unwrap();
    assert!(!removed.is_empty(), "{out}");
    let after: Vec<String> = files(&v)
        .into_keys()
        .filter(|p| p.starts_with("loc/"))
        .collect();
    assert_eq!(after.len() + removed.len(), before.len());
    assert_eq!(out["retention"]["loc"]["kept"], after.len());
    // every head's record and every live checkpoint's versions' are there
    let vault = rootmarkcore::Vault::open(&v).unwrap();
    let mut want = std::collections::BTreeSet::new();
    for d in vault.doc_paths() {
        want.insert((d.clone(), vault.snapshot(&d).unwrap().unwrap().snapshot_id));
    }
    for c in vault.checkpoints() {
        if !vault.is_archived(&c.checkpoint_id) {
            for (d, s) in c.docs {
                want.insert((d, s));
            }
        }
    }
    for (d, s) in &want {
        let p = rootmarkcore::loc::record_path(&meta(&v), d, s);
        assert!(p.exists(), "{d} {s}");
    }
    assert_eq!(want.len(), after.len());
    // blame still locates the head
    let b = ok_json(&["blame", vs, "a.md", "--loc"]);
    assert!(b["blocks"][0]["loc"].is_object(), "{b}");
}

#[test]
fn a_dry_run_changes_no_file_and_no_table_archives_nothing() {
    let v = scene("dry", 5);
    let vs = v.to_str().unwrap();
    set_recipe(&v, true, RETENTION);
    let before = files(&v);
    let dry = ok_json(&["prune", vs, "--dry-run", "--now-ms", "0"]);
    assert_eq!(dry["dry_run"], true);
    assert!(dry["retention"]["traces"]["lines"].as_u64().unwrap() > 0);
    assert_eq!(files(&v), before);
    // HTTP dry run: the same plan, no file changed
    let s = serve_http(&v);
    let (st, h) = http(
        &s,
        "prune",
        &json!({"dry_run": true, "now_ms": 0}),
        Some("bot"),
    );
    drop(s);
    assert_eq!(st, 200, "{h}");
    for k in ["rules", "retention", "journal", "events_hash"] {
        assert_eq!(h[k], dry[k], "{k}");
    }
    let mut after = files(&v);
    // the server's own index compaction may write; none of the pruned kinds
    after.retain(|p, _| !p.starts_with("index/"));
    let mut b = before.clone();
    b.retain(|p, _| !p.starts_with("index/"));
    assert_eq!(after, b);
    // without [retention]: nothing archived, and the answer says so
    set_recipe(&v, true, "");
    let out = ok_json(&["prune", vs, "--now-ms", "0"]);
    let r = &out["retention"];
    assert!(
        r["note"].as_str().unwrap().contains("no [retention]"),
        "{out}"
    );
    assert_eq!(r["traces"]["lines"], 0);
    assert_eq!(r["checkpoints"], json!([]));
    assert!(r["bound"].is_null());
    assert!(!meta(&v).join("traces/archive").exists());
    assert!(!meta(&v).join("checkpoints/archive").exists());
    assert_eq!(
        std::fs::read(meta(&v).join("traces.jsonl")).unwrap(),
        before["traces.jsonl"]
    );
    assert_eq!(out["events_hash"], out["events_hash_before"]);
}

#[test]
fn a_broken_archive_or_base_is_an_error_naming_it() {
    let v = scene("tamper", 6);
    let vs = v.to_str().unwrap();
    set_recipe(&v, true, RETENTION);
    let out = ok_json(&["prune", vs, "--now-ms", "0"]);
    let m = meta(&v);
    let base = out["retention"]["traces"]["base"].as_u64().unwrap();
    let arch = m.join(format!("traces/archive/0-{}.jsonl", base - 1));
    assert!(arch.exists());
    let cp = out["retention"]["checkpoints"][0]
        .as_str()
        .unwrap()
        .to_string();
    let open_err = || {
        rootmarkcore::Vault::open(&v)
            .err()
            .map(|e| format!("{e:#}"))
            .expect("the vault opened")
    };
    let bp = m.join("traces/base");
    for (bad, want) in [
        ("x\n", "not a trace base"),
        ("03\n", "not a trace base"),
        (
            &format!("{}\n", base + 1)[..],
            "does not match the trace archive",
        ),
        (&format!("{}", base)[..], "not a trace base"),
    ] {
        std::fs::write(&bp, bad).unwrap();
        assert!(open_err().contains(want), "{bad:?}: {}", open_err());
    }
    std::fs::remove_file(&bp).unwrap();
    assert!(open_err().contains("does not match the trace archive"));
    std::fs::write(&bp, format!("{base}\n")).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
    // an archive file gone, or one more
    let tmp = m.join("moved.jsonl");
    std::fs::rename(&arch, &tmp).unwrap();
    // (its filters file names it: "… has no trace archive file …")
    assert!(open_err().contains("trace archive"), "{}", open_err());
    std::fs::rename(&tmp, &arch).unwrap();
    std::fs::write(m.join("traces/archive/notes.txt"), "x").unwrap();
    assert!(open_err().contains("not a trace archive file"));
    std::fs::remove_file(m.join("traces/archive/notes.txt")).unwrap();
    // a trace line changed in the archive: the archived reader refuses it
    let orig = std::fs::read(&arch).unwrap();
    let text = String::from_utf8(orig.clone()).unwrap();
    std::fs::write(&arch, text.replacen("\"at_seq\":0", "\"at_seq\":7", 1)).unwrap();
    let e = err_text(&["trace", "list", vs, "--archived"]);
    assert!(e.contains("at_seq"), "{e}");
    std::fs::write(&arch, &orig).unwrap();
    // a checkpoint file deleted by hand (archived or live), or in both places
    let ap = m.join(format!("checkpoints/archive/{cp}.json"));
    let bytes = std::fs::read(&ap).unwrap();
    std::fs::remove_file(&ap).unwrap();
    assert!(open_err().contains("has no file"), "{}", open_err());
    std::fs::write(&ap, &bytes).unwrap();
    std::fs::write(m.join(format!("checkpoints/{cp}.json")), &bytes).unwrap();
    assert!(open_err().contains("in checkpoints/ and in checkpoints/archive/"));
    std::fs::remove_file(m.join(format!("checkpoints/{cp}.json"))).unwrap();
    std::fs::write(m.join("checkpoints/archive/0000.json"), "{}").unwrap();
    assert!(open_err().contains("not the file of a logged checkpoint"));
    std::fs::remove_file(m.join("checkpoints/archive/0000.json")).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
    // the live traces' first at_seq no longer the base
    let tp = m.join("traces.jsonl");
    let live = std::fs::read(&tp).unwrap();
    let t = String::from_utf8(live.clone()).unwrap();
    std::fs::write(
        &tp,
        t.lines()
            .skip(1)
            .map(|l| format!("{l}\n"))
            .collect::<String>(),
    )
    .unwrap();
    assert!(open_err().contains("trace base"), "{}", open_err());
    std::fs::write(&tp, &live).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
}

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
            "--watch",
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

fn http(s: &Served, tool: &str, body: &Value, consumer: Option<&str>) -> (u16, Value) {
    let body = body.to_string();
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    let xc = consumer.map_or(String::new(), |c| format!("X-Consumer: {c}\r\n"));
    write!(
        c,
        "POST /v1/{tool} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\n{xc}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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

const RULE: &str = "\n[prune]\nversion = \"r2\"\n\n[[prune.rules]]\nid = \"stale\"\nlayer = \"proposal\"\nidle_checkpoints = 1\n";

#[test]
fn only_an_explicit_human_call_prunes() {
    let v = scene("http", 3);
    let vs = v.to_str().unwrap();
    let f = v.parent().unwrap().join("p.md");
    std::fs::write(&f, "# Agent\n\nan agent note about wombats\n").unwrap();
    ok_json(&[
        "propose",
        vs,
        "agents/p.md",
        f.to_str().unwrap(),
        "--source",
        "bot",
    ]);
    ok_json(&["checkpoint", "create", vs]);
    // MCP has no prune: not listed, an unknown tool
    assert!(!rootmarkserver::tools::specs()
        .iter()
        .any(|s| s.name == "prune"));
    let k = Kernel::open(&v).unwrap();
    let e = k
        .call("prune", &json!({}), Caller::mcp(Some("agent/1")))
        .unwrap_err();
    assert_eq!(code_of(&e), Code::NotFound, "{e:#}");
    // without human_id: 403 denied, nothing written
    set_recipe(&v, false, &format!("{RETENTION}{RULE}"));
    let before = files(&v);
    let s = serve_http(&v);
    let (st, body) = http(&s, "prune", &json!({}), Some("bot"));
    assert_eq!(st, 403, "{body}");
    assert_eq!(body["error"]["code"], "denied");
    drop(s);
    let mut b = before.clone();
    b.retain(|p, _| !p.starts_with("index/"));
    let mut a = files(&v);
    a.retain(|p, _| !p.starts_with("index/"));
    assert_eq!(a, b);
    // with it: the prune, its Demoted carrying the consumer as meta.source
    set_recipe(&v, true, &format!("{RETENTION}{RULE}"));
    let s = serve_http(&v);
    let (st, body) = http(
        &s,
        "prune",
        &json!({"dry_run": false, "now_ms": 0}),
        Some("telegram-bot"),
    );
    assert_eq!(st, 200, "{body}");
    assert_eq!(
        body["rules"]["pruned"].as_array().unwrap().len(),
        2,
        "{body}"
    );
    assert!(body["retention"]["traces"]["lines"].as_u64().unwrap() > 0);
    // unknown argument: invalid_arguments
    let (st, body2) = http(&s, "prune", &json!({"dry": true}), Some("bot"));
    assert_eq!(st, 400, "{body2}");
    // status.retention
    let (_, status) = http(&s, "status", &json!({}), Some("bot"));
    let r = &status["retention"];
    assert_eq!(r["keep_checkpoints"], 2);
    assert_eq!(r["traces_base_at_seq"], body["retention"]["traces"]["base"]);
    assert_eq!(
        r["archived_checkpoints"],
        body["retention"]["checkpoints"].as_array().unwrap().len()
    );
    assert_eq!(r["rules"], json!(["stale"]));
    drop(s);
    let log = std::fs::read_to_string(meta(&v).join("events.jsonl")).unwrap();
    let demoted: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .filter(|e| e["kind"] == "Demoted")
        .collect();
    assert_eq!(demoted.len(), 2);
    for e in &demoted {
        assert_eq!(e["meta"]["source"], "telegram-bot");
        assert_eq!(e["meta"]["actor"]["kind"], "policy");
    }
}

#[test]
fn a_long_serve_session_prunes_nothing() {
    let v = scene("serve", 4);
    set_recipe(&v, true, &format!("{RETENTION}{RULE}"));
    let f = v.parent().unwrap().join("p.md");
    std::fs::write(&f, "# Agent\n\nan agent note about wombats\n").unwrap();
    ok_json(&[
        "propose",
        v.to_str().unwrap(),
        "agents/p.md",
        f.to_str().unwrap(),
        "--source",
        "bot",
    ]);
    let before = files(&v);
    let s = serve_http(&v);
    for i in 0..60 {
        let (st, _) = http(
            &s,
            "search",
            &json!({"query": format!("alpha {i}")}),
            Some("bot"),
        );
        assert_eq!(st, 200);
        if i % 10 == 0 {
            std::fs::write(v.join("b/c.md"), format!("# Beta\n\nalpha beta {i}\n")).unwrap();
            std::thread::sleep(Duration::from_millis(400));
            let (st, _) = http(&s, "checkpoint", &json!({}), Some("bot"));
            assert_eq!(st, 200);
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    drop(s);
    let after = files(&v);
    assert!(!meta(&v).join("traces/archive").exists());
    assert!(!meta(&v).join("checkpoints/archive").exists());
    assert!(!meta(&v).join("traces/base").exists());
    // nothing that was there is gone or changed, but for the appended logs
    // and the index
    for (p, b) in &before {
        if p.starts_with("index/") {
            continue;
        }
        let a = after.get(p).unwrap_or_else(|| panic!("{p} is gone"));
        if ["events.jsonl", "traces.jsonl", "trace_filters.jsonl"].contains(&p.as_str()) {
            assert!(a.starts_with(b), "{p} was rewritten");
        } else {
            assert_eq!(a, b, "{p} changed");
        }
    }
    let log = std::fs::read_to_string(meta(&v).join("events.jsonl")).unwrap();
    assert!(!log.contains("\"prune\":true"), "serve pruned on its own");
}

#[test]
fn the_same_vault_and_call_prune_alike() {
    let a = scene("det-a", 6);
    let b = scene("det-b", 6);
    for v in [&a, &b] {
        set_recipe(v, true, &format!("{RETENTION}{RULE}"));
    }
    let oa = Command::new(rootmark())
        .args(["prune", a.to_str().unwrap(), "--now-ms", "86400000"])
        .output()
        .unwrap();
    let ob = Command::new(rootmark())
        .args(["prune", b.to_str().unwrap(), "--now-ms", "86400000"])
        .output()
        .unwrap();
    assert!(
        oa.status.success(),
        "{}",
        String::from_utf8_lossy(&oa.stderr)
    );
    assert_eq!(oa.stdout, ob.stdout);
    let keep = |v: &Path| {
        let mut f = files(v);
        f.retain(|p, _| !p.starts_with("index/") && p != "policy.toml");
        f
    };
    assert_eq!(keep(&a), keep(&b));
}
