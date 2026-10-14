//! P1-B/B1: layers (spec §7) — defaults by `author_kind` (C2-01), layer
//! transitions as events and replay (C2-08), opening v3-final vaults (B-5).

use mdkcore::{AuthorKind, EventKind, Layer, MatcherVersion, Provenance, Vault};
use std::fs;
use std::path::{Path, PathBuf};

const M: MatcherVersion = MatcherVersion::V3;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-layers-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn prov(kind: AuthorKind) -> Provenance {
    Provenance {
        author_kind: kind,
        source: format!("{kind:?}").to_lowercase(),
    }
}

fn layers(v: &Vault, doc: &str) -> Vec<Layer> {
    let s = v.snapshot(doc).unwrap().unwrap();
    v.snapshot_blocks(&s)
        .unwrap()
        .iter()
        .map(|b| b.layer)
        .collect()
}

/// layer of every live lineage of every document
fn layer_map(v: &Vault) -> Vec<(String, String, Layer, bool)> {
    let mut out = Vec::new();
    for d in v.doc_paths() {
        let s = v.snapshot(&d).unwrap().unwrap();
        for (l, _) in &s.blocks {
            out.push((
                d.clone(),
                l.clone(),
                v.layer_of(l).unwrap(),
                v.is_inactive(l).unwrap(),
            ));
        }
    }
    out
}

const DOC: &str = "# Title\n\nFirst paragraph about espresso extraction and grind size.\n\nSecond paragraph about milk texture and latte art.\n";

#[test]
fn c2_01_default_layer_by_author_kind() {
    let root = tmp("c201");
    let mut v = Vault::open(&root).unwrap();
    for (doc, kind, want) in [
        ("h.md", AuthorKind::Human, Layer::Canonical),
        ("a.md", AuthorKind::Agent, Layer::Proposal),
        ("t.md", AuthorKind::Transcript, Layer::Proposal),
        ("s.md", AuthorKind::Synthetic, Layer::Canonical),
    ] {
        v.ingest(doc, DOC, prov(kind), M).unwrap();
        assert!(layers(&v, doc).iter().all(|&l| l == want), "{doc}");
    }
    // synthetic: as declared
    v.ingest_declared(
        "d.md",
        DOC,
        prov(AuthorKind::Synthetic),
        M,
        Some(Layer::Derived),
    )
    .unwrap();
    assert!(layers(&v, "d.md").iter().all(|&l| l == Layer::Derived));
    // a declaration is for synthetic content only
    let e = v
        .ingest_declared(
            "x.md",
            DOC,
            prov(AuthorKind::Agent),
            M,
            Some(Layer::Canonical),
        )
        .unwrap_err();
    assert!(format!("{e:#}").contains("synthetic content only"));
    // replay rebuilds the same layers
    let before = layer_map(&v);
    drop(v);
    assert_eq!(layer_map(&Vault::open(&root).unwrap()), before);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn agent_edit_lowers_canonical_with_layer_changed() {
    let root = tmp("lower");
    let mut v = Vault::open(&root).unwrap();
    v.ingest("doc.md", DOC, prov(AuthorKind::Human), M).unwrap();
    let edited = DOC.replace("grind size", "grind size and water temperature");
    let r = v
        .ingest("doc.md", &edited, prov(AuthorKind::Agent), M)
        .unwrap();
    let kinds: Vec<EventKind> = r.events.iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![
            EventKind::BlockUpdated,
            EventKind::LayerChanged,
            EventKind::SnapshotCommitted
        ]
    );
    let lc = &r.events[1];
    assert_eq!(lc.meta["from"], "canonical");
    assert_eq!(lc.meta["to"], "proposal");
    assert_eq!(lc.meta["cause"], "ingest");
    assert_eq!(lc.lineage_ids, r.events[0].lineage_ids);
    assert_eq!(
        layers(&v, "doc.md"),
        vec![Layer::Canonical, Layer::Proposal, Layer::Canonical]
    );
    // a human edit never raises it back (no implicit approval, K12/K16)
    let again = edited.replace("water temperature", "water temperature and dose");
    let r = v
        .ingest("doc.md", &again, prov(AuthorKind::Human), M)
        .unwrap();
    assert!(r.events.iter().all(|e| e.kind != EventKind::LayerChanged));
    assert_eq!(
        layers(&v, "doc.md"),
        vec![Layer::Canonical, Layer::Proposal, Layer::Canonical]
    );
    let before = layer_map(&v);
    drop(v);
    assert_eq!(layer_map(&Vault::open(&root).unwrap()), before);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn merge_takes_the_lowest_parent_layer() {
    let root = tmp("merge");
    let mut v = Vault::open(&root).unwrap();
    let base = "# T\n\nAlpha paragraph with enough words to be matched reliably here.\n";
    v.ingest("m.md", base, prov(AuthorKind::Human), M).unwrap();
    let two = format!("{base}\nBeta paragraph proposed by an agent, also long enough.\n");
    v.ingest("m.md", &two, prov(AuthorKind::Agent), M).unwrap();
    assert_eq!(
        layers(&v, "m.md"),
        vec![Layer::Canonical, Layer::Canonical, Layer::Proposal]
    );
    // a human merges both paragraphs into one: the result holds proposal text
    let merged = "# T\n\nAlpha paragraph with enough words to be matched reliably here. Beta paragraph proposed by an agent, also long enough.\n";
    let r = v
        .ingest("m.md", merged, prov(AuthorKind::Human), M)
        .unwrap();
    assert!(r.events.iter().any(|e| e.kind == EventKind::BlockMerged));
    assert!(r.events.iter().any(|e| e.kind == EventKind::LayerChanged));
    assert_eq!(layers(&v, "m.md"), vec![Layer::Canonical, Layer::Proposal]);
    let before = layer_map(&v);
    drop(v);
    assert_eq!(layer_map(&Vault::open(&root).unwrap()), before);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn c2_08_tampered_layer_event_fails_replay() {
    let root = tmp("tamper");
    let mut v = Vault::open(&root).unwrap();
    v.ingest("doc.md", DOC, prov(AuthorKind::Human), M).unwrap();
    let edited = DOC.replace("latte art", "latte art and cups");
    v.ingest("doc.md", &edited, prov(AuthorKind::Agent), M)
        .unwrap();
    drop(v);
    let p = root.join(".mdkernel/events.jsonl");
    let log = fs::read_to_string(&p).unwrap();
    assert!(log.contains(r#""to":"proposal""#));
    fs::write(&p, log.replace(r#""to":"proposal""#, r#""to":"derived""#)).unwrap();
    let e = Vault::open(&root).err().expect("tampered log must fail");
    assert!(format!("{e:#}").contains("LayerChanged"), "{e:#}");
    fs::remove_dir_all(&root).unwrap();
}

/// B-5 (in-process half; the v3-final binary half is scripts/check_v3_compat.sh):
/// the v3-final event logs (no layer events) open with the current core, the
/// log bytes stay unchanged, replay gives the same heads as a fresh ingest
/// and every layer is derived from `author_kind`.
#[test]
fn b5_v3_final_logs_open_unchanged() {
    for (m, name) in [(MatcherVersion::V3, "v3"), (MatcherVersion::V2, "v2")] {
        let root = tmp(&format!("b5-{name}"));
        fs::create_dir_all(root.join(".mdkernel")).unwrap();
        let src = here(&format!(
            "tests/fixtures/v3_final/mini_vault_edit.{name}.events.jsonl"
        ));
        let bytes = fs::read(&src).unwrap();
        fs::write(root.join(".mdkernel/events.jsonl"), &bytes).unwrap();
        let old = Vault::open(&root).unwrap();
        let fresh_root = tmp(&format!("b5-{name}-fresh"));
        let mut fresh = Vault::open(&fresh_root).unwrap();
        fresh
            .ingest_dir(&here("../spec/fixtures/mini_vault"), m)
            .unwrap();
        fresh
            .ingest_dir(&here("tests/fixtures/v3_final/edit"), m)
            .unwrap();
        assert_eq!(old.doc_paths(), fresh.doc_paths());
        for d in old.doc_paths() {
            let a = old.snapshot(&d).unwrap().unwrap();
            let b = fresh.snapshot(&d).unwrap().unwrap();
            assert_eq!(a.snapshot_id, b.snapshot_id, "{d}");
            for (l, _) in &a.blocks {
                assert_eq!(old.layer_of(l).unwrap(), Layer::Canonical);
            }
        }
        assert_eq!(old.events_hash().unwrap(), fresh.events_hash().unwrap());
        drop(old);
        assert_eq!(
            fs::read(root.join(".mdkernel/events.jsonl")).unwrap(),
            bytes
        );
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&fresh_root).unwrap();
    }
}

/// Split (B1 review note 4): an agent splits a canonical paragraph — the
/// dominant piece keeps its lineage and is lowered by `LayerChanged`, the new
/// piece starts as a proposal; a human split of a proposal stays proposal.
#[test]
fn split_pieces_take_the_lowest_layer() {
    let root = tmp("split");
    let mut v = Vault::open(&root).unwrap();
    let one = "# S\n\nThe grinder needs new burrs every year because steel wears down. The water filter should be replaced each month to protect the boiler.\n";
    v.ingest("s.md", one, prov(AuthorKind::Human), M).unwrap();
    let two = "# S\n\nThe grinder needs new burrs every year because steel wears down.\n\nThe water filter should be replaced each month to protect the boiler.\n";
    let r = v.ingest("s.md", two, prov(AuthorKind::Agent), M).unwrap();
    let kinds: Vec<EventKind> = r.events.iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![
            EventKind::BlockSplit,
            EventKind::LayerChanged,
            EventKind::SnapshotCommitted
        ]
    );
    assert_eq!(
        layers(&v, "s.md"),
        vec![Layer::Canonical, Layer::Proposal, Layer::Proposal]
    );
    let before = layer_map(&v);
    drop(v);
    assert_eq!(layer_map(&Vault::open(&root).unwrap()), before);
    fs::remove_dir_all(&root).unwrap();
}
