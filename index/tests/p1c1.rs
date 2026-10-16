//! P1-C.1/C1.2 (GOAL P1-C.1 C1-2..C1-6; K-C1.1, K-C1.3..K-C1.5): on a vault
//! with proposals (agent documents, an agent edit inside a human document,
//! one proposal demoted to derived):
//!
//! - C1-2: 1000 queries × 3 modes — with `include_proposal` the canonical
//!   prefix is byte-identical to the result without it
//! - C1-3: the same 3000 searches — proposal hits are labelled (layer from
//!   `blame`, independent of the index), after every canonical hit, at most
//!   `k`, and are exactly the top `k` of a proposal ranking recomputed here
//!   independently (canonical-only BM25 statistics, cosine, RRF) with score
//!   × 0.5
//! - C1-4: searching (every mode, both settings), `Store::open` and
//!   `status` leave `.mdkernel/index/` byte-identical, no file added (no
//!   `.tantivy-meta.lock`)
//! - C1-5: empty / whitespace-only queries are `Err` in every entry point;
//!   no trace line, no checkpoint, `events.jsonl` unchanged
//! - C1-6: the manifest holds `lexical.{variant, idf, k1, b, tokenizer, dl}`
//!   (`dl` since P1-D.1, K-D1.7);
//!   removing or changing any of them makes opening the index (`status`) an
//!   error
//!
//! With `MDK_P1C1_OUT=<dir>` each test writes `<dir>/c1_<n>.json`.

use mdkcore::{
    Actor, ActorKind, AuthorKind, HumanOnly, Layer, MatcherVersion, Provenance, TransitionKind,
    Vault,
};
use mdkindex::embed::{Embedder, OnnxMiniLm};
use mdkindex::search::{self, Hit, Mode, Searcher, View, PROPOSAL_WEIGHT};
use mdkindex::store::{self, Store};
use mdkindex::tokenize::tokens;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MODES: [Mode; 3] = [Mode::Flat, Mode::Lexical, Mode::Hybrid];
const K: usize = 10;
const N_QUERIES: usize = 1000;

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
    let d = std::env::temp_dir().join(format!("mdk-c11-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn out(name: &str, v: Value) {
    if let Ok(d) = std::env::var("MDK_P1C1_OUT") {
        let p = Path::new(&d).join(format!("{name}.json"));
        fs::write(&p, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}

fn agent() -> Provenance {
    Provenance {
        author_kind: AuthorKind::Agent,
        source: "claude-code".into(),
    }
}

/// mini_vault (human) + agent documents (proposal) + an agent edit of a
/// human paragraph + one proposal demoted to derived; checkpoint "gov"
/// (the construction of `tests/governance.rs`, P1-C/C4).
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

/// Layer of each `(doc_path, chunk_id)` from `blame` at the checkpoint.
fn blame_layers(v: &Vault, s: &Store, cp: &str) -> HashMap<(String, String), Layer> {
    let view = View::new(v, s, cp).unwrap();
    let mut out = HashMap::new();
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

/// Seeded queries over the vocabulary of every chunk of the checkpoint
/// (proposal and derived words included).
fn queries(v: &Vault, s: &Store, cp: &str, n: usize, seed: u64) -> Vec<String> {
    let view = View::new(v, s, cp).unwrap();
    let words: Vec<String> = view
        .entries
        .iter()
        .flat_map(|&(i, _)| tokens(&s.records[i].text))
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

/// Proposal ranking recomputed here, from the record texts and vectors
/// only: BM25 (Lucene IDF, k1 1.2, b 0.75, token counts) with the statistics
/// of the canonical records, exact cosine, RRF 1/(60 + rank) of the top 50
/// of each side; `(position, unweighted score)`, best first, ties by
/// `(doc_path, chunk_id)`.
struct Independent<'a> {
    s: &'a Store,
    canon: Vec<usize>,
    prop: Vec<usize>,
    toks: HashMap<usize, Vec<String>>,
}

impl<'a> Independent<'a> {
    fn new(s: &'a Store, view: &View) -> Independent<'a> {
        let pick = |l: Layer| -> Vec<usize> {
            view.entries
                .iter()
                .filter(|e| e.1 == l)
                .map(|e| e.0)
                .collect()
        };
        let (canon, prop) = (pick(Layer::Canonical), pick(Layer::Proposal));
        let toks = canon
            .iter()
            .chain(&prop)
            .map(|&i| (i, tokens(&s.records[i].text)))
            .collect();
        Independent {
            s,
            canon,
            prop,
            toks,
        }
    }

    fn order(&self, mut v: Vec<(usize, f64)>) -> Vec<(usize, f64)> {
        v.sort_by(|a, b| {
            b.1.total_cmp(&a.1).then_with(|| {
                let (ra, rb) = (&self.s.records[a.0], &self.s.records[b.0]);
                (&ra.doc_path, &ra.chunk_id).cmp(&(&rb.doc_path, &rb.chunk_id))
            })
        });
        v
    }

    fn bm25(&self, q: &str) -> Vec<(usize, f64)> {
        let n = self.canon.len() as f64;
        let total: usize = self.canon.iter().map(|i| self.toks[i].len()).sum();
        let avgdl = if total == 0 { 1.0 } else { total as f64 / n };
        let qt = tokens(q);
        let mut out = Vec::new();
        for &i in &self.prop {
            let d = &self.toks[&i];
            let dl = d.len() as f64;
            let mut score = 0.0;
            let mut hit = false;
            for t in &qt {
                let tf = d.iter().filter(|x| *x == t).count() as f64;
                if tf == 0.0 {
                    continue;
                }
                let df = self
                    .canon
                    .iter()
                    .filter(|c| self.toks[c].contains(t))
                    .count() as f64;
                let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
                score += idf * tf * 2.2 / (tf + 1.2 * (0.25 + 0.75 * dl / avgdl));
                hit = true;
            }
            if hit {
                out.push((i, score));
            }
        }
        self.order(out)
    }

    fn cosine(&self, qv: &[f32]) -> Vec<(usize, f64)> {
        let v = self
            .prop
            .iter()
            .map(|&i| {
                let c: f64 = qv
                    .iter()
                    .zip(&self.s.vectors[i])
                    .map(|(a, b)| f64::from(*a) * f64::from(*b))
                    .sum();
                (i, c)
            })
            .collect();
        self.order(v)
    }

    fn ranking(&self, mode: Mode, q: &str) -> Vec<(usize, f64)> {
        let qv = || embedder().embed(&[q]).unwrap().pop().unwrap();
        match mode {
            Mode::Lexical => self.bm25(q),
            Mode::Flat => self.cosine(&qv()),
            Mode::Hybrid => {
                let mut fused: BTreeMap<usize, f64> = BTreeMap::new();
                for list in [self.cosine(&qv()), self.bm25(q)] {
                    for (r, (i, _)) in list.into_iter().take(50).enumerate() {
                        *fused.entry(i).or_insert(0.0) += 1.0 / (60.0 + (r + 1) as f64);
                    }
                }
                self.order(fused.into_iter().collect())
            }
        }
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
}

/// C1-2 and C1-3 over the same 1000 queries × 3 modes.
#[test]
fn c12_c13_include_proposal_1000_queries() {
    let (root, cp) = governed_vault("c12");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let truth = blame_layers(&v, &s, &cp);
    let view = View::new(&v, &s, &cp).unwrap();
    let ind = Independent::new(&s, &view);
    let count = |l: Layer| truth.values().filter(|x| **x == l).count();
    assert!(count(Layer::Proposal) >= 4 && count(Layer::Derived) >= 1);
    // the layer sets of the passes (canonical statistics included) are the
    // `blame` layers (C1.2 review note 2)
    for &(i, l) in &view.entries {
        let r = &s.records[i];
        assert_eq!(l, truth[&(r.doc_path.clone(), r.chunk_id.clone())]);
    }
    let qs = queries(&v, &s, &cp, N_QUERIES, 20261015);
    let mut se = Searcher::new(&s, embedder(), "c1-2").unwrap();
    let (mut prefix_equal, mut searches, mut prop_hits) = (0usize, 0usize, 0usize);
    let (mut c13_ok, mut stats_sensitive) = (0usize, 0usize);
    let canon_all = view.layer(Layer::Canonical);
    let eligible_all = view.eligible(true);
    for q in &qs {
        for mode in MODES {
            searches += 1;
            let off = se.search(&mut v, mode, q, K, Some("gov"), false).unwrap();
            let on = se.search(&mut v, mode, q, K, Some("gov"), true).unwrap();
            // C1-2: canonical prefix byte-identical
            let n_canon = on
                .hits
                .iter()
                .take_while(|h| h.layer == Layer::Canonical)
                .count();
            let a = serde_json::to_string(&on.hits[..n_canon]).unwrap();
            let b = serde_json::to_string(&off.hits).unwrap();
            if a == b {
                prefix_equal += 1;
            }
            // C1-3: the rest are proposals, labelled, top k of the
            // independent ranking, score × 0.5
            let rest: &[Hit] = &on.hits[n_canon..];
            let want: Vec<(usize, f64)> = ind.ranking(mode, q).into_iter().take(K).collect();
            let ok = rest.len() == want.len()
                && rest.iter().zip(&want).all(|(h, &(i, raw))| {
                    let r = &s.records[i];
                    h.layer == Layer::Proposal
                        && truth[&(h.doc_path.clone(), h.chunk_id.clone())] == Layer::Proposal
                        && (h.doc_path.as_str(), h.chunk_id.as_str())
                            == (r.doc_path.as_str(), r.chunk_id.as_str())
                        && close(h.score, raw * PROPOSAL_WEIGHT)
                });
            if ok {
                c13_ok += 1;
            } else {
                eprintln!("C1-3 {mode:?} {q:?}: got {rest:?}, want {want:?}");
            }
            prop_hits += rest.len();
            // sensitivity: would P1-C's statistics (canonical + proposal)
            // have changed the canonical lexical ranking of this query?
            if mode == Mode::Lexical {
                let mixed: Vec<(usize, f64)> = search::bm25(&s, &eligible_all, q)
                    .unwrap()
                    .into_iter()
                    .filter(|x| x.1 == Layer::Canonical)
                    .map(|x| (x.0, x.2))
                    .take(K)
                    .collect();
                let mine: Vec<(usize, f64)> = search::bm25(&s, &canon_all, q)
                    .unwrap()
                    .into_iter()
                    .map(|x| (x.0, x.2))
                    .take(K)
                    .collect();
                if mixed != mine {
                    stats_sensitive += 1;
                }
            }
        }
    }
    eprintln!(
        "c1-2: {prefix_equal}/{searches} canonical prefixes equal; c1-3: {c13_ok}/{searches}, {prop_hits} proposal hits; {stats_sensitive} lexical queries whose canonical ranking P1-C's mixed statistics would change"
    );
    out(
        "c1_2_3",
        json!({
            "queries": N_QUERIES,
            "modes": ["flat", "lexical", "hybrid"],
            "k": K,
            "searches": searches,
            "chunks_by_layer": {
                "canonical": count(Layer::Canonical),
                "proposal": count(Layer::Proposal),
                "derived": count(Layer::Derived),
            },
            "C1-2": {"canonical_prefix_equal": prefix_equal, "pass": prefix_equal == searches},
            "C1-3": {
                "proposal_hits": prop_hits,
                "searches_matching_independent_ranking": c13_ok,
                "pass": c13_ok == searches && prop_hits > 0,
            },
            "lexical_queries_sensitive_to_mixed_statistics": stats_sensitive,
        }),
    );
    assert_eq!(prefix_equal, searches, "C1-2");
    assert_eq!(c13_ok, searches, "C1-3");
    assert!(prop_hits > 0 && stats_sensitive > 0, "vacuous property");
    fs::remove_dir_all(root).unwrap();
}

/// Every file under `dir` (hidden ones included) → bytes.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().to_string();
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// C1-4: a query never writes into the index directory.
#[test]
fn c14_search_leaves_the_index_directory_untouched() {
    let (root, _) = governed_vault("c14");
    let mut v = Vault::open(&root).unwrap();
    let dir = store::index_dir(&v);
    let before = tree(&dir);
    assert!(before.keys().all(|f| !f.ends_with(".lock")), "{before:?}");
    let mut searches = 0;
    for round in 0..2 {
        let s = Store::open(&v, embedder()).unwrap();
        s.status(&v).unwrap();
        let mut se = Searcher::new(&s, embedder(), "c1-4").unwrap();
        for mode in MODES {
            for p in [false, true] {
                for at in [Some("gov"), None] {
                    se.search(&mut v, mode, "espresso grind water", K, at, p)
                        .unwrap();
                    searches += 1;
                }
            }
        }
        assert_eq!(tree(&dir), before, "round {round}");
    }
    let after = tree(&dir);
    out(
        "c1_4",
        json!({
            "searches": searches,
            "files": before.len(),
            "lock_files": after.keys().filter(|f| f.ends_with(".lock")).count(),
            "pass": after == before,
        }),
    );
    fs::remove_dir_all(root).unwrap();
}

/// C1-5: empty and whitespace-only queries are rejected; nothing is written.
#[test]
fn c15_empty_queries_are_rejected_without_a_trace() {
    let (root, _) = governed_vault("c15");
    let mut v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, "gov").unwrap();
    let store_dir = v.store_dir().to_path_buf();
    let snap = |d: &Path| {
        (
            fs::read(d.join("events.jsonl")).unwrap(),
            fs::read(d.join("traces.jsonl")).unwrap_or_default(),
            fs::read_dir(d.join("checkpoints")).unwrap().count(),
        )
    };
    let before = snap(&store_dir);
    let mut se = Searcher::new(&s, embedder(), "c1-5").unwrap();
    let mut rejected = 0;
    let mut tried = 0;
    for q in ["", " ", "\t\n  ", "\u{3000}\u{a0}\u{2003}"] {
        for mode in MODES {
            for p in [false, true] {
                for at in [Some("gov"), None] {
                    tried += 1;
                    match se.search(&mut v, mode, q, K, at, p) {
                        Err(e) if format!("{e:#}").contains("empty query") => rejected += 1,
                        other => panic!("{q:?} {mode:?}: {other:?}"),
                    }
                }
            }
            for r in [
                search::lexical(&s, &view, q, K, true).err(),
                search::flat(&s, embedder(), &view, q, K, true).err(),
                search::hybrid(&s, embedder(), &view, q, K, true).err(),
                search::ranked(&s, embedder(), &view, mode, q, false).err(),
            ] {
                tried += 1;
                assert!(r.is_some(), "{q:?}");
                rejected += 1;
            }
        }
    }
    let after = snap(&store_dir);
    // a non-empty query without tokens is still a query (flat ranks it)
    let r = se
        .search(&mut v, Mode::Flat, "?!", K, Some("gov"), false)
        .unwrap();
    assert!(!r.hits.is_empty());
    out(
        "c1_5",
        json!({
            "tried": tried,
            "rejected": rejected,
            "events_unchanged": before.0 == after.0,
            "traces_unchanged": before.1 == after.1,
            "checkpoints_unchanged": before.2 == after.2,
            "pass": rejected == tried && before == after,
        }),
    );
    assert_eq!(rejected, tried);
    assert_eq!(before, after, "an empty query wrote into the vault");
    fs::remove_dir_all(root).unwrap();
}

/// C1-6: the lexical record is in the manifest; a missing or different
/// field makes the index unopenable.
#[test]
fn c16_manifest_lexical_record() {
    let (root, _) = governed_vault("c16");
    let v = Vault::open(&root).unwrap();
    let p = store::index_dir(&v).join("manifest.json");
    let orig = fs::read_to_string(&p).unwrap();
    let m: Value = serde_json::from_str(&orig).unwrap();
    // P1-D.1 (K-D1.7, freeze exception P1-D.1/F1): format `/3`, `lexical.dl`
    assert_eq!(m["format"], "mdkernel-index/3");
    assert_eq!(
        m["lexical"],
        json!({
            "variant": "lucene-bm25",
            "idf": "ln(1 + (N - n + 0.5) / (n + 0.5))",
            "k1": 1.2,
            "b": 0.75,
            "tokenizer": "mdk_lower_alnum",
            "dl": "exact token count, f64, no fieldnorm quantization",
        })
    );
    Store::open(&v, embedder()).unwrap().status(&v).unwrap();
    let mut cases: Vec<(String, Value)> = Vec::new();
    for f in ["variant", "idf", "k1", "b", "tokenizer", "dl"] {
        let mut x = m.clone();
        x["lexical"].as_object_mut().unwrap().remove(f);
        cases.push((format!("remove lexical.{f}"), x));
    }
    let mut x = m.clone();
    x.as_object_mut().unwrap().remove("lexical");
    cases.push(("remove lexical".into(), x));
    for (f, val) in [
        ("variant", json!("okapi")),
        ("idf", json!("ln((N - n + 0.5) / (n + 0.5))")),
        ("k1", json!(1.5)),
        ("b", json!(0.7)),
        ("tokenizer", json!("whitespace")),
        ("dl", json!("fieldnorm, 1 byte")),
    ] {
        let mut x = m.clone();
        x["lexical"][f] = val;
        cases.push((format!("change lexical.{f}"), x));
    }
    let mut x = m.clone();
    x["lexical"]["extra"] = json!(1);
    cases.push(("unknown lexical.extra".into(), x));
    let mut results = Vec::new();
    for (name, x) in &cases {
        fs::write(&p, serde_json::to_string_pretty(x).unwrap() + "\n").unwrap();
        let e = Store::open(&v, embedder())
            .err()
            .unwrap_or_else(|| panic!("{name}: index opened"));
        results.push(json!({"case": name, "error": format!("{e:#}")}));
    }
    fs::write(&p, &orig).unwrap();
    Store::open(&v, embedder()).unwrap();
    out(
        "c1_6",
        json!({"lexical": m["lexical"], "rejected": results, "pass": results.len() == cases.len()}),
    );
    fs::remove_dir_all(root).unwrap();
}
