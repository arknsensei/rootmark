//! Conformance Levels 1 and 2 (spec/CONFORMANCE.md C1-01..C1-17,
//! C2-01..C2-13; C2-09 negative trace input, K-C0.2; C2-10 provenance of
//! events, spec 1.0.3, K-D1.2; C2-11 `unverified` by default, C2-12
//! attestation matching, spec 2.0.0, K-P2-0.4; C2-13 partial match, spec
//! 2.0.1, K-P2-01.3); C2-14 since spec 2.0.2 (P2-0.2, K-P2-02.2).
//!
//! Each `spec/fixtures/C1-xx/` holds its inputs (`case.json`, documents) and
//! `expected.json`, written from the v3-final core by scripts/gen_fixtures.sh
//! with the same evaluator (`conformance/eval.rs`). The fixtures are frozen:
//! `spec/fixtures/MANIFEST.json` lists the blake3 of every fixture file and a
//! mismatch fails the run before any case is evaluated.
//!
//! `make conformance` sets `ROOTMARK_CONFORMANCE_REPORT` to write
//! `reports/conformance.json`: one entry per test with pass/fail and the
//! blake3 of the actual and expected output, plus the absolute fixture root
//! and the blake3 of its `MANIFEST.json`.
//!
//! Level 2 (P1-B/B7): each `spec/fixtures/C2-xx/` holds `case.c2.json` (a
//! governance script; not `case.json`, which the frozen v3-final C1
//! generator walks) and `expected.json` written from the current core by
//! `scripts/gen_fixtures_c2.sh` with `conformance/eval_c2.rs`. The level
//! claimed is the highest level whose tests, and every lower level's, pass
//! with an intact manifest.
//!
//! The fixture root is resolved at run time, never compiled in (P1-A verifier
//! recommendation Ö2): `ROOTMARK_FIXTURES` (`make conformance` passes
//! `$(CURDIR)/spec/fixtures`), else `<cwd>/../spec/fixtures` — cargo runs
//! tests in `core/`. A test binary moved with its `target/` therefore reads
//! the fixtures of the tree it runs in.

#[allow(dead_code)] // `case_dirs` is used by the fixture generator only
#[path = "conformance/eval.rs"]
mod eval;
#[allow(dead_code)] // `c2_case_dirs` is used by the C2 fixture generator only
#[path = "conformance/eval_c2.rs"]
mod eval_c2;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const LEVEL1: [&str; 17] = [
    "C1-01", "C1-02", "C1-03", "C1-04", "C1-05", "C1-06", "C1-07", "C1-08", "C1-09", "C1-10",
    "C1-11", "C1-12", "C1-13", "C1-14", "C1-15", "C1-16", "C1-17",
];

const LEVEL2: [&str; 14] = [
    "C2-01", "C2-02", "C2-03", "C2-04", "C2-05", "C2-06", "C2-07", "C2-08", "C2-09", "C2-10",
    "C2-11", "C2-12", "C2-13", "C2-14",
];

/// `(absolute fixture root, how it was resolved)`; a root without
/// `MANIFEST.json` is an error, not a fallback.
fn fixtures() -> (PathBuf, &'static str) {
    let (root, source) = match std::env::var_os("ROOTMARK_FIXTURES") {
        Some(p) if !p.is_empty() => (PathBuf::from(p), "ROOTMARK_FIXTURES"),
        _ => (
            std::env::current_dir()
                .expect("current directory")
                .join("../spec/fixtures"),
            "cwd",
        ),
    };
    let root = fs::canonicalize(&root)
        .unwrap_or_else(|e| panic!("fixture root {} ({source}): {e}", root.display()));
    assert!(
        root.join("MANIFEST.json").is_file(),
        "fixture root {} ({source}) has no MANIFEST.json",
        root.display()
    );
    (root, source)
}

fn blake32(bytes: &[u8]) -> String {
    let mut h = blake3::hash(bytes).to_hex().to_string();
    h.truncate(32);
    h
}

fn hash(v: &Value) -> String {
    blake32(serde_json::to_string(v).unwrap().as_bytes())
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

/// Evaluates one case; `(entry, failure)`.
fn run_case(root: &Path, id: &str, case_file: &str, level2: bool) -> (Value, Option<String>) {
    let dir = root.join(id);
    let title = fs::read_to_string(dir.join(case_file))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|c| c["title"].as_str().map(str::to_string))
        .unwrap_or_default();
    let expected: Option<Value> = fs::read_to_string(dir.join("expected.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let actual = if level2 {
        eval_c2::evaluate_c2(&dir)
    } else {
        eval::evaluate(&dir)
    };
    let (result, actual_hash, detail) = match (actual, &expected) {
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
            "level": if level2 { 2 } else { 1 },
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
fn conformance_levels_1_and_2() {
    let (root, root_source) = fixtures();
    let manifest_hash = fs::read(root.join("MANIFEST.json"))
        .ok()
        .map(|b| blake32(&b));
    let manifest = manifest_problems(&root);
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    let mut failed_level = [false; 3];
    for (ids, case_file, level2) in [
        (&LEVEL1[..], "case.json", false),
        (&LEVEL2[..], "case.c2.json", true),
    ] {
        for id in ids {
            let (entry, failure) = run_case(&root, id, case_file, level2);
            if let Some(f) = failure {
                failed_level[if level2 { 2 } else { 1 }] = true;
                failures.push(f);
            }
            entries.push(entry);
        }
    }
    let passed = entries.iter().filter(|e| e["result"] == "pass").count();
    let all_ok = manifest.is_empty() && failures.is_empty();
    let level = match (manifest.is_empty(), failed_level[1], failed_level[2]) {
        (false, _, _) | (true, true, _) => 0,
        (true, false, true) => 1,
        (true, false, false) => 2,
    };
    let report = json!({
        "spec_version": "2.0.3",
        "level_claimed": level,
        "fixtures_root": root.to_str().expect("UTF-8 fixture root"),
        "fixtures_root_source": root_source,
        "fixtures_manifest_hash": manifest_hash,
        "fixtures_manifest": if manifest.is_empty() { "ok" } else { "mismatch" },
        "fixtures_manifest_problems": manifest,
        "passed": passed,
        "total": LEVEL1.len() + LEVEL2.len(),
        "tests": entries,
    });
    if let Ok(path) = std::env::var("ROOTMARK_CONFORMANCE_REPORT") {
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
    assert!(all_ok);
    assert_eq!(passed, LEVEL1.len() + LEVEL2.len());
}
