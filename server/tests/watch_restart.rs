//! P2-Q.3/R1 (D0F-1/PQ2-1, D0F-5; `docs/rounds/P2-Q.3/progress/TESTS.md`
//! A1–G2, DESIGN §2–§3): `rootmark serve --watch` reconciles the vault with
//! the event log when it starts — a removal the watcher held when the
//! server was SIGKILLed, and every edit, deletion and addition made while
//! it was down, are taken at the next start, by spec §7.1's external
//! change rule; an unchanged vault writes nothing. Real process signals
//! (`kill -KILL/-TERM/-INT`) on a child `rootmark serve`. Each case runs
//! `ROOTMARK_RESTART_REPS` times (20 by default). Runs on macOS and Linux
//! (`make test`, CI). Needs the model files (`make models`).

mod restartkit;

use restartkit::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A prepared V0 for `policy`, copied fresh for each repetition.
struct Base {
    root: PathBuf,
    base: PathBuf,
}

impl Base {
    fn new(name: &str, policy: &str) -> Base {
        let root = tmp(name);
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

/// Every lineage in `lin` deleted exactly once in `doc`, with the actor
/// and source the policy gives.
fn assert_closed(v: &Path, doc: &str, lin: &[String], policy: &str) {
    let evs = events(v);
    let mut d = deleted(&evs, doc);
    d.sort();
    let mut want = lin.to_vec();
    want.sort();
    assert_eq!(d, want, "{doc}: BlockDeleted once per lineage");
    for e in of_kind(&evs, "BlockDeleted", doc) {
        let (kind, id, source) = actor_of(policy);
        assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
        assert_eq!(e["meta"]["actor"]["id"], id, "{e}");
        assert_eq!(e["meta"]["source"], source, "{e}");
    }
}

/// The actor (kind, id) and source of a watched external change.
fn actor_of(policy: &str) -> (&'static str, &'static str, &'static str) {
    match policy {
        "unverified" => ("unattested_fs", "fs", "unattested_fs"),
        _ => ("human", "tester", "rootmark-serve-watch"),
    }
}

/// Waits until `doc`'s lineages are all deleted.
fn wait_closed(v: &Path, doc: &str, lin: &[String]) {
    wait_for(LIMIT, &format!("{doc}'s removal ingested"), || {
        let d = deleted(&events(v), doc);
        lin.iter().all(|l| d.contains(l))
    });
}

/// A1/A2: a removal held when the server is SIGKILLed is taken at the next
/// start.
fn sigkill_case(case: &str, policy: &str, mv: bool) {
    let b = Base::new(case, policy);
    repeat(case, |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n0.md");
        assert_eq!(lin.len(), 3);
        let sv = serve(&v);
        assert_eq!(sv.hits("zqr0"), vec!["n0.md".to_string()]);
        if mv {
            std::fs::rename(v.join("n0.md"), v.join("sub/moved.md")).unwrap();
        } else {
            std::fs::remove_file(v.join("n0.md")).unwrap();
        }
        std::thread::sleep(Duration::from_millis(300));
        // the hold (1.5 s) is not over: the removal is not taken yet
        assert!(
            deleted(&events(&v), "n0.md").is_empty(),
            "removal taken before the kill (not a reproduction)"
        );
        sv.kill();
        assert!(deleted(&events(&v), "n0.md").is_empty());
        let sv = serve(&v);
        let s = sv.reconciled();
        wait_closed(&v, "n0.md", &lin);
        settle(&v, Duration::from_millis(600));
        assert_closed(&v, "n0.md", &lin, policy);
        assert!(sv.blame("n0.md").is_empty());
        let hits = sv.hits("zqr0");
        if mv {
            assert_eq!(hits, vec!["sub/moved.md".to_string()]);
            assert_eq!(sv.blame("sub/moved.md").len(), 3);
        } else {
            assert!(hits.is_empty(), "{hits:?}");
        }
        sv.stop();
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({"summary": format!("{s:?}")})
    });
}

#[test]
fn a1_sigkill_delete_canonical() {
    sigkill_case("a1_canonical", "canonical", false);
}

#[test]
fn a1_sigkill_delete_unverified() {
    sigkill_case("a1_unverified", "unverified", false);
}

#[test]
fn a2_sigkill_move_canonical() {
    sigkill_case("a2_canonical", "canonical", true);
}

#[test]
fn a2_sigkill_move_unverified() {
    sigkill_case("a2_unverified", "unverified", true);
}

/// B1: SIGTERM ~0.5 s after a deletion takes the removal at the stop; the
/// restart writes nothing.
#[test]
fn b1_sigterm_takes_the_removal() {
    let b = Base::new("b1", "canonical");
    repeat("b1", |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n0.md");
        let sv = serve(&v);
        std::fs::remove_file(v.join("n0.md")).unwrap();
        std::thread::sleep(Duration::from_millis(500));
        sv.stop();
        assert_closed(&v, "n0.md", &lin, "canonical");
        let before = log_bytes(&v);
        let sv = serve(&v);
        std::thread::sleep(HOLD * 2);
        let after = log_bytes(&v);
        assert_eq!(
            after.get(before.len()..).unwrap_or(""),
            "",
            "the restart wrote events; its log: {}",
            sv.log()
        );
        assert!(sv.hits("zqr0").is_empty());
        let log = sv.stop();
        // the search writes its trace's checkpoint (spec §8), nothing else
        let after = log_bytes(&v);
        let rest = after.get(before.len()..).unwrap_or("");
        assert!(
            rest.lines()
                .all(|l| l.contains("\"kind\":\"CheckpointCreated\"")),
            "the stop wrote events: {rest}; its log: {log}"
        );
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

/// C1: the real client's sequence — SIGINT, SIGTERM, ~0.5 s, SIGKILL.
fn client_sequence(case: &str, policy: &str) {
    let b = Base::new(case, policy);
    repeat(case, |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n0.md");
        let mut sv = serve(&v);
        std::fs::remove_file(v.join("n0.md")).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        sv.signal("INT");
        sv.signal("TERM");
        std::thread::sleep(Duration::from_millis(500));
        let killed = sv.alive();
        if killed {
            sv.signal("KILL");
        }
        let (_, _, _) = sv.wait();
        let at_stop = !deleted(&events(&v), "n0.md").is_empty();
        let sv = serve(&v);
        wait_closed(&v, "n0.md", &lin);
        settle(&v, Duration::from_millis(600));
        assert_closed(&v, "n0.md", &lin, policy);
        assert!(sv.blame("n0.md").is_empty());
        assert!(sv.hits("zqr0").is_empty());
        sv.stop();
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({"sigkill_sent": killed, "taken_at_stop": at_stop})
    });
}

#[test]
fn c1_client_sequence_canonical() {
    client_sequence("c1_canonical", "canonical");
}

#[test]
fn c1_client_sequence_unverified() {
    client_sequence("c1_unverified", "unverified");
}

fn edited(i: usize) -> String {
    format!(
        "# Note {i}\n\nThis synthetic note now carries the word zqedit{i} instead.\n\nA second paragraph about topic {} and item {i}.\n",
        i % 3
    )
}

/// The offline changes of D4 (and F1, F2, G1): edit n1, delete n2, move n3
/// to sub/n3.md, add new.md.
fn offline_all(v: &Path) {
    write(&v.join("n1.md"), &edited(1));
    std::fs::remove_file(v.join("n2.md")).unwrap();
    std::fs::rename(v.join("n3.md"), v.join("sub/n3.md")).unwrap();
    write(&v.join("new.md"), &note(7));
}

/// Serves, stops with SIGTERM (nothing changed), applies `change` with the
/// server down, waits `GAP`, serves again and waits until the
/// reconciliation and its held removals are done; the restarted server.
fn down_change(v: &Path, change: impl FnOnce(&Path)) -> Served {
    let sv = serve(v);
    sv.stop();
    change(v);
    std::thread::sleep(GAP);
    let sv = serve(v);
    std::thread::sleep(HOLD);
    settle(v, Duration::from_millis(800));
    sv
}

#[test]
fn d1_edit_while_down() {
    let b = Base::new("d1", "canonical");
    repeat("d1", |i| {
        let v = b.vault(i);
        let sv = down_change(&v, |v| write(&v.join("n1.md"), &edited(1)));
        let s = sv.reconciled();
        assert_eq!(sv.hits("zqedit1"), vec!["n1.md".to_string()]);
        assert!(sv.hits("zqr1").is_empty());
        assert_eq!(sv.blame("n1.md").len(), 3);
        sv.stop();
        if !legacy() {
            assert_eq!(s.unwrap().changed, 1, "{s:?}");
        }
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({"summary": format!("{s:?}")})
    });
}

#[test]
fn d2_delete_while_down() {
    let b = Base::new("d2", "canonical");
    repeat("d2", |i| {
        let v = b.vault(i);
        let lin = lineages(&events(&v), "n2.md");
        let sv = down_change(&v, |v| std::fs::remove_file(v.join("n2.md")).unwrap());
        wait_closed(&v, "n2.md", &lin);
        assert_closed(&v, "n2.md", &lin, "canonical");
        assert!(sv.hits("zqr2").is_empty());
        sv.stop();
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

#[test]
fn d3_add_while_down() {
    let b = Base::new("d3", "canonical");
    repeat("d3", |i| {
        let v = b.vault(i);
        let sv = down_change(&v, |v| write(&v.join("new.md"), &note(7)));
        wait_for(LIMIT, "new.md ingested", || sv.blame("new.md").len() == 3);
        assert_eq!(sv.hits("zqr7"), vec!["new.md".to_string()]);
        sv.stop();
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

/// D4/F1/F2: every kind of offline change together; the log matches the
/// disk, the changes come in byte order and then the removals in byte
/// order, with the layer and actor of `policy`.
fn all_down(case: &str, policy: &str) {
    let b = Base::new(case, policy);
    repeat(case, |i| {
        let v = b.vault(i);
        let before = events(&v).len();
        let l2 = lineages(&events(&v), "n2.md");
        let l3 = lineages(&events(&v), "n3.md");
        let sv = down_change(&v, offline_all);
        wait_closed(&v, "n2.md", &l2);
        wait_closed(&v, "n3.md", &l3);
        settle(&v, Duration::from_millis(600));
        assert_eq!(sv.hits("zqedit1"), vec!["n1.md".to_string()]);
        assert!(sv.hits("zqr2").is_empty());
        assert_eq!(sv.hits("zqr3"), vec!["sub/n3.md".to_string()]);
        assert_eq!(sv.hits("zqr7"), vec!["new.md".to_string()]);
        let layer = if policy == "unverified" {
            "unverified"
        } else {
            "canonical"
        };
        for d in ["new.md", "sub/n3.md"] {
            let bl = sv.blame(d);
            assert_eq!(bl.len(), 3, "{d}");
            assert!(bl.iter().all(|x| x["layer"] == layer), "{d}: {bl:?}");
        }
        // n1.md: the changed paragraph takes the policy's layer
        let bl = sv.blame("n1.md");
        assert!(bl.iter().any(|x| x["layer"] == layer), "{bl:?}");
        sv.stop();
        let evs = events(&v);
        let new = &evs[before..];
        // order: the changes in byte order, then the removals
        let mut order: Vec<String> = Vec::new();
        for e in new {
            if let Some(d) = e["doc_path"].as_str().filter(|d| !d.is_empty()) {
                if order.last().map(String::as_str) != Some(d) {
                    order.push(d.to_string());
                }
            }
        }
        assert_eq!(
            order,
            ["n1.md", "new.md", "sub/n3.md", "n2.md", "n3.md"],
            "event order"
        );
        let (kind, id, source) = actor_of(policy);
        for e in new {
            let k = e["kind"].as_str().unwrap();
            if matches!(
                k,
                "BlockUpdated" | "BlockDeleted" | "BlockMoved" | "BlockSplit" | "BlockMerged"
            ) {
                assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
                assert_eq!(e["meta"]["actor"]["id"], id, "{e}");
                assert_eq!(e["meta"]["source"], source, "{e}");
            }
            if matches!(
                k,
                "BlockCreated" | "BlockUpdated" | "BlockSplit" | "BlockMerged"
            ) {
                if policy == "unverified" {
                    assert_eq!(e["meta"]["layer"], "unverified", "{e}");
                } else {
                    assert!(e["meta"].get("layer").is_none(), "{e}");
                }
            }
        }
        assert_closed(&v, "n2.md", &l2, policy);
        assert_closed(&v, "n3.md", &l3, policy);
        assert_log_matches_disk(&v);
        let hash = blake3_of(log_bytes(&v).as_bytes());
        std::fs::remove_dir_all(&v).unwrap();
        json!({"events_after_base": evs.len() - before, "log_b3": hash})
    });
}

fn blake3_of(b: &[u8]) -> String {
    blake3::hash(b).to_hex()[..32].to_string()
}

#[test]
fn d4_all_while_down() {
    all_down("d4", "canonical");
}

#[test]
fn f1_policy_unverified() {
    all_down("f1_unverified", "unverified");
}

#[test]
fn f2_policy_canonical() {
    all_down("f2_canonical", "canonical");
}

/// E1/E2: a restart with nothing changed writes no event, twice.
#[test]
fn e1_unchanged_restart_is_silent() {
    let b = Base::new("e1", "canonical");
    repeat("e1", |i| {
        let v = b.vault(i);
        let sv = serve(&v);
        sv.stop();
        let before = log_bytes(&v);
        for _ in 0..2 {
            let sv = serve(&v);
            let s = sv.reconciled();
            std::thread::sleep(HOLD * 2);
            assert_eq!(log_bytes(&v), before, "an unchanged restart wrote events");
            sv.stop();
            assert_eq!(log_bytes(&v), before);
            if !legacy() {
                let s = s.unwrap();
                assert_eq!(
                    (s.files, s.changed, s.held, s.errors),
                    (6, 0, 0, 0),
                    "{s:?}"
                );
            }
        }
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}

/// G1: the same offline changes on two fresh vaults give byte-equal logs.
#[test]
fn g1_reconciliation_is_deterministic() {
    let b = Base::new("g1", "canonical");
    repeat("g1", |i| {
        let mut logs = Vec::new();
        for k in 0..2 {
            let v = b.vault(i * 2 + k);
            let sv = down_change(&v, offline_all);
            let l3 = lineages(&events(&b.base), "n3.md");
            wait_closed(&v, "n3.md", &l3);
            settle(&v, Duration::from_millis(600));
            sv.stop();
            logs.push(log_bytes(&v));
            std::fs::remove_dir_all(&v).unwrap();
        }
        assert_eq!(logs[0], logs[1], "two runs, two logs");
        json!({"log_b3": blake3_of(logs[0].as_bytes())})
    });
}

/// G2 (D0F-6): an edit made right before the start (no gap) is taken once,
/// whichever of the reconciliation and a late file event sees it.
#[test]
fn g2_edit_right_before_start_taken_once() {
    let b = Base::new("g2", "canonical");
    repeat("g2", |i| {
        let v = b.vault(i);
        let sv = serve(&v);
        sv.stop();
        let n1 = of_kind(&events(&v), "SnapshotCommitted", "n1.md").len();
        write(&v.join("n1.md"), &edited(1));
        let sv = serve(&v);
        std::thread::sleep(HOLD);
        settle(&v, Duration::from_millis(800));
        assert_eq!(sv.hits("zqedit1"), vec!["n1.md".to_string()]);
        sv.stop();
        let evs = events(&v);
        assert_eq!(
            of_kind(&evs, "SnapshotCommitted", "n1.md").len(),
            n1 + 1,
            "the edit ingested more than once"
        );
        assert_log_matches_disk(&v);
        std::fs::remove_dir_all(&v).unwrap();
        json!({})
    });
}
