//! P2-Q.4/R3 (PQ3-6; `docs/rounds/P2-Q.4/progress/TESTS.md` N1–N4, DESIGN
//! §4): only a path that is not there (`NotFound`) is a removal. A note
//! under a directory that cannot be read (`chmod 000`) keeps its blocks — at
//! the start's reconciliation, in a running watcher and in `ingest-dir` —
//! with a note line, and once the permission is back nothing new is
//! written. Each case runs `ROOTMARK_RESTART_REPS` times (20 by default);
//! macOS and Linux, as a user other than root (root reads through `chmod
//! 000`: the test then fails as no reproduction). Needs the model files.

mod restartkit;

use restartkit::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

struct Base {
    root: PathBuf,
    base: PathBuf,
}

impl Base {
    fn new(name: &str) -> Base {
        let root = tmp(&format!("noacc-{name}"));
        let base = root.join("base");
        std::fs::create_dir_all(&base).unwrap();
        base_vault(&base, "canonical");
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

fn mode(p: &Path, m: u32) {
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(m)).unwrap();
}

/// `V/sub` made unreadable; it must really be (not as root).
fn lock(v: &Path) {
    mode(&v.join("sub"), 0o000);
    assert!(
        std::fs::read_dir(v.join("sub")).is_err()
            && std::fs::symlink_metadata(v.join("sub/k.md")).is_err(),
        "chmod 000 does not stop this user (root?): not a reproduction"
    );
}

fn unlock(v: &Path) {
    mode(&v.join("sub"), 0o755);
}

/// `sub/k.md` keeps its blocks: no `BlockDeleted` of it.
fn assert_kept(v: &Path, what: &str) {
    let evs = events(v);
    assert!(
        deleted(&evs, "sub/k.md").is_empty(),
        "{what}: sub/k.md was closed"
    );
    assert!(live(&evs).contains_key("sub/k.md"), "{what}");
}

/// N1 + N3: the directory locked while the server is down; the start keeps
/// the note, says so; once unlocked, a restart writes nothing.
#[test]
fn n1_n3_locked_while_down() {
    let b = Base::new("n1");
    repeat("n1_n3", |i| {
        let v = b.vault(i);
        lock(&v);
        let sv = serve(&v);
        let s = sv.reconciled().unwrap();
        std::thread::sleep(HOLD * 2);
        settle(&v, std::time::Duration::from_millis(400));
        let log = sv.stop();
        unlock(&v);
        assert_kept(&v, "N1");
        assert!(
            log.lines()
                .any(|l| l.contains("sub/k.md: not read (") && l.contains("not taken as a removal")),
            "N1: no note: {log}"
        );
        assert_eq!(s.held, 0, "{s:?}");
        assert!(s.errors >= 1, "{s:?}");
        // N3: permission back, a restart writes nothing
        let before = log_bytes(&v);
        let sv = serve(&v);
        std::thread::sleep(HOLD * 2);
        settle(&v, std::time::Duration::from_millis(400));
        let s3 = sv.reconciled().unwrap();
        let log3 = sv.stop();
        assert_eq!(log_bytes(&v), before, "N3: new events; log: {log3}");
        assert_eq!((s3.changed, s3.held, s3.errors), (0, 0, 0), "{s3:?}");
        assert_kept(&v, "N3");
        std::fs::remove_dir_all(&v).unwrap();
        json!({"summary": format!("{s:?}")})
    });
}

/// N2: the directory locked and unlocked while the server runs: nothing.
#[test]
fn n2_locked_while_running() {
    let b = Base::new("n2");
    repeat("n2", |i| {
        let v = b.vault(i);
        let sv = serve(&v);
        let before = log_bytes(&v);
        lock(&v);
        std::thread::sleep(HOLD * 2);
        let mid = log_bytes(&v);
        unlock(&v);
        std::thread::sleep(HOLD * 2);
        settle(&v, std::time::Duration::from_millis(400));
        let after = log_bytes(&v);
        let log = sv.stop();
        assert_eq!(mid, before, "N2: events while locked; log: {log}");
        assert_eq!(after, before, "N2: events once unlocked; log: {log}");
        assert_kept(&v, "N2");
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

/// N4: `ingest-dir` with the directory locked closes nothing under it, and
/// says why (a note, or the core's read error).
#[test]
fn n4_ingest_dir_with_a_locked_directory() {
    let b = Base::new("n4");
    repeat("n4", |i| {
        let v = b.vault(i);
        lock(&v);
        let vs = v.to_str().unwrap();
        let (ok, _, err) = run(&["ingest-dir", vs, vs]);
        unlock(&v);
        assert_kept(&v, "N4");
        assert!(!err.trim().is_empty(), "N4: silent");
        if ok {
            assert!(err.contains("sub/k.md: not read ("), "N4: {err}");
        }
        std::fs::remove_dir_all(&v).unwrap();
        json!({"exit_ok": ok, "stderr_first": err.lines().next().unwrap_or("")
            .replace(vs, "<vault>")})
    });
}
