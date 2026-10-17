//! P1-B/B3: checkpoints (spec §3, §8) — id formula and write-once file
//! (C2-05), queries as of a checkpoint ignore later events (C2-06).

use rootmarkcore::store::write_immutable;
use rootmarkcore::{
    checkpoint_id, hash32, Actor, ActorKind, AuthorKind, EventKind, HumanOnly, Layer,
    MatcherVersion, Provenance, TransitionKind, Vault,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const M: MatcherVersion = MatcherVersion::V3;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rootmark-cp-{}-{name}", std::process::id()));
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

fn seeded(root: &PathBuf) -> Vault {
    let mut v = Vault::open(root).unwrap();
    v.ingest(
        "b.md",
        "# Beans\n\nWashed beans taste clean and bright.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    v.ingest(
        "a.md",
        "# Agent notes\n\nThe agent suggests a finer grind for this roast.\n",
        agent(),
        M,
    )
    .unwrap();
    v
}

#[test]
fn c2_05_checkpoint_id_formula_and_write_once() {
    let root = tmp("c205");
    let mut v = seeded(&root);
    let n_events = fs::read_to_string(v.events_path()).unwrap().lines().count() as u64;
    let cp = v.checkpoint(Some("day-1")).unwrap();
    assert_eq!(cp.event_seq, n_events);
    let mut docs = BTreeMap::new();
    for d in ["a.md", "b.md"] {
        docs.insert(d.to_string(), v.snapshot(d).unwrap().unwrap().snapshot_id);
    }
    assert_eq!(cp.docs, docs);
    // spec §3, written out: sorted "doc_path\tsnapshot_id" lines, "\n", event_seq
    let manual = hash32(&format!(
        "a.md\t{}\nb.md\t{}\n{}",
        docs["a.md"], docs["b.md"], n_events
    ));
    assert_eq!(cp.checkpoint_id, manual);
    assert_eq!(cp.checkpoint_id, checkpoint_id(&docs, n_events));
    // file and event
    let file = root.join(format!(".rootmark/checkpoints/{}.json", cp.checkpoint_id));
    let bytes = fs::read(&file).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&cp).unwrap());
    let last = fs::read_to_string(v.events_path()).unwrap();
    let ev: rootmarkcore::Event = serde_json::from_str(last.lines().last().unwrap()).unwrap();
    assert_eq!(ev.kind, EventKind::CheckpointCreated);
    assert_eq!(ev.meta["checkpoint_id"], cp.checkpoint_id.as_str());
    // deterministic: the same state gives the same id, and nothing is written again
    let again = v.checkpoint(Some("day-1")).unwrap();
    assert_eq!(again, cp);
    assert_eq!(fs::read_to_string(v.events_path()).unwrap(), last);
    // a second name for the same state, or a taken name, is refused
    assert!(v.checkpoint(Some("other")).is_err());
    assert!(v.checkpoint(None).is_err());
    // write-once: different bytes under the same id fail; identical bytes are a no-op
    assert!(write_immutable(&file, b"{}").is_err());
    write_immutable(&file, &bytes).unwrap();
    // a changed file is detected when the vault is opened
    drop(v);
    fs::write(
        &file,
        String::from_utf8(bytes.clone())
            .unwrap()
            .replace("day-1", "day-2"),
    )
    .unwrap();
    let e = format!(
        "{:#}",
        Vault::open(&root).err().expect("tampered checkpoint")
    );
    assert!(
        e.contains("does not match its CheckpointCreated event"),
        "{e}"
    );
    fs::remove_file(&file).unwrap();
    let e = format!(
        "{:#}",
        Vault::open(&root).err().expect("missing checkpoint")
    );
    assert!(e.contains("has no file"), "{e}");
    fs::write(&file, &bytes).unwrap();
    let mut v = Vault::open(&root).unwrap();
    assert_eq!(v.checkpoints(), vec![cp.clone()]);
    // a later state gets another id; names stay unique
    v.ingest(
        "b.md",
        "# Beans\n\nWashed beans taste clean and bright.\n\nNatural beans taste fruity.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    assert!(v.checkpoint(Some("day-1")).is_err());
    let cp2 = v.checkpoint(Some("day-2")).unwrap();
    assert_ne!(cp2.checkpoint_id, cp.checkpoint_id);
    assert!(cp2.event_seq > cp.event_seq + 1);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn c2_06_queries_as_of_checkpoint_ignore_later_events() {
    let root = tmp("c206");
    let mut v = seeded(&root);
    let cp = v.checkpoint(Some("before")).unwrap();
    let old_a = v.snapshot("a.md").unwrap().unwrap();
    let old_b = v.snapshot("b.md").unwrap().unwrap();
    let l_agent = old_a.blocks[1].0.clone();
    let old_block = v.block_at(&l_agent, "before").unwrap();
    assert_eq!(old_block.layer, Layer::Proposal);

    // later: an edit, a new document, a promotion
    v.ingest(
        "b.md",
        "# Beans\n\nWashed beans taste clean, bright and sweet.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    v.ingest(
        "c.md",
        "# New\n\nLater document.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    let human = Actor::new(ActorKind::Human, "alice").unwrap();
    let req = v
        .request(
            TransitionKind::Promote,
            &l_agent,
            Layer::Canonical,
            human,
            "ok",
        )
        .unwrap();
    v.promote(&req, &HumanOnly).unwrap();
    assert_eq!(v.layer_of(&l_agent).unwrap(), Layer::Canonical);

    for id in ["before", cp.checkpoint_id.as_str()] {
        assert_eq!(v.snapshot_at("b.md", id).unwrap().unwrap(), old_b);
        assert_eq!(v.snapshot_at("a.md", id).unwrap().unwrap(), old_a);
        assert_eq!(v.snapshot_at("c.md", id).unwrap(), None);
        let b = v.block_at(&l_agent, id).unwrap();
        assert_eq!(b, old_block);
        assert_eq!(
            b.layer,
            Layer::Proposal,
            "promotion after the checkpoint is invisible"
        );
    }
    assert_ne!(v.snapshot("b.md").unwrap().unwrap(), old_b);
    assert!(v.snapshot_at("b.md", "no-such-checkpoint").is_err());
    // the result is the same after reopening (replay)
    drop(v);
    let v = Vault::open(&root).unwrap();
    assert_eq!(v.snapshot_at("b.md", "before").unwrap().unwrap(), old_b);
    assert_eq!(v.block_at(&l_agent, "before").unwrap(), old_block);
    fs::remove_dir_all(&root).unwrap();
}

/// B3 review notes 1–2: a checkpoint name in the log must be a string, and a
/// name cannot take the form of a checkpoint id.
#[test]
fn checkpoint_name_checks() {
    let root = tmp("names");
    let mut v = seeded(&root);
    let cp = v.checkpoint(Some("n1")).unwrap();
    assert!(v
        .checkpoint(Some(&"a".repeat(32)))
        .is_err_and(|e| e.to_string().contains("form of a checkpoint id")));
    drop(v);
    let p = root.join(".rootmark/events.jsonl");
    let log = fs::read_to_string(&p).unwrap();
    fs::write(&p, log.replace(r#""name":"n1""#, r#""name":5"#)).unwrap();
    let e = format!("{:#}", Vault::open(&root).err().expect("bad name type"));
    assert!(e.contains("is not a name"), "{e}");
    fs::write(&p, &log).unwrap();
    assert_eq!(Vault::open(&root).unwrap().checkpoints(), vec![cp]);
    fs::remove_dir_all(&root).unwrap();
}

/// P1-C/C4: `head_checkpoint` returns the checkpoint of the current state
/// (named or not) and creates an unnamed one only when the state has none.
#[test]
fn head_checkpoint_reuses_or_creates() {
    let root = std::env::temp_dir().join(format!("rootmark-headcp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut v = rootmarkcore::Vault::open(&root).unwrap();
    v.ingest(
        "a.md",
        "# A\n\nText.\n",
        rootmarkcore::Provenance::default(),
        rootmarkcore::MatcherVersion::V3,
    )
    .unwrap();
    let named = v.checkpoint(Some("t0")).unwrap();
    let n_events = std::fs::read_to_string(v.events_path())
        .unwrap()
        .lines()
        .count();
    assert_eq!(v.head_checkpoint().unwrap(), named);
    assert_eq!(
        std::fs::read_to_string(v.events_path())
            .unwrap()
            .lines()
            .count(),
        n_events,
        "nothing written for an existing checkpoint"
    );
    v.ingest(
        "a.md",
        "# A\n\nText, edited.\n",
        rootmarkcore::Provenance::default(),
        rootmarkcore::MatcherVersion::V3,
    )
    .unwrap();
    let fresh = v.head_checkpoint().unwrap();
    assert_ne!(fresh.checkpoint_id, named.checkpoint_id);
    assert_eq!(fresh.name, None);
    assert_eq!(v.head_checkpoint().unwrap(), fresh);
    assert_eq!(v.checkpoints().len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}
