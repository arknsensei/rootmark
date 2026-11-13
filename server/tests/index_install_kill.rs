//! P2-Q.4/R2 (PQ3-7; `docs/rounds/P2-Q.4/progress/TESTS.md` I1–I6, DESIGN
//! §3): the index install is killed at each of its steps — deterministically,
//! with the test setting `ROOTMARK_TEST_ABORT_AT` (the process aborts at the
//! named step, as a SIGKILL there would leave it) — in both installs, a
//! `rootmark index build` and the shutdown compaction of `rootmark serve`.
//! The next start finds either the complete previous index or the complete
//! new one, or an incomplete one that it rebuilds with a line saying so;
//! `search` answers 200 with the results of a clean build, and the index it
//! leaves is the bytes a clean `rootmark index build` writes. Each case runs
//! `ROOTMARK_RESTART_REPS` times (20 by default); macOS and Linux. Needs the
//! model files (`make models`).

mod restartkit;

use restartkit::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

struct Base {
    root: PathBuf,
    base: PathBuf,
}

impl Base {
    fn new(name: &str, extra: &str) -> Base {
        let root = tmp(&format!("inst-{name}"));
        let base = root.join("base");
        std::fs::create_dir_all(&base).unwrap();
        base_vault(&base, "canonical");
        if !extra.is_empty() {
            // a line of the recipe's `[index]` table
            let p = base.join(".rootmark/policy.toml");
            let r = std::fs::read_to_string(&p).unwrap();
            assert!(r.contains("\n[index]\n"), "{r}");
            write(
                &p,
                &r.replacen("\n[index]\n", &format!("\n[index]\n{extra}"), 1),
            );
        }
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

fn tree_bytes(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, d: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                walk(root, &e.path(), out);
            } else {
                let rel = e
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                out.insert(rel, std::fs::read(e.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn index_of(v: &Path) -> BTreeMap<String, Vec<u8>> {
    tree_bytes(&v.join(".rootmark/index"))
}

fn assert_same_index(
    got: &BTreeMap<String, Vec<u8>>,
    want: &BTreeMap<String, Vec<u8>>,
    what: &str,
) {
    let names = |m: &BTreeMap<String, Vec<u8>>| m.keys().cloned().collect::<Vec<_>>();
    assert_eq!(names(got), names(want), "{what}: index files");
    for (k, b) in want {
        assert!(got[k] == *b, "{what}: {k}: bytes differ");
    }
}

/// A copy of `v` with its index rebuilt by a clean `rootmark index build`.
fn clean_copy(v: &Path) -> PathBuf {
    let r = v.with_extension("ref");
    copy_tree(v, &r);
    let _ = std::fs::remove_dir_all(r.join(".rootmark/index"));
    let _ = std::fs::remove_dir_all(r.join(".rootmark/index.old"));
    let _ = std::fs::remove_dir_all(r.join(".rootmark/index.build"));
    run_ok(&["index", "build", r.to_str().unwrap()]);
    r
}

/// The searches compared with the clean build: lexical and hybrid; each
/// hit's document, chunk and score.
fn results(sv: &Served) -> Vec<Value> {
    let mut out = Vec::new();
    for (q, mode) in [
        ("zqr1", "lexical"),
        ("zqextra", "lexical"),
        ("synthetic note topic", "hybrid"),
    ] {
        let (st, v) = sv.http(
            "POST",
            "/v1/search",
            &json!({"query": q, "mode": mode, "k": 20, "include_proposal": true}),
        );
        assert_eq!(st, 200, "search {q:?} ({mode}): {v}; log:\n{}", sv.log());
        let hits: Vec<Value> = v["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| json!([h["doc_path"], h["chunk_id"], h["score"]]))
            .collect();
        out.push(json!({"q": q, "mode": mode, "hits": hits}));
    }
    out
}

/// `rootmark <args>` with `ROOTMARK_TEST_ABORT_AT=<step>`: it must end by
/// the abort (a signal), saying so.
fn aborted(args: &[&str], step: &str) {
    let o = Command::new(rootmark())
        .args(args)
        .env("ROOTMARK_TEST_ABORT_AT", step)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        !o.status.success(),
        "rootmark {args:?} did not abort: {err}"
    );
    assert!(
        err.contains(&format!("ROOTMARK_TEST_ABORT_AT={step}: aborting")),
        "{err}"
    );
}

/// What the restart must say for the install killed at `step`, given that
/// the previous index did not hold the log's heads (`behind`).
fn check_start_lines(log: &str, step: &str, behind: bool) -> Value {
    let restored = log.contains("rootmark serve: index: an interrupted install left the previous index at .rootmark/index.old: restored");
    let removed = log
        .contains("rootmark serve: index: removed the previous index left at .rootmark/index.old");
    let rebuilt = log
        .lines()
        .any(|l| l.contains("rootmark serve: index incomplete (") && l.contains("rebuilt"));
    match step {
        "install:before_swap" => {
            assert!(!restored && !removed, "{log}");
            assert_eq!(rebuilt, behind, "{log}");
        }
        "install:swapped" => {
            assert!(restored && !removed, "{log}");
            assert_eq!(rebuilt, behind, "{log}");
        }
        "install:installed" => {
            assert!(removed && !restored, "{log}");
            assert!(!rebuilt, "{log}");
        }
        _ => unreachable!(),
    }
    json!({"restored": restored, "removed": removed, "rebuilt": rebuilt})
}

/// The checks after the restart (TESTS I1/I2): search 200 with the clean
/// build's results, no `index.old`/`index.build`, and after the stop the
/// index bytes of a clean build.
fn check_recovered(v: &Path, step: &str, behind: bool) -> Value {
    let r = clean_copy(v);
    let sv = serve(v);
    let rs = serve(&r);
    let got = results(&sv);
    let want = results(&rs);
    assert_eq!(got, want, "search results differ from a clean build");
    let log = sv.stop();
    rs.stop();
    let lines = check_start_lines(&log, step, behind);
    assert!(!v.join(".rootmark/index.old").exists());
    assert!(!v.join(".rootmark/index.build").exists());
    assert_same_index(
        &index_of(v),
        &index_of(&r),
        "after the stop vs the clean build",
    );
    std::fs::remove_dir_all(&r).unwrap();
    lines
}

/// I1: `rootmark index build` killed at `step`; the index it replaced was
/// behind the log (a note ingested with `ingest-dir`).
fn build_case(step: &str) {
    let case = format!("i1_{}", step.trim_start_matches("install:"));
    let b = Base::new(&case, "");
    repeat(&case, |i| {
        let v = b.vault(i);
        write(
            &v.join("extra.md"),
            "# Extra\n\nA note added later, word zqextra.\n",
        );
        let vs = v.to_str().unwrap();
        run_ok(&["ingest-dir", vs, vs]);
        aborted(&["index", "build", vs], step);
        let x = check_recovered(&v, step, true);
        std::fs::remove_dir_all(&v).unwrap();
        x
    });
}

#[test]
fn i1_index_build_killed_before_swap() {
    build_case("install:before_swap");
}

#[test]
fn i1_index_build_killed_between_renames() {
    build_case("install:swapped");
}

#[test]
fn i1_index_build_killed_after_rename() {
    build_case("install:installed");
}

/// I2: the shutdown compaction's install killed at `step`; the hot tier
/// held a note the watcher took.
fn compact_case(step: &str) {
    let case = format!("i2_{}", step.trim_start_matches("install:"));
    let b = Base::new(&case, "");
    repeat(&case, |i| {
        let v = b.vault(i);
        let sv = serve_env(&v, &[("ROOTMARK_TEST_ABORT_AT", step)]);
        write(
            &v.join("extra.md"),
            "# Extra\n\nA note added later, word zqextra.\n",
        );
        wait_for(LIMIT, "extra.md in the hot tier", || {
            std::fs::read_to_string(v.join(".rootmark/index/hot/snapshots.jsonl"))
                .is_ok_and(|t| t.contains("\"extra.md\""))
        });
        settle(&v, Duration::from_millis(300));
        sv.signal("TERM");
        let (ok, by_signal, log) = sv.wait();
        assert!(!ok && by_signal, "the compaction did not abort: {log}");
        assert!(
            log.contains(&format!("ROOTMARK_TEST_ABORT_AT={step}: aborting")),
            "{log}"
        );
        let x = check_recovered(&v, step, false);
        std::fs::remove_dir_all(&v).unwrap();
        x
    });
}

#[test]
fn i2_compaction_killed_before_swap() {
    compact_case("install:before_swap");
}

#[test]
fn i2_compaction_killed_between_renames() {
    compact_case("install:swapped");
}

#[test]
fn i2_compaction_killed_after_rename() {
    compact_case("install:installed");
}

/// I3: an `index.old` left beside a complete index is removed at the start;
/// the index is untouched.
#[test]
fn i3_leftover_index_old_is_removed() {
    let b = Base::new("i3", "");
    repeat("i3", |i| {
        let v = b.vault(i);
        let before = index_of(&v);
        write(&v.join(".rootmark/index.old/records.jsonl"), "left over\n");
        let sv = serve(&v);
        let log = sv.stop();
        assert!(
            log.contains(
                "rootmark serve: index: removed the previous index left at .rootmark/index.old"
            ),
            "{log}"
        );
        assert!(!v.join(".rootmark/index.old").exists());
        assert_same_index(&index_of(&v), &before, "untouched");
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

/// One file of the lexical index removed, the manifest kept: what an older
/// rootmark's install killed in its `remove_dir_all` left (PQ3-7).
fn half_removed(v: &Path) -> String {
    let d = v.join(".rootmark/index/lexical");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&d)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    let f = files
        .into_iter()
        .find(|p| p.extension().is_some_and(|e| e == "store"))
        .expect("a .store file");
    std::fs::remove_file(&f).unwrap();
    f.file_name().unwrap().to_str().unwrap().to_string()
}

/// I4: an older rootmark's half-removed index is rebuilt at the start, with
/// a line saying so; search 200; a clean build's bytes.
#[test]
fn i4_half_removed_index_is_rebuilt() {
    let b = Base::new("i4", "");
    repeat("i4", |i| {
        let v = b.vault(i);
        half_removed(&v);
        let r = clean_copy(&v);
        let sv = serve(&v);
        let rs = serve(&r);
        assert_eq!(results(&sv), results(&rs));
        let log = sv.stop();
        rs.stop();
        let line = log
            .lines()
            .find(|l| l.contains("rootmark serve: index incomplete ("))
            .unwrap_or_else(|| panic!("no `index incomplete` line: {log}"));
        assert!(line.contains("rebuilt"), "{line}");
        assert_same_index(&index_of(&v), &index_of(&r), "rebuilt vs clean build");
        std::fs::remove_dir_all(&r).unwrap();
        std::fs::remove_dir_all(&v).unwrap();
        json!({"line": line.split(" (").next().unwrap_or("")})
    });
}

/// I5: with `[index] auto_rebuild = false` the incomplete index is left as
/// it is, with a line saying so.
#[test]
fn i5_incomplete_index_without_auto_rebuild_is_left() {
    let b = Base::new("i5", "auto_rebuild = false\n");
    repeat("i5", |i| {
        let v = b.vault(i);
        half_removed(&v);
        let before = index_of(&v);
        let sv = serve(&v);
        // the shutdown compaction of an index that does not open fails:
        // exit 1 (contract §3, as before this round); recorded, not asserted
        sv.signal("TERM");
        let (ok, _, log) = sv.wait();
        assert!(
            log.lines()
                .any(|l| l.contains("rootmark serve: index incomplete (")
                    && l.contains(
                        "[index] auto_rebuild = false: rebuild it with `rootmark index build`"
                    )),
            "{log}"
        );
        assert_same_index(&index_of(&v), &before, "left as it is");
        std::fs::remove_dir_all(&v).unwrap();
        json!({"exit_0": ok})
    });
}

/// I6: a value of the test setting that names no step is an error; the
/// index is untouched.
#[test]
fn i6_unknown_abort_step_is_an_error() {
    let b = Base::new("i6", "");
    let v = b.vault(0);
    let before = index_of(&v);
    let o = Command::new(rootmark())
        .args(["index", "build", v.to_str().unwrap()])
        .env("ROOTMARK_TEST_ABORT_AT", "bogus")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success(), "{err}");
    assert!(
        err.contains("ROOTMARK_TEST_ABORT_AT=\"bogus\" names no step"),
        "{err}"
    );
    assert_same_index(&index_of(&v), &before, "untouched");
    record("i6", 1, 1, json!({}));
}
