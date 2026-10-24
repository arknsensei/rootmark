//! P2-P/P0 (GOAL P2-P 1; P-1, P-9):
//! - (b) K-P2L-T1, option (a): `rootmark migrate`'s `policy_attestation_1`
//!   appends `[attestation] unattested = "canonical"` to a recipe without
//!   the table (P2-0's three `v0.1.0` vaults: the recipe 0.1.0's `serve`
//!   wrote), every other byte kept, with a rollback record; `--dry-run`
//!   plans it; it is recommended, never `migration_required`; a tableless
//!   recipe is reported `implicit` and `serve` warns; a 0.2.0 recipe (with
//!   the table) and a vault without a recipe are left alone;
//! - (a2) the rejection paths of the P2-L/L0 b fix (a copy is no rename, a
//!   client's hint alone names no base, a lower-layer base raises nothing);
//! - (a4) a trace whose filter line cannot follow it is not left behind,
//!   and `status.migrations` names an error instead of an empty list.
//!
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
use rootmarkserver::error::{code_of, Code};
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

fn ok_json(args: &[&str]) -> Value {
    let o = rootmark(args);
    assert!(
        o.status.success(),
        "rootmark {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2p0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

/// One of P2-0's three `v0.1.0` vaults (`research/data/p2_0_old_vaults`),
/// its store as `.rootmark/` (as P2-L's L-8 measurer brings it).
fn old_vault(tag: &str, name: &str) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../research/data/p2_0_old_vaults")
        .join(name);
    let v = scratch(&format!("old-{tag}-{name}")).join("vault");
    copy_tree(&src.join("vault"), &v);
    copy_tree(&src.join("store"), &v.join(".rootmark"));
    v
}

fn status(v: &Path) -> Value {
    Kernel::open(v)
        .unwrap()
        .call("status", &json!({}), Caller::http("p2-p-p0"))
        .unwrap()
}

fn records(v: &Path) -> Vec<String> {
    let d = v.join(".rootmark/migrations");
    if !d.is_dir() {
        return Vec::new();
    }
    let mut ids: Vec<String> = std::fs::read_dir(d)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    ids.sort();
    ids
}

const TABLE: &str = "[attestation]\nunattested = \"canonical\"\n";

/// P-1: the three vaults 0.1.0's recipe set up.
#[test]
fn policy_attestation_1_on_the_three_0_1_0_vaults() {
    for name in ["e2e", "mixed", "scene"] {
        let v = old_vault("p1", name);
        let vs = v.to_str().unwrap();
        let pol = v.join(".rootmark/policy.toml");
        let before = std::fs::read(&pol).unwrap();
        assert!(!String::from_utf8_lossy(&before).contains("[attestation]"));
        // tableless: implicit `unverified`, recommended, never required
        let st = status(&v);
        assert_eq!(st["attestation"]["unattested"], "unverified", "{name}");
        assert_eq!(st["attestation"]["implicit"], true, "{name}");
        let m = &st["migrations"];
        assert_eq!(
            m["artifacts"]["policy"]["format"], "recipe/implicit",
            "{name}"
        );
        assert_eq!(m["artifacts"]["policy"]["state"], "ok", "{name}");
        assert_eq!(m["recommended"], json!(["policy_attestation_1"]), "{name}");
        assert!(
            !m["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x == "policy_attestation_1"),
            "{name}: {m}"
        );
        for (k, a) in m["artifacts"].as_object().unwrap() {
            assert_ne!(a["state"], "migration_required", "{name}: {k}");
        }
        let hash0 = ok_json(&["migrate", "--dry-run", vs])["events_hash"].clone();
        // --dry-run plans it and changes nothing
        let dry = ok_json(&["migrate", "--dry-run", vs]);
        let plan: Vec<&Value> = dry["plan"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["migration"] == "policy_attestation_1")
            .collect();
        assert_eq!(plan.len(), 1, "{name}: {dry}");
        assert_eq!(plan[0]["recommended"], true);
        assert_eq!(plan[0]["touches"], json!([".rootmark/policy.toml"]));
        assert_eq!(std::fs::read(&pol).unwrap(), before, "{name}: dry run");
        // migrate: the table appended, the rest byte for byte
        let mig = ok_json(&["migrate", vs]);
        assert!(
            mig["applied"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["migration"] == "policy_attestation_1"),
            "{name}: {mig}"
        );
        let after = std::fs::read(&pol).unwrap();
        assert!(after.starts_with(&before), "{name}: the old bytes are kept");
        let mut want = before.clone();
        if !want.ends_with(b"\n") {
            want.push(b'\n');
        }
        want.push(b'\n');
        want.extend_from_slice(TABLE.as_bytes());
        assert_eq!(after, want, "{name}: only the table is added");
        assert_eq!(mig["events_hash"], hash0, "{name}");
        let st = status(&v);
        assert_eq!(st["attestation"]["unattested"], "canonical", "{name}");
        assert_eq!(st["attestation"]["implicit"], false, "{name}");
        assert_eq!(st["migrations"]["recommended"], json!([]), "{name}");
        assert_eq!(
            st["migrations"]["artifacts"]["policy"]["format"],
            "recipe/attestation"
        );
        // a second run does nothing to the recipe
        let again = ok_json(&["migrate", vs]);
        assert!(
            !again["applied"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["migration"] == "policy_attestation_1"),
            "{name}"
        );
        // --rollback, newest first, down to the recipe's record
        let ids = records(&v);
        let pid = ids
            .iter()
            .find(|i| i.ends_with("-policy_attestation_1"))
            .unwrap()
            .clone();
        for id in ids.iter().rev() {
            let r = ok_json(&["migrate", "--rollback", id, vs]);
            assert_eq!(r["byte_equal"], true, "{name}: {id}");
            if *id == pid {
                break;
            }
        }
        assert_eq!(std::fs::read(&pol).unwrap(), before, "{name}: rolled back");
        assert_eq!(status(&v)["attestation"]["implicit"], true);
        assert_eq!(ok_json(&["migrate", "--dry-run", vs])["events_hash"], hash0);
    }
}

/// A recipe the migration left can be rolled back only as it left it.
#[test]
fn a_recipe_changed_after_the_migration_is_not_rolled_back() {
    let v = old_vault("changed", "scene");
    let vs = v.to_str().unwrap();
    ok_json(&["migrate", vs]);
    let pid = records(&v)
        .into_iter()
        .find(|i| i.ends_with("-policy_attestation_1"))
        .unwrap();
    // the validations recorded after it go first
    for id in records(&v).iter().rev() {
        if *id == pid {
            break;
        }
        ok_json(&["migrate", "--rollback", id, vs]);
    }
    let pol = v.join(".rootmark/policy.toml");
    let edited = format!(
        "{}# the user's note\n",
        std::fs::read_to_string(&pol).unwrap()
    );
    std::fs::write(&pol, &edited).unwrap();
    let o = rootmark(&["migrate", "--rollback", &pid, vs]);
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("changed since migration"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(std::fs::read_to_string(&pol).unwrap(), edited);
}

/// A 0.2.0 recipe (with the table) and a vault without a recipe: no-op.
#[test]
fn a_recipe_with_the_table_and_no_recipe_are_left_alone() {
    // the recipe 0.2.0's `serve` writes
    let v = old_vault("noop", "mixed");
    let vs = v.to_str().unwrap();
    let pol = v.join(".rootmark/policy.toml");
    std::fs::write(&pol, rootmarkserver::config::default_recipe()).unwrap();
    let before = std::fs::read(&pol).unwrap();
    let st = status(&v);
    assert_eq!(st["attestation"]["implicit"], false);
    assert_eq!(st["attestation"]["unattested"], "canonical");
    assert_eq!(st["migrations"]["recommended"], json!([]));
    let dry = ok_json(&["migrate", "--dry-run", vs]);
    assert!(!dry["plan"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["migration"] == "policy_attestation_1"));
    ok_json(&["migrate", vs]);
    assert_eq!(std::fs::read(&pol).unwrap(), before);
    assert!(!records(&v)
        .iter()
        .any(|i| i.ends_with("-policy_attestation_1")));
    // a table that only sets the window is a table: left alone
    let w = old_vault("window", "e2e");
    let wp = w.join(".rootmark/policy.toml");
    let text = format!(
        "{}\n[attestation]\nwindow_s = 3.0\n",
        std::fs::read_to_string(&wp).unwrap()
    );
    std::fs::write(&wp, &text).unwrap();
    ok_json(&["migrate", w.to_str().unwrap()]);
    assert_eq!(std::fs::read_to_string(&wp).unwrap(), text);
    // no recipe at all
    let n = old_vault("none", "e2e");
    std::fs::remove_file(n.join(".rootmark/policy.toml")).unwrap();
    let st = status(&n);
    assert_eq!(
        st["migrations"]["artifacts"]["policy"]["note"],
        "no policy.toml"
    );
    assert_eq!(st["migrations"]["recommended"], json!([]));
    ok_json(&["migrate", n.to_str().unwrap()]);
    assert!(!n.join(".rootmark/policy.toml").exists());
}

/// `serve` warns at start when the recipe has no `[attestation]` table.
#[test]
fn serve_warns_about_a_tableless_recipe() {
    let v = old_vault("serve", "scene");
    let o = rootmark(&["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("warning:") && err.contains("no [attestation] table"),
        "{err}"
    );
    // with the table: no warning
    ok_json(&["migrate", v.to_str().unwrap()]);
    let o = rootmark(&["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"]);
    assert!(!String::from_utf8_lossy(&o.stderr).contains("no [attestation] table"));
}

/// A recipe that is not TOML is an error in `status.migrations`, not "no table".
#[test]
fn a_broken_recipe_or_record_is_an_error_in_status_migrations() {
    let v = old_vault("broken", "mixed");
    let vs = v.to_str().unwrap();
    ok_json(&["migrate", vs]);
    // a rollback record this reader cannot read (P2-P/P0 a4: no longer an
    // empty `applied`/`required`)
    let rec = v.join(".rootmark/migrations/9999-broken");
    std::fs::create_dir_all(&rec).unwrap();
    std::fs::write(rec.join("manifest.json"), "{").unwrap();
    let st = status(&v);
    let e = st["migrations"]["error"].as_str().unwrap_or_default();
    assert!(e.contains("not a migration record"), "{st}");
    assert!(st["migrations"].get("required").is_none());
}

// ---- a4: the trace filter ---------------------------------------------------------

#[test]
fn a_filtered_trace_is_not_written_without_its_filter() {
    let root = scratch("trace");
    let v = root.join("vault");
    std::fs::create_dir_all(v.join("a")).unwrap();
    std::fs::write(v.join("a/x.md"), "# X\n\nkahve demlemek\n").unwrap();
    std::fs::write(v.join("y.md"), "# Y\n\nkahve içmek\n").unwrap();
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe(),
    )
    .unwrap();
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let q = json!({"query": "kahve", "mode": "lexical", "path_prefix": ["a"]});
    let c = Caller::http("p2-p-p0");
    k.call("search", &q, c).unwrap();
    let traces = v.join(".rootmark/traces.jsonl");
    let filters = v.join(".rootmark/trace_filters.jsonl");
    let (t0, f0) = (
        std::fs::read(&traces).unwrap(),
        std::fs::read(&filters).unwrap(),
    );
    // the trace line cannot be written: the filter line is taken back
    std::fs::rename(&traces, root.join("traces.keep")).unwrap();
    std::fs::create_dir(&traces).unwrap();
    assert!(k.call("search", &q, c).is_err());
    assert_eq!(
        std::fs::read(&filters).unwrap(),
        f0,
        "no filter left behind"
    );
    std::fs::remove_dir(&traces).unwrap();
    std::fs::rename(root.join("traces.keep"), &traces).unwrap();
    assert_eq!(std::fs::read(&traces).unwrap(), t0);
    // the log still reads, and the next search records both
    let r = k.call("search", &q, c).unwrap();
    let all = k.call("trace", &json!({}), c).unwrap();
    let last = all["traces"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["trace_id"], r["trace_id"]);
    assert_eq!(last["filter"], json!({"path_prefix": ["a"]}));
}

// ---- a2: the rejection paths of the P2-L/L0 b fix ---------------------------------

fn att_vault(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let v = scratch(&format!("att-{name}")).join("vault");
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    let recipe = format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nunattested = \"unverified\"\nwindow_s = 2.0\n",
        rootmarkserver::config::base_recipe()
    );
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn human() -> Caller<'static> {
    Caller::http("p2-p-test")
}

fn layer_of(k: &Kernel, doc: &str, needle: &str) -> String {
    let bl = k.call("blame", &json!({"doc_path": doc}), human()).unwrap();
    for b in bl["blocks"].as_array().unwrap() {
        let t = k
            .call(
                "read_block",
                &json!({"doc_path": doc, "lineage_id": b["lineage_id"]}),
                human(),
            )
            .unwrap();
        if t["block"]["block"]["text"]
            .as_str()
            .unwrap()
            .contains(needle)
        {
            return b["layer"].as_str().unwrap().to_string();
        }
    }
    panic!("no block with {needle:?} in {doc}")
}

struct Session {
    id: String,
    key: Vec<u8>,
}

fn session(k: &Kernel) -> Session {
    let s = k.call("attest_session", &json!({}), human()).unwrap();
    let key = s["key"].as_str().unwrap();
    Session {
        id: s["session_id"].as_str().unwrap().to_string(),
        key: (0..key.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&key[i..i + 2], 16).unwrap())
            .collect(),
    }
}

fn attest(k: &Kernel, s: &Session, v: &Path, doc: &str, extra: Value) -> anyhow::Result<Value> {
    let bytes = std::fs::read(v.join(doc)).unwrap();
    let h = rootmarkcore::attest::content_hash(&bytes);
    let mut body = json!({"doc_path": doc, "content_hash": h, "session_id": s.id,
                "mac": mac(&s.key, &mac_message(doc, &h, None))});
    for (key, val) in extra.as_object().unwrap() {
        body[key] = val.clone();
    }
    k.call("attest", &body, human())
}

fn watch(k: &Kernel, doc: &str) {
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
}

fn mv(v: &Path, from: &str, to: &str) {
    std::fs::create_dir_all(v.join(to).parent().unwrap()).unwrap();
    std::fs::rename(v.join(from), v.join(to)).unwrap();
}

const SELF_PLAN: &str = "# Plan\n\nBu not kendine bağlanır: [[plan]]\n\nBir paragraf.\n";

/// P2-L/L0 b's scene up to edit 30: `plan.md` moved to `proje/plan-son.md`
/// (its own link rewritten), a block appended there — attested
/// (`canonical`) or not (`unverified`).
fn moved_with_block(name: &str, attested: bool) -> (PathBuf, Kernel, Session) {
    let v = att_vault(name, &[("plan.md", SELF_PLAN)]);
    let k = Kernel::open(&v).unwrap();
    let s = session(&k);
    mv(&v, "plan.md", "proje/plan-son.md");
    attest(
        &k,
        &s,
        &v,
        "proje/plan-son.md",
        json!({"renamed_from": "plan.md"}),
    )
    .unwrap();
    let t = std::fs::read_to_string(v.join("proje/plan-son.md")).unwrap();
    std::fs::write(
        v.join("proje/plan-son.md"),
        t.replace("[[plan]]", "[[plan-son]]"),
    )
    .unwrap();
    attest(
        &k,
        &s,
        &v,
        "proje/plan-son.md",
        json!({"link_update_of": "proje/plan-son.md", "link_update_from": "plan.md"}),
    )
    .unwrap();
    watch(&k, "proje/plan-son.md");
    let t = std::fs::read_to_string(v.join("proje/plan-son.md")).unwrap();
    std::fs::write(
        v.join("proje/plan-son.md"),
        format!("{t}\nSon plan notu.\n"),
    )
    .unwrap();
    if attested {
        attest(&k, &s, &v, "proje/plan-son.md", json!({})).unwrap();
    }
    watch(&k, "proje/plan-son.md");
    let want = if attested { "canonical" } else { "unverified" };
    assert_eq!(layer_of(&k, "proje/plan-son.md", "Son plan"), want);
    (v, k, s)
}

fn self_back(k: &Kernel, s: &Session, v: &Path) -> anyhow::Result<Value> {
    let t = std::fs::read_to_string(v.join("plan.md")).unwrap();
    std::fs::write(v.join("plan.md"), t.replace("[[plan-son]]", "[[plan]]")).unwrap();
    attest(
        k,
        s,
        v,
        "plan.md",
        json!({"link_update_of": "plan.md", "link_update_from": "proje/plan-son.md"}),
    )
}

/// A copy back onto the earlier path is no rename: the old path's file is
/// still there, so its head is never the base (the earlier head is, and the
/// rewrite is `denied`).
#[test]
fn a2_a_copy_back_is_no_base() {
    let (v, k, s) = moved_with_block("copy", true);
    std::fs::copy(v.join("proje/plan-son.md"), v.join("plan.md")).unwrap();
    let _ = attest(
        &k,
        &s,
        &v,
        "plan.md",
        json!({"renamed_from": "proje/plan-son.md"}),
    );
    let e = self_back(&k, &s, &v).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied, "{e:#}");
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "Son plan"), "unverified");
}

/// The client's `link_update_from` alone (no accepted rename attestation)
/// names no fallback base.
#[test]
fn a2_a_hint_alone_is_no_base() {
    let (v, k, s) = moved_with_block("hint", true);
    mv(&v, "proje/plan-son.md", "plan.md");
    let e = self_back(&k, &s, &v).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied, "{e:#}");
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "Son plan"), "unverified");
}

/// A base whose block is `unverified` carries it over as it is: the
/// fallback raises nothing.
#[test]
fn a2_a_lower_layer_base_raises_nothing() {
    let (v, k, s) = moved_with_block("lower", false);
    mv(&v, "proje/plan-son.md", "plan.md");
    attest(
        &k,
        &s,
        &v,
        "plan.md",
        json!({"renamed_from": "proje/plan-son.md"}),
    )
    .unwrap();
    self_back(&k, &s, &v).unwrap();
    watch(&k, "plan.md");
    assert_eq!(layer_of(&k, "plan.md", "Son plan"), "unverified");
    for b in ["# Plan", "kendine", "Bir paragraf"] {
        assert_eq!(layer_of(&k, "plan.md", b), "canonical", "{b}");
    }
}
