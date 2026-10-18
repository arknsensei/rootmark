//! P2-0/Z2 (spec 2.0.0 §7.1, §8; GOAL P2-0 "Tasarım"): attested writes in
//! the server.
//! - the default policy ingests an unattested external change (the watcher,
//!   `propose`'s sync) as `unverified`, actor `unattested_fs`; canonical
//!   content is never changed in place (its versions stay readable);
//! - an attestation within the window before the change → `canonical`;
//!   within the window after → `unverified`, then raised (`LayerChanged`,
//!   `cause: late_attestation`); beyond the window, another hash, another
//!   document → stays `unverified`; a proposal the change touched returns to
//!   `proposal`, never `canonical`;
//! - several quick saves, each attested; a rename (a new document);
//! - `unattested = "canonical"` keeps the 1.0.3 behaviour;
//! - attestation endpoints: HTTP only (MCP lists no `attest` and answers an
//!   unknown tool), a wrong MAC or session is `denied`, and `status` reports
//!   the policy;
//! - a vault written before 2.0.0 opens with every layer and its
//!   `events_hash` unchanged.
//!
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

const NOTES: &str =
    "# Notes\n\nA human paragraph about roasting.\n\n## Water\n\nDescale the boiler monthly.\n";

/// One human document ingested (canonical), index built, the default
/// recipe with a human id and `attestation` (a TOML table body, or "").
fn vault(name: &str, attestation: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2z2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("notes.md"), NOTES).unwrap();
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    let mut recipe = format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n",
        rootmarkserver::config::default_recipe()
    );
    if !attestation.is_empty() {
        recipe.push_str(&format!("\n[attestation]\n{attestation}\n"));
    }
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-0-test")
}

fn blame(k: &Kernel, doc: &str) -> Vec<Value> {
    k.call("blame", &json!({"doc_path": doc}), human()).unwrap()["blocks"]
        .as_array()
        .unwrap()
        .clone()
}

fn layers(k: &Kernel, doc: &str) -> Vec<String> {
    blame(k, doc)
        .iter()
        .map(|b| b["layer"].as_str().unwrap().to_string())
        .collect()
}

fn events(v: &Path) -> Vec<Value> {
    std::fs::read_to_string(v.join(".rootmark/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

struct Session {
    id: String,
    key: Vec<u8>,
}

fn session(k: &Kernel) -> Session {
    let s = k.call("attest_session", &json!({}), human()).unwrap();
    let key = s["key"].as_str().unwrap();
    Session {
        id: s["session_id"].as_str().unwrap().to_string(),
        key: (0..key.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&key[i..i + 2], 16).unwrap())
            .collect(),
    }
}

fn attest(k: &Kernel, s: &Session, doc: &str, bytes: &[u8]) -> anyhow::Result<Value> {
    let h = rootmarkcore::attest::content_hash(bytes);
    k.call(
        "attest",
        &json!({"doc_path": doc, "content_hash": h, "session_id": s.id,
                "mac": mac(&s.key, &mac_message(doc, &h, None))}),
        human(),
    )
}

/// Writes `text` to `doc` and runs the watcher's ingest of it.
fn save(k: &Kernel, v: &Path, doc: &str, text: &str) {
    let p = v.join(doc);
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).unwrap();
    }
    std::fs::write(&p, text).unwrap();
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
}

#[test]
fn unattested_change_is_unverified_by_default() {
    let v = vault("default", "");
    let k = Kernel::open(&v).unwrap();
    let before = blame(&k, "notes.md");
    let n_before = events(&v).len();
    // an agent writes the file directly: rewrites a canonical paragraph,
    // appends a new one
    let agent_text = "# Notes\n\nAn agent rewrote this paragraph about roasting entirely.\n\n## Water\n\nDescale the boiler monthly.\n\nAn agent paragraph appended.\n";
    save(&k, &v, "notes.md", agent_text);
    let after = blame(&k, "notes.md");
    // every new or changed block is unverified; untouched ones stay canonical
    let old_versions: Vec<&str> = before
        .iter()
        .map(|b| b["version_id"].as_str().unwrap())
        .collect();
    for b in &after {
        let changed = !old_versions.contains(&b["version_id"].as_str().unwrap());
        assert_eq!(
            b["layer"],
            if changed { "unverified" } else { "canonical" },
            "{b}"
        );
        if changed {
            assert_eq!(b["source"], "unattested_fs");
        }
    }
    let new: Vec<Value> = events(&v).into_iter().skip(n_before).collect();
    for e in &new {
        match e["kind"].as_str().unwrap() {
            "BlockUpdated" | "BlockDeleted" | "BlockSplit" | "BlockMerged" | "BlockMoved" => {
                assert_eq!(
                    e["meta"]["actor"],
                    json!({"kind": "unattested_fs", "id": "fs"})
                );
                assert_eq!(e["meta"]["source"], "unattested_fs");
            }
            _ => {}
        }
        if matches!(
            e["kind"].as_str().unwrap(),
            "BlockCreated" | "BlockUpdated" | "BlockSplit" | "BlockMerged"
        ) {
            assert_eq!(e["meta"]["layer"], "unverified", "{e}");
        }
    }
    // the changed canonical lineage dropped by LayerChanged; its old version
    // is still in its history
    let lc = new
        .iter()
        .find(|e| e["kind"] == "LayerChanged")
        .expect("a LayerChanged");
    assert_eq!(lc["meta"]["from"], "canonical");
    assert_eq!(lc["meta"]["to"], "unverified");
    let h = k
        .call(
            "history",
            &json!({"lineage_id": lc["lineage_ids"][0]}),
            human(),
        )
        .unwrap();
    assert!(h["events"].as_array().unwrap().len() >= 2);
    // search: the unverified content is not returned by default, labelled
    // with include_proposal
    let s = k
        .call(
            "search",
            &json!({"query": "agent paragraph appended", "mode": "lexical"}),
            human(),
        )
        .unwrap();
    assert!(s["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["layer"] == "canonical"));
    let s = k
        .call(
            "search",
            &json!({"query": "agent paragraph appended", "mode": "lexical", "include_proposal": true}),
            human(),
        )
        .unwrap();
    assert!(s["hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|h| h["layer"] == "unverified"));
    // the status reports the policy
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["attestation"]["unattested"], "unverified");
    assert_eq!(st["attestation"]["window_s"], 2.0);
    assert_eq!(st["attestation"]["stage"], 1);
}

#[test]
fn attestation_before_within_window_is_canonical() {
    let v = vault("before", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let text = format!("{NOTES}\nA human addition, attested.\n");
    let r = attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
    assert_eq!(r["recorded"], true);
    assert_eq!(r["raised"], json!([]));
    save(&k, &v, "notes.md", &text);
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    // the attestation log: one line, its id formula
    let log = std::fs::read_to_string(v.join(".rootmark/attestations.jsonl")).unwrap();
    let a: Value = serde_json::from_str(log.lines().next().unwrap()).unwrap();
    assert_eq!(
        a["attestation_id"],
        rootmarkcore::attest::attestation_id(
            "notes.md",
            a["content_hash"].as_str().unwrap(),
            a["at_seq"].as_u64().unwrap()
        )
    );
}

#[test]
fn late_attestation_raises_within_window_only() {
    // after, within W
    let v = vault("late", "window_s = 2.0");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let text = format!("{NOTES}\nA human addition, attested late.\n");
    save(&k, &v, "notes.md", &text);
    assert!(layers(&k, "notes.md").iter().any(|l| l == "unverified"));
    let r = attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
    assert_eq!(r["raised"].as_array().unwrap().len(), 1, "{r}");
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    let last = events(&v).pop().unwrap();
    assert_eq!(last["kind"], "LayerChanged");
    assert_eq!(last["meta"]["cause"], "late_attestation");
    assert_eq!(last["meta"]["attestation_id"], r["attestation_id"]);
    // the vault still opens (the late attestation is in the log)
    rootmarkcore::Vault::open(&v).unwrap();

    // after, beyond W
    let v = vault("beyond", "window_s = 0.1");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    save(&k, &v, "notes.md", &text);
    std::thread::sleep(Duration::from_millis(400));
    let r = attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
    assert_eq!(r["raised"], json!([]));
    assert!(layers(&k, "notes.md").iter().any(|l| l == "unverified"));

    // another hash, another document: no match
    let v = vault("mismatch", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    save(&k, &v, "notes.md", &text);
    let r = attest(&k, &s, "notes.md", b"other bytes").unwrap();
    assert_eq!(r["raised"], json!([]));
    std::fs::write(v.join("other.md"), "# Other\n").unwrap();
    let r = attest(&k, &s, "other.md", text.as_bytes()).unwrap();
    assert_eq!(r["raised"], json!([]));
    assert!(layers(&k, "notes.md").iter().any(|l| l == "unverified"));
}

#[test]
fn late_attestation_never_promotes_a_proposal() {
    let v = vault("proposal", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    // an agent proposal through MCP, then an unattested change of its block
    k.call(
        "propose",
        &json!({"doc_path": "notes.md", "mode": "append", "reason": "r",
                "content": "## Agent notes\n\nAn agent suggestion about grinders."}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    let text = std::fs::read_to_string(v.join("notes.md"))
        .unwrap()
        .replace(
            "An agent suggestion about grinders.",
            "An agent suggestion about grinders, edited by hand.",
        );
    save(&k, &v, "notes.md", &text);
    let b = blame(&k, "notes.md");
    assert_eq!(b.last().unwrap()["layer"], "unverified");
    attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
    let b = blame(&k, "notes.md");
    assert_eq!(b.last().unwrap()["layer"], "proposal", "{b:?}");
    let last = events(&v).pop().unwrap();
    assert_eq!(last["meta"]["to"], "proposal");
}

#[test]
fn quick_saves_and_a_rename() {
    let v = vault("quick", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    // five saves in a row, each attested before the watcher sees it
    let mut text = NOTES.to_string();
    for i in 0..5 {
        text.push_str(&format!("\nQuick human line {i}.\n"));
        attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
        save(&k, &v, "notes.md", &text);
    }
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    // a rename: the new path is a new document; attested → canonical
    std::fs::rename(v.join("notes.md"), v.join("renamed.md")).unwrap();
    attest(&k, &s, "renamed.md", text.as_bytes()).unwrap();
    k.watch_doc("renamed.md", "rootmark-serve-watch").unwrap();
    assert!(layers(&k, "renamed.md").iter().all(|l| l == "canonical"));
    // an unattested rename → unverified
    std::fs::rename(v.join("renamed.md"), v.join("moved.md")).unwrap();
    k.watch_doc("moved.md", "rootmark-serve-watch").unwrap();
    assert!(layers(&k, "moved.md").iter().all(|l| l == "unverified"));
}

#[test]
fn canonical_policy_keeps_the_1_0_3_behaviour() {
    let v = vault("canon", "unattested = \"canonical\"");
    let k = Kernel::open(&v).unwrap();
    save(
        &k,
        &v,
        "notes.md",
        &format!("{NOTES}\nAn unattested line.\n"),
    );
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["attestation"]["unattested"], "canonical");
}

#[test]
fn propose_sync_of_an_external_edit_is_unverified() {
    let v = vault("sync", "");
    let k = Kernel::open(&v).unwrap();
    // an edit nobody ingested, then an agent proposal to the same document
    std::fs::write(
        v.join("notes.md"),
        format!("{NOTES}\nAn offline line nobody attested.\n"),
    )
    .unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "notes.md", "mode": "append", "reason": "r",
                    "content": "## Agent\n\nAn agent line."}),
            Caller::mcp(Some("agent/1")),
        )
        .unwrap();
    assert!(r["synced_external_events"].as_u64().unwrap() > 0);
    let b = blame(&k, "notes.md");
    assert!(b
        .iter()
        .any(|x| x["layer"] == "unverified" && x["source"] == "unattested_fs"));
}

#[test]
fn attestation_endpoints_refuse_bad_macs_and_need_the_human() {
    let v = vault("mac", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let h = rootmarkcore::attest::content_hash(b"x");
    let call = |sid: &str, m: &str| {
        k.call(
            "attest",
            &json!({"doc_path": "notes.md", "content_hash": h, "session_id": sid, "mac": m}),
            human(),
        )
    };
    let good = mac(&s.key, &mac_message("notes.md", &h, None));
    assert_eq!(
        code_of(&call(&s.id, &"0".repeat(64)).unwrap_err()),
        Code::Denied
    );
    assert_eq!(code_of(&call("nope", &good).unwrap_err()), Code::Denied);
    call(&s.id, &good).unwrap();
    // a bad path or hash is invalid
    let bad = k.call(
        "attest",
        &json!({"doc_path": "../x.md", "content_hash": h, "session_id": s.id, "mac": good}),
        human(),
    );
    assert!(bad.is_err());
    // the agent channel has no attestation endpoint
    let e = k
        .call("attest_session", &json!({}), Caller::mcp(Some("agent/1")))
        .unwrap_err();
    assert_eq!(code_of(&e), Code::NotFound);
    // HTTP without [identity] human_id: denied
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe(),
    )
    .unwrap();
    let e = k.call("attest_session", &json!({}), human()).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
}

/// MCP over stdio: `tools/list` has no attestation tool and `tools/call
/// attest` is an unknown tool (JSON-RPC -32602; CLIENT_CONTRACT §3).
#[test]
fn mcp_has_no_attest_tool() {
    let v = vault("mcp", "");
    let mut child = Command::new(rootmark())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut send = |m: Value| {
        writeln!(stdin, "{m}").unwrap();
        stdin.flush().unwrap();
    };
    send(
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "p2-0", "version": "1.0"}}}),
    );
    send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    send(json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}));
    for (id, name) in [(2, "attest"), (3, "attest_session")] {
        send(json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": {"doc_path": "notes.md"}}}));
    }
    let mut seen = std::collections::BTreeMap::new();
    while seen.len() < 4 {
        let l = out.next().expect("mcp closed").unwrap();
        let m: Value = serde_json::from_str(&l).unwrap();
        if let Some(id) = m["id"].as_u64() {
            seen.insert(id, m);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    let tools: Vec<&str> = seen[&1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(!tools.iter().any(|t| t.contains("attest")), "{tools:?}");
    for id in [2, 3] {
        assert_eq!(seen[&id]["error"]["code"], -32602, "{}", seen[&id]);
        assert!(seen[&id]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unknown tool"));
    }
}

/// A vault written before 2.0.0 (the P1-E.1 old-index fixture, written by
/// `p1d-final`): every layer and the log fingerprint are unchanged by
/// opening it with this build, and no block is `unverified`.
#[test]
fn a_vault_from_before_2_0_0_keeps_its_layers() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_v2/vault");
    let root = std::env::temp_dir().join(format!("rootmark-p2z2-{}-old", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    assert!(Command::new("cp")
        .args(["-R", src.to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    let v = root.join("vault");
    let m = Command::new(rootmark())
        .args(["migrate", "--rename-store", v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(m.status.success());
    let log = std::fs::read(v.join(".rootmark/events.jsonl")).unwrap();
    let vault = rootmarkcore::Vault::open(&v).unwrap();
    let hash = vault.events_hash().unwrap();
    let mut n = 0;
    for d in vault.doc_paths() {
        for b in vault.blame(&d, None).unwrap() {
            assert_ne!(b.layer, rootmarkcore::Layer::Unverified);
            n += 1;
        }
    }
    assert!(n > 0);
    drop(vault);
    let k = Kernel::open(&v).unwrap();
    k.call("status", &json!({}), human()).unwrap();
    assert_eq!(
        std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
        log
    );
    assert_eq!(
        rootmarkcore::Vault::open(&v)
            .unwrap()
            .events_hash()
            .unwrap(),
        hash
    );
    assert!(!v.join(".rootmark/attestations.jsonl").exists());
}

/// `rootmark attest` against a running `rootmark serve --http --watch`: a
/// watched edit is `unverified`, the CLI's attestation right after raises
/// it (late, within the window); `--all` and a missing server are handled.
#[test]
fn cli_attest_raises_a_watched_edit() {
    let v = vault("cli", "window_s = 5.0");
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let p = v.join(".rootmark/policy.toml");
    let recipe = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        &p,
        recipe.replace("[server]\n", &format!("[server]\nhttp_port = {port}\n")),
    )
    .unwrap();
    let vs = v.to_str().unwrap();
    // no server yet: a clear error
    let o = Command::new(rootmark())
        .args(["attest", vs, "notes.md"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    let mut child = Command::new(rootmark())
        .args(["serve", "--vault", vs, "--http", "127.0.0.1", "--watch"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut err = BufReader::new(child.stderr.take().unwrap()).lines();
    loop {
        let l = err.next().expect("serve exited").unwrap();
        if l.contains("watching") {
            break;
        }
    }
    std::thread::spawn(move || for _ in err {});
    std::fs::write(v.join("notes.md"), format!("{NOTES}\nA vim user's line.\n")).unwrap();
    let k = Kernel::open(&v).unwrap();
    let mut seen = false;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(50));
        if layers(&k, "notes.md").iter().any(|l| l == "unverified") {
            seen = true;
            break;
        }
    }
    assert!(seen, "the watcher did not ingest the edit");
    let o = Command::new(rootmark())
        .args(["attest", vs, "notes.md"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["attested"], 1);
    assert_eq!(
        r["attestations"][0]["raised"].as_array().unwrap().len(),
        1,
        "{r}"
    );
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    // `--all` raises nothing already ingested: an unattested agent write in
    // the window stays unverified (security review of Z2)
    std::fs::write(
        v.join("notes.md"),
        format!("{NOTES}\nA vim user's line.\n\nAn agent's line.\n"),
    )
    .unwrap();
    let mut seen = false;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(50));
        if layers(&k, "notes.md").iter().any(|l| l == "unverified") {
            seen = true;
            break;
        }
    }
    assert!(seen, "the watcher did not ingest the agent's line");
    let o = Command::new(rootmark())
        .args(["attest", vs, "--all"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    for a in r["attestations"].as_array().unwrap() {
        assert_eq!(a["raised"], json!([]), "{r}");
    }
    assert!(layers(&k, "notes.md").iter().any(|l| l == "unverified"));
    let _ = child.kill();
    let _ = child.wait();
}

/// The human promotes `unverified` content in one step (spec 2.0.0 §7;
/// `Promoted` with `meta.from = "unverified"`); the agent channel cannot.
#[test]
fn unverified_is_promoted_by_the_human_only() {
    let v = vault("promote", "");
    let k = Kernel::open(&v).unwrap();
    save(
        &k,
        &v,
        "notes.md",
        &format!("{NOTES}\nAn unattested line to promote.\n"),
    );
    let l = blame(&k, "notes.md")
        .into_iter()
        .find(|b| b["layer"] == "unverified")
        .unwrap()["lineage_id"]
        .clone();
    let r = k
        .call(
            "promote",
            &json!({"lineage_id": l, "reason": "r"}),
            Caller::mcp(Some("agent/1")),
        )
        .unwrap();
    assert_eq!(r["decision"], "denied");
    let r = k
        .call(
            "promote",
            &json!({"lineage_id": l, "reason": "mine"}),
            human(),
        )
        .unwrap();
    assert_eq!(r["decision"], "promoted", "{r}");
    assert_eq!(r["event"]["meta"]["from"], "unverified");
    assert_eq!(r["layer"], "canonical");
}

/// Z2 review, finding 1: a human's unattested save, then an agent's
/// direct write to the same paragraph, then the human's late attestation:
/// the agent's text is not raised (the attested bytes are not its
/// content).
#[test]
fn a_late_attestation_does_not_launder_a_later_agent_write() {
    let v = vault("launder", "window_s = 5.0");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let human_text = format!("{NOTES}\nA human line, saved before its attestation.\n");
    save(&k, &v, "notes.md", &human_text);
    let agent_text = human_text.replace("A human line", "An agent replaced the human line");
    save(&k, &v, "notes.md", &agent_text);
    let r = attest(&k, &s, "notes.md", human_text.as_bytes()).unwrap();
    assert_eq!(r["raised"], json!([]), "{r}");
    let b = blame(&k, "notes.md");
    let agent = b
        .iter()
        .find(|x| x["layer"] == "unverified")
        .expect("the agent's paragraph stays unverified");
    assert_eq!(agent["source"], "unattested_fs");
}

/// Two saves the watcher sees as one change: the last bytes are the
/// attested ones → canonical; an attestation of the first bytes alone does
/// not match.
#[test]
fn saves_merged_by_the_watcher() {
    let v = vault("merged", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let one = format!("{NOTES}\nFirst quick save.\n");
    let two = format!("{one}\nSecond quick save.\n");
    attest(&k, &s, "notes.md", one.as_bytes()).unwrap();
    attest(&k, &s, "notes.md", two.as_bytes()).unwrap();
    save(&k, &v, "notes.md", &two);
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
    let v = vault("merged-first", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    attest(&k, &s, "notes.md", one.as_bytes()).unwrap();
    save(&k, &v, "notes.md", &two);
    assert!(layers(&k, "notes.md").iter().any(|l| l == "unverified"));
}

/// `propose`'s sync of an external edit the human attested: canonical.
#[test]
fn propose_sync_of_an_attested_edit_is_canonical() {
    let v = vault("sync-attested", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let text = format!("{NOTES}\nAn offline line the human attested.\n");
    std::fs::write(v.join("notes.md"), &text).unwrap();
    attest(&k, &s, "notes.md", text.as_bytes()).unwrap();
    k.call(
        "propose",
        &json!({"doc_path": "notes.md", "mode": "append", "reason": "r",
                "content": "## Agent\n\nAn agent line."}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    let b = blame(&k, "notes.md");
    assert!(b.iter().all(|x| x["layer"] != "unverified"), "{b:?}");
}

/// A known limit (Z2 review, finding 3; ASSUMPTIONS Z2-1): a save the
/// server saw unattested (here: no attestation in the window), then an
/// attested save of the same paragraph — the paragraph keeps the lowest
/// layer of its lineage (spec §7: ingest never raises), `unverified`, until
/// the human promotes it ("Mine" in the client).
#[test]
fn an_unattested_then_attested_paragraph_stays_unverified_until_promoted() {
    let v = vault("limit", "window_s = 0.1");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let one = NOTES.replace(
        "A human paragraph about roasting.",
        "A human paragraph about roasting, half saved",
    );
    save(&k, &v, "notes.md", &one);
    std::thread::sleep(Duration::from_millis(300));
    let two = NOTES.replace(
        "A human paragraph about roasting.",
        "A human paragraph about roasting, fully saved.",
    );
    attest(&k, &s, "notes.md", two.as_bytes()).unwrap();
    save(&k, &v, "notes.md", &two);
    let b = blame(&k, "notes.md");
    let p = b
        .iter()
        .find(|x| x["layer"] == "unverified")
        .expect("the paragraph keeps unverified")
        .clone();
    let r = k
        .call(
            "promote",
            &json!({"lineage_id": p["lineage_id"], "reason": "mine"}),
            human(),
        )
        .unwrap();
    assert_eq!(r["decision"], "promoted");
    assert!(layers(&k, "notes.md").iter().all(|l| l == "canonical"));
}

/// A rename attestation (`renamed_from`, contract §2.6): accepted only when
/// the renamed file holds exactly the canonical head of the old document;
/// the new path is then human content. Changed bytes, or a proposal in the
/// old document, are `denied`.
#[test]
fn a_rename_attestation_needs_the_canonical_head() {
    let v = vault("rename-from", "");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let call = |doc: &str, bytes: &[u8], from: &str| {
        let h = rootmarkcore::attest::content_hash(bytes);
        k.call(
            "attest",
            &json!({"doc_path": doc, "content_hash": h, "session_id": s.id,
                    "mac": mac(&s.key, &mac_message(doc, &h, None)), "renamed_from": from}),
            human(),
        )
    };
    // a canonical document renamed as it is
    std::fs::rename(v.join("notes.md"), v.join("kept.md")).unwrap();
    call("kept.md", NOTES.as_bytes(), "notes.md").unwrap();
    k.watch_doc("kept.md", "rootmark-serve-watch").unwrap();
    assert!(layers(&k, "kept.md").iter().all(|l| l == "canonical"));
    // renamed with other bytes (an agent's edit not yet observed): denied
    let other = format!("{NOTES}\nAn agent line slipped in before the rename.\n");
    std::fs::write(v.join("kept.md"), &other).unwrap();
    std::fs::rename(v.join("kept.md"), v.join("moved.md")).unwrap();
    let e = call("moved.md", other.as_bytes(), "kept.md").unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    // a document holding a proposal: denied
    k.call(
        "propose",
        &json!({"doc_path": "plan.md", "mode": "create", "reason": "r",
                "content": "# Plan\n\nAn agent's plan."}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    let plan = std::fs::read(v.join("plan.md")).unwrap();
    std::fs::rename(v.join("plan.md"), v.join("plan2.md")).unwrap();
    let e = call("plan2.md", &plan, "plan.md").unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
}
