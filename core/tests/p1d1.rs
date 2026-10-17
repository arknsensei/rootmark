//! P1-D.1/D1.1: actors on change events (K-D1.2), the edit policy hook
//! (K-D1.1), checkpoint naming (K-D1.6) and the `[edit]`/`[identity]`
//! recipe tables.

use rootmarkcore::{
    Actor, ActorKind, AppendOnly, AuthorKind, Decision, Edit, EditMode, EditPolicy, EditRequest,
    EventKind, Layer, MatcherVersion, Policy, PolicySource, Provenance, Unrestricted, Vault,
};
use std::fs;
use std::path::{Path, PathBuf};

const M: MatcherVersion = MatcherVersion::V3;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-p1d1-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn agent_prov() -> Provenance {
    Provenance {
        author_kind: AuthorKind::Agent,
        source: "claude-code/1.0".into(),
    }
}

fn agent() -> Actor {
    Actor::new(ActorKind::Agent, "claude-code/1.0").unwrap()
}

const HUMAN_DOC: &str = "# Safety\n\n## Fire\n\nKeep a class D extinguisher next to the roaster at all times.\n\nNever leave a running roast unattended.\n\n## Water\n\nThe boiler is descaled every month by the shift lead.\n";

fn human_vault(root: &Path) -> Vault {
    let mut v = Vault::open(root).unwrap();
    v.ingest("safety.md", HUMAN_DOC, Provenance::default(), M)
        .unwrap();
    v
}

fn decide(v: &Vault, doc: &str, text: &str, mode: EditMode, policy: &dyn EditPolicy) -> Decision {
    let plan = v
        .plan_ingest(doc, text, agent_prov(), M, None, Some(&agent()))
        .unwrap();
    let effect = v.edit_effect(&plan).unwrap();
    policy.decide(&EditRequest {
        doc_path: doc.into(),
        mode,
        actor: agent(),
        section: Vec::new(),
        effect,
    })
}

#[test]
fn ingest_by_stamps_actor_and_source_on_change_events() {
    let root = tmp("actor");
    let mut v = human_vault(&root);
    // update one block, delete one, keep the rest
    let edited = "# Safety\n\n## Fire\n\nKeep a class D extinguisher beside the roaster at all times.\n\n## Water\n\nThe boiler is descaled every month by the shift lead.\n";
    let r = v
        .ingest_by("safety.md", edited, agent_prov(), M, &agent())
        .unwrap();
    let kinds: Vec<EventKind> = r.events.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&EventKind::BlockUpdated), "{kinds:?}");
    assert!(kinds.contains(&EventKind::BlockDeleted), "{kinds:?}");
    for ev in &r.events {
        let stamped = matches!(
            ev.kind,
            EventKind::BlockUpdated
                | EventKind::BlockMoved
                | EventKind::BlockSplit
                | EventKind::BlockMerged
                | EventKind::BlockDeleted
        );
        assert_eq!(ev.meta.get("actor").is_some(), stamped, "{:?}", ev.kind);
        if stamped {
            assert_eq!(
                ev.meta["actor"],
                serde_json::json!({"kind": "agent", "id": "claude-code/1.0"})
            );
            assert_eq!(ev.meta["source"], "claude-code/1.0");
        }
    }
    // history names the deleting actor; the log replays after reopening
    let del = r
        .events
        .iter()
        .find(|e| e.kind == EventKind::BlockDeleted)
        .unwrap();
    let v2 = Vault::open(&root).unwrap();
    let h = v2.history(&del.lineage_ids[0]).unwrap();
    let last = h.last().unwrap();
    assert_eq!(last.kind, EventKind::BlockDeleted);
    assert_eq!(last.meta["actor"]["kind"], "agent");
    assert_eq!(
        v2.replay("safety.md").unwrap(),
        v2.snapshot("safety.md").unwrap()
    );
}

#[test]
fn unattributed_ingest_writes_no_actor() {
    // the Level 1 API keeps the 1.0.2 bytes (C1 fixtures, events_hash)
    let a = tmp("plain-a");
    let b = tmp("plain-b");
    for root in [&a, &b] {
        let mut v = human_vault(root);
        v.ingest(
            "safety.md",
            "# Safety\n\n## Water\n\nThe boiler is descaled every month by the shift lead.\n",
            Provenance::default(),
            M,
        )
        .unwrap();
    }
    let log = fs::read_to_string(a.join(".rootmark/events.jsonl")).unwrap();
    assert!(!log.contains("\"actor\""), "{log}");
    assert_eq!(
        Vault::open(&a).unwrap().events_hash().unwrap(),
        Vault::open(&b).unwrap().events_hash().unwrap()
    );
}

#[test]
fn malformed_actor_is_rejected_on_open() {
    let root = tmp("bad-actor");
    let mut v = human_vault(&root);
    v.ingest_by("safety.md", "# Safety\n", agent_prov(), M, &agent())
        .unwrap();
    let p = root.join(".rootmark/events.jsonl");
    let log = fs::read_to_string(&p).unwrap();
    for (from, to) in [
        ("\"id\":\"claude-code/1.0\"", "\"id\":\"  \""),
        ("\"kind\":\"agent\"", "\"kind\":\"robot\""),
        ("\"source\":\"claude-code/1.0\"", "\"source\":7"),
    ] {
        assert!(log.contains(from));
        fs::write(&p, log.replacen(from, to, 1)).unwrap();
        assert!(Vault::open(&root).is_err(), "{to}");
    }
    fs::write(&p, &log).unwrap();
    Vault::open(&root).unwrap();
}

#[test]
fn append_only_denies_touching_canonical_and_allows_additions() {
    let root = tmp("policy");
    let v = human_vault(&root);
    // a new document: allowed
    assert_eq!(
        decide(
            &v,
            "agent/new.md",
            "# Note\n\nA proposal.\n",
            EditMode::Create,
            &AppendOnly
        ),
        Decision::Allow
    );
    // removing a human paragraph: denied (touched canonical)
    let without = HUMAN_DOC.replace("Never leave a running roast unattended.\n\n", "");
    assert!(matches!(
        decide(&v, "safety.md", &without, EditMode::ReplaceSection, &AppendOnly),
        Decision::Deny(m) if m.contains("not proposals")
    ));
    // text appended under the human `Water` section joins its chunk: denied
    let appended = format!("{HUMAN_DOC}\nAgents suggest descaling twice a month.\n");
    assert!(matches!(
        decide(&v, "safety.md", &appended, EditMode::Append, &AppendOnly),
        Decision::Deny(m) if m.contains("canonical retrieval units")
    ));
    // the same text under a heading of its own: its own chunk, allowed
    let headed = format!("{HUMAN_DOC}\n## Agent note\n\nAgents suggest descaling twice a month.\n");
    assert_eq!(
        decide(&v, "safety.md", &headed, EditMode::Append, &AppendOnly),
        Decision::Allow
    );
    // Unrestricted allows the deletion
    assert_eq!(
        decide(
            &v,
            "safety.md",
            &without,
            EditMode::ReplaceSection,
            &Unrestricted
        ),
        Decision::Allow
    );
}

#[test]
fn append_only_replace_section_needs_an_all_proposal_section() {
    let req = |section: Vec<(String, Layer)>| EditRequest {
        doc_path: "a.md".into(),
        mode: EditMode::ReplaceSection,
        actor: agent(),
        section,
        effect: Default::default(),
    };
    assert_eq!(
        AppendOnly.decide(&req(vec![
            ("h".into(), Layer::Proposal),
            ("p".into(), Layer::Proposal)
        ])),
        Decision::Allow
    );
    assert!(matches!(
        AppendOnly.decide(&req(vec![
            ("h".into(), Layer::Canonical),
            ("p".into(), Layer::Proposal)
        ])),
        Decision::Deny(m) if m.contains("replace_section")
    ));
}

#[test]
fn checkpoint_naming() {
    let root = tmp("named");
    let mut v = human_vault(&root);
    let unnamed = v.head_checkpoint().unwrap();
    assert_eq!(unnamed.name, None);
    let named = v.checkpoint(Some("before-agent")).unwrap();
    assert_eq!(named.checkpoint_id, unnamed.checkpoint_id);
    assert_eq!(named.name.as_deref(), Some("before-agent"));
    // idempotent; another name for a named state is an error
    assert_eq!(v.checkpoint(Some("before-agent")).unwrap(), named);
    assert!(v.checkpoint(Some("other")).is_err());
    assert!(v.checkpoint(None).is_err());
    // the file is unchanged, the log reopens, the name resolves
    let v2 = Vault::open(&root).unwrap();
    assert_eq!(
        v2.find_checkpoint("before-agent").unwrap().checkpoint_id,
        unnamed.checkpoint_id
    );
    let kinds: Vec<EventKind> = rootmarkcore::events::read_events(&v2.events_path())
        .unwrap()
        .iter()
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        kinds
            .iter()
            .filter(|k| **k == EventKind::CheckpointNamed)
            .count(),
        1
    );
    // a name another checkpoint holds: error
    let mut v = v2;
    v.ingest("b.md", "# B\n\nbody\n", Provenance::default(), M)
        .unwrap();
    v.head_checkpoint().unwrap();
    assert!(v.checkpoint(Some("before-agent")).is_err());
    // naming is not a content event: the same state keeps its checkpoint
    let cp = v.checkpoint(Some("after-b")).unwrap();
    assert_eq!(v.head_checkpoint().unwrap(), cp);
}

#[test]
fn tampered_checkpoint_name_is_rejected() {
    let root = tmp("named-bad");
    let mut v = human_vault(&root);
    v.head_checkpoint().unwrap();
    v.checkpoint(Some("one")).unwrap();
    let p = root.join(".rootmark/events.jsonl");
    let log = fs::read_to_string(&p).unwrap();
    let n = log.lines().count() as u64;
    let named: serde_json::Value = serde_json::from_str(log.lines().last().unwrap()).unwrap();
    assert_eq!(named["kind"], "CheckpointNamed");
    let with = |ev: serde_json::Value| format!("{log}{ev}\n");
    // the log as written opens
    Vault::open(&root).unwrap();
    // naming a named checkpoint again (valid seq): error
    let mut again = named.clone();
    again["seq"] = n.into();
    again["meta"]["name"] = "two".into();
    fs::write(&p, with(again)).unwrap();
    assert!(Vault::open(&root).is_err());
    // naming an unknown checkpoint: error
    let mut unknown = named.clone();
    unknown["seq"] = n.into();
    unknown["meta"]["checkpoint_id"] = "0".repeat(32).into();
    fs::write(&p, with(unknown)).unwrap();
    assert!(Vault::open(&root).is_err());
    // a name of the form of an id: error
    let mut idlike = named.clone();
    let lines: Vec<&str> = log.lines().collect();
    idlike["meta"]["name"] = "a".repeat(32).into();
    fs::write(
        &p,
        format!("{}\n{idlike}\n", lines[..lines.len() - 1].join("\n")),
    )
    .unwrap();
    assert!(Vault::open(&root).is_err());
    fs::write(&p, &log).unwrap();
    Vault::open(&root).unwrap();
}

#[test]
fn edit_and_identity_tables() {
    let base = "promotion = \"human_only\"\n[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n[journal]\nretention_hours = 168\n";
    let p = Policy::parse(base, PolicySource::Builtin).unwrap();
    assert_eq!(p.edit, Edit::AppendOnly);
    assert_eq!(p.to_json()["edit"], "append_only");
    assert_eq!(Policy::builtin().edit, Edit::AppendOnly);
    let u = Policy::parse(
        &format!("{base}[edit]\npolicy = \"unrestricted\"\n[identity]\nhuman_id = \"ada\"\n"),
        PolicySource::Builtin,
    )
    .unwrap();
    assert_eq!(u.edit, Edit::Unrestricted);
    assert_eq!(u.identity.unwrap()["human_id"].as_str(), Some("ada"));
    for bad in [
        "[edit]\npolicy = \"anything\"\n",
        "[edit]\npolicy = 1\n",
        "[edit]\nmode = \"x\"\n",
        "[edit]\n",
    ] {
        assert!(
            Policy::parse(&format!("{base}{bad}"), PolicySource::Builtin).is_err(),
            "{bad}"
        );
    }
}
