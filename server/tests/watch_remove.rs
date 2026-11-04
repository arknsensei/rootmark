//! P2-Q.1/R2 (PQ-18; `docs/rounds/P2-Q.1/progress/TESTS.md` D1–D10):
//! `rootmark serve --watch` ingests a removed Markdown file as the document
//! without blocks (spec §5 step 7: `BlockDeleted`, with the actor and
//! source of §7/§7.1), after the attestation window (DESIGN §2.4): deletion,
//! unattested moves, attested renames (no double processing), delete and
//! recreate, directories removed or moved, the service's stop, determinism
//! and §7.1's (2.0.3) rename back (DESIGN §1.1). Runs on macOS and Linux
//! (`make test`, CI). Needs the model files (`make models`).

mod watchkit;

use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use watchkit::*;

const LIMIT: Duration = Duration::from_secs(60);
const SETTLE: Duration = Duration::from_millis(800);

fn note(word: &str) -> String {
    format!("# {word}\n\nFirst paragraph about {word} beans.\n\nSecond paragraph about {word} grinders.\n")
}

/// Writes `doc` and waits until it is ingested; its live lineages.
fn put(sv: &Served, doc: &str, text: &str) -> Vec<String> {
    let p = sv.vault.join(doc);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    write(&p, text);
    wait_for(LIMIT, &format!("{doc} ingested"), || {
        sv.blame(doc).len() == text.split("\n\n").count()
    });
    lineages(sv, doc)
}

fn lineages(sv: &Served, doc: &str) -> Vec<String> {
    sv.blame(doc)
        .iter()
        .map(|b| b["lineage_id"].as_str().unwrap().to_string())
        .collect()
}

/// The `BlockDeleted` events of `doc`, by lineage.
fn deleted(evs: &[Value], doc: &str) -> Vec<String> {
    of_kind(evs, "BlockDeleted", doc)
        .iter()
        .flat_map(|e| e["lineage_ids"].as_array().unwrap().iter())
        .map(|l| l.as_str().unwrap().to_string())
        .collect()
}

fn wait_deleted(sv: &Served, doc: &str, lin: &[String]) {
    wait_for(LIMIT, &format!("{doc}'s removal ingested"), || {
        let d = deleted(&sv.events(), doc);
        lin.iter().all(|l| d.contains(l))
    });
}

/// Each lineage deleted exactly once, with the actor `kind`/`source`.
fn assert_deleted_once(sv: &Served, doc: &str, lin: &[String], kind: &str, source: &str) {
    let evs = sv.events();
    let mut d = deleted(&evs, doc);
    d.sort();
    let mut want = lin.to_vec();
    want.sort();
    assert_eq!(d, want, "{doc}: BlockDeleted once per lineage");
    for e in of_kind(&evs, "BlockDeleted", doc) {
        assert_eq!(e["meta"]["actor"]["kind"], kind, "{e}");
        assert_eq!(e["meta"]["source"], source, "{e}");
        assert!(e["meta"].get("layer").is_none(), "{e}");
    }
    assert!(sv.blame(doc).is_empty(), "{doc}: blame not empty");
}

fn vault(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = tmp(name);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    (root, v)
}

fn done(sv: Served, root: &Path) {
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}

/// D1: a deleted note (policy `canonical`).
#[test]
fn d1_deleted_note_is_closed() {
    let (root, v) = vault("d1");
    let sv = serve(&v, "canonical", 1.0);
    let lin = put(&sv, "a.md", &note("okapione"));
    assert_eq!(sv.hits("okapione"), vec!["a.md".to_string()]);
    std::fs::remove_file(v.join("a.md")).unwrap();
    wait_deleted(&sv, "a.md", &lin);
    assert_deleted_once(&sv, "a.md", &lin, "human", "rootmark-serve-watch");
    let evs = sv.events();
    assert_eq!(evs.last().unwrap()["kind"], "SnapshotCommitted");
    assert!(
        sv.hits("okapione").is_empty(),
        "a deleted note is still found"
    );
    done(sv, &root);
}

/// D2: an unattested deletion under `unverified` carries `unattested_fs`.
#[test]
fn d2_unattested_deletion_under_unverified() {
    let (root, v) = vault("d2");
    let sv = serve(&v, "unverified", 2.0);
    write(&v.join("h.md"), &note("okapitwo"));
    let (ok, out) = sv.attest("h.md", None);
    assert!(ok, "{out}");
    wait_for(LIMIT, "h.md canonical", || {
        let b = sv.blame("h.md");
        b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical")
    });
    let lin = lineages(&sv, "h.md");
    std::fs::remove_file(v.join("h.md")).unwrap();
    wait_deleted(&sv, "h.md", &lin);
    assert_deleted_once(&sv, "h.md", &lin, "unattested_fs", "unattested_fs");
    for e in of_kind(&sv.events(), "BlockDeleted", "h.md") {
        assert_eq!(e["meta"]["actor"]["id"], "fs");
    }
    assert!(sv.hits("okapitwo").is_empty());
    done(sv, &root);
}

/// D3: an unattested move inside the vault: the old path closed, the new
/// one ingested.
#[test]
fn d3_unattested_move() {
    let (root, v) = vault("d3");
    let sv = serve(&v, "canonical", 1.0);
    let lin = put(&sv, "a.md", &note("okapithree"));
    std::fs::create_dir_all(v.join("sub")).unwrap();
    std::fs::rename(v.join("a.md"), v.join("sub/b.md")).unwrap();
    wait_deleted(&sv, "a.md", &lin);
    assert_deleted_once(&sv, "a.md", &lin, "human", "rootmark-serve-watch");
    let b = sv.blame("sub/b.md");
    assert_eq!(b.len(), 3);
    assert!(b.iter().all(|b| b["layer"] == "canonical"));
    assert_eq!(sv.hits("okapithree"), vec!["sub/b.md".to_string()]);
    done(sv, &root);
}

/// D4a/D4b: attested renames under `unverified`, the attestation right
/// after the move (a) and after the watcher took the new path (b, late):
/// accepted, the new path `canonical` and ingested once, the old path
/// deleted once, after the attestation.
#[test]
fn d4_attested_rename_is_not_processed_twice() {
    let (root, v) = vault("d4");
    let sv = serve(&v, "unverified", 2.0);
    for (doc, word) in [("r.md", "okapifour"), ("s.md", "okapifive")] {
        write(&v.join(doc), &note(word));
        let (ok, out) = sv.attest(doc, None);
        assert!(ok, "{out}");
    }
    wait_for(LIMIT, "r.md and s.md canonical", || {
        ["r.md", "s.md"].iter().all(|d| {
            let b = sv.blame(d);
            b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical")
        })
    });
    let (lr, ls) = (lineages(&sv, "r.md"), lineages(&sv, "s.md"));
    // (a) on time
    std::fs::rename(v.join("r.md"), v.join("r2.md")).unwrap();
    let (ok, out) = sv.attest("r2.md", Some("r.md"));
    assert!(ok, "on-time rename refused: {out}");
    // (b) late: the watcher has taken the new path unattested
    std::fs::rename(v.join("s.md"), v.join("s2.md")).unwrap();
    wait_for(LIMIT, "s2.md ingested", || sv.blame("s2.md").len() == 3);
    let (ok, out) = sv.attest("s2.md", Some("s.md"));
    assert!(ok, "late rename refused: {out}");
    wait_deleted(&sv, "r.md", &lr);
    wait_deleted(&sv, "s.md", &ls);
    settle(&v, SETTLE, LIMIT);
    let evs = sv.events();
    for (old, new, lin) in [("r.md", "r2.md", &lr), ("s.md", "s2.md", &ls)] {
        assert_deleted_once(&sv, old, lin, "unattested_fs", "unattested_fs");
        let b = sv.blame(new);
        assert!(
            b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical"),
            "{new}: {b:?}"
        );
        assert_eq!(
            filled_snapshots(&evs, new),
            1,
            "{new} ingested more than once"
        );
        // the old path's removal comes after the new path's ingest
        let first_del = evs
            .iter()
            .position(|e| e["kind"] == "BlockDeleted" && e["doc_path"] == old)
            .unwrap();
        let new_ingest = evs.iter().position(|e| e["doc_path"] == new).unwrap();
        assert!(new_ingest < first_del);
    }
    // the late one was raised by the late attestation
    assert!(evs.iter().any(|e| e["kind"] == "LayerChanged"
        && e["doc_path"] == "s2.md"
        && e["meta"]["cause"] == "late_attestation"));
    let pos_raise = evs
        .iter()
        .position(|e| e["kind"] == "LayerChanged" && e["doc_path"] == "s2.md")
        .unwrap();
    let pos_del = evs
        .iter()
        .position(|e| e["kind"] == "BlockDeleted" && e["doc_path"] == "s.md")
        .unwrap();
    assert!(
        pos_raise < pos_del,
        "s.md removed before the late attestation"
    );
    done(sv, &root);
}

/// D5a/D5b: deleted and recreated within the hold (a change: nothing), and
/// after the removal was ingested (deleted, then created).
#[test]
fn d5_delete_and_recreate() {
    let (root, v) = vault("d5");
    let sv = serve(&v, "canonical", 1.0);
    let text = note("okapisix");
    let lin = put(&sv, "t.md", &text);
    let n0 = sv.events().len();
    std::fs::remove_file(v.join("t.md")).unwrap();
    std::thread::sleep(Duration::from_millis(150));
    write(&v.join("t.md"), &text);
    std::thread::sleep(Duration::from_millis(2500)); // past the hold (1.5 s)
    settle(&v, SETTLE, LIMIT);
    let evs = sv.events();
    assert!(
        deleted(&evs, "t.md").is_empty(),
        "D5a: deleted within the hold"
    );
    assert_eq!(evs.len(), n0, "D5a: same content, no event");
    // D5b
    std::fs::remove_file(v.join("t.md")).unwrap();
    wait_deleted(&sv, "t.md", &lin);
    write(&v.join("t.md"), &text);
    wait_for(LIMIT, "t.md recreated", || sv.blame("t.md").len() == 3);
    let evs = sv.events();
    let last_del = evs
        .iter()
        .rposition(|e| e["kind"] == "BlockDeleted" && e["doc_path"] == "t.md")
        .unwrap();
    let created_after = evs[last_del..]
        .iter()
        .filter(|e| e["kind"] == "BlockCreated" && e["doc_path"] == "t.md")
        .count();
    assert_eq!(created_after, 3);
    let mut d = deleted(&evs, "t.md");
    d.sort();
    let mut want = lin.clone();
    want.sort();
    assert_eq!(d, want, "D5b: each lineage deleted once");
    done(sv, &root);
}

/// D6: a directory removed; D7a: moved out of the vault; D7b: moved inside.
#[test]
fn d6_d7_directories_removed_and_moved() {
    let (root, v) = vault("d6");
    let sv = serve(&v, "canonical", 1.0);
    let dd = [
        put(&sv, "dd/a.md", &note("okapiseven")),
        put(&sv, "dd/b.md", &note("okapieight")),
        put(&sv, "dd/in/c.md", &note("okapinine")),
    ];
    let mv = [
        put(&sv, "mv/a.md", &note("okapiten")),
        put(&sv, "mv/x/b.md", &note("okapieleven")),
    ];
    let mw = [
        put(&sv, "mw/a.md", &note("okapitwelve")),
        put(&sv, "mw/b.md", &note("okapithirteen")),
    ];
    std::fs::remove_dir_all(v.join("dd")).unwrap();
    std::fs::rename(v.join("mv"), root.join("mv-outside")).unwrap();
    std::fs::rename(v.join("mw"), v.join("mw2")).unwrap();
    for (doc, lin) in ["dd/a.md", "dd/b.md", "dd/in/c.md"].iter().zip(&dd) {
        wait_deleted(&sv, doc, lin);
    }
    for (doc, lin) in ["mv/a.md", "mv/x/b.md"].iter().zip(&mv) {
        wait_deleted(&sv, doc, lin);
    }
    for (doc, lin) in ["mw/a.md", "mw/b.md"].iter().zip(&mw) {
        wait_deleted(&sv, doc, lin);
    }
    settle(&v, SETTLE, LIMIT);
    for (doc, lin) in [
        "dd/a.md",
        "dd/b.md",
        "dd/in/c.md",
        "mv/a.md",
        "mv/x/b.md",
        "mw/a.md",
        "mw/b.md",
    ]
    .iter()
    .zip(dd.iter().chain(&mv).chain(&mw))
    {
        assert_deleted_once(&sv, doc, lin, "human", "rootmark-serve-watch");
    }
    for w in ["okapiseven", "okapinine", "okapiten", "okapieleven"] {
        assert!(sv.hits(w).is_empty(), "{w} still found");
    }
    assert_eq!(sv.hits("okapitwelve"), vec!["mw2/a.md".to_string()]);
    assert_eq!(sv.blame("mw2/b.md").len(), 3);
    done(sv, &root);
}

/// D8: a removal held when the service stops is ingested before it exits.
#[test]
fn d8_held_removal_is_ingested_at_stop() {
    let (root, v) = vault("d8");
    let sv = serve(&v, "canonical", 2.0);
    let lin = put(&sv, "u.md", &note("okapifourteen"));
    std::fs::remove_file(v.join("u.md")).unwrap();
    std::thread::sleep(Duration::from_millis(400)); // within the hold (2.5 s)
    assert!(deleted(&sv.events(), "u.md").is_empty());
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    let mut d = deleted(&events(&v), "u.md");
    d.sort();
    let mut want = lin.clone();
    want.sort();
    assert_eq!(d, want, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}

/// D9: the same steps in two fresh vaults give the same event log, byte
/// for byte.
#[test]
fn d9_same_steps_same_log() {
    let mut logs = Vec::new();
    for k in 0..2 {
        let (root, v) = vault(&format!("d9-{k}"));
        let sv = serve(&v, "canonical", 1.0);
        let step = |f: &dyn Fn()| {
            f();
            settle(&v, Duration::from_millis(2500), LIMIT); // past the hold
        };
        step(&|| write(&v.join("a.md"), &note("pumaone")));
        step(&|| {
            std::fs::create_dir_all(v.join("dd/in")).unwrap();
            write(&v.join("dd/b.md"), &note("pumatwo"));
            write(&v.join("dd/in/c.md"), &note("pumathree"));
        });
        step(&|| std::fs::remove_file(v.join("a.md")).unwrap());
        step(&|| write(&v.join("e.md"), &note("pumafour")));
        step(&|| {
            std::fs::create_dir_all(v.join("sub")).unwrap();
            std::fs::rename(v.join("e.md"), v.join("sub/e.md")).unwrap()
        });
        step(&|| std::fs::remove_dir_all(v.join("dd")).unwrap());
        let (ok, log) = sv.stop();
        assert!(ok, "{log}");
        logs.push(std::fs::read(v.join(".rootmark/events.jsonl")).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
    let evs: Vec<Value> = String::from_utf8(logs[0].clone())
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(evs.iter().filter(|e| e["kind"] == "BlockDeleted").count() >= 9);
    assert!(logs[0] == logs[1], "the two logs differ");
}

/// D10: spec §7.1 (2.0.3) rename back with the removals ingested (DESIGN
/// §1.1): a → b attested, a's removal ingested, b → a attested; each with
/// the note's own link rewritten.
#[test]
fn d10_rename_back_after_the_removal() {
    let (root, v) = vault("d10");
    let sv = serve(&v, "unverified", 2.0);
    let text = |me: &str| {
        format!("# Kiln\n\nA note about the zorilla kiln, see [[{me}]].\n\nThe second zorilla paragraph.\n")
    };
    write(&v.join("a.md"), &text("a"));
    let (ok, out) = sv.attest("a.md", None);
    assert!(ok, "{out}");
    wait_for(LIMIT, "a.md canonical", || {
        let b = sv.blame("a.md");
        b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical")
    });
    let la = lineages(&sv, "a.md");
    // a → b, own link rewritten
    std::fs::rename(v.join("a.md"), v.join("b.md")).unwrap();
    write(&v.join("b.md"), &text("b"));
    let (ok, out) = sv.attest("b.md", Some("a.md"));
    assert!(ok, "a → b refused: {out}");
    wait_deleted(&sv, "a.md", &la);
    wait_for(LIMIT, "b.md canonical", || {
        let b = sv.blame("b.md");
        b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical")
    });
    let lb = lineages(&sv, "b.md");
    // b → a, the link back
    std::fs::rename(v.join("b.md"), v.join("a.md")).unwrap();
    write(&v.join("a.md"), &text("a"));
    let (ok, out) = sv.attest("a.md", Some("b.md"));
    assert!(ok, "b → a refused: {out}");
    wait_deleted(&sv, "b.md", &lb);
    wait_for(LIMIT, "a.md canonical again", || {
        let b = sv.blame("a.md");
        b.len() == 3 && b.iter().all(|b| b["layer"] == "canonical")
    });
    settle(&v, SETTLE, LIMIT);
    assert_deleted_once(&sv, "b.md", &lb, "unattested_fs", "unattested_fs");
    let mut d = deleted(&sv.events(), "a.md");
    d.sort();
    let mut want = la.clone();
    want.sort();
    assert_eq!(
        d, want,
        "a.md's first lineages deleted once (its first removal)"
    );
    done(sv, &root);
}
