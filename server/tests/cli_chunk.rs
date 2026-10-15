//! CLI half of `core/tests/chunk.rs` (moved with the `mdk` binary in
//! P1-D/D0): `ingest-dir`, `ingest`, `replay`, `chunk`, `resolve`,
//! `events-hash` through `mdk`.

use std::path::PathBuf;
use std::process::Command;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mdk-chunk-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn mini_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault")
}

fn mdk(args: &[&str]) -> serde_json::Value {
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn cli_ingest_dir_twice_gives_same_events_hash() {
    let mv = mini_vault();
    let (a, b) = (tmp("cli-a"), tmp("cli-b"));
    let ra = mdk(&["ingest-dir", a.to_str().unwrap(), mv.to_str().unwrap()]);
    let rb = mdk(&["ingest-dir", b.to_str().unwrap(), mv.to_str().unwrap()]);
    assert_eq!(ra["ingested"].as_array().unwrap().len(), 12);
    assert_eq!(ra["ingested"], rb["ingested"]);
    let ha = mdk(&["events-hash", a.to_str().unwrap()]);
    let hb = mdk(&["events-hash", b.to_str().unwrap()]);
    assert_eq!(ha, hb);
    assert_eq!(ha["events_hash"], ra["events_hash"]);
    // ingest order is byte-wise doc_path order
    let docs: Vec<_> = ra["ingested"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["doc_path"].as_str().unwrap().to_string())
        .collect();
    let mut sorted = docs.clone();
    sorted.sort();
    assert_eq!(docs, sorted);
    // replay / chunk / resolve
    let rep = mdk(&["replay", a.to_str().unwrap(), "espresso.md"]);
    let vid = rep["blocks"][0][1].as_str().unwrap().to_string();
    let lid = rep["blocks"][0][0].as_str().unwrap().to_string();
    let res = mdk(&["resolve", a.to_str().unwrap(), "espresso.md", &vid, "0"]);
    assert_eq!(res["lineage_id"], lid);
    let ch = mdk(&[
        "chunk",
        a.to_str().unwrap(),
        "espresso.md",
        "--strategy",
        "fixed",
    ]);
    assert_eq!(ch[0]["strategy"], "fixed");
    let pretty = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["--pretty", "events-hash", a.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&pretty.stdout).contains("\n  \"events_hash\""));
    let bad = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(["resolve", a.to_str().unwrap(), "espresso.md", "0000", "0"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
}

#[test]
fn cli_ingest_files_relative_to_vault() {
    let root = tmp("files");
    std::fs::create_dir_all(root.join("notes")).unwrap();
    std::fs::write(root.join("notes/a.md"), "# A\n\ntext\n").unwrap();
    let r = mdk(&[
        "ingest",
        root.to_str().unwrap(),
        root.join("notes/a.md").to_str().unwrap(),
    ]);
    assert_eq!(r["ingested"][0]["doc_path"], "notes/a.md");
}
