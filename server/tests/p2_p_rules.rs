//! P2-P/P1 (GOAL P2-P 2; P-2, P-3): declarative promotion rules
//! `[[promotion.rules]]`.
//! - P-2: eight invalid recipes (three actor kinds of S2, a wrong `to`, a
//!   repeated `id`, a missing `version`, a `path_prefix` with `..`, an
//!   absolute one) are refused at load, the message names the field;
//!   `rootmark policy lint` exits 1 and `rootmark serve` does not start;
//! - P-3: a matching agent's `propose` promotes each lineage it wrote by
//!   one `Promoted` (`meta.actor = policy:<rule>`, `meta.reason =
//!   "rule:<rule>"`, `meta.policy_version`, `from`, `to`), the response
//!   carries the events, `history` shows the rule; the first matching rule
//!   wins (two overlapping rules); a recipe without rules answers as before.
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

/// The default recipe with `[promotion] version` (if any) and `rules`
/// (`[[promotion.rules]]` tables) appended.
pub fn recipe(version: Option<&str>, rules: &str) -> String {
    let base = rootmarkserver::config::default_recipe();
    let base = match version {
        Some(v) => base.replace(
            "[promotion]\npolicy = \"human_only\"\n",
            &format!("[promotion]\npolicy = \"human_only\"\nversion = \"{v}\"\n"),
        ),
        None => base,
    };
    format!("{base}\n[identity]\nhuman_id = \"ada\"\n{rules}")
}

fn rule(id: &str, actor: &str, source: Option<&str>, prefix: &str) -> String {
    let src = source.map_or(String::new(), |s| format!("source = \"{s}\"\n"));
    format!("\n[[promotion.rules]]\nid = \"{id}\"\nactor = \"{actor}\"\n{src}path_prefix = \"{prefix}\"\nto = \"canonical\"\n")
}

fn vault(name: &str, recipe: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2p1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(v.join("notes")).unwrap();
    std::fs::write(v.join("notes/human.md"), "# Human\n\nA human note.\n").unwrap();
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    v
}

fn agent(name: &'static str) -> Caller<'static> {
    Caller::mcp(Some(name))
}

fn propose(k: &Kernel, who: &'static str, doc: &str, text: &str) -> Value {
    k.call(
        "propose",
        &json!({"doc_path": doc, "content": text, "reason": "collect"}),
        agent(who),
    )
    .unwrap()
}

fn layers(k: &Kernel, doc: &str) -> Vec<String> {
    let b = k
        .call("blame", &json!({"doc_path": doc}), agent("reader"))
        .unwrap();
    b["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["layer"].as_str().unwrap().to_string())
        .collect()
}

const DOC: &str = "# Kur\n\nTCMB politika faizi.\n\nİkinci paragraf.\n";

/// P-2: eight invalid recipes, each refused with its field named.
#[test]
fn eight_invalid_recipes_are_refused_naming_the_field() {
    let ok = rule("r", "agent:collector", None, "sources");
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "unattested_fs",
            recipe(Some("1"), &rule("r", "unattested_fs", None, "sources")),
            "actor",
        ),
        (
            "human",
            recipe(Some("1"), &rule("r", "human:ada", None, "sources")),
            "actor",
        ),
        (
            "policy",
            recipe(Some("1"), &rule("r", "policy:other", None, "sources")),
            "actor",
        ),
        (
            "to",
            recipe(
                Some("1"),
                &ok.replace("to = \"canonical\"", "to = \"proposal\""),
            ),
            "to",
        ),
        ("dup", recipe(Some("1"), &format!("{ok}{ok}")), "id"),
        ("version", recipe(None, &ok), "version"),
        (
            "dotdot",
            recipe(
                Some("1"),
                &rule("r", "agent:collector", None, "sources/../x"),
            ),
            "path_prefix",
        ),
        (
            "absolute",
            recipe(Some("1"), &rule("r", "agent:collector", None, "/etc")),
            "path_prefix",
        ),
    ];
    assert_eq!(cases.len(), 8);
    for (name, text, field) in cases {
        let e = rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::Builtin)
            .expect_err(name);
        let msg = format!("{e:#}");
        assert!(msg.contains(field), "{name}: {msg}");
        let v = vault(&format!("bad-{name}"), &text);
        let vs = v.to_str().unwrap();
        let lint = rootmark(&["policy", "lint", vs]);
        assert_eq!(lint.status.code(), Some(1), "{name}: lint");
        let err = String::from_utf8_lossy(&lint.stderr);
        assert!(err.contains(field), "{name}: {err}");
        let serve = rootmark(&["serve", "--vault", vs, "--mcp", "stdio"]);
        assert!(!serve.status.success(), "{name}: serve started");
        assert!(
            String::from_utf8_lossy(&serve.stderr).contains(field),
            "{name}"
        );
    }
}

/// More shapes the loader refuses (not among P-2's eight): an id outside
/// `[a-z0-9_-]{1,64}`, an empty version, an unknown field, the whole vault
/// as a prefix, an empty source.
#[test]
fn other_rule_shapes_are_refused() {
    let ok = rule("r", "agent:c", None, "s");
    for (text, field) in [
        (
            recipe(Some("1"), &ok.replace("id = \"r\"", "id = \"R!\"")),
            "id",
        ),
        (recipe(Some(""), &ok), "version"),
        (
            recipe(Some("1"), &ok.replace("to =", "when = \"now\"\nto =")),
            "when",
        ),
        (
            recipe(Some("1"), &rule("r", "agent:c", None, "/")),
            "path_prefix",
        ),
        (
            recipe(Some("1"), &rule("r", "agent:c", Some(""), "s")),
            "source",
        ),
        (recipe(Some("1"), &rule("r", "agent:", None, "s")), "actor"),
    ] {
        let e =
            rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::Builtin).unwrap_err();
        assert!(format!("{e:#}").contains(field), "{field}: {e:#}");
    }
}

/// `rootmark policy lint` on a valid recipe lists the rules (normalized).
#[test]
fn lint_lists_the_rules() {
    let text = recipe(
        Some("2026-10-25"),
        &(rule(
            "a",
            "agent:collector",
            Some("collector"),
            "sources/official/",
        ) + &rule("b", "agent:writer", None, "drafts")),
    );
    let v = vault("lint", &text);
    let o = rootmark(&["policy", "lint", v.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["ok"], true);
    assert_eq!(r["promotion"], "human_only");
    assert_eq!(r["promotion_version"], "2026-10-25");
    assert_eq!(r["rules"][0]["id"], "a");
    assert_eq!(r["rules"][0]["path_prefix"], "sources/official");
    assert_eq!(
        r["rules"][0]["actor"],
        json!({"kind": "agent", "id": "collector"})
    );
    assert_eq!(r["rules"][1]["source"], Value::Null);
    // and a recipe without rules
    let plain = vault("lint-plain", &recipe(None, ""));
    let o = rootmark(&["policy", "lint", plain.to_str().unwrap()]);
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["rules"], json!([]));
    assert_eq!(r["promotion_version"], Value::Null);
}

/// P-3: the four fields, the response's events, `history`, first match.
#[test]
fn a_matching_propose_is_promoted_by_the_first_rule() {
    // two overlapping rules: the broader one first
    let text = recipe(
        Some("v7"),
        &(rule("broad", "agent:collector", None, "sources")
            + &rule(
                "official",
                "agent:collector",
                Some("collector"),
                "sources/official",
            )),
    );
    let v = vault("match", &text);
    let k = Kernel::open(&v).unwrap();
    let r = propose(&k, "collector", "sources/official/kur.md", DOC);
    let promoted = r["promoted"].as_array().unwrap();
    assert_eq!(promoted.len(), 3, "{r}");
    // ingest events + one Promoted per lineage
    let ingest: u64 = rootmarkcore::events::read_events(&v.join(".rootmark/events.jsonl"))
        .unwrap()
        .iter()
        .filter(|e| {
            e.doc_path == "sources/official/kur.md" && e.kind != rootmarkcore::EventKind::Promoted
        })
        .count() as u64;
    assert_eq!(r["events"].as_u64().unwrap(), ingest + 3);
    let mut ids: Vec<&str> = promoted
        .iter()
        .map(|e| e["lineage_ids"][0].as_str().unwrap())
        .collect();
    let sorted = {
        let mut s = ids.clone();
        s.sort();
        s
    };
    assert_eq!(ids, sorted, "lineage-id order");
    for e in promoted {
        assert_eq!(e["kind"], "Promoted");
        let m = &e["meta"];
        assert_eq!(m["actor"], json!({"kind": "policy", "id": "broad"}));
        assert_eq!(m["reason"], "rule:broad");
        assert_eq!(m["policy_version"], "v7");
        assert_eq!(m["policy"], "rules");
        assert_eq!(m["from"], "proposal");
        assert_eq!(m["to"], "canonical");
        assert!(m.get("source").is_none());
    }
    assert_eq!(layers(&k, "sources/official/kur.md"), ["canonical"; 3]);
    // the response's events_hash is the log's
    let st = rootmark(&["events-hash", v.to_str().unwrap()]);
    assert!(String::from_utf8_lossy(&st.stdout).contains(r["events_hash"].as_str().unwrap()));
    // history shows the rule through Promoted.meta
    let l = ids.remove(0);
    let h = k
        .call("history", &json!({"lineage_id": l}), agent("reader"))
        .unwrap();
    let last = h["events"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last, promoted[0]);
    assert_eq!(h["layer"], "canonical");
    // the vault reopens (replay checks every Promoted)
    rootmarkcore::Vault::open(&v).unwrap();
}

/// The first rule in recipe order wins; a rule whose source does not match
/// is passed over.
#[test]
fn rule_order_and_source_decide() {
    let text = recipe(
        Some("1"),
        &(rule("by-source", "agent:collector", Some("other"), "sources")
            + &rule("narrow", "agent:collector", None, "sources/official")
            + &rule("broad", "agent:collector", None, "sources")),
    );
    let v = vault("order", &text);
    let k = Kernel::open(&v).unwrap();
    let r = propose(&k, "collector", "sources/official/a.md", "# A\n\nmetin.\n");
    for e in r["promoted"].as_array().unwrap() {
        assert_eq!(e["meta"]["reason"], "rule:narrow");
    }
    let r = propose(&k, "collector", "sources/b.md", "# B\n\nmetin.\n");
    for e in r["promoted"].as_array().unwrap() {
        assert_eq!(e["meta"]["reason"], "rule:broad");
    }
    // another agent, another path: nothing
    let r = propose(&k, "writer", "sources/c.md", "# C\n\nmetin.\n");
    assert!(r.get("promoted").is_none(), "{r}");
    assert_eq!(layers(&k, "sources/c.md"), ["proposal"; 2]);
    let r = propose(&k, "collector", "drafts/d.md", "# D\n\nmetin.\n");
    assert!(r.get("promoted").is_none(), "{r}");
    // a prefix is a path component boundary: `sources-x/` is not under `sources`
    let r = propose(&k, "collector", "sources-x/e.md", "# E\n\nmetin.\n");
    assert!(r.get("promoted").is_none(), "{r}");
}

/// An append promotes only the lineages it wrote; earlier proposals of the
/// document (written before the rule existed) stay proposals.
#[test]
fn only_the_call_s_own_lineages_are_promoted() {
    let v = vault("own", &recipe(None, ""));
    let k = Kernel::open(&v).unwrap();
    propose(&k, "collector", "sources/x.md", "# X\n\nönceki öneri.\n");
    assert_eq!(layers(&k, "sources/x.md"), ["proposal"; 2]);
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        recipe(Some("1"), &rule("r", "agent:collector", None, "sources")),
    )
    .unwrap();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": "sources/x.md", "content": "## Yeni\n\nyeni metin.\n",
                    "reason": "more", "mode": "append"}),
            agent("collector"),
        )
        .unwrap();
    assert_eq!(r["promoted"].as_array().unwrap().len(), 2, "{r}");
    assert_eq!(
        layers(&k, "sources/x.md"),
        ["proposal", "proposal", "canonical", "canonical"]
    );
}

/// Without rules the response, the log and the layers are as before (no
/// `promoted` field; the events count is the ingest's).
#[test]
fn without_rules_nothing_changes() {
    let mk = |name: &str, text: String| {
        let v = vault(name, &text);
        let k = Kernel::open(&v).unwrap();
        let r = propose(&k, "collector", "sources/official/kur.md", DOC);
        (v, r)
    };
    let (va, a) = mk("plain", recipe(None, ""));
    // a version alone (no rules) is allowed and changes nothing
    let (vb, b) = mk(
        "version-only",
        recipe(None, "").replace(
            "[promotion]\npolicy = \"human_only\"\n",
            "[promotion]\npolicy = \"human_only\"\nversion = \"9\"\n",
        ),
    );
    for (v, r) in [(&va, &a), (&vb, &b)] {
        assert!(r.get("promoted").is_none(), "{r}");
        let k = Kernel::open(v).unwrap();
        assert_eq!(layers(&k, "sources/official/kur.md"), ["proposal"; 3]);
    }
    assert_eq!(a["events_hash"], b["events_hash"]);
    assert_eq!(a["events"], b["events"]);
    // the status of a recipe without rules has no rule field
    let k = Kernel::open(&va).unwrap();
    let st = k.call("status", &json!({}), agent("reader")).unwrap();
    assert!(st["policy"].get("promotion_rules").is_none());
}

/// The kernel's reading rule (P2-P/P1): a policy actor's `Promoted` names
/// its rule (`reason` "rule:<id>"); another reason is refused when the
/// vault opens.
#[test]
fn a_policy_promotion_without_a_rule_reason_is_refused_on_open() {
    let text = recipe(Some("1"), &rule("r", "agent:collector", None, "sources"));
    let v = vault("reading", &text);
    let k = Kernel::open(&v).unwrap();
    propose(&k, "collector", "sources/y.md", "# Y\n\nmetin.\n");
    let p = v.join(".rootmark/events.jsonl");
    let log = std::fs::read_to_string(&p).unwrap();
    assert!(log.contains("\"reason\":\"rule:r\""));
    for bad in ["\"reason\":\"auto\"", "\"reason\":\"rule:\""] {
        std::fs::write(&p, log.replace("\"reason\":\"rule:r\"", bad)).unwrap();
        let e = rootmarkcore::Vault::open(&v).err().expect(bad);
        assert!(format!("{e:#}").contains("rule:<id>"), "{bad}: {e:#}");
    }
    std::fs::write(&p, &log).unwrap();
    rootmarkcore::Vault::open(&v).unwrap();
}

/// `serve`'s status shows the rules of a recipe that has them.
#[test]
fn status_shows_the_rules() {
    let text = recipe(Some("v1"), &rule("r", "agent:collector", None, "sources/"));
    let v = vault("status", &text);
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), agent("reader")).unwrap();
    assert_eq!(st["policy"]["promotion_rules"]["version"], "v1");
    assert_eq!(
        st["policy"]["promotion_rules"]["rules"][0]["path_prefix"],
        "sources"
    );
    assert_eq!(
        st["policy"]["promotion_rules"]["rules"][0]["actor"],
        "agent:collector"
    );
    let _ = Path::new("");
}
