//! P2-A/A2 (GOAL P2-A A-3; CLIENT_CONTRACT §2.5, §3): the version matrix
//! and `migration_required`, on the real `/2` index of the `p1d-final`
//! binary (`server/tests/fixtures/index_v2`, renamed with
//! `rootmark migrate --rename-store`) with `[index] auto_rebuild = false`:
//! - `status` answers 200: `index.state = "stale"` (K-E1.2, unchanged) and
//!   `migrations.artifacts.index = {format: "mdkernel-index/2", expected,
//!   state: "migration_required"}`, `migrations.required =
//!   ["index_rebuild"]`, `migrations.hint` names `rootmark migrate`;
//! - `search` (and `propose` on the indexed vault) answer 503
//!   `migration_required` with a `hint`, not `index_stale`; read-only calls
//!   answer;
//! - after `rootmark migrate` every artifact is `ok`, `required` is empty
//!   and `search` answers;
//! - a format this rootmark does not know stays `index_stale`
//!   (`migrations.artifacts.index.state = "unknown"`).
//!
//! Needs the model files (`make models`).

use rootmarkserver::error::{code_of, error_json, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn rootmark(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(args)
        .output()
        .unwrap()
}

fn old_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2a2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_v2/vault");
    assert!(Command::new("cp")
        .args(["-R", src.to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    let v = root.join("vault");
    assert!(
        rootmark(&["migrate", "--rename-store", v.to_str().unwrap()])
            .status
            .success()
    );
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe()
            .replace("[index]\n", "[index]\nauto_rebuild = false\n")
            + "\n[identity]\nhuman_id = \"ada\"\n",
    )
    .unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-a-status")
}

#[test]
fn an_older_index_is_migration_required_until_migrated() {
    let v = old_vault("old");
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["index"]["state"], "stale");
    let ix = &st["migrations"]["artifacts"]["index"];
    assert_eq!(ix["format"], "mdkernel-index/2");
    assert_eq!(ix["expected"], rootmarkindex::store::FORMAT);
    assert_eq!(ix["state"], "migration_required");
    assert_eq!(st["migrations"]["required"], json!(["index_rebuild"]));
    assert!(st["migrations"]["hint"]
        .as_str()
        .unwrap()
        .contains("rootmark migrate"));
    for a in ["store", "events", "attestations", "checkpoints"] {
        assert_eq!(st["migrations"]["artifacts"][a]["state"], "ok", "{a}");
    }
    let e = k
        .call("search", &json!({"query": "extinguisher"}), human())
        .unwrap_err();
    assert_eq!(code_of(&e), Code::MigrationRequired);
    assert_eq!(code_of(&e).http_status(), 503);
    assert!(error_json(&e)["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("rootmark migrate"));
    let e = k
        .call(
            "propose",
            &json!({"doc_path": "new.md", "content": "# New\n\nAgent text.\n", "reason": "r"}),
            Caller::mcp(Some("p2a/1.0")),
        )
        .unwrap_err();
    assert_eq!(code_of(&e), Code::MigrationRequired);
    // read-only calls answer
    k.call("blame", &json!({"doc_path": "espresso.md"}), human())
        .unwrap();
    k.call("checkpoints", &json!({}), human()).unwrap();

    // migrated: everything ok, search answers
    let o = rootmark(&["migrate", v.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["index"]["state"], "ok");
    assert_eq!(st["migrations"]["required"], json!([]));
    assert_eq!(st["migrations"]["hint"], Value::Null);
    assert_eq!(st["migrations"]["artifacts"]["index"]["state"], "ok");
    k.call("search", &json!({"query": "extinguisher"}), human())
        .unwrap();
}

#[test]
fn an_unknown_format_stays_index_stale() {
    let v = old_vault("unknown");
    let o = rootmark(&["migrate", v.to_str().unwrap()]);
    assert!(o.status.success());
    let m = v.join(".rootmark/index/manifest.json");
    let t = std::fs::read_to_string(&m)
        .unwrap()
        .replace(rootmarkindex::store::FORMAT, "rootmark-index/99");
    std::fs::write(&m, t).unwrap();
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["migrations"]["artifacts"]["index"]["state"], "unknown");
    assert_eq!(st["migrations"]["required"], json!([]));
    let e = k
        .call("search", &json!({"query": "extinguisher"}), human())
        .unwrap_err();
    assert_eq!(code_of(&e), Code::IndexStale);
}
