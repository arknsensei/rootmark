use mdkcore::lineage::{ctx_anchor, lineage_formula};
use mdkcore::{parse_document, EventKind, MatchParams, MatcherVersion, Provenance, Vault};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mdk-lin-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn vault(name: &str) -> Vault {
    Vault::open(tmp(name)).unwrap()
}

fn ingest(v: &mut Vault, text: &str) -> mdkcore::IngestResult {
    v.ingest("d.md", text, Provenance::default(), MatcherVersion::V2)
        .unwrap()
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

fn kinds(r: &mdkcore::IngestResult) -> Vec<EventKind> {
    r.events
        .iter()
        .map(|e| e.kind)
        .filter(|k| *k != EventKind::SnapshotCommitted)
        .collect()
}

const P1: &str = "The roaster controls heat, airflow and time for every batch of green coffee.";
const P2: &str = "Cupping is the standard tasting protocol that we use to evaluate each lot.";
const P3: &str = "Grinders are calibrated every morning before the first espresso is pulled.";
const P4: &str = "Filter coffee uses a ratio of one part coffee to sixteen parts of water.";

fn doc(ps: &[&str]) -> String {
    ps.join("\n\n") + "\n"
}

// ---------- §5.1 deterministic lineage ids

#[test]
fn lineage_ids_follow_formula() {
    let mut v = vault("formula");
    let r = ingest(&mut v, &doc(&[P1, "dup", P2, "dup", "dup"]));
    let (drafts, _) = parse_document(&doc(&[P1, "dup", P2, "dup", "dup"]), "d.md").unwrap();
    let mut k_dup = 0;
    for (b, d) in r.snapshot.blocks.iter().zip(&drafts) {
        let k = if d.normalized_text == "dup" {
            k_dup += 1;
            k_dup - 1
        } else {
            0
        };
        assert_eq!(b.0, lineage_formula("d.md", &d.version_id, k));
        let raw = blake3::hash(format!("lineage\nd.md\n{}\n{k}", d.version_id).as_bytes());
        assert_eq!(b.0, raw.to_hex()[..32]);
    }
    // same input in a fresh vault → identical ids and log bytes
    let mut w = vault("formula2");
    ingest(&mut w, &doc(&[P1, "dup", P2, "dup", "dup"]));
    assert_eq!(v.events_hash().unwrap(), w.events_hash().unwrap());
}

#[test]
fn new_lineage_never_reuses_a_used_one() {
    let mut v = vault("noreuse");
    ingest(&mut v, &doc(&[P1, "dup"]));
    let first = lineages(&v);
    // a second copy of "dup" appears: its k=0 lineage is live → next free k
    ingest(&mut v, &doc(&[P1, "dup", P2, "dup"]));
    let second = lineages(&v);
    assert_eq!(second[..2], first[..]);
    let all: std::collections::HashSet<_> = second.iter().collect();
    assert_eq!(all.len(), 4);
    // delete then recreate: the dead lineage is not resurrected
    ingest(&mut v, &doc(&[P1]));
    ingest(&mut v, &doc(&[P1, "dup"]));
    let fourth = lineages(&v);
    assert!(!second.contains(&fourth[1]));
}

// ---------- step 1: exact match + moves (LIS)

#[test]
fn identical_reingest_produces_no_events() {
    let mut v = vault("noop");
    ingest(&mut v, &doc(&[P1, P2, P3]));
    let h = v.events_hash().unwrap();
    // whitespace-only reflow keeps every version id
    let r = ingest(
        &mut v,
        &format!("{}\n\n{}\n\n{}\n", P1, P2.replace(' ', "\n"), P3),
    );
    assert!(r.events.is_empty());
    assert_eq!(v.events_hash().unwrap(), h);
}

#[test]
fn swap_moves_only_outside_lis() {
    let mut v = vault("move");
    ingest(&mut v, &doc(&[P1, P2, P3, P4]));
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[P1, P3, P2, P4]));
    assert_eq!(kinds(&r), vec![EventKind::BlockMoved]);
    let after = lineages(&v);
    assert_eq!(
        after,
        vec![
            before[0].clone(),
            before[2].clone(),
            before[1].clone(),
            before[3].clone()
        ]
    );
    // moving the first block to the end moves exactly one block
    let r = ingest(&mut v, &doc(&[P3, P2, P4, P1]));
    assert_eq!(kinds(&r), vec![EventKind::BlockMoved]);
    assert_eq!(r.events[0].lineage_ids, vec![before[0].clone()]);
    assert_eq!(r.events[0].meta["anchor_kind"], "exact");
}

// ---------- §5.2 duplicates: ctx anchor, then ordinal

#[test]
fn duplicate_ctx_anchor_keeps_remaining_copies() {
    // scattered copies; the first one is deleted (dup_delete)
    let mut v = vault("ctx");
    ingest(
        &mut v,
        &doc(&[
            P1,
            "Note: check.",
            P2,
            "Note: check.",
            P3,
            "Note: check.",
            P4,
        ]),
    );
    let before = lineages(&v);
    let r = ingest(
        &mut v,
        &doc(&[P1, P2, "Note: check.", P3, "Note: check.", P4]),
    );
    let after = lineages(&v);
    assert_eq!(after[2], before[3]);
    assert_eq!(after[4], before[5]);
    let del: Vec<_> = r
        .events
        .iter()
        .filter(|e| e.kind == EventKind::BlockDeleted)
        .collect();
    assert_eq!(del.len(), 1);
    assert_eq!(del[0].lineage_ids, vec![before[1].clone()]);
}

#[test]
fn duplicate_ctx_anchor_follows_a_moved_copy() {
    let mut v = vault("ctx2");
    ingest(&mut v, &doc(&[P1, "dup", P2, P3, "dup", P4]));
    let before = lineages(&v);
    // the second copy moves (with its neighbours) to the front
    let r = ingest(&mut v, &doc(&[P3, "dup", P4, P1, "dup", P2]));
    let after = lineages(&v);
    assert_eq!(after[1], before[4]);
    assert_eq!(after[4], before[1]);
    assert!(r.events.iter().any(|e| e.meta["anchor_kind"] == "ctx"));
}

#[test]
fn duplicate_ordinal_when_contexts_change() {
    let mut v = vault("ordinal");
    ingest(&mut v, &doc(&[P1, "dup", P2, "dup", P3]));
    let before = lineages(&v);
    // all neighbours replaced → no ctx match → k-th → k-th
    let r = ingest(&mut v, &doc(&["alpha", "dup", "beta", "dup", "gamma"]));
    let after = lineages(&v);
    assert_eq!(after[1], before[1]);
    assert_eq!(after[3], before[3]);
    assert!(!r.events.iter().any(|e| e.meta["anchor_kind"] == "ctx"));
    // ordinal with fewer copies: the first old copy is kept
    let r2 = ingest(&mut v, &doc(&["dup", "omega"]));
    assert_eq!(lineages(&v)[0], before[1]);
    assert!(r2
        .events
        .iter()
        .any(|e| e.kind == EventKind::BlockDeleted && e.lineage_ids == vec![before[3].clone()]));
}

#[test]
fn ctx_anchor_formula() {
    assert_eq!(ctx_anchor(None, "v", Some("n")), mdkcore::hash32("\nv\nn"));
    assert_ne!(
        ctx_anchor(Some("a"), "v", None),
        ctx_anchor(None, "v", Some("a"))
    );
}

// ---------- step 2: split

const LONG: &str = "Roasting turns green coffee into the brown beans we grind every day. The roaster controls heat, airflow and time, and every batch is logged carefully. Light profiles keep floral notes while dark profiles mute the acidity of the cup.";

#[test]
fn split_into_two() {
    let mut v = vault("split2");
    ingest(&mut v, &doc(&[P1, LONG, P2]));
    let before = lineages(&v);
    let (a, b) = LONG.split_at(LONG.find(" The roaster").unwrap());
    let r = ingest(&mut v, &doc(&[P1, a, b.trim(), P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockSplit]);
    let ev = &r.events[0];
    assert_eq!(ev.meta["from_lineages"][0], before[1].as_str());
    assert_eq!(ev.lineage_ids.len(), 2);
    assert_eq!(ev.meta["residual"], false);
    let blocks = v.snapshot_blocks(&r.snapshot).unwrap();
    assert_eq!(blocks[1].parents, vec![ev.old_version_ids[0].clone()]);
}

#[test]
fn split_into_three() {
    let mut v = vault("split3");
    ingest(&mut v, &doc(&[P1, LONG]));
    let parts: Vec<&str> = LONG.split_inclusive(". ").map(str::trim).collect();
    assert_eq!(parts.len(), 3);
    let r = ingest(&mut v, &doc(&[P1, parts[0], parts[1], parts[2]]));
    assert_eq!(kinds(&r), vec![EventKind::BlockSplit]);
    assert_eq!(r.events[0].new_version_ids.len(), 3);
}

// ---------- step 3: merge

#[test]
fn merge_two() {
    let mut v = vault("merge2");
    let (a, b) = LONG.split_at(LONG.find(" The roaster").unwrap());
    ingest(&mut v, &doc(&[P1, a, b.trim(), P2]));
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[P1, LONG, P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockMerged]);
    let ev = &r.events[0];
    assert_eq!(
        ev.meta["from_lineages"],
        serde_json::json!([before[1], before[2]])
    );
    assert_eq!(ev.old_version_ids.len(), 2);
}

#[test]
fn merge_three() {
    let mut v = vault("merge3");
    let parts: Vec<&str> = LONG.split_inclusive(". ").map(str::trim).collect();
    ingest(&mut v, &doc(&[parts[0], parts[1], parts[2], P2]));
    let r = ingest(&mut v, &doc(&[LONG, P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockMerged]);
    assert_eq!(r.events[0].old_version_ids.len(), 3);
}

// ---------- step 4: similarity assignment

#[test]
fn light_rephrase_is_update() {
    let mut v = vault("upd");
    ingest(&mut v, &doc(&["# Title", P1, P2]));
    let before = lineages(&v);
    let edited = P1.replace("controls", "manages").replace("every", "each");
    let r = ingest(&mut v, &doc(&["# Title", &edited, P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockUpdated]);
    assert_eq!(lineages(&v), before);
    assert!(r.events[0].meta["score"].as_f64().unwrap() >= 0.60);
}

#[test]
fn hungarian_assigns_best_pairs_and_respects_type() {
    let mut v = vault("hung");
    ingest(&mut v, &doc(&[P1, P2, "- a list item about grinders"]));
    let before = lineages(&v);
    // both paragraphs edited and swapped; the list item becomes a paragraph
    let e1 = P1.replace("roaster", "machine");
    let e2 = P2.replace("standard", "usual");
    let r = ingest(&mut v, &doc(&[&e2, &e1, "a list item about grinders"]));
    let after = lineages(&v);
    assert_eq!(after[0], before[1]);
    assert_eq!(after[1], before[0]);
    assert!(
        !before.contains(&after[2]),
        "type change must not inherit lineage"
    );
    assert!(kinds(&r).contains(&EventKind::BlockCreated));
    assert!(kinds(&r).contains(&EventKind::BlockDeleted));
}

#[test]
fn theta_changes_the_outcome() {
    let heavy = "The roaster manages heat and air for each batch of raw coffee beans today.";
    let run = |theta: f64, name: &str| {
        let mut v = Vault::open_with(
            tmp(name),
            MatchParams {
                theta,
                theta_split: 0.70,
            },
        )
        .unwrap();
        ingest(&mut v, &doc(&[P1, P2]));
        let r = ingest(&mut v, &doc(&[heavy, P2]));
        kinds(&r)
    };
    assert_eq!(run(0.50, "th-lo"), vec![EventKind::BlockUpdated]);
    assert_eq!(
        run(0.70, "th-hi"),
        vec![EventKind::BlockCreated, EventKind::BlockDeleted]
    );
}

// ---------- step 5: residual check

const BIG: &str = "Espresso is brewed with nine bars of pressure for about twenty eight seconds, using eighteen grams of finely ground coffee and yielding thirty six grams of liquid in the cup, which the shop treats as its house standard for every blend it serves to customers during the day and the evening. Keep the portafilter dry and the basket clean.";

#[test]
fn residual_after_large_part_becomes_split() {
    let mut v = vault("resid");
    ingest(&mut v, &doc(&[P1, BIG, P2]));
    let cut = BIG.find(" Keep the").unwrap();
    let (big, small) = (&BIG[..cut], BIG[cut..].trim());
    // the residual must be < 15 % (else step 2 handles it) and large enough
    // that the concatenation gains more than RESIDUAL_MARGIN
    assert!(
        (small.len() as f64) < 0.15 * BIG.len() as f64,
        "step 2 must not see it"
    );
    assert!((small.len() as f64) > 0.10 * BIG.len() as f64);
    let r = ingest(&mut v, &doc(&[P1, big, small, P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockSplit]);
    assert_eq!(r.events[0].meta["residual"], true);
    assert_eq!(r.events[0].new_version_ids.len(), 2);
}

#[test]
fn residual_before_large_part_becomes_split_unrelated_does_not() {
    let mut v = vault("resid2");
    let lead = "Before anything else, warm the cup with water.";
    let whole = format!("{lead} {BIG}");
    ingest(&mut v, &doc(&[P1, &whole]));
    let r = ingest(&mut v, &doc(&[P1, lead, BIG]));
    assert_eq!(kinds(&r), vec![EventKind::BlockSplit]);
    assert_eq!(r.events[0].meta["residual"], true);
    // an unrelated short neighbour stays a separate creation
    let mut w = vault("resid3");
    ingest(&mut w, &doc(&[P1, BIG]));
    let big2 = BIG.replace("nine bars", "9 bars");
    let r = ingest(&mut w, &doc(&[P1, "Totally unrelated.", &big2]));
    assert_eq!(
        kinds(&r),
        vec![EventKind::BlockCreated, EventKind::BlockUpdated]
    );
}

// ---------- step 6: create / delete

#[test]
fn insert_creates_and_remove_deletes() {
    let mut v = vault("cd");
    ingest(&mut v, &doc(&[P1, P2]));
    let before = lineages(&v);
    let r = ingest(&mut v, &doc(&[P1, P3, P2]));
    assert_eq!(kinds(&r), vec![EventKind::BlockCreated]);
    let r = ingest(&mut v, &doc(&[P1, P3]));
    assert_eq!(kinds(&r), vec![EventKind::BlockDeleted]);
    assert_eq!(r.events[0].lineage_ids, vec![before[1].clone()]);
    assert_eq!(r.changed_lineages, vec![before[1].clone()]);
}

#[test]
fn first_ingest_creates_everything_and_empty_doc_has_snapshot() {
    let mut v = vault("first");
    let r = ingest(&mut v, &doc(&[P1, P2]));
    assert_eq!(
        kinds(&r),
        vec![EventKind::BlockCreated, EventKind::BlockCreated]
    );
    let r = v
        .ingest(
            "empty.md",
            "---\na: 1\n---\n",
            Provenance::default(),
            MatcherVersion::V2,
        )
        .unwrap();
    assert!(r.snapshot.blocks.is_empty());
    assert_eq!(r.events.len(), 1);
}

// ---------- resolve_truth, replay

#[test]
fn resolve_truth_errs_on_bad_reference() {
    let mut v = vault("resolve");
    ingest(&mut v, &doc(&[P1, "dup", "dup"]));
    let snap = v.snapshot("d.md").unwrap().unwrap();
    let dup_vid = snap.blocks[1].1.clone();
    assert_eq!(
        v.resolve_truth("d.md", &dup_vid, 1).unwrap(),
        snap.blocks[2].0
    );
    assert!(v.resolve_truth("d.md", &dup_vid, 2).is_err());
    assert!(v
        .resolve_truth("d.md", "0123456789abcdef0123456789abcdef", 0)
        .is_err());
    assert!(v.resolve_truth("nope.md", &dup_vid, 0).is_err());
}

#[test]
fn replay_after_many_edits_equals_head() {
    let mut v = vault("replay");
    ingest(&mut v, &doc(&[P1, LONG, P2]));
    let (a, b) = LONG.split_at(LONG.find(" The roaster").unwrap());
    ingest(&mut v, &doc(&[P2, a, b.trim(), P1]));
    ingest(&mut v, &doc(&[P2, LONG, P3]));
    let head = v.snapshot("d.md").unwrap().unwrap();
    assert_eq!(v.replay("d.md").unwrap().unwrap(), head);
    let w = Vault::open(v.root()).unwrap();
    assert_eq!(w.snapshot("d.md").unwrap().unwrap(), head);
}

#[test]
fn theta_split_changes_the_outcome() {
    // a split whose parts were also edited: concat similarity lies between 0.60 and 0.80
    let (a, b) = LONG.split_at(LONG.find(" The roaster").unwrap());
    let a2 = a
        .replace("Roasting turns green coffee", "Our process makes raw seeds")
        .replace("every day", "daily");
    let b2 = b.trim().replace(
        "controls heat, airflow and time",
        "sets the temperature and duration",
    );
    // returns (event kinds, split came from the residual check?)
    let run = |ts: f64, name: &str| {
        let mut v = Vault::open_with(
            tmp(name),
            MatchParams {
                theta: 0.60,
                theta_split: ts,
            },
        )
        .unwrap();
        ingest(&mut v, &doc(&[P1, LONG, P2]));
        let r = ingest(&mut v, &doc(&[P1, &a2, &b2, P2]));
        let residual = r
            .events
            .iter()
            .find(|e| e.kind == EventKind::BlockSplit)
            .map(|e| e.meta["residual"].clone());
        (kinds(&r), residual)
    };
    // low THETA_SPLIT: step 2 detects the split directly
    assert_eq!(
        run(0.60, "ts-lo"),
        (vec![EventKind::BlockSplit], Some(serde_json::json!(false)))
    );
    // high THETA_SPLIT: step 2 rejects it; only the step 5 residual check recovers it
    assert_ne!(run(0.80, "ts-hi").1, Some(serde_json::json!(false)));
}

#[test]
fn residual_right_side_tried_when_left_neighbours_are_unmatched() {
    // regression: three unmatched blocks before the matched part must not
    // shrink the right-hand residual window
    let mut v = vault("resid-left");
    ingest(&mut v, &doc(&[P1, BIG]));
    let cut = BIG.find(" Keep the").unwrap();
    let (big, small) = (&BIG[..cut], BIG[cut..].trim());
    let r = ingest(
        &mut v,
        &doc(&[
            P1,
            "New one here.",
            "New two here.",
            "New three here.",
            big,
            small,
        ]),
    );
    let k = kinds(&r);
    assert!(k.contains(&EventKind::BlockSplit), "{k:?}");
    let sp = r
        .events
        .iter()
        .find(|e| e.kind == EventKind::BlockSplit)
        .unwrap();
    assert_eq!(sp.meta["residual"], true);
    assert_eq!(sp.new_version_ids.len(), 2);
}
