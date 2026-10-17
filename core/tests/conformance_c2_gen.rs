//! Writes `spec/fixtures/C2-*/expected.json` from the **current** core
//! (TASKS B7: "beklenen değerler mevcut çekirdekten üretilir"). Ignored in
//! normal runs; `scripts/gen_fixtures_c2.sh` runs it with `ROOTMARK_FIXTURES`,
//! then the frozen `scripts/gen_fixtures.sh` rewrites `MANIFEST.json` (C1
//! files byte-identical, C2 files listed).

#[path = "conformance/eval_c2.rs"]
mod eval_c2;

use std::fs;
use std::path::PathBuf;

#[test]
#[ignore = "fixture generator: scripts/gen_fixtures_c2.sh"]
fn write_c2_expected() {
    let root = PathBuf::from(std::env::var("ROOTMARK_FIXTURES").expect("ROOTMARK_FIXTURES"));
    let dirs = eval_c2::c2_case_dirs(&root).unwrap();
    assert_eq!(dirs.len(), 10, "C2-01..C2-10");
    for d in dirs {
        let v = eval_c2::evaluate_c2(&d).unwrap_or_else(|e| panic!("{}: {e:#}", d.display()));
        let mut text = serde_json::to_string_pretty(&v).unwrap();
        text.push('\n');
        fs::write(d.join("expected.json"), text).unwrap();
        eprintln!("wrote {}", d.join("expected.json").display());
    }
}
