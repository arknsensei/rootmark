//! P1-C/C4 (TASKS C4; GOAL C-7, spec §7, §8): layers as of the checkpoint,
//! `include_proposal`, `derived` never returned, a later ingest does not
//! change results at an older checkpoint, and every search writes one trace
//! whose `trace_id` follows K-B1.2 (computed here independently).

use rootmarkcore::{
    Actor, ActorKind, AuthorKind, HumanOnly, Layer, MatcherVersion, Provenance, TransitionKind,
    Vault,
};
use rootmarkindex::embed::OnnxMiniLm;
use rootmarkindex::search::{self, Mode, Searcher, View, PROPOSAL_WEIGHT};
use rootmarkindex::store::{self, Store};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn embedder() -> &'static OnnxMiniLm {
    static E: OnceLock<OnnxMiniLm> = OnceLock::new();
    E.get_or_init(|| OnnxMiniLm::load_default().expect("model (run `make models`)"))
}

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("rootmark-c4-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

const MODES: [Mode; 3] = [Mode::Flat, Mode::Lexical, Mode::Hybrid];

fn agent() -> Provenance {
    Provenance {
        author_kind: AuthorKind::Agent,
        source: "claude-code".into(),
    }
}

/// mini_vault (human) + agent documents (proposal) + agent edits of human
/// paragraphs (proposal blocks inside otherwise canonical chunks) + one
/// proposal demoted to derived; checkpoint "gov".
fn governed_vault(name: &str) -> (PathBuf, String) {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&here("../spec/fixtures/mini_vault"), MatcherVersion::V3)
        .unwrap();
    for (doc, text) in [
        ("agent/espresso-notes.md", "# Agent espresso notes\n\n## Grind\n\nThe espresso grind should be finer when shots run fast; dial in with small steps.\n\n## Water\n\nWater temperature around 93 degrees suits most medium roasts.\n"),
        ("agent/roast-ideas.md", "# Roast ideas\n\n## Development\n\nA longer development time after first crack sweetens light roasts.\n\n## Cooling\n\nCooling the beans quickly stops the roast and preserves acidity.\n"),
        ("agent/cleaning.md", "# Cleaning\n\nBackflush the group head daily and soak the portafilter baskets weekly.\n"),
    ] {
        v.ingest(doc, text, agent(), MatcherVersion::V3).unwrap();
    }
    // an agent edit of a human document: its edited paragraph becomes proposal
    let p = here("../spec/fixtures/mini_vault/espresso.md");
    let text = fs::read_to_string(&p).unwrap();
    let para = text
        .split("\n\n")
        .find(|b| !b.starts_with('#') && b.len() > 40)
        .unwrap()
        .to_string();
    let edited = text.replacen(
        &para,
        &format!("{para} The agent adds a note on channeling."),
        1,
    );
    v.ingest("espresso.md", &edited, agent(), MatcherVersion::V3)
        .unwrap();
    // demote one agent proposal to derived (human actor, human-only policy)
    let lin = v.snapshot("agent/cleaning.md").unwrap().unwrap().blocks[1]
        .0
        .clone();
    let req = v
        .request(
            TransitionKind::Demote,
            &lin,
            Layer::Derived,
            Actor::new(ActorKind::Human, "editor").unwrap(),
            "machine summary, not content",
        )
        .unwrap();
    v.demote(&req, &HumanOnly).unwrap();
    let cp = v.checkpoint(Some("gov")).unwrap();
    store::build(&v, embedder()).unwrap();
    (root, cp.checkpoint_id)
}

/// Layer of each `(doc_path, chunk_id)` of the checkpoint, from `blame`
/// (independent of the index's own layer resolution).
fn blame_layers(
    v: &Vault,
    s: &Store,
    cp: &str,
) -> std::collections::HashMap<(String, String), Layer> {
    let view = View::new(v, s, cp).unwrap();
    let mut out = std::collections::HashMap::new();
    for &(i, _) in &view.entries {
        let r = &s.records[i];
        let blame = v.blame(&r.doc_path, Some(cp)).unwrap();
        let mut layer = Layer::Canonical;
        for b in &r.blocks {
            let l = blame
                .iter()
                .find(|x| x.lineage_id == b.lineage_id)
                .unwrap()
                .layer;
            layer = match (layer, l) {
                (_, Layer::Derived) | (Layer::Derived, _) => Layer::Derived,
                (_, Layer::Proposal) | (Layer::Proposal, _) => Layer::Proposal,
                _ => Layer::Canonical,
            };
        }
        out.insert((r.doc_path.clone(), r.chunk_id.clone()), layer);
    }
    out
}

/// Deterministic pseudo-random queries over the vault's vocabulary.
fn queries(v: &Vault, s: &Store, cp: &str, n: usize, seed: u64) -> Vec<String> {
    let view = View::new(v, s, cp).unwrap();
    let words: Vec<String> = view
        .entries
        .iter()
        .flat_map(|&(i, _)| rootmarkindex::tokenize::tokens(&s.records[i].text))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut x = seed;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 33) as usize
    };
    (0..n)
        .map(|_| {
            let len = 1 + next() % 5;
            (0..len)
                .map(|_| words[next() % words.len()].clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// GOAL C-7: 1000 queries (×3 modes): by default no `proposal` and no
/// `derived` hit; with `include_proposal` no `derived`, proposals labelled,
/// after every canonical hit, scored ×0.5 of their unweighted score.
#[test]
fn c7_property_1000_queries() {
    let (root, cp) = governed_vault("c7");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let truth = blame_layers(&v, &s, &cp);
    let counts = |l: Layer| truth.values().filter(|x| **x == l).count();
    assert!(
        counts(Layer::Proposal) >= 4 && counts(Layer::Derived) >= 1,
        "{truth:?}"
    );
    let qs = queries(&v, &s, &cp, 1000, 20261015);
    let mut searcher = Searcher::new(&s, embedder(), "c7-property").unwrap();
    let view = View::new(&v, &s, &cp).unwrap();
    let mut seen_proposal = 0usize;
    for (n, q) in qs.iter().enumerate() {
        let mode = MODES[n % 3];
        let off = searcher
            .search(&mut v, mode, q, 10, Some("gov"), false)
            .unwrap();
        for h in &off.hits {
            let want = truth[&(h.doc_path.clone(), h.chunk_id.clone())];
            assert_eq!(h.layer, want, "{q}");
            assert_eq!(
                h.layer,
                Layer::Canonical,
                "default returned {:?} for {q:?}",
                h.layer
            );
        }
        let on = searcher
            .search(&mut v, mode, q, 10, Some("gov"), true)
            .unwrap();
        let mut in_proposals = false;
        for h in &on.hits {
            assert_eq!(
                h.layer,
                truth[&(h.doc_path.clone(), h.chunk_id.clone())],
                "{q}"
            );
            assert_ne!(h.layer, Layer::Derived, "derived returned for {q:?}");
            if h.layer == Layer::Proposal {
                in_proposals = true;
                seen_proposal += 1;
            } else {
                assert!(!in_proposals, "canonical after a proposal for {q:?}");
            }
        }
        // ×0.5: recompute the unweighted score of the same pool
        if mode != Mode::Hybrid || n % 2 == 0 {
            let raw = search::ranked(&s, embedder(), &view, mode, q, true).unwrap();
            for h in on.hits.iter().filter(|h| h.layer == Layer::Proposal) {
                let r = raw
                    .iter()
                    .find(|x| {
                        s.records[x.0].doc_path == h.doc_path
                            && s.records[x.0].chunk_id == h.chunk_id
                    })
                    .unwrap();
                assert_eq!(h.score, r.2 * PROPOSAL_WEIGHT);
            }
        }
    }
    eprintln!("c7: {} queries x 2 settings, {seen_proposal} proposal hits returned with include_proposal; layers {:?}", qs.len(), [counts(Layer::Canonical), counts(Layer::Proposal), counts(Layer::Derived)]);
    if let Ok(p) = std::env::var("ROOTMARK_C7_OUT") {
        let out = serde_json::json!({
            "queries": qs.len(),
            "modes": ["flat", "lexical", "hybrid"],
            "settings": ["default", "include_proposal"],
            "chunks_by_layer": {
                "canonical": counts(Layer::Canonical),
                "proposal": counts(Layer::Proposal),
                "derived": counts(Layer::Derived),
            },
            "proposal_hits_with_include_proposal": seen_proposal,
            "proposal_or_derived_hits_by_default": 0,
            "derived_hits": 0,
            "pass": seen_proposal > 0,
        });
        fs::write(p, serde_json::to_string_pretty(&out).unwrap() + "\n").unwrap();
    }
    assert!(seen_proposal > 0, "no query reached a proposal");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn later_ingest_does_not_change_results_at_an_older_checkpoint() {
    let (root, cp) = governed_vault("at");
    let mut v = Vault::open(&root).unwrap();
    let qs = queries(&v, &Store::open(&v, embedder()).unwrap(), &cp, 30, 7);
    let before: Vec<_> = {
        let s = Store::open(&v, embedder()).unwrap();
        let mut se = Searcher::new(&s, embedder(), "c4-at").unwrap();
        qs.iter()
            .flat_map(|q| MODES.map(|m| se.search(&mut v, m, q, 10, Some(&cp), true).unwrap().hits))
            .collect()
    };
    // later: edits, a new document, a promotion; new checkpoint; rebuild
    let p = here("../spec/fixtures/mini_vault/espresso.md");
    let text =
        fs::read_to_string(&p).unwrap() + "\nA new paragraph about espresso grind and water.\n";
    v.ingest(
        "espresso.md",
        &text,
        Provenance::default(),
        MatcherVersion::V3,
    )
    .unwrap();
    v.ingest(
        "new.md",
        "# New\n\nEspresso grind roast water crack.\n",
        Provenance::default(),
        MatcherVersion::V3,
    )
    .unwrap();
    let lin = v.snapshot("agent/roast-ideas.md").unwrap().unwrap().blocks[1]
        .0
        .clone();
    let req = v
        .request(
            TransitionKind::Promote,
            &lin,
            Layer::Canonical,
            Actor::new(ActorKind::Human, "editor").unwrap(),
            "reviewed",
        )
        .unwrap();
    v.promote(&req, &HumanOnly).unwrap();
    v.checkpoint(Some("later")).unwrap();
    store::build(&v, embedder()).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let mut se = Searcher::new(&s, embedder(), "c4-at").unwrap();
    let after: Vec<_> = qs
        .iter()
        .flat_map(|q| MODES.map(|m| se.search(&mut v, m, q, 10, Some(&cp), true).unwrap().hits))
        .collect();
    assert_eq!(after, before);
    // and the later checkpoint does see the change (sanity)
    let now = se
        .search(&mut v, Mode::Lexical, "crack", 20, Some("later"), false)
        .unwrap();
    assert!(now.hits.iter().any(|h| h.doc_path == "new.md"));
    fs::remove_dir_all(root).unwrap();
}

fn h32(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex()[..32].to_string()
}

#[test]
fn every_search_writes_one_trace_with_the_k_b1_2_id() {
    let (root, cp) = governed_vault("trace");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let mut se = Searcher::new(&s, embedder(), "claude-code").unwrap();
    let before = v.traces(0).unwrap().len();
    let qs = [
        "espresso grind",
        "roast development",
        "zzzz-no-such-term",
        "cleaning",
        "water",
    ];
    let mut results = Vec::new();
    for (n, q) in qs.iter().enumerate() {
        let r = se
            .search(&mut v, MODES[n % 3], q, 5, Some("gov"), n % 2 == 0)
            .unwrap();
        results.push((q, r));
    }
    let traces = v.traces(0).unwrap();
    assert_eq!(traces.len(), before + qs.len(), "one line per search");
    for (i, (q, r)) in results.iter().enumerate() {
        let t = &traces[before + i];
        let at_seq = (before + i) as u64;
        assert_eq!(t.at_seq, at_seq);
        assert_eq!(t.query_hash, h32(q));
        assert_eq!(r.query_hash, h32(q));
        let want = h32(&format!("{cp}\nclaude-code\n{}\n{at_seq}", h32(q)));
        assert_eq!(t.trace_id, want);
        assert_eq!(r.trace_id, want);
        assert_eq!(t.checkpoint_id, cp);
        // returned = every block of every hit, in hit order
        let mut want_ret = Vec::new();
        for h in &r.hits {
            let rec = s
                .records
                .iter()
                .find(|x| {
                    x.doc_path == h.doc_path
                        && x.chunk_id == h.chunk_id
                        && View::new(&v, &s, &cp)
                            .unwrap()
                            .entries
                            .iter()
                            .any(|e| s.records[e.0].record_id == x.record_id)
                })
                .unwrap();
            for b in &rec.blocks {
                want_ret.push((
                    h.doc_path.clone(),
                    b.lineage_id.clone(),
                    b.version_id.clone(),
                ));
            }
        }
        let got: Vec<_> = t
            .returned
            .iter()
            .map(|e| {
                (
                    e.doc_path.clone(),
                    e.lineage_id.clone(),
                    e.version_id.clone(),
                )
            })
            .collect();
        assert_eq!(got, want_ret, "{q}");
    }
    // a search without --at uses (and creates once) the head checkpoint
    let a = se
        .search(&mut v, Mode::Hybrid, "espresso", 3, None, false)
        .unwrap();
    let b = se
        .search(&mut v, Mode::Hybrid, "espresso", 3, None, false)
        .unwrap();
    assert_eq!(a.checkpoint_id, b.checkpoint_id);
    assert_eq!(
        a.checkpoint_id, cp,
        "the current state is the one checkpointed as \"gov\""
    );
    fs::remove_dir_all(root).unwrap();
}

/// Free functions and `Searcher` share one ranking (C4 review note 1).
#[test]
fn searcher_equals_the_free_functions() {
    let (root, cp) = governed_vault("same");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, &cp).unwrap();
    let mut se = Searcher::new(&s, embedder(), "c4-same").unwrap();
    for q in queries(&v, &s, &cp, 20, 99) {
        for p in [false, true] {
            assert_eq!(
                se.search(&mut v, Mode::Flat, &q, 7, Some("gov"), p)
                    .unwrap()
                    .hits,
                search::flat(&s, embedder(), &view, &q, 7, p).unwrap()
            );
            assert_eq!(
                se.search(&mut v, Mode::Lexical, &q, 7, Some("gov"), p)
                    .unwrap()
                    .hits,
                search::lexical(&s, &view, &q, 7, p).unwrap()
            );
            assert_eq!(
                se.search(&mut v, Mode::Hybrid, &q, 7, Some("gov"), p)
                    .unwrap()
                    .hits,
                search::hybrid(&s, embedder(), &view, &q, 7, p).unwrap()
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
}

/// A promotion alone (no new snapshot, no rebuild) is visible at the next
/// checkpoint and invisible at the earlier one (C4-1; review note 4).
#[test]
fn promotion_without_rebuild_changes_the_layer_at_the_new_checkpoint() {
    let (root, _) = governed_vault("promote");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let mut se = Searcher::new(&s, embedder(), "c4-promote").unwrap();
    let q = "development time first crack sweetens light roasts";
    let doc = "agent/roast-ideas.md";
    let before = se
        .search(&mut v, Mode::Lexical, q, 40, Some("gov"), false)
        .unwrap();
    assert!(before.hits.iter().all(|h| h.doc_path != doc));
    let snap = v.snapshot(doc).unwrap().unwrap();
    for (lin, _) in &snap.blocks {
        let req = v
            .request(
                TransitionKind::Promote,
                lin,
                Layer::Canonical,
                Actor::new(ActorKind::Human, "editor").unwrap(),
                "reviewed",
            )
            .unwrap();
        v.promote(&req, &HumanOnly).unwrap();
    }
    v.checkpoint(Some("promoted")).unwrap();
    let after = se
        .search(&mut v, Mode::Lexical, q, 40, Some("promoted"), false)
        .unwrap();
    assert!(after
        .hits
        .iter()
        .any(|h| h.doc_path == doc && h.layer == Layer::Canonical));
    let again = se
        .search(&mut v, Mode::Lexical, q, 40, Some("gov"), false)
        .unwrap();
    assert_eq!(again.hits, before.hits);
    fs::remove_dir_all(root).unwrap();
}
