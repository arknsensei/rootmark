//! Matcher v3 (SPEC_DELTA §5.3, §7): K4 dominant inheritance, no minimum part
//! ratio, merge-direction residual check, per-vault version lock, and V2
//! byte-equality with v2-final.

use mdkcore::lineage::{THETA_SPLIT_V3, THETA_V3};
use mdkcore::{EventKind, IngestResult, MatchParams, MatcherVersion, Provenance, Vault};
use std::path::{Path, PathBuf};

const V2: MatcherVersion = MatcherVersion::V2;
const V3: MatcherVersion = MatcherVersion::V3;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mdk-lin3-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn ingest(v: &mut Vault, text: &str, m: MatcherVersion) -> IngestResult {
    v.ingest("d.md", text, Provenance::default(), m).unwrap()
}

fn lineages(v: &Vault) -> Vec<String> {
    v.snapshot("d.md")
        .unwrap()
        .unwrap()
        .blocks
        .into_iter()
        .map(|(l, _)| l)
        .collect()
}

fn parents(v: &Vault) -> Vec<Vec<String>> {
    let s = v.snapshot("d.md").unwrap().unwrap();
    v.snapshot_blocks(&s)
        .unwrap()
        .into_iter()
        .map(|b| b.parents)
        .collect()
}

fn versions(v: &Vault) -> Vec<String> {
    v.snapshot("d.md")
        .unwrap()
        .unwrap()
        .blocks
        .into_iter()
        .map(|(_, x)| x)
        .collect()
}

fn block_events(r: &IngestResult) -> Vec<&mdkcore::Event> {
    r.events
        .iter()
        .filter(|e| e.kind != EventKind::SnapshotCommitted)
        .collect()
}

fn doc(ps: &[&str]) -> String {
    ps.join("\n\n") + "\n"
}

fn replay_equals_head(v: &Vault) {
    let head = v.snapshot("d.md").unwrap().unwrap();
    assert_eq!(v.replay("d.md").unwrap().unwrap(), head);
    let reopened = Vault::open(v.root()).unwrap();
    assert_eq!(reopened.snapshot("d.md").unwrap().unwrap(), head);
}

const X: &str = "Opening notes describe the scope of this page and who maintains it today.";
const Y: &str = "Closing remarks list the people to contact when something here looks wrong.";
const A: &str = "The roaster controls heat, airflow and drum speed for every batch of green coffee, and the operator writes each change into the roast log so that the next batch can be compared.";
const B: &str = "Cupping follows within one day of roasting.";
const C: &str = "Grinders are calibrated every morning before service.";

// ---------- K4 split

#[test]
fn k4_split_two_parts_longest_keeps_lineage() {
    let mut v = Vault::open(tmp("split2")).unwrap();
    ingest(&mut v, &doc(&[X, &format!("{A} {B}"), Y]), V3);
    let before = lineages(&v);
    let old_vid = versions(&v)[1].clone();
    let r = ingest(&mut v, &doc(&[X, A, B, Y]), V3);
    let after = lineages(&v);
    assert_eq!(
        after[1], before[1],
        "longest part (A) keeps the old lineage"
    );
    assert!(
        !before.contains(&after[2]),
        "the other part gets a new lineage"
    );
    assert_eq!(parents(&v)[1], vec![old_vid.clone()]);
    assert_eq!(parents(&v)[2], vec![old_vid]);
    let evs = block_events(&r);
    assert_eq!(evs.len(), 1);
    let e = evs[0];
    assert_eq!(e.kind, EventKind::BlockSplit);
    assert_eq!(e.meta["dominant"], after[1].as_str());
    assert!(e.lineage_ids.contains(&after[1]));
    assert_eq!(e.meta["matcher_version"], "v3");
    assert_eq!(e.meta["from_lineages"][0], before[1].as_str());
    replay_equals_head(&v);
}

#[test]
fn k4_split_dominant_is_longest_not_first() {
    let mut v = Vault::open(tmp("split2b")).unwrap();
    ingest(&mut v, &doc(&[X, &format!("{B} {A}"), Y]), V3);
    let before = lineages(&v);
    ingest(&mut v, &doc(&[X, B, A, Y]), V3);
    let after = lineages(&v);
    assert_eq!(after[2], before[1], "A (second, longest) keeps the lineage");
    assert!(!before.contains(&after[1]));
    replay_equals_head(&v);
}

#[test]
fn k4_split_three_parts() {
    let mut v = Vault::open(tmp("split3")).unwrap();
    ingest(&mut v, &doc(&[X, &format!("{B} {A} {C}"), Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, B, A, C, Y]), V3);
    let after = lineages(&v);
    assert_eq!(after[2], before[1]);
    assert!(!before.contains(&after[1]) && !before.contains(&after[3]));
    assert_ne!(after[1], after[3]);
    let evs = block_events(&r);
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].kind, EventKind::BlockSplit);
    assert_eq!(evs[0].lineage_ids, after[1..4].to_vec());
    assert_eq!(evs[0].meta["dominant"], after[2].as_str());
    for p in &parents(&v)[1..4] {
        assert_eq!(p.len(), 1);
    }
    replay_equals_head(&v);
}

#[test]
fn v2_split_still_gives_every_part_a_new_lineage() {
    let mut v = Vault::open(tmp("split2-v2")).unwrap();
    ingest(&mut v, &doc(&[X, &format!("{A} {B}"), Y]), V2);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, A, B, Y]), V2);
    let after = lineages(&v);
    assert!(!before.contains(&after[1]) && !before.contains(&after[2]));
    let e = block_events(&r)[0];
    assert_eq!(e.kind, EventKind::BlockSplit);
    assert!(e.meta.get("dominant").is_none());
    assert_eq!(e.meta["matcher_version"], "v2");
}

// ---------- K4 merge

#[test]
fn k4_merge_two_blocks_keeps_longest_lineage() {
    let mut v = Vault::open(tmp("merge2")).unwrap();
    ingest(&mut v, &doc(&[X, B, A, Y]), V3);
    let before = lineages(&v);
    let olds = versions(&v)[1..3].to_vec();
    let r = ingest(&mut v, &doc(&[X, &format!("{B} {A}"), Y]), V3);
    let after = lineages(&v);
    assert_eq!(
        after[1], before[2],
        "merge result keeps the longest old (A)"
    );
    assert_eq!(parents(&v)[1], olds);
    let e = block_events(&r)[0];
    assert_eq!(e.kind, EventKind::BlockMerged);
    assert_eq!(e.meta["dominant"], after[1].as_str());
    assert_eq!(e.lineage_ids, vec![after[1].clone()]);
    assert_eq!(e.old_version_ids, olds);
    replay_equals_head(&v);
}

#[test]
fn k4_merge_three_blocks() {
    let mut v = Vault::open(tmp("merge3")).unwrap();
    ingest(&mut v, &doc(&[X, B, A, C, Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, &format!("{B} {A} {C}"), Y]), V3);
    let after = lineages(&v);
    assert_eq!(after[1], before[2]);
    assert_eq!(parents(&v)[1].len(), 3);
    let e = block_events(&r)[0];
    assert_eq!(e.kind, EventKind::BlockMerged);
    assert_eq!(e.meta["from_lineages"].as_array().unwrap().len(), 3);
    replay_equals_head(&v);
}

// ---------- minimum part ratio removed

const SHORT: &str = "Yes, always.";

#[test]
fn tiny_part_split_detected_in_v3_not_in_v2() {
    // SHORT is far below 15 % of the whole: v2 never forms a split candidate
    let whole = format!("{A} {SHORT}");
    let mut v2 = Vault::open(tmp("tiny-v2")).unwrap();
    ingest(&mut v2, &doc(&[X, &whole, Y]), V2);
    let r2 = ingest(&mut v2, &doc(&[X, A, SHORT, Y]), V2);
    let k2: Vec<EventKind> = block_events(&r2).iter().map(|e| e.kind).collect();
    assert!(!k2.contains(&EventKind::BlockSplit), "{k2:?}");

    let mut v3 = Vault::open(tmp("tiny-v3")).unwrap();
    ingest(&mut v3, &doc(&[X, &whole, Y]), V3);
    let before = lineages(&v3);
    let r3 = ingest(&mut v3, &doc(&[X, A, SHORT, Y]), V3);
    let k3: Vec<EventKind> = block_events(&r3).iter().map(|e| e.kind).collect();
    assert_eq!(k3, vec![EventKind::BlockSplit]);
    assert_eq!(lineages(&v3)[1], before[1]);
}

#[test]
fn tiny_part_merge_detected_in_v3_not_in_v2() {
    let mut v2 = Vault::open(tmp("tinym-v2")).unwrap();
    ingest(&mut v2, &doc(&[X, A, SHORT, Y]), V2);
    let r2 = ingest(&mut v2, &doc(&[X, &format!("{A} {SHORT}"), Y]), V2);
    let k2: Vec<EventKind> = block_events(&r2).iter().map(|e| e.kind).collect();
    assert!(!k2.contains(&EventKind::BlockMerged), "{k2:?}");

    let mut v3 = Vault::open(tmp("tinym-v3")).unwrap();
    ingest(&mut v3, &doc(&[X, A, SHORT, Y]), V3);
    let before = lineages(&v3);
    let r3 = ingest(&mut v3, &doc(&[X, &format!("{A} {SHORT}"), Y]), V3);
    let k3: Vec<EventKind> = block_events(&r3).iter().map(|e| e.kind).collect();
    assert_eq!(k3, vec![EventKind::BlockMerged]);
    assert_eq!(lineages(&v3)[1], before[1]);
}

// ---------- step 6: merge-direction residual check

/// Step 3 disabled (THETA_SPLIT unreachable) so only step 6 can merge.
fn no_step3(name: &str) -> Vault {
    Vault::open_with(
        tmp(name),
        MatchParams {
            theta: THETA_V3,
            theta_split: 1.01,
        },
    )
    .unwrap()
}

const A_EDIT: &str = "The roaster controls heat, airflow and drum speed for every batch of green coffee, and the operator records each change into the roast log so that the next batch can be compared.";

fn merged_residual(r: &IngestResult) -> &mdkcore::Event {
    let evs = block_events(r);
    assert_eq!(
        evs.len(),
        1,
        "{:?}",
        evs.iter().map(|e| e.kind).collect::<Vec<_>>()
    );
    assert_eq!(evs[0].kind, EventKind::BlockMerged);
    assert_eq!(evs[0].meta["residual"], true);
    evs[0]
}

#[test]
fn residual_merge_right_neighbour() {
    let mut v = no_step3("res-right");
    ingest(&mut v, &doc(&[X, A, B, Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, &format!("{A_EDIT} {B}"), Y]), V3);
    let e = merged_residual(&r);
    assert_eq!(lineages(&v)[1], before[1]);
    assert_eq!(e.old_version_ids.len(), 2);
    assert_eq!(e.meta["dominant"], before[1].as_str());
    replay_equals_head(&v);
}

#[test]
fn residual_merge_left_neighbour() {
    let mut v = no_step3("res-left");
    ingest(&mut v, &doc(&[X, B, A, Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, &format!("{B} {A_EDIT}"), Y]), V3);
    let e = merged_residual(&r);
    assert_eq!(lineages(&v)[1], before[2]);
    assert_eq!(
        e.meta["from_lineages"],
        serde_json::json!([before[1], before[2]])
    );
    replay_equals_head(&v);
}

#[test]
fn residual_merge_three_blocks() {
    let mut v = no_step3("res-three");
    ingest(&mut v, &doc(&[X, B, A, C, Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, &format!("{B} {A_EDIT} {C}"), Y]), V3);
    let e = merged_residual(&r);
    assert_eq!(e.old_version_ids.len(), 3);
    assert_eq!(lineages(&v)[1], before[2]);
    replay_equals_head(&v);
}

#[test]
fn residual_merge_ignores_unrelated_deleted_neighbour() {
    let mut v = no_step3("res-unrelated");
    ingest(&mut v, &doc(&[X, A, C, Y]), V3);
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[X, A_EDIT, Y]), V3);
    let kinds: Vec<EventKind> = block_events(&r).iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![EventKind::BlockUpdated, EventKind::BlockDeleted]
    );
    assert_eq!(lineages(&v)[1], before[1]);
}

#[test]
fn v2_has_no_residual_merge() {
    let mut v = Vault::open_with(
        tmp("res-v2"),
        MatchParams {
            theta: THETA_V3,
            theta_split: 1.01,
        },
    )
    .unwrap();
    ingest(&mut v, &doc(&[X, A, B, Y]), V2);
    let r = ingest(&mut v, &doc(&[X, &format!("{A_EDIT} {B}"), Y]), V2);
    let kinds: Vec<EventKind> = block_events(&r).iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![EventKind::BlockUpdated, EventKind::BlockDeleted]
    );
}

// ---------- version lock

#[test]
fn mixed_versions_are_refused() {
    let root = tmp("lock");
    let mut v = Vault::open(&root).unwrap();
    assert_eq!(v.matcher_lock(), None);
    ingest(&mut v, &doc(&[X]), V2);
    assert_eq!(v.matcher_lock(), Some(V2));
    let e = v
        .ingest("d.md", &doc(&[X, Y]), Provenance::default(), V3)
        .unwrap_err();
    assert!(format!("{e:#}").contains("locked to matcher v2"), "{e:#}");
    // also for another document and after reopening
    assert!(v
        .ingest("other.md", &doc(&[Y]), Provenance::default(), V3)
        .is_err());
    let mut w = Vault::open(&root).unwrap();
    assert_eq!(w.matcher_lock(), Some(V2));
    assert!(w
        .ingest("d.md", &doc(&[X, Y]), Provenance::default(), V3)
        .is_err());
    w.ingest("d.md", &doc(&[X, Y]), Provenance::default(), V2)
        .unwrap();
}

#[test]
fn v3_lock_refuses_v2() {
    let root = tmp("lock3");
    let mut v = Vault::open(&root).unwrap();
    ingest(&mut v, &doc(&[X]), V3);
    let mut w = Vault::open(&root).unwrap();
    assert_eq!(w.matcher_lock(), Some(V3));
    assert!(w
        .ingest("d.md", &doc(&[X, Y]), Provenance::default(), V2)
        .is_err());
}

#[test]
fn default_matcher_is_v3_and_params_follow_version() {
    assert_eq!(MatcherVersion::default(), V3);
    let v = Vault::open(tmp("params")).unwrap();
    assert_eq!(v.params_for(V2), MatchParams::default());
    assert_eq!(
        v.params_for(V3),
        MatchParams {
            theta: THETA_V3,
            theta_split: THETA_SPLIT_V3
        }
    );
}

// ---------- V2 byte-equality with v2-final (mini_vault)

fn mini_vault() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/mini_vault")
}

fn mini_hash(name: &str, m: MatcherVersion) -> String {
    let mut v = Vault::open(tmp(name)).unwrap();
    v.ingest_dir(&mini_vault(), m).unwrap();
    v.events_hash().unwrap()
}

#[test]
fn mini_vault_v2_hash_equals_v2_final() {
    assert_eq!(mini_hash("mini-v2", V2), "8914f9b418ae360d49c85c951050f432");
}

#[test]
fn mini_vault_v3_hash_differs_and_is_deterministic() {
    let a = mini_hash("mini-v3a", V3);
    let b = mini_hash("mini-v3b", V3);
    assert_eq!(a, b);
    assert_ne!(a, "8914f9b418ae360d49c85c951050f432");
}
