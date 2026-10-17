//! P1-E/E5 (GOAL E-7): `mdk conformance` runs Levels 1–3 on the fixtures
//! embedded in the binary — level 3, 34/34 — and refuses a fixture tree that
//! does not match its manifest. `scripts/conformance_equiv.sh` compares its
//! report with `make conformance`'s test by test. Needs the model files.

use std::path::Path;
use std::process::Command;

fn mdk() -> &'static str {
    env!("CARGO_BIN_EXE_mdk")
}

#[test]
fn embedded_suite_is_level_3() {
    let out = Command::new(mdk()).arg("conformance").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["level_claimed"], 3);
    assert_eq!(r["passed"], 34);
    assert_eq!(r["total"], 34);
    assert_eq!(r["fixtures_root_source"], "embedded");
    assert_eq!(r["fixtures_manifest"], "ok");
    assert_eq!(r["spec_version"], mdkserver::tools::SPEC_VERSION);
}

#[test]
fn version_flags() {
    let v = Command::new(mdk()).arg("--version").output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&v.stdout).trim(),
        format!("mdk {}", env!("CARGO_PKG_VERSION"))
    );
    let s = Command::new(mdk()).arg("--spec-version").output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&s.stdout).trim(),
        mdkserver::tools::SPEC_VERSION
    );
}

#[test]
fn a_changed_fixture_is_refused() {
    let root = std::env::temp_dir().join(format!("mdk-conf-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    mdkserver::conformance::extract(&root).unwrap();
    let f = root.join("C1-01/expected.json");
    assert!(Path::new(&f).is_file());
    let mut t = std::fs::read_to_string(&f).unwrap();
    t.push(' ');
    std::fs::write(&f, t).unwrap();
    let out = Command::new(mdk())
        .args(["conformance", "--fixtures", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["fixtures_manifest"], "mismatch");
    assert_eq!(r["level_claimed"], 0);
    let _ = std::fs::remove_dir_all(&root);
}
