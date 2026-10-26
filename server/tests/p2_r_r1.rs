//! P2-R/R1 (GOAL P2-R 2; R-2, R-4): proposal decay by `[[prune.rules]]`.
//! - the recipe: `[prune] version`, `[[prune.rules]] {id, layer,
//!   idle_checkpoints, path_prefix}`; out-of-bound values, `canonical`, an
//!   NFD prefix, an unknown field, rules without a version: load errors;
//! - `rootmark prune <vault>`: a lineage of the rule's layer, under its
//!   prefixes, with at least `idle_checkpoints` checkpoints after its last
//!   event gets one `Demoted` with `meta.prune`, the layer kept,
//!   `policy:<id>`, `rule:<id>`, `policy_version`; `canonical` never;
//!   `unverified` only by a rule of that layer; without rules no event and
//!   `events_hash` unchanged; `--dry-run` writes nothing;
//! - the reading rule: a policy's prune names its own rule (`rule:<actor
//!   id>`), else the vault does not open;
//! - a pruned lineage: not in search (also with `include_proposal`),
//!   `read_block` `inactive: true`, at a checkpoint before the prune read
//!   and found as before, `history` shows the `Demoted`.
//!
//! Needs the model files (`make models`).

use rootmarkcore::{AuthorKind, Layer, MatcherVersion, Provenance, Vault};
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
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

fn err_text(args: &[&str]) -> String {
    let o = rootmark(args);
    assert!(!o.status.success(), "{args:?} succeeded");
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn root(name: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("rootmark-p2r1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(&r).unwrap();
    r
}

const RULES: &str = "\n[prune]\nversion = \"2026-10-26\"\n\n[[prune.rules]]\nid = \"stale\"\nlayer = \"proposal\"\nidle_checkpoints = 2\npath_prefix = [\"agents\"]\n";

const UNVERIFIED_RULE: &str =
    "\n[[prune.rules]]\nid = \"unv\"\nlayer = \"unverified\"\nidle_checkpoints = 3\n";

fn recipe(extra: &str) -> String {
    format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n{extra}",
        rootmarkserver::config::default_recipe()
    )
}

fn set_recipe(v: &Path, extra: &str) {
    std::fs::write(v.join(".rootmark/policy.toml"), recipe(extra)).unwrap();
}

fn prov(kind: AuthorKind, source: &str) -> Provenance {
    Provenance {
        author_kind: kind,
        source: source.into(),
    }
}

/// A content change (a human document's new version), then a checkpoint.
fn tick(v: &mut Vault, i: usize) -> String {
    v.ingest(
        "clock.md",
        &format!("# Clock\n\ntick {i}\n"),
        prov(AuthorKind::Human, "test"),
        MatcherVersion::V3,
    )
    .unwrap();
    v.checkpoint(None).unwrap().checkpoint_id
}

/// h.md (human, canonical), agents/a.md and other/b.md (agent proposals),
/// u.md (unverified); then `ticks` checkpoints, each after a change of
/// clock.md. Returns the vault and the checkpoint ids.
fn scene(name: &str, ticks: usize) -> (PathBuf, Vec<String>) {
    let r = root(name);
    let m = MatcherVersion::V3;
    let mut v = Vault::open(&r).unwrap();
    v.ingest(
        "h.md",
        "# Human\n\nA canonical note about espresso.\n",
        prov(AuthorKind::Human, "test"),
        m,
    )
    .unwrap();
    v.ingest(
        "agents/a.md",
        "# Agent plan\n\nThe agent proposes a zephyr roast curve.\n",
        prov(AuthorKind::Agent, "collector"),
        m,
    )
    .unwrap();
    v.ingest(
        "other/b.md",
        "# Other\n\nAnother proposal about quokka filters.\n",
        prov(AuthorKind::Agent, "collector"),
        m,
    )
    .unwrap();
    v.ingest_declared(
        "u.md",
        "# Unverified\n\nAn unattested wombat edit.\n",
        prov(AuthorKind::Human, "unattested_fs"),
        m,
        Some(Layer::Unverified),
    )
    .unwrap();
    let mut cps = Vec::new();
    for i in 0..ticks {
        cps.push(tick(&mut v, i));
    }
    drop(v);
    set_recipe(&r, "");
    (r, cps)
}

fn lineages(v: &Path, doc: &str) -> Vec<String> {
    ok_json(&["blame", v.to_str().unwrap(), doc])["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["lineage_id"].as_str().unwrap().to_string())
        .collect()
}

fn log(v: &Path) -> Vec<u8> {
    std::fs::read(v.join(".rootmark/events.jsonl")).unwrap()
}

#[test]
fn the_recipe_is_checked_on_load() {
    let (v, _) = scene("recipe", 0);
    let vs = v.to_str().unwrap();
    set_recipe(&v, RULES);
    let lint = ok_json(&["policy", "lint", vs]);
    assert_eq!(lint["ok"], true);
    let show = ok_json(&["policy", "show", vs]);
    assert_eq!(
        show["prune_rules"],
        json!({"version": "2026-10-26", "rules": [{"id": "stale", "layer": "proposal", "idle_checkpoints": 2, "path_prefix": ["agents"]}]})
    );
    // without rules the recipe's output is as before (no prune_rules key)
    set_recipe(&v, "");
    assert!(ok_json(&["policy", "show", vs])
        .get("prune_rules")
        .is_none());
    let bad: &[(&str, &str)] = &[
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 0\n",
            "the kernel minimum is 1",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = -3\n",
            "the kernel minimum is 1",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"canonical\"\nidle_checkpoints = 1\n",
            "never pruned",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"derived\"\nidle_checkpoints = 1\n",
            "must be \"proposal\" or \"unverified\"",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\npath_prefix = [\"c\u{0327}ay\"]\n",
            "NFC",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\npath_prefix = [\"../out\"]\n",
            "path_prefix",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\npath_prefix = []\n",
            "path_prefix is empty",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\nolder_than = 3\n",
            "unknown field `older_than`",
        ),
        (
            "\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\n",
            "missing field `version`",
        ),
        (
            "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\n[[prune.rules]]\nid = \"x\"\nlayer = \"unverified\"\nidle_checkpoints = 1\n",
            "given twice",
        ),
        (
            "\n[prune]\nversion = \"1\"\nwhen = \"daily\"\n",
            "unknown field `when`",
        ),
    ];
    for (extra, want) in bad {
        set_recipe(&v, extra);
        let e = err_text(&["policy", "lint", vs]);
        assert!(e.contains(want), "{extra}: {e}");
        // the prune refuses the recipe too, writing nothing
        let before = log(&v);
        let e = err_text(&["prune", vs]);
        assert!(e.contains(want), "{extra}: {e}");
        assert_eq!(log(&v), before);
    }
    // the NFC spelling of the same prefix loads
    set_recipe(&v, "\n[prune]\nversion = \"1\"\n[[prune.rules]]\nid = \"x\"\nlayer = \"proposal\"\nidle_checkpoints = 1\npath_prefix = [\"\u{e7}ay\"]\n");
    ok_json(&["policy", "lint", vs]);
}

#[test]
fn without_rules_no_event_is_written() {
    let (v, _) = scene("norules", 5);
    let vs = v.to_str().unwrap();
    let before = log(&v);
    let hash = ok_json(&["events-hash", vs])["events_hash"].clone();
    for args in [vec!["prune", vs, "--dry-run"], vec!["prune", vs]] {
        let out = ok_json(&args);
        assert_eq!(out["rules"]["pruned"], json!([]), "{out}");
        assert_eq!(out["rules"]["events"], json!([]), "{out}");
        assert_eq!(out["events_hash"], hash);
        assert_eq!(out["events_hash_before"], hash);
    }
    assert_eq!(log(&v), before);
}

#[test]
fn a_rule_prunes_idle_lineages_of_its_layer_and_prefix_only() {
    let (v, _) = scene("rules", 1);
    let vs = v.to_str().unwrap();
    set_recipe(&v, RULES);
    let agent = lineages(&v, "agents/a.md");
    // one checkpoint since the agent's write: not yet idle
    let dry = ok_json(&["prune", vs, "--dry-run"]);
    assert_eq!(dry["rules"]["pruned"], json!([]), "{dry}");
    {
        let mut vv = Vault::open(&v).unwrap();
        tick(&mut vv, 1);
    }
    // two: the dry run plans both lineages of agents/a.md and changes nothing
    let before = log(&v);
    let dry = ok_json(&["prune", vs, "--dry-run"]);
    assert_eq!(log(&v), before);
    let planned: Vec<&str> = dry["rules"]["pruned"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["lineage_id"].as_str().unwrap())
        .collect();
    let mut want: Vec<&str> = agent.iter().map(String::as_str).collect();
    want.sort();
    assert_eq!(planned, want, "{dry}");
    for d in dry["rules"]["pruned"].as_array().unwrap() {
        assert_eq!(d["doc_path"], "agents/a.md");
        assert_eq!(d["rule"], "stale");
        assert_eq!(d["layer"], "proposal");
        assert_eq!(d["idle_checkpoints"], 2);
    }
    assert_eq!(dry["dry_run"], true);
    assert_eq!(dry["events_hash"], dry["events_hash_before"]);
    // the prune
    let out = ok_json(&["prune", vs]);
    assert_eq!(out["rules"]["pruned"], dry["rules"]["pruned"]);
    assert_eq!(out["rules"]["events"].as_array().unwrap().len(), 2);
    assert_ne!(out["events_hash"], out["events_hash_before"]);
    // only Demoted events were appended, the old log is a prefix
    let after = log(&v);
    assert!(after.starts_with(&before));
    let added: Vec<Value> = std::str::from_utf8(&after[before.len()..])
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(added.len(), 2);
    for e in &added {
        assert_eq!(e["kind"], "Demoted");
        let m = &e["meta"];
        assert_eq!(m["prune"], true);
        assert_eq!(m["from"], "proposal");
        assert_eq!(m["to"], "proposal");
        assert_eq!(m["actor"], json!({"kind": "policy", "id": "stale"}));
        assert_eq!(m["reason"], "rule:stale");
        assert_eq!(m["policy"], "prune_rules");
        assert_eq!(m["policy_version"], "2026-10-26");
    }
    // the rest untouched: canonical, the other prefix, unverified
    for doc in ["h.md", "other/b.md", "u.md", "clock.md"] {
        let b = ok_json(&["blame", vs, doc]);
        for r in b["blocks"].as_array().unwrap() {
            assert_eq!(r["inactive"], false, "{doc}: {r}");
        }
    }
    for l in &agent {
        let h = ok_json(&["history", vs, l]);
        assert_eq!(h["layer"], "proposal");
        let last = h["events"].as_array().unwrap().last().unwrap().clone();
        assert_eq!(last["kind"], "Demoted");
        assert_eq!(last["meta"]["prune"], true);
    }
    // again: nothing left to prune
    let again = ok_json(&["prune", vs]);
    assert_eq!(again["rules"]["pruned"], json!([]));
    assert_eq!(log(&v), after);
    // unverified only by a rule of its layer
    set_recipe(&v, &format!("{RULES}{UNVERIFIED_RULE}"));
    assert_eq!(
        ok_json(&["prune", vs, "--dry-run"])["rules"]["pruned"],
        json!([])
    );
    {
        let mut vv = Vault::open(&v).unwrap();
        tick(&mut vv, 2);
    }
    let out = ok_json(&["prune", vs]);
    let docs: Vec<&str> = out["rules"]["pruned"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["doc_path"].as_str().unwrap())
        .collect();
    assert_eq!(docs, ["u.md", "u.md"], "{out}");
    for d in out["rules"]["pruned"].as_array().unwrap() {
        assert_eq!(d["layer"], "unverified");
        assert_eq!(d["rule"], "unv");
    }
    // canonical is never pruned, whatever the recipe says it can't say
    for doc in ["h.md", "clock.md"] {
        for r in ok_json(&["blame", vs, doc])["blocks"].as_array().unwrap() {
            assert_eq!(r["inactive"], false);
            assert_eq!(r["layer"], "canonical");
        }
    }
}

#[test]
fn the_kernel_refuses_a_rule_prune_of_canonical_or_of_another_rule() {
    let (v, _) = scene("hook", 0);
    let mut vv = Vault::open(&v).unwrap();
    let rule = rootmarkcore::PruneRule {
        id: "stale".into(),
        layer: Layer::Proposal,
        idle_checkpoints: 1,
        path_prefix: Vec::new(),
    };
    let hook = rootmarkserver::prune::PruneHook {
        rule: &rule,
        version: "1",
    };
    let policy = |id: &str| rootmarkcore::Actor::new(rootmarkcore::ActorKind::Policy, id).unwrap();
    let canon = lineages(&v, "h.md")[0].clone();
    let e = vv
        .prune_inactive(&canon, policy("stale"), "rule:stale", &hook)
        .unwrap_err();
    assert!(format!("{e:#}").contains("prune keeps a proposal"), "{e:#}");
    let prop = lineages(&v, "agents/a.md")[0].clone();
    let e = vv
        .prune_inactive(&prop, policy("stale"), "rule:other", &hook)
        .unwrap_err();
    assert!(format!("{e:#}").contains("denied"), "{e:#}");
    let unv = lineages(&v, "u.md")[0].clone();
    let e = vv
        .prune_inactive(&unv, policy("stale"), "rule:stale", &hook)
        .unwrap_err();
    assert!(format!("{e:#}").contains("denied"), "{e:#}");
    vv.prune_inactive(&prop, policy("stale"), "rule:stale", &hook)
        .unwrap();
}

#[test]
fn the_reading_rule_refuses_a_prune_that_does_not_name_its_rule() {
    let (v, _) = scene("read", 2);
    let vs = v.to_str().unwrap();
    set_recipe(&v, RULES);
    ok_json(&["prune", vs]);
    let good = log(&v);
    let text = String::from_utf8(good.clone()).unwrap();
    let last = text.lines().last().unwrap();
    for (from, to, want) in [
        (
            "\"reason\":\"rule:stale\"",
            "\"reason\":\"rule:other\"",
            "names its own rule",
        ),
        (
            "\"reason\":\"rule:stale\"",
            "\"reason\":\"stale\"",
            "names its own rule",
        ),
    ] {
        assert!(last.contains(from), "{last}");
        let bad = text.replace(last, &last.replace(from, to));
        std::fs::write(v.join(".rootmark/events.jsonl"), bad).unwrap();
        let e = Vault::open(&v).err().expect("opened a bad log");
        assert!(format!("{e:#}").contains(want), "{e:#}");
    }
    std::fs::write(v.join(".rootmark/events.jsonl"), &good).unwrap();
    Vault::open(&v).unwrap();
}

#[test]
fn a_pruned_proposal_is_read_from_a_checkpoint_before_the_prune() {
    let (v, cps) = scene("read-at", 2);
    let vs = v.to_str().unwrap();
    set_recipe(&v, RULES);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let c = Caller::http("p2-r-r1");
    let q = json!({"query": "zephyr roast curve", "include_proposal": true, "mode": "lexical"});
    let docs = |r: &Value| -> Vec<String> {
        r["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["doc_path"].as_str().unwrap().to_string())
            .collect()
    };
    let before = k.call("search", &q, c).unwrap();
    assert!(
        docs(&before).contains(&"agents/a.md".to_string()),
        "{before}"
    );
    let at = cps.last().unwrap().clone();
    let l = lineages(&v, "agents/a.md")[1].clone();
    let rb0 = k
        .call(
            "read_block",
            &json!({"doc_path": "agents/a.md", "lineage_id": l, "at": at}),
            c,
        )
        .unwrap();
    ok_json(&["prune", vs]);
    // not found by search any more (the proposal pass included)
    let after = k.call("search", &q, c).unwrap();
    assert!(
        !docs(&after).contains(&"agents/a.md".to_string()),
        "{after}"
    );
    // read_block at the head: inactive
    let rb = k
        .call(
            "read_block",
            &json!({"doc_path": "agents/a.md", "lineage_id": l}),
            c,
        )
        .unwrap();
    assert_eq!(rb["block"]["inactive"], true, "{rb}");
    assert_eq!(
        rb["block"]["block"]["normalized_text"],
        rb0["block"]["block"]["normalized_text"]
    );
    // at the checkpoint before the prune: the same answer as before
    let mut at_q = q.clone();
    at_q["at"] = json!(at);
    let then = k.call("search", &at_q, c).unwrap();
    assert!(docs(&then).contains(&"agents/a.md".to_string()), "{then}");
    assert_eq!(then["hits"], {
        let mut b = before.clone();
        b["hits"].take()
    });
    let rb1 = k
        .call(
            "read_block",
            &json!({"doc_path": "agents/a.md", "lineage_id": l, "at": at}),
            c,
        )
        .unwrap();
    assert_eq!(rb1["block"], rb0["block"]);
    assert_eq!(rb1["block"]["inactive"], false);
    // history shows the prune
    let h = k.call("history", &json!({"lineage_id": l}), c).unwrap();
    let last = h["events"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["kind"], "Demoted");
    assert_eq!(last["meta"]["prune"], true);
    assert_eq!(last["meta"]["reason"], "rule:stale");
}

#[test]
fn the_same_vault_and_recipe_prune_alike() {
    let (a, _) = scene("det-a", 3);
    let (b, _) = scene("det-b", 3);
    for v in [&a, &b] {
        set_recipe(v, &format!("{RULES}{UNVERIFIED_RULE}"));
    }
    let oa = rootmark(&["prune", a.to_str().unwrap(), "--now-ms", "0"]);
    let ob = rootmark(&["prune", b.to_str().unwrap(), "--now-ms", "0"]);
    assert!(oa.status.success() && ob.status.success());
    assert_eq!(oa.stdout, ob.stdout);
    assert_eq!(log(&a), log(&b));
}
