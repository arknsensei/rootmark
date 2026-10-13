use mdkcore::store::{object_path, put_object};
use mdkcore::{hash32, snapshot_id, version_id, BlockObject, BlockType, Event, EventKind, Vault};
use regex::Regex;
use serde_json::json;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mdk-store-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn obj(text: &str) -> BlockObject {
    BlockObject {
        version_id: version_id(BlockType::Paragraph, text),
        block_type: BlockType::Paragraph,
        normalized_text: text.into(),
        heading_level: None,
    }
}

fn ev(doc: &str, kind: EventKind, l: &[&str], old: &[&str], new: &[&str]) -> Event {
    Event {
        seq: 0,
        doc_path: doc.into(),
        kind,
        lineage_ids: l.iter().map(|s| s.to_string()).collect(),
        old_version_ids: old.iter().map(|s| s.to_string()).collect(),
        new_version_ids: new.iter().map(|s| s.to_string()).collect(),
        snapshot_id: None,
        meta: json!({"matcher_version": "v2"}),
    }
}

fn commit_ev(doc: &str, pairs: &[(&str, &str)], parent: Option<&str>) -> Event {
    let mut e = ev(
        doc,
        EventKind::SnapshotCommitted,
        &pairs.iter().map(|p| p.0).collect::<Vec<_>>(),
        &[],
        &pairs.iter().map(|p| p.1).collect::<Vec<_>>(),
    );
    e.snapshot_id = Some(snapshot_id(pairs.iter().map(|p| p.1)));
    e.meta = json!({"matcher_version": "v2", "frontmatter": null, "parent_snapshot": parent});
    e
}

/// Two commits of one document: create a,b; then update b → c and delete a.
fn build(root: &PathBuf) -> (Vault, String, String) {
    let mut v = Vault::open(root).unwrap();
    let (a, b, c) = (obj("alpha"), obj("beta"), obj("gamma"));
    let (la, lb) = (hash32("la"), hash32("lb"));
    let s1 = vec![
        ev(
            "d.md",
            EventKind::BlockCreated,
            &[&la],
            &[],
            &[&a.version_id],
        ),
        ev(
            "d.md",
            EventKind::BlockCreated,
            &[&lb],
            &[],
            &[&b.version_id],
        ),
        commit_ev("d.md", &[(&la, &a.version_id), (&lb, &b.version_id)], None),
    ];
    let sid1 = s1[2].snapshot_id.clone().unwrap();
    v.commit(&[a.clone(), b.clone()], s1).unwrap();
    let s2 = vec![
        ev(
            "d.md",
            EventKind::BlockUpdated,
            &[&lb],
            &[&b.version_id],
            &[&c.version_id],
        ),
        ev(
            "d.md",
            EventKind::BlockDeleted,
            &[&la],
            &[&a.version_id],
            &[],
        ),
        commit_ev("d.md", &[(&lb, &c.version_id)], Some(&sid1)),
    ];
    let sid2 = s2[2].snapshot_id.clone().unwrap();
    v.commit(&[c], s2).unwrap();
    (v, sid1, sid2)
}

#[test]
fn same_object_twice_leaves_bytes_unchanged() {
    let root = tmp("same");
    let o = obj("hello");
    put_object(&root, &o).unwrap();
    let p = object_path(&root, &o.version_id);
    let before = std::fs::read(&p).unwrap();
    let mtime = std::fs::metadata(&p).unwrap().modified().unwrap();
    put_object(&root, &o).unwrap();
    assert_eq!(std::fs::read(&p).unwrap(), before);
    assert_eq!(std::fs::metadata(&p).unwrap().modified().unwrap(), mtime);
}

#[test]
fn same_id_different_content_is_err() {
    let root = tmp("conflict");
    let o = obj("hello");
    put_object(&root, &o).unwrap();
    let mut bad = o.clone();
    bad.normalized_text = "tampered".into();
    assert!(put_object(&root, &bad).is_err());
}

#[test]
fn replay_matches_head_snapshot_id() {
    let root = tmp("replay");
    let (v, _, sid2) = build(&root);
    let head = v.snapshot("d.md").unwrap().unwrap();
    let rep = v.replay("d.md").unwrap().unwrap();
    assert_eq!(head, rep);
    assert_eq!(rep.snapshot_id, sid2);
    assert_eq!(rep.blocks.len(), 1);
    // reopening rebuilds the same head from the log
    let v2 = Vault::open(&root).unwrap();
    assert_eq!(v2.snapshot("d.md").unwrap().unwrap(), head);
    assert!(v.replay("missing.md").unwrap().is_none());
}

#[test]
fn log_is_append_only_prefix() {
    let root = tmp("prefix");
    let mut v = Vault::open(&root).unwrap();
    let a = obj("one");
    let la = hash32("l1");
    v.commit(
        std::slice::from_ref(&a),
        vec![
            ev(
                "x.md",
                EventKind::BlockCreated,
                &[&la],
                &[],
                &[&a.version_id],
            ),
            commit_ev("x.md", &[(&la, &a.version_id)], None),
        ],
    )
    .unwrap();
    let first = std::fs::read(v.events_path()).unwrap();
    let h1 = v.events_hash().unwrap();
    let b = obj("two");
    let lb = hash32("l2");
    let sid = snapshot_id([a.version_id.as_str()]);
    v.commit(
        std::slice::from_ref(&b),
        vec![
            ev(
                "x.md",
                EventKind::BlockCreated,
                &[&lb],
                &[],
                &[&b.version_id],
            ),
            commit_ev(
                "x.md",
                &[(&la, &a.version_id), (&lb, &b.version_id)],
                Some(&sid),
            ),
        ],
    )
    .unwrap();
    let second = std::fs::read(v.events_path()).unwrap();
    assert!(second.len() > first.len());
    assert_eq!(&second[..first.len()], &first[..]);
    assert_ne!(v.events_hash().unwrap(), h1);
    // seq is contiguous from 0
    let lines: Vec<serde_json::Value> = String::from_utf8(second)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    for (i, l) in lines.iter().enumerate() {
        assert_eq!(l["seq"], i as u64);
    }
}

#[test]
fn inconsistent_snapshot_is_rejected_and_not_logged() {
    let root = tmp("reject");
    let mut v = Vault::open(&root).unwrap();
    let a = obj("one");
    let la = hash32("l1");
    let mut bad = commit_ev("x.md", &[(&la, &a.version_id)], None);
    bad.snapshot_id = Some(hash32("wrong"));
    let r = v.commit(
        std::slice::from_ref(&a),
        vec![
            ev(
                "x.md",
                EventKind::BlockCreated,
                &[&la],
                &[],
                &[&a.version_id],
            ),
            bad,
        ],
    );
    assert!(r.is_err());
    assert!(!v.events_path().exists());
}

#[test]
fn events_have_no_timestamps_or_random_ids() {
    let root = tmp("regex");
    let (v, _, _) = build(&root);
    let log = std::fs::read_to_string(v.events_path()).unwrap();
    let iso = Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}").unwrap();
    let ulid = Regex::new(r#""[0-9A-HJKMNP-TV-Z]{26}""#).unwrap();
    let uuid = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-").unwrap();
    let epoch = Regex::new(r":\s*1[0-9]{9}(\.\d+)?[,}]").unwrap();
    for line in log.lines() {
        assert!(!iso.is_match(line), "timestamp in {line}");
        assert!(!ulid.is_match(line), "ULID in {line}");
        assert!(!uuid.is_match(line), "UUID in {line}");
        assert!(!epoch.is_match(line), "epoch in {line}");
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        assert_eq!(
            keys,
            vec![
                "doc_path",
                "kind",
                "lineage_ids",
                "meta",
                "new_version_ids",
                "old_version_ids",
                "seq",
                "snapshot_id"
            ]
        );
        for k in v["meta"].as_object().unwrap().keys() {
            assert!(
                !k.contains("time") && !k.contains("date") && !k.contains("host"),
                "{k}"
            );
        }
    }
}

#[test]
fn events_hash_equal_for_identical_logs_and_object_must_hash() {
    let (a, _, _) = build(&tmp("eq-a"));
    let (b, _, _) = build(&tmp("eq-b"));
    assert_eq!(a.events_hash().unwrap(), b.events_hash().unwrap());
    let mut bad = obj("x");
    bad.version_id = hash32("not-the-content");
    assert!(put_object(&tmp("eq-c"), &bad).is_err());
}
