//! Conformance Level 1 (spec/CONFORMANCE.md C1-01..C1-17).
//!
//! Each `spec/fixtures/C1-xx/` holds its inputs (`case.json`, documents) and
//! `expected.json`, written from the v3-final core by scripts/gen_fixtures.sh
//! with the same evaluator (`conformance/eval.rs`). The fixtures are frozen:
//! `spec/fixtures/MANIFEST.json` lists the blake3 of every fixture file and a
//! mismatch fails the run before any case is evaluated.
//!
//! `make conformance` sets `MDK_CONFORMANCE_REPORT` to write
//! `reports/conformance.json`: one entry per test with pass/fail and the
//! blake3 of the actual and expected output.

#[allow(dead_code)] // `case_dirs` is used by the fixture generator only
#[path = "conformance/eval.rs"]
mod eval;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const LEVEL1: [&str; 17] = [
    "C1-01", "C1-02", "C1-03", "C1-04", "C1-05", "C1-06", "C1-07", "C1-08", "C1-09", "C1-10",
    "C1-11", "C1-12", "C1-13", "C1-14", "C1-15", "C1-16", "C1-17",
];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures")
}

fn hash(v: &Value) -> String {
    let mut h = blake3::hash(serde_json::to_string(v).unwrap().as_bytes())
        .to_hex()
        .to_string();
    h.truncate(32);
    h
}

/// Fixture freeze: every file listed with its hash, no file missing or added.
fn manifest_problems(root: &Path) -> Vec<String> {
    let text = match fs::read_to_string(root.join("MANIFEST.json")) {
        Ok(t) => t,
        Err(e) => return vec![format!("MANIFEST.json unreadable: {e}")],
    };
    let manifest: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return vec![format!("MANIFEST.json invalid: {e}")],
    };
    let want = manifest["files"].as_object().cloned().unwrap_or_default();
    let got = match eval::fixture_manifest(root) {
        Ok(m) => m,
        Err(e) => return vec![format!("cannot hash fixtures: {e}")],
    };
    let mut out = Vec::new();
    for (path, h) in &want {
        match got.get(path) {
            None => out.push(format!("{path}: listed but missing")),
            Some(g) if Some(g.as_str()) != h.as_str() => {
                out.push(format!("{path}: blake3 {g} != manifest {h}"))
            }
            _ => {}
        }
    }
    for path in got.keys() {
        if !want.contains_key(path) {
            out.push(format!("{path}: not in MANIFEST.json"));
        }
    }
    if want.is_empty() {
        out.push("MANIFEST.json lists no files".into());
    }
    out
}

#[test]
fn conformance_level_1() {
    let root = fixtures();
    let manifest = manifest_problems(&root);
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    for id in LEVEL1 {
        let dir = root.join(id);
        let title = fs::read_to_string(dir.join("case.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|c| c["title"].as_str().map(str::to_string))
            .unwrap_or_default();
        let expected: Option<Value> = fs::read_to_string(dir.join("expected.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        let (result, actual_hash, detail) = match (eval::evaluate(&dir), &expected) {
            (Err(e), _) => ("fail", None, Some(format!("evaluation error: {e:#}"))),
            (Ok(_), None) => (
                "fail",
                None,
                Some("expected.json missing or invalid".into()),
            ),
            (Ok(a), Some(x)) if &a == x => ("pass", Some(hash(&a)), None),
            (Ok(a), Some(_)) => (
                "fail",
                Some(hash(&a)),
                Some("actual output differs from expected.json".into()),
            ),
        };
        if result != "pass" {
            failures.push(format!("{id}: {}", detail.clone().unwrap_or_default()));
        }
        entries.push(json!({
            "id": id,
            "title": title,
            "result": result,
            "actual_hash": actual_hash,
            "expected_hash": expected.as_ref().map(hash),
            "detail": detail,
        }));
    }
    let passed = entries.iter().filter(|e| e["result"] == "pass").count();
    let all_ok = manifest.is_empty() && failures.is_empty();
    let report = json!({
        "spec_version": "1.0",
        "level_claimed": if all_ok { 1 } else { 0 },
        "fixtures_manifest": if manifest.is_empty() { "ok" } else { "mismatch" },
        "fixtures_manifest_problems": manifest,
        "passed": passed,
        "total": LEVEL1.len(),
        "tests": entries,
    });
    if let Ok(path) = std::env::var("MDK_CONFORMANCE_REPORT") {
        let path = PathBuf::from(path);
        if let Some(d) = path.parent() {
            fs::create_dir_all(d).unwrap();
        }
        let mut text = serde_json::to_string_pretty(&report).unwrap();
        text.push('\n');
        fs::write(&path, text).unwrap();
    }
    assert!(
        manifest.is_empty(),
        "spec/fixtures changed (frozen): {manifest:#?}"
    );
    assert!(failures.is_empty(), "conformance failures: {failures:#?}");
    assert_eq!(passed, LEVEL1.len());
}
