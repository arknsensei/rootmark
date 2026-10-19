//! P2-0.2/X1 (GOAL P2-0.2 T1, T2; spec §7.1, contract §2.6):
//! - T1: a Markdown link whose label is the old note's name (or its target
//!   as written, `%20`/UTF-8 encoded included) follows the rename — label
//!   and target together — and the link update is accepted; any other label
//!   change, or a label change beside other text, is `denied`;
//! - T2: a rename carries every block's layer over as it is (`canonical`,
//!   `unverified`, `proposal`), on time and late; it raises nothing, so an
//!   agent's block stays `unverified` at the new path, and a later rename of
//!   the same note is attested again (no cascade);
//! - the renamed note's own links (a self link) are checked in the same
//!   attestation: with the rename, or as the note's link update against the
//!   old path when the watcher never saw the moved bytes.
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

/// A vault of human notes (`files`: path → text), canonical; the default
/// policy (`unverified`) with a human id.
fn vault(name: &str, files: &[(&str, &str)]) -> PathBuf {
    vault_w(name, files, 2.0)
}

/// `vault` with the window `window_s`.
fn vault_w(name: &str, files: &[(&str, &str)], window_s: f64) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p202x1-{}-{name}", std::process::id()));
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
        "{}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nwindow_s = {window_s:?}\n",
        rootmarkserver::config::default_recipe()
    );
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-0-2-test")
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

/// Moves `from` to `to` on disk (creating `to`'s folder).
fn mv(v: &Path, from: &str, to: &str) {
    std::fs::create_dir_all(v.join(to).parent().unwrap()).unwrap();
    std::fs::rename(v.join(from), v.join(to)).unwrap();
}

const B: &str = "# B\n\nThe roast note.\n";

#[test]
fn markdown_labels_follow_the_rename() {
    // the forms Obsidian rewrites: label = name, label = target as written,
    // %20 and UTF-8 percent-encoded paths; a label that is not the name stays
    let cases: [(&str, &str, &str, &str); 5] = [
        ("[b](b.md)", "[bee](bee.md)", "b.md", "bee.md"),
        ("[notes](b.md)", "[notes](bee.md)", "b.md", "bee.md"),
        (
            "[b](dizin%20ad%C4%B1/b.md)",
            "[bee](dizin%20ad%C4%B1/bee.md)",
            "dizin adı/b.md",
            "dizin adı/bee.md",
        ),
        (
            "[b](dizin%20adı/b.md)",
            "[bee](dizin%20adı/bee.md)",
            "dizin adı/b.md",
            "dizin adı/bee.md",
        ),
        ("[b.md](b.md)", "[bee.md](bee.md)", "b.md", "bee.md"),
    ];
    for (i, (old, new, from, to)) in cases.iter().enumerate() {
        let a = format!("# A\n\nSee {old} for the roast.\n\nAnother human paragraph.\n");
        let v = vault(&format!("label{i}"), &[("a.md", &a), (from, B)]);
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        mv(&v, from, to);
        std::fs::write(v.join("a.md"), a.replace(old, new)).unwrap();
        attest(
            &k,
            &s,
            &v,
            "a.md",
            json!({"link_update_of": to, "link_update_from": from}),
        )
        .unwrap_or_else(|e| panic!("{old} → {new}: {e:#}"));
        watch(&k, "a.md");
        assert!(
            blocks(&k, "a.md").iter().all(|(_, l)| l == "canonical"),
            "{old} → {new}"
        );
    }
}

#[test]
fn a_label_beyond_the_name_is_denied() {
    let a = "# A\n\nSee [b](b.md) for the roast.\n\nAnother human paragraph.\n";
    let cases: [(&str, &str); 4] = [
        // the label changed to something that is not the new name
        ("label", "See [the bee](bee.md) for the roast."),
        // the label follows, and a word changed beside it
        ("label and text", "See [bee](bee.md) for the coffee."),
        // the label changed but the target did not
        ("label only", "See [bee](b.md) for the roast."),
        // a wikilink's alias is no Markdown label: it never follows
        ("alias", "See [[bee|bee]] for the roast."),
    ];
    for (i, (what, line)) in cases.iter().enumerate() {
        let src = if *what == "alias" {
            a.replace("[b](b.md)", "[[b|b]]")
        } else {
            a.to_string()
        };
        let v = vault(&format!("deny{i}"), &[("a.md", &src), ("b.md", B)]);
        let k = Kernel::open(&v).unwrap();
        let s = session(&k);
        mv(&v, "b.md", "bee.md");
        std::fs::write(
            v.join("a.md"),
            src.replace(src.lines().nth(2).unwrap(), line),
        )
        .unwrap();
        let e = attest(
            &k,
            &s,
            &v,
            "a.md",
            json!({"link_update_of": "bee.md", "link_update_from": "b.md"}),
        )
        .expect_err(what);
        assert_eq!(code_of(&e), Code::Denied, "{what}: {e:#}");
        watch(&k, "a.md");
        assert!(
            blocks(&k, "a.md").iter().any(|(_, l)| l == "unverified"),
            "{what}: the observation stays unattested"
        );
    }
}

/// `b.md`: two human paragraphs, then an agent's paragraph written to the
/// file directly (observed: `unverified`).
fn with_agent(name: &str, b_text: &str) -> (PathBuf, Kernel, Session) {
    let v = vault(name, &[("b.md", b_text)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("b.md"),
        format!("{b_text}\nAgent paragraph written to the file.\n"),
    )
    .unwrap();
    watch(&k, "b.md");
    assert_eq!(layer_of(&k, "b.md", "Agent paragraph"), "unverified");
    assert_eq!(layer_of(&k, "b.md", "Human one"), "canonical");
    (v, k, s)
}

const BH: &str = "# B\n\nHuman one about the roast.\n\nHuman two about the beans.\n";

#[test]
fn a_rename_carries_layers_and_does_not_cascade() {
    let (v, k, s) = with_agent("carry", BH);
    let before: Vec<String> = blocks(&k, "b.md").into_iter().map(|(_, l)| l).collect();
    // first rename: attested on time
    mv(&v, "b.md", "bee.md");
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    watch(&k, "bee.md");
    let after: Vec<String> = blocks(&k, "bee.md").into_iter().map(|(_, l)| l).collect();
    assert_eq!(before, after, "every layer carried over as it is");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
    // second rename of the same note, into a folder: attested again
    mv(&v, "bee.md", "arşiv/bee.md");
    attest(
        &k,
        &s,
        &v,
        "arşiv/bee.md",
        json!({"renamed_from": "bee.md"}),
    )
    .unwrap();
    watch(&k, "arşiv/bee.md");
    let again: Vec<String> = blocks(&k, "arşiv/bee.md")
        .into_iter()
        .map(|(_, l)| l)
        .collect();
    assert_eq!(before, again);
    // a human edit after the renames is canonical, the agent's block is not
    let t = std::fs::read_to_string(v.join("arşiv/bee.md")).unwrap();
    std::fs::write(
        v.join("arşiv/bee.md"),
        format!("{t}\nHuman three after the renames.\n"),
    )
    .unwrap();
    attest(&k, &s, &v, "arşiv/bee.md", json!({})).unwrap();
    watch(&k, "arşiv/bee.md");
    assert_eq!(layer_of(&k, "arşiv/bee.md", "Human three"), "canonical");
    assert_eq!(
        layer_of(&k, "arşiv/bee.md", "Agent paragraph"),
        "unverified"
    );
}

#[test]
fn a_late_rename_attestation_raises_only_what_was_canonical() {
    let (v, k, s) = with_agent("carry-late", BH);
    mv(&v, "b.md", "bee.md");
    // the watcher sees the new path first: all unverified
    watch(&k, "bee.md");
    assert!(blocks(&k, "bee.md").iter().all(|(_, l)| l == "unverified"));
    let r = attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    // the heading and the two human paragraphs, not the agent's
    assert_eq!(r["raised"].as_array().unwrap().len(), 3, "{r}");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
}

#[test]
fn a_rename_with_other_bytes_is_denied_and_raises_nothing() {
    let (v, k, s) = with_agent("carry-deny", BH);
    // an agent line that reached the file before the rename, unobserved
    let t = std::fs::read_to_string(v.join("b.md")).unwrap();
    std::fs::write(v.join("b.md"), format!("{t}\nAgent line unobserved.\n")).unwrap();
    mv(&v, "b.md", "bee.md");
    let e = attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    watch(&k, "bee.md");
    assert!(blocks(&k, "bee.md").iter().all(|(_, l)| l == "unverified"));
}

const SELF: &str = "# B\n\nHuman one, see [[b]] and [[b#B]].\n\nHuman two, see [b](b.md).\n";

#[test]
fn a_self_linking_note_is_renamed_in_one_attestation() {
    // the watcher never sees the moved bytes: only the rewritten ones; the
    // rename attestation of the moved bytes is denied (the file no longer
    // holds them), the note's own link update carries the rename
    let (v, k, s) = with_agent("self", SELF);
    let moved = std::fs::read_to_string(v.join("b.md")).unwrap();
    mv(&v, "b.md", "bee.md");
    let rewritten = moved
        .replace("[[b]]", "[[bee]]")
        .replace("[[b#B]]", "[[bee#B]]")
        .replace("[b](b.md)", "[bee](bee.md)");
    std::fs::write(v.join("bee.md"), &rewritten).unwrap();
    attest(
        &k,
        &s,
        &v,
        "bee.md",
        json!({"link_update_of": "bee.md", "link_update_from": "b.md"}),
    )
    .unwrap();
    watch(&k, "bee.md");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Human two"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
}

#[test]
fn a_self_linking_note_renamed_then_rewritten() {
    // the watcher sees the moved bytes (rename attestation), then the
    // rewrite (the note's own link update, against its new head)
    let (v, k, s) = with_agent("self2", SELF);
    let moved = std::fs::read_to_string(v.join("b.md")).unwrap();
    mv(&v, "b.md", "bee.md");
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    watch(&k, "bee.md");
    let rewritten = moved
        .replace("[[b]]", "[[bee]]")
        .replace("[b](b.md)", "[bee](bee.md)");
    std::fs::write(v.join("bee.md"), &rewritten).unwrap();
    attest(
        &k,
        &s,
        &v,
        "bee.md",
        json!({"link_update_of": "bee.md", "link_update_from": "b.md"}),
    )
    .unwrap();
    watch(&k, "bee.md");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Human two"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
}

#[test]
fn a_self_link_in_an_unverified_block_stays_unverified() {
    // the agent's own paragraph links to the note: renamed with the note,
    // its link rewritten — it keeps its layer, the human blocks theirs
    let v = vault("self-agent", &[("b.md", BH)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("b.md"),
        format!("{BH}\nAgent paragraph, see [[b]].\n"),
    )
    .unwrap();
    watch(&k, "b.md");
    let moved = std::fs::read_to_string(v.join("b.md")).unwrap();
    mv(&v, "b.md", "bee.md");
    std::fs::write(v.join("bee.md"), moved.replace("[[b]]", "[[bee]]")).unwrap();
    attest(
        &k,
        &s,
        &v,
        "bee.md",
        json!({"link_update_of": "bee.md", "link_update_from": "b.md"}),
    )
    .unwrap();
    watch(&k, "bee.md");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");

    // the same, with the moved bytes observed first (the rewrite is then
    // checked against the note's new head): the same layers
    let v = vault("self-agent2", &[("b.md", BH)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("b.md"),
        format!("{BH}\nAgent paragraph, see [[b]].\n"),
    )
    .unwrap();
    watch(&k, "b.md");
    let moved = std::fs::read_to_string(v.join("b.md")).unwrap();
    mv(&v, "b.md", "bee.md");
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    watch(&k, "bee.md");
    std::fs::write(v.join("bee.md"), moved.replace("[[b]]", "[[bee]]")).unwrap();
    attest(
        &k,
        &s,
        &v,
        "bee.md",
        json!({"link_update_of": "bee.md", "link_update_from": "b.md"}),
    )
    .unwrap();
    watch(&k, "bee.md");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
}

#[test]
fn a_renamed_proposal_stays_a_proposal_late_too() {
    let v = vault("proposal", &[("a.md", "# A\n\nHuman text.\n")]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    k.call(
        "propose",
        &json!({"doc_path": "plan.md", "mode": "create", "reason": "r",
                "content": "# Plan\n\nAn agent's plan."}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    mv(&v, "plan.md", "plan2.md");
    watch(&k, "plan2.md");
    attest(&k, &s, &v, "plan2.md", json!({"renamed_from": "plan.md"})).unwrap();
    assert!(blocks(&k, "plan2.md").iter().all(|(_, l)| l == "proposal"));
}

#[test]
fn a_fake_rename_never_raises() {
    // an agent with the token claims a rename of a human note to a path
    // holding its own bytes: denied; claimed from its own (unverified) note
    // to a new path: accepted, and the bytes stay unverified
    let (v, k, s) = with_agent("fake", BH);
    std::fs::write(v.join("evil.md"), "# Evil\n\nAgent prose.\n").unwrap();
    let e = attest(&k, &s, &v, "evil.md", json!({"renamed_from": "b.md"})).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    watch(&k, "evil.md");
    assert!(blocks(&k, "evil.md").iter().all(|(_, l)| l == "unverified"));
    mv(&v, "evil.md", "evil2.md");
    attest(&k, &s, &v, "evil2.md", json!({"renamed_from": "evil.md"})).unwrap();
    watch(&k, "evil2.md");
    assert!(blocks(&k, "evil2.md")
        .iter()
        .all(|(_, l)| l == "unverified"));
}

#[test]
fn a_human_edited_proposal_is_carried_lower_never_higher() {
    // K-P2-02.2 (2), ASSUMPTIONS X1-1: a proposal whose provenance is human
    // (the human edited the agent's paragraph; it stays a proposal) cannot
    // be recorded as a proposal of a new lineage: it is carried unverified;
    // the agent's untouched heading stays a proposal
    let v = vault("human-proposal", &[("a.md", "# A\n\nHuman text.\n")]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    k.call(
        "propose",
        &json!({"doc_path": "plan.md", "mode": "create", "reason": "r",
                "content": "# Plan\n\nAn agent's plan."}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    std::fs::write(
        v.join("plan.md"),
        "# Plan\n\nAn agent's plan, edited by the human.\n",
    )
    .unwrap();
    attest(&k, &s, &v, "plan.md", json!({})).unwrap();
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "edited by the human"), "proposal");
    mv(&v, "plan.md", "plan2.md");
    attest(&k, &s, &v, "plan2.md", json!({"renamed_from": "plan.md"})).unwrap();
    watch(&k, "plan2.md");
    assert_eq!(layer_of(&k, "plan2.md", "# Plan"), "proposal");
    assert_eq!(
        layer_of(&k, "plan2.md", "edited by the human"),
        "unverified"
    );
}

#[test]
fn the_same_block_twice_keeps_each_place_s_layer() {
    // X1 review R6: the agent copied a human paragraph (same version, its
    // copy unverified); each place keeps its own layer
    let para = "Human one about the roast.";
    let (v, k, s) = with_agent("dup", BH);
    let t = std::fs::read_to_string(v.join("b.md")).unwrap();
    std::fs::write(v.join("b.md"), format!("{t}\n{para}\n")).unwrap();
    watch(&k, "b.md");
    let before: Vec<String> = blocks(&k, "b.md").into_iter().map(|(_, l)| l).collect();
    assert_eq!(before.iter().filter(|l| *l == "unverified").count(), 2);
    mv(&v, "b.md", "bee.md");
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    watch(&k, "bee.md");
    let after: Vec<String> = blocks(&k, "bee.md").into_iter().map(|(_, l)| l).collect();
    assert_eq!(before, after);
}

#[test]
fn caps_are_never_lost_while_a_rename_can_match() {
    // X1 security B1: more than 256 capped attestations after a rename, its
    // observation still in the window: the agent's block stays unverified
    let notes: Vec<(String, String)> = (0..260)
        .map(|i| (format!("n/n{i}.md"), format!("# N{i}\n\nHuman note {i}.\n")))
        .collect();
    let mut files: Vec<(&str, &str)> = notes
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    files.push(("b.md", BH));
    let v = vault_w("evict", &files, 60.0);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("b.md"),
        format!("{BH}\nAgent paragraph written to the file.\n"),
    )
    .unwrap();
    watch(&k, "b.md");
    mv(&v, "b.md", "bee.md");
    attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
    for i in 0..260 {
        let (f, t) = (format!("n/n{i}.md"), format!("m/n{i}.md"));
        mv(&v, &f, &t);
        attest(&k, &s, &v, &t, json!({"renamed_from": f})).unwrap();
    }
    watch(&k, "bee.md");
    assert_eq!(layer_of(&k, "bee.md", "Agent paragraph"), "unverified");
    assert_eq!(layer_of(&k, "bee.md", "Human one"), "canonical");
}

#[test]
fn a_copy_is_no_rename() {
    // X1 security B2: with the old file still on disk, a note's "own link
    // update" against it (`link_update_from`) is no rename: denied
    let c = "# C\n\nHuman text, see [c](c.md) and [[c]].\n";
    let v = vault("copy", &[("c.md", c)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    std::fs::write(
        v.join("x.md"),
        c.replace("[c](c.md)", "[x](x.md)")
            .replace("[[c]]", "[[x]]"),
    )
    .unwrap();
    let e = attest(
        &k,
        &s,
        &v,
        "x.md",
        json!({"link_update_of": "x.md", "link_update_from": "c.md"}),
    )
    .unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    watch(&k, "x.md");
    assert!(blocks(&k, "x.md").iter().all(|(_, l)| l == "unverified"));
}

#[test]
fn a_plain_attestation_never_lifts_a_rename_s_caps() {
    // X1 security B3: a plain attestation of the moved bytes beside the
    // rename attestation (either order): the caps hold
    for order in ["plain-first", "rename-first"] {
        let (v, k, s) = with_agent(&format!("lift-{order}"), BH);
        mv(&v, "b.md", "bee.md");
        if order == "plain-first" {
            attest(&k, &s, &v, "bee.md", json!({})).unwrap();
            attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
        } else {
            attest(&k, &s, &v, "bee.md", json!({"renamed_from": "b.md"})).unwrap();
            attest(&k, &s, &v, "bee.md", json!({})).unwrap();
        }
        watch(&k, "bee.md");
        assert_eq!(
            layer_of(&k, "bee.md", "Agent paragraph"),
            "unverified",
            "{order}"
        );
    }
}
