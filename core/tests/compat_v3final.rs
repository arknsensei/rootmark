//! P1-B.1/B1.1 (GOAL B1-1, B1-2; K-B1.1): vaults written by the **v3-final**
//! core library with non-human and mixed authors open with the current core.
//!
//! `fixtures/v3_final_mixed/<scenario>/` was written by
//! `scripts/gen_v3final_mixed.sh` (v3-final public API, `Vault::ingest` with
//! `Provenance { author_kind, .. }`; scenarios in
//! `scripts/v3final-mixed-gen/scenarios.json`): `store/` is the vault's
//! `.mdkernel/` directory and `expected.json` what v3-final itself reports
//! (replay snapshot_ids, events_hash). Every scenario is opened from a copy:
//! replay gives v3-final's snapshot_ids, `events.jsonl` is unchanged, and
//! `blame` gives the layer the rule derives — a drop the log does not record
//! is `layer_source: derived`.
//!
//! B1-2: a log carrying layer information still needs every `LayerChanged`.

use mdkcore::{AuthorKind, Event, Layer, LayerSource, MatcherVersion, Provenance, Vault};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp(name: &str) -> PathBuf {
    // unique per call: tests run in parallel and may open the same scenario
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("mdk-b11-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &dst);
        } else {
            fs::copy(e.path(), &dst).unwrap();
        }
    }
}

use AuthorKind::{Agent as A, Human as H, Synthetic as S, Transcript as T};
use Layer::{Canonical as C, Proposal as P};
use LayerSource::{Derived as D, Event as E};

type Row = (AuthorKind, Layer, LayerSource);

/// notes.md of the two-step scenarios: [heading, p1 (updated in step 2), p2,
/// p3, p4 (created in step 2)].
fn pair(
    a1: AuthorKind,
    a2: AuthorKind,
    l1: Layer,
    l2_p1: (Layer, LayerSource),
    l2: Layer,
) -> Vec<Row> {
    vec![
        (a1, l1, E),
        (a2, l2_p1.0, l2_p1.1),
        (a1, l1, E),
        (a1, l1, E),
        (a2, l2, E),
    ]
}

/// Opens the scenario from a copy and checks B1-1; returns the vault root.
fn check(name: &str, want: &[(&str, Vec<Row>)]) -> PathBuf {
    let fx = here(&format!("tests/fixtures/v3_final_mixed/{name}"));
    let exp: Value =
        serde_json::from_str(&fs::read_to_string(fx.join("expected.json")).unwrap()).unwrap();
    let root = tmp(name);
    copy_tree(&fx.join("store"), &root.join(".mdkernel"));
    let log = root.join(".mdkernel/events.jsonl");
    let before = fs::read(&log).unwrap();
    assert!(!String::from_utf8_lossy(&before).contains("\"layer\""));

    let v = Vault::open(&root).unwrap_or_else(|e| panic!("{name}: open: {e:#}"));
    let ids = exp["replay_snapshot_ids"].as_object().unwrap();
    assert_eq!(v.doc_paths().len(), ids.len(), "{name}: documents");
    for (doc, sid) in ids {
        let s = v.replay(doc).unwrap().unwrap();
        assert_eq!(
            &s.snapshot_id,
            sid.as_str().unwrap(),
            "{name}: {doc} replay"
        );
        assert_eq!(
            &v.snapshot(doc).unwrap().unwrap().snapshot_id,
            sid.as_str().unwrap()
        );
    }
    assert_eq!(
        v.events_hash().unwrap(),
        exp["events_hash"].as_str().unwrap(),
        "{name}"
    );
    assert_eq!(
        want.len(),
        ids.len(),
        "{name}: every document has expectations"
    );
    for (doc, rows) in want {
        let got: Vec<Row> = v
            .blame(doc, None)
            .unwrap()
            .into_iter()
            .map(|b| (b.author_kind, b.layer, b.layer_source))
            .collect();
        assert_eq!(&got, rows, "{name}: {doc} blame");
    }
    drop(v);
    // a reopened vault gives the same, and nothing was written
    let v = Vault::open(&root).unwrap();
    assert_eq!(
        v.events_hash().unwrap(),
        exp["events_hash"].as_str().unwrap()
    );
    assert_eq!(
        fs::read(&log).unwrap(),
        before,
        "{name}: events.jsonl changed"
    );
    root
}

fn done(root: PathBuf) {
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn agent_only() {
    done(check(
        "agent_only",
        &[("notes.md", pair(A, A, P, (P, E), P))],
    ));
}

#[test]
fn transcript_only() {
    done(check(
        "transcript_only",
        &[("notes.md", pair(T, T, P, (P, E), P))],
    ));
}

#[test]
fn synthetic_only() {
    done(check(
        "synthetic_only",
        &[("notes.md", pair(S, S, C, (C, E), C))],
    ));
}

#[test]
fn human_then_synthetic() {
    done(check(
        "human_synthetic",
        &[("notes.md", pair(H, S, C, (C, E), C))],
    ));
}

#[test]
fn agent_then_human_keeps_proposal() {
    // K17-T: a human edit never raises a layer
    done(check(
        "agent_human",
        &[("notes.md", pair(A, H, P, (P, E), C))],
    ));
}

#[test]
fn human_then_agent_derives_the_drop() {
    done(check(
        "human_agent",
        &[("notes.md", pair(H, A, C, (P, D), P))],
    ));
}

#[test]
fn human_then_transcript_derives_the_drop() {
    done(check(
        "human_transcript",
        &[("notes.md", pair(H, T, C, (P, D), P))],
    ));
}

#[test]
fn synthetic_then_agent_derives_the_drop() {
    done(check(
        "synthetic_agent",
        &[("notes.md", pair(S, A, C, (P, D), P))],
    ));
}

#[test]
fn mixed_four_document_vault() {
    let root = check(
        "mixed_vault",
        &[
            // last step: an agent splits a human paragraph (dominant piece
            // inherits and drops, the new piece is proposal by its event) and
            // updates another
            (
                "garden.md",
                vec![(H, C, E), (A, P, D), (A, P, E), (A, P, D)],
            ),
            (
                "release.md",
                vec![(A, P, E), (H, P, E), (A, P, E), (H, C, E)],
            ),
            ("summary.md", vec![(S, C, E), (A, P, D), (S, C, E)]),
            (
                "meeting.md",
                vec![(T, P, E), (T, P, E), (T, P, E), (T, P, E)],
            ),
        ],
    );
    let log = fs::read_to_string(root.join(".mdkernel/events.jsonl")).unwrap();
    assert!(
        log.contains("\"BlockSplit\""),
        "the scenario exercises a split"
    );
    done(root);
}

#[test]
fn current_core_continues_a_v3_final_vault() {
    // the derived prefix stays derived; the current core logs its own drops
    let root = check("human_agent", &[("notes.md", pair(H, A, C, (P, D), P))]);
    let mut v = Vault::open(&root).unwrap();
    let human = Provenance::default();
    let agent = Provenance {
        author_kind: A,
        source: "claude-code".into(),
    };
    v.ingest(
        "b.md",
        "# B\n\nA human paragraph about tea.\n",
        human,
        MatcherVersion::V3,
    )
    .unwrap();
    let r = v
        .ingest(
            "b.md",
            "# B\n\nA human paragraph about green tea.\n",
            agent,
            MatcherVersion::V3,
        )
        .unwrap();
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == mdkcore::EventKind::LayerChanged));
    let v = Vault::open(&root).unwrap();
    let rows = |d: &str| -> Vec<Row> {
        v.blame(d, None)
            .unwrap()
            .into_iter()
            .map(|b| (b.author_kind, b.layer, b.layer_source))
            .collect()
    };
    assert_eq!(rows("notes.md"), pair(H, A, C, (P, D), P));
    assert_eq!(rows("b.md"), vec![(H, C, E), (A, P, E)]);
    done(root);
}

// ---- B1-2: logs carrying layer information

fn read_log(root: &Path) -> Vec<Event> {
    fs::read_to_string(root.join(".mdkernel/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// Rewrites the log without the `i`-th event, `seq` renumbered (no gap).
fn drop_event(root: &Path, i: usize) {
    let mut evs = read_log(root);
    evs.remove(i);
    let mut out = String::new();
    for (n, mut e) in evs.into_iter().enumerate() {
        e.seq = n as u64;
        out.push_str(&serde_json::to_string(&e).unwrap());
        out.push('\n');
    }
    fs::write(root.join(".mdkernel/events.jsonl"), out).unwrap();
}

fn human_doc_edited_by_agent(name: &str, checkpoint_first: bool) -> PathBuf {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    let agent = Provenance {
        author_kind: A,
        source: "claude-code".into(),
    };
    v.ingest(
        "d.md",
        "# D\n\nHuman text one.\n\nHuman text two.\n",
        Provenance::default(),
        MatcherVersion::V3,
    )
    .unwrap();
    if checkpoint_first {
        v.checkpoint(Some("before")).unwrap();
    }
    v.ingest(
        "d.md",
        "# D\n\nHuman text one, edited.\n\nHuman text two.\n",
        agent,
        MatcherVersion::V3,
    )
    .unwrap();
    root
}

fn layer_changed_at(root: &Path) -> usize {
    let evs = read_log(root);
    let ix: Vec<usize> = (0..evs.len())
        .filter(|&i| evs[i].kind == mdkcore::EventKind::LayerChanged)
        .collect();
    assert_eq!(ix.len(), 1);
    ix[0]
}

#[test]
fn deleted_layer_changed_line_is_an_error() {
    let root = human_doc_edited_by_agent("b12-line", false);
    let i = layer_changed_at(&root);
    let p = root.join(".mdkernel/events.jsonl");
    let text = fs::read_to_string(&p).unwrap();
    let kept: Vec<&str> = text
        .lines()
        .enumerate()
        .filter(|(n, _)| *n != i)
        .map(|(_, l)| l)
        .collect();
    fs::write(&p, kept.join("\n") + "\n").unwrap();
    let e = Vault::open(&root).err().expect("open must fail");
    assert!(format!("{e:#}").contains("seq"), "{e:#}");
    done(root);
}

#[test]
fn missing_layer_changed_in_a_layered_log_is_an_error() {
    // the checkpoint marks the log as carrying layer information
    let root = human_doc_edited_by_agent("b12-renumbered", true);
    let i = layer_changed_at(&root);
    drop_event(&root, i);
    let e = Vault::open(&root).err().expect("open must fail");
    assert!(
        format!("{e:#}").contains("snapshot committed before LayerChanged"),
        "{e:#}"
    );
    done(root);
}

#[test]
fn missing_second_layer_changed_is_an_error() {
    let root = human_doc_edited_by_agent("b12-second", false);
    let mut v = Vault::open(&root).unwrap();
    let agent = Provenance {
        author_kind: A,
        source: "claude-code".into(),
    };
    v.ingest(
        "d.md",
        "# D\n\nHuman text one, edited.\n\nHuman text two, edited.\n",
        agent,
        MatcherVersion::V3,
    )
    .unwrap();
    let evs = read_log(&root);
    let last = (0..evs.len())
        .rev()
        .find(|&i| evs[i].kind == mdkcore::EventKind::LayerChanged)
        .unwrap();
    drop_event(&root, last);
    let e = Vault::open(&root).err().expect("open must fail");
    assert!(
        format!("{e:#}").contains("snapshot committed before LayerChanged"),
        "{e:#}"
    );
    done(root);
}

#[test]
fn only_layer_event_removed_reads_as_v3_final_and_is_marked_derived() {
    // limit of K-B1.1 (ASSUMPTIONS B11-1): without its only layer-carrying
    // event the log is byte-for-byte a v3-final log; the layer is the same,
    // only its source says that no event recorded it
    let root = human_doc_edited_by_agent("b12-limit", false);
    let i = layer_changed_at(&root);
    drop_event(&root, i);
    let v = Vault::open(&root).unwrap();
    let b = v.blame("d.md", None).unwrap();
    assert_eq!(
        b.iter()
            .map(|b| (b.layer, b.layer_source))
            .collect::<Vec<_>>(),
        vec![(C, E), (P, D), (C, E)]
    );
    done(root);
}

#[test]
fn first_of_two_layer_changed_removed_reads_as_derived_prefix() {
    // limit of K-B1.1 (ASSUMPTIONS B11-2, B1.1 review): with the first of two
    // drops removed the log equals a v3-final log continued by the current
    // core; it opens, the layer is the same, the first drop reads as derived
    let root = human_doc_edited_by_agent("b12-first", false);
    let mut v = Vault::open(&root).unwrap();
    let agent = Provenance {
        author_kind: A,
        source: "claude-code".into(),
    };
    v.ingest(
        "d.md",
        "# D\n\nHuman text one, edited.\n\nHuman text two, edited.\n",
        agent,
        MatcherVersion::V3,
    )
    .unwrap();
    drop(v);
    let evs = read_log(&root);
    let first = (0..evs.len())
        .find(|&i| evs[i].kind == mdkcore::EventKind::LayerChanged)
        .unwrap();
    drop_event(&root, first);
    let v = Vault::open(&root).unwrap();
    let b = v.blame("d.md", None).unwrap();
    assert_eq!(
        b.iter()
            .map(|b| (b.layer, b.layer_source))
            .collect::<Vec<_>>(),
        vec![(C, E), (P, D), (P, E)]
    );
    done(root);
}

// ---- K-C0.1 (P1-C/C0; P1-B.1 verification Ş1, recommendation 1): a
// renumbered deletion of a `LayerChanged` opens as a "derived prefix" only
// when no other layer-carrying event remains up to the same snapshot;
// otherwise it is an error (ASSUMPTIONS P1-B.1 B11-2, dated addendum).

/// One document by `first`, then one agent ingest that edits two of its
/// blocks: two `LayerChanged` before the same `SnapshotCommitted`.
fn two_drops_in_one_snapshot(name: &str, first: AuthorKind) -> PathBuf {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    let by = |author_kind, source: &str| Provenance {
        author_kind,
        source: source.into(),
    };
    v.ingest(
        "d.md",
        "# D\n\nFirst paragraph of the document.\n\nSecond paragraph of the document.\n",
        by(first, "writer-1"),
        MatcherVersion::V3,
    )
    .unwrap();
    v.ingest(
        "d.md",
        "# D\n\nFirst paragraph of the document, edited.\n\nSecond paragraph of the document, edited.\n",
        by(A, "claude-code"),
        MatcherVersion::V3,
    )
    .unwrap();
    drop(v);
    let evs = read_log(&root);
    let lc: Vec<usize> = (0..evs.len())
        .filter(|&i| evs[i].kind == mdkcore::EventKind::LayerChanged)
        .collect();
    assert_eq!(lc.len(), 2, "{name}: two drops");
    let snap = (lc[1]..evs.len())
        .find(|&i| evs[i].kind == mdkcore::EventKind::SnapshotCommitted)
        .unwrap();
    assert!(
        (lc[0]..snap).all(|i| evs[i].kind != mdkcore::EventKind::SnapshotCommitted),
        "{name}: both drops belong to the same snapshot"
    );
    root
}

fn first_layer_changed(root: &Path) -> usize {
    let evs = read_log(root);
    (0..evs.len())
        .find(|&i| evs[i].kind == mdkcore::EventKind::LayerChanged)
        .unwrap()
}

#[test]
fn kc01_first_of_two_drops_in_one_snapshot_removed_is_an_error_human() {
    let root = two_drops_in_one_snapshot("kc01-human", H);
    drop_event(&root, first_layer_changed(&root));
    // the remaining drop no longer matches the replayed layer rule
    let e = Vault::open(&root).err().expect("open must fail");
    assert!(
        format!("{e:#}").contains("does not match the replayed layer rule"),
        "{e:#}"
    );
    done(root);
}

#[test]
fn kc01_first_of_two_drops_in_one_snapshot_removed_is_an_error_synthetic() {
    let root = two_drops_in_one_snapshot("kc01-synthetic", S);
    drop_event(&root, first_layer_changed(&root));
    // the remaining drop no longer matches the replayed layer rule
    let e = Vault::open(&root).err().expect("open must fail");
    assert!(
        format!("{e:#}").contains("does not match the replayed layer rule"),
        "{e:#}"
    );
    done(root);
}

#[test]
fn kc01_single_drop_removed_opens_as_derived_prefix() {
    // the accepted limit of K-C0.1: nothing else carries layer information
    // up to that snapshot, so the file equals a v3-final log; the layer is
    // the same, the source reads `derived`
    let root = human_doc_edited_by_agent("kc01-prefix", false);
    let before = Vault::open(&root)
        .unwrap()
        .blame("d.md", None)
        .unwrap()
        .into_iter()
        .map(|b| b.layer)
        .collect::<Vec<_>>();
    drop_event(&root, first_layer_changed(&root));
    let v = Vault::open(&root).unwrap();
    let b = v.blame("d.md", None).unwrap();
    assert_eq!(b.iter().map(|b| b.layer).collect::<Vec<_>>(), before);
    assert_eq!(
        b.iter()
            .map(|b| (b.layer, b.layer_source))
            .collect::<Vec<_>>(),
        vec![(C, E), (P, D), (C, E)]
    );
    done(root);
}
