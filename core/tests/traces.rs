//! P1-B/B4: consumption traces (spec §8, C2-07) — every `read_blocks` call
//! appends one line to `.mdkernel/traces.jsonl`; `trace_id` formula.

use mdkcore::{hash32, trace_id, AuthorKind, Layer, MatcherVersion, Provenance, TraceEntry, Vault};
use std::fs;
use std::path::PathBuf;

const M: MatcherVersion = MatcherVersion::V3;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-trace-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn c2_07_trace_per_read_and_id_formula() {
    let root = tmp("c207");
    let mut v = Vault::open(&root).unwrap();
    v.ingest(
        "notes.md",
        "# Notes\n\nGrind finer for espresso.\n\nDescale monthly.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    v.ingest(
        "agent.md",
        "# Draft\n\nAgent draft paragraph.\n",
        Provenance {
            author_kind: AuthorKind::Agent,
            source: "claude-code".into(),
        },
        M,
    )
    .unwrap();
    let cp = v.checkpoint(Some("t0")).unwrap();
    let notes = v.snapshot("notes.md").unwrap().unwrap();
    let draft = v.snapshot("agent.md").unwrap().unwrap();
    let refs = vec![notes.blocks[2].0.clone(), draft.blocks[1].0.clone()];
    assert!(!root.join(".mdkernel/traces.jsonl").exists());

    let (blocks, t0) = v.read_blocks(&refs, "claude-code", "t0").unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].block.layer, Layer::Canonical);
    assert_eq!(
        blocks[1].block.layer,
        Layer::Proposal,
        "proposals are labelled"
    );
    assert_eq!(blocks[0].block.normalized_text, "Descale monthly.");

    // the line, byte for byte
    let qh = hash32(&format!("read_blocks\n{}\n{}", refs[0], refs[1]));
    let tid = hash32(&format!("{}claude-code{qh}0", cp.checkpoint_id));
    assert_eq!(tid, trace_id(&cp.checkpoint_id, "claude-code", &qh, 0));
    let want = format!(
        concat!(
            r#"{{"trace_id":"{}","checkpoint_id":"{}","consumer":"claude-code","query_hash":"{}","#,
            r#""returned":[{{"doc_path":"notes.md","lineage_id":"{}","version_id":"{}"}},"#,
            r#"{{"doc_path":"agent.md","lineage_id":"{}","version_id":"{}"}}],"at_seq":0}}"#,
            "\n"
        ),
        tid,
        cp.checkpoint_id,
        qh,
        notes.blocks[2].0,
        notes.blocks[2].1,
        draft.blocks[1].0,
        draft.blocks[1].1
    );
    let traces = root.join(".mdkernel/traces.jsonl");
    assert_eq!(fs::read_to_string(&traces).unwrap(), want);
    assert_eq!(t0.trace_id, tid);
    assert_eq!(
        t0.returned[0],
        TraceEntry {
            doc_path: "notes.md".into(),
            lineage_id: notes.blocks[2].0.clone(),
            version_id: notes.blocks[2].1.clone(),
        }
    );

    // the same read again: a second line, another at_seq and trace_id
    let (_, t1) = v
        .read_blocks(&refs, "claude-code", &cp.checkpoint_id)
        .unwrap();
    assert_eq!(t1.at_seq, 1);
    assert_ne!(t1.trace_id, t0.trace_id);
    assert_eq!(t1.query_hash, t0.query_hash);
    assert_eq!(fs::read_to_string(&traces).unwrap().lines().count(), 2);
    assert!(
        fs::read_to_string(&traces).unwrap().starts_with(&want),
        "append-only"
    );
    assert_eq!(v.traces(0).unwrap(), vec![t0.clone(), t1.clone()]);
    assert_eq!(v.traces(1).unwrap(), vec![t1.clone()]);

    // failed reads write no trace
    let before = fs::read(&traces).unwrap();
    assert!(v.read_blocks(&["nope".into()], "c", "t0").is_err());
    assert!(v.read_blocks(&refs, "c", "no-such").is_err());
    assert!(v.read_blocks(&refs, "", "t0").is_err());
    assert!(v.read_blocks(&[], "c", "t0").is_err());
    assert!(v
        .record_trace("t0", "c", "q", Vec::new())
        .is_err_and(|e| e.to_string().contains("checkpoint id")));
    assert_eq!(fs::read(&traces).unwrap(), before);

    // later events do not change what a read at the checkpoint returns
    v.ingest(
        "notes.md",
        "# Notes\n\nGrind finer for espresso.\n\nDescale every two weeks.\n",
        Provenance::default(),
        M,
    )
    .unwrap();
    let (later, _) = v.read_blocks(&refs[..1], "c", "t0").unwrap();
    assert_eq!(later[0].block.normalized_text, "Descale monthly.");

    // reopen: at_seq continues; a tampered line is an error
    drop(v);
    let mut v = Vault::open(&root).unwrap();
    let (_, t3) = v.read_blocks(&refs[..1], "c", "t0").unwrap();
    assert_eq!(t3.at_seq, 3);
    drop(v);
    let text = fs::read_to_string(&traces).unwrap();
    fs::write(&traces, text.replacen("claude-code", "someone-else", 1)).unwrap();
    let e = format!("{:#}", Vault::open(&root).err().expect("tampered trace"));
    assert!(e.contains("trace_id"), "{e}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn derived_is_never_returned() {
    let root = tmp("derived");
    let mut v = Vault::open(&root).unwrap();
    v.ingest_declared(
        "sum.md",
        "# Summary\n\nDerived summary text.\n",
        Provenance {
            author_kind: AuthorKind::Synthetic,
            source: "summarizer".into(),
        },
        M,
        Some(Layer::Derived),
    )
    .unwrap();
    v.checkpoint(None).unwrap();
    let cid = v.checkpoints()[0].checkpoint_id.clone();
    let l = v.snapshot("sum.md").unwrap().unwrap().blocks[1].0.clone();
    let e = format!("{:#}", v.read_blocks(&[l], "c", &cid).unwrap_err());
    assert!(e.contains("derived"), "{e}");
    assert!(!root.join(".mdkernel/traces.jsonl").exists());
    fs::remove_dir_all(&root).unwrap();
}
