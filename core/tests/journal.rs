//! P1-B/B5: raw journal and version triggers (spec §11) — kernel bounds
//! (B-6: retention < 24 h, idle_ms < 0, min_sim_change ∉ [0, 1] → Err),
//! idle and distance triggers on a seeded time sequence, retention pruning
//! without events, unversioned content never indexable.

use mdkcore::{
    IdleOrDistance, JournalRetention, MatcherVersion, Provenance, TriggerInput, Vault,
    VersionTrigger, MIN_RETENTION_HOURS,
};
use std::fs;
use std::path::{Path, PathBuf};

const M: MatcherVersion = MatcherVersion::V3;
const HOUR: u64 = 3_600_000;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-journal-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn file_count(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    let mut n = 0;
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        n += if p.is_dir() { file_count(&p) } else { 1 };
    }
    n
}

#[test]
fn b6_kernel_bounds_are_errors() {
    // retention < 24 h
    for h in [-1, 0, 1, 23] {
        let e = JournalRetention::new(h).unwrap_err().to_string();
        assert!(e.contains("kernel minimum is 24"), "{e}");
    }
    assert_eq!(
        JournalRetention::new(24).unwrap().hours(),
        MIN_RETENTION_HOURS
    );
    // idle_ms < 0
    let e = IdleOrDistance::new(-1, 0.2).unwrap_err().to_string();
    assert!(e.contains("idle_ms = -1"), "{e}");
    // min_sim_change ∉ [0, 1]
    for x in [-0.01, 1.01, f64::NAN, f64::INFINITY] {
        let e = IdleOrDistance::new(1000, x).unwrap_err().to_string();
        assert!(e.contains("must lie in [0, 1]"), "{x}: {e}");
    }
    for x in [0.0, 0.5, 1.0] {
        IdleOrDistance::new(0, x).unwrap();
    }
}

/// splitmix64
fn rng(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Saves arrive at seeded intervals; each save is one more word. The idle
/// trigger (distance disabled with min_sim_change = 1) versions exactly when
/// the gap since the last save reaches idle_ms at the time it is checked;
/// the distance trigger (idle disabled with a huge idle_ms) versions exactly
/// when the edit distance to the last version reaches the threshold.
#[test]
fn seeded_idle_and_distance_triggers() {
    let words: Vec<String> = (0..60).map(|i| format!("word{i:02}")).collect();
    let text = |n: usize| format!("# Log\n\n{}\n", words[..n].join(" "));

    // --- idle only
    let root = tmp("idle");
    let mut v = Vault::open(&root).unwrap();
    let idle = IdleOrDistance::new(5_000, 1.0).unwrap();
    let mut seed = 20261014u64;
    let mut now = 1_000_000u64;
    let (mut versions, mut expected) = (0, 0);
    for n in 1..=40 {
        v.journal_append("log.md", text(n).as_bytes(), now).unwrap();
        // check after a seeded pause
        let pause = rng(&mut seed) % 10_000;
        let check_at = now + pause;
        let first = v.snapshot("log.md").unwrap().is_none();
        // distance is 1.0 only for the very first version (no previous)
        let want = pause >= 5_000 || first;
        let got = v
            .maybe_version("log.md", &idle, check_at, Provenance::default(), M)
            .unwrap();
        assert_eq!(got.is_some(), want, "save {n}, pause {pause}");
        if want {
            expected += 1;
        }
        if got.is_some() {
            versions += 1;
            // the version is the latest save, never less
            assert!(v.journal_versioned("log.md").unwrap());
        }
        // an already versioned save is never versioned twice
        if got.is_some() {
            assert!(v
                .maybe_version(
                    "log.md",
                    &idle,
                    check_at + 100_000,
                    Provenance::default(),
                    M
                )
                .unwrap()
                .is_none());
        }
        now = check_at + rng(&mut seed) % 2_000;
    }
    assert_eq!(versions, expected);
    assert!(versions > 3 && versions < 40, "{versions}");
    fs::remove_dir_all(&root).unwrap();

    // --- distance only
    let root = tmp("dist");
    let mut v = Vault::open(&root).unwrap();
    let dist = IdleOrDistance::new(i64::MAX, 0.10).unwrap();
    let mut last_version: Option<String> = None;
    let mut versions = 0;
    let mut now = 0u64;
    for n in 1..=60 {
        now += 1 + rng(&mut seed) % 50;
        v.journal_append("log.md", text(n).as_bytes(), now).unwrap();
        let cur = words[..n].join(" ");
        let current = format!("Log\n{cur}");
        let want = match &last_version {
            None => true,
            Some(p) => IdleOrDistance::distance(p, &current) >= 0.10,
        };
        let got = v
            .maybe_version("log.md", &dist, now, Provenance::default(), M)
            .unwrap();
        assert_eq!(got.is_some(), want, "save {n}");
        if want {
            last_version = Some(current);
            versions += 1;
        }
    }
    assert!(versions > 3 && versions < 60, "{versions}");
    // trigger input as the kernel builds it
    let input = TriggerInput {
        now_ms: 10,
        last_save_ms: 0,
        previous: Some("a"),
        current: "a",
    };
    assert!(!dist.should_version(&input));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn unversioned_content_is_never_indexable() {
    let root = tmp("index");
    let mut v = Vault::open(&root).unwrap();
    v.journal_append("n.md", b"# N\n\nDraft one.\n", 0).unwrap();
    assert_eq!(v.indexable_snapshot("n.md").unwrap(), None);
    assert!(!v.journal_versioned("n.md").unwrap());
    let never = IdleOrDistance::new(i64::MAX, 1.0).unwrap();
    // a first version has distance 1 from nothing: versioned at once
    let r = v
        .maybe_version("n.md", &never, 1, Provenance::default(), M)
        .unwrap()
        .unwrap();
    v.journal_append("n.md", b"# N\n\nDraft one, edited.\n", 2)
        .unwrap();
    assert!(v
        .maybe_version("n.md", &never, 3, Provenance::default(), M)
        .unwrap()
        .is_none());
    // the index still sees the version, not the newer save
    assert_eq!(v.indexable_snapshot("n.md").unwrap().unwrap(), r.snapshot);
    assert!(!v.journal_versioned("n.md").unwrap());
    // bad input
    assert!(v.journal_append("n.md", b"\xff\xfe", 4).is_err());
    assert!(
        v.journal_append("n.md", b"x", 1).is_err(),
        "time goes backwards"
    );
    assert!(v.journal_append("", b"x", 9).is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn retention_pruning_below_and_above_and_no_events() {
    let root = tmp("prune");
    let mut v = Vault::open(&root).unwrap();
    let ret = JournalRetention::new(24).unwrap();
    let always = IdleOrDistance::new(0, 0.0).unwrap();
    // five saves one hour apart, the last one versioned
    for i in 0..5u64 {
        let t = format!("# P\n\nSave number {i}.\n");
        v.journal_append("p.md", t.as_bytes(), i * HOUR).unwrap();
    }
    v.maybe_version("p.md", &always, 4 * HOUR, Provenance::default(), M)
        .unwrap()
        .unwrap();
    let log = fs::read(v.events_path()).unwrap();
    let blobs = root.join(".mdkernel/journal/blobs");
    assert_eq!(file_count(&blobs), 5);

    // all saves younger than 24 h: nothing pruned
    let rep = v.journal_prune(24 * HOUR, ret).unwrap();
    assert_eq!((rep.records_removed, rep.records_kept), (0, 5));
    // saves 0 and 1 are older than 24 h at t = 25.5 h: pruned; 2..4 kept
    let rep = v.journal_prune(25 * HOUR + HOUR / 2, ret).unwrap();
    assert_eq!(
        (rep.records_removed, rep.records_kept, rep.blobs_removed),
        (2, 3, 2)
    );
    let seqs: Vec<u64> = v.journal("p.md").unwrap().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, vec![2, 3, 4]);
    assert_eq!(file_count(&blobs), 3);
    // far later: only the latest save survives
    let rep = v.journal_prune(1000 * HOUR, ret).unwrap();
    assert_eq!(rep.records_kept, 1);
    assert_eq!(v.journal("p.md").unwrap()[0].seq, 4);
    // pruning wrote no event
    assert_eq!(fs::read(v.events_path()).unwrap(), log);
    // appends continue the sequence
    assert_eq!(
        v.journal_append("p.md", b"# P\n\nNext.\n", 1001 * HOUR)
            .unwrap()
            .seq,
        5
    );

    // an unversioned latest save protects the whole journal
    v.journal_append("q.md", b"# Q\n\nOld.\n", 0).unwrap();
    v.maybe_version("q.md", &always, 0, Provenance::default(), M)
        .unwrap()
        .unwrap();
    v.journal_append("q.md", b"# Q\n\nNew, unversioned.\n", HOUR)
        .unwrap();
    let rep = v.journal_prune(10_000 * HOUR, ret).unwrap();
    assert_eq!(v.journal("q.md").unwrap().len(), 2, "{rep:?}");
    fs::remove_dir_all(&root).unwrap();
}
