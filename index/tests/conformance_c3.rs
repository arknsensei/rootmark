//! Conformance Level 3 (spec/CONFORMANCE.md C3-01..C3-04; spec §9) — the
//! index part of `make conformance`.
//!
//! `core`'s frozen checker (`core/tests/conformance.rs`) evaluates Levels 1
//! and 2 and writes `reports/conformance.json`; it cannot run the index
//! (`core` does not depend on `index`). This checker evaluates
//! `spec/fixtures/C3-*/case.c3.json` with `conformance/eval_c3.rs` and, with
//! `MDK_CONFORMANCE_REPORT`, extends that report: four Level 3 entries,
//! `total` 30, `level_claimed` 3 when Levels 1–2 were claimed (2, intact
//! fixture manifest — which also lists every C3 file) and every C3 case
//! passes. Expected outputs are written by `scripts/gen_fixtures_c3.sh`.
//! The fixture root is resolved at run time (`MDK_FIXTURES`, else
//! `<cwd>/../spec/fixtures`).

#[allow(dead_code)] // `c3_case_dirs` is used by the fixture generator only
#[path = "conformance/eval_c3.rs"]
mod eval_c3;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const LEVEL3: [&str; 4] = ["C3-01", "C3-02", "C3-03", "C3-04"];
const LOWER_TOTAL: u64 = 26;

fn fixtures() -> PathBuf {
    let root = match std::env::var_os("MDK_FIXTURES") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => std::env::current_dir().unwrap().join("../spec/fixtures"),
    };
    let root =
        fs::canonicalize(&root).unwrap_or_else(|e| panic!("fixture root {}: {e}", root.display()));
    assert!(
        root.join("MANIFEST.json").is_file(),
        "no MANIFEST.json in {}",
        root.display()
    );
    root
}

fn blake32(bytes: &[u8]) -> String {
    let mut h = blake3::hash(bytes).to_hex().to_string();
    h.truncate(32);
    h
}

fn hash(v: &Value) -> String {
    blake32(serde_json::to_string(v).unwrap().as_bytes())
}

fn run_case(root: &Path, id: &str) -> (Value, Option<String>) {
    let dir = root.join(id);
    let title = fs::read_to_string(dir.join("case.c3.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|c| c["title"].as_str().map(str::to_string))
        .unwrap_or_default();
    let expected: Option<Value> = fs::read_to_string(dir.join("expected.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let (result, actual_hash, detail) = match (eval_c3::evaluate_c3(&dir), &expected) {
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
    let failure =
        (result != "pass").then(|| format!("{id}: {}", detail.clone().unwrap_or_default()));
    (
        json!({
            "id": id,
            "level": 3,
            "title": title,
            "result": result,
            "actual_hash": actual_hash,
            "expected_hash": expected.as_ref().map(hash),
            "detail": detail,
        }),
        failure,
    )
}

#[test]
fn conformance_level_3() {
    let root = fixtures();
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    for id in LEVEL3 {
        let (e, f) = run_case(&root, id);
        failures.extend(f);
        entries.push(e);
    }
    if let Ok(path) = std::env::var("MDK_CONFORMANCE_REPORT") {
        let path = PathBuf::from(path);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("{}: run the Level 1–2 checker first ({e})", path.display())
        });
        let mut report: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            report["fixtures_root"].as_str(),
            root.to_str(),
            "the report is of another fixture root"
        );
        let lower_ok = report["level_claimed"] == 2
            && report["fixtures_manifest"] == "ok"
            && report["passed"] == LOWER_TOTAL
            && report["total"] == LOWER_TOTAL;
        let tests = report["tests"].as_array_mut().expect("tests");
        assert!(
            tests.iter().all(|t| t["level"] != 3),
            "report already holds Level 3 entries"
        );
        tests.extend(entries.iter().cloned());
        let passed = tests.iter().filter(|t| t["result"] == "pass").count();
        report["passed"] = json!(passed);
        report["total"] = json!(LOWER_TOTAL as usize + LEVEL3.len());
        if lower_ok && failures.is_empty() {
            report["level_claimed"] = json!(3);
        }
        let mut out = serde_json::to_string_pretty(&report).unwrap();
        out.push('\n');
        fs::write(&path, out).unwrap();
    }
    assert!(failures.is_empty(), "conformance failures: {failures:#?}");
}
