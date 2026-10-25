//! P2-B/B0 (GOAL P2-B 1; B-1): Block 1's leftovers.
//! - (a) Block 1 verification, recommendation 1: a search `path_prefix`
//!   written in NFD is folded to NFC (a read), a rule's `path_prefix` in
//!   NFD is refused at load (an authority; K-P2B.1);
//! - (b) recommendation 3: the `--dry-run` plan of `policy_attestation_1`
//!   says `recommended: true` and no `required` (review finding 7);
//! - (c) the P2-P reviewer's findings: (5) the reading rule ties a policy
//!   `Promoted`'s `rule:<id>` to `meta.actor.id` and to `meta.from =
//!   "proposal"`; (8) a recipe this reader cannot read is an error of
//!   `status.migrations` (no silent `unknown`); (11) an external change not
//!   yet ingested, then a matching `propose`: the external blocks are the
//!   watcher's (never promoted), only the agent's are.
//!
//! Needs the model files (`make models`).

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

fn recipe(attestation: &str, rules: &str) -> String {
    let base = rootmarkserver::config::base_recipe().replace(
        "[promotion]\npolicy = \"human_only\"\n",
        "[promotion]\npolicy = \"human_only\"\nversion = \"b0-1\"\n",
    );
    format!("{base}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nunattested = \"{attestation}\"\nwindow_s = 2.0\n{rules}")
}

fn rule(id: &str, prefix: &str) -> String {
    format!("\n[[promotion.rules]]\nid = \"{id}\"\nactor = \"agent:collector\"\nsource = \"collector\"\npath_prefix = \"{prefix}\"\nto = \"canonical\"\n")
}

fn root(name: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("rootmark-p2b0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    r
}

fn vault(name: &str, recipe: Option<&str>, files: &[(&str, &str)]) -> PathBuf {
    let v = root(name).join("vault");
    std::fs::create_dir_all(&v).unwrap();
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    if let Some(r) = recipe {
        std::fs::write(v.join(".rootmark/policy.toml"), r).unwrap();
    }
    v
}

fn ok_json(args: &[&str]) -> Value {
    let o = rootmark(args);
    assert!(
        o.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

// ---- (a) NFD: search folds, a rule refuses -------------------------------------------

/// A search `path_prefix` in NFD (`c` + U+0327) is the NFC prefix (`ç`):
/// the same hits, the filter shown (and traced) in NFC.
#[test]
fn a_search_prefix_in_nfd_is_folded_to_nfc() {
    let v = vault(
        "nfd-search",
        Some(&recipe("canonical", "")),
        &[
            (
                "çay/demleme.md",
                "# Demleme\n\nÇay demlenirken su kaynamaz.\n",
            ),
            (
                "kahve/demleme.md",
                "# Demleme\n\nKahve demlenirken su kaynar.\n",
            ),
        ],
    );
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let search = |prefix: &str| {
        k.call(
            "search",
            &json!({"query": "demlenirken su", "mode": "lexical", "path_prefix": [prefix]}),
            Caller::http("p2-b-b0"),
        )
        .unwrap()
    };
    let nfc = search("çay");
    let nfd = search("c\u{327}ay");
    assert_eq!(nfd["filter"]["path_prefix"], json!(["çay"]));
    assert_eq!(nfd["hits"], nfc["hits"]);
    assert_eq!(nfd["query_hash"], nfc["query_hash"]);
    let docs: Vec<&str> = nfd["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["doc_path"].as_str().unwrap())
        .collect();
    assert_eq!(docs, ["çay/demleme.md"]);
    // the trace carries the NFC filter
    let t = k
        .call("trace", &json!({}), Caller::http("p2-b-b0"))
        .unwrap();
    let last = t["traces"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["filter"]["path_prefix"], json!(["çay"]));
}

/// A rule's `path_prefix` in NFD is refused at load (the recipe is an
/// authority, written once in NFC): `serve`'s loader and `policy lint`.
#[test]
fn a_rule_prefix_in_nfd_is_refused() {
    let text = recipe("canonical", &rule("r", "c\u{327}ay/"));
    let e = rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::Builtin).unwrap_err();
    assert!(format!("{e:#}").contains("NFC"), "{e:#}");
    let v = vault("nfd-rule", Some(&text), &[("a.md", "# A\n\nmetin.\n")]);
    let o = rootmark(&["policy", "lint", v.to_str().unwrap()]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("NFC"));
    // the NFC spelling loads
    rootmarkcore::Policy::parse(
        &recipe("canonical", &rule("r", "çay/")),
        rootmarkcore::PolicySource::Builtin,
    )
    .unwrap();
}

// ---- (b) the dry-run plan --------------------------------------------------------------

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

/// `policy_attestation_1` in the `--dry-run` plan: `recommended: true`,
/// no `required` key (P2-P review finding 7); `status.migrations` agrees.
#[test]
fn the_dry_run_plan_of_a_recommended_migration_is_not_required() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/data/p2_0_old_vaults/e2e");
    let v = root("dry-run").join("vault");
    copy_dir(&src.join("vault"), &v);
    copy_dir(&src.join("store"), &v.join(".rootmark"));
    let vs = v.to_str().unwrap();
    let dry = ok_json(&["migrate", "--dry-run", vs]);
    let plan = dry["plan"].as_array().unwrap();
    let pa: Vec<&Value> = plan
        .iter()
        .filter(|p| p["migration"] == "policy_attestation_1")
        .collect();
    assert_eq!(pa.len(), 1, "{dry}");
    assert_eq!(pa[0]["recommended"], true);
    assert_eq!(pa[0]["required"], false, "{}", pa[0]);
    // the other plans (validations) keep their `required` as before
    for p in plan
        .iter()
        .filter(|p| p["migration"] != "policy_attestation_1")
    {
        assert_eq!(p["required"], false, "{p}");
        assert!(p.get("recommended").is_none(), "{p}");
    }
    assert_eq!(
        dry["matrix"]["recommended"],
        json!(["policy_attestation_1"])
    );
    assert_eq!(dry["matrix"]["required"], json!([]));
}

// ---- (c) the reviewer's findings 5, 8, 11 ----------------------------------------------

/// Finding 5: a policy `Promoted` whose `reason` names another rule than
/// its actor, or whose `from` is not `proposal`, is refused when the vault
/// opens; the log as written opens.
#[test]
fn a_policy_promotion_of_another_rule_or_layer_is_refused_on_open() {
    let text = recipe("canonical", &rule("official", "sources"));
    let v = vault("reading", Some(&text), &[("notes/h.md", "# H\n\nİnsan.\n")]);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "sources/y.md", "content": "# Y\n\nmetin.\n", "reason": "collect"}),
            Caller::mcp(Some("collector")),
        )
        .unwrap();
    assert_eq!(r["promoted"].as_array().unwrap().len(), 2, "{r}");
    let p = v.join(".rootmark/events.jsonl");
    let log = std::fs::read_to_string(&p).unwrap();
    assert!(log.contains("\"reason\":\"rule:official\""));
    assert!(log.contains("\"from\":\"proposal\""));
    for (from, to, says) in [
        (
            "\"reason\":\"rule:official\"",
            "\"reason\":\"rule:other\"",
            "does not name meta.actor.id",
        ),
        (
            "\"from\":\"proposal\"",
            "\"from\":\"unverified\"",
            "comes from proposal",
        ),
    ] {
        std::fs::write(&p, log.replace(from, to)).unwrap();
        let e = rootmarkcore::Vault::open(&v).err().expect(to);
        assert!(format!("{e:#}").contains(says), "{to}: {e:#}");
    }
    std::fs::write(&p, &log).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
}

/// Finding 8: a recipe that is not TOML is an error of the version matrix
/// (`status.migrations.error`; `migrate --dry-run` fails naming the file),
/// never an `unknown` row.
#[test]
fn an_unreadable_recipe_is_an_error_of_the_matrix() {
    let v = vault(
        "unreadable",
        Some("[promotion\n"),
        &[("a.md", "# A\n\nmetin.\n")],
    );
    let e = rootmarkserver::migrate::matrix(&v).unwrap_err();
    assert!(format!("{e:#}").contains("policy.toml"), "{e:#}");
    let o = rootmark(&["migrate", "--dry-run", v.to_str().unwrap()]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("policy.toml"));
    // a readable recipe's row is `ok`
    std::fs::write(v.join(".rootmark/policy.toml"), recipe("canonical", "")).unwrap();
    let m = rootmarkserver::migrate::matrix(&v).unwrap();
    assert_eq!(m["artifacts"]["policy"]["state"], "ok");
}

/// Finding 11: the document under the rule's prefix is changed on disk and
/// not ingested (no watcher call), then the collector appends to it. The
/// external blocks are synced first as the watcher's (`unverified` under
/// `unverified`, the human's `canonical` under `canonical`) and none of
/// them is promoted; only the agent's new lineages are.
#[test]
fn an_external_change_before_a_matching_propose_is_never_promoted() {
    for attestation in ["unverified", "canonical"] {
        let text = recipe(attestation, &rule("official", "sources/official/"));
        let v = vault(
            &format!("sync-{attestation}"),
            Some(&text),
            &[("sources/official/kur.md", "# Kur\n\nİlk satır.\n")],
        );
        let k = Kernel::open(&v).unwrap();
        k.ensure_index().unwrap();
        // the external change: not seen by the server
        std::fs::write(
            v.join("sources/official/kur.md"),
            "# Kur\n\nİlk satır.\n\nDışarıdan eklenen paragraf.\n",
        )
        .unwrap();
        let r = k
            .call(
                "propose",
                &json!({
                    "doc_path": "sources/official/kur.md",
                    "content": "## Ek\n\nToplayıcının metni.\n",
                    "reason": "collect",
                    "mode": "append",
                }),
                Caller::mcp(Some("collector")),
            )
            .unwrap();
        assert!(r["synced_external_events"].as_u64().unwrap() > 0, "{r}");
        let promoted: Vec<String> = r["promoted"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|e| e["lineage_ids"].as_array().unwrap().clone())
            .map(|l| l.as_str().unwrap().to_string())
            .collect();
        let b = k
            .call(
                "blame",
                &json!({"doc_path": "sources/official/kur.md"}),
                Caller::mcp(Some("reader")),
            )
            .unwrap();
        let mut agent = 0;
        for blk in b["blocks"].as_array().unwrap() {
            let l = blk["lineage_id"].as_str().unwrap().to_string();
            if blk["author_kind"] == "Agent" {
                agent += 1;
                assert!(promoted.contains(&l), "{attestation}: {blk}");
                assert_eq!(blk["layer"], "canonical");
            } else {
                assert!(!promoted.contains(&l), "{attestation}: {blk}");
            }
        }
        assert_eq!(agent, 2, "{attestation}: {b}");
        assert_eq!(promoted.len(), 2, "{attestation}: {r}");
        // the external paragraph: the watcher's layer under the policy
        let want = if attestation == "unverified" {
            "unverified"
        } else {
            "canonical"
        };
        let ext_layers: Vec<&str> = b["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["author_kind"] != "Agent")
            .map(|x| x["layer"].as_str().unwrap())
            .collect();
        assert!(ext_layers.contains(&want), "{attestation}: {b}");
    }
}
