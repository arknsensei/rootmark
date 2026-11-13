//! P2-Q.4/R3 (PQ3-4; `docs/rounds/P2-Q.4/progress/TESTS.md` C2, DESIGN
//! §5): the real client's stop sequence (C1 of `watch_restart.rs`: SIGINT,
//! SIGTERM, ~0.5 s, SIGKILL) with the SIGKILL forced on every platform —
//! the test setting `ROOTMARK_WATCH_TEST_STOP_DELAY_MS` slows the stop, so
//! the process is alive at 0.5 s and the SIGKILL lands in the stop, before
//! the held removal is taken. The next start takes it. C1's checks, plus:
//! SIGKILL sent in every repetition. Each case runs
//! `ROOTMARK_RESTART_REPS` times (20 by default); macOS and Linux.

mod restartkit;

use restartkit::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

struct Base {
    root: PathBuf,
    base: PathBuf,
}

impl Base {
    fn new(name: &str, policy: &str) -> Base {
        let root = tmp(&format!("forced-{name}"));
        let base = root.join("base");
        std::fs::create_dir_all(&base).unwrap();
        base_vault(&base, policy);
        Base { root, base }
    }

    fn vault(&self, i: usize) -> PathBuf {
        let v = self.root.join(format!("v{i:02}"));
        copy_tree(&self.base, &v);
        v
    }
}

impl Drop for Base {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn lineages(evs: &[Value], doc: &str) -> Vec<String> {
    live(evs)
        .get(doc)
        .map(|s| s.iter().cloned().collect())
        .unwrap_or_default()
}

fn assert_closed(v: &Path, doc: &str, lin: &[String], policy: &str) {
    let evs = events(v);
    let mut d = deleted(&evs, doc);
    d.sort();
    let mut want = lin.to_vec();
    want.sort();
    assert_eq!(d, want, "{doc}: BlockDeleted once per lineage");
    let (kind, id, source) = match policy {
        "unverified" => ("unattested_fs", "fs", "unattested_fs"),
        _ => ("human", "tester", "rootmark-serve-watch"),
    };
    for e in of_kind(&evs, "BlockDeleted", doc) {
        assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
        assert_eq!(e["meta"]["actor"]["id"], id, "{e}");
        assert_eq!(e["meta"]["source"], source, "{e}");
    }
}

fn forced_sequence(case: &str, policy: &str) {
    let b = Base::new(case, policy);
    repeat(case, |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n0.md");
        let mut sv = serve_env(&v, &[("ROOTMARK_WATCH_TEST_STOP_DELAY_MS", "2000")]);
        assert!(
            sv.log()
                .contains("test setting ROOTMARK_WATCH_TEST_STOP_DELAY_MS=2000"),
            "{}",
            sv.log()
        );
        std::fs::remove_file(v.join("n0.md")).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        sv.signal("INT");
        sv.signal("TERM");
        std::thread::sleep(Duration::from_millis(500));
        let alive = sv.alive();
        assert!(alive, "the slowed stop ended before 0.5 s: {}", sv.log());
        sv.signal("KILL");
        let (_, by_signal, _) = sv.wait();
        assert!(by_signal, "not killed");
        let at_stop = !deleted(&events(&v), "n0.md").is_empty();
        assert!(!at_stop, "the removal was taken before the SIGKILL");
        let sv = serve(&v);
        wait_for(LIMIT, "n0.md's removal ingested", || {
            let d = deleted(&events(&v), "n0.md");
            lin.iter().all(|l| d.contains(l))
        });
        settle(&v, Duration::from_millis(600));
        assert_closed(&v, "n0.md", &lin, policy);
        assert!(sv.blame("n0.md").is_empty());
        assert!(sv.hits("zqr0").is_empty());
        sv.stop();
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({"sigkill_sent": alive, "taken_at_stop": at_stop})
    });
}

#[test]
fn c2_forced_sigkill_canonical() {
    forced_sequence("c2_canonical", "canonical");
}

#[test]
fn c2_forced_sigkill_unverified() {
    forced_sequence("c2_unverified", "unverified");
}

/// A value of the setting that is not a number of milliseconds stops the
/// start (never ignored).
#[test]
fn c2_bad_setting_is_an_error() {
    let b = Base::new("c2_bad", "canonical");
    let v = b.vault(0);
    let o = Command::new(rootmark())
        .args([
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--http",
            "127.0.0.1:0",
            "--watch",
        ])
        .env("ROOTMARK_WATCH_TEST_STOP_DELAY_MS", "x")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success(), "{err}");
    assert!(
        err.contains("ROOTMARK_WATCH_TEST_STOP_DELAY_MS=\"x\" is not a number of milliseconds"),
        "{err}"
    );
    record("c2_bad", 1, 1, json!({}));
}
