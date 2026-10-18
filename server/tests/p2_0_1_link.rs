//! P2-0.1/Y1 (GOAL P2-0.1 T1; spec §7.1, contract §2.6 `link_update_of`):
//! the link updates Obsidian writes while renaming a note are attested,
//! with laundering protection.
//! - bytes that differ from the head only in link targets naming the
//!   renamed note, in canonical blocks → accepted, the change is ingested
//!   `canonical`; `status.attestation.link_updates` counts it;
//! - any other difference (text, a label, a new link, a paragraph) → 403
//!   `denied`, the observation stays unattested (`unverified`);
//! - a note holding an agent's content: a link inside the agent's block →
//!   `denied`; a link in a human block → accepted, the agent's block stays
//!   `unverified`; agent bytes the log has not ingested yet → `denied`;
//! - a note renamed (attested) and not yet observed: its own link update is
//!   checked against the head of the path it was renamed from;
//! - `link_update_of` must name a note on disk; it excludes `renamed_from`.
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
    let root = std::env::temp_dir().join(format!("rootmark-p201y1-{}-{name}", std::process::id()));
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

/// Obsidian's rename of `b.md` → `sub/bee.md`: the file moves, then the
/// link in `a.md` is rewritten (`adapter.process`).
fn rename_b(v: &Path, a_text: &str) -> String {
    std::fs::create_dir_all(v.join("sub")).unwrap();
    std::fs::rename(v.join("b.md"), v.join("sub/bee.md")).unwrap();
    let new = a_text.replace("[[b]]", "[[bee]]");
    std::fs::write(v.join("a.md"), &new).unwrap();
    new
}

fn status_link_updates(k: &Kernel) -> u64 {
    k.call("status", &json!({}), human()).unwrap()["attestation"]["link_updates"]
        .as_u64()
        .unwrap()
}

#[test]
fn a_link_update_is_attested_and_canonical() {
    let v = vault("ok");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    assert_eq!(status_link_updates(&k), 0);
    rename_b(&v, A);
    attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap();
    assert_eq!(status_link_updates(&k), 1);
    watch(&k, "a.md");
    let bl = blocks(&k, "a.md");
    assert!(bl.iter().any(|(t, _)| t.contains("[[bee]]")));
    assert!(bl.iter().all(|(_, l)| l == "canonical"), "{bl:?}");
}

#[test]
fn a_late_link_update_attestation_raises() {
    let v = vault("late");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    rename_b(&v, A);
    // the watcher sees the rewrite first: unverified, then the attestation
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "[[bee]]"), "unverified");
    let r = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap();
    assert_eq!(r["raised"].as_array().unwrap().len(), 1);
    assert_eq!(layer_of(&k, "a.md", "[[bee]]"), "canonical");
}

/// A named change of the renamed text.
type Case = (&'static str, fn(&str) -> String);

#[test]
fn anything_but_link_targets_is_denied() {
    let cases: [Case; 5] = [
        ("an appended paragraph", |t| format!("{t}\nAgent text.\n")),
        ("a changed word", |t| {
            t.replace("Another human", "Another agent")
        }),
        ("a label", |t| t.replace("[[bee]]", "[[bee|the bee]]")),
        ("a new link", |t| {
            t.replace("Another human paragraph.", "Another [[bee]] paragraph.")
        }),
        ("a deleted paragraph", |t| {
            t.replace("\nAnother human paragraph.\n", "")
        }),
    ];
    for (i, (what, f)) in cases.iter().enumerate() {
        let v = vault(&format!("deny{i}"));
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        let renamed = rename_b(&v, A);
        std::fs::write(v.join("a.md"), f(&renamed)).unwrap();
        let e =
            attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).expect_err(what);
        assert_eq!(code_of(&e), Code::Denied, "{what}: {e:#}");
        assert_eq!(status_link_updates(&k), 0);
        watch(&k, "a.md");
        assert!(
            blocks(&k, "a.md").iter().any(|(_, l)| l == "unverified"),
            "{what}: the observation stays unattested"
        );
    }
}

#[test]
fn a_target_naming_another_note_is_denied() {
    let v = vault("other");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    rename_b(&v, A);
    std::fs::write(v.join("a.md"), A.replace("[[b]]", "[[c]]")).unwrap();
    std::fs::write(v.join("c.md"), "# C\n").unwrap();
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
}

#[test]
fn an_agents_block_is_never_laundered() {
    // (i) the link sits in the agent's own paragraph: denied
    let v = vault("agent-link");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let with_agent = format!("{A}\nAgent paragraph about [[b]] roasting.\n");
    std::fs::write(v.join("a.md"), &with_agent).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
    let renamed = rename_b(&v, &with_agent);
    assert!(renamed.contains("Agent paragraph about [[bee]]"));
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");

    // (ii) the link sits in a human block, the agent's paragraph elsewhere:
    // accepted; the agent's block is untouched and stays unverified
    let v = vault("agent-elsewhere");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let with_agent = format!("{A}\nAgent paragraph without links.\n");
    std::fs::write(v.join("a.md"), &with_agent).unwrap();
    watch(&k, "a.md");
    rename_b(&v, &with_agent);
    attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "[[bee]]"), "canonical");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");

    // (iii) the agent's bytes reached the file but not the log yet: the
    // rewrite carries them, so it is no link update of the head
    let v = vault("agent-unobserved");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let with_agent = format!("{A}\nAgent paragraph not observed yet.\n");
    std::fs::write(v.join("a.md"), &with_agent).unwrap();
    rename_b(&v, &with_agent);
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
    assert_ne!(layer_of(&k, "a.md", "[[bee]]"), "canonical");
}

#[test]
fn a_renamed_note_linking_to_itself() {
    let v = vault("self");
    let self_text = "# B\n\nThe roast note, see [[b]].\n";
    std::fs::write(v.join("b.md"), self_text).unwrap();
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    attest(&k, &s, &v, "b.md", json!({})).unwrap();
    watch(&k, "b.md");
    // Obsidian: rename (attested with renamed_from), then the self link
    std::fs::rename(v.join("b.md"), v.join("bee.md")).unwrap();
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    std::fs::write(v.join("bee.md"), self_text.replace("[[b]]", "[[bee]]")).unwrap();
    // the log does not know bee.md yet: checked against the head of b.md
    attest(&k, &s, &v, "bee.md", json!({"link_update_of": "bee.md"})).unwrap();
    watch(&k, "bee.md");
    assert!(blocks(&k, "bee.md").iter().all(|(_, l)| l == "canonical"));
}

#[test]
fn link_update_of_must_name_a_note_and_excludes_renamed_from() {
    let v = vault("args");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    rename_b(&v, A);
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "nowhere.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    let e = attest(
        &k,
        &s,
        &v,
        "a.md",
        json!({"link_update_of": "sub/bee.md", "renamed_from": "b.md"}),
    )
    .unwrap_err();
    assert_eq!(code_of(&e), Code::InvalidArguments);
}

#[test]
fn a_late_link_update_never_raises_an_agents_paragraph() {
    // Y1 review 1: the watcher ingests the agent's paragraph and Obsidian's
    // rewrite in one observation (unattested), then the link update's
    // attestation arrives: it is checked against the head before that
    // ingest, so it is denied and raises nothing
    let v = vault("late-agent");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let with_agent = format!("{A}\nAgent paragraph written just before the rename.\n");
    std::fs::write(v.join("a.md"), &with_agent).unwrap();
    rename_b(&v, &with_agent);
    watch(&k, "a.md");
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied, "{e:#}");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
    assert_eq!(layer_of(&k, "a.md", "[[bee]]"), "unverified");
    assert_eq!(status_link_updates(&k), 0);
}

#[test]
fn a_known_rename_checks_the_old_target() {
    // with b.md → sub/bee.md attested, a link to another note (c) turned
    // into a link to bee is no link update of that rename
    let v = vault("old-target");
    std::fs::write(v.join("a.md"), format!("{A}\nAnd [[c]] too.\n")).unwrap();
    std::fs::write(v.join("c.md"), "# C\n").unwrap();
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    attest(&k, &s, &v, "a.md", json!({})).unwrap();
    watch(&k, "a.md");
    std::fs::create_dir_all(v.join("sub")).unwrap();
    std::fs::rename(v.join("b.md"), v.join("sub/bee.md")).unwrap();
    attest(&k, &s, &v, "sub/bee.md", json!({"renamed_from": "b.md"})).unwrap();
    let a = std::fs::read_to_string(v.join("a.md")).unwrap();
    std::fs::write(v.join("a.md"), a.replace("[[c]]", "[[bee]]")).unwrap();
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    // the real rewrite ([[b]] → [[bee]]) is accepted
    std::fs::write(v.join("a.md"), a.replace("[[b]]", "[[bee]]")).unwrap();
    attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap();
}

#[test]
fn replayed_bytes_do_not_match_again() {
    // Y1 security review B1 (T2): a link update is attested and observed;
    // within W the agent removes its block (observed, unattested) and
    // writes the attested bytes back: the attestation was consumed and is
    // voided by the unattested change, so the agent's block is not raised
    let v = vault("replay");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let with_agent = format!("{A}\nAgent paragraph without links.\n");
    std::fs::write(v.join("a.md"), &with_agent).unwrap();
    watch(&k, "a.md");
    let renamed = rename_b(&v, &with_agent);
    attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
    std::fs::write(
        v.join("a.md"),
        renamed.replace("\nAgent paragraph without links.\n", ""),
    )
    .unwrap();
    watch(&k, "a.md");
    std::fs::write(v.join("a.md"), &renamed).unwrap();
    watch(&k, "a.md");
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
}

#[test]
fn a_link_update_that_changes_no_link_is_denied() {
    // Y1 security review B2
    let v = vault("zero");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::create_dir_all(v.join("sub")).unwrap();
    std::fs::rename(v.join("b.md"), v.join("sub/bee.md")).unwrap();
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
}

#[test]
fn the_same_bytes_pending_twice_raise_nothing() {
    // Y1 security review B5 (T12): Obsidian's rewrite is observed before
    // its attestation (pending P1), the agent removes and restores its
    // block (pending P3, the same bytes); the late link update is ambiguous
    // and is denied, nothing is raised
    let v = vault("two-pending");
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    let agent = "\nAgent paragraph without links.\n";
    let x1 = format!("{A}{agent}");
    std::fs::write(v.join("a.md"), &x1).unwrap();
    watch(&k, "a.md");
    // the agent's block older than W: not pending itself
    std::thread::sleep(std::time::Duration::from_millis(2300));
    let x = rename_b(&v, &x1);
    watch(&k, "a.md");
    std::fs::write(v.join("a.md"), x.replace(agent, "")).unwrap();
    watch(&k, "a.md");
    std::fs::write(v.join("a.md"), &x).unwrap();
    watch(&k, "a.md");
    let e = attest(&k, &s, &v, "a.md", json!({"link_update_of": "sub/bee.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
    // a plain late attestation of those bytes raises nothing either
    let r = attest(&k, &s, &v, "a.md", json!({})).unwrap();
    assert!(r["raised"].as_array().unwrap().is_empty());
    assert_eq!(layer_of(&k, "a.md", "Agent paragraph"), "unverified");
}
