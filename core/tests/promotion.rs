//! P1-B/B2: the promotion hook (spec §7) — `Promoted` requires actor,
//! reason and policy_version (C2-02), no promotion without a hook decision
//! (C2-03), demote/prune delete nothing (C2-04), B-6 promotion cases.

use rootmarkcore::{
    Actor, ActorKind, AuthorKind, Deny, EventKind, HumanOnly, Layer, MatcherVersion, Provenance,
    TransitionKind, Vault,
};
use std::fs;
use std::path::{Path, PathBuf};

const M: MatcherVersion = MatcherVersion::V3;
const DOC: &str = "# Plan\n\nAgent proposes to descale the machine every two weeks.\n\nAgent proposes a new grinder burr set for the bar.\n";

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-promo-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn agent() -> Provenance {
    Provenance {
        author_kind: AuthorKind::Agent,
        source: "claude-code".into(),
    }
}

fn human() -> Actor {
    Actor::new(ActorKind::Human, "alice").unwrap()
}

/// vault with one agent-written document; returns (vault, its lineages)
fn proposal_vault(root: &Path) -> (Vault, Vec<String>) {
    let mut v = Vault::open(root).unwrap();
    let r = v.ingest("plan.md", DOC, agent(), M).unwrap();
    let ls = r.snapshot.blocks.iter().map(|(l, _)| l.clone()).collect();
    (v, ls)
}

fn file_count(dir: &Path) -> usize {
    let mut n = 0;
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        n += if p.is_dir() { file_count(&p) } else { 1 };
    }
    n
}

fn err(r: anyhow::Result<impl std::fmt::Debug>) -> String {
    format!("{:#}", r.expect_err("must fail"))
}

#[test]
fn c2_02_promoted_carries_actor_reason_policy_version() {
    let root = tmp("c202");
    let (mut v, ls) = proposal_vault(&root);
    let req = v
        .request(
            TransitionKind::Promote,
            &ls[1],
            Layer::Canonical,
            human(),
            "reviewed",
        )
        .unwrap();
    let ev = v.promote(&req, &HumanOnly).unwrap();
    assert_eq!(ev.kind, EventKind::Promoted);
    assert_eq!(ev.meta["actor"]["kind"], "human");
    assert_eq!(ev.meta["actor"]["id"], "alice");
    assert_eq!(ev.meta["reason"], "reviewed");
    assert_eq!(ev.meta["policy"], "human_only");
    assert_eq!(ev.meta["policy_version"], "1");
    assert_eq!(
        (ev.meta["from"].as_str(), ev.meta["to"].as_str()),
        (Some("proposal"), Some("canonical"))
    );
    assert_eq!(v.layer_of(&ls[1]).unwrap(), Layer::Canonical);

    // API: an empty reason or actor is refused before the hook is asked
    let mut bad = v
        .request(
            TransitionKind::Promote,
            &ls[2],
            Layer::Canonical,
            human(),
            "",
        )
        .unwrap();
    assert!(err(v.promote(&bad, &HumanOnly)).contains("requires a reason"));
    bad.reason = "ok".into();
    bad.actor.id = " ".into();
    assert!(err(v.promote(&bad, &HumanOnly)).contains("requires an actor"));
    assert!(Actor::new(ActorKind::Human, "").is_err());
    assert!(Actor::parse("alice").is_err());
    assert!(Actor::parse("robot:x").is_err());
    drop(v);

    // log: a Promoted line without actor / reason / policy_version fails replay
    let log_path = root.join(".rootmark/events.jsonl");
    let log = fs::read_to_string(&log_path).unwrap();
    let line = log.lines().find(|l| l.contains("\"Promoted\"")).unwrap();
    for key in ["actor", "reason", "policy_version"] {
        let mut ev: serde_json::Value = serde_json::from_str(line).unwrap();
        ev["meta"].as_object_mut().unwrap().remove(key);
        let tampered = log.replace(line, &serde_json::to_string(&ev).unwrap());
        fs::write(&log_path, &tampered).unwrap();
        let e = format!("{:#}", Vault::open(&root).err().expect("missing field"));
        assert!(e.contains(&format!("requires meta.{key}")), "{key}: {e}");
    }
    fs::write(&log_path, &log).unwrap();
    Vault::open(&root).unwrap();
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn c2_03_no_promotion_without_hook_decision() {
    let root = tmp("c203");
    let (mut v, ls) = proposal_vault(&root);
    let log = fs::read(v.events_path()).unwrap();
    let req = v
        .request(
            TransitionKind::Promote,
            &ls[1],
            Layer::Canonical,
            human(),
            "reviewed",
        )
        .unwrap();
    // the default policy refuses everything
    assert!(err(v.promote(&req, &Deny)).contains("denied by policy deny"));
    // HumanOnly refuses a non-human actor
    let mut by_agent = req.clone();
    by_agent.actor = Actor::new(ActorKind::Agent, "claude").unwrap();
    assert!(err(v.promote(&by_agent, &HumanOnly)).contains("is not human"));
    let mut by_policy = req.clone();
    by_policy.actor = Actor::new(ActorKind::Policy, "auto").unwrap();
    assert!(err(v.promote(&by_policy, &HumanOnly)).contains("is not human"));
    assert_eq!(
        fs::read(v.events_path()).unwrap(),
        log,
        "refusals write nothing"
    );
    assert_eq!(v.layer_of(&ls[1]).unwrap(), Layer::Proposal);
    // the request must match the state and the rules even with a permissive hook
    assert!(err(v.promote(
        &v.request(
            TransitionKind::Promote,
            &ls[1],
            Layer::Derived,
            human(),
            "x"
        )
        .unwrap(),
        &HumanOnly
    ))
    .contains("proposal → canonical only"));
    let mut stale = req.clone();
    stale.from = Layer::Canonical;
    assert!(err(v.promote(&stale, &HumanOnly)).contains("requested from canonical"));
    let mut wrong_kind = req.clone();
    wrong_kind.kind = TransitionKind::Demote;
    assert!(err(v.promote(&wrong_kind, &HumanOnly)).contains("promote request"));
    assert!(v
        .request(
            TransitionKind::Promote,
            "nope",
            Layer::Canonical,
            human(),
            "x"
        )
        .is_err());
    assert_eq!(fs::read(v.events_path()).unwrap(), log);
    // promote is the only way up: canonical cannot be promoted again
    v.promote(&req, &HumanOnly).unwrap();
    assert!(err(v.promote(&req, &HumanOnly)).contains("requested from proposal"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn c2_04_demote_and_prune_delete_nothing() {
    let root = tmp("c204");
    let (mut v, ls) = proposal_vault(&root);
    let objects = root.join(".rootmark/objects");
    let snapshots = root.join(".rootmark/snapshots");
    let (n_obj, n_snap) = (file_count(&objects), file_count(&snapshots));
    let head = v.snapshot("plan.md").unwrap().unwrap();

    // promote, then demote back (canonical → proposal) and further (→ derived)
    let up = v
        .request(
            TransitionKind::Promote,
            &ls[1],
            Layer::Canonical,
            human(),
            "ok",
        )
        .unwrap();
    v.promote(&up, &HumanOnly).unwrap();
    let down = v
        .request(
            TransitionKind::Demote,
            &ls[1],
            Layer::Proposal,
            human(),
            "revert",
        )
        .unwrap();
    assert_eq!(
        v.demote(&down, &HumanOnly).unwrap().kind,
        EventKind::Demoted
    );
    assert_eq!(v.layer_of(&ls[1]).unwrap(), Layer::Proposal);
    let not_lower = v
        .request(
            TransitionKind::Demote,
            &ls[1],
            Layer::Canonical,
            human(),
            "x",
        )
        .unwrap();
    assert!(err(v.demote(&not_lower, &HumanOnly)).contains("must lower"));

    // prune: inactive, same layer, objects and snapshot untouched
    let ev = v
        .prune_inactive(&ls[2], human(), "stale proposal", &HumanOnly)
        .unwrap();
    assert_eq!(ev.kind, EventKind::Demoted);
    assert_eq!(ev.meta["prune"], true);
    assert!(v.is_inactive(&ls[2]).unwrap());
    assert_eq!(v.layer_of(&ls[2]).unwrap(), Layer::Proposal);
    assert_eq!(file_count(&objects), n_obj);
    assert_eq!(file_count(&snapshots), n_snap);
    assert_eq!(v.snapshot("plan.md").unwrap().unwrap(), head);
    // an inactive lineage takes no further transition; canonical is not pruned
    let again = v
        .request(
            TransitionKind::Promote,
            &ls[2],
            Layer::Canonical,
            human(),
            "x",
        )
        .unwrap();
    assert!(err(v.promote(&again, &HumanOnly)).contains("inactive"));
    // prune also goes through the hook
    let wroot = tmp("c204b");
    let (mut w, wl) = proposal_vault(&wroot);
    assert!(err(w.prune_inactive(&wl[1], human(), "x", &Deny)).contains("denied"));
    drop(w);
    fs::remove_dir_all(&wroot).unwrap();
    // replay rebuilds layers and the inactive mark
    drop(v);
    let v = Vault::open(&root).unwrap();
    assert!(v.is_inactive(&ls[2]).unwrap());
    assert_eq!(v.layer_of(&ls[1]).unwrap(), Layer::Proposal);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn prune_refuses_canonical() {
    let root = tmp("prune-canon");
    let mut v = Vault::open(&root).unwrap();
    let r = v
        .ingest(
            "h.md",
            "# H\n\nHuman text here.\n",
            Provenance::default(),
            M,
        )
        .unwrap();
    let l = &r.snapshot.blocks[1].0;
    assert!(err(v.prune_inactive(l, human(), "x", &HumanOnly)).contains("prune keeps a proposal"));
    fs::remove_dir_all(&root).unwrap();
}

/// Replay re-checks the built-in policies and rejects a malformed provenance
/// (B1 review notes 1–2).
#[test]
fn replay_rechecks_builtin_policy_and_provenance() {
    let root = tmp("recheck");
    let (mut v, ls) = proposal_vault(&root);
    let req = v
        .request(
            TransitionKind::Promote,
            &ls[1],
            Layer::Canonical,
            human(),
            "ok",
        )
        .unwrap();
    v.promote(&req, &HumanOnly).unwrap();
    drop(v);
    let log_path = root.join(".rootmark/events.jsonl");
    let log = fs::read_to_string(&log_path).unwrap();
    let as_agent = log.replace(
        r#""id":"alice","kind":"human""#,
        r#""id":"alice","kind":"agent""#,
    );
    assert_ne!(as_agent, log);
    fs::write(&log_path, &as_agent).unwrap();
    let e = format!(
        "{:#}",
        Vault::open(&root).err().expect("agent under human_only")
    );
    assert!(e.contains("human_only cannot have allowed"), "{e}");
    let by_deny = log.replace(r#""policy":"human_only""#, r#""policy":"deny""#);
    fs::write(&log_path, &by_deny).unwrap();
    let e = format!("{:#}", Vault::open(&root).err().expect("deny allowed"));
    assert!(e.contains("policy deny cannot have allowed"), "{e}");
    let bad_prov = log.replacen(r#""author_kind":"Agent""#, r#""author_kind":"Robot""#, 1);
    assert_ne!(bad_prov, log);
    fs::write(&log_path, &bad_prov).unwrap();
    let e = format!(
        "{:#}",
        Vault::open(&root).err().expect("malformed provenance")
    );
    assert!(e.contains("malformed meta.provenance"), "{e}");
    fs::write(&log_path, &log).unwrap();
    Vault::open(&root).unwrap();
    fs::remove_dir_all(&root).unwrap();
}

/// B2 review: a transition event cannot be written around the hook through
/// the low-level `commit`.
#[test]
fn commit_refuses_transition_events() {
    let root = tmp("bypass");
    let (mut v, ls) = proposal_vault(&root);
    let log = fs::read(v.events_path()).unwrap();
    let ver = v.snapshot("plan.md").unwrap().unwrap().blocks[1].1.clone();
    for (kind, to) in [
        (EventKind::Promoted, "canonical"),
        (EventKind::Demoted, "derived"),
    ] {
        let ev = rootmarkcore::Event {
            seq: 0,
            doc_path: "plan.md".into(),
            kind,
            lineage_ids: vec![ls[1].clone()],
            old_version_ids: vec![ver.clone()],
            new_version_ids: vec![ver.clone()],
            snapshot_id: None,
            meta: serde_json::json!({
                "actor": {"kind": "agent", "id": "claude"},
                "reason": "self-approval",
                "policy": "anything",
                "policy_version": "1",
                "from": "proposal",
                "to": to,
            }),
        };
        assert!(err(v.commit(&[], vec![ev])).contains("only through the policy hook"));
    }
    assert_eq!(fs::read(v.events_path()).unwrap(), log);
    assert_eq!(v.layer_of(&ls[1]).unwrap(), Layer::Proposal);
    fs::remove_dir_all(&root).unwrap();
}
