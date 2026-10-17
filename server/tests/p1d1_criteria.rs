//! P1-D.1 criteria D1-1..D1-9 (GOAL; TASKS D1.6). With
//! `ROOTMARK_P1D1_OUT=<dir>` each test writes `<dir>/d1_<n>.json` **before** it
//! asserts its threshold, so a failing criterion is recorded, not lost
//! (`scripts/test_p1d1.sh` collects them). Needs the model files.
//!
//! - D1-1: six `replace_section` requests touching canonical blocks under
//!   `append_only` → 6/6 `denied`, files and `events.jsonl` unchanged
//! - D1-2: `replace_section` of a section made of proposals → accepted
//! - D1-3: 500 deterministic pseudo-random agent writes (create, append,
//!   replace_section) → canonical results of 200 queries × 3 modes equal
//!   (bytes) to those before the writes
//! - D1-4: `unrestricted` deletion → `BlockDeleted.meta.actor` = the agent;
//!   `history` shows it
//! - D1-5: the same 100 calls (40 `propose`) pipelined to two identical
//!   vaults → `events.jsonl`, `traces.jsonl`, `.md` files equal (bytes)
//! - D1-6: 100 pipelined read-only calls → 100/100 the lock-step answers;
//!   pipelined time / serial time (information)
//! - D1-7: HTTP promote (`human_id` set) → `Promoted` actor `human:<id>`;
//!   MCP promote → `denied`; HTTP write without `human_id` → 403
//! - D1-8: 12 bad inputs (P1-D VERIFICATION V2/V3) × MCP and HTTP → the
//!   expected code (24)
//! - D1-9: naming the unnamed checkpoint; a taken name → `exists`

use rootmarkserver::error::code_of;
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

fn out(name: &str, v: &Value) {
    if let Ok(d) = std::env::var("ROOTMARK_P1D1_OUT") {
        let p = Path::new(&d).join(format!("{name}.json"));
        std::fs::write(&p, serde_json::to_string_pretty(v).unwrap() + "\n").unwrap();
    }
}

/// mini_vault (human content) ingested and indexed; `extra` documents too.
fn vault(name: &str, extra: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p1d1c-{}-{name}", std::process::id()));
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
    for (doc, text) in extra {
        std::fs::write(v.join(doc), text).unwrap();
    }
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn recipe(v: &Path, edit: &str, human_id: Option<&str>) {
    let mut r = rootmarkserver::config::default_recipe()
        .replace("policy = \"append_only\"", &format!("policy = \"{edit}\""));
    if let Some(h) = human_id {
        r.push_str(&format!("\n[identity]\nhuman_id = \"{h}\"\n"));
    }
    std::fs::write(v.join(".rootmark/policy.toml"), r).unwrap();
}

fn read(v: &Path, rel: &str) -> Vec<u8> {
    std::fs::read(v.join(rel)).unwrap_or_default()
}

fn md_files(v: &Path) -> Vec<(String, Vec<u8>)> {
    let mut o: Vec<(String, Vec<u8>)> = rootmarkcore::files::markdown_files(v)
        .unwrap()
        .into_iter()
        .map(|(d, p)| (d, std::fs::read(p).unwrap()))
        .collect();
    o.sort();
    o
}

fn agent() -> Caller<'static> {
    Caller::mcp(Some("criteria-agent/1.0"))
}

const SAFETY: &str = "# Safety\n\nIntro written by a human.\n\n## Fire\n\nKeep a class D extinguisher next to the roaster.\n\nNever leave a roast unattended.\n\n### Drills\n\nA fire drill every quarter.\n\n## Water\n\nDescale the boiler monthly.\n";

// ---- D1-1, D1-2 -------------------------------------------------------------

/// The D1-1 document: human sections, one of them (`Notes`) only a heading.
const GUIDE: &str = "# Safety\n\nIntro written by a human.\n\n## Fire\n\nKeep a class D extinguisher next to the roaster.\n\nNever leave a roast unattended.\n\n### Drills\n\nA fire drill every quarter.\n\n## Notes\n\n## Water\n\nDescale the boiler monthly.\n";

#[test]
fn d1_1_and_d1_2_replace_section_under_append_only() {
    let v = vault("d1-1", &[("guide.md", GUIDE)]);
    let k = Kernel::open(&v).unwrap();
    // setup: an agent subsection inside the human `Water` section, accepted
    // under append_only (its own heading, its own chunk): `Water` now holds
    // canonical and proposal blocks
    let setup = k
        .call(
            "propose",
            &json!({"doc_path": "guide.md", "mode": "append",
                    "content": "### Agent water\n\nAgent water note.", "reason": "D1-1 setup"}),
            agent(),
        )
        .unwrap();
    let water: Vec<Value> = setup["blocks"].as_array().unwrap()
        [setup["blocks"].as_array().unwrap().len() - 4..]
        .iter()
        .map(|b| b["layer"].clone())
        .collect();
    let ev0 = read(&v, ".rootmark/events.jsonl");
    let md0 = md_files(&v);
    let pr0 = read(&v, ".rootmark/proposals.jsonl");
    // (case, heading, content, what the targeted section holds)
    let cases = [
        (
            "full section",
            "Fire",
            "Agents rewrite the fire rules.",
            "canonical paragraphs and a canonical subsection, all replaced",
        ),
        (
            "partial",
            "Drills",
            "A fire drill every quarter.\n\nAnd one every month, says the agent.",
            "a canonical paragraph kept verbatim plus agent text",
        ),
        (
            "heading",
            "Notes",
            "Agent notes under a human heading.",
            "only a canonical heading (empty body)",
        ),
        (
            "mixed canonical+proposal",
            "Water",
            "Descale the boiler weekly.",
            "canonical heading and paragraph plus the agent's proposal subsection",
        ),
        (
            "nested heading",
            "Fire",
            "### Drills\n\nA fire drill every quarter.\n\n### Agent drills\n\nMore drills.",
            "a canonical section, replaced by text with nested headings",
        ),
        (
            "whole human document",
            "Safety",
            "Replaced by an agent.",
            "the whole human document",
        ),
    ];
    let mut results = Vec::new();
    for (name, heading, content, holds) in cases {
        let r = k.call(
            "propose",
            &json!({"doc_path": "guide.md", "mode": "replace_section", "heading": heading,
                    "content": content, "reason": "D1-1"}),
            agent(),
        );
        let code = r.as_ref().err().map(|e| code_of(e).as_str());
        let why = r.as_ref().err().map(|e| format!("{e:#}"));
        results.push(
            json!({"case": name, "heading": heading, "section_holds": holds,
                            "code": code, "reason": why}),
        );
    }
    let unchanged = read(&v, ".rootmark/events.jsonl") == ev0
        && md_files(&v) == md0
        && read(&v, ".rootmark/proposals.jsonl") == pr0;
    let denied = results.iter().filter(|r| r["code"] == "denied").count();
    let d11 = json!({"criterion": "D1-1", "water_layers_after_setup": water,
                     "cases": results, "denied": denied,
                     "files_and_events_unchanged": unchanged,
                     "pass": denied == 6 && unchanged
                         && water == vec![json!("canonical"), json!("canonical"), json!("proposal"), json!("proposal")]});
    out("d1_1", &d11);
    // D1-2: an agent section (own heading), then its replacement
    let a = k
        .call(
            "propose",
            &json!({"doc_path": "guide.md", "mode": "append", "content": "## Agent notes\n\nFirst agent text.",
                    "reason": "D1-2 setup"}),
            agent(),
        )
        .unwrap();
    let r = k.call(
        "propose",
        &json!({"doc_path": "guide.md", "mode": "replace_section", "heading": "Agent notes",
                "content": "Corrected agent text.", "reason": "D1-2"}),
        agent(),
    );
    let (ok, layers) = match &r {
        Ok(x) => {
            let b = x["blocks"].as_array().unwrap();
            let tail: Vec<Value> = b[b.len() - 2..]
                .iter()
                .map(|x| x["layer"].clone())
                .collect();
            (true, tail)
        }
        Err(_) => (false, Vec::new()),
    };
    let text = String::from_utf8(read(&v, "guide.md")).unwrap();
    let d12 = json!({"criterion": "D1-2", "setup_events": a["events"], "accepted": ok,
        "code": r.as_ref().err().map(|e| code_of(e).as_str()), "section_layers": layers,
        "file_ends_with_new_text": text.ends_with("## Agent notes\n\nCorrected agent text.\n"),
        "pass": ok && layers == vec![json!("proposal"), json!("proposal")]
            && text.ends_with("## Agent notes\n\nCorrected agent text.\n")});
    out("d1_2", &d12);
    assert_eq!(d11["pass"], true, "{d11}");
    assert_eq!(d12["pass"], true, "{d12}");
}

// ---- D1-3 ---------------------------------------------------------------------

/// A small deterministic generator (no randomness enters any identifier).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.next() as usize) % xs.len()]
    }
}

fn words(v: &Path) -> Vec<String> {
    let mut w: Vec<String> = md_files(v)
        .iter()
        .flat_map(|(_, b)| {
            String::from_utf8_lossy(b)
                .split(|c: char| !c.is_ascii_alphabetic())
                .filter(|t| t.len() >= 4)
                .map(|t| t.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .collect();
    w.sort();
    w.dedup();
    w
}

fn headings(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let t = l.trim_start_matches('#');
            (l.starts_with('#') && t.starts_with(' ')).then(|| t.trim().to_string())
        })
        .collect()
}

fn canonical_results(v: &Path, queries: &[String], at: Option<&str>) -> Vec<Value> {
    let k = Kernel::open(v).unwrap();
    let mut outv = Vec::new();
    for mode in ["lexical", "flat", "hybrid"] {
        for q in queries {
            let mut a = json!({"query": q, "mode": mode, "k": 10});
            if let Some(at) = at {
                a["at"] = json!(at);
            }
            let r = k
                .call("search", &a, Caller::mcp(Some("d1-3-reader/1.0")))
                .unwrap();
            assert!(r["hits"]
                .as_array()
                .unwrap()
                .iter()
                .all(|h| h["layer"] == "canonical"));
            outv.push(json!({"mode": mode, "query": q, "hits": r["hits"]}));
        }
    }
    outv
}

#[test]
fn d1_3_canonical_invariance_under_500_agent_writes() {
    let v = vault("d1-3", &[]);
    let k = Kernel::open(&v).unwrap();
    let w = words(&v);
    let mut g = Lcg(20261016);
    let queries: Vec<String> = (0..200)
        .map(|_| format!("{} {}", g.pick(&w), g.pick(&w)))
        .collect();
    k.call("checkpoint", &json!({"name": "before"}), agent())
        .unwrap();
    let before = canonical_results(&v, &queries, Some("before"));
    let mut agent_docs: Vec<String> = Vec::new();
    let human_docs: Vec<String> = md_files(&v).into_iter().map(|(d, _)| d).collect();
    let mut stats = std::collections::BTreeMap::<String, u64>::new();
    let para = |g: &mut Lcg| -> String {
        (0..(3 + g.next() % 8))
            .map(|_| g.pick(&w).clone())
            .collect::<Vec<_>>()
            .join(" ")
            + "."
    };
    for n in 0..500 {
        let choice = g.next() % 3;
        let (mode, args) = match choice {
            0 => {
                let doc = format!("agent/w{n}.md");
                let content = format!(
                    "# Agent {n}\n\n{}\n\n## Part {n}\n\n{}",
                    para(&mut g),
                    para(&mut g)
                );
                ("create", json!({"doc_path": doc, "content": content}))
            }
            1 => {
                let all: Vec<String> = human_docs
                    .iter()
                    .chain(agent_docs.iter())
                    .cloned()
                    .collect();
                let doc = g.pick(&all).clone();
                let content = if g.next().is_multiple_of(2) {
                    format!("## Agent addition {n}\n\n{}", para(&mut g))
                } else {
                    para(&mut g)
                };
                ("append", json!({"doc_path": doc, "content": content}))
            }
            _ => {
                let all: Vec<String> = human_docs
                    .iter()
                    .chain(agent_docs.iter())
                    .cloned()
                    .collect();
                let doc = g.pick(&all).clone();
                let text = String::from_utf8(read(&v, &doc)).unwrap();
                let hs = headings(&text);
                if hs.is_empty() {
                    (
                        "replace_section",
                        json!({"doc_path": doc, "heading": "none", "content": para(&mut g)}),
                    )
                } else {
                    let h = g.pick(&hs).clone();
                    (
                        "replace_section",
                        json!({"doc_path": doc, "heading": h, "content": para(&mut g)}),
                    )
                }
            }
        };
        let mut a = args;
        a["mode"] = json!(mode);
        a["reason"] = json!("D1-3");
        let r = k.call("propose", &a, agent());
        let outcome = match &r {
            Ok(_) => "accepted".to_string(),
            Err(e) => code_of(e).as_str().to_string(),
        };
        if r.is_ok() && mode == "create" {
            agent_docs.push(a["doc_path"].as_str().unwrap().to_string());
        }
        *stats.entry(format!("{mode}:{outcome}")).or_insert(0) += 1;
    }
    let after = canonical_results(&v, &queries, None);
    let empty_before = before
        .iter()
        .filter(|r| r["hits"].as_array().is_none_or(|h| h.is_empty()))
        .count();
    // the agent's accepted writes are indexed: with include_proposal they are found
    let kp = Kernel::open(&v).unwrap();
    let mut proposal_hits = 0usize;
    for q in queries.iter().take(50) {
        let r = kp
            .call(
                "search",
                &json!({"query": q, "mode": "lexical", "k": 10, "include_proposal": true}),
                Caller::mcp(Some("d1-3-reader/1.0")),
            )
            .unwrap();
        proposal_hits += r["hits"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["layer"] == "proposal")
            .count();
    }
    let diffs: Vec<usize> = before
        .iter()
        .zip(&after)
        .enumerate()
        .filter(|(_, (a, b))| {
            serde_json::to_string(a).unwrap() != serde_json::to_string(b).unwrap()
        })
        .map(|(i, _)| i)
        .collect();
    let accepted: u64 = stats
        .iter()
        .filter(|(k, _)| k.ends_with(":accepted"))
        .map(|(_, n)| n)
        .sum();
    let res = json!({"criterion": "D1-3", "writes": 500, "outcomes": stats, "accepted": accepted,
        "queries": queries.len(), "modes": 3, "comparisons": before.len(),
        "differing": diffs.len(), "first_differing": diffs.first(),
        "proposal_docs_created": agent_docs.len(),
        "empty_canonical_results_before": empty_before,
        "proposal_hits_after_50_lexical_queries_with_include_proposal": proposal_hits,
        "before_hash": rootmarkcore::hash32(&serde_json::to_string(&before).unwrap()),
        "after_hash": rootmarkcore::hash32(&serde_json::to_string(&after).unwrap()),
        "pass": diffs.is_empty() && before.len() == 600 && accepted > 0 && proposal_hits > 0});
    out("d1_3", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- D1-4 ---------------------------------------------------------------------

#[test]
fn d1_4_unrestricted_deletion_names_the_agent() {
    let v = vault("d1-4", &[("guide.md", SAFETY)]);
    recipe(&v, "unrestricted", None);
    let k = Kernel::open(&v).unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "guide.md", "mode": "replace_section", "heading": "Water",
                    "content": "", "reason": "D1-4"}),
            agent(),
        )
        .unwrap();
    let events: Vec<Value> = String::from_utf8(read(&v, ".rootmark/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let del = events
        .iter()
        .rfind(|e| e["kind"] == "BlockDeleted")
        .cloned()
        .unwrap_or(Value::Null);
    let lineage = del["lineage_ids"][0].clone();
    let h = k
        .call("history", &json!({"lineage_id": lineage}), agent())
        .unwrap();
    let last = h["events"].as_array().unwrap().last().cloned().unwrap();
    let want = json!({"kind": "agent", "id": "criteria-agent/1.0"});
    let res = json!({"criterion": "D1-4", "edit_policy": r["edit_policy"], "deleted_lineage": lineage,
        "event_actor": del["meta"]["actor"], "event_source": del["meta"]["source"],
        "history_last_kind": last["kind"], "history_actor": last["meta"]["actor"],
        "pass": del["meta"]["actor"] == want && last["kind"] == "BlockDeleted" && last["meta"]["actor"] == want});
    out("d1_4", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- MCP drivers ------------------------------------------------------------

fn init_msgs(name: &str) -> Vec<Value> {
    vec![
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": name, "version": "1.0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    ]
}

fn spawn_mcp(v: &Path) -> Child {
    Command::new(rootmark())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// `pipelined`: every message written at once; else lock-step. Returns the
/// responses in request order and the time from the first call to the last
/// response (the handshake excluded).
fn mcp_run(v: &Path, name: &str, calls: &[Value], pipelined: bool) -> (Vec<Value>, f64) {
    let mut child = spawn_mcp(v);
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let next_response =
        |lines: &mut std::io::Lines<BufReader<std::process::ChildStdout>>| -> Value {
            loop {
                let r: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                if r.get("id").is_some() {
                    return r;
                }
            }
        };
    for m in init_msgs(name) {
        writeln!(stdin, "{m}").unwrap();
    }
    stdin.flush().unwrap();
    next_response(&mut lines);
    let msgs: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, c)| json!({"jsonrpc": "2.0", "id": i + 1, "method": "tools/call", "params": c}))
        .collect();
    let start = Instant::now();
    let mut resps: Vec<Value> = Vec::new();
    if pipelined {
        let mut buf = String::new();
        for m in &msgs {
            buf.push_str(&m.to_string());
            buf.push('\n');
        }
        stdin.write_all(buf.as_bytes()).unwrap();
        stdin.flush().unwrap();
        for _ in &msgs {
            resps.push(next_response(&mut lines));
        }
    } else {
        for m in &msgs {
            writeln!(stdin, "{m}").unwrap();
            stdin.flush().unwrap();
            resps.push(next_response(&mut lines));
        }
    }
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    drop(stdin);
    assert!(child.wait().unwrap().success());
    resps.sort_by_key(|r| r["id"].as_u64().unwrap());
    (resps, ms)
}

fn call(name: &str, args: Value) -> Value {
    json!({"name": name, "arguments": args})
}

// ---- D1-5 ---------------------------------------------------------------------

fn d1_5_calls() -> Vec<Value> {
    let mut c = Vec::new();
    for i in 0..40 {
        let p = match i % 5 {
            0 => {
                json!({"doc_path": format!("agent/p{i}.md"), "content": format!("# Plan {i}\n\nAgent plan {i} about grinders and water.")})
            }
            1 => {
                json!({"doc_path": format!("agent/p{}.md", i - 1), "mode": "append", "content": format!("## Step {i}\n\nGrind finer for step {i}.")})
            }
            2 => {
                json!({"doc_path": format!("agent/p{}.md", i - 2), "mode": "replace_section", "heading": format!("Step {}", i - 1), "content": format!("Grind coarser for step {i}.")})
            }
            3 => {
                json!({"doc_path": "espresso.md", "mode": "append", "content": format!("## Agent espresso note {i}\n\nPreinfusion idea {i}.")})
            }
            _ => {
                json!({"doc_path": "filter.md", "mode": "append", "content": format!("A plain line {i} that joins a human chunk.")})
            }
        };
        let mut p = p;
        p["reason"] = json!(format!("D1-5 {i}"));
        c.push(call("propose", p));
        c.push(call(
            "search",
            json!({"query": format!("grinders water {i}"), "include_proposal": i % 2 == 0}),
        ));
        if i % 2 == 0 {
            c.push(call("blame", json!({"doc_path": "espresso.md"})));
        }
    }
    c
}

#[test]
fn d1_5_pipelined_calls_give_identical_logs() {
    let calls = d1_5_calls();
    assert_eq!(calls.len(), 100);
    let a = vault("d1-5-a", &[]);
    let b = vault("d1-5-b", &[]);
    let c = vault("d1-5-lockstep", &[]);
    let (ra, ta) = mcp_run(&a, "d1-5", &calls, true);
    let (rb, _) = mcp_run(&b, "d1-5", &calls, true);
    let (rc, tc) = mcp_run(&c, "d1-5", &calls, false);
    let files = ["events.jsonl", "traces.jsonl", "proposals.jsonl"];
    let eq = |x: &Path, y: &Path| -> Value {
        let mut m = serde_json::Map::new();
        for f in files {
            m.insert(
                f.into(),
                json!(read(x, &format!(".rootmark/{f}")) == read(y, &format!(".rootmark/{f}"))),
            );
        }
        m.insert("md_files".into(), json!(md_files(x) == md_files(y)));
        Value::Object(m)
    };
    let ab = eq(&a, &b);
    let ac = eq(&a, &c);
    let all_true = |m: &Value| m.as_object().unwrap().values().all(|x| x == true);
    // the responses of the writing and trace-writing calls (propose,
    // search): deterministic by contract §5 (one FIFO queue, arrival order);
    // the read-only blame runs in parallel and may not see an earlier write,
    // so `responses_*_equal` (every response) is informational (P1-E/E0, K-E0.2)
    let writes = |r: &[Value]| -> Vec<Value> {
        r.iter()
            .zip(&calls)
            .filter(|(_, c)| c["name"] == "propose" || c["name"] == "search")
            .map(|(r, _)| r.clone())
            .collect()
    };
    let proposes: Vec<&Value> = ra
        .iter()
        .zip(&calls)
        .filter(|(_, c)| c["name"] == "propose")
        .map(|(r, _)| r)
        .collect();
    let denied = proposes
        .iter()
        .filter(|r| r["result"]["structuredContent"]["error"]["code"] == "denied")
        .count();
    let res = json!({"criterion": "D1-5", "calls": calls.len(), "propose": proposes.len(),
        "propose_denied": denied, "traces": String::from_utf8(read(&a, ".rootmark/traces.jsonl")).unwrap().lines().count(),
        "pipelined_a_vs_b": ab, "pipelined_vs_lockstep": ac,
        "responses_a_vs_b_equal": ra == rb, "responses_pipelined_vs_lockstep_equal": ra == rc,
        "write_responses": writes(&ra).len(),
        "write_responses_a_vs_b_equal": writes(&ra) == writes(&rb),
        "write_responses_pipelined_vs_lockstep_equal": writes(&ra) == writes(&rc),
        "elapsed_ms_pipelined": ta, "elapsed_ms_lockstep": tc,
        "events_hash32": rootmarkcore::hash32(&String::from_utf8(read(&a, ".rootmark/events.jsonl")).unwrap()),
        "traces_hash32": rootmarkcore::hash32(&String::from_utf8(read(&a, ".rootmark/traces.jsonl")).unwrap()),
        "pass": all_true(&ab) && proposes.len() == 40});
    out("d1_5", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- D1-6 ---------------------------------------------------------------------

#[test]
fn d1_6_read_only_calls_run_in_parallel() {
    let v = vault("d1-6", &[]);
    let k = Kernel::open(&v).unwrap();
    let s = k
        .call("search", &json!({"query": "espresso"}), agent())
        .unwrap();
    let lineage = s["hits"][0]["lineage_ids"][0].clone();
    drop(k);
    let docs: Vec<String> = md_files(&v).into_iter().map(|(d, _)| d).collect();
    let mut calls = Vec::new();
    for i in 0..100 {
        calls.push(match i % 5 {
            0 => call("blame", json!({"doc_path": docs[i % docs.len()]})),
            1 => call("history", json!({"lineage_id": lineage})),
            2 => call("checkpoints", json!({})),
            3 => call("trace", json!({})),
            _ => call("blame", json!({"doc_path": docs[(i * 7) % docs.len()]})),
        });
    }
    let ev0 = read(&v, ".rootmark/events.jsonl");
    let tr0 = read(&v, ".rootmark/traces.jsonl");
    let (serial, ts) = mcp_run(&v, "d1-6", &calls, false);
    let (piped, tp) = mcp_run(&v, "d1-6", &calls, true);
    let correct = serial
        .iter()
        .zip(&piped)
        .filter(|(a, b)| a == b && a["result"]["isError"] != true)
        .count();
    let unchanged =
        read(&v, ".rootmark/events.jsonl") == ev0 && read(&v, ".rootmark/traces.jsonl") == tr0;
    let res = json!({"criterion": "D1-6", "calls": calls.len(), "correct": correct,
        "log_and_traces_unchanged": unchanged,
        "elapsed_ms_serial": ts, "elapsed_ms_pipelined": tp, "ratio": tp / ts,
        "ratio_below_0_6": tp / ts < 0.6,
        "pass": correct == 100 && unchanged});
    out("d1_6", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- HTTP -----------------------------------------------------------------------

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
        assert!(start.elapsed() < Duration::from_secs(120));
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

fn http(s: &Served, tool: &str, body: &str) -> (u16, Value) {
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    write!(
        c,
        "POST /v1/{tool} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nX-Consumer: criteria-http\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        s.token,
        body.len()
    )
    .unwrap();
    // a large body may be refused before it is read whole
    let _ = c.write_all(body.as_bytes());
    let mut resp = Vec::new();
    let _ = c.read_to_end(&mut resp);
    let resp = String::from_utf8_lossy(&resp).to_string();
    let status: u16 = resp.get(9..12).and_then(|x| x.parse().ok()).unwrap_or(0);
    let json = resp.split("\r\n\r\n").nth(1).unwrap_or("");
    (status, serde_json::from_str(json).unwrap_or(Value::Null))
}

// ---- D1-7 -------------------------------------------------------------------------

#[test]
fn d1_7_channel_decides_the_actor() {
    let v = vault("d1-7", &[]);
    let k = Kernel::open(&v).unwrap();
    let p = k
        .call(
            "propose",
            &json!({"doc_path": "agent/x.md", "content": "# X\n\nAgent text.", "reason": "D1-7"}),
            agent(),
        )
        .unwrap();
    let lineage = p["blocks"][1]["lineage_id"].as_str().unwrap().to_string();
    drop(k);
    // MCP promote: denied, no event
    let ev0 = read(&v, ".rootmark/events.jsonl");
    let (r, _) = mcp_run(
        &v,
        "d1-7",
        &[call(
            "promote",
            json!({"lineage_id": lineage, "reason": "agent wants it"}),
        )],
        false,
    );
    let mcp = &r[0]["result"]["structuredContent"];
    let mcp_ok = mcp["decision"] == "denied"
        && mcp["actor"]["kind"] == "agent"
        && read(&v, ".rootmark/events.jsonl") == ev0;
    // HTTP without human_id: write → 403
    let s = serve_http(&v);
    let (st, b) = http(
        &s,
        "promote",
        &json!({"lineage_id": lineage, "reason": "r"}).to_string(),
    );
    let no_id_ok = st == 403 && b["error"]["code"] == "denied";
    // HTTP with human_id: Promoted, actor human:<id>
    recipe(&v, "append_only", Some("editor-ada"));
    let (st2, b2) = http(
        &s,
        "promote",
        &json!({"lineage_id": lineage, "reason": "human approves"}).to_string(),
    );
    let promoted_in_log = String::from_utf8(read(&v, ".rootmark/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .any(|e| {
            e["kind"] == "Promoted"
                && e["lineage_ids"][0] == json!(lineage)
                && e["meta"]["actor"] == json!({"kind": "human", "id": "editor-ada"})
        });
    let human_ok = promoted_in_log
        && st2 == 200
        && b2["decision"] == "promoted"
        && b2["event"]["kind"] == "Promoted"
        && b2["event"]["meta"]["actor"] == json!({"kind": "human", "id": "editor-ada"});
    let res = json!({"criterion": "D1-7",
        "mcp_promote": {"decision": mcp["decision"], "actor": mcp["actor"], "pass": mcp_ok},
        "http_without_human_id": {"status": st, "code": b["error"]["code"], "pass": no_id_ok},
        "http_promote": {"status": st2, "decision": b2["decision"], "actor": b2["event"]["meta"]["actor"], "promoted_in_events_jsonl": promoted_in_log, "pass": human_ok},
        "passed": usize::from(mcp_ok) + usize::from(no_id_ok) + usize::from(human_ok),
        "pass": mcp_ok && no_id_ok && human_ok});
    out("d1_7", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- D1-8 -------------------------------------------------------------------------

#[test]
fn d1_8_error_codes_mcp_and_http() {
    let v = vault("d1-8", &[]);
    recipe(&v, "append_only", Some("editor-ada"));
    let big = "x".repeat(10 * 1024 * 1024 + 1);
    // P1-D VERIFICATION V2 (parameters) and V3 (propose): its 18 bad inputs
    // less six — the unknown tool (over MCP a JSON-RPC error, not a tool
    // error: CLIENT_CONTRACT §3 exception), `k: -1` (the class of `k: 0`),
    // and from V3 the symbolic link, `.txt`, `a/../../escape.md` and the
    // empty `reason` (the classes of kept cases: path_outside_vault,
    // invalid_arguments) — giving the GOAL's 12. The six left out are tested
    // elsewhere: `k: -1`, the symbolic link, `.txt` and the empty `reason` in
    // server/tests/p1d1_server.rs `error_codes_through_the_kernel`;
    // `..` paths in server/src/propose.rs `paths`; the unknown tool over HTTP
    // in `http_is_the_human_channel` (404 `not_found`)
    let cases: Vec<(&str, &str, Value, &str)> = vec![
        (
            "V2 search mode=bogus",
            "search",
            json!({"query": "x", "mode": "bogus"}),
            "invalid_arguments",
        ),
        (
            "V2 read_block without lineage_id",
            "read_block",
            json!({"doc_path": "espresso.md"}),
            "invalid_arguments",
        ),
        (
            "V2 history extra=1",
            "history",
            json!({"lineage_id": "x", "extra": 1}),
            "invalid_arguments",
        ),
        (
            "V2 search k=\"abc\"",
            "search",
            json!({"query": "x", "k": "abc"}),
            "invalid_arguments",
        ),
        (
            "V2 search k=0",
            "search",
            json!({"query": "x", "k": 0}),
            "invalid_arguments",
        ),
        (
            "V2 search query=\"\"",
            "search",
            json!({"query": ""}),
            "empty_query",
        ),
        (
            "V3 ../escape.md",
            "propose",
            json!({"doc_path": "../escape.md", "content": "x", "reason": "r"}),
            "path_outside_vault",
        ),
        (
            "V3 absolute path outside the vault",
            "propose",
            json!({"doc_path": "/tmp/rootmark-escape.md", "content": "x", "reason": "r"}),
            "path_outside_vault",
        ),
        (
            "V3 10 MB + 1 byte",
            "propose",
            json!({"doc_path": "big.md", "content": big, "reason": "r"}),
            "too_large",
        ),
        (
            "V3 replace_section missing heading",
            "propose",
            json!({"doc_path": "espresso.md", "mode": "replace_section", "heading": "No such heading", "content": "x", "reason": "r"}),
            "heading_not_found",
        ),
        (
            "V3 create over an existing file",
            "propose",
            json!({"doc_path": "espresso.md", "content": "x", "reason": "r"}),
            "exists",
        ),
        (
            "V3 .rootmark/evil.md",
            "propose",
            json!({"doc_path": ".rootmark/evil.md", "content": "x", "reason": "r"}),
            "path_outside_vault",
        ),
    ];
    assert_eq!(cases.len(), 12);
    let ev0 = read(&v, ".rootmark/events.jsonl");
    let calls: Vec<Value> = cases
        .iter()
        .map(|(_, t, a, _)| call(t, a.clone()))
        .collect();
    let (mcp, _) = mcp_run(&v, "d1-8", &calls, false);
    let s = serve_http(&v);
    let mut rows = Vec::new();
    let mut passed = 0;
    for ((name, tool, args, want), m) in cases.iter().zip(&mcp) {
        let mc = m["result"]["structuredContent"]["error"]["code"].clone();
        let mcp_ok = m["result"]["isError"] == true && mc == *want;
        let (st, b) = http(&s, tool, &args.to_string());
        let hc = b["error"]["code"].clone();
        let status_want = match *want {
            "invalid_arguments" | "empty_query" | "path_outside_vault" => 400,
            "exists" => 409,
            "too_large" => 413,
            "heading_not_found" => 422,
            _ => 0,
        };
        let http_ok = hc == *want && st == status_want;
        passed += usize::from(mcp_ok) + usize::from(http_ok);
        rows.push(json!({"case": name, "tool": tool, "expected": want,
            "mcp": {"code": mc, "pass": mcp_ok}, "http": {"status": st, "code": hc, "pass": http_ok}}));
    }
    drop(s);
    let unchanged =
        read(&v, ".rootmark/events.jsonl") == ev0 && !Path::new("/tmp/rootmark-escape.md").exists();
    let res = json!({"criterion": "D1-8", "cases": rows, "passed": passed, "of": 24,
        "events_unchanged": unchanged, "pass": passed == 24 && unchanged});
    out("d1_8", &res);
    assert_eq!(res["pass"], true, "{res}");
}

// ---- D1-9 -------------------------------------------------------------------------

#[test]
fn d1_9_name_the_unnamed_checkpoint() {
    let v = vault("d1-9", &[]);
    let k = Kernel::open(&v).unwrap();
    let s = k
        .call("search", &json!({"query": "espresso"}), agent())
        .unwrap();
    let cid = s["checkpoint_id"].clone();
    let c = k.call("checkpoint", &json!({"name": "yesterday"}), agent());
    let named_ok = c
        .as_ref()
        .is_ok_and(|c| c["checkpoint_id"] == cid && c["name"] == "yesterday");
    // a later state, a taken name
    k.call(
        "propose",
        &json!({"doc_path": "agent/y.md", "content": "# Y\n\ntext", "reason": "D1-9"}),
        agent(),
    )
    .unwrap();
    let e = k.call("checkpoint", &json!({"name": "yesterday"}), agent());
    let code = e.as_ref().err().map(|e| code_of(e).as_str());
    let collision_ok = code == Some("exists");
    let at = k
        .call(
            "search",
            &json!({"query": "espresso", "at": "yesterday"}),
            agent(),
        )
        .unwrap();
    let res = json!({"criterion": "D1-9", "unnamed_checkpoint": cid,
        "named": c.as_ref().ok(), "named_pass": named_ok,
        "taken_name_code": code, "collision_pass": collision_ok,
        "search_at_name_checkpoint": at["checkpoint_id"],
        "passed": usize::from(named_ok) + usize::from(collision_ok),
        "pass": named_ok && collision_ok && at["checkpoint_id"] == cid});
    out("d1_9", &res);
    assert_eq!(res["pass"], true, "{res}");
}
