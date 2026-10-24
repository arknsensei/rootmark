//! P2-L/L0 b (P2-0.3 verification Ö3, V3 observation 5): a block that a
//! late attestation raised to `canonical` keeps `canonical` through a later
//! rename of its note (spec §7.1 "Rename": a rename raises no block and
//! drops none). The verifier saw "Son plan notu." — appended on a blur save,
//! observed by the watcher before its attestation, raised late — come back
//! `unverified` after the note's folder move.
//!
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

/// A vault of human notes, canonical; `unattested = "unverified"` (the
/// opt-in under test), window 2 s.
fn vault(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2l0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        st.status.success(),
        "{}",
        String::from_utf8_lossy(&st.stderr)
    );
    let recipe = format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nunattested = \"unverified\"\nwindow_s = 2.0\n",
        rootmarkserver::config::base_recipe()
    );
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-l-test")
}

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

fn mv(v: &Path, from: &str, to: &str) {
    std::fs::create_dir_all(v.join(to).parent().unwrap()).unwrap();
    std::fs::rename(v.join(from), v.join(to)).unwrap();
}

const PLAN: &str = "# Plan\n\nBu not kendine bağlanır: [[plan-son]]\n\nBir paragraf.\n";

/// The note with a human block appended, observed before its attestation
/// and raised late (edit 30 of the verifier's V3).
fn late_raised(name: &str) -> (PathBuf, Kernel, Session) {
    let v = vault(name, &[("proje/plan-son.md", PLAN)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("proje/plan-son.md"),
        format!("{PLAN}\nSon plan notu.\n"),
    )
    .unwrap();
    watch(&k, "proje/plan-son.md");
    assert_eq!(layer_of(&k, "proje/plan-son.md", "Son plan"), "unverified");
    let r = attest(&k, &s, &v, "proje/plan-son.md", json!({})).unwrap();
    assert!(!r["raised"].as_array().unwrap().is_empty(), "{r}");
    assert_eq!(layer_of(&k, "proje/plan-son.md", "Son plan"), "canonical");
    (v, k, s)
}

#[test]
fn a_late_raised_block_stays_canonical_through_a_rename() {
    let (v, k, s) = late_raised("on-time");
    // edit 38: the folder move, attested on time
    mv(&v, "proje/plan-son.md", "plan.md");
    let t = std::fs::read_to_string(v.join("plan.md")).unwrap();
    std::fs::write(v.join("plan.md"), t.replace("[[plan-son]]", "[[plan]]")).unwrap();
    attest(
        &k,
        &s,
        &v,
        "plan.md",
        json!({"link_update_of": "plan.md", "link_update_from": "proje/plan-son.md"}),
    )
    .unwrap();
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "Bir paragraf"), "canonical");
    assert_eq!(layer_of(&k, "plan.md", "Son plan"), "canonical");
}

#[test]
fn a_late_raised_block_stays_canonical_through_a_plain_rename() {
    let (v, k, s) = late_raised("plain");
    mv(&v, "proje/plan-son.md", "plan-son.md");
    attest(
        &k,
        &s,
        &v,
        "plan-son.md",
        json!({"renamed_from": "proje/plan-son.md"}),
    )
    .unwrap();
    watch(&k, "plan-son.md");
    assert_eq!(layer_of(&k, "plan-son.md", "Bir paragraf"), "canonical");
    assert_eq!(layer_of(&k, "plan-son.md", "Son plan"), "canonical");
}

#[test]
fn a_late_raised_block_stays_canonical_through_a_late_rename() {
    let (v, k, s) = late_raised("late");
    mv(&v, "proje/plan-son.md", "plan-son.md");
    watch(&k, "plan-son.md");
    attest(
        &k,
        &s,
        &v,
        "plan-son.md",
        json!({"renamed_from": "proje/plan-son.md"}),
    )
    .unwrap();
    assert_eq!(layer_of(&k, "plan-son.md", "Bir paragraf"), "canonical");
    assert_eq!(layer_of(&k, "plan-son.md", "Son plan"), "canonical");
}

/// V3 edits 22, 30, 38: the note leaves `plan.md`, gets a block, and is
/// renamed back to `plan.md` — a path whose earlier head the log keeps.
#[test]
fn a_block_survives_a_rename_back_to_an_earlier_path() {
    for late in [false, true] {
        let name = if late { "back-late" } else { "back" };
        let v = vault(name, &[("plan.md", PLAN)]);
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        mv(&v, "plan.md", "proje/plan-son.md");
        attest(
            &k,
            &s,
            &v,
            "proje/plan-son.md",
            json!({"renamed_from": "plan.md"}),
        )
        .unwrap();
        watch(&k, "proje/plan-son.md");
        std::fs::write(
            v.join("proje/plan-son.md"),
            format!("{PLAN}\nSon plan notu.\n"),
        )
        .unwrap();
        attest(&k, &s, &v, "proje/plan-son.md", json!({})).unwrap();
        watch(&k, "proje/plan-son.md");
        assert_eq!(layer_of(&k, "proje/plan-son.md", "Son plan"), "canonical");
        mv(&v, "proje/plan-son.md", "plan.md");
        if late {
            watch(&k, "plan.md");
        }
        let r = attest(
            &k,
            &s,
            &v,
            "plan.md",
            json!({"renamed_from": "proje/plan-son.md"}),
        );
        if !late {
            watch(&k, "plan.md");
        }
        eprintln!("{name}: {r:?}");
        assert_eq!(
            layer_of(&k, "plan.md", "Bir paragraf"),
            "canonical",
            "{name}"
        );
        assert_eq!(layer_of(&k, "plan.md", "Son plan"), "canonical", "{name}");
    }
}

const SELF_PLAN: &str = "# Plan\n\nBu not kendine bağlanır: [[plan]]\n\nBir paragraf.\n";

/// The verifier's V3 edits 22, 30 and 38 with the note's own link: `plan.md`
/// is moved to `proje/plan-son.md` (its link rewritten to `[[plan-son]]`),
/// gets a human block, and is moved back to `plan.md` — a path whose earlier
/// head the log keeps (the watcher does not ingest removals) — and its link
/// is rewritten back to `[[plan]]`. Root cause (attestation log of the
/// reproduction, L0.md): the self link update was checked against that
/// earlier head (one block fewer) and `denied`, so the rewritten bytes were
/// ingested unattested and the block new since then dropped to
/// `unverified`. On time and late.
#[test]
fn a_self_link_update_back_on_an_earlier_path_keeps_every_layer() {
    for late in [false, true] {
        let name = if late { "self-back-late" } else { "self-back" };
        let v = vault(name, &[("plan.md", SELF_PLAN)]);
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        // edit 22: into the folder, the link rewritten
        mv(&v, "plan.md", "proje/plan-son.md");
        attest(
            &k,
            &s,
            &v,
            "proje/plan-son.md",
            json!({"renamed_from": "plan.md"}),
        )
        .unwrap();
        let t = std::fs::read_to_string(v.join("proje/plan-son.md")).unwrap();
        std::fs::write(
            v.join("proje/plan-son.md"),
            t.replace("[[plan]]", "[[plan-son]]"),
        )
        .unwrap();
        attest(
            &k,
            &s,
            &v,
            "proje/plan-son.md",
            json!({"link_update_of": "proje/plan-son.md", "link_update_from": "plan.md"}),
        )
        .unwrap();
        watch(&k, "proje/plan-son.md");
        // edit 30
        let t = std::fs::read_to_string(v.join("proje/plan-son.md")).unwrap();
        std::fs::write(
            v.join("proje/plan-son.md"),
            format!("{t}\nSon plan notu.\n"),
        )
        .unwrap();
        attest(&k, &s, &v, "proje/plan-son.md", json!({})).unwrap();
        watch(&k, "proje/plan-son.md");
        assert_eq!(layer_of(&k, "proje/plan-son.md", "Son plan"), "canonical");
        // edit 38: back to plan.md, the rename attested while the file holds
        // the moved bytes, then the note's own link rewritten
        mv(&v, "proje/plan-son.md", "plan.md");
        attest(
            &k,
            &s,
            &v,
            "plan.md",
            json!({"renamed_from": "proje/plan-son.md"}),
        )
        .unwrap();
        let t = std::fs::read_to_string(v.join("plan.md")).unwrap();
        std::fs::write(v.join("plan.md"), t.replace("[[plan-son]]", "[[plan]]")).unwrap();
        let lu = json!({"link_update_of": "plan.md", "link_update_from": "proje/plan-son.md"});
        if late {
            watch(&k, "plan.md");
            assert_eq!(layer_of(&k, "plan.md", "Son plan"), "unverified", "{name}");
            let r = attest(&k, &s, &v, "plan.md", lu).unwrap();
            assert!(!r["raised"].as_array().unwrap().is_empty(), "{name}: {r}");
        } else {
            attest(&k, &s, &v, "plan.md", lu).unwrap();
            watch(&k, "plan.md");
        }
        for b in ["# Plan", "kendine", "Bir paragraf", "Son plan"] {
            assert_eq!(layer_of(&k, "plan.md", b), "canonical", "{name}: {b}");
        }
    }
}

/// The fallback to the renamed-from path's head never vouches for bytes it
/// does not hold: an agent line beside the rewritten link is `denied`.
#[test]
fn a_self_link_update_back_on_an_earlier_path_never_vouches_for_other_bytes() {
    let v = vault("self-back-deny", &[("plan.md", SELF_PLAN)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    mv(&v, "plan.md", "proje/plan-son.md");
    attest(
        &k,
        &s,
        &v,
        "proje/plan-son.md",
        json!({"renamed_from": "plan.md"}),
    )
    .unwrap();
    watch(&k, "proje/plan-son.md");
    mv(&v, "proje/plan-son.md", "plan.md");
    attest(
        &k,
        &s,
        &v,
        "plan.md",
        json!({"renamed_from": "proje/plan-son.md"}),
    )
    .unwrap();
    let t = std::fs::read_to_string(v.join("plan.md")).unwrap();
    std::fs::write(
        v.join("plan.md"),
        format!(
            "{}\nAgent line beside the rewrite.\n",
            t.replace("[[plan]]", "[[plan]] ")
        ),
    )
    .unwrap();
    let e = attest(
        &k,
        &s,
        &v,
        "plan.md",
        json!({"link_update_of": "plan.md", "link_update_from": "proje/plan-son.md"}),
    )
    .unwrap_err();
    assert_eq!(
        rootmarkserver::error::code_of(&e),
        rootmarkserver::error::Code::Denied
    );
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "Agent line"), "unverified");
}
