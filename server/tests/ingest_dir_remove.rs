//! P2-Q.3/R2 (D0F-5; `docs/rounds/P2-Q.3/progress/TESTS.md` I1–I9, DESIGN
//! §4): `rootmark ingest-dir <vault> <dir>`, with the server down, removes
//! the documents under `dir` whose file is gone (spec §7.1's external
//! change: `BlockDeleted` with the policy's actor, source
//! `rootmark-ingest-dir`) and never touches a document outside `dir`; with
//! nothing changed it writes no event. Runs on macOS and Linux (`make
//! test`, CI).

mod restartkit;

use restartkit::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// V0 without an index (`ingest-dir` needs none) and the recipe of
/// `policy`; `None`: no recipe at all.
fn vault(name: &str, policy: Option<&str>) -> (PathBuf, PathBuf) {
    let root = tmp(name);
    let v = root.join("v");
    for i in 0..5 {
        write(&v.join(format!("n{i}.md")), &note(i));
    }
    write(
        &v.join("sub/k.md"),
        "# Keep\n\nA note that keeps the sub directory, word zqrkeep.\n",
    );
    if let Some(p) = policy {
        write(&v.join(".rootmark/policy.toml"), &recipe(p));
    }
    ingest(&v, &v);
    (root, v)
}

/// `rootmark ingest-dir <v> <dir>`: its JSON and stderr.
fn ingest(v: &Path, dir: &Path) -> (Value, String) {
    let (ok, out, err) = run(&["ingest-dir", v.to_str().unwrap(), dir.to_str().unwrap()]);
    assert!(ok, "ingest-dir failed: {err}");
    let j: Value = serde_json::from_str(&out).unwrap();
    let keys: Vec<&String> = j.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["events_hash", "ingested"], "the JSON's shape");
    (j, err)
}

fn lineages(v: &Path, doc: &str) -> Vec<String> {
    live(&events(v))
        .get(doc)
        .map(|s| s.iter().cloned().collect())
        .unwrap_or_default()
}

/// `doc`'s lineages each deleted once, with the actor and source.
fn assert_removed(v: &Path, doc: &str, lin: &[String], kind: &str, id: &str, source: &str) {
    let evs = events(v);
    let mut d = deleted(&evs, doc);
    d.sort();
    let mut want = lin.to_vec();
    want.sort();
    assert!(!want.is_empty());
    assert_eq!(d, want, "{doc}: BlockDeleted once per lineage");
    for e in of_kind(&evs, "BlockDeleted", doc) {
        assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
        assert_eq!(e["meta"]["actor"]["id"], id, "{e}");
        assert_eq!(e["meta"]["source"], source, "{e}");
    }
    assert!(head_texts(v, doc).is_empty(), "{doc}: blocks left");
}

/// The events of `doc` after `n` events of the log.
fn new_events_of(v: &Path, n: usize, doc: &str) -> usize {
    events(v)[n..]
        .iter()
        .filter(|e| e["doc_path"] == doc)
        .count()
}

/// I1, I2: a deleted note is removed (policy `canonical`).
#[test]
fn i1_i2_removed_canonical() {
    let (root, v) = vault("i1", Some("canonical"));
    let lin = lineages(&v, "n0.md");
    std::fs::remove_file(v.join("n0.md")).unwrap();
    let (j, err) = ingest(&v, &v);
    assert!(
        err.contains("rootmark ingest-dir: n0.md: no file under") && err.contains("removed ("),
        "{err}"
    );
    assert_removed(&v, "n0.md", &lin, "human", "tester", "rootmark-ingest-dir");
    let eh = run_ok(&["events-hash", v.to_str().unwrap()]);
    let eh: Value = serde_json::from_str(&eh).unwrap();
    assert_eq!(
        j["events_hash"], eh["events_hash"],
        "the JSON's events_hash is the log's"
    );
    assert_log_matches_disk(&v);
    std::fs::remove_dir_all(root).unwrap();
}

/// I3: policy `unverified`: the actor `unattested_fs`.
#[test]
fn i3_removed_unverified() {
    let (root, v) = vault("i3", Some("unverified"));
    let lin = lineages(&v, "n0.md");
    std::fs::remove_file(v.join("n0.md")).unwrap();
    ingest(&v, &v);
    assert_removed(&v, "n0.md", &lin, "unattested_fs", "fs", "unattested_fs");
    assert_log_matches_disk(&v);
    std::fs::remove_dir_all(root).unwrap();
}

/// Without a recipe the kernel reads the policy as `unverified` (K-P2L-T1).
#[test]
fn i3b_no_recipe_reads_unverified() {
    let (root, v) = vault("i3b", None);
    let lin = lineages(&v, "n0.md");
    std::fs::remove_file(v.join("n0.md")).unwrap();
    ingest(&v, &v);
    assert_removed(&v, "n0.md", &lin, "unattested_fs", "fs", "unattested_fs");
    std::fs::remove_dir_all(root).unwrap();
}

/// I4: a subdirectory root never touches a document outside it.
#[test]
fn i4_subdir_root_leaves_the_outside() {
    let (root, v) = vault("i4", Some("canonical"));
    std::fs::remove_file(v.join("n1.md")).unwrap();
    let n = events(&v).len();
    ingest(&v, &v.join("sub"));
    assert_eq!(
        new_events_of(&v, n, "n1.md"),
        0,
        "n1.md is outside the root"
    );
    assert_eq!(lineages(&v, "n1.md").len(), 3);
    assert_eq!(
        new_events_of(&v, n, "sub/k.md"),
        0,
        "sub/k.md's file is there"
    );
    assert_eq!(lineages(&v, "sub/k.md").len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}

/// I5: a subdirectory root removes a document under it.
#[test]
fn i5_subdir_root_removes_under_it() {
    let (root, v) = vault("i5", Some("canonical"));
    let lin = lineages(&v, "sub/k.md");
    std::fs::remove_file(v.join("sub/k.md")).unwrap();
    let n = events(&v).len();
    ingest(&v, &v.join("sub"));
    assert_removed(
        &v,
        "sub/k.md",
        &lin,
        "human",
        "tester",
        "rootmark-ingest-dir",
    );
    for i in 0..5 {
        assert_eq!(new_events_of(&v, n, &format!("n{i}.md")), 0);
    }
    std::fs::remove_dir_all(root).unwrap();
}

/// I6: nothing changed, no event.
#[test]
fn i6_unchanged_is_silent() {
    let (root, v) = vault("i6", Some("canonical"));
    let before = log_bytes(&v);
    let (_, err) = ingest(&v, &v);
    assert_eq!(log_bytes(&v), before, "ingest-dir wrote events");
    assert!(!err.contains("removed"), "{err}");
    std::fs::remove_dir_all(root).unwrap();
}

/// I7: a directory outside the vault removes nothing (today's behaviour).
#[test]
fn i7_outside_dir_removes_nothing() {
    let (root, v) = vault("i7", Some("canonical"));
    let other = root.join("corpus");
    write(&other.join("x.md"), &note(9));
    let n = events(&v).len();
    ingest(&v, &other);
    for i in 0..5 {
        assert_eq!(new_events_of(&v, n, &format!("n{i}.md")), 0);
    }
    assert!(events(&v)[n..].iter().all(|e| e["kind"] != "BlockDeleted"));
    std::fs::remove_dir_all(root).unwrap();
}

/// I8: hidden paths and a document reached through a symbolic link are
/// not removed.
#[cfg(unix)]
#[test]
fn i8_hidden_and_symlinks_are_not_removed() {
    let (root, v) = vault("i8", Some("canonical"));
    // a note ingested through a linked directory: its path holds a file
    let real = root.join("real");
    write(&real.join("l.md"), &note(8));
    std::os::unix::fs::symlink(&real, v.join("linked")).unwrap();
    ingest(&v, &v);
    assert_eq!(lineages(&v, "linked/l.md").len(), 3);
    // a hidden directory's note is not in the log, its removal is nothing
    write(&v.join(".hidden/x.md"), &note(6));
    ingest(&v, &v);
    let n = events(&v).len();
    std::fs::remove_dir_all(v.join(".hidden")).unwrap();
    ingest(&v, &v);
    assert_eq!(events(&v).len(), n, "no event");
    assert_eq!(
        lineages(&v, "linked/l.md").len(),
        3,
        "the linked note stays"
    );
    std::fs::remove_dir_all(root).unwrap();
}

/// I9: the same removal on two fresh vaults gives the same log.
#[test]
fn i9_deterministic() {
    let mut hashes = Vec::new();
    for k in 0..2 {
        let (root, v) = vault(&format!("i9{k}"), Some("canonical"));
        std::fs::remove_file(v.join("n0.md")).unwrap();
        std::fs::remove_file(v.join("sub/k.md")).unwrap();
        let (j, _) = ingest(&v, &v);
        hashes.push(j["events_hash"].clone());
        let evs = events(&v);
        let order: Vec<&str> = evs
            .iter()
            .filter(|e| e["kind"] == "BlockDeleted")
            .map(|e| e["doc_path"].as_str().unwrap())
            .collect();
        assert_eq!(order.first(), Some(&"n0.md"));
        assert_eq!(order.last(), Some(&"sub/k.md"), "byte order");
        std::fs::remove_dir_all(root).unwrap();
    }
    assert_eq!(hashes[0], hashes[1]);
}
