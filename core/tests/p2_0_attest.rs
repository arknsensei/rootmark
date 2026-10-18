//! P2-0/Z2 (spec 2.0.0 §7, §7.1, §8): the kernel's side of attested writes.
//! - an ingest declaring `unverified` for human content, with the
//!   `unattested_fs` actor, drops the changed canonical lineages and replays;
//! - `raise_late` writes a late attestation's `LayerChanged`, which replays
//!   only with its attestation in `attestations.jsonl`;
//! - `Promoted` moves `unverified` to `canonical` under `HumanOnly`;
//! - reading rejects: `meta.layer` on an event that gives no layer, an
//!   `unattested_fs` change without `meta.layer = unverified`, a raising
//!   `LayerChanged` without (or with a foreign) attestation, invalid
//!   attestation lines; the low-level `commit` refuses a raising
//!   `LayerChanged`.

use rootmarkcore::attest::{content_hash, Raise};
use rootmarkcore::{
    Actor, ActorKind, AuthorKind, HumanOnly, Layer, MatcherVersion, Provenance, TransitionKind,
    Vault,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

const M: MatcherVersion = MatcherVersion::V3;
const DOC: &str =
    "# Notes\n\nA human paragraph about roasting.\n\n## Water\n\nDescale the boiler monthly.\n";
const EDIT: &str = "# Notes\n\nAn agent rewrote this paragraph about roasting entirely.\n\n## Water\n\nDescale the boiler monthly.\n\nAn agent paragraph.\n";

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-p2z2core-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn human(source: &str) -> Provenance {
    Provenance {
        author_kind: AuthorKind::Human,
        source: source.to_string(),
    }
}

fn fs_actor() -> Actor {
    Actor::new(ActorKind::UnattestedFs, "fs").unwrap()
}

/// A vault with `DOC` (canonical) and an unattested ingest of `EDIT`.
fn unattested(name: &str) -> (PathBuf, Vault) {
    let d = dir(name);
    let mut v = Vault::open(&d).unwrap();
    v.ingest("notes.md", DOC, human(""), M).unwrap();
    let plan = v
        .plan_ingest(
            "notes.md",
            EDIT,
            human("unattested_fs"),
            M,
            Some(Layer::Unverified),
            Some(&fs_actor()),
        )
        .unwrap();
    v.commit_plan(plan).unwrap();
    (d, v)
}

fn log(d: &Path) -> Vec<Value> {
    std::fs::read_to_string(d.join(".rootmark/events.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn write_log(d: &Path, events: &[Value]) {
    let mut s = String::new();
    for e in events {
        s.push_str(&serde_json::to_string(e).unwrap());
        s.push('\n');
    }
    std::fs::write(d.join(".rootmark/events.jsonl"), s).unwrap();
}

fn open_err(d: &Path) -> String {
    format!(
        "{:#}",
        Vault::open(d).err().expect("the vault must not open")
    )
}

#[test]
fn unattested_ingest_drops_changed_lineages_and_replays() {
    let (d, v) = unattested("ingest");
    let blame = v.blame("notes.md", None).unwrap();
    let layers: Vec<Layer> = blame.iter().map(|b| b.layer).collect();
    assert!(layers.contains(&Layer::Canonical));
    assert!(layers.contains(&Layer::Unverified));
    let ev = log(&d);
    assert!(ev.iter().any(|e| e["kind"] == "LayerChanged"
        && e["meta"]["from"] == "canonical"
        && e["meta"]["to"] == "unverified"));
    // reopen: the log replays, the same layers
    let w = Vault::open(&d).unwrap();
    assert_eq!(w.blame("notes.md", None).unwrap(), blame);
    // an unverified lineage is promoted by a human
    let l = blame.iter().find(|b| b.layer == Layer::Unverified).unwrap();
    let mut w = w;
    let req = w
        .request(
            TransitionKind::Promote,
            &l.lineage_id,
            Layer::Canonical,
            Actor::new(ActorKind::Human, "ada").unwrap(),
            "mine",
        )
        .unwrap();
    let e = w.promote(&req, &HumanOnly).unwrap();
    assert_eq!(e.meta["from"], "unverified");
    assert_eq!(w.layer_of(&l.lineage_id).unwrap(), Layer::Canonical);
    Vault::open(&d).unwrap();
}

#[test]
fn late_attestation_replays_only_with_its_attestation() {
    let (d, mut v) = unattested("late");
    let unv: Vec<Raise> = v
        .blame("notes.md", None)
        .unwrap()
        .into_iter()
        .filter(|b| b.layer == Layer::Unverified)
        .map(|b| Raise {
            lineage_id: b.lineage_id,
            version_id: b.version_id,
            to: Layer::Canonical,
        })
        .collect();
    let (a, new) = v
        .record_attestation("notes.md", &content_hash(EDIT.as_bytes()), None)
        .unwrap();
    assert!(new);
    // the same attestation again is a no-op
    assert!(
        !v.record_attestation("notes.md", &content_hash(EDIT.as_bytes()), None)
            .unwrap()
            .1
    );
    let evs = v.raise_late("notes.md", &unv, &a.attestation_id).unwrap();
    assert_eq!(evs.len(), unv.len());
    assert!(v
        .blame("notes.md", None)
        .unwrap()
        .iter()
        .all(|b| b.layer == Layer::Canonical));
    Vault::open(&d).unwrap();
    // without the attestation log the late LayerChanged does not replay
    let att = d.join(".rootmark/attestations.jsonl");
    let saved = std::fs::read(&att).unwrap();
    std::fs::remove_file(&att).unwrap();
    assert!(open_err(&d).contains("not in attestations.jsonl"));
    std::fs::write(&att, &saved).unwrap();
    // an attestation of another document does not count
    let other =
        rootmarkcore::attest::Attestation::new("other.md", &content_hash(b"x"), a.at_seq, None)
            .unwrap();
    let mut ev = log(&d);
    for e in ev
        .iter_mut()
        .filter(|e| e["meta"]["cause"] == "late_attestation")
    {
        e["meta"]["attestation_id"] = Value::String(other.attestation_id.clone());
    }
    let good = log(&d);
    write_log(&d, &ev);
    std::fs::write(
        &att,
        format!(
            "{}{}\n",
            String::from_utf8(saved.clone()).unwrap(),
            serde_json::to_string(&other).unwrap()
        ),
    )
    .unwrap();
    assert!(open_err(&d).contains("is of other.md"));
    write_log(&d, &good);
    std::fs::write(&att, &saved).unwrap();
    // a raising LayerChanged with another cause, or to derived, is rejected
    let mut ev = log(&d);
    let i = ev
        .iter()
        .position(|e| e["meta"]["cause"] == "late_attestation")
        .unwrap();
    ev[i]["meta"]["cause"] = Value::String("ingest".into());
    write_log(&d, &ev);
    assert!(open_err(&d).contains("must be a late attestation"));
    write_log(&d, &good);
    Vault::open(&d).unwrap();
}

#[test]
fn reading_rejects_misplaced_layer_and_actor() {
    let (d, _) = unattested("reading");
    let good = log(&d);
    // meta.layer on an event that gives no layer
    let mut ev = good.clone();
    let i = ev
        .iter()
        .position(|e| e["kind"] == "SnapshotCommitted")
        .unwrap();
    ev[i]["meta"]["layer"] = Value::String("unverified".into());
    write_log(&d, &ev);
    assert!(open_err(&d).contains("carries meta.layer"));
    // an unattested_fs change without its declared layer
    let mut ev = good.clone();
    let i = ev
        .iter()
        .position(|e| e["kind"] == "BlockUpdated" && e["meta"]["actor"]["kind"] == "unattested_fs")
        .expect("an unattested update");
    ev[i]["meta"].as_object_mut().unwrap().remove("layer");
    write_log(&d, &ev);
    let e = open_err(&d);
    assert!(e.contains("must declare meta.layer unverified"), "{e}");
    // a declared layer other than unverified for human content (a created
    // block: it carries no actor)
    let mut ev = good.clone();
    let i = ev
        .iter()
        .position(|e| e["kind"] == "BlockCreated" && e["meta"]["layer"] == "unverified")
        .expect("an unattested creation");
    ev[i]["meta"]["layer"] = Value::String("proposal".into());
    write_log(&d, &ev);
    assert!(open_err(&d).contains("synthetic only"));
    // an unknown layer value
    let mut ev = good.clone();
    ev[i]["meta"]["layer"] = Value::String("draft".into());
    write_log(&d, &ev);
    assert!(open_err(&d).contains("unknown layer"));
    write_log(&d, &good);
    Vault::open(&d).unwrap();
}

#[test]
fn attestation_lines_are_checked_and_commit_refuses_raises() {
    let (d, mut v) = unattested("lines");
    let (a, _) = v
        .record_attestation("notes.md", &content_hash(b"y"), Some(1))
        .unwrap();
    Vault::open(&d).unwrap();
    let att = d.join(".rootmark/attestations.jsonl");
    let good = std::fs::read_to_string(&att).unwrap();
    for (bad, why) in [
        (
            good.replace(&a.attestation_id, &"0".repeat(32)),
            "does not match",
        ),
        (good.replace(&a.content_hash, "ABC"), "32 lowercase"),
        (
            good.replace("\"notes.md\"", "\"../notes.md\""),
            "vault-relative",
        ),
        (
            good.replace(&format!("\"at_seq\":{}", a.at_seq), "\"at_seq\":99999"),
            "",
        ),
        (format!("{good}{good}"), "recorded twice"),
        ("{not json\n".to_string(), "malformed"),
    ] {
        std::fs::write(&att, &bad).unwrap();
        let e = open_err(&d);
        assert!(e.contains(why), "{why}: {e}");
    }
    std::fs::write(&att, &good).unwrap();
    // the low-level commit never writes a late attestation
    let lin = v
        .blame("notes.md", None)
        .unwrap()
        .into_iter()
        .find(|b| b.layer == Layer::Unverified)
        .unwrap();
    let ev = rootmarkcore::Event {
        seq: 0,
        doc_path: "notes.md".into(),
        kind: rootmarkcore::EventKind::LayerChanged,
        lineage_ids: vec![lin.lineage_id.clone()],
        old_version_ids: vec![lin.version_id.clone()],
        new_version_ids: vec![lin.version_id.clone()],
        snapshot_id: None,
        meta: serde_json::json!({"from": "unverified", "to": "canonical",
            "cause": "late_attestation", "attestation_id": a.attestation_id}),
    };
    let e = v.commit(&[], vec![ev]).unwrap_err();
    assert!(format!("{e:#}").contains("only by raise_late"));
}

/// Z2 review, finding 1: a lineage changed again after the unattested change
/// (here by another unattested write) is not raised by that change's
/// attestation: the attested bytes are no longer its content.
#[test]
fn late_attestation_never_raises_a_later_version() {
    let (d, mut v) = unattested("launder");
    let unv: Vec<Raise> = v
        .blame("notes.md", None)
        .unwrap()
        .into_iter()
        .filter(|b| b.layer == Layer::Unverified)
        .map(|b| Raise {
            lineage_id: b.lineage_id,
            version_id: b.version_id,
            to: Layer::Canonical,
        })
        .collect();
    // another writer changes the same paragraph before the attestation
    let again = EDIT.replace("entirely", "entirely, then again by someone else");
    let plan = v
        .plan_ingest(
            "notes.md",
            &again,
            human("unattested_fs"),
            M,
            Some(Layer::Unverified),
            Some(&fs_actor()),
        )
        .unwrap();
    v.commit_plan(plan).unwrap();
    let (a, _) = v
        .record_attestation("notes.md", &content_hash(EDIT.as_bytes()), None)
        .unwrap();
    let raised = v.raise_late("notes.md", &unv, &a.attestation_id).unwrap();
    let changed: Vec<String> = v
        .blame("notes.md", None)
        .unwrap()
        .into_iter()
        .filter(|b| b.layer == Layer::Unverified)
        .map(|b| b.lineage_id)
        .collect();
    assert!(!changed.is_empty());
    for e in &raised {
        assert!(!changed.contains(&e.lineage_ids[0]));
    }
    // the rewritten paragraph stays unverified
    assert!(v
        .blame("notes.md", None)
        .unwrap()
        .iter()
        .any(|b| b.layer == Layer::Unverified));
    Vault::open(&d).unwrap();
}

/// Z2 review, finding 2: a raising `LayerChanged` inside an ingest (before
/// its `SnapshotCommitted`) is rejected when the vault is opened.
#[test]
fn late_attestation_inside_an_ingest_is_rejected() {
    let (d, mut v) = unattested("inside");
    let unv: Vec<Raise> = v
        .blame("notes.md", None)
        .unwrap()
        .into_iter()
        .filter(|b| b.layer == Layer::Unverified)
        .map(|b| Raise {
            lineage_id: b.lineage_id,
            version_id: b.version_id,
            to: Layer::Canonical,
        })
        .collect();
    let (a, _) = v
        .record_attestation("notes.md", &content_hash(EDIT.as_bytes()), None)
        .unwrap();
    v.raise_late("notes.md", &unv[..1], &a.attestation_id)
        .unwrap();
    Vault::open(&d).unwrap();
    // move the late LayerChanged before the ingest's SnapshotCommitted
    let mut ev = log(&d);
    let late = ev
        .iter()
        .position(|e| e["meta"]["cause"] == "late_attestation")
        .unwrap();
    let snap = ev[..late]
        .iter()
        .rposition(|e| e["kind"] == "SnapshotCommitted")
        .unwrap();
    let e = ev.remove(late);
    ev.insert(snap, e);
    for (i, e) in ev.iter_mut().enumerate() {
        e["seq"] = serde_json::json!(i);
    }
    write_log(&d, &ev);
    let err = open_err(&d);
    assert!(err.contains("inside an unfinished ingest"), "{err}");
}
