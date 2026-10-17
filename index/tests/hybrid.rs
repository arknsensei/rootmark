//! P1-C/C3 (TASKS C3; GOAL C-4): vector store and hybrid retrieval — the
//! three modes give identical rankings over 100 repeats, RRF by hand,
//! exact cosine, ties by `(doc_path, chunk_id)`, `embedder_id` mixing is an
//! error. Needs the model files (`make models`).

use anyhow::Result;
use rootmarkcore::{Layer, MatcherVersion, Vault};
use rootmarkindex::embed::{Embedder, OnnxMiniLm};
use rootmarkindex::search::{self, Hit, Scored, View};
use rootmarkindex::store::{self, Store};
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
    let d = std::env::temp_dir().join(format!("rootmark-c3-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn mini_vault(name: &str) -> PathBuf {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&here("../spec/fixtures/mini_vault"), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t0")).unwrap();
    store::build(&v, embedder()).unwrap();
    root
}

type Mode = fn(&Store, &dyn Embedder, &View, &str, usize, bool) -> Result<Vec<Hit>>;

fn lexical_mode(
    s: &Store,
    _: &dyn Embedder,
    v: &View,
    q: &str,
    k: usize,
    p: bool,
) -> Result<Vec<Hit>> {
    search::lexical(s, v, q, k, p)
}

const MODES: [(&str, Mode); 3] = [
    ("flat", search::flat),
    ("lexical", lexical_mode),
    ("hybrid", search::hybrid),
];

const QUERIES: [&str; 5] = [
    "how fine should the espresso grind be",
    "roast profile first crack",
    "cleaning the grinder burrs",
    "water temperature for filter coffee",
    "who is on staff on weekends",
];

#[test]
fn c4_three_modes_identical_over_100_repeats() {
    let root = mini_vault("c4");
    let v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    for (name, mode) in MODES {
        for q in QUERIES {
            let first = mode(&s, embedder(), &view, q, 10, false).unwrap();
            assert!(!first.is_empty(), "{name}: {q}");
            for _ in 0..100 {
                assert_eq!(
                    mode(&s, embedder(), &view, q, 10, false).unwrap(),
                    first,
                    "{name}: {q}"
                );
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn flat_is_exact_cosine_over_every_record() {
    let root = mini_vault("flat");
    let v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    let q = QUERIES[0];
    let qv = embedder().embed(&[q]).unwrap().remove(0);
    let mut want: Vec<(f64, String, String)> = view
        .entries
        .iter()
        .map(|&(i, _)| {
            let r = &s.records[i];
            let e = embedder().embed(&[&r.text]).unwrap().remove(0);
            let dot: f64 = qv
                .iter()
                .zip(&e)
                .map(|(a, b)| f64::from(*a) * f64::from(*b))
                .sum();
            (dot, r.doc_path.clone(), r.chunk_id.clone())
        })
        .collect();
    want.sort_by(|a, b| b.0.total_cmp(&a.0).then((&a.1, &a.2).cmp(&(&b.1, &b.2))));
    let n = view.entries.len();
    let got = search::flat(&s, embedder(), &view, q, n, false).unwrap();
    assert_eq!(got.len(), n);
    for (g, w) in got.iter().zip(&want) {
        assert_eq!((&g.doc_path, &g.chunk_id), (&w.1, &w.2));
        assert_eq!(g.score, w.0);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rrf_by_hand() {
    let root = mini_vault("rrf");
    let v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    // records 0..4 in storage order; two lists:
    // dense  [2, 0, 1], sparse [0, 3]
    let c = Layer::Canonical;
    let dense: Vec<Scored> = vec![(2, c, 0.9), (0, c, 0.8), (1, c, 0.7)];
    let sparse: Vec<Scored> = vec![(0, c, 5.0), (3, c, 4.0)];
    let fused = search::rrf(&s, &[&dense, &sparse]);
    let r = |rank: f64| 1.0 / (60.0 + rank);
    // 0: 1/62 + 1/61; 2: 1/61; 3: 1/62; 1: 1/63
    let want = vec![(0, r(2.0) + r(1.0)), (2, r(1.0)), (3, r(2.0)), (1, r(3.0))];
    let got: Vec<(usize, f64)> = fused.iter().map(|x| (x.0, x.2)).collect();
    assert_eq!(got, want);
    // a tie (records 5 and 4 both first in one list) is broken by key
    let a: Vec<Scored> = vec![(5, c, 1.0)];
    let b: Vec<Scored> = vec![(4, c, 1.0)];
    let fused = search::rrf(&s, &[&a, &b]);
    let key = |i: usize| (s.records[i].doc_path.clone(), s.records[i].chunk_id.clone());
    assert_eq!(fused[0].2, fused[1].2);
    assert!(key(fused[0].0) < key(fused[1].0));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hybrid_fuses_the_top_50_of_each_side() {
    let root = mini_vault("hyb");
    let v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    let el = view.eligible(false);
    for q in QUERIES {
        let qv = embedder().embed(&[q]).unwrap().remove(0);
        let mut dense = search::cosine_ranking(&s, &el, &qv);
        dense.truncate(50);
        let mut sparse = search::bm25(&s, &el, q).unwrap();
        sparse.truncate(50);
        let want = search::finalize(&s, search::rrf(&s, &[&dense, &sparse]), 10);
        assert_eq!(
            search::hybrid(&s, embedder(), &view, q, 10, false).unwrap(),
            want
        );
    }
    fs::remove_dir_all(root).unwrap();
}

/// A stand-in embedder with another id (same vectors): mixing must fail.
struct Other<'a>(&'a OnnxMiniLm);

impl Embedder for Other<'_> {
    fn id(&self) -> &str {
        "other/00000000"
    }
    fn dim(&self) -> usize {
        self.0.dim()
    }
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.0.embed(texts)
    }
    fn embedded_count(&self) -> u64 {
        self.0.embedded_count()
    }
}

fn set_embedders_hash(v: &Vault, bytes: &[u8]) {
    let mp = store::index_dir(v).join("manifest.json");
    let mut m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mp).unwrap()).unwrap();
    m["embedders_blake3"] = blake3::hash(bytes).to_hex().to_string().into();
    fs::write(&mp, serde_json::to_string_pretty(&m).unwrap() + "\n").unwrap();
}

fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    fn walk(base: &Path, d: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for e in fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().to_string();
                if !rel.ends_with(".tantivy-meta.lock") {
                    out.push((rel, fs::read(&p).unwrap()));
                }
            }
        }
    }
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// C3 review fix: a damaged previous index lends no vector; the rebuild is
/// byte-identical to a clean build and reports why nothing was reused.
#[test]
fn damaged_previous_index_lends_no_vectors() {
    let root = mini_vault("reuse");
    let v = Vault::open(&root).unwrap();
    let clean = tree(&store::index_dir(&v));
    let p = store::index_dir(&v).join("vectors/embedders.jsonl");
    let text = fs::read_to_string(&p).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines.swap(0, 1);
    let swapped = lines.join("\n") + "\n";
    // (a) swapped rows, manifest untouched: hash mismatch
    fs::write(&p, &swapped).unwrap();
    assert!(Store::open(&v, embedder()).is_err());
    let r = store::build(&v, embedder()).unwrap();
    assert_eq!(r.embedded, r.records);
    assert!(
        r.reuse.contains("previous index invalid") && r.reuse.contains("blake3"),
        "{}",
        r.reuse
    );
    assert_eq!(tree(&store::index_dir(&v)), clean);
    // (b) swapped rows with a matching manifest hash: rows no longer line up
    fs::write(&p, &swapped).unwrap();
    set_embedders_hash(&v, swapped.as_bytes());
    assert!(Store::open(&v, embedder()).is_err());
    let r = store::build(&v, embedder()).unwrap();
    assert_eq!(r.embedded, r.records);
    assert!(r.reuse.contains("line up"), "{}", r.reuse);
    assert_eq!(tree(&store::index_dir(&v)), clean);
    // (c) intact: everything reused
    let r = store::build(&v, embedder()).unwrap();
    assert_eq!((r.embedded, r.reuse.as_str()), (0, "previous index"));
    assert_eq!(tree(&store::index_dir(&v)), clean);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn embedder_id_mixing_is_an_error() {
    let root = mini_vault("mix");
    let v = Vault::open(&root).unwrap();
    // querying with another embedder
    let e = Store::open(&v, &Other(embedder()))
        .err()
        .expect("other embedder");
    assert!(format!("{e:#}").contains("embedder"), "{e:#}");
    let s = Store::open(&v, embedder()).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    assert!(search::flat(&s, &Other(embedder()), &view, "x", 5, false).is_err());
    assert!(search::hybrid(&s, &Other(embedder()), &view, "x", 5, false).is_err());
    // a store whose rows mix two embedder ids
    let p = store::index_dir(&v).join("vectors/embedders.jsonl");
    let text = fs::read_to_string(&p).unwrap();
    let first = text.lines().next().unwrap().to_string();
    let mixed = text.replacen(&first, &first.replace(embedder().id(), "other/00000000"), 1);
    fs::write(&p, &mixed).unwrap();
    // keep the manifest hash consistent so that the mixing check itself fires
    set_embedders_hash(&v, mixed.as_bytes());
    let e = Store::open(&v, embedder()).err().expect("mixed rows");
    assert!(format!("{e:#}").contains("mixes embedder ids"), "{e:#}");
    // a rebuild with another embedder embeds everything again (no reuse)
    let r = store::build(&v, &Other(embedder())).unwrap();
    assert_eq!(r.embedded, r.records);
    let r = store::build(&v, &Other(embedder())).unwrap();
    assert_eq!(r.embedded, 0, "same embedder: vectors reused");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn proposal_weight_and_order_by_hand() {
    let root = mini_vault("final");
    let v = Vault::open(&root).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    let (c, p) = (Layer::Canonical, Layer::Proposal);
    let ranked: Vec<Scored> = vec![(0, p, 1.0), (1, c, 0.6), (2, c, 0.4), (3, p, 0.7)];
    // K-C1.3 (P1-C.1; ASSUMPTIONS C1.2-1): the top k canonical hits, then the
    // top k proposal hits with ×0.5 (0 → 0.5, 3 → 0.35)
    let got = |k: usize| -> Vec<(String, Layer, f64)> {
        search::finalize(&s, ranked.clone(), k)
            .iter()
            .map(|h| (h.chunk_id.clone(), h.layer, h.score))
            .collect()
    };
    let id = |i: usize| s.records[i].chunk_id.clone();
    assert_eq!(
        got(3),
        vec![
            (id(1), c, 0.6),
            (id(2), c, 0.4),
            (id(0), p, 0.5),
            (id(3), p, 0.35)
        ]
    );
    assert_eq!(got(1), vec![(id(1), c, 0.6), (id(0), p, 0.5)]);
    fs::remove_dir_all(root).unwrap();
}
