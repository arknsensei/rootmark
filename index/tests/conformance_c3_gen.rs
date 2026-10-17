//! Writes `spec/fixtures/C3-*/expected.json` from the current implementation
//! (TASKS C7). Ignored in normal runs; `scripts/gen_fixtures_c3.sh` runs it
//! with `ROOTMARK_FIXTURES`, then the frozen `scripts/gen_fixtures.sh` rewrites
//! `MANIFEST.json` (C1 files byte-identical, C2 and C3 files listed).

#[path = "conformance/eval_c3.rs"]
mod eval_c3;

use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "fixture generator: scripts/gen_fixtures_c3.sh"]
fn write_c3_expected() {
    let root = PathBuf::from(std::env::var("ROOTMARK_FIXTURES").expect("ROOTMARK_FIXTURES"));
    let dirs = eval_c3::c3_case_dirs(&root).unwrap();
    assert_eq!(dirs.len(), 7, "C3-01..C3-07");
    for d in dirs {
        let v = eval_c3::evaluate_c3(&d).unwrap_or_else(|e| panic!("{}: {e:#}", d.display()));
        let mut text = serde_json::to_string_pretty(&v).unwrap();
        text.push('\n');
        fs::write(d.join("expected.json"), text).unwrap();
        eprintln!("wrote {}", d.join("expected.json").display());
    }
}
