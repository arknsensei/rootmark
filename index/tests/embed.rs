//! P1-C/C1 (GOAL C-10; TASKS C1): the ONNX embedder against the prototype's
//! sentence-transformers vectors (`research/data/embed_ref.json`, written by
//! `research/eval/gen_embed_ref.py`), bit-level determinism, batch ≡ single,
//! and the model manifest check. Needs the model files (`make models`).

use rootmarkindex::embed::{Embedder, OnnxMiniLm, DIM};
use rootmarkindex::models;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn embedder() -> &'static OnnxMiniLm {
    static E: OnceLock<OnnxMiniLm> = OnceLock::new();
    E.get_or_init(|| OnnxMiniLm::load_default().expect("model (run `make models`)"))
}

fn reference() -> Value {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/data/embed_ref.json");
    serde_json::from_str(&fs::read_to_string(&p).expect("embed_ref.json")).unwrap()
}

fn cosine(a: &[f32], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| f64::from(*x) * y).sum();
    let na: f64 = a.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|y| y.powi(2)).sum::<f64>().sqrt();
    dot / (na * nb)
}

fn bits(v: &[Vec<f32>]) -> Vec<Vec<u32>> {
    v.iter()
        .map(|x| x.iter().map(|f| f.to_bits()).collect())
        .collect()
}

#[test]
fn embedder_id_is_derived_from_the_manifest() {
    let m = models::verify(&models::models_root()).unwrap();
    let b3 = &m.manifest().files[models::MODEL_FILE].blake3;
    // the rule of P2-V.1/E1: name, model hash, numerics tag
    assert_eq!(embedder().id(), format!("minilm-l6-v2/{}+fixed1", &b3[..8]));
    assert_eq!(embedder().id(), m.embedder_id());
    assert_eq!(embedder().dim(), DIM);
}

/// GOAL C-10: cosine ≥ 0.999 for each of the 50 reference chunks. With
/// `ROOTMARK_C10_OUT` the per-chunk cosines are written as JSON (C9 report).
#[test]
fn c10_onnx_matches_the_prototype_embeddings() {
    let r = reference();
    let chunks = r["chunks"].as_array().unwrap();
    assert_eq!(chunks.len(), 50, "embed_ref.json holds 50 chunks");
    let texts: Vec<&str> = chunks.iter().map(|c| c["text"].as_str().unwrap()).collect();
    let got = embedder().embed(&texts).unwrap();
    let mut rows = Vec::new();
    let mut min = f64::INFINITY;
    for (c, v) in chunks.iter().zip(&got) {
        let want: Vec<f64> = c["vector"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        assert_eq!(want.len(), DIM);
        assert_eq!(v.len(), DIM);
        let cos = cosine(v, &want);
        min = min.min(cos);
        rows.push(serde_json::json!({
            "doc_path": c["doc_path"], "chunk_id": c["chunk_id"], "cosine": cos,
        }));
    }
    if let Ok(p) = std::env::var("ROOTMARK_C10_OUT") {
        let out = serde_json::json!({
            "threshold": 0.999, "n": rows.len(), "min_cosine": min,
            "pass": min >= 0.999, "embedder_id": embedder().id(), "chunks": rows,
        });
        fs::write(p, serde_json::to_string_pretty(&out).unwrap() + "\n").unwrap();
    }
    assert!(
        min >= 0.999,
        "C-10: minimum cosine {min} < 0.999: {rows:#?}"
    );
}

#[test]
fn same_text_100_times_is_bit_identical() {
    let text = reference()["chunks"][7]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let first = bits(&embedder().embed(&[&text]).unwrap());
    for _ in 0..100 {
        assert_eq!(bits(&embedder().embed(&[&text]).unwrap()), first);
    }
    // a fresh session gives the same bits
    let other = OnnxMiniLm::load_default().unwrap();
    assert_eq!(bits(&other.embed(&[&text]).unwrap()), first);
}

#[test]
fn batch_equals_one_by_one_bit_for_bit() {
    let r = reference();
    let texts: Vec<String> = r["chunks"].as_array().unwrap()[..20]
        .iter()
        .map(|c| c["text"].as_str().unwrap().to_string())
        .collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let batch = bits(&embedder().embed(&refs).unwrap());
    let single: Vec<Vec<u32>> = refs
        .iter()
        .flat_map(|t| bits(&embedder().embed(&[t]).unwrap()))
        .collect();
    assert_eq!(batch, single);
    // and in reverse order
    let mut rev = refs.clone();
    rev.reverse();
    let mut back = bits(&embedder().embed(&rev).unwrap());
    back.reverse();
    assert_eq!(back, batch);
}

#[test]
fn vectors_are_unit_length_and_count_is_tracked() {
    let e = OnnxMiniLm::load_default().unwrap();
    assert_eq!(e.embedded_count(), 0);
    let v = e.embed(&["", "a", "Espresso needs a fine grind."]).unwrap();
    assert_eq!(e.embedded_count(), 3);
    for x in &v {
        let n: f64 = x.iter().map(|f| f64::from(*f).powi(2)).sum::<f64>().sqrt();
        assert!((n - 1.0).abs() < 1e-5, "norm {n}");
    }
    assert!(e.embed(&[]).unwrap().is_empty());
}

// ---- manifest check (C0 review note 3)

fn copy_models(name: &str) -> PathBuf {
    let src = models::models_root();
    let m = models::read_manifest(&src).unwrap();
    let dst =
        std::env::temp_dir().join(format!("rootmark-c1-models-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dst);
    fs::create_dir_all(&dst).unwrap();
    fs::copy(src.join("MANIFEST.json"), dst.join("MANIFEST.json")).unwrap();
    for f in m.files.keys() {
        let to = dst.join(&m.name).join(f);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        // the 90 MB model is linked, not copied; tests below never write it
        if f == models::MODEL_FILE {
            std::os::unix::fs::symlink(src.join(&m.name).join(f), &to).unwrap();
        } else {
            fs::copy(src.join(&m.name).join(f), &to).unwrap();
        }
    }
    dst
}

#[test]
fn manifest_mismatches_are_errors() {
    let ok = copy_models("ok");
    assert!(models::verify(&ok).is_ok());

    let bad_hash = copy_models("hash");
    let p = bad_hash.join("MANIFEST.json");
    let text = fs::read_to_string(&p).unwrap();
    let b3 = models::read_manifest(&bad_hash).unwrap().files["tokenizer.json"]
        .blake3
        .clone();
    let flipped = format!(
        "{}{}",
        if b3.starts_with('0') { "1" } else { "0" },
        &b3[1..]
    );
    fs::write(&p, text.replace(&b3, &flipped)).unwrap();
    let e = models::verify(&bad_hash).unwrap_err();
    assert!(format!("{e:#}").contains("blake3"), "{e:#}");

    let changed = copy_models("bytes");
    let tj = changed.join("minilm-l6-v2/sentence_bert_config.json");
    let mut b = fs::read(&tj).unwrap();
    b.push(b' ');
    fs::write(&tj, b).unwrap();
    let e = models::verify(&changed).unwrap_err();
    assert!(format!("{e:#}").contains("size"), "{e:#}");

    let missing = copy_models("missing");
    fs::remove_file(missing.join("minilm-l6-v2/tokenizer.json")).unwrap();
    assert!(models::verify(&missing).is_err());

    let no_entry = copy_models("entry");
    let p = no_entry.join("MANIFEST.json");
    let mut m: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    m["files"].as_object_mut().unwrap().remove("tokenizer.json");
    fs::write(&p, serde_json::to_string(&m).unwrap()).unwrap();
    assert!(models::read_manifest(&no_entry).is_err());

    for d in [ok, bad_hash, changed, missing, no_entry] {
        fs::remove_dir_all(d).unwrap();
    }
}
