//! P1-B.1/B1.2 (GOAL B1-4, B1-5; K-B1.2, spec 1.0.1 §8): `trace_id` with
//! `"\n"` separators; `query_hash` must be 32 lowercase hex and `consumer`
//! must not contain `"\n"`, in `record_trace` and when `traces.jsonl` is read.

use mdkcore::{hash32, trace_id, MatcherVersion, Provenance, Vault};
use std::fs;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-b12-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn vault(name: &str) -> (PathBuf, Vault, String) {
    let root = tmp(name);
    let mut v = Vault::open(&root).unwrap();
    v.ingest(
        "n.md",
        "# N\n\nText.\n",
        Provenance::default(),
        MatcherVersion::V3,
    )
    .unwrap();
    let cid = v.checkpoint(None).unwrap().checkpoint_id;
    (root, v, cid)
}

#[test]
fn formula_has_separators() {
    let cid = "0123456789abcdef0123456789abcdef";
    let qh = hash32("query");
    assert_eq!(
        trace_id(cid, "claude-code", &qh, 7),
        hash32(&format!("{cid}\nclaude-code\n{qh}\n7"))
    );
    assert_ne!(
        trace_id(cid, "claude-code", &qh, 7),
        hash32(&format!("{cid}claude-code{qh}7"))
    );
}

#[test]
fn invalid_inputs_are_refused() {
    // B1-4: 31 hex, 33 hex, non-hex query_hash; consumer with "\n"
    let (root, mut v, cid) = vault("invalid");
    let qh = hash32("q");
    let cases = [
        ("c", qh[..31].to_string()),
        ("c", format!("{qh}0")),
        ("c", format!("{}g", &qh[..31])),
        ("a\nb", qh.clone()),
    ];
    for (consumer, q) in &cases {
        assert!(
            v.record_trace(&cid, consumer, q, Vec::new()).is_err(),
            "{consumer:?} {q:?} accepted"
        );
    }
    // uppercase hex is not the §3 identifier form either
    assert!(v
        .record_trace(&cid, "c", &qh.to_uppercase(), Vec::new())
        .is_err());
    assert!(!root.join(".mdkernel/traces.jsonl").exists());
    let t = v.record_trace(&cid, "c", &qh, Vec::new()).unwrap();
    assert_eq!(t.trace_id, trace_id(&cid, "c", &qh, 0));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn shifted_fields_get_different_ids() {
    // B1-5: ("ab", qh) and ("a", "b" + qh) — the second is not a valid
    // query_hash and is refused; as raw formula inputs the ids differ
    let (root, mut v, cid) = vault("shift");
    let qh = hash32("q");
    let shifted = format!("b{qh}");
    assert_ne!(
        trace_id(&cid, "ab", &qh, 0),
        trace_id(&cid, "a", &shifted, 0)
    );
    // the old formula's collision with two valid hashes (DECISIONS K-B1.2)
    let q1 = "0123456789abcdef0123456789abcde1";
    let q2 = format!("b{}", &q1[..31]);
    assert_eq!(
        hash32(&format!("{cid}ab{q1}2")),
        hash32(&format!("{cid}a{q2}12"))
    );
    assert_ne!(trace_id(&cid, "ab", q1, 2), trace_id(&cid, "a", &q2, 12));
    let t = v.record_trace(&cid, "ab", &qh, Vec::new()).unwrap();
    assert!(v.record_trace(&cid, "a", &shifted, Vec::new()).is_err());
    assert_eq!(v.traces(0).unwrap(), vec![t]);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn invalid_trace_lines_are_refused_on_open() {
    let (root, mut v, cid) = vault("open");
    let qh = hash32("q");
    v.record_trace(&cid, "c", &qh, Vec::new()).unwrap();
    drop(v);
    let p = root.join(".mdkernel/traces.jsonl");
    let line = fs::read_to_string(&p).unwrap();
    // a 31-hex query_hash with a trace_id recomputed by the formula
    let bad = line.replace(&qh, &qh[..31]).replace(
        &trace_id(&cid, "c", &qh, 0),
        &trace_id(&cid, "c", &qh[..31], 0),
    );
    fs::write(&p, bad).unwrap();
    let e = format!(
        "{:#}",
        Vault::open(&root).err().expect("invalid query_hash")
    );
    assert!(e.contains("query_hash"), "{e}");
    fs::remove_dir_all(&root).unwrap();
}
