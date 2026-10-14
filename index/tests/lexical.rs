//! P1-C/C2 (TASKS C2): tokenizer ≡ prototype (`research/data/tok_ref.json`),
//! deterministic lexical index (directory bytes equal across two builds in
//! two places), ranking stable over 100 repeats, BM25 checked by hand, and
//! the checkpoint restriction.

use mdkcore::{MatcherVersion, Provenance, Vault};
use mdkindex::search::{self, View};
use mdkindex::store::{self, Store};
use mdkindex::tokenize::{tokens, MdkTokenizer};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tantivy::tokenizer::{TextAnalyzer, TokenStream};

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("mdk-c2-{}-{n}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// spec/fixtures/mini_vault ingested (human), checkpoint "t0".
fn mini_vault(name: &str) -> PathBuf {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&here("../spec/fixtures/mini_vault"), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t0")).unwrap();
    root
}

fn analyzer_tokens(text: &str) -> Vec<String> {
    let mut a = TextAnalyzer::from(MdkTokenizer);
    let mut s = a.token_stream(text);
    let mut out = Vec::new();
    while s.advance() {
        out.push(s.token().text.clone());
    }
    out
}

#[test]
fn tokenizer_equals_the_prototype_on_200_chunks() {
    let r: Value =
        serde_json::from_str(&fs::read_to_string(here("../research/data/tok_ref.json")).unwrap())
            .unwrap();
    let chunks = r["chunks"].as_array().unwrap();
    assert_eq!(chunks.len(), 200);
    let edge = r["edge"].as_array().unwrap();
    for c in chunks.iter().chain(edge) {
        let text = c["text"].as_str().unwrap();
        let want: Vec<String> = c["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();
        assert_eq!(tokens(text), want, "{text:?}");
        assert_eq!(analyzer_tokens(text), want, "analyzer: {text:?}");
    }
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
                out.push((rel, fs::read(&p).unwrap()));
            }
        }
    }
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn index_directory_is_byte_identical_across_builds_and_places() {
    let a = mini_vault("a");
    let b = mini_vault("b");
    let va = Vault::open(&a).unwrap();
    let vb = Vault::open(&b).unwrap();
    let ra = store::build(&va).unwrap();
    store::build(&vb).unwrap();
    assert!(ra.records > 0);
    let ta = tree(&store::index_dir(&va));
    assert_eq!(ta, tree(&store::index_dir(&vb)));
    // single segment, no writer locks, sorted file list
    let lex: Vec<&String> = ta
        .iter()
        .map(|(n, _)| n)
        .filter(|n| n.starts_with("lexical/"))
        .collect();
    assert!(lex.iter().all(|n| !n.contains(".lock")), "{lex:?}");
    let seg: std::collections::BTreeSet<&str> = lex
        .iter()
        .filter_map(|n| n.strip_prefix("lexical/"))
        .filter(|n| !n.starts_with('.') && *n != "meta.json")
        .map(|n| n.split('.').next().unwrap())
        .collect();
    assert_eq!(seg.len(), 1, "{lex:?}");
    // a rebuild in place gives the same bytes; so does opening and querying
    store::build(&va).unwrap();
    let s = Store::open(&va).unwrap();
    let view = View::new(&va, &s, "t0").unwrap();
    search::lexical(&s, &view, "espresso grind", 10, false).unwrap();
    let after: Vec<(String, Vec<u8>)> = tree(&store::index_dir(&va))
        .into_iter()
        .filter(|(n, _)| !n.ends_with(".tantivy-meta.lock"))
        .collect();
    assert_eq!(after, ta);
    fs::remove_dir_all(a).unwrap();
    fs::remove_dir_all(b).unwrap();
}

#[test]
fn ranking_is_identical_over_100_repeats() {
    let root = mini_vault("repeat");
    let v = Vault::open(&root).unwrap();
    store::build(&v).unwrap();
    let s = Store::open(&v).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    for q in [
        "espresso grind size",
        "roast",
        "the coffee",
        "water temperature brew ratio",
    ] {
        let first = search::lexical(&s, &view, q, 10, false).unwrap();
        assert!(!first.is_empty(), "{q}");
        for _ in 0..100 {
            assert_eq!(search::lexical(&s, &view, q, 10, false).unwrap(), first);
        }
        // a fresh store and view give the same
        let s2 = Store::open(&v).unwrap();
        let view2 = View::new(&v, &s2, "t0").unwrap();
        assert_eq!(search::lexical(&s2, &view2, q, 10, false).unwrap(), first);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bm25_by_hand_and_ties_by_key() {
    let root = tmp("bm25");
    let mut v = Vault::open(&root).unwrap();
    let h = Provenance::default;
    // one paragraph per document: one chunk each
    v.ingest("a.md", "alpha beta beta\n", h(), MatcherVersion::V3)
        .unwrap();
    v.ingest("b.md", "alpha gamma\n", h(), MatcherVersion::V3)
        .unwrap();
    v.ingest("c.md", "gamma delta delta delta\n", h(), MatcherVersion::V3)
        .unwrap();
    v.ingest("d.md", "alpha gamma\n", h(), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t0")).unwrap();
    store::build(&v).unwrap();
    let s = Store::open(&v).unwrap();
    let view = View::new(&v, &s, "t0").unwrap();
    let hits = search::lexical(&s, &view, "alpha beta", 10, false).unwrap();
    // N 4, lengths 3 2 4 2 → avgdl 2.75; df(alpha) 3, df(beta) 1
    let (k1, b, n, avgdl) = (1.2f64, 0.75f64, 4.0f64, 2.75f64);
    let idf = |df: f64| (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
    let term = |tf: f64, dl: f64, df: f64| {
        idf(df) * tf * (k1 + 1.0) / (tf + k1 * (1.0 - b + b * dl / avgdl))
    };
    let a = term(1.0, 3.0, 3.0) + term(2.0, 3.0, 1.0);
    let bd = term(1.0, 2.0, 3.0);
    let got: Vec<(&str, f64)> = hits
        .iter()
        .map(|h| (h.doc_path.as_str(), h.score))
        .collect();
    assert_eq!(got.len(), 3, "c.md has no query term: {got:?}");
    assert_eq!(got[0].0, "a.md");
    assert!((got[0].1 - a).abs() < 1e-12, "{got:?} {a}");
    // b.md and d.md tie exactly: broken by doc_path
    assert_eq!((got[1].0, got[2].0), ("b.md", "d.md"));
    assert_eq!(got[1].1, got[2].1);
    assert!((got[1].1 - bd).abs() < 1e-12);
    // no query token → nothing; k bounds the list
    assert!(search::lexical(&s, &view, "?!", 10, false)
        .unwrap()
        .is_empty());
    assert_eq!(
        search::lexical(&s, &view, "alpha", 1, false).unwrap().len(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn query_sees_only_the_checkpoint_and_unindexed_snapshots_fail() {
    let root = tmp("at");
    let mut v = Vault::open(&root).unwrap();
    let h = Provenance::default;
    v.ingest(
        "a.md",
        "first version about espresso\n",
        h(),
        MatcherVersion::V3,
    )
    .unwrap();
    v.checkpoint(Some("t0")).unwrap();
    v.ingest(
        "a.md",
        "second version about filter coffee\n",
        h(),
        MatcherVersion::V3,
    )
    .unwrap();
    v.ingest("b.md", "espresso again\n", h(), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t1")).unwrap();
    store::build(&v).unwrap();
    let s = Store::open(&v).unwrap();
    let t0 = View::new(&v, &s, "t0").unwrap();
    let t1 = View::new(&v, &s, "t1").unwrap();
    let docs = |hits: Vec<search::Hit>| hits.into_iter().map(|h| h.doc_path).collect::<Vec<_>>();
    assert_eq!(
        docs(search::lexical(&s, &t0, "espresso", 10, false).unwrap()),
        ["a.md"]
    );
    assert_eq!(
        docs(search::lexical(&s, &t1, "espresso", 10, false).unwrap()),
        ["b.md"]
    );
    assert!(search::lexical(&s, &t0, "filter", 10, false)
        .unwrap()
        .is_empty());
    // a checkpoint taken after the build names a snapshot the index never saw
    v.ingest("b.md", "espresso and more\n", h(), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t2")).unwrap();
    let e = View::new(&v, &s, "t2").err().expect("unindexed snapshot");
    assert!(format!("{e:#}").contains("is not indexed"), "{e:#}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_index_files_are_errors() {
    let root = mini_vault("tamper");
    let v = Vault::open(&root).unwrap();
    store::build(&v).unwrap();
    let p = store::index_dir(&v).join("records.jsonl");
    let mut bytes = fs::read(&p).unwrap();
    let i = bytes.iter().position(|&c| c == b'a').unwrap();
    bytes[i] = b'b';
    fs::write(&p, bytes).unwrap();
    assert!(Store::open(&v).is_err());
    fs::remove_dir_all(store::index_dir(&v)).unwrap();
    assert!(format!("{:#}", Store::open(&v).err().unwrap()).contains("no index"));
    fs::remove_dir_all(root).unwrap();
}

/// C2 review fix 1: an intermediate segment (writer over budget) is an
/// error, never a nondeterministic merge; the corpus-sized budget writes the
/// same records as one segment.
#[test]
fn intermediate_segments_are_an_error_and_the_sized_budget_avoids_them() {
    use mdkindex::lexical;
    use mdkindex::record::ChunkRecord;
    let word = |i: usize| format!("w{} ", i % 5000);
    let records: Vec<ChunkRecord> = (0..3000)
        .map(|i| {
            let text: String = (0..1000).map(|j| word(i * 7 + j)).collect();
            ChunkRecord {
                record_id: format!("{i:032x}"),
                doc_path: format!("d{i:05}.md"),
                chunk_id: format!("{i:032x}"),
                heading_path: vec![],
                blocks: vec![],
                lineage_ids: vec![],
                lexical_len: 1000,
                text,
            }
        })
        .collect();
    let small = tmp("small-budget");
    let e =
        lexical::write_with_budget(&small.join("lexical"), &records, lexical::WRITER_BUDGET_MIN)
            .expect_err("over budget");
    assert!(format!("{e:#}").contains("segments"), "{e:#}");
    let (a, b) = (tmp("sized-a"), tmp("sized-b"));
    lexical::write(&a.join("lexical"), &records).unwrap();
    lexical::write(&b.join("lexical"), &records).unwrap();
    assert_eq!(tree(&a), tree(&b));
    for d in [small, a, b] {
        fs::remove_dir_all(d).unwrap();
    }
}
