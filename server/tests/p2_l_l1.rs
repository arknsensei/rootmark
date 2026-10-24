//! P2-L/L1 (GOAL P2-L L-8's unit): a vault whose index was built with the
//! default lexical variant and whose recipe then names `uni_fold_tr`
//! (`[index] tokenizer`) — or the other way round — reports its index
//! `migration_required` (`status.migrations`, `search` 503), `status.index`
//! names both variants (`format` / `expected_format` with `+uni_fold_tr`);
//! `rootmark migrate` rebuilds it with the vault's variant and
//! `events_hash` does not change; an unknown variant is refused at start.
//!
//! Needs the model files (`make models`).

use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

fn rootmark(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(args)
        .output()
        .unwrap()
}

fn ok_json(args: &[&str]) -> Value {
    let o = rootmark(args);
    assert!(
        o.status.success(),
        "rootmark {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

fn recipe(tokenizer: Option<&str>) -> String {
    let base = rootmarkserver::config::default_recipe()
        .replace("[index]\n", "[index]\nauto_rebuild = false\n");
    let base = match tokenizer {
        Some(t) => base.replace("[index]\n", &format!("[index]\ntokenizer = \"{t}\"\n")),
        None => base,
    };
    base + "\n[identity]\nhuman_id = \"ada\"\n"
}

fn vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2l1s-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(
        v.join("sehir.md"),
        "# İSTANBUL\n\nBoğaz kıyısında ILIK bir akşam.\n",
    )
    .unwrap();
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(v.join(".rootmark/policy.toml"), recipe(None)).unwrap();
    assert!(rootmark(&["index", "build", vs]).status.success());
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-l-l1")
}

fn search(k: &Kernel, q: &str) -> anyhow::Result<Value> {
    k.call(
        "search",
        &json!({"query": q, "mode": "lexical", "k": 5}),
        human(),
    )
}

#[test]
fn another_tokenizer_s_index_is_migration_required_until_migrated() {
    for (from, to) in [(None, Some("uni_fold_tr")), (Some("uni_fold_tr"), None)] {
        let v = vault(&format!("{from:?}-{to:?}").replace(['"', '(', ')'], ""));
        let vs = v.to_str().unwrap();
        if from.is_some() {
            std::fs::write(v.join(".rootmark/policy.toml"), recipe(from)).unwrap();
            assert!(rootmark(&["index", "build", vs]).status.success());
        }
        let h0 = ok_json(&["events-hash", vs]);
        std::fs::write(v.join(".rootmark/policy.toml"), recipe(to)).unwrap();
        let k = Kernel::open(&v).unwrap();
        let st = k.call("status", &json!({}), human()).unwrap();
        assert_eq!(st["index"]["state"], "stale", "{st:#}");
        let suffix = |t: Option<&str>| t.map_or(String::new(), |t| format!("+{t}"));
        assert_eq!(
            st["index"]["format"],
            format!("{}{}", rootmarkindex::store::FORMAT, suffix(from))
        );
        assert_eq!(
            st["index"]["expected_format"],
            format!("{}{}", rootmarkindex::store::FORMAT, suffix(to))
        );
        let ix = &st["migrations"]["artifacts"]["index"];
        assert_eq!(ix["state"], "migration_required", "{st:#}");
        assert_eq!(st["migrations"]["required"], json!(["index_rebuild"]));
        let e = search(&k, "istanbul").unwrap_err();
        assert_eq!(code_of(&e), Code::MigrationRequired);
        drop(k);
        let m = ok_json(&["migrate", vs]);
        let applied: Vec<&str> = m["applied"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["migration"].as_str().unwrap())
            .collect();
        assert!(applied.contains(&"index_rebuild"), "{m:#}");
        assert_eq!(m["matrix"]["required"], json!([]));
        assert_eq!(m["events_hash"], h0["events_hash"], "{m:#}");
        let manifest: Value = serde_json::from_slice(
            &std::fs::read(v.join(".rootmark/index/manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            manifest["lexical"]["tokenizer"],
            to.unwrap_or("mdk_lower_alnum")
        );
        // (a search writes a checkpoint event: the log is compared before)
        assert_eq!(
            ok_json(&["events-hash", vs])["events_hash"],
            h0["events_hash"]
        );
        let k = Kernel::open(&v).unwrap();
        let st = k.call("status", &json!({}), human()).unwrap();
        assert_eq!(st["index"]["state"], "ok");
        assert_eq!(st["index"]["format"], st["index"]["expected_format"]);
        let r = search(&k, "istanbul").unwrap();
        let n = r["hits"].as_array().unwrap().len();
        // uni_fold_tr folds "İSTANBUL" to the query's term; the default does not
        assert_eq!(n, usize::from(to.is_some()), "{r:#}");
    }
}

#[test]
fn a_default_vault_s_status_is_unchanged() {
    let v = vault("default");
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["index"]["format"], rootmarkindex::store::FORMAT);
    assert_eq!(st["index"]["expected_format"], rootmarkindex::store::FORMAT);
    assert_eq!(st["index"]["state"], "ok");
}

#[test]
fn an_unknown_tokenizer_is_refused() {
    let v = vault("unknown");
    std::fs::write(v.join(".rootmark/policy.toml"), recipe(Some("turkish"))).unwrap();
    let o = rootmark(&["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("[index] tokenizer"), "{err}");
}
