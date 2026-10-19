//! P1-E/E5 (GOAL E-7): `rootmark conformance` runs Levels 1–3 on the fixtures
//! embedded in the binary — level 3, 39/39 since P2-0.2 (C2-14, K-P2-02.3; 38/38 in P2-0.1, 37/37 in P2-0) — and refuses a fixture tree that
//! does not match its manifest. `scripts/conformance_equiv.sh` compares its
//! report with `make conformance`'s test by test. Needs the model files.

use std::path::Path;
use std::process::Command;

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

#[test]
fn embedded_suite_is_level_3() {
    let out = Command::new(rootmark())
        .arg("conformance")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["level_claimed"], 3);
    assert_eq!(r["passed"], 39);
    assert_eq!(r["total"], 39);
    assert_eq!(r["fixtures_root_source"], "embedded");
    assert_eq!(r["fixtures_manifest"], "ok");
    assert_eq!(r["spec_version"], rootmarkserver::tools::SPEC_VERSION);
}

#[test]
fn version_flags() {
    let v = Command::new(rootmark()).arg("--version").output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&v.stdout).trim(),
        format!("rootmark {}", env!("CARGO_PKG_VERSION"))
    );
    let s = Command::new(rootmark())
        .arg("--spec-version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&s.stdout).trim(),
        rootmarkserver::tools::SPEC_VERSION
    );
}

#[test]
fn a_changed_fixture_is_refused() {
    let root = std::env::temp_dir().join(format!("rootmark-conf-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    rootmarkserver::conformance::extract(&root).unwrap();
    let f = root.join("C1-01/expected.json");
    assert!(Path::new(&f).is_file());
    let mut t = std::fs::read_to_string(&f).unwrap();
    t.push(' ');
    std::fs::write(&f, t).unwrap();
    let out = Command::new(rootmark())
        .args(["conformance", "--fixtures", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["fixtures_manifest"], "mismatch");
    assert_eq!(r["level_claimed"], 0);
    let _ = std::fs::remove_dir_all(&root);
}
