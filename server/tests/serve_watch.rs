//! D-10 (GOAL P1-D; TASKS D2): `mdk serve --watch`. A copy of
//! spec/fixtures/mini_vault is served (`--http 127.0.0.1:0 --watch`); 20
//! times a human paragraph with a new word is appended to a document from
//! outside the server, and the time until the HTTP `search` (lexical, at
//! the checkpoint of the current state) returns that document for the new
//! word is measured (P1-D/D3; in D2 the HTTP `blame` stood in for it). The
//! hit must be `canonical` and the new block's author `Human` (`blame`).
//! Pass: p95 < 1000 ms.
//! `MDK_D10_OUT` receives the latencies and the result. Then SIGTERM: the
//! server compacts and exits 0. Needs the model files (`make models`).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn mdk() -> &'static str {
    env!("CARGO_BIN_EXE_mdk")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-d10-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// One HTTP/1.1 request on a fresh connection: (status, JSON body).
fn http(port: u16, method: &str, path: &str, token: &str, body: &str) -> (u16, Value) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\n\
         X-Consumer: d10-test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
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

fn blame(port: u16, token: &str, doc: &str) -> Vec<Value> {
    let (st, v) = http(
        port,
        "POST",
        "/v1/blame",
        token,
        &json!({"doc_path": doc}).to_string(),
    );
    assert_eq!(st, 200, "{v}");
    v["blocks"].as_array().unwrap().clone()
}

fn p95(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((s.len() as f64) * 0.95).ceil() as usize - 1;
    s[i.min(s.len() - 1)]
}

#[test]
fn d10_watched_human_edit_is_indexed_within_a_second() {
    let root = tmp("vault");
    let vault = root.join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    let mini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault");
    for e in std::fs::read_dir(&mini).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "md") {
            std::fs::copy(&p, vault.join(p.file_name().unwrap())).unwrap();
        }
    }
    let vs = vault.to_str().unwrap();
    let st = Command::new(mdk())
        .args(["ingest-dir", vs, vs])
        .output()
        .unwrap();
    assert!(st.status.success());

    let mut child = Command::new(mdk())
        .args(["serve", "--vault", vs, "--http", "127.0.0.1:0", "--watch"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut err = BufReader::new(child.stderr.take().unwrap());
    let mut log = String::new();
    let port: u16 = loop {
        let mut l = String::new();
        assert!(err.read_line(&mut l).unwrap() > 0, "server exited: {log}");
        log.push_str(&l);
        if let Some(p) = l
            .trim()
            .strip_prefix("mdk serve: http listening on 127.0.0.1:")
        {
            break p.parse().unwrap();
        }
    };
    // wait for the watcher to be armed
    loop {
        let mut l = String::new();
        assert!(err.read_line(&mut l).unwrap() > 0, "server exited: {log}");
        log.push_str(&l);
        if l.contains("mdk serve: watching") {
            break;
        }
    }
    let rest = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    let token = std::fs::read_to_string(vault.join(".mdkernel/server.token"))
        .unwrap()
        .trim()
        .to_string();

    let docs = ["espresso.md", "filter.md", "roasting.md", "glossary.md"];
    let mut lat = Vec::new();
    let mut layers_ok = true;
    for i in 0..20 {
        let doc = docs[i % docs.len()];
        let word = format!("quetzal{i}");
        let p = vault.join(doc);
        let mut text = std::fs::read_to_string(&p).unwrap();
        text.push_str(&format!(
            "\nWatched human paragraph number {i} about {word} extraction.\n"
        ));
        let t0 = Instant::now();
        std::fs::write(&p, text).unwrap();
        // found by `search` (hot tier), at the checkpoint of the current state
        let hit = loop {
            let (st, v) = http(
                port,
                "POST",
                "/v1/search",
                &token,
                &json!({"query": word, "mode": "lexical", "k": 1}).to_string(),
            );
            assert_eq!(st, 200, "{v}");
            if let Some(h) = v["hits"].as_array().and_then(|h| h.first()) {
                if h["doc_path"] == doc {
                    break h.clone();
                }
            }
            assert!(
                t0.elapsed() < Duration::from_secs(10),
                "edit {i} of {doc} not searchable after 10 s"
            );
            std::thread::sleep(Duration::from_millis(5));
        };
        lat.push(t0.elapsed().as_secs_f64() * 1000.0);
        let blocks = blame(port, &token, doc);
        let last = blocks.last().unwrap();
        if hit["layer"] != "canonical"
            || last["layer"] != "canonical"
            || last["author_kind"] != "Human"
        {
            layers_ok = false;
        }
        // let the watcher's batch window close before the next edit
        std::thread::sleep(Duration::from_millis(120));
    }

    // SIGTERM → compact, exit 0
    let st = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(st.success());
    let code = child.wait().unwrap();
    let tail = rest.join().unwrap();
    let p95v = p95(&lat);
    let report = json!({
        "edits": lat.len(),
        "latencies_ms": lat,
        "p95_ms": p95v,
        "max_ms": lat.iter().cloned().fold(0.0, f64::max),
        "layers_canonical_human": layers_ok,
        "sigterm_exit_0": code.success(),
        "sigterm_compacted": tail.contains("compact (shutdown)"),
        "pass": p95v < 1000.0 && layers_ok && code.success() && tail.contains("compact (shutdown)"),
    });
    if let Ok(p) = std::env::var("MDK_D10_OUT") {
        if !p.is_empty() {
            std::fs::write(&p, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        }
    }
    assert_eq!(report["pass"], true, "{report}\n{log}{tail}");
    std::fs::remove_dir_all(root).unwrap();
}

/// D2 review 1: a SIGTERM that arrives while a timer compaction runs is not
/// lost. 162 documents (mini_vault + 150 generated), `compact_every_s = 1`,
/// a document edited every 100 ms from outside; in each of 10 trials one
/// SIGTERM is sent at a different moment and the server must compact and
/// exit 0 within 15 s, without a second signal.
#[test]
fn sigterm_is_never_lost_during_timer_compaction() {
    let root = tmp("sigterm");
    let vault = root.join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    let mini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault");
    for e in std::fs::read_dir(&mini).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "md") {
            std::fs::copy(&p, vault.join(p.file_name().unwrap())).unwrap();
        }
    }
    for i in 0..150 {
        let mut t = format!("# Generated note {i}\n\n");
        for j in 0..6 {
            t.push_str(&format!(
                "Paragraph {j} of note {i} about roast profile {}, grind {} and water {}.\n\n",
                i * 7 + j,
                (i + j) % 13,
                (i * j) % 17
            ));
        }
        std::fs::write(vault.join(format!("gen-{i:03}.md")), t).unwrap();
    }
    let vs = vault.to_str().unwrap();
    assert!(Command::new(mdk())
        .args(["ingest-dir", vs, vs])
        .output()
        .unwrap()
        .status
        .success());
    std::fs::create_dir_all(vault.join(".mdkernel")).unwrap();
    std::fs::write(
        vault.join(".mdkernel/policy.toml"),
        mdkserver::config::default_recipe().replace("compact_every_s = 300", "compact_every_s = 1"),
    )
    .unwrap();
    // index once (the first start builds it)
    assert!(Command::new(mdk())
        .args(["index", "build", vs])
        .output()
        .unwrap()
        .status
        .success());

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let editor = {
        let stop = stop.clone();
        let vault = vault.clone();
        std::thread::spawn(move || {
            let mut n = 0u32;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let p = vault.join(format!("gen-{:03}.md", n % 150));
                let mut t = std::fs::read_to_string(&p).unwrap();
                t.push_str(&format!("\nEdit {n} under load about tamarind{n}.\n"));
                std::fs::write(&p, t).unwrap();
                n += 1;
                std::thread::sleep(Duration::from_millis(100));
            }
        })
    };
    let mut outcomes = Vec::new();
    for trial in 0..10u64 {
        let mut child = Command::new(mdk())
            .args(["serve", "--vault", vs, "--watch"])
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let err = child.stderr.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let reader = std::thread::spawn(move || {
            let mut all = String::new();
            for l in BufReader::new(err).lines() {
                let l = l.unwrap_or_default();
                let _ = tx.send(l.clone());
                all.push_str(&l);
                all.push('\n');
            }
            all
        });
        loop {
            let l = rx
                .recv_timeout(Duration::from_secs(60))
                .expect("server start");
            if l.contains("mdk serve: watching") {
                break;
            }
        }
        // different moments relative to the 1 s compaction timer
        std::thread::sleep(Duration::from_millis(1000 + trial * 137));
        assert!(Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap()
            .success());
        let t0 = Instant::now();
        let exited = loop {
            if let Some(st) = child.try_wait().unwrap() {
                break Some(st);
            }
            if t0.elapsed() > Duration::from_secs(15) {
                break None;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        if exited.is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let log = reader.join().unwrap();
        outcomes.push(json!({
            "trial": trial,
            "exited": exited.is_some(),
            "exit_0": exited.is_some_and(|s| s.success()),
            "compacted_on_shutdown": log.contains("compact (shutdown)"),
            "timer_compactions": log.matches("compact (timer)").count(),
        }));
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    editor.join().unwrap();
    let all_ok = outcomes
        .iter()
        .all(|o| o["exit_0"] == true && o["compacted_on_shutdown"] == true);
    let timer_ran = outcomes
        .iter()
        .map(|o| o["timer_compactions"].as_u64().unwrap())
        .sum::<u64>();
    let report = json!({"trials": outcomes, "timer_compactions": timer_ran, "pass": all_ok && timer_ran > 0});
    if let Ok(p) = std::env::var("MDK_SIGTERM_OUT") {
        if !p.is_empty() {
            std::fs::write(&p, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        }
    }
    assert_eq!(report["pass"], true, "{report}");
    std::fs::remove_dir_all(root).unwrap();
}
