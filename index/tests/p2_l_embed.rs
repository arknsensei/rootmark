//! P2-L/L2 (GOAL P2-L 3): the multilingual embedder option
//! `multilingual-minilm-l12-v2` (paraphrase-multilingual-MiniLM-L12-v2, ONNX,
//! pinned by `models/MANIFEST.multilingual-minilm-l12-v2.json`, sha256 and
//! blake3): its `embedder_id` from the manifest, unit vectors of 384,
//! bit-identical across two loads and batch ≡ single (the C-10 equivalent),
//! a Turkish sentence nearer to its English paraphrase than to an unrelated
//! one; chosen per vault (`[index] embedder`), recorded in the manifest, an
//! index of the other embedder refused; the default unchanged.
//!
//! Needs the model files (`make models`, `make models-multilingual`).

use rootmarkcore::{MatcherVersion, Vault};
use rootmarkindex::config::{IndexConfig, EMBEDDER_DEFAULT, EMBEDDER_MULTILINGUAL};
use rootmarkindex::embed::{Embedder, OnnxMiniLm, DIM};
use rootmarkindex::models;
use rootmarkindex::store::{self, Store};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

fn multilingual() -> &'static OnnxMiniLm {
    static E: OnceLock<OnnxMiniLm> = OnceLock::new();
    E.get_or_init(|| {
        OnnxMiniLm::load_named(EMBEDDER_MULTILINGUAL)
            .expect("model (run `make models-multilingual`)")
    })
}

const TEXTS: [&str; 6] = [
    "Türk kahvesi cezvede kısık ateşte pişirilir.",
    "Turkish coffee is brewed slowly in a cezve over a low flame.",
    "Borsa İstanbul'da hisse senetleri bugün yükseldi.",
    "İSTANBUL Boğazı'nda ılık bir akşam.",
    "A warm evening on the Bosphorus in Istanbul.",
    "değerlendirme",
];

fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

#[test]
fn the_multilingual_model_is_pinned_and_named_by_its_manifest() {
    let m = models::verify_named(&models::models_root(), EMBEDDER_MULTILINGUAL).unwrap();
    assert_eq!(
        m.manifest().source,
        "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
    );
    assert_eq!(
        m.manifest().revision,
        "e8f8c211226b894fcb81acc59f3b34ba3efd5f42"
    );
    let b3 = &m.manifest().files[models::MODEL_FILE].blake3;
    assert_eq!(
        multilingual().id(),
        format!("{EMBEDDER_MULTILINGUAL}/{}", &b3[..8])
    );
    assert_eq!(multilingual().dim(), DIM);
    // the default model and its manifest are unchanged
    let d = models::verify_named(&models::models_root(), EMBEDDER_DEFAULT).unwrap();
    assert_eq!(
        d.embedder_id(),
        models::verify(&models::models_root())
            .unwrap()
            .embedder_id()
    );
    assert!(d.embedder_id().starts_with("minilm-l6-v2/"));
    assert!(models::verify_named(&models::models_root(), "bert").is_err());
}

#[test]
fn vectors_are_unit_deterministic_and_batch_equals_single() {
    let a = multilingual().embed(&TEXTS).unwrap();
    // a second, independent load gives the same bits
    let b = OnnxMiniLm::load_named(EMBEDDER_MULTILINGUAL)
        .unwrap()
        .embed(&TEXTS)
        .unwrap();
    let bits = |v: &[Vec<f32>]| -> Vec<Vec<u32>> {
        v.iter()
            .map(|x| x.iter().map(|f| f.to_bits()).collect())
            .collect()
    };
    assert_eq!(bits(&a), bits(&b));
    for (t, v) in TEXTS.iter().zip(&a) {
        assert_eq!(v.len(), DIM);
        let n = dot(v, v).sqrt();
        assert!((n - 1.0).abs() < 1e-5, "{t:?}: norm {n}");
        let one = multilingual().embed(&[t]).unwrap();
        assert_eq!(
            bits(&one),
            bits(std::slice::from_ref(v)),
            "{t:?}: batch != single"
        );
    }
}

#[test]
fn a_turkish_sentence_is_nearest_its_english_paraphrase() {
    let v = multilingual().embed(&TEXTS).unwrap();
    // coffee TR ↔ coffee EN above coffee TR ↔ stocks TR; evening TR ↔ EN too
    assert!(
        dot(&v[0], &v[1]) > dot(&v[0], &v[2]),
        "{} {}",
        dot(&v[0], &v[1]),
        dot(&v[0], &v[2])
    );
    assert!(
        dot(&v[3], &v[4]) > dot(&v[3], &v[2]),
        "{} {}",
        dot(&v[3], &v[4]),
        dot(&v[3], &v[2])
    );
}

fn recipe(embedder: &str) -> String {
    let b = rootmarkcore::Policy::builtin();
    format!(
        "[promotion]\npolicy = \"human_only\"\n\n[edit]\npolicy = \"append_only\"\n\n\
         [version_trigger]\nidle_ms = {}\nmin_sim_change = {:?}\n\n[journal]\nretention_hours = {}\n\n\
         [index]\nembedder = \"{embedder}\"\n",
        b.trigger.idle_ms(),
        b.trigger.min_sim_change(),
        b.retention.hours(),
    )
}

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2l2-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("kahve.md"), format!("# Kahve\n\n{}\n", TEXTS[0])).unwrap();
    fs::write(src.join("borsa.md"), format!("# Borsa\n\n{}\n", TEXTS[2])).unwrap();
    let v = root.join("vault");
    let mut vault = Vault::open(&v).unwrap();
    vault.ingest_dir(&src, MatcherVersion::V3).unwrap();
    vault.checkpoint(Some("t0")).unwrap();
    v
}

#[test]
fn a_vault_s_index_records_its_embedder_and_refuses_the_other() {
    let v = vault("ml");
    fs::write(
        v.join(".rootmark/policy.toml"),
        recipe(EMBEDDER_MULTILINGUAL),
    )
    .unwrap();
    assert_eq!(
        IndexConfig::of_vault(&v).unwrap().embedder,
        EMBEDDER_MULTILINGUAL
    );
    let vault = Vault::open(&v).unwrap();
    let e = OnnxMiniLm::for_vault(&v).unwrap();
    assert_eq!(e.id(), multilingual().id());
    let r = store::build(&vault, &e).unwrap();
    assert_eq!(r.embedder_id, multilingual().id());
    let m: Value =
        serde_json::from_str(&fs::read_to_string(v.join(".rootmark/index/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(m["embedder_id"], multilingual().id());
    assert!(Store::open(&vault, &e).is_ok());
    // the default model cannot query it
    let d = OnnxMiniLm::load_default().unwrap();
    let err = Store::open(&vault, &d).err().expect("refused");
    assert!(format!("{err:#}").contains("embedder_id"), "{err:#}");
    // two builds: the same index bytes
    let a = fs::read(v.join(".rootmark/index/vectors/vectors.f32")).unwrap();
    fs::remove_dir_all(v.join(".rootmark/index")).unwrap();
    store::build(&vault, &e).unwrap();
    assert_eq!(
        a,
        fs::read(v.join(".rootmark/index/vectors/vectors.f32")).unwrap()
    );
}
