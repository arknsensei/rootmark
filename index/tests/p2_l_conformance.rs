//! P2-L/L-10 (GOAL P2-L L-10): the Level 3 cases (`spec/fixtures/C3-*`)
//! with both lexical variants, through the frozen evaluator
//! (`conformance/eval_c3.rs`, included as it is):
//! - `mdk_lower_alnum` (no knob): every case's output equals its committed
//!   `expected.json` — the 39 tests of `make conformance` are unchanged;
//! - `uni_fold_tr` (`ROOTMARK_DEFAULT_TOKENIZER`, the evaluator's scratch
//!   vaults have no recipe): every case's output equals the new fixture
//!   `spec/fixtures_variants/uni_fold_tr/<case>/expected.json`, written by
//!   `write_uni_fold_tr_expected` (ignored; `make fixtures-variants`) from
//!   this implementation; a build under the knob records `uni_fold_tr`.
//!   (Measured: the eight English cases rank the same with either variant.)
//!
//! One test function: the knob is a process environment variable.
//!
//! Needs the model files (`make models`).

#[path = "conformance/eval_c3.rs"]
mod eval_c3;

use rootmarkindex::config::DEFAULT_TOKENIZER_ENV;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures")
}

fn variant_dir(case: &Path) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../spec/fixtures_variants/uni_fold_tr")
        .join(case.file_name().unwrap())
}

fn read(p: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display())))
        .unwrap()
}

#[test]
fn level_3_with_both_lexical_variants() {
    let dirs = eval_c3::c3_case_dirs(&fixtures()).unwrap();
    assert_eq!(dirs.len(), 8, "C3-01..C3-08");
    std::env::remove_var(DEFAULT_TOKENIZER_ENV);
    let mut default = Vec::new();
    for d in &dirs {
        let got = eval_c3::evaluate_c3(d).unwrap_or_else(|e| panic!("{}: {e:#}", d.display()));
        assert_eq!(
            got,
            read(&d.join("expected.json")),
            "{} (mdk_lower_alnum)",
            d.display()
        );
        default.push(got);
    }
    std::env::set_var(DEFAULT_TOKENIZER_ENV, "uni_fold_tr");
    let mut differ = 0;
    for (d, base) in dirs.iter().zip(&default) {
        let got = eval_c3::evaluate_c3(d).unwrap_or_else(|e| panic!("{}: {e:#}", d.display()));
        assert_eq!(
            got,
            read(&variant_dir(d).join("expected.json")),
            "{} (uni_fold_tr)",
            d.display()
        );
        differ += usize::from(&got != base);
    }
    // the knob is in force: a vault without a recipe builds uni_fold_tr
    let root = std::env::temp_dir().join(format!("rootmark-p2l10-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let mut v = rootmarkcore::Vault::open(&root).unwrap();
    v.ingest_dir(
        &fixtures().join("mini_vault"),
        rootmarkcore::MatcherVersion::V3,
    )
    .unwrap();
    let e = rootmarkindex::embed::OnnxMiniLm::load_default().unwrap();
    rootmarkindex::store::build(&v, &e).unwrap();
    let m = read(&root.join(".rootmark/index/manifest.json"));
    assert_eq!(m["lexical"]["tokenizer"], "uni_fold_tr");
    std::env::set_var(DEFAULT_TOKENIZER_ENV, "turkish");
    assert!(
        eval_c3::evaluate_c3(&dirs[0]).is_err(),
        "an unknown variant is an error"
    );
    std::env::remove_var(DEFAULT_TOKENIZER_ENV);
    eprintln!(
        "uni_fold_tr: {differ} of {} case outputs differ from mdk_lower_alnum's",
        dirs.len()
    );
}

#[test]
#[ignore = "fixture generator: make fixtures-variants"]
fn write_uni_fold_tr_expected() {
    let dirs = eval_c3::c3_case_dirs(&fixtures()).unwrap();
    std::env::set_var(DEFAULT_TOKENIZER_ENV, "uni_fold_tr");
    for d in &dirs {
        let v = eval_c3::evaluate_c3(d).unwrap_or_else(|e| panic!("{}: {e:#}", d.display()));
        let out = variant_dir(d);
        fs::create_dir_all(&out).unwrap();
        let mut text = serde_json::to_string_pretty(&v).unwrap();
        text.push('\n');
        fs::write(out.join("expected.json"), text).unwrap();
        eprintln!("wrote {}", out.join("expected.json").display());
    }
    std::env::remove_var(DEFAULT_TOKENIZER_ENV);
}
