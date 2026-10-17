//! P1-D/D4 `propose` through `tools::Kernel` (no transport):
//! 1. a human edit made outside the server and not yet ingested is taken as
//!    human content before the agent's text is written — the agent is never
//!    credited with it;
//! 2. a failing ingest leaves the document exactly as it was (no agent text
//!    in a file the vault did not take);
//! 3. the size limit comes from `[server] max_propose_bytes`.
//!
//! Needs the model files (`make models`).

use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-propose-{}-{name}", std::process::id()));
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
    let st = std::process::Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    v
}

fn blame(k: &Kernel, doc: &str) -> Vec<Value> {
    k.call("blame", &json!({"doc_path": doc}), Caller::mcp(Some("t/1")))
        .unwrap()["blocks"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn external_human_edit_is_not_credited_to_the_agent() {
    let v = vault("sync");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    // P1-D.1 (K-D1.1, ASSUMPTIONS D1.0-4): under the default `append_only`
    // a heading-less append to a section of canonical blocks is refused
    // (it would join their chunk); this test is about the sync, so the
    // vault allows it explicitly
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe()
            .replace("policy = \"append_only\"", "policy = \"unrestricted\""),
    )
    .unwrap();
    let p = v.join("espresso.md");
    let mut text = std::fs::read_to_string(&p).unwrap();
    text.push_str("\nA human wrote this offline about kumquat beans.\n");
    std::fs::write(&p, &text).unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "espresso.md", "content": "An agent adds a line about lychee.",
                    "reason": "sync test", "mode": "append"}),
            Caller::mcp(Some("agent/1")),
        )
        .unwrap();
    assert!(r["synced_external_events"].as_u64().unwrap() > 0, "{r}");
    let b = blame(&k, "espresso.md");
    let n = b.len();
    assert_eq!(b[n - 2]["author_kind"], "Human", "{:?}", b[n - 2]);
    assert_eq!(b[n - 2]["layer"], "canonical");
    assert_eq!(b[n - 1]["author_kind"], "Agent");
    assert_eq!(b[n - 1]["layer"], "proposal");
    assert_eq!(b[n - 1]["source"], "agent/1");
    let after = std::fs::read_to_string(&p).unwrap();
    assert_eq!(
        after,
        format!("{text}\nAn agent adds a line about lychee.\n")
    );
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}

#[test]
fn failed_ingest_leaves_the_file_unchanged() {
    let v = vault("fail");
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    // the vault is locked to matcher v3: an ingest under v2 is refused
    let k = Kernel::open_with(&v, Some(rootmarkcore::MatcherVersion::V2)).unwrap();
    let p = v.join("filter.md");
    let before = std::fs::read(&p).unwrap();
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "filter.md", "content": "agent text", "reason": "fail test",
                    "mode": "append"}),
            Caller::mcp(Some("agent/1")),
        )
        .unwrap_err();
    assert!(format!("{e:#}").contains("locked to matcher"), "{e:#}");
    assert_eq!(std::fs::read(&p).unwrap(), before);
    // a new document under v2: written, refused, removed again
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "newdir/new.md", "content": "agent text", "reason": "fail test"}),
            Caller::mcp(Some("agent/1")),
        )
        .unwrap_err();
    // P1-D.1: the ingest is planned (and refused) before the file is
    // written (the edit policy needs its effect, K-D1.1), so nothing is
    // written to restore
    assert!(format!("{e:#}").contains("locked to matcher"), "{e:#}");
    assert!(!v.join("newdir/new.md").exists());
    assert!(!v.join("newdir").exists());
    assert!(!v.join(".rootmark/proposals.jsonl").exists());
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}

#[test]
fn size_limit_from_policy() {
    let v = vault("size");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe()
            .replace("max_propose_bytes = 1048576", "max_propose_bytes = 16"),
    )
    .unwrap();
    let call = |c: &str| {
        k.call(
            "propose",
            &json!({"doc_path": "s.md", "content": c, "reason": "size"}),
            Caller::mcp(Some("agent/1")),
        )
    };
    let e = call("seventeen bytes!!").unwrap_err();
    assert!(format!("{e:#}").contains("max_propose_bytes = 16"), "{e:#}");
    assert!(!v.join("s.md").exists());
    call("sixteen bytes!!!").unwrap();
    assert_eq!(
        std::fs::read_to_string(v.join("s.md")).unwrap(),
        "sixteen bytes!!!\n"
    );
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}

/// D4 review / security review: other spellings of existing names (case,
/// Unicode form) never reach a document under a foreign doc_path.
#[test]
fn other_spellings_of_existing_names_are_refused() {
    let v = vault("spell");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let p = |doc: &str, mode: &str, c: &str| {
        k.call(
            "propose",
            &json!({"doc_path": doc, "content": c, "reason": "spelling", "mode": mode}),
            Caller::mcp(Some("agent/1")),
        )
    };
    let before = std::fs::read(v.join("espresso.md")).unwrap();
    // case variant of an existing document (macOS: the same file)
    assert!(p("ESPRESSO.md", "append", "agent text").is_err());
    assert_eq!(std::fs::read(v.join("espresso.md")).unwrap(), before);
    // case variant of an existing directory
    p("sub/a.md", "create", "# A\n\nfirst").unwrap();
    let r = p("SUB/b.md", "create", "agent text about quokka");
    let in_sub: Vec<String> = std::fs::read_dir(v.join("sub"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(!in_sub.contains(&"b.md".to_string()), "{r:?} {in_sub:?}");
    // NFD spelling of an NFC name
    p("caf\u{e9}.md", "create", "# Caf\u{e9}\n\nnfc").unwrap();
    let before = std::fs::read(v.join("caf\u{e9}.md")).unwrap();
    assert!(p("cafe\u{301}.md", "append", "agent text").is_err());
    assert_eq!(std::fs::read(v.join("caf\u{e9}.md")).unwrap(), before);
    // the log names no foreign spelling
    let log = std::fs::read_to_string(v.join(".rootmark/events.jsonl")).unwrap();
    for bad in ["\"ESPRESSO.md\"", "\"SUB/b.md\"", "cafe\u{301}.md"] {
        assert!(!log.contains(bad), "{bad} in the log");
    }
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}

/// D4 review / security review: a symbolic link planted at the temporary
/// file's name is never followed.
#[cfg(unix)]
#[test]
fn planted_temporary_symlink_is_not_followed() {
    let v = vault("symlink");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let outside = v.parent().unwrap().join("outside.txt");
    std::os::unix::fs::symlink(&outside, v.join(".evil.md.rootmark-propose.tmp")).unwrap();
    k.call(
        "propose",
        &json!({"doc_path": "evil.md", "content": "agent text", "reason": "symlink"}),
        Caller::mcp(Some("agent/1")),
    )
    .unwrap();
    assert!(!outside.exists(), "the write followed the link");
    let m = std::fs::symlink_metadata(v.join("evil.md")).unwrap();
    assert!(m.file_type().is_file());
    assert_eq!(
        std::fs::read_to_string(v.join("evil.md")).unwrap(),
        "agent text\n"
    );
    assert!(std::fs::symlink_metadata(v.join(".evil.md.rootmark-propose.tmp")).is_err());
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}
