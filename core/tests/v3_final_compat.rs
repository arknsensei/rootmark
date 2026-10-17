//! P1-A/A2: the moved core reproduces the v3-final core byte for byte.
//!
//! `fixtures/v3_final/mini_vault_edit.{v2,v3}.events.jsonl` were written by
//! the `rootmark` binary built from `git archive v3-final core` (prototype sha
//! 71c244e, docs/rounds/P1-A/progress/SOURCE.md):
//!   rootmark --matcher <m> ingest-dir V spec/fixtures/mini_vault
//!   rootmark --matcher <m> ingest-dir V core/tests/fixtures/v3_final/edit
//! The second pass moves, splits, updates and deletes blocks, so both
//! matcher paths are exercised, not only first ingest.

use rootmarkcore::{AuthorKind, Layer, MatcherVersion, Provenance, Vault};
use std::fs;
use std::path::{Path, PathBuf};

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-compat-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// Returns (events_hash after the first pass, events.jsonl bytes after both).
fn run(m: MatcherVersion, name: &str) -> (String, Vec<u8>) {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&here("../spec/fixtures/mini_vault"), m)
        .unwrap();
    let first = v.events_hash().unwrap();
    v.ingest_dir(&here("tests/fixtures/v3_final/edit"), m)
        .unwrap();
    let bytes = fs::read(root.join(".rootmark/events.jsonl")).unwrap();
    fs::remove_dir_all(&root).unwrap();
    (first, bytes)
}

#[test]
fn v2_events_equal_v3_final_bytes() {
    let (first, bytes) = run(MatcherVersion::V2, "v2");
    assert_eq!(first, "8914f9b418ae360d49c85c951050f432"); // GOAL A-5
    let want = fs::read(here(
        "tests/fixtures/v3_final/mini_vault_edit.v2.events.jsonl",
    ))
    .unwrap();
    assert!(
        bytes == want,
        "matcher v2 events.jsonl differs from v3-final"
    );
}

#[test]
fn v3_events_equal_v3_final_bytes() {
    let (first, bytes) = run(MatcherVersion::V3, "v3");
    assert_eq!(first, "808a41f21a43e121d7d688fee9f5bc24"); // GOAL A-5
    let want = fs::read(here(
        "tests/fixtures/v3_final/mini_vault_edit.v3.events.jsonl",
    ))
    .unwrap();
    assert!(
        bytes == want,
        "matcher v3 events.jsonl differs from v3-final"
    );
}

#[test]
fn default_layer_by_author_kind_and_not_logged() {
    assert_eq!(Layer::default_for(AuthorKind::Human), Layer::Canonical);
    assert_eq!(Layer::default_for(AuthorKind::Agent), Layer::Proposal);
    assert_eq!(Layer::default_for(AuthorKind::Transcript), Layer::Proposal);
    assert_eq!(Layer::default_for(AuthorKind::Synthetic), Layer::Canonical);

    let root = tmp("layer");
    let mut v = Vault::open(&root).unwrap();
    let agent = Provenance {
        author_kind: AuthorKind::Agent,
        source: "claude-code".into(),
    };
    let r = v
        .ingest("a.md", "# A\n\nAgent text.\n", agent, MatcherVersion::V3)
        .unwrap();
    let blocks = v.snapshot_blocks(&r.snapshot).unwrap();
    assert!(blocks.iter().all(|b| b.layer == Layer::Proposal));
    let r = v
        .ingest(
            "b.md",
            "# B\n\nHuman text.\n",
            Provenance::default(),
            MatcherVersion::V3,
        )
        .unwrap();
    let blocks = v.snapshot_blocks(&r.snapshot).unwrap();
    assert!(blocks.iter().all(|b| b.layer == Layer::Canonical));
    // no layer in the log in P1-A (A-5: events.jsonl format unchanged)
    let log = fs::read_to_string(root.join(".rootmark/events.jsonl")).unwrap();
    assert!(!log.contains("layer") && !log.contains("canonical") && !log.contains("proposal"));
    fs::remove_dir_all(&root).unwrap();
}
