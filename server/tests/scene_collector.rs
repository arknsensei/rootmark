//! P2-P/P4 (GOAL P2-P 5; P-3, P-6): the collector scene — an agent collects,
//! a rule makes the official sources `canonical` at once, a brief is traced.
//! Deterministic, no Obsidian:
//! 1. the recipe's rule: `agent:collector`, `source = "collector"`,
//!    `path_prefix = "sources/official/"`;
//! 2. the collector (`rootmark serve --mcp stdio`, clientInfo
//!    `collector/1.0`) proposes five documents, three under the prefix, two
//!    outside;
//! 3. default `search`: the three are `canonical`, the two appear only with
//!    `include_proposal` (labelled `proposal`);
//! 4. the brief's trace lists the canonical blocks it returned;
//! 5. `history` names the rule; 6. a human `demote`s one, search again;
//! 7. `checkpoint`; the whole scene twice: `events_hash`, the log, the trace
//!    log and the checkpoint id byte-equal.
//!
//! The same scene once through the CLI (`rootmark propose --source collector`).
//!
//! Needs the model files (`make models`).

use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

const RULE: &str = "\n[[promotion.rules]]\nid = \"official\"\nactor = \"agent:collector\"\nsource = \"collector\"\npath_prefix = \"sources/official/\"\nto = \"canonical\"\n";

/// The five documents: three official sources, two others.
const DOCS: [(&str, &str); 5] = [
    (
        "sources/official/tcmb-faiz.md",
        "# TCMB faiz kararı\n\nPolitika faizi yüzde elli sabit tutuldu.\n",
    ),
    (
        "sources/official/tuik-enflasyon.md",
        "# TÜİK enflasyon\n\nYıllık enflasyon yüzde kırk sekiz açıklandı.\n",
    ),
    (
        "sources/official/hazine-ihale.md",
        "# Hazine ihalesi\n\nHazine iki yıllık tahvil ihalesi düzenledi.\n",
    ),
    (
        "sources/news/piyasa-yorumu.md",
        "# Piyasa yorumu\n\nAnalistler faiz kararının enflasyonu düşüreceğini düşünüyor.\n",
    ),
    (
        "notes/llm-ozet.md",
        "# Özet\n\nFaiz ve enflasyon üzerine model özeti.\n",
    ),
];

fn vault(dir: &Path) -> PathBuf {
    let _ = std::fs::remove_dir_all(dir);
    let v = dir.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(
        v.join("README.md"),
        "# Hafıza\n\nToplayıcı ajanların vault'u.\n",
    )
    .unwrap();
    let vs = v.to_str().unwrap();
    let o = Command::new(rootmark())
        .args(["ingest-dir", vs, vs])
        .output()
        .unwrap();
    assert!(o.status.success());
    let recipe = rootmarkserver::config::default_recipe().replace(
        "[promotion]\npolicy = \"human_only\"\n",
        "[promotion]\npolicy = \"human_only\"\nversion = \"qorbit-1\"\n",
    );
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        format!("{recipe}\n[identity]\nhuman_id = \"ada\"\n{RULE}"),
    )
    .unwrap();
    v
}

/// One MCP session (`rootmark serve --mcp stdio`) as `<name>/1.0`: the tool
/// calls in order, their results.
fn mcp(v: &Path, name: &str, calls: &[(&str, Value)]) -> Vec<Value> {
    let mut child = Command::new(rootmark())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
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
    let mut send = |m: Value| {
        stdin.write_all(format!("{m}\n").as_bytes()).unwrap();
        stdin.flush().unwrap();
    };
    let wait = |id: u64| -> Value {
        loop {
            let l = rx
                .recv_timeout(Duration::from_secs(120))
                .expect("no MCP response");
            let r: Value = serde_json::from_str(&l).unwrap();
            if r["id"].as_u64() == Some(id) {
                return r;
            }
        }
    };
    send(
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": name, "version": "1.0"}}}),
    );
    wait(0);
    send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    let mut out = Vec::new();
    for (i, (tool, args)) in calls.iter().enumerate() {
        let id = i as u64 + 1;
        // one call at a time: the scene's order is the log's
        send(json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": tool, "arguments": args}}));
        let r = wait(id);
        let res = &r["result"];
        assert_ne!(res["isError"], true, "{tool}: {r}");
        let text = res["content"][0]["text"].as_str().unwrap();
        out.push(serde_json::from_str(text).unwrap());
    }
    // end of the MCP stream: the server stops
    drop(stdin);
    let _ = child.wait();
    out
}

fn hits(r: &Value) -> Vec<(String, String)> {
    r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            (
                h["doc_path"].as_str().unwrap().to_string(),
                h["layer"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn official(d: &str) -> bool {
    d.starts_with("sources/official/")
}

struct Run {
    events: Vec<u8>,
    traces: Vec<u8>,
    events_hash: String,
    checkpoint: String,
}

/// The scene; `cli`: the collector writes through `rootmark propose`.
fn scene(dir: &Path, cli: bool) -> Run {
    let v = vault(dir);
    let vs = v.to_str().unwrap();
    // 2. five proposals
    let proposed: Vec<Value> = if cli {
        DOCS.iter()
            .map(|(doc, text)| {
                let f = dir.join("in.md");
                std::fs::write(&f, text).unwrap();
                let o = Command::new(rootmark())
                    .args([
                        "propose",
                        vs,
                        doc,
                        f.to_str().unwrap(),
                        "--source",
                        "collector",
                        "--reason",
                        "collect",
                    ])
                    .output()
                    .unwrap();
                assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
                serde_json::from_slice(&o.stdout).unwrap()
            })
            .collect()
    } else {
        let calls: Vec<(&str, Value)> = DOCS
            .iter()
            .map(|(doc, text)| {
                (
                    "propose",
                    json!({"doc_path": doc, "content": text, "reason": "collect"}),
                )
            })
            .collect();
        mcp(&v, "collector", &calls)
    };
    let actor = if cli { "collector" } else { "collector/1.0" };
    for (r, (doc, _)) in proposed.iter().zip(DOCS) {
        assert_eq!(r["actor"], json!({"kind": "agent", "id": actor}), "{doc}");
        let p = r.get("promoted").and_then(Value::as_array);
        if official(doc) {
            let p = p.unwrap_or_else(|| panic!("{doc}: not promoted: {r}"));
            assert_eq!(p.len(), 2, "{doc}");
            for e in p {
                assert_eq!(
                    e["meta"]["actor"],
                    json!({"kind": "policy", "id": "official"})
                );
                assert_eq!(e["meta"]["reason"], "rule:official");
                assert_eq!(e["meta"]["policy_version"], "qorbit-1");
                assert_eq!(e["meta"]["from"], "proposal");
                assert_eq!(e["meta"]["to"], "canonical");
            }
        } else {
            assert!(p.is_none(), "{doc}: promoted outside the prefix");
        }
    }
    // 3.–5. the brief: default search, with proposals, the trace, history
    let q = json!({"query": "faiz enflasyon tahvil", "mode": "lexical", "k": 10});
    let mut qp = q.clone();
    qp["include_proposal"] = json!(true);
    let first_para = proposed[0]["blocks"][1]["lineage_id"].clone();
    let r = mcp(
        &v,
        "brief",
        &[
            ("search", q.clone()),
            ("search", qp.clone()),
            ("trace", json!({"consumer": "brief/1.0"})),
            ("history", json!({"lineage_id": first_para})),
        ],
    );
    let (default, with_p, trace, history) = (&r[0], &r[1], &r[2], &r[3]);
    let d = hits(default);
    assert!(
        d.iter().all(|(doc, l)| official(doc) && l == "canonical"),
        "{d:?}"
    );
    // the three official sources, each `canonical`
    let found: std::collections::BTreeSet<&str> = d.iter().map(|(doc, _)| doc.as_str()).collect();
    let want: std::collections::BTreeSet<&str> = DOCS
        .iter()
        .map(|(doc, _)| *doc)
        .filter(|doc| official(doc))
        .collect();
    assert_eq!(found, want, "{d:?}");
    let wp = hits(with_p);
    for (doc, _) in DOCS.iter().filter(|(d, _)| !official(d)) {
        assert!(
            wp.iter().any(|(x, l)| x == doc && l == "proposal"),
            "{doc} not in include_proposal: {wp:?}"
        );
        assert!(!d.iter().any(|(x, _)| x == doc));
    }
    // the trace of the default search lists canonical official blocks only
    let t0 = &trace["traces"][0];
    assert_eq!(t0["trace_id"], default["trace_id"]);
    let returned = t0["returned"].as_array().unwrap();
    assert!(!returned.is_empty());
    assert!(returned
        .iter()
        .all(|e| official(e["doc_path"].as_str().unwrap())));
    // history names the rule
    let last = history["events"].as_array().unwrap().last().unwrap();
    assert_eq!(last["kind"], "Promoted");
    assert_eq!(last["meta"]["reason"], "rule:official");
    // 6. a human demotes the first official source (both blocks)
    let k = Kernel::open(&v).unwrap();
    let human = Caller::http("telegram-bot");
    for b in proposed[0]["blocks"].as_array().unwrap() {
        let r = k
            .call(
                "demote",
                &json!({"lineage_id": b["lineage_id"], "reason": "kaynak doğrulanmadı"}),
                human,
            )
            .unwrap();
        assert_eq!(r["decision"], "demoted");
        assert_eq!(r["event"]["meta"]["source"], "telegram-bot");
    }
    drop(k);
    // 7. search again, and the checkpoint
    let r = mcp(
        &v,
        "brief",
        &[
            ("search", q),
            ("search", qp),
            ("checkpoint", json!({"name": "brief-1"})),
        ],
    );
    let d = hits(&r[0]);
    assert!(!d.iter().any(|(x, _)| x == DOCS[0].0), "{d:?}");
    assert!(hits(&r[1])
        .iter()
        .any(|(x, l)| x == DOCS[0].0 && l == "proposal"));
    let ev = v.join(".rootmark/events.jsonl");
    let events = std::fs::read(&ev).unwrap();
    let events_hash = rootmarkcore::Vault::open(&v)
        .unwrap()
        .events_hash()
        .unwrap();
    Run {
        events,
        traces: std::fs::read(v.join(".rootmark/traces.jsonl")).unwrap(),
        events_hash,
        checkpoint: r[2]["checkpoint_id"].as_str().unwrap().to_string(),
    }
}

fn dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("rootmark-p2p4-{}-{name}", std::process::id()))
}

/// P-3 (the scene) and P-6 (twice, byte-equal).
#[test]
fn the_collector_scene_twice_byte_equal() {
    let a = scene(&dir("a"), false);
    let b = scene(&dir("b"), false);
    assert_eq!(a.events_hash, b.events_hash);
    assert_eq!(a.events, b.events, "events.jsonl");
    assert_eq!(a.traces, b.traces, "traces.jsonl");
    assert_eq!(a.checkpoint, b.checkpoint);
    eprintln!(
        "scene: events_hash {} checkpoint {} traces {} bytes",
        a.events_hash,
        a.checkpoint,
        a.traces.len()
    );
}

/// The same scene through the CLI (`rootmark propose --source collector`).
#[test]
fn the_collector_scene_through_the_cli() {
    let c = scene(&dir("cli"), true);
    assert!(!c.events.is_empty());
}
