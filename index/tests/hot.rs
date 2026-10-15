//! P1-C/C5 (TASKS C5; GOAL C-8, C-9): the hot tier — `watch` makes a write
//! searchable without a rebuild, embeds only the chunks it changed, persists
//! across reopen; `compact` moves hot to cold without changing any ranking;
//! write→searchable latency over 200 documents with seeded edits.
//! Needs the model files (`make models`).

use mdkcore::{MatcherVersion, Provenance, Vault};
use mdkindex::embed::{Embedder, OnnxMiniLm};
use mdkindex::search::{Mode, Searcher, View};
use mdkindex::store::{self, Store};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

fn embedder() -> &'static OnnxMiniLm {
    static E: OnceLock<OnnxMiniLm> = OnceLock::new();
    E.get_or_init(|| OnnxMiniLm::load_default().expect("model (run `make models`)"))
}

fn tmp(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("mdk-c5-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// Seeded generator (LCG); the same seed gives the same vault and edits.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize
    }
}

const WORDS: [&str; 48] = [
    "espresso",
    "grind",
    "roast",
    "water",
    "temperature",
    "brew",
    "ratio",
    "filter",
    "crema",
    "bean",
    "origin",
    "acidity",
    "body",
    "sweetness",
    "bitter",
    "extraction",
    "pressure",
    "milk",
    "foam",
    "latte",
    "burr",
    "grinder",
    "kettle",
    "scale",
    "timer",
    "dose",
    "yield",
    "shot",
    "channel",
    "tamp",
    "basket",
    "portafilter",
    "cupping",
    "aroma",
    "finish",
    "light",
    "medium",
    "dark",
    "crack",
    "cooling",
    "storage",
    "fresh",
    "stale",
    "valve",
    "bag",
    "cleaning",
    "descale",
    "boiler",
];

fn sentence(rng: &mut Rng) -> String {
    let n = 8 + rng.next() % 10;
    let mut s: Vec<&str> = (0..n).map(|_| WORDS[rng.next() % WORDS.len()]).collect();
    s[0] = "The";
    format!("{}.", s.join(" "))
}

fn document(rng: &mut Rng, i: usize) -> String {
    let mut out = format!("# Document {i}\n");
    for sec in 0..3 {
        out.push_str(&format!("\n## Section {sec}\n"));
        for _ in 0..2 {
            out.push_str(&format!("\n{} {}\n", sentence(rng), sentence(rng)));
        }
    }
    out
}

/// `n` generated documents (human), checkpoint "base", cold index.
fn vault(name: &str, n: usize, seed: u64) -> (PathBuf, Vec<String>) {
    let root = tmp(name);
    let mut rng = Rng(seed);
    let mut v = Vault::open(&root).unwrap();
    let mut docs = Vec::new();
    for i in 0..n {
        let text = document(&mut rng, i);
        v.ingest(
            &format!("d{i:03}.md"),
            &text,
            Provenance::default(),
            MatcherVersion::V3,
        )
        .unwrap();
        docs.push(text);
    }
    v.checkpoint(Some("base")).unwrap();
    store::build(&v, embedder()).unwrap();
    (root, docs)
}

/// One seeded edit: one paragraph of one document replaced; a unique marker
/// word makes the new text findable.
fn edit(rng: &mut Rng, docs: &mut [String], marker: &str) -> (usize, String) {
    let d = rng.next() % docs.len();
    let paras: Vec<&str> = docs[d].split("\n\n").collect();
    let candidates: Vec<usize> = (0..paras.len())
        .filter(|&i| !paras[i].starts_with('#'))
        .collect();
    let p = candidates[rng.next() % candidates.len()];
    let mut new: Vec<String> = paras.iter().map(|s| s.to_string()).collect();
    new[p] = format!("{} {marker} {}", sentence(rng), sentence(rng));
    let mut text = new.join("\n\n");
    if !text.ends_with('\n') {
        text.push('\n');
    }
    docs[d] = text.clone();
    (d, text)
}

#[test]
fn watch_makes_a_write_searchable_and_embeds_only_changed_chunks() {
    let (root, mut docs) = vault("watch", 20, 11);
    let mut v = Vault::open(&root).unwrap();
    let e = OnnxMiniLm::load_default().unwrap();
    let mut s = Store::open(&v, &e).unwrap();
    let mut rng = Rng(5);
    let (d, text) = edit(&mut rng, &mut docs, "zebracoffee");
    let doc = format!("d{d:03}.md");
    let before = e.embedded_count();
    let r = s
        .watch(&mut v, &e, &doc, &text, Provenance::default())
        .unwrap();
    // one paragraph changed: exactly its section's chunk is new
    assert_eq!(r.embedded, 1, "{r:?}");
    assert_eq!(e.embedded_count() - before, 1);
    assert_eq!(r.chunks, 4, "heading chunk + 3 sections");
    assert_eq!(s.hot_len(), 1);
    // searchable at the head without a rebuild, hot and cold merged
    let mut se = Searcher::new(&s, &e, "c5").unwrap();
    for mode in [Mode::Lexical, Mode::Hybrid] {
        let res = se
            .search(&mut v, mode, "zebracoffee", 5, None, false)
            .unwrap();
        assert_eq!(res.hits[0].doc_path, doc, "{mode:?}");
    }
    let cold = se
        .search(&mut v, Mode::Lexical, "espresso grind", 50, None, false)
        .unwrap();
    assert!(cold.hits.iter().any(|h| h.doc_path != doc));
    // the old checkpoint does not see it
    let old = se
        .search(&mut v, Mode::Lexical, "zebracoffee", 5, Some("base"), false)
        .unwrap();
    assert!(old.hits.is_empty());
    drop(se);
    // the same write again: nothing new
    let r2 = s
        .watch(&mut v, &e, &doc, &text, Provenance::default())
        .unwrap();
    assert_eq!((r2.embedded, r2.events), (0, 0));
    // the hot tier persists: a reopened store sees it
    let s2 = Store::open(&v, &e).unwrap();
    assert_eq!(s2.hot_len(), 1);
    let st = s2.status(&v).unwrap();
    assert_eq!((st.hot_records, st.hot_snapshots), (1, 1));
    assert!(st.stale_docs.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hot_tier_problems_are_errors() {
    let (root, mut docs) = vault("hotbad", 5, 3);
    let mut v = Vault::open(&root).unwrap();
    let mut s = Store::open(&v, embedder()).unwrap();
    let mut rng = Rng(9);
    let (d, text) = edit(&mut rng, &mut docs, "marker");
    s.watch(
        &mut v,
        embedder(),
        &format!("d{d:03}.md"),
        &text,
        Provenance::default(),
    )
    .unwrap();
    let hot = store::index_dir(&v).join("hot");
    // a truncated vector file
    let p = hot.join("vectors.f32");
    let bytes = fs::read(&p).unwrap();
    fs::write(&p, &bytes[..bytes.len() - 4]).unwrap();
    assert!(Store::open(&v, embedder()).is_err());
    fs::write(&p, &bytes).unwrap();
    assert!(Store::open(&v, embedder()).is_ok());
    // a hot tier of another cold index
    let b = hot.join("base.json");
    let base = fs::read_to_string(&b).unwrap();
    fs::write(&b, base.replace("\": \"", "\": \"0")).unwrap();
    let e = Store::open(&v, embedder()).err().expect("foreign hot tier");
    assert!(format!("{e:#}").contains("another cold index"), "{e:#}");
    fs::remove_dir_all(root).unwrap();
}

fn rankings(v: &mut Vault, s: &Store, qs: &[String], at: &str) -> Vec<Vec<mdkindex::search::Hit>> {
    let mut se = Searcher::new(s, embedder(), "c9").unwrap();
    qs.iter()
        .enumerate()
        .map(|(i, q)| {
            let mode = [Mode::Flat, Mode::Lexical, Mode::Hybrid][i % 3];
            se.search(v, mode, q, 10, Some(at), i % 2 == 0)
                .unwrap()
                .hits
        })
        .collect()
}

/// GOAL C-9: 200 queries before and after `compact()` → identical rankings,
/// at the checkpoint that names hot snapshots and at the older one.
#[test]
fn c9_compact_keeps_200_rankings() {
    let (root, mut docs) = vault("c9", 30, 21);
    let mut v = Vault::open(&root).unwrap();
    let mut s = Store::open(&v, embedder()).unwrap();
    let mut rng = Rng(77);
    for n in 0..15 {
        let (d, text) = edit(&mut rng, &mut docs, &format!("marker{n}"));
        s.watch(
            &mut v,
            embedder(),
            &format!("d{d:03}.md"),
            &text,
            Provenance::default(),
        )
        .unwrap();
    }
    v.checkpoint(Some("hot")).unwrap();
    assert!(s.hot_len() >= 10, "{}", s.hot_len());
    let qs: Vec<String> = (0..200)
        .map(|i| {
            let n = 1 + rng.next() % 4;
            let mut q: Vec<String> = (0..n)
                .map(|_| WORDS[rng.next() % WORDS.len()].to_string())
                .collect();
            if i % 5 == 0 {
                q.push(format!("marker{}", rng.next() % 15));
            }
            q.join(" ")
        })
        .collect();
    let hot_before = rankings(&mut v, &s, &qs, "hot");
    let base_before = rankings(&mut v, &s, &qs, "base");
    let cold_before = s.cold_len;
    drop(s);
    let rep = store::compact(&v, embedder()).unwrap();
    assert!(rep.moved_from_hot >= 10);
    let s = Store::open(&v, embedder()).unwrap();
    assert_eq!(s.hot_len(), 0);
    assert_eq!(s.cold_len, cold_before + rep.moved_from_hot);
    assert!(!store::index_dir(&v).join("hot").exists());
    assert_eq!(rankings(&mut v, &s, &qs, "hot"), hot_before);
    assert_eq!(rankings(&mut v, &s, &qs, "base"), base_before);
    // a compacted index ranks as one rebuilt from the vault (GOAL C-5 view)
    store::build(&v, embedder()).unwrap();
    let s = Store::open(&v, embedder()).unwrap();
    assert_eq!(rankings(&mut v, &s, &qs, "hot"), hot_before);
    fs::remove_dir_all(root).unwrap();
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    // nearest rank
    let i = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[i.clamp(1, sorted.len()) - 1]
}

/// GOAL C-8: write → searchable in the hot tier, 200 documents, 200 seeded
/// edits; the model is loaded before timing. Each sample is one `watch`
/// (ingest, chunk, embed the changed chunks, append); after each the edit's
/// marker is found at the head. p50/p95 are written to `MDK_C8_OUT` (the
/// round report judges the threshold; the number depends on the machine).
#[test]
fn c8_write_to_searchable_latency() {
    let (root, mut docs) = vault("c8", 200, 2026);
    let mut v = Vault::open(&root).unwrap();
    let e = embedder();
    e.embed(&["warm up"]).unwrap();
    let mut s = Store::open(&v, e).unwrap();
    let mut rng = Rng(1015);
    let mut ms = Vec::new();
    let mut embedded = Vec::new();
    for n in 0..200 {
        let marker = format!("hotmarker{n}");
        let (d, text) = edit(&mut rng, &mut docs, &marker);
        let doc = format!("d{d:03}.md");
        let t0 = Instant::now();
        let r = s
            .watch(&mut v, e, &doc, &text, Provenance::default())
            .unwrap();
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        embedded.push(r.embedded);
        let mut se = Searcher::new(&s, e, "c8").unwrap();
        let hit = se
            .search(&mut v, Mode::Lexical, &marker, 1, None, false)
            .unwrap();
        assert_eq!(
            hit.hits.first().map(|h| h.doc_path.as_str()),
            Some(doc.as_str())
        );
    }
    let mut sorted = ms.clone();
    sorted.sort_by(f64::total_cmp);
    let (p50, p95) = (percentile(&sorted, 50.0), percentile(&sorted, 95.0));
    let out = serde_json::json!({
        "documents": 200,
        "edits": ms.len(),
        "p50_ms": p50,
        "p95_ms": p95,
        "max_ms": sorted[sorted.len() - 1],
        "threshold_p95_ms": 100.0,
        "pass": p95 < 100.0,
        "embedded_per_edit": {
            "min": embedded.iter().min(),
            "max": embedded.iter().max(),
            "total": embedded.iter().sum::<usize>(),
        },
        "embedder_id": e.id(),
        "threads": 1,
    });
    eprintln!("C-8: {out}");
    if let Ok(p) = std::env::var("MDK_C8_OUT") {
        fs::write(p, serde_json::to_string_pretty(&out).unwrap() + "\n").unwrap();
    }
    assert!(embedded.iter().all(|&n| n <= 1), "one paragraph per edit");
    let _ = View::new(&v, &s, "base").unwrap();
    fs::remove_dir_all(root).unwrap();
}
