//! P2-V.1/E1 (GOAL P2-V.1 Kapsam 2, V1-2): the embedder's vectors are the
//! same bytes on every processor (`crate::detbert`). The golden blake3 of
//! each model's vectors for fixed texts were computed once; the test runs on
//! every CI platform (`make test`: macOS arm64, Linux x86_64) and on the
//! developer's machine, so a processor-dependent byte fails it. An index of
//! the ONNX Runtime numerics (an `embedder_id` without the tag) is stale.
//!
//! Needs the model files (`make models`, `make models-multilingual`).

use rootmarkcore::{MatcherVersion, Vault};
use rootmarkindex::config::EMBEDDER_MULTILINGUAL;
use rootmarkindex::embed::{onnx_runtime_numerics, Embedder, OnnxMiniLm, NUMERICS};
use rootmarkindex::store;
use std::fs;
use std::path::{Path, PathBuf};

const TEXTS: [&str; 5] = [
    "Espresso is brewed by forcing hot water through finely ground coffee.",
    "Türk kahvesi cezvede kısık ateşte pişirilir; köpüğü bozulmadan fincana alınır.",
    "x",
    "## Maintenance\n\nDescale the machine every month; backflush the group head daily with a blind basket.",
    // a long text: truncated to 256 tokens
    "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua ut enim ad minim veniam quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur excepteur sint occaecat cupidatat non proident sunt in culpa qui officia deserunt mollit anim id est laborum lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua ut enim ad minim veniam quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur excepteur sint occaecat cupidatat non proident sunt in culpa qui officia deserunt mollit anim id est laborum lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua",
];

fn digest(e: &OnnxMiniLm) -> String {
    let vecs = e.embed(&TEXTS).unwrap();
    let mut h = blake3::Hasher::new();
    for v in &vecs {
        for x in v {
            h.update(&x.to_le_bytes());
        }
    }
    h.finalize().to_hex().to_string()
}

#[test]
fn the_default_model_gives_the_same_bytes_on_every_processor() {
    let e = OnnxMiniLm::load_default().expect("model (run `make models`)");
    assert_eq!(digest(&e), GOLDEN_DEFAULT, "vector bytes changed");
}

#[test]
fn the_multilingual_model_gives_the_same_bytes_on_every_processor() {
    let e = OnnxMiniLm::load_named(EMBEDDER_MULTILINGUAL)
        .expect("model (run `make models-multilingual`)");
    assert_eq!(digest(&e), GOLDEN_MULTILINGUAL, "vector bytes changed");
}

// computed on Apple M4 Max (P2-V.1/E1); E0 showed the same bytes on Xeon 8370C,
// EPYC 7763, Neoverse-N2 and Apple M1
const GOLDEN_DEFAULT: &str = "7e1fd50af24655bcca91ba092cd8f73c2404e4c17c243557cd14136890c98bd1";
const GOLDEN_MULTILINGUAL: &str =
    "f78c5be1aa0e15883bb465233a08f46b571de7df100427b1eed9de0257fdd075";

#[test]
fn the_onnx_runtime_ids_are_the_manifest_models_without_the_tag() {
    for (name, e) in [
        ("minilm-l6-v2", OnnxMiniLm::load_default().unwrap()),
        (
            "multilingual",
            OnnxMiniLm::load_named(EMBEDDER_MULTILINGUAL).unwrap(),
        ),
    ] {
        let id = e.id().to_string();
        let old = id.strip_suffix(&format!("+{NUMERICS}")).expect(name);
        assert!(onnx_runtime_numerics(old), "{old}");
        assert!(!onnx_runtime_numerics(&id), "{id}");
    }
}

fn here(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
}

#[test]
fn an_index_of_the_onnx_runtime_numerics_is_stale() {
    let root = std::env::temp_dir().join(format!("rootmark-p2v1-stale-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&here("../spec/fixtures/mini_vault"), MatcherVersion::V3)
        .unwrap();
    v.checkpoint(Some("t0")).unwrap();
    let e = OnnxMiniLm::load_default().unwrap();
    store::build(&v, &e).unwrap();
    assert!(store::staleness(&v).unwrap().is_none());
    let m = store::index_dir(&v).join("manifest.json");
    let text = fs::read_to_string(&m).unwrap();
    fs::write(&m, text.replace(&format!("+{NUMERICS}"), "")).unwrap();
    let why = store::staleness(&v).unwrap().expect("stale");
    assert!(format!("{why:?}").contains("older rootmark"), "{why:?}");
    assert!(store::Store::open(&v, &e).is_err());
}
