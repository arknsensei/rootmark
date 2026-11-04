//! P2-Q.1/R1 (PQ-22; `docs/rounds/P2-Q.1/progress/TESTS.md` N1–N6):
//! `rootmark serve --watch` takes the Markdown files of a directory created
//! in (or moved into) the vault, also those written before the watcher
//! could watch the new directory (Linux: inotify adds the watch only after
//! the directory's creation is read). Each repetition creates a directory
//! and writes into it at once. Runs on macOS and Linux (`make test`, CI).
//! Needs the model files (`make models`).

mod watchkit;

use std::time::Duration;
use watchkit::*;

const LIMIT: Duration = Duration::from_secs(60);

fn note(word: &str) -> String {
    format!("# {word}\n\nA note about {word} written into a new folder.\n")
}

/// N1–N4: new directory + an immediate write (20), nested directories
/// (20), many files (20 × 10); every document ingested exactly once.
#[test]
fn n1_to_n4_files_of_new_directories_are_ingested_once() {
    let root = tmp("newdir");
    let vault = root.join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    let sv = serve(&vault, "canonical", 2.0);
    let mut docs = Vec::new();
    for i in 0..20 {
        // N1
        let d = vault.join(format!("n1/d{i:02}"));
        std::fs::create_dir_all(&d).unwrap();
        write(&d.join("a.md"), &note(&format!("ninaone{i}")));
        docs.push(format!("n1/d{i:02}/a.md"));
    }
    for i in 0..20 {
        // N2
        let d = vault.join(format!("n2/e{i:02}/x/y"));
        std::fs::create_dir_all(&d).unwrap();
        write(&d.join("deep.md"), &note(&format!("ninatwo{i}")));
        docs.push(format!("n2/e{i:02}/x/y/deep.md"));
    }
    for i in 0..20 {
        // N3
        let d = vault.join(format!("n3/f{i:02}"));
        std::fs::create_dir_all(&d).unwrap();
        for k in 0..10 {
            write(
                &d.join(format!("k{k}.md")),
                &note(&format!("ninathree{i}x{k}")),
            );
            docs.push(format!("n3/f{i:02}/k{k}.md"));
        }
    }
    assert_eq!(docs.len(), 240);
    wait_for(LIMIT, "every new document ingested", || {
        let evs = events(&vault);
        docs.iter().all(|d| filled_snapshots(&evs, d) >= 1)
    });
    settle(&vault, Duration::from_millis(1500), LIMIT);
    // N4: once each, nothing deleted
    let evs = sv.events();
    let twice: Vec<_> = docs
        .iter()
        .filter(|d| of_kind(&evs, "SnapshotCommitted", d).len() != 1)
        .collect();
    assert!(
        twice.is_empty(),
        "documents not committed exactly once: {twice:?}"
    );
    assert_eq!(
        evs.iter().filter(|e| e["kind"] == "BlockDeleted").count(),
        0
    );
    // searchable (hot tier) and canonical
    for (w, d) in [
        ("ninaone7", "n1/d07/a.md"),
        ("ninathree19x9", "n3/f19/k9.md"),
    ] {
        assert_eq!(sv.hits(w), vec![d.to_string()]);
    }
    assert!(sv
        .blame("n2/e13/x/y/deep.md")
        .iter()
        .all(|b| b["layer"] == "canonical"));
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}

/// N5: hidden directories stay outside (PQ-13's rule, unchanged).
#[test]
fn n5_hidden_directories_are_not_ingested() {
    let root = tmp("hidden");
    let vault = root.join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    let sv = serve(&vault, "canonical", 2.0);
    std::fs::create_dir_all(vault.join(".gizli")).unwrap();
    write(&vault.join(".gizli/a.md"), &note("hiddenone"));
    std::fs::create_dir_all(vault.join("n5/.obs/z")).unwrap();
    write(&vault.join("n5/.obs/z/b.md"), &note("hiddentwo"));
    write(&vault.join("n5/marker.md"), &note("markerword"));
    wait_for(LIMIT, "the marker ingested", || {
        filled_snapshots(&events(&vault), "n5/marker.md") == 1
    });
    settle(&vault, Duration::from_millis(1000), LIMIT);
    let evs = sv.events();
    assert!(
        evs.iter().all(|e| {
            let d = e["doc_path"].as_str().unwrap_or("");
            !d.contains(".gizli") && !d.contains(".obs")
        }),
        "a hidden path was ingested"
    );
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}

/// N6: a full directory moved into the vault from outside.
#[test]
fn n6_directory_moved_into_the_vault_is_ingested() {
    let root = tmp("movein");
    let vault = root.join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    let outside = root.join("outside/pack");
    std::fs::create_dir_all(outside.join("inner")).unwrap();
    write(&outside.join("p.md"), &note("movedone"));
    write(&outside.join("inner/q.md"), &note("movedtwo"));
    let sv = serve(&vault, "canonical", 2.0);
    std::fs::rename(&outside, vault.join("pack")).unwrap();
    wait_for(LIMIT, "both moved notes ingested", || {
        let evs = events(&vault);
        filled_snapshots(&evs, "pack/p.md") == 1 && filled_snapshots(&evs, "pack/inner/q.md") == 1
    });
    assert_eq!(sv.hits("movedtwo"), vec!["pack/inner/q.md".to_string()]);
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}
