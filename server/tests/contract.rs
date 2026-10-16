//! `docs/CLIENT_CONTRACT.md` cases no other test covers (P1-D.1/D1.5;
//! `scripts/contract_check.sh` requires every contract code in a test).
//! Needs the model files.

use mdkserver::error::{code_of, Code};
use mdkserver::tools::{Caller, Kernel};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("mdk-contract-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("a.md"), "# A\n\nSome text about espresso.\n").unwrap();
    let st = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

/// §3 `internal`: a failure of the vault or its index, not of the call —
/// here a damaged index manifest — is `internal` (HTTP 500), never a silent
/// empty result; the call writes no trace.
#[test]
fn a_damaged_index_is_internal() {
    let v = vault("internal");
    std::fs::write(v.join(".mdkernel/index/manifest.json"), "{ not json").unwrap();
    let k = Kernel::open(&v).unwrap();
    let e = k
        .call(
            "search",
            &json!({"query": "espresso"}),
            Caller::mcp(Some("c/1")),
        )
        .unwrap_err();
    assert_eq!(code_of(&e), Code::Internal, "{e:#}");
    assert_eq!(Code::Internal.http_status(), 500);
    let traces = v.join(".mdkernel/traces.jsonl");
    assert!(!traces.exists() || std::fs::read_to_string(traces).unwrap().is_empty());
}

/// §3: every code's HTTP status, as the contract's table states.
#[test]
fn http_statuses_of_the_codes() {
    let want = [
        (Code::InvalidArguments, 400),
        (Code::EmptyQuery, 400),
        (Code::PathOutsideVault, 400),
        (Code::Denied, 403),
        (Code::NotFound, 404),
        (Code::Exists, 409),
        (Code::TooLarge, 413),
        (Code::HeadingNotFound, 422),
        (Code::PolicyViolation, 422),
        (Code::IndexStale, 503),
        (Code::Internal, 500),
    ];
    assert_eq!(want.len(), Code::ALL.len());
    for (c, s) in want {
        assert_eq!(c.http_status(), s, "{}", c.as_str());
    }
}

// ---- every tool and every code of the contract from a real call
// (`scripts/contract_check.sh` matches `call_tool(&k, "<tool>"` and
// `expect_code(&k, … Code::<Variant>);` in this file, comments stripped)

fn call_tool(k: &Kernel, tool: &str, args: serde_json::Value) -> serde_json::Value {
    k.call(tool, &args, Caller::mcp(Some("contract/1.0")))
        .unwrap_or_else(|e| panic!("{tool}: {e:#}"))
}

fn expect_code(k: &Kernel, tool: &str, args: serde_json::Value, want: Code) {
    let e = k
        .call(tool, &args, Caller::mcp(Some("contract/1.0")))
        .expect_err(tool);
    assert_eq!(code_of(&e), want, "{tool}: {e:#}");
}

#[test]
fn every_tool_answers() {
    let v = vault("tools");
    let k = Kernel::open(&v).unwrap();
    let st = call_tool(&k, "status", json!({}));
    assert_eq!(st["spec_version"], mdkserver::tools::SPEC_VERSION);
    let s = call_tool(&k, "search", json!({"query": "espresso"}));
    let lineage = s["hits"][0]["lineage_ids"][0].clone();
    let rb = call_tool(
        &k,
        "read_block",
        json!({"doc_path": "a.md", "lineage_id": lineage}),
    );
    assert_eq!(rb["block"]["block"]["layer"], "canonical");
    let h = call_tool(&k, "history", json!({"lineage_id": lineage}));
    assert!(!h["events"].as_array().unwrap().is_empty());
    let b = call_tool(&k, "blame", json!({"doc_path": "a.md"}));
    assert_eq!(b["blocks"].as_array().unwrap().len(), 2);
    let p = call_tool(
        &k,
        "propose",
        json!({"doc_path": "agent/p.md", "content": "# P\n\nAgent text.", "reason": "contract"}),
    );
    let pl = p["blocks"][1]["lineage_id"].clone();
    let pr = call_tool(
        &k,
        "promote",
        json!({"lineage_id": pl, "reason": "contract"}),
    );
    assert_eq!(pr["decision"], "denied");
    let c = call_tool(&k, "checkpoint", json!({"name": "contract"}));
    let cs = call_tool(&k, "checkpoints", json!({}));
    assert!(cs["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["checkpoint_id"] == c["checkpoint_id"] && x["name"] == "contract"));
    let t = call_tool(&k, "trace", json!({"consumer": "contract/1.0"}));
    assert_eq!(t["traces"].as_array().unwrap().len(), 2);
}

#[test]
fn every_code_from_a_real_call() {
    let v = vault("codes");
    let k = Kernel::open(&v).unwrap();
    expect_code(
        &k,
        "search",
        json!({"query": "x", "mode": "bogus"}),
        Code::InvalidArguments,
    );
    expect_code(&k, "search", json!({"query": " "}), Code::EmptyQuery);
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "../x.md", "content": "x", "reason": "r"}),
        Code::PathOutsideVault,
    );
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "a.md", "content": "x", "reason": "r"}),
        Code::Exists,
    );
    expect_code(&k, "history", json!({"lineage_id": "0123"}), Code::NotFound);
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "a.md", "mode": "replace_section", "heading": "Nope", "content": "x", "reason": "r"}),
        Code::HeadingNotFound,
    );
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "a.md", "mode": "replace_section", "heading": "A", "content": "x", "reason": "r"}),
        Code::Denied,
    );
    std::fs::write(
        v.join(".mdkernel/policy.toml"),
        mdkserver::config::default_recipe()
            .replace("max_propose_bytes = 1048576", "max_propose_bytes = 4"),
    )
    .unwrap();
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "big.md", "content": "five!", "reason": "r"}),
        Code::TooLarge,
    );
    std::fs::write(v.join(".mdkernel/policy.toml"), "promotion = 7\n").unwrap();
    expect_code(&k, "status", json!({}), Code::PolicyViolation);
    std::fs::remove_file(v.join(".mdkernel/policy.toml")).unwrap();
    // K-E0.4: an append to human blocks without a heading of its own
    expect_code(
        &k,
        "propose",
        json!({"doc_path": "a.md", "mode": "append", "content": "x", "reason": "r"}),
        Code::PolicyViolation,
    );
    // K-E0.3: an index of another format is stale, not internal
    let m = v.join(".mdkernel/index/manifest.json");
    let fresh = std::fs::read_to_string(&m).unwrap();
    std::fs::write(&m, fresh.replace("mdkernel-index/3", "mdkernel-index/2")).unwrap();
    expect_code(&k, "search", json!({"query": "espresso"}), Code::IndexStale);
    std::fs::write(v.join(".mdkernel/index/manifest.json"), "{ not json").unwrap();
    expect_code(&k, "search", json!({"query": "espresso"}), Code::Internal);
}
