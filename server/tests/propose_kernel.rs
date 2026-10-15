//! P1-D/D4 `propose` through `tools::Kernel` (no transport):
//! 1. a human edit made outside the server and not yet ingested is taken as
//!    human content before the agent's text is written — the agent is never
//!    credited with it;
//! 2. a failing ingest leaves the document exactly as it was (no agent text
//!    in a file the vault did not take);
//! 3. the size limit comes from `[server] max_propose_bytes`.
//!
//! Needs the model files (`make models`).

use mdkserver::tools::Kernel;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("mdk-propose-{}-{name}", std::process::id()));
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
    let st = std::process::Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    v
}

fn blame(k: &Kernel, doc: &str) -> Vec<Value> {
    k.call("blame", &json!({"doc_path": doc}), Some("t/1"))
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
    let p = v.join("espresso.md");
    let mut text = std::fs::read_to_string(&p).unwrap();
    text.push_str("\nA human wrote this offline about kumquat beans.\n");
    std::fs::write(&p, &text).unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "espresso.md", "content": "An agent adds a line about lychee.",
                    "reason": "sync test", "mode": "append"}),
            Some("agent/1"),
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
    let k = Kernel::open_with(&v, Some(mdkcore::MatcherVersion::V2)).unwrap();
    let p = v.join("filter.md");
    let before = std::fs::read(&p).unwrap();
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "filter.md", "content": "agent text", "reason": "fail test",
                    "mode": "append"}),
            Some("agent/1"),
        )
        .unwrap_err();
    assert!(format!("{e:#}").contains("locked to matcher"), "{e:#}");
    assert_eq!(std::fs::read(&p).unwrap(), before);
    // a new document under v2: written, refused, removed again
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "newdir/new.md", "content": "agent text", "reason": "fail test"}),
            Some("agent/1"),
        )
        .unwrap_err();
    assert!(format!("{e:#}").contains("restored"), "{e:#}");
    assert!(!v.join("newdir/new.md").exists());
    assert!(!v.join("newdir").exists());
    assert!(!v.join(".mdkernel/proposals.jsonl").exists());
    std::fs::remove_dir_all(v.parent().unwrap()).unwrap();
}

#[test]
fn size_limit_from_policy() {
    let v = vault("size");
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        mdkserver::config::default_recipe()
            .replace("max_propose_bytes = 1048576", "max_propose_bytes = 16"),
    )
    .unwrap();
    let call = |c: &str| {
        k.call(
            "propose",
            &json!({"doc_path": "s.md", "content": c, "reason": "size"}),
            Some("agent/1"),
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
