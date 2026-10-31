//! P2-V.1/E1 (GOAL P2-V.1 Kapsam 2, V1-4): an index of the ONNX Runtime
//! numerics — built by the `rootmark` of `ROUND_START` (`749a4a5`; its
//! product code is P2-R's, P2-V changed none) in
//! `server/tests/fixtures/index_ort` — is `migration_required`;
//! `rootmark migrate` rebuilds it (`index_rebuild`) into an index byte-equal
//! to a fresh build of the same vault; the rollback brings the old index
//! back byte for byte; `--dry-run` changes nothing; the log and its replay
//! never change.
//!
//! Needs the model files (`make models`).

use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn rootmark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(args)
        .output()
        .unwrap()
}

fn ok_json(o: Output) -> Value {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2v1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// A copy of the ONNX Runtime index fixture.
fn ort_vault(name: &str) -> PathBuf {
    let root = tmp(name);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_ort/vault");
    assert!(Command::new("cp")
        .args(["-R", src.to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    root.join("vault")
}

/// Every file under `dir` with its bytes (relative paths).
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    if !dir.exists() {
        return out;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn manifest(v: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(v.join(".rootmark/index/manifest.json")).unwrap())
        .unwrap()
}

#[test]
fn an_onnx_runtime_index_is_migrated_to_a_fresh_build_and_back() {
    let v = ort_vault("forward");
    let vs = v.to_str().unwrap();
    let old_index = tree(&v.join(".rootmark/index"));
    let old_all = tree(&v);
    let old = manifest(&v);
    assert_eq!(
        old["format"],
        rootmarkindex::store::FORMAT,
        "same index format"
    );
    assert_eq!(
        old["embedder_id"], "minilm-l6-v2/e73168f9",
        "ONNX Runtime numerics"
    );
    let hash = ok_json(rootmark(&["events-hash", vs]));

    // status: migration_required, index_rebuild required
    let st = ok_json(rootmark(&["migrate", "--dry-run", vs]));
    assert_eq!(tree(&v), old_all, "--dry-run changed a byte");
    let plan: Vec<&str> = st["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["migration"].as_str().unwrap())
        .collect();
    assert!(plan.contains(&"index_rebuild"), "{st:#}");
    assert_eq!(
        st["matrix"]["artifacts"]["index"]["state"],
        "migration_required"
    );
    assert!(st["matrix"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m == "index_rebuild"));
    // on a copy (the kernel keeps its own files): status and search say
    // migration_required (the API's code), not internal, not stale
    let probe = ort_vault("probe");
    let recipe = rootmarkserver::config::default_recipe()
        .replace("[index]\n", "[index]\nauto_rebuild = false\n")
        + "\n[identity]\nhuman_id = \"ada\"\n";
    std::fs::write(probe.join(".rootmark/policy.toml"), recipe).unwrap();
    let k = Kernel::open(&probe).unwrap();
    let st = k.call("status", &json!({}), Caller::http("p2v1")).unwrap();
    assert_eq!(st["index"]["state"], "stale", "{st:#}");
    assert_eq!(
        st["migrations"]["artifacts"]["index"]["state"],
        "migration_required"
    );
    assert_eq!(st["migrations"]["required"], json!(["index_rebuild"]));
    let e = k
        .call(
            "search",
            &json!({"query": "espresso", "mode": "hybrid", "k": 5}),
            Caller::http("p2v1"),
        )
        .unwrap_err();
    assert_eq!(code_of(&e), Code::MigrationRequired, "{e:#}");
    assert!(format!("{e:#}").contains("ONNX Runtime"), "{e:#}");
    drop(k);
    // the CLI refuses it too
    assert!(!rootmark(&["search", vs, "espresso", "--at", "t0"])
        .status
        .success());

    // forward
    let r = ok_json(rootmark(&["migrate", vs]));
    let applied: Vec<&str> = r["applied"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["migration"].as_str().unwrap())
        .collect();
    assert!(applied.contains(&"index_rebuild"), "{r:#}");
    assert_eq!(r["matrix"]["artifacts"]["index"]["state"], "ok");
    assert_eq!(ok_json(rootmark(&["events-hash", vs])), hash);
    let new = manifest(&v);
    assert_eq!(
        new["embedder_id"],
        format!("minilm-l6-v2/e73168f9+{}", rootmarkindex::embed::NUMERICS)
    );
    let migrated = tree(&v.join(".rootmark/index"));
    // the rebuilt index opens (a search would write a trace)
    ok_json(rootmark(&["index", "status", vs]));

    // byte-equal to a fresh build of the same vault
    let f = ort_vault("fresh");
    let fs_ = f.to_str().unwrap();
    std::fs::remove_dir_all(f.join(".rootmark/index")).unwrap();
    ok_json(rootmark(&["index", "build", fs_]));
    assert_eq!(
        tree(&f.join(".rootmark/index")),
        migrated,
        "the migrated index differs from a fresh build"
    );

    // idempotent
    let again = ok_json(rootmark(&["migrate", vs]));
    assert_eq!(again["applied"], serde_json::json!([]));

    // back: newest record first, the old index byte for byte
    let mut ids: Vec<String> = rootmarkserver::migrate::records(&v)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    ids.reverse();
    for id in &ids {
        let b = ok_json(rootmark(&["migrate", "--rollback", id, vs]));
        assert_eq!(b["byte_equal"], true, "{b:#}");
    }
    assert_eq!(
        tree(&v.join(".rootmark/index")),
        old_index,
        "old index not back"
    );
    assert_eq!(tree(&v), old_all, "vault not byte-equal after the rollback");
    assert_eq!(ok_json(rootmark(&["events-hash", vs])), hash);
}
