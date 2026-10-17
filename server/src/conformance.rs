//! `mdk conformance` (P1-E/E5; GOAL E-7): the conformance suite of
//! spec/CONFORMANCE.md, Levels 1–3, run by the binary itself on the
//! fixtures embedded at build time (build.rs) — or on a fixture tree given
//! with `--fixtures`. It is the `make conformance` equivalent of a release
//! binary: the case evaluators are the frozen ones, compiled in unchanged
//! (`core/tests/conformance/eval.rs`, `eval_c2.rs`,
//! `index/tests/conformance/eval_c3.rs`; never a copy), and the report has
//! the shape of `reports/conformance.json` (the same entries, hashes and
//! level rule as `core/tests/conformance.rs` + `index/tests/
//! conformance_c3.rs`; `server/tests/conformance_cli.rs` checks that both
//! reports agree test by test).
//!
//! Level 3 needs the embedder model files (`--models`, else `MDK_MODELS`,
//! else the build tree's `models/`; `make models`).

#[allow(dead_code)] // `case_dirs` is the fixture generator's
#[path = "../../core/tests/conformance/eval.rs"]
mod eval;
#[allow(dead_code)] // `c2_case_dirs` is the fixture generator's
#[path = "../../core/tests/conformance/eval_c2.rs"]
mod eval_c2;
#[allow(dead_code)] // `c3_case_dirs` is the fixture generator's
#[path = "../../index/tests/conformance/eval_c3.rs"]
mod eval_c3;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/fixtures.rs"));
}

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// The spec version the suite checks (`core/tests/conformance.rs`).
pub const SPEC_VERSION: &str = crate::tools::SPEC_VERSION;

const LEVEL1: [&str; 17] = [
    "C1-01", "C1-02", "C1-03", "C1-04", "C1-05", "C1-06", "C1-07", "C1-08", "C1-09", "C1-10",
    "C1-11", "C1-12", "C1-13", "C1-14", "C1-15", "C1-16", "C1-17",
];
const LEVEL2: [&str; 10] = [
    "C2-01", "C2-02", "C2-03", "C2-04", "C2-05", "C2-06", "C2-07", "C2-08", "C2-09", "C2-10",
];
const LEVEL3: [&str; 7] = [
    "C3-01", "C3-02", "C3-03", "C3-04", "C3-05", "C3-06", "C3-07",
];
pub const TOTAL: usize = LEVEL1.len() + LEVEL2.len() + LEVEL3.len();

fn blake32(bytes: &[u8]) -> String {
    let mut h = blake3::hash(bytes).to_hex().to_string();
    h.truncate(32);
    h
}

fn hash(v: &Value) -> String {
    blake32(serde_json::to_string(v).expect("JSON").as_bytes())
}

/// The embedded fixtures written into `dir` (a fresh directory).
pub fn extract(dir: &Path) -> Result<usize> {
    for (rel, bytes) in embedded::FIXTURES {
        let p = dir.join(rel);
        if let Some(d) = p.parent() {
            fs::create_dir_all(d).with_context(|| format!("create {}", d.display()))?;
        }
        fs::write(&p, bytes).with_context(|| format!("write {}", p.display()))?;
    }
    Ok(embedded::FIXTURES.len())
}

/// Fixture freeze: every file listed with its hash, none missing or added
/// (as `core/tests/conformance.rs`).
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

fn run_case(root: &Path, id: &str, level: u8) -> Value {
    let dir = root.join(id);
    let case_file = match level {
        1 => "case.json",
        2 => "case.c2.json",
        _ => "case.c3.json",
    };
    let title = fs::read_to_string(dir.join(case_file))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|c| c["title"].as_str().map(str::to_string))
        .unwrap_or_default();
    let expected: Option<Value> = fs::read_to_string(dir.join("expected.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let actual = match level {
        1 => eval::evaluate(&dir),
        2 => eval_c2::evaluate_c2(&dir),
        _ => eval_c3::evaluate_c3(&dir),
    };
    let (result, actual_hash, detail) = match (actual, &expected) {
        (Err(e), _) => ("fail", None, Some(format!("evaluation error: {e:#}"))),
        (Ok(_), None) => (
            "fail",
            None,
            Some("expected.json missing or invalid".to_string()),
        ),
        (Ok(a), Some(x)) if &a == x => ("pass", Some(hash(&a)), None),
        (Ok(a), Some(_)) => (
            "fail",
            Some(hash(&a)),
            Some("actual output differs from expected.json".to_string()),
        ),
    };
    json!({
        "id": id,
        "level": level,
        "title": title,
        "result": result,
        "actual_hash": actual_hash,
        "expected_hash": expected.as_ref().map(hash),
        "detail": detail,
    })
}

/// Runs Levels 1–3 on the fixture tree `root`; `source` names where it came
/// from ("embedded" or "--fixtures"). The level rule is the frozen
/// checkers': 2 iff the manifest is intact and every C1/C2 case passes, 3
/// iff also every C3 case passes.
pub fn run(root: &Path, source: &str) -> Result<Value> {
    if !root.join("MANIFEST.json").is_file() {
        bail!("fixture root {} has no MANIFEST.json", root.display());
    }
    let manifest_hash = blake32(&fs::read(root.join("MANIFEST.json"))?);
    let manifest = manifest_problems(root);
    let mut tests = Vec::new();
    let mut failed = [false; 4];
    for (ids, level) in [(&LEVEL1[..], 1u8), (&LEVEL2[..], 2), (&LEVEL3[..], 3)] {
        for id in ids {
            let e = run_case(root, id, level);
            if e["result"] != "pass" {
                failed[level as usize] = true;
            }
            tests.push(e);
        }
    }
    let passed = tests.iter().filter(|t| t["result"] == "pass").count();
    let level = match (manifest.is_empty(), failed[1], failed[2], failed[3]) {
        (false, ..) | (true, true, ..) => 0,
        (true, false, true, _) => 1,
        (true, false, false, true) => 2,
        (true, false, false, false) => 3,
    };
    Ok(json!({
        "spec_version": SPEC_VERSION,
        "mdk_version": crate::tools::MDK_VERSION,
        "level_claimed": level,
        "fixtures_root_source": source,
        "fixtures_manifest_hash": manifest_hash,
        "fixtures_manifest": if manifest.is_empty() { "ok" } else { "mismatch" },
        "fixtures_manifest_problems": manifest,
        "passed": passed,
        "total": TOTAL,
        "tests": tests,
    }))
}

/// `mdk conformance`: the embedded fixtures (extracted into a scratch
/// directory) or `fixtures`; `models` sets the embedder files for Level 3.
pub fn command(fixtures: Option<PathBuf>, models: Option<PathBuf>) -> Result<Value> {
    if let Some(m) = models {
        let m = fs::canonicalize(&m).with_context(|| format!("--models {}", m.display()))?;
        // read by the embedder loader (`mdkindex::models::models_root`);
        // set before any thread starts
        std::env::set_var("MDK_MODELS", m);
    }
    match fixtures {
        Some(f) => {
            let root =
                fs::canonicalize(&f).with_context(|| format!("--fixtures {}", f.display()))?;
            run(&root, "--fixtures")
        }
        None => {
            let dir = std::env::temp_dir().join(format!("mdk-conformance-{}", std::process::id()));
            if dir.exists() {
                fs::remove_dir_all(&dir)?;
            }
            fs::create_dir_all(&dir)?;
            let res = extract(&dir).and_then(|_| run(&dir, "embedded"));
            let _ = fs::remove_dir_all(&dir);
            res
        }
    }
}
