//! P1-C/C6 (TASKS C6): `mdk index build/status/compact/watch` and `mdk
//! search` end to end through the `mdk` binary (which runs `mdk-index`, built
//! by the `index` crate next to it; ASSUMPTIONS P1-C C0-1). JSON output,
//! `--at` an old checkpoint, `--include-proposal` labelled, `--mode`, one
//! trace per search. Needs the model files (`make models`).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// `mdk-index` next to `mdk`: built by `cargo test --workspace`; a lone
/// `cargo test -p mdkernel-core` builds it here (same profile).
fn ensure_index_bin() {
    let mdk = PathBuf::from(env!("CARGO_BIN_EXE_mdk"));
    let bin = mdk.with_file_name(format!("mdk-index{}", std::env::consts::EXE_SUFFIX));
    if bin.is_file() {
        return;
    }
    let release = mdk.parent().and_then(|p| p.file_name()) == Some("release".as_ref());
    let mut c = Command::new(env!("CARGO"));
    c.args(["build", "-q", "-p", "mdkernel-index", "--bin", "mdk-index"]);
    if release {
        c.arg("--release");
    }
    let st = c.status().expect("cargo build mdk-index");
    assert!(
        st.success() && bin.is_file(),
        "mdk-index not built at {}",
        bin.display()
    );
}

fn mdk(args: &[&str]) -> (i32, Value, String) {
    ensure_index_bin();
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    (
        out.status.code().unwrap_or(-1),
        v,
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn ok(args: &[&str]) -> Value {
    let (code, v, err) = mdk(args);
    assert_eq!(code, 0, "mdk {args:?}: {err}");
    v
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-c6-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn docs(r: &Value) -> Vec<String> {
    r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["doc_path"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn index_and_search_end_to_end() {
    let root = tmp("e2e");
    let vault = root.join("vault");
    let vs = vault.to_str().unwrap();
    let mini = here("../spec/fixtures/mini_vault");
    ok(&["ingest-dir", vs, mini.to_str().unwrap()]);
    let agent_file = root.join("agent.md");
    fs::write(
        &agent_file,
        "# Zanzibar brewing\n\nThe zanzibar method steeps espresso grounds twice before pressing.\n",
    )
    .unwrap();
    ok(&[
        "propose",
        vs,
        "agent/zanzibar.md",
        agent_file.to_str().unwrap(),
        "--source",
        "claude-code",
    ]);
    let t0 = ok(&["checkpoint", "create", vs, "--name", "t0"]);
    let t0_id = t0["checkpoint_id"].as_str().unwrap().to_string();

    let b = ok(&["index", "build", vs]);
    assert!(b["records"].as_u64().unwrap() > 0);
    assert_eq!(b["embedded"], b["records"]);
    let (code, _, err) = mdk(&["index", "build", vs, "--embedder", "bge-large"]);
    assert_ne!(code, 0);
    assert!(err.contains("unknown embedder"), "{err}");

    // JSON search, default mode hybrid, --at a checkpoint
    let r = ok(&["search", vs, "espresso grind", "--at", "t0", "--k", "5"]);
    assert_eq!(r["mode"], "hybrid");
    assert_eq!(r["checkpoint_id"], t0_id.as_str());
    assert_eq!(r["hits"].as_array().unwrap().len(), 5);
    assert!(r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["layer"] == "canonical"));
    for m in ["flat", "lexical", "hybrid"] {
        let r = ok(&[
            "search",
            vs,
            "zanzibar steeps grounds",
            "--at",
            "t0",
            "--mode",
            m,
        ]);
        assert!(
            !docs(&r).contains(&"agent/zanzibar.md".to_string()),
            "{m}: proposal by default"
        );
        let p = ok(&[
            "search",
            vs,
            "zanzibar steeps grounds",
            "--at",
            "t0",
            "--mode",
            m,
            "--include-proposal",
        ]);
        let hits = p["hits"].as_array().unwrap();
        let i = hits
            .iter()
            .position(|h| h["doc_path"] == "agent/zanzibar.md")
            .unwrap_or_else(|| panic!("{m}: proposal not returned"));
        assert_eq!(hits[i]["layer"], "proposal");
        assert!(
            hits[i..].iter().all(|h| h["layer"] == "proposal"),
            "{m}: labelled and last"
        );
    }
    let (code, _, _) = mdk(&["search", vs, "x", "--mode", "graph"]);
    assert_ne!(code, 0);
    let (code, _, err) = mdk(&["search", vs, "x", "--at", "no-such"]);
    assert_ne!(code, 0);
    assert!(err.contains("no checkpoint"), "{err}");

    // a later write through the hot tier; the old checkpoint does not change
    fs::write(
        vault.join("later.md"),
        "# Later\n\nQuokka roasting notes for the espresso grind.\n",
    )
    .unwrap();
    let w = ok(&["index", "watch", vs, "later.md"]);
    assert_eq!(w["embedded"], 1);
    let head = ok(&["search", vs, "quokka", "--mode", "lexical"]);
    assert_eq!(docs(&head), ["later.md"]);
    assert_ne!(head["checkpoint_id"], t0_id.as_str());
    let old = ok(&["search", vs, "espresso grind", "--at", "t0", "--k", "5"]);
    assert_eq!(old["hits"], r["hits"]);
    assert!(
        ok(&["search", vs, "quokka", "--mode", "lexical", "--at", "t0"])["hits"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let st = ok(&["index", "status", vs]);
    assert_eq!(st["hot_records"], 1);
    assert_eq!(st["stale_docs"].as_array().unwrap().len(), 0);
    let c = ok(&["index", "compact", vs]);
    assert_eq!(c["moved_from_hot"], 1);
    assert_eq!(ok(&["index", "status", vs])["hot_records"], 0);
    assert_eq!(
        ok(&["search", vs, "espresso grind", "--at", "t0", "--k", "5"])["hits"],
        r["hits"]
    );

    // one trace per search (the failed ones wrote none)
    let traces = ok(&["trace", "list", vs]);
    let n = traces
        .as_array()
        .map(Vec::len)
        .unwrap_or_else(|| traces["traces"].as_array().unwrap().len());
    assert_eq!(n, 1 + 6 + 1 + 1 + 1 + 1, "{traces}");
    // --pretty is passed through (indented output)
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["--pretty", "index", "status", vs])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("\n  \""));
    // --matcher before the delegated command is accepted (C6 review fix)
    for m in [&["--matcher", "v3"][..], &["--matcher=v3"][..]] {
        let mut a: Vec<&str> = m.to_vec();
        a.extend(["search", vs, "espresso", "--at", "t0", "--mode", "lexical"]);
        let (code, _, err) = mdk(&a);
        assert_eq!(code, 0, "{a:?}: {err}");
    }
    // `mdk help search` is mdk-index's help
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["help", "search"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("--include-proposal"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_mdk_index_is_an_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["search", "/nonexistent", "q"])
        .env("MDK_INDEX_BIN", "/nonexistent/mdk-index")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("mdk-index"));
}

/// `$MDK_MATCHER` / `--matcher` reach `index watch` as they reach `mdk
/// ingest` (C6 review fix): an empty vault watched with v2 is locked to v2;
/// a vault locked to v3 refuses an explicit v2.
#[test]
fn watch_uses_the_matcher_like_ingest() {
    let root = tmp("matcher");
    let vault = root.join("vault");
    let vs = vault.to_str().unwrap();
    fs::create_dir_all(&vault).unwrap();
    // an index of an empty vault, then a watched document under v2
    ok(&["index", "build", vs]);
    fs::write(vault.join("a.md"), "# A\n\nFirst text about espresso.\n").unwrap();
    ensure_index_bin();
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["index", "watch", vs, "a.md"])
        .env("MDK_MATCHER", "v2")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let log = fs::read_to_string(vault.join(".mdkernel/events.jsonl")).unwrap();
    assert!(log.contains("\"matcher_version\":\"v2\""), "{log}");
    // the vault is now v2: an explicit v3 is refused, the lock is used otherwise
    fs::write(
        vault.join("a.md"),
        "# A\n\nFirst text about espresso, edited.\n",
    )
    .unwrap();
    let (code, _, err) = mdk(&["--matcher", "v3", "index", "watch", vs, "a.md"]);
    assert_ne!(code, 0);
    assert!(err.to_lowercase().contains("matcher"), "{err}");
    ok(&["index", "watch", vs, "a.md"]);
    fs::remove_dir_all(root).unwrap();
}
