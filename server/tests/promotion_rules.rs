//! P2-P/P2 (GOAL P2-P 3; P-4, P-5): the pre-registered security cases of
//! the declarative promotion rules.
//! - S1: an unattested file write that matches a rule's prefix and source is
//!   never promoted (the watcher's ingest, under `unverified` and under
//!   `canonical`): the matcher runs only in `propose`;
//! - S2: a rule naming `unattested_fs`, `human:…` or `policy:…` is refused
//!   at load (also P-2);
//! - S3: the matching actor's `propose` outside the prefix is not promoted;
//! - S4: an `unverified` lineage is never promoted by a rule, nor one that
//!   a late attestation returned to `proposal`;
//! - S5: `append_only` decides first: a `propose` it refuses writes nothing
//!   and promotes nothing;
//! - S6: without rules, C3-06 and the 39 conformance tests are unchanged
//!   (`rootmark conformance`) and so is `events_hash` (P2-0's three
//!   `v0.1.0` vaults; the same calls under a recipe without rules and one
//!   with a non-matching rule); S6b: with a rule, the default (`canonical`)
//!   ranking changes only by the documents under the rule's prefix;
//! - S7: the rule id and `policy_version` are in `history`; after a human
//!   `demote` the lineage is a `proposal`, out of default search.
//!
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
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

fn recipe(attestation: &str, rules: &str, version: bool) -> String {
    let base = rootmarkserver::config::base_recipe();
    let base = if version {
        base.replace(
            "[promotion]\npolicy = \"human_only\"\n",
            "[promotion]\npolicy = \"human_only\"\nversion = \"s-1\"\n",
        )
    } else {
        base
    };
    format!("{base}\n[identity]\nhuman_id = \"ada\"\n\n[attestation]\nunattested = \"{attestation}\"\nwindow_s = 2.0\n{rules}")
}

fn rule(id: &str, actor: &str, source: Option<&str>, prefix: &str) -> String {
    let src = source.map_or(String::new(), |s| format!("source = \"{s}\"\n"));
    format!("\n[[promotion.rules]]\nid = \"{id}\"\nactor = \"{actor}\"\n{src}path_prefix = \"{prefix}\"\nto = \"canonical\"\n")
}

/// The collector's rule (P4's scene).
fn collector() -> String {
    rule(
        "official",
        "agent:collector",
        Some("collector"),
        "sources/official/",
    )
}

fn vault(name: &str, recipe: &str, files: &[(&str, &str)]) -> (PathBuf, Kernel) {
    let root = std::env::temp_dir().join(format!("rootmark-p2p2-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(v.join(".rootmark/policy.toml"), recipe).unwrap();
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    (v, k)
}

fn agent(name: &'static str) -> Caller<'static> {
    Caller::mcp(Some(name))
}

fn human() -> Caller<'static> {
    Caller::http("p2-p-security")
}

fn propose(k: &Kernel, who: &'static str, doc: &str, text: &str) -> anyhow::Result<Value> {
    k.call(
        "propose",
        &json!({"doc_path": doc, "content": text, "reason": "collect"}),
        agent(who),
    )
}

fn events(v: &Path) -> Vec<rootmarkcore::Event> {
    rootmarkcore::events::read_events(&v.join(".rootmark/events.jsonl")).unwrap()
}

fn promoted(v: &Path) -> usize {
    events(v)
        .iter()
        .filter(|e| e.kind == rootmarkcore::EventKind::Promoted)
        .count()
}

fn blame(k: &Kernel, doc: &str) -> Vec<(String, String)> {
    let b = k
        .call("blame", &json!({"doc_path": doc}), agent("reader"))
        .unwrap();
    b["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["lineage_id"].as_str().unwrap().to_string(),
                b["layer"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn layers(k: &Kernel, doc: &str) -> Vec<String> {
    blame(k, doc).into_iter().map(|(_, l)| l).collect()
}

fn search(k: &Kernel, q: &str, mode: &str) -> Vec<String> {
    let r = k
        .call(
            "search",
            &json!({"query": q, "mode": mode, "k": 20}),
            agent("reader"),
        )
        .unwrap();
    r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["doc_path"].as_str().unwrap().to_string())
        .collect()
}

const NOTE: &str = "# Resmi kaynak\n\nTCMB politika faizini sabit tuttu.\n";

// ---- S1 ---------------------------------------------------------------------------

/// S1: an unattested file write under the rule's prefix, with the rule's
/// source as its source, is never promoted — under `unverified` and under
/// `canonical` (where it is the human's `canonical` content anyway, but by
/// no `Promoted`).
#[test]
fn s1_an_unattested_file_write_is_never_promoted() {
    for policy in ["unverified", "canonical"] {
        let (v, k) = vault(
            &format!("s1-{policy}"),
            &recipe(policy, &collector(), true),
            &[("notes/a.md", "# A\n\nbir not.\n")],
        );
        let doc = "sources/official/direct.md";
        std::fs::create_dir_all(v.join("sources/official")).unwrap();
        std::fs::write(v.join(doc), NOTE).unwrap();
        // the watcher's ingest; its source as the rule's (and as the agent's name)
        k.watch_doc(doc, "collector").unwrap();
        let want = if policy == "unverified" {
            "unverified"
        } else {
            "canonical"
        };
        assert_eq!(layers(&k, doc), [want, want], "{policy}");
        assert_eq!(promoted(&v), 0, "{policy}: a Promoted was written");
        // a later edit of the same file, and an append by the file: still none
        std::fs::write(v.join(doc), format!("{NOTE}\n## Ek\n\nek metin.\n")).unwrap();
        k.watch_doc(doc, "collector").unwrap();
        assert_eq!(promoted(&v), 0, "{policy}");
        // the agent's own propose next to it is promoted (the rule works)
        let r = propose(&k, "collector", "sources/official/p.md", NOTE).unwrap();
        assert_eq!(r["promoted"].as_array().unwrap().len(), 2, "{policy}");
        // and the unattested lineages are left as they were
        assert_eq!(layers(&k, doc), [want; 4], "{policy}");
    }
}

// ---- S2 ---------------------------------------------------------------------------

/// S2: a rule for `unattested_fs`, a human or a policy is refused at load.
#[test]
fn s2_a_rule_for_another_actor_kind_is_refused() {
    for actor in [
        "unattested_fs",
        "human:ada",
        "policy:rules",
        "unattested_fs:fs",
    ] {
        let text = recipe("unverified", &rule("x", actor, None, "sources"), true);
        let e = rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::Builtin)
            .expect_err(actor);
        assert!(format!("{e:#}").contains("actor"), "{actor}: {e:#}");
    }
}

// ---- S3 ---------------------------------------------------------------------------

/// S3: the matching actor's `propose` outside the prefix is not promoted.
#[test]
fn s3_outside_the_prefix_is_not_promoted() {
    let (v, k) = vault(
        "s3",
        &recipe("canonical", &collector(), true),
        &[("notes/a.md", "# A\n\nbir not.\n")],
    );
    for doc in [
        "sources/other/x.md",
        "sources/official-x/y.md",
        "sources/z.md",
        "official/sources/official/w.md",
    ] {
        let r = propose(&k, "collector", doc, NOTE).unwrap();
        assert!(r.get("promoted").is_none(), "{doc}: {r}");
        assert_eq!(layers(&k, doc), ["proposal"; 2], "{doc}");
    }
    // the right source but another agent
    let r = propose(&k, "collector2", "sources/official/q.md", NOTE).unwrap();
    assert!(r.get("promoted").is_none());
    assert_eq!(promoted(&v), 0);
}

// ---- S4 ---------------------------------------------------------------------------

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

fn attest(k: &Kernel, s: &Session, v: &Path, doc: &str) -> Value {
    let bytes = std::fs::read(v.join(doc)).unwrap();
    let h = rootmarkcore::attest::content_hash(&bytes);
    k.call(
        "attest",
        &json!({"doc_path": doc, "content_hash": h, "session_id": s.id,
                "mac": mac(&s.key, &mac_message(doc, &h, None))}),
        human(),
    )
    .unwrap()
}

/// S4: an `unverified` lineage under the prefix is never promoted by a rule
/// — neither by a later `propose` of the matching agent into the same
/// document, nor after a late attestation returned it to `proposal` (the
/// rule looks only at the lineages its own call wrote).
#[test]
fn s4_unverified_and_late_attested_lineages_are_never_promoted() {
    let doc = "sources/official/s4.md";
    // another agent's proposal there (no rule for it), then a human edit of
    // that proposal, unattested: the lineage drops to `unverified`
    let (v, k) = vault(
        "s4",
        &recipe("unverified", &collector(), true),
        &[("notes/a.md", "# A\n\nbir not.\n")],
    );
    propose(&k, "other", doc, "# Taslak\n\nilk taslak paragrafı.\n").unwrap();
    assert_eq!(layers(&k, doc), ["proposal"; 2]);
    std::fs::write(
        v.join(doc),
        "# Taslak\n\nilk taslak paragrafı, düzeltildi.\n",
    )
    .unwrap();
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
    let after_edit = layers(&k, doc);
    assert!(
        after_edit.contains(&"unverified".to_string()),
        "{after_edit:?}"
    );
    let unverified: Vec<String> = blame(&k, doc)
        .into_iter()
        .filter(|(_, l)| l == "unverified")
        .map(|(id, _)| id)
        .collect();
    // the matching agent appends: only its own lineages are promoted
    let r = k
        .call(
            "propose",
            &json!({"doc_path": doc, "content": "## Resmi\n\nresmi ek.\n",
                    "reason": "collect", "mode": "append"}),
            agent("collector"),
        )
        .unwrap();
    let ids: Vec<&str> = r["promoted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["lineage_ids"][0].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 2, "{r}");
    for u in &unverified {
        assert!(
            !ids.contains(&u.as_str()),
            "an unverified lineage was promoted"
        );
    }
    for u in &unverified {
        let l = blame(&k, doc)
            .into_iter()
            .find(|(id, _)| id == u)
            .unwrap()
            .1;
        assert_eq!(l, "unverified");
    }
    let before = promoted(&v);
    // a late attestation returns the edited proposal to `proposal`, and no
    // rule promotes it afterwards either
    let (v2, k2) = vault(
        "s4-late",
        &recipe("unverified", &collector(), true),
        &[("notes/a.md", "# A\n\nbir not.\n")],
    );
    let s2 = session(&k2);
    propose(&k2, "other", doc, "# Taslak\n\nilk taslak paragrafı.\n").unwrap();
    std::fs::write(
        v2.join(doc),
        "# Taslak\n\nilk taslak paragrafı, düzeltildi.\n",
    )
    .unwrap();
    k2.watch_doc(doc, "rootmark-serve-watch").unwrap();
    let a = attest(&k2, &s2, &v2, doc);
    assert!(!a["raised"].as_array().unwrap().is_empty(), "{a}");
    let returned: Vec<(String, String)> = blame(&k2, doc);
    assert!(
        returned.iter().all(|(_, l)| l == "proposal"),
        "a late attestation returns the touched proposal to proposal: {returned:?}"
    );
    assert_eq!(promoted(&v2), 0, "a late attestation runs no rule");
    // the collector's append promotes its own two lineages only
    let r = k2
        .call(
            "propose",
            &json!({"doc_path": doc, "content": "## Resmi\n\nresmi ek.\n",
                    "reason": "collect", "mode": "append"}),
            agent("collector"),
        )
        .unwrap();
    assert_eq!(r["promoted"].as_array().unwrap().len(), 2);
    let l2 = layers(&k2, doc);
    assert_eq!(
        l2,
        ["proposal", "proposal", "canonical", "canonical"],
        "{l2:?}"
    );
    assert_eq!(promoted(&v), before);
}

// ---- S5 ---------------------------------------------------------------------------

/// S5: `append_only` decides before any rule: a refused `propose` writes
/// nothing — no file change, no event, no promotion.
#[test]
fn s5_a_refused_propose_promotes_nothing() {
    let doc = "sources/official/human.md";
    let (v, k) = vault(
        "s5",
        &recipe("canonical", &collector(), true),
        &[(doc, "# İnsan\n\ninsanın paragrafı.\n")],
    );
    let file0 = std::fs::read(v.join(doc)).unwrap();
    let log0 = std::fs::read(v.join(".rootmark/events.jsonl")).unwrap();
    // replace the human's section; append into the human's chunk
    let e = k
        .call(
            "propose",
            &json!({"doc_path": doc, "content": "ajan metni\n", "reason": "x",
                    "mode": "replace_section", "heading": "İnsan"}),
            agent("collector"),
        )
        .unwrap_err();
    assert_eq!(
        rootmarkserver::error::code_of(&e),
        rootmarkserver::error::Code::Denied
    );
    let e = k
        .call(
            "propose",
            &json!({"doc_path": doc, "content": "ajan metni\n", "reason": "x", "mode": "append"}),
            agent("collector"),
        )
        .unwrap_err();
    assert_eq!(
        rootmarkserver::error::code_of(&e),
        rootmarkserver::error::Code::PolicyViolation
    );
    assert_eq!(std::fs::read(v.join(doc)).unwrap(), file0);
    assert_eq!(
        std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
        log0
    );
    assert_eq!(promoted(&v), 0);
    assert_eq!(layers(&k, doc), ["canonical"; 2]);
}

// ---- S6 ---------------------------------------------------------------------------

/// S6: `rootmark conformance` (no recipe in its scratch vaults): 39/39,
/// C3-06 passes with its committed hashes.
#[test]
fn s6_conformance_and_c3_06_unchanged() {
    let o = rootmark(&["conformance"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["passed"], 39);
    assert_eq!(r["total"], 39);
    let c306 = r["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "C3-06")
        .unwrap();
    assert_eq!(c306["result"], "pass");
    assert_eq!(c306["actual_hash"], c306["expected_hash"]);
}

/// S6: `events_hash` of P2-0's three `v0.1.0` vaults (recipe without rules)
/// as their `expected.json` says, read by this build; and the same calls
/// under a recipe without rules and one whose rule matches nothing give the
/// same log.
#[test]
fn s6_events_hash_unchanged_without_rules() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/data/p2_0_old_vaults");
    for name in ["e2e", "mixed", "scene"] {
        let want: Value = serde_json::from_str(
            &std::fs::read_to_string(data.join(name).join("expected.json")).unwrap(),
        )
        .unwrap();
        // the committed store is opened in a copy
        let root =
            std::env::temp_dir().join(format!("rootmark-p2p2-{}-s6-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let copy = |from: &Path, to: &Path| {
            let o = Command::new("cp")
                .arg("-R")
                .arg(from)
                .arg(to)
                .output()
                .unwrap();
            assert!(o.status.success());
        };
        copy(&data.join(name).join("vault"), &root);
        copy(&data.join(name).join("store"), &root.join(".rootmark"));
        let vt = rootmarkcore::Vault::open(&root).unwrap();
        assert_eq!(
            json!(vt.events_hash().unwrap()),
            want["events_hash"],
            "{name}"
        );
    }
    let run = |name: &str, rules: &str| {
        let (_, k) = vault(
            name,
            &recipe("canonical", rules, !rules.is_empty()),
            &[("notes/a.md", "# A\n\nbir not.\n")],
        );
        let mut hashes = Vec::new();
        for (who, doc) in [
            ("collector", "sources/official/a.md"),
            ("writer", "drafts/b.md"),
        ] {
            let r = propose(&k, who, doc, NOTE).unwrap();
            assert!(r.get("promoted").is_none());
            hashes.push(r["events_hash"].clone());
        }
        hashes
    };
    let none = run("s6-none", "");
    let other = run(
        "s6-other",
        &rule("elsewhere", "agent:nobody", None, "nowhere"),
    );
    assert_eq!(none, other);
}

const CORPUS: [(&str, &str); 4] = [
    (
        "notes/faiz.md",
        "# Faiz\n\nMerkez bankası politika faizini açıkladı.\n\nEnflasyon beklentisi yüksek.\n",
    ),
    (
        "notes/kur.md",
        "# Kur\n\nDolar kuru ve faiz ilişkisi tartışıldı.\n",
    ),
    (
        "notes/borsa.md",
        "# Borsa\n\nBorsa endeksi faiz kararından sonra yükseldi.\n",
    ),
    ("notes/hava.md", "# Hava\n\nYarın yağmur bekleniyor.\n"),
];

/// S6b: the same `propose` calls with and without the rule; the default
/// (`canonical`) results differ only by documents under the rule's prefix:
/// with them taken out, the rule vault's ranking is the other's, in every
/// mode.
#[test]
fn s6b_the_canonical_ranking_changes_only_under_the_prefix() {
    let run = |name: &str, rules: &str| {
        let (_, k) = vault(
            name,
            &recipe("canonical", rules, !rules.is_empty()),
            &CORPUS,
        );
        for (who, doc, text) in [
            (
                "collector",
                "sources/official/karar.md",
                "# Karar\n\nPolitika faizi yüzde elli sabit.\n",
            ),
            (
                "collector",
                "sources/official/rapor.md",
                "# Rapor\n\nEnflasyon raporu faiz patikasını anlattı.\n",
            ),
            (
                "collector",
                "sources/blog/yorum.md",
                "# Yorum\n\nFaiz kararı piyasayı şaşırttı.\n",
            ),
            (
                "writer",
                "sources/official/taslak.md",
                "# Taslak\n\nFaiz notu taslağı.\n",
            ),
        ] {
            propose(&k, who, doc, text).unwrap();
        }
        k
    };
    let plain = run("s6b-plain", "");
    let ruled = run("s6b-rule", &collector());
    let under = |d: &str| d.starts_with("sources/official/");
    let mut changed = 0;
    for mode in ["lexical", "flat", "hybrid"] {
        for q in ["faiz", "politika faizi", "enflasyon", "kur", "yağmur"] {
            let a = search(&plain, q, mode);
            let b = search(&ruled, q, mode);
            assert!(a.iter().all(|d| !under(d)), "{mode} {q}: {a:?}");
            let b_out: Vec<String> = b.iter().filter(|d| !under(d)).cloned().collect();
            assert_eq!(a, b_out, "{mode} {q:?}: outside the prefix");
            // what the rule added is the collector's official documents only
            for d in b.iter().filter(|d| under(d)) {
                assert!(
                    d == "sources/official/karar.md" || d == "sources/official/rapor.md",
                    "{mode} {q}: {d}"
                );
            }
            changed += usize::from(a != b);
        }
    }
    assert!(changed > 0, "the rule changes some canonical ranking");
}

// ---- S7 ---------------------------------------------------------------------------

/// S7: `history` shows the rule and the policy version; a human's `demote`
/// (HTTP) makes the lineage a `proposal` again, out of default search.
#[test]
fn s7_history_shows_the_rule_and_demote_undoes_it() {
    let (v, k) = vault(
        "s7",
        &recipe("canonical", &collector(), true),
        &[("notes/a.md", "# A\n\nbaşka bir konu.\n")],
    );
    let doc = "sources/official/s7.md";
    let r = propose(&k, "collector", doc, NOTE).unwrap();
    assert_eq!(r["promoted"].as_array().unwrap().len(), 2);
    assert!(search(&k, "politika faizini", "lexical").contains(&doc.to_string()));
    let (para, _) = blame(&k, doc).remove(1);
    let h = k
        .call("history", &json!({"lineage_id": para}), agent("reader"))
        .unwrap();
    let p = h["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] == "Promoted")
        .unwrap()
        .clone();
    assert_eq!(p["meta"]["actor"]["id"], "official");
    assert_eq!(p["meta"]["reason"], "rule:official");
    assert_eq!(p["meta"]["policy_version"], "s-1");
    // an agent cannot demote (HTTP-only endpoint; MCP does not serve it)
    let e = k
        .call(
            "demote",
            &json!({"lineage_id": para, "reason": "x"}),
            agent("collector"),
        )
        .unwrap_err();
    assert_eq!(
        rootmarkserver::error::code_of(&e),
        rootmarkserver::error::Code::NotFound
    );
    // the human demotes both blocks of the chunk
    for (l, _) in blame(&k, doc) {
        let d = k
            .call(
                "demote",
                &json!({"lineage_id": l, "reason": "kaynak yanlış"}),
                human(),
            )
            .unwrap();
        assert_eq!(d["decision"], "demoted", "{d}");
        assert_eq!(d["layer"], "proposal");
        let m = &d["event"]["meta"];
        assert_eq!(m["actor"], json!({"kind": "human", "id": "ada"}));
        assert_eq!(m["reason"], "kaynak yanlış");
        assert_eq!(m["from"], "canonical");
        assert_eq!(m["to"], "proposal");
    }
    assert_eq!(layers(&k, doc), ["proposal"; 2]);
    assert!(!search(&k, "politika faizini", "lexical").contains(&doc.to_string()));
    let h = k
        .call("history", &json!({"lineage_id": para}), agent("reader"))
        .unwrap();
    let kinds: Vec<&str> = h["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.last(), Some(&"Demoted"));
    assert!(kinds.contains(&"Promoted"));
    // `include_proposal` still finds it, labelled
    let r = k
        .call(
            "search",
            &json!({"query": "politika faizini", "mode": "lexical", "include_proposal": true}),
            agent("reader"),
        )
        .unwrap();
    assert!(r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|h| h["doc_path"] == doc && h["layer"] == "proposal"));
    // the vault reopens
    rootmarkcore::Vault::open(&v).unwrap();
}

/// The demote endpoint follows the promotion policy: `deny` refuses it as
/// a decision (no event); an unknown lineage is `not_found`. Rules are not
/// loaded under `deny` (their promotions could not be demoted; P5 review 2,
/// K-P2P.5).
#[test]
fn demote_follows_the_promotion_policy() {
    let deny = |rules: &str, version: bool| {
        recipe("canonical", rules, version).replace("policy = \"human_only\"", "policy = \"deny\"")
    };
    let e = rootmarkcore::Policy::parse(
        &deny(&collector(), true),
        rootmarkcore::PolicySource::Builtin,
    )
    .unwrap_err();
    assert!(format!("{e:#}").contains("deny"), "{e:#}");
    let (v, k) = vault(
        "demote-deny",
        &deny("", false),
        &[("notes/a.md", "# A\n\nbir not.\n")],
    );
    // a canonical human block: deny refuses its demotion
    let (l, _) = blame(&k, "notes/a.md").remove(0);
    let log0 = std::fs::read(v.join(".rootmark/events.jsonl")).unwrap();
    let d = k
        .call("demote", &json!({"lineage_id": l, "reason": "x"}), human())
        .unwrap();
    assert_eq!(d["decision"], "denied");
    assert_eq!(
        std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
        log0
    );
    let e = k
        .call(
            "demote",
            &json!({"lineage_id": "0".repeat(32), "reason": "x"}),
            human(),
        )
        .unwrap_err();
    assert_eq!(
        rootmarkserver::error::code_of(&e),
        rootmarkserver::error::Code::NotFound
    );
}

/// P5 review 1: a new block whose text repeats one already in the document
/// (same version, new lineage) is the call's own and is promoted.
#[test]
fn a_repeated_text_block_of_the_call_is_promoted() {
    let doc = "sources/official/tekrar.md";
    let (_, k) = vault(
        "repeat",
        &recipe("canonical", &collector(), true),
        &[("notes/a.md", "# A\n\nbir not.\n")],
    );
    // another agent's proposal with the same section first (no rule for it)
    propose(&k, "other", doc, "# Taslak\n\n## Özet\n\naynı metin.\n").unwrap();
    let old: Vec<String> = blame(&k, doc).into_iter().map(|(l, _)| l).collect();
    let r = k
        .call(
            "propose",
            &json!({"doc_path": doc, "content": "## Özet\n\naynı metin.\n",
                    "reason": "collect", "mode": "append"}),
            agent("collector"),
        )
        .unwrap();
    // the two lineages the call created (the lineage matcher decides which
    // of the repeated blocks keeps the old lineage), and only they
    let promoted: Vec<&str> = r["promoted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["lineage_ids"][0].as_str().unwrap())
        .collect();
    assert_eq!(promoted.len(), 2, "{r}");
    assert!(promoted.iter().all(|l| !old.iter().any(|o| o == l)));
    let now = blame(&k, doc);
    assert_eq!(now.len(), 5);
    for (l, layer) in now {
        let want = if old.contains(&l) {
            "proposal"
        } else {
            "canonical"
        };
        assert_eq!(layer, want, "{l}");
    }
}
