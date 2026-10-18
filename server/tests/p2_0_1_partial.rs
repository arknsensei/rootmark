//! P2-0.1/Y2 (GOAL P2-0.1 T2; spec §7.1 "Partial match", §8 `partial`):
//! observation-wise block matching.
//! - an attestation carrying `content` that matches exactly behaves as
//!   without it (same events, same log);
//! - the human saved, the agent appended before the watcher looked (one
//!   observation): the attested blocks are `canonical`, the agent's
//!   `unverified`; the log holds `partial: true` and never the content;
//! - the same, attested late: the human's lineages are raised, the agent's
//!   are not; an agent's copy of a human block is never raised;
//! - `content` that does not hash to `content_hash` → `invalid_arguments`;
//!   an observation that does not contain the attested blocks (the agent
//!   changed the human's paragraph) stays `unverified`.
//!
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

const A: &str = "# A\n\nSee [[b]] for the roast.\n\nAnother human paragraph.\n";
const B: &str = "# B\n\nThe roast note.\n";

/// Two human notes (`a.md` links to `b.md`), canonical; the default
/// policy (`unverified`) with a human id.
fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p201y2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("a.md"), A).unwrap();
    std::fs::write(v.join("b.md"), B).unwrap();
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    let recipe = format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nwindow_s = 2.0\n",
        rootmarkserver::config::default_recipe()
    );
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-0-1-test")
}

/// `(text, layer)` of every head block.
fn blocks(k: &Kernel, doc: &str) -> Vec<(String, String)> {
    let bl = k.call("blame", &json!({"doc_path": doc}), human()).unwrap();
    bl["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            let t = k
                .call(
                    "read_block",
                    &json!({"doc_path": doc, "lineage_id": b["lineage_id"]}),
                    human(),
                )
                .unwrap();
            (
                t["block"]["block"]["text"].as_str().unwrap().to_string(),
                b["layer"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn layer_of(k: &Kernel, doc: &str, needle: &str) -> String {
    blocks(k, doc)
        .into_iter()
        .find(|(t, _)| t.contains(needle))
        .unwrap_or_else(|| panic!("no block with {needle:?} in {doc}"))
        .1
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

/// An attestation of `doc`'s bytes on disk with extra fields.
fn attest(k: &Kernel, s: &Session, v: &Path, doc: &str, extra: Value) -> anyhow::Result<Value> {
    let bytes = std::fs::read(v.join(doc)).unwrap();
    let h = rootmarkcore::attest::content_hash(&bytes);
    let mut body = json!({"doc_path": doc, "content_hash": h, "session_id": s.id,
                "mac": mac(&s.key, &mac_message(doc, &h, None))});
    for (key, val) in extra.as_object().unwrap() {
        body[key] = val.clone();
    }
    k.call("attest", &body, human())
}

fn watch(k: &Kernel, doc: &str) {
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
}

fn with_content(text: &str) -> Value {
    json!({ "content": text })
}

fn log_lines(v: &Path) -> Vec<Value> {
    std::fs::read_to_string(v.join(".rootmark/attestations.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

const HUMAN: &str = "A human paragraph saved in the editor.";
const AGENT: &str = "An agent paragraph appended right after.";

#[test]
fn an_exact_match_with_content_is_unchanged() {
    let mut logs = Vec::new();
    for with in [false, true] {
        let v = vault(&format!("exact-{with}"));
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        let h = format!("{A}\n{HUMAN}\n");
        std::fs::write(v.join("a.md"), &h).unwrap();
        attest(
            &k,
            &s,
            &v,
            "a.md",
            if with { with_content(&h) } else { json!({}) },
        )
        .unwrap();
        watch(&k, "a.md");
        assert_eq!(layer_of(&k, "a.md", HUMAN), "canonical");
        logs.push((
            std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
            std::fs::read(v.join(".rootmark/attestations.jsonl")).unwrap(),
        ));
    }
    assert_eq!(logs[0], logs[1]);
}

#[test]
fn a_collision_keeps_the_human_canonical_and_the_agent_unverified() {
    let v = vault("early");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let h = format!("{A}\n{HUMAN}\n");
    std::fs::write(v.join("a.md"), &h).unwrap();
    attest(&k, &s, &v, "a.md", with_content(&h)).unwrap();
    // the agent appends before the watcher looks: one observation
    std::fs::write(v.join("a.md"), format!("{h}\n{AGENT}\n")).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", HUMAN), "canonical");
    assert_eq!(layer_of(&k, "a.md", AGENT), "unverified");
    let lines = log_lines(&v);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(lines[1]["partial"], json!(true));
    assert_eq!(lines[1]["content_hash"], lines[0]["content_hash"]);
    assert!(lines.iter().all(|l| l.get("content").is_none()));
    // two snapshots for the one observation: the attested bytes, then the
    // rest (the agent's paragraph, declared unverified)
    let ev = std::fs::read_to_string(v.join(".rootmark/events.jsonl")).unwrap();
    let snaps = ev
        .lines()
        .filter(|l| l.contains("SnapshotCommitted") && l.contains("\"a.md\""))
        .count();
    assert_eq!(snaps, 3, "ingest-dir, the human's bytes, the rest");
    let created: Vec<Value> = ev
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .filter(|e| e["kind"] == "BlockCreated" && e["doc_path"] == "a.md")
        .collect();
    assert_eq!(
        created.last().unwrap()["meta"]["layer"],
        json!("unverified")
    );
}

#[test]
fn a_late_collision_raises_the_human_blocks_only() {
    let v = vault("late");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let h = format!("{A}\n{HUMAN}\n");
    std::fs::write(v.join("a.md"), format!("{h}\n{AGENT}\n")).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", HUMAN), "unverified");
    let hash = rootmarkcore::attest::content_hash(h.as_bytes());
    let r = k
        .call(
            "attest",
            &json!({"doc_path": "a.md", "content_hash": hash, "session_id": s.id,
                "mac": mac(&s.key, &mac_message("a.md", &hash, None)), "content": h}),
            human(),
        )
        .unwrap();
    assert_eq!(r["raised"].as_array().unwrap().len(), 1, "{r}");
    assert_eq!(layer_of(&k, "a.md", HUMAN), "canonical");
    assert_eq!(layer_of(&k, "a.md", AGENT), "unverified");
    assert_eq!(log_lines(&v)[0]["partial"], json!(true));
}

#[test]
fn an_agents_copy_of_a_human_block_is_never_raised() {
    // the agent appends a copy of the human's existing paragraph (the same
    // text, the same version): early and late, the copy stays unverified
    for late in [false, true] {
        let v = vault(&format!("copy-{late}"));
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        let h = format!("{A}\n{HUMAN}\n");
        let observed = format!("{h}\nAnother human paragraph.\n");
        let hash = rootmarkcore::attest::content_hash(h.as_bytes());
        let body = json!({"doc_path": "a.md", "content_hash": hash, "session_id": s.id,
            "mac": mac(&s.key, &mac_message("a.md", &hash, None)), "content": h});
        std::fs::write(v.join("a.md"), if late { &observed } else { &h }).unwrap();
        if !late {
            k.call("attest", &body, human()).unwrap();
            std::fs::write(v.join("a.md"), &observed).unwrap();
        }
        watch(&k, "a.md");
        if late {
            k.call("attest", &body, human()).unwrap();
        }
        let bl = blocks(&k, "a.md");
        let copies: Vec<_> = bl
            .iter()
            .filter(|(t, _)| t == "Another human paragraph.")
            .collect();
        assert_eq!(copies.len(), 2);
        assert_eq!(
            copies.iter().filter(|(_, l)| l == "canonical").count(),
            1,
            "late {late}: only the original is canonical: {bl:?}"
        );
        assert_eq!(layer_of(&k, "a.md", HUMAN), "canonical");
    }
}

#[test]
fn content_must_be_the_attested_bytes() {
    let v = vault("mismatch");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let e = attest(&k, &s, &v, "a.md", with_content("something else")).unwrap_err();
    assert_eq!(code_of(&e), Code::InvalidArguments);
}

#[test]
fn an_observation_without_the_attested_blocks_stays_unverified() {
    let v = vault("changed");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let h = format!("{A}\n{HUMAN}\n");
    std::fs::write(v.join("a.md"), &h).unwrap();
    attest(&k, &s, &v, "a.md", with_content(&h)).unwrap();
    // the agent rewrites the human's new paragraph
    std::fs::write(
        v.join("a.md"),
        h.replace("saved in the editor", "rewritten by an agent"),
    )
    .unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "rewritten by an agent"), "unverified");
    assert_eq!(log_lines(&v).len(), 1);
}

fn events(v: &Path) -> Vec<Value> {
    std::fs::read_to_string(v.join(".rootmark/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn content_body(s: &Session, text: &str) -> Value {
    let h = rootmarkcore::attest::content_hash(text.as_bytes());
    json!({"doc_path": "a.md", "content_hash": h, "session_id": s.id,
        "mac": mac(&s.key, &mac_message("a.md", &h, None)), "content": text})
}

#[test]
fn a_save_after_the_agent_is_never_a_late_partial_match() {
    // Y2 review 1 (X1, X2): the agent's change is pending (observed,
    // unattested); the human then saves bytes that are a subset of it and
    // attests with content before the watcher sees the save. The file holds
    // the attested bytes, so their own observation is coming: no late
    // partial match; the save is matched exactly, the deletion is the
    // human's and the agent's block stays unverified.
    for (name, saved) in [
        ("x1", A.to_string()),
        (
            "x2",
            format!("# A\n\nSee [[b]] for the roast.\n\n{AGENT}\n"),
        ),
    ] {
        let v = vault(name);
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        std::fs::write(v.join("a.md"), format!("{A}\n{AGENT}\n")).unwrap();
        watch(&k, "a.md");
        let n0 = events(&v).len();
        std::fs::write(v.join("a.md"), &saved).unwrap();
        let r = k
            .call("attest", &content_body(&s, &saved), human())
            .unwrap();
        assert!(r["raised"].as_array().unwrap().is_empty(), "{name}: {r}");
        watch(&k, "a.md");
        let deleted: Vec<Value> = events(&v)[n0..]
            .iter()
            .filter(|e| e["kind"] == "BlockDeleted")
            .cloned()
            .collect();
        assert_eq!(deleted.len(), 1, "{name}");
        assert_eq!(
            deleted[0]["meta"]["actor"]["kind"],
            json!("human"),
            "{name}"
        );
        if name == "x2" {
            assert_eq!(layer_of(&k, "a.md", AGENT), "unverified");
        }
        assert!(log_lines(&v).iter().all(|l| l.get("partial").is_none()));
    }
}

#[test]
fn saved_bytes_missing_a_kept_block_are_no_late_partial_match() {
    // Y2 review 1 c: as X2, but the agent writes again before the
    // attestation arrives (the file no longer holds the saved bytes): the
    // saved bytes lack a block the pending change kept, so they were saved
    // after it — no late partial match; the agent's first block stays
    // unverified
    let v = vault("x2c");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(v.join("a.md"), format!("{A}\n{AGENT}\n")).unwrap();
    watch(&k, "a.md");
    let saved = format!("# A\n\nSee [[b]] for the roast.\n\n{AGENT}\n");
    std::fs::write(v.join("a.md"), format!("{saved}\nA second agent line.\n")).unwrap();
    let r = k
        .call("attest", &content_body(&s, &saved), human())
        .unwrap();
    assert!(r["raised"].as_array().unwrap().is_empty(), "{r}");
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", AGENT), "unverified");
    assert_eq!(layer_of(&k, "a.md", "A second agent line."), "unverified");
}
