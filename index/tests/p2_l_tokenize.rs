//! P2-L/L1 (GOAL P2-L 2, L-1): the lexical variant `uni_fold_tr` — Turkish
//! words one token each, Turkish case folding, NFC — chosen per vault
//! (`[index] tokenizer`) and declared in the index manifest; the default
//! variant `mdk_lower_alnum` unchanged on the 200 reference chunks (the C2
//! reference `research/data/tok_ref.json`); an index of the other variant is
//! stale for the vault.
//!
//! Needs the model files (`make models`).

use rootmarkcore::{MatcherVersion, Vault};
use rootmarkindex::config::IndexConfig;
use rootmarkindex::embed::OnnxMiniLm;
use rootmarkindex::search::{self, View};
use rootmarkindex::store::{self, LexicalParams, Store};
use rootmarkindex::tokenize::{tokens, uni_fold_tr, Variant, VariantTokenizer};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tantivy::tokenizer::{TextAnalyzer, TokenStream};

fn embedder() -> &'static OnnxMiniLm {
    static E: OnceLock<OnnxMiniLm> = OnceLock::new();
    E.get_or_init(|| OnnxMiniLm::load_default().expect("model (run `make models`)"))
}

fn here(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-p2l1-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A complete recipe (the kernel's built-in values) with `[index] tokenizer`.
fn recipe(tok: &str) -> String {
    let b = rootmarkcore::Policy::builtin();
    format!(
        "[promotion]\npolicy = \"human_only\"\n\n[edit]\npolicy = \"append_only\"\n\n\
         [version_trigger]\nidle_ms = {}\nmin_sim_change = {:?}\n\n[journal]\nretention_hours = {}\n\n\
         [index]\ntokenizer = \"{tok}\"\n",
        b.trigger.idle_ms(),
        b.trigger.min_sim_change(),
        b.retention.hours(),
    )
}

fn t(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn analyzer(v: Variant, text: &str) -> Vec<String> {
    let mut a = TextAnalyzer::from(VariantTokenizer(v));
    let mut s = a.token_stream(text);
    let mut out = Vec::new();
    while s.advance() {
        out.push(s.token().text.clone());
    }
    out
}

#[test]
fn turkish_words_are_one_token_each() {
    for (text, want) in [
        ("değerlendirme", t(&["değerlendirme"])),
        ("Şengönül", t(&["şengönül"])),
        ("İSTANBUL", t(&["istanbul"])),
        ("istanbul", t(&["istanbul"])),
        ("ILIK", t(&["ılık"])),
        ("ılık", t(&["ılık"])),
        (
            "Çağrı, öğün: ŞÜKRÜ'nün 3 çiçeği.",
            t(&["çağrı", "öğün", "şükrü", "nün", "3", "çiçeği"]),
        ),
    ] {
        assert_eq!(uni_fold_tr(text), want, "{text:?}");
        assert_eq!(
            analyzer(Variant::UniFoldTr, text),
            want,
            "analyzer {text:?}"
        );
    }
    // the same word in either case is one term
    assert_eq!(uni_fold_tr("İSTANBUL"), uni_fold_tr("istanbul"));
    assert_eq!(uni_fold_tr("ILIK"), uni_fold_tr("ılık"));
    // the default variant splits them (why the variant exists)
    assert_eq!(tokens("değerlendirme"), t(&["de", "erlendirme"]));
    assert_eq!(tokens("Şengönül"), t(&["eng", "n", "l"]));
}

#[test]
fn folding_is_turkish_then_unicode_lower_case_after_nfc() {
    // İ → i (no combining dot), I → ı, then lower case
    assert_eq!(uni_fold_tr("\u{130}"), t(&["i"]));
    assert_eq!(uni_fold_tr("I"), t(&["\u{131}"]));
    assert_eq!(uni_fold_tr("İı Iİ"), t(&["iı", "ıi"]));
    // NFC first: a decomposed İ (I + U+0307) and Ş (S + U+0327) fold as the
    // composed ones
    assert_eq!(uni_fold_tr("I\u{307}STANBUL"), uni_fold_tr("İSTANBUL"));
    assert_eq!(
        uni_fold_tr("S\u{327}engo\u{308}nu\u{308}l"),
        t(&["şengönül"])
    );
    // other scripts and letters: Unicode lower case, Alphabetic/Numeric runs
    assert_eq!(uni_fold_tr("Straße STRASSE"), t(&["straße", "strasse"]));
    assert_eq!(uni_fold_tr("Москва МОСКВА"), t(&["москва", "москва"]));
    assert_eq!(uni_fold_tr("Ελλάδα"), t(&["ελλάδα"]));
    assert_eq!(uni_fold_tr("abc123def ٣٤"), t(&["abc123def", "٣٤"]));
    // punctuation, symbols and emoji are separators
    assert_eq!(
        uni_fold_tr("kahve☕️demleme 🙂 a-b_c"),
        t(&["kahve", "demleme", "a", "b", "c"])
    );
    assert_eq!(uni_fold_tr(""), Vec::<String>::new());
    assert_eq!(uni_fold_tr("  ..  "), Vec::<String>::new());
}

#[test]
fn the_default_variant_is_unchanged_on_the_200_reference_chunks() {
    let r: Value =
        serde_json::from_str(&fs::read_to_string(here("../research/data/tok_ref.json")).unwrap())
            .unwrap();
    let chunks = r["chunks"].as_array().unwrap();
    assert_eq!(chunks.len(), 200);
    for c in chunks.iter().chain(r["edge"].as_array().unwrap()) {
        let text = c["text"].as_str().unwrap();
        let want: Vec<String> = c["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect();
        assert_eq!(Variant::LowerAlnum.tokens(text), want, "{text:?}");
        assert_eq!(analyzer(Variant::LowerAlnum, text), want, "{text:?}");
    }
    assert_eq!(Variant::default(), Variant::LowerAlnum);
    assert_eq!(
        LexicalParams::current(),
        LexicalParams::of(Variant::LowerAlnum)
    );
    assert_eq!(LexicalParams::current().tokenizer, "mdk_lower_alnum");
}

#[test]
fn names_and_the_recipe() {
    for v in Variant::ALL {
        assert_eq!(Variant::of_name(v.name()), Some(v));
    }
    assert_eq!(Variant::of_name("mdk_lower"), None);
    let tbl = |s: &str| s.parse::<toml::Table>().unwrap();
    assert_eq!(IndexConfig::of_table(None).unwrap(), IndexConfig::default());
    assert_eq!(
        IndexConfig::of_table(Some(&tbl("tokenizer = \"uni_fold_tr\"")))
            .unwrap()
            .tokenizer,
        Variant::UniFoldTr
    );
    for bad in [
        "tokenizer = \"turkish\"",
        "tokenizer = 1",
        "embedder = \"bert\"",
    ] {
        assert!(IndexConfig::of_table(Some(&tbl(bad))).is_err(), "{bad}");
    }
}

/// A vault of three Turkish notes with `[index] tokenizer = <tok>`.
fn turkish_vault(name: &str, tok: Option<&str>) -> PathBuf {
    let root = tmp(name);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("sehir.md"),
        "# İSTANBUL\n\nBoğaz kıyısında ILIK bir akşam.\n",
    )
    .unwrap();
    fs::write(
        src.join("kahve.md"),
        "# Kahve\n\nTürk kahvesi değerlendirme notları.\n",
    )
    .unwrap();
    fs::write(
        src.join("kisi.md"),
        "# Kişiler\n\nŞengönül ailesi Ankara'da yaşar.\n",
    )
    .unwrap();
    let v = root.join("vault");
    let mut vault = Vault::open(&v).unwrap();
    vault.ingest_dir(&src, MatcherVersion::V3).unwrap();
    vault.checkpoint(Some("t0")).unwrap();
    if let Some(tok) = tok {
        fs::write(v.join(".rootmark/policy.toml"), recipe(tok)).unwrap();
    }
    v
}

fn lexical_hits(v: &Path, q: &str) -> Vec<String> {
    let vault = Vault::open(v).unwrap();
    let s = Store::open(&vault, embedder()).unwrap();
    let view = View::new(&vault, &s, "t0").unwrap();
    search::lexical(&s, &view, q, 10, false)
        .unwrap()
        .iter()
        .map(|h| h.doc_path.clone())
        .collect()
}

#[test]
fn a_vault_s_index_is_built_and_queried_with_its_variant() {
    let v = turkish_vault("uni", Some("uni_fold_tr"));
    let vault = Vault::open(&v).unwrap();
    store::build(&vault, embedder()).unwrap();
    let m: Value =
        serde_json::from_str(&fs::read_to_string(v.join(".rootmark/index/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(m["lexical"]["tokenizer"], "uni_fold_tr");
    assert_eq!(m["format"], store::FORMAT);
    let meta = fs::read_to_string(v.join(".rootmark/index/lexical/meta.json")).unwrap();
    assert!(meta.contains("\"uni_fold_tr\""), "{meta}");
    assert_eq!(lexical_hits(&v, "istanbul"), ["sehir.md"]);
    assert_eq!(lexical_hits(&v, "ılık"), ["sehir.md"]);
    assert_eq!(lexical_hits(&v, "Değerlendirme"), ["kahve.md"]);
    assert_eq!(lexical_hits(&v, "şengönül"), ["kisi.md"]);
    // the default variant does not fold the Turkish capitals: "İSTANBUL"
    // is "i" + "stanbul" there, the query "istanbul" one other term
    let d = turkish_vault("default", None);
    store::build(&Vault::open(&d).unwrap(), embedder()).unwrap();
    assert!(lexical_hits(&d, "istanbul").is_empty());
}

#[test]
fn the_other_variant_s_index_is_stale_and_a_rebuild_is_byte_equal() {
    let v = turkish_vault("stale", None);
    store::build(&Vault::open(&v).unwrap(), embedder()).unwrap();
    assert!(store::staleness(&Vault::open(&v).unwrap())
        .unwrap()
        .is_none());
    // the vault now asks for the other variant: the index is stale
    fs::write(v.join(".rootmark/policy.toml"), recipe("uni_fold_tr")).unwrap();
    let vault = Vault::open(&v).unwrap();
    let s = store::staleness(&vault).unwrap().expect("stale");
    assert!(s.0.contains("uni_fold_tr"), "{s}");
    assert!(Store::open(&vault, embedder()).is_err());
    // rebuilt (vectors reused: they do not depend on the variant), twice:
    // the same bytes
    let r = store::build(&vault, embedder()).unwrap();
    assert_eq!(r.embedded, 0, "{}", r.reuse);
    let a = tree(&v.join(".rootmark/index"));
    store::build(&vault, embedder()).unwrap();
    assert_eq!(a, tree(&v.join(".rootmark/index")));
    assert!(store::staleness(&vault).unwrap().is_none());
}

fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().to_string();
                out.push((rel, fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}
