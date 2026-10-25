//! P2-P/P3 (GOAL P2-P 4; P-7): the channel label of a human's transition.
//! - HTTP `promote` (and `demote`) with `X-Consumer` writes `Promoted`
//!   (`Demoted`) with `meta.source` = that consumer; `history` shows it;
//!   an HTTP call without `X-Consumer` is refused as before (400) and writes
//!   nothing;
//! - CLI `rootmark promote --source` / `rootmark demote --source` write it,
//!   without the flag no field (as before);
//! - the reading rule (spec 2.0.3 §7): absent accepted — the logs of earlier
//!   builds (P2-0's three `v0.1.0` vaults) read as before; present, a
//!   non-empty string, else the vault does not open.
//!
//! Needs the model files (`make models`).

use serde_json::{json, Value};
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

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2p3-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("a.md"), "# A\n\nbir insan notu.\n").unwrap();
    let vs = v.to_str().unwrap();
    ok_json(&["ingest-dir", vs, vs]);
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        format!(
            "{}\n[identity]\nhuman_id = \"ada\"\n",
            rootmarkserver::config::default_recipe()
        ),
    )
    .unwrap();
    v
}

/// An agent's proposal (two lineages) to promote.
fn proposal(v: &Path) -> Vec<String> {
    let f = v.parent().unwrap().join("p.md");
    std::fs::write(&f, "# Öneri\n\najanın önerisi.\n").unwrap();
    let r = ok_json(&[
        "propose",
        v.to_str().unwrap(),
        "notes/p.md",
        f.to_str().unwrap(),
        "--source",
        "agent-x",
    ]);
    r["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["lineage_id"].as_str().unwrap().to_string())
        .collect()
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

/// `POST /v1/<tool>`; `consumer` `None`: no `X-Consumer` header.
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

fn log(v: &Path) -> Vec<u8> {
    std::fs::read(v.join(".rootmark/events.jsonl")).unwrap()
}

/// P-7: HTTP `promote` / `demote` carry `X-Consumer` as `meta.source`;
/// without the header the call is refused as before and writes nothing.
#[test]
fn http_transitions_carry_the_consumer() {
    let v = vault("http");
    let ls = proposal(&v);
    let s = serve_http(&v);
    // no X-Consumer: 400 as before, nothing written
    let before = log(&v);
    let (st, body) = http(
        &s,
        "promote",
        &json!({"lineage_id": ls[0], "reason": "ok"}),
        None,
    );
    assert_eq!(st, 400, "{body}");
    assert_eq!(log(&v), before);
    // with it: meta.source = the consumer
    let (st, r) = http(
        &s,
        "promote",
        &json!({"lineage_id": ls[0], "reason": "uzaktan onay"}),
        Some("telegram-bot"),
    );
    assert_eq!(st, 200, "{r}");
    assert_eq!(r["decision"], "promoted");
    let m = &r["event"]["meta"];
    assert_eq!(m["source"], "telegram-bot");
    assert_eq!(m["actor"], json!({"kind": "human", "id": "ada"}));
    assert_eq!(m["reason"], "uzaktan onay");
    // history shows it
    let (_, h) = http(&s, "history", &json!({"lineage_id": ls[0]}), Some("reader"));
    let last = h["events"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["kind"], "Promoted");
    assert_eq!(last["meta"]["source"], "telegram-bot");
    // demote, the same
    let (st, d) = http(
        &s,
        "demote",
        &json!({"lineage_id": ls[0], "reason": "geri al"}),
        Some("telegram-bot"),
    );
    assert_eq!(st, 200, "{d}");
    assert_eq!(d["decision"], "demoted");
    assert_eq!(d["event"]["meta"]["source"], "telegram-bot");
    assert_eq!(d["layer"], "proposal");
    let before = log(&v);
    let (st, _) = http(
        &s,
        "demote",
        &json!({"lineage_id": ls[1], "reason": "x"}),
        None,
    );
    assert_eq!(st, 400);
    assert_eq!(log(&v), before);
    drop(s);
    rootmarkcore::Vault::open(&v).unwrap();
}

/// CLI `--source` writes the field; without it, no field (as before).
#[test]
fn cli_transitions_carry_source_only_when_given() {
    let v = vault("cli");
    let ls = proposal(&v);
    let vs = v.to_str().unwrap();
    let a = ok_json(&[
        "promote",
        vs,
        &ls[0],
        "--actor",
        "human:ada",
        "--reason",
        "ok",
    ]);
    assert!(a["event"]["meta"].get("source").is_none(), "{a}");
    let b = ok_json(&[
        "promote",
        vs,
        &ls[1],
        "--actor",
        "human:ada",
        "--reason",
        "ok",
        "--source",
        "terminal",
    ]);
    assert_eq!(b["event"]["meta"]["source"], "terminal");
    let c = ok_json(&[
        "demote",
        vs,
        &ls[1],
        "--actor",
        "human:ada",
        "--reason",
        "geri",
        "--source",
        "terminal",
    ]);
    assert_eq!(c["event"]["meta"]["source"], "terminal");
    // an empty label is refused before anything is written
    let before = log(&v);
    let o = Command::new(rootmark())
        .args([
            "demote",
            vs,
            &ls[0],
            "--actor",
            "human:ada",
            "--reason",
            "x",
            "--source",
            "",
        ])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("meta.source"));
    assert_eq!(log(&v), before);
    rootmarkcore::Vault::open(&v).unwrap();
}

/// The reading rule: absent accepted (earlier logs), present a non-empty
/// string, else the vault does not open.
#[test]
fn the_reading_rule_of_a_transition_s_source() {
    // earlier builds' logs: P2-0's three v0.1.0 vaults (their Promoted
    // events carry no source) open and keep their events_hash
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/data/p2_0_old_vaults");
    let mut transitions = 0;
    for name in ["e2e", "mixed", "scene"] {
        let want: Value = serde_json::from_str(
            &std::fs::read_to_string(data.join(name).join("expected.json")).unwrap(),
        )
        .unwrap();
        let root =
            std::env::temp_dir().join(format!("rootmark-p2p3-{}-old-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (from, to) in [("vault", root.clone()), ("store", root.join(".rootmark"))] {
            let o = Command::new("cp")
                .arg("-R")
                .arg(data.join(name).join(from))
                .arg(&to)
                .output()
                .unwrap();
            assert!(o.status.success());
        }
        let v = rootmarkcore::Vault::open(&root).unwrap();
        assert_eq!(json!(v.events_hash().unwrap()), want["events_hash"]);
        let evs = rootmarkcore::events::read_events(&root.join(".rootmark/events.jsonl")).unwrap();
        for e in evs.iter().filter(|e| {
            matches!(
                e.kind,
                rootmarkcore::EventKind::Promoted | rootmarkcore::EventKind::Demoted
            )
        }) {
            assert!(e.meta.get("source").is_none());
            transitions += 1;
        }
    }
    assert!(transitions > 0, "the old vaults hold transitions");
    // a written label, then damaged ones
    let v = vault("reading");
    let ls = proposal(&v);
    let vs = v.to_str().unwrap();
    ok_json(&[
        "promote",
        vs,
        &ls[0],
        "--actor",
        "human:ada",
        "--reason",
        "ok",
        "--source",
        "bot",
    ]);
    let p = v.join(".rootmark/events.jsonl");
    let good = std::fs::read_to_string(&p).unwrap();
    assert!(good.contains("\"source\":\"bot\""));
    for bad in ["\"source\":\"\"", "\"source\":7", "\"source\":null"] {
        std::fs::write(&p, good.replace("\"source\":\"bot\"", bad)).unwrap();
        let e = rootmarkcore::Vault::open(&v).err().expect(bad);
        assert!(format!("{e:#}").contains("meta.source"), "{bad}: {e:#}");
    }
    std::fs::write(&p, &good).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
}
