//! P2-Q.4/R1 (PQ3-9; `docs/rounds/P2-Q.4/progress/TESTS.md` H0–H4, DESIGN
//! §1–§2): a SIGKILL that lands while the watcher indexes a moved note —
//! after the event log took its new head and before, or between, the hot
//! tier's four appends (`records.jsonl`, `embedders.jsonl`, `vectors.f32`,
//! `snapshots.jsonl`) — leaves an index that does not match the log. The
//! next `rootmark serve` start must find it (`index incomplete … rebuilt`),
//! then take the old path's held removal and answer searches, and the index
//! it leaves must be the bytes a clean `rootmark index build` writes from
//! the same log.
//!
//! The disk state of each point is made after a real SIGKILL (the removal
//! still held): the hot tier files are cut back, in the order `index_head`
//! appends them, to their size before the append — the state the R0
//! diagnostic run observed after an abort at those points (DESIGN §1.2).
//! Each case runs `ROOTMARK_RESTART_REPS` times (20 by default); macOS and
//! Linux. Needs the model files (`make models`).

mod restartkit;

use restartkit::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The hot tier files in `index_head`'s append order.
const HOT_FILES: [&str; 4] = [
    "records.jsonl",
    "embedders.jsonl",
    "vectors.f32",
    "snapshots.jsonl",
];

/// How long the restarted server is given to take the held removal: the
/// hold is `window_s` (4 s here) + 0.5 s.
const CLOSE_LIMIT: Duration = Duration::from_secs(30);

/// Where the SIGKILL lands.
#[derive(Clone, Copy, Debug)]
enum Point {
    /// after these many of the four appends (0: the log is ahead)
    After(usize),
    /// in the middle of the first append (`records.jsonl` half written)
    MidFirst,
}

struct Base {
    root: PathBuf,
    base: PathBuf,
}

impl Base {
    fn new(name: &str, policy: &str) -> Base {
        let root = tmp(&format!("hot-{name}"));
        let base = root.join("base");
        std::fs::create_dir_all(&base).unwrap();
        base_vault(&base, policy);
        // a long hold: the moved note's index append is over before the kill
        let p = base.join(".rootmark/policy.toml");
        let r = std::fs::read_to_string(&p).unwrap();
        assert!(r.contains("window_s = 1.0"), "{r}");
        write(&p, &r.replace("window_s = 1.0", "window_s = 4.0"));
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

fn hot(v: &Path) -> PathBuf {
    v.join(".rootmark/index/hot")
}

/// The size of each hot file (`None`: no file).
fn sizes(v: &Path) -> Vec<Option<u64>> {
    HOT_FILES
        .iter()
        .map(|f| std::fs::metadata(hot(v).join(f)).ok().map(|m| m.len()))
        .collect()
}

/// Cuts the hot file `f` back to `size` (`None`: removes it).
fn cut(v: &Path, f: &str, size: Option<u64>) {
    let p = hot(v).join(f);
    match size {
        None => {
            let _ = std::fs::remove_file(&p);
        }
        Some(n) => {
            let fh = std::fs::OpenOptions::new().write(true).open(&p).unwrap();
            fh.set_len(n).unwrap();
            fh.sync_all().unwrap();
        }
    }
}

/// Makes the disk state of `point` from the state after the whole append
/// (`before`: the sizes before it).
fn tear(v: &Path, point: Point, before: &[Option<u64>]) {
    match point {
        Point::After(k) => {
            for (i, f) in HOT_FILES.iter().enumerate().skip(k) {
                cut(v, f, before[i]);
            }
        }
        Point::MidFirst => {
            let now = std::fs::metadata(hot(v).join(HOT_FILES[0])).unwrap().len();
            let was = before[0].unwrap_or(0);
            assert!(now > was + 1, "records.jsonl did not grow");
            cut(v, HOT_FILES[0], Some(was + (now - was) / 2));
            for (i, f) in HOT_FILES.iter().enumerate().skip(1) {
                cut(v, f, before[i]);
            }
        }
    }
}

/// Every file under `dir`, relative path → bytes.
fn tree_bytes(dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, d: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
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
    let mut out = Default::default();
    walk(dir, dir, &mut out);
    out
}

/// The vault's index equals what a clean `rootmark index build` writes from
/// the same log (on a copy of the vault), file by file.
pub fn assert_index_is_clean_build(v: &Path) {
    let r = v.with_extension("ref");
    copy_tree(v, &r);
    std::fs::remove_dir_all(r.join(".rootmark/index")).unwrap();
    run_ok(&["index", "build", r.to_str().unwrap()]);
    let got = tree_bytes(&v.join(".rootmark/index"));
    let want = tree_bytes(&r.join(".rootmark/index"));
    let names =
        |m: &std::collections::BTreeMap<String, Vec<u8>>| m.keys().cloned().collect::<Vec<_>>();
    assert_eq!(names(&got), names(&want), "index files");
    for (k, b) in &want {
        assert!(got[k] == *b, "{k}: bytes differ from a clean index build");
    }
    std::fs::remove_dir_all(&r).unwrap();
}

fn lineages(evs: &[Value], doc: &str) -> Vec<String> {
    live(evs)
        .get(doc)
        .map(|s| s.iter().cloned().collect())
        .unwrap_or_default()
}

fn actor_of(policy: &str) -> (&'static str, &'static str, &'static str) {
    match policy {
        "unverified" => ("unattested_fs", "fs", "unattested_fs"),
        _ => ("human", "tester", "rootmark-serve-watch"),
    }
}

fn torn_case(case: &str, policy: &str, point: Point) {
    let b = Base::new(case, policy);
    repeat(case, |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n0.md");
        assert_eq!(lin.len(), 3);
        let sv = serve(&v);
        assert_eq!(sv.hits("zqr0"), vec!["n0.md".to_string()]);
        let before = sizes(&v);
        std::fs::rename(v.join("n0.md"), v.join("sub/moved.md")).unwrap();
        // the moved note's head is in the log and its append is over
        wait_for(LIMIT, "sub/moved.md appended to the hot tier", || {
            std::fs::read_to_string(hot(&v).join("snapshots.jsonl"))
                .is_ok_and(|t| t.contains("\"sub/moved.md\""))
        });
        assert!(
            deleted(&events(&v), "n0.md").is_empty(),
            "removal taken before the kill (not a reproduction)"
        );
        let first = sv.kill();
        assert!(deleted(&events(&v), "n0.md").is_empty(), "{first}");
        tear(&v, point, &before);
        let torn = sizes(&v);
        let sv = serve(&v);
        let s = sv.reconciled();
        let t0 = std::time::Instant::now();
        loop {
            let d = deleted(&events(&v), "n0.md");
            if lin.iter().all(|l| d.contains(l)) {
                break;
            }
            assert!(
                t0.elapsed() < CLOSE_LIMIT,
                "timed out: n0.md's removal ingested; hot after the kill {torn:?}; the restart's log:\n{}",
                sv.log()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
        settle(&v, Duration::from_millis(600));
        let evs = events(&v);
        let mut d = deleted(&evs, "n0.md");
        d.sort();
        let mut want = lin.clone();
        want.sort();
        assert_eq!(d, want, "n0.md: BlockDeleted once per lineage");
        let (kind, id, source) = actor_of(policy);
        for e in of_kind(&evs, "BlockDeleted", "n0.md") {
            assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
            assert_eq!(e["meta"]["actor"]["id"], id, "{e}");
            assert_eq!(e["meta"]["source"], source, "{e}");
        }
        let (st, body) = sv.http(
            "POST",
            "/v1/search",
            &json!({"query": "zqr0", "mode": "lexical", "k": 20, "include_proposal": true}),
        );
        assert_eq!(
            st,
            200,
            "search after the restart: {body}; log:\n{}",
            sv.log()
        );
        assert_eq!(sv.hits("zqr0"), vec!["sub/moved.md".to_string()]);
        assert!(sv.blame("n0.md").is_empty());
        assert_eq!(sv.blame("sub/moved.md").len(), 3);
        if !legacy() {
            assert_eq!(s.map(|s| s.held), Some(1), "{s:?}");
        }
        let log = sv.stop();
        let line = log
            .lines()
            .find(|l| l.contains("rootmark serve: index incomplete ("))
            .unwrap_or_else(|| panic!("no `index incomplete` line: {log}"));
        assert!(line.contains("rebuilt"), "{line}");
        assert!(!v.join(".rootmark/index.old").exists());
        assert!(!hot(&v).exists(), "the stop compacts the hot tier");
        assert_index_is_clean_build(&v);
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({"point": format!("{point:?}"), "hot_after_kill": format!("{torn:?}"), "line": line})
    });
}

#[test]
fn h0_log_ahead_of_the_index() {
    torn_case("h0", "canonical", Point::After(0));
}

#[test]
fn h1_after_the_first_append() {
    torn_case("h1", "canonical", Point::After(1));
}

#[test]
fn h2_after_the_second_append() {
    torn_case("h2", "canonical", Point::After(2));
}

#[test]
fn h2_after_the_second_append_unverified() {
    torn_case("h2u", "unverified", Point::After(2));
}

#[test]
fn h3_after_the_third_append() {
    torn_case("h3", "canonical", Point::After(3));
}

#[test]
fn h4_in_the_middle_of_the_first_append() {
    torn_case("h4", "canonical", Point::MidFirst);
}
