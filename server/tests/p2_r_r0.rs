//! P2-R/R0 (GOAL P2-R 1; R-1): P2-B's remainders.
//! - (a) K-P2B-T1 (a): an indented code block's location starts at the start
//!   of its first line (its indent included), so the slice re-parses into
//!   the block; the block's raw text, ids, events and `events_hash` are
//!   those of the binary before the change (recorded below);
//! - (c8) `search` computes the hits' locations before its trace;
//! - (c9) `events` reads the log with a line counter: the answer's bytes
//!   are as before, a page from the end of a large log is quick;
//! - (c11) `commit_plan` checks the planned snapshot before anything is
//!   written.
//!
//! Needs the model files (`make models`).

use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::PathBuf;
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

fn root(name: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("rootmark-p2r0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    r
}

fn vault(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let v = root(name).join("vault");
    std::fs::create_dir_all(&v).unwrap();
    for (p, t) in files {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        format!(
            "{}\n[identity]\nhuman_id = \"ada\"\n",
            rootmarkserver::config::default_recipe()
        ),
    )
    .unwrap();
    v
}

/// Indented code blocks: four spaces (with a deeper second line), a tab,
/// and indented code inside a list item (not a top-level block).
const CODE: &[u8] = b"# Kod\n\nGirintili kod:\n\n    fn main() {\n        println!(\"x\");\n    }\n\nAra paragraf.\n\n\tsekmeli kod\n\tikinci sat\xc4\xb1r\n\n- liste\n\n      liste i\xc3\xa7i kod\n\nSon.\n";

/// `events_hash` of a vault holding only `code.md`, by the binary before
/// R0 (a) (`main` ae7e692: `rootmark ingest-dir`, then `events-hash`).
const CODE_EVENTS_HASH: &str = "2e28e8c8beb89b13b25457345dd5ecc7";

/// The block locations before R0 (a), `main` ae7e692 (`replay --loc`):
/// `(block_type, byte_start, byte_end)`. Only the two indented code blocks'
/// `byte_start` changes (27 → 23, 85 → 84: their line starts).
const BEFORE: [(&str, u64, u64); 7] = [
    ("Heading", 0, 5),
    ("Paragraph", 7, 21),
    ("Code", 27, 67),
    ("Paragraph", 69, 82),
    ("Code", 85, 111),
    ("ListItem", 113, 142),
    ("Paragraph", 144, 148),
];

#[test]
fn a_an_indented_code_block_is_located_from_its_line_start() {
    let v = vault("code", &[("code.md", CODE)]);
    let vs = v.to_str().unwrap();
    assert_eq!(
        ok_json(&["events-hash", vs])["events_hash"],
        CODE_EVENTS_HASH
    );
    let r = ok_json(&["replay", "--loc", vs, "code.md"]);
    let text = std::str::from_utf8(CODE).unwrap();
    let rows = r["blocks_detail"].as_array().unwrap();
    assert_eq!(rows.len(), BEFORE.len());
    for (row, (bt, bs0, be0)) in rows.iter().zip(BEFORE) {
        assert_eq!(row["block_type"], bt, "{row}");
        let (bs, be) = (
            row["loc"]["byte_start"].as_u64().unwrap(),
            row["loc"]["byte_end"].as_u64().unwrap(),
        );
        // only an indented code block moves, and only to its line start
        assert_eq!(be, be0, "{row}");
        if bs != bs0 {
            assert_eq!(bt, "Code", "{row}");
            let line = text[..bs0 as usize].rfind('\n').map_or(0, |i| i + 1) as u64;
            assert_eq!(bs, line, "{row}");
            assert!(text[bs as usize..bs0 as usize]
                .bytes()
                .all(|b| b == b' ' || b == b'\t'));
        }
        // the slice re-parses alone into the block (B-3's criterion (i))
        let slice = &text[bs as usize..be as usize];
        let (d, _) = rootmarkcore::parse_document(slice, "x.md").unwrap();
        assert_eq!(d.len(), 1, "{slice:?}");
        assert_eq!(d[0].version_id, row["version_id"].as_str().unwrap());
        assert_eq!(
            d[0].normalized_text,
            row["normalized_text"].as_str().unwrap()
        );
    }
    // both indented code blocks moved
    let moved = rows
        .iter()
        .zip(BEFORE)
        .filter(|(r, (_, bs0, _))| r["loc"]["byte_start"].as_u64().unwrap() != *bs0)
        .count();
    assert_eq!(moved, 2);
    // the block's raw text is unchanged: it still begins after the indent
    let (drafts, _) = rootmarkcore::parse_document(text, "code.md").unwrap();
    assert!(
        drafts[2].text.starts_with("fn main() {"),
        "{:?}",
        drafts[2].text
    );
}

#[test]
fn c8_search_locations_are_computed_before_the_trace() {
    let v = vault(
        "search",
        &[
            ("a.md", b"# Alpha\n\nAlpha beta gamma.\n"),
            ("b.md", b"# Beta\n\nBeta delta epsilon.\n"),
        ],
    );
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let c = Caller::http("p2-r-r0");
    let traces = v.join(".rootmark/traces.jsonl");
    let ok = k
        .call("search", &json!({"query": "beta", "loc": true}), c)
        .unwrap();
    assert!(ok["hits"][0]["loc"][0]["byte_start"].is_u64(), "{ok}");
    let n = std::fs::read_to_string(&traces).unwrap().lines().count();
    // a location record that cannot be read: the search fails and records
    // no trace (before R0 the trace line was written first)
    let loc_dir = v.join(".rootmark/loc");
    for d in std::fs::read_dir(&loc_dir).unwrap() {
        for f in std::fs::read_dir(d.unwrap().path()).unwrap() {
            std::fs::write(f.unwrap().path(), b"not a record").unwrap();
        }
    }
    // the file no longer holds the version: the record is read
    std::fs::write(v.join("a.md"), b"# Alpha\n\nchanged on disk\n").unwrap();
    std::fs::write(v.join("b.md"), b"# Beta\n\nchanged on disk\n").unwrap();
    let err = k
        .call("search", &json!({"query": "beta", "loc": true}), c)
        .unwrap_err();
    assert!(format!("{err:#}").contains("location record"), "{err:#}");
    assert_eq!(
        std::fs::read_to_string(&traces).unwrap().lines().count(),
        n,
        "a failed search wrote a trace"
    );
    // without `loc` the search is unaffected and traces as before
    k.call("search", &json!({"query": "beta"}), c).unwrap();
    assert_eq!(
        std::fs::read_to_string(&traces).unwrap().lines().count(),
        n + 1
    );
}

/// The `events` page as the server answers it (HTTP body bytes).
fn events_body(k: &Kernel, a: Value) -> String {
    k.run_events(&a).unwrap().to_body()
}

/// The answer before R0 (c9): the whole log read and every line from
/// `since_seq` parsed until the page is full (P2-B's `events_page`), here
/// over the log's text for the same arguments.
fn events_reference(log: &str, since: u64, limit: usize, kinds: Option<&[&str]>) -> String {
    let all: Vec<&str> = log.lines().collect();
    let head = all.len() as u64;
    let mut lines = Vec::new();
    let mut next = head;
    for (i, line) in all.iter().enumerate().skip(since as usize) {
        if lines.len() == limit {
            next = i as u64;
            break;
        }
        let ev: Value = serde_json::from_str(line).unwrap();
        if kinds.is_none_or(|k| k.contains(&ev["kind"].as_str().unwrap())) {
            lines.push(line.to_string());
        }
    }
    let filter = kinds.map_or(String::new(), |k| {
        let mut k = k.to_vec();
        k.sort();
        format!(",\"filter\":{}", json!({ "kinds": k }))
    });
    format!(
        "{{\"events\":[{}]{filter},\"head_seq\":{head},\"next_seq\":{next}}}",
        lines.join(",")
    )
}

#[test]
fn c9_events_reads_from_since_seq_with_the_same_bytes() {
    // a log of some thousands of events: 300 documents, each ingested, then
    // a checkpoint every 50
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..300 {
        files.push((
            format!("d/{i:03}.md"),
            format!("# Doc {i}\n\nParagraph {i} one.\n\nParagraph {i} two.\n").into_bytes(),
        ));
    }
    let refs: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_slice()))
        .collect();
    let v = vault("events", &refs);
    let vs = v.to_str().unwrap();
    for _ in 0..3 {
        ok_json(&["checkpoint", "create", vs]);
    }
    let log = std::fs::read_to_string(v.join(".rootmark/events.jsonl")).unwrap();
    let head = log.lines().count() as u64;
    assert!(head > 1000, "{head}");
    let k = Kernel::open(&v).unwrap();
    let cases: Vec<(u64, usize, Option<&[&str]>)> = vec![
        (0, 100, None),
        (0, 1000, None),
        (17, 5, None),
        (head - 3, 100, None),
        (head, 100, None),
        (0, 7, Some(&["CheckpointCreated"])),
        (head / 2, 3, Some(&["SnapshotCommitted", "BlockCreated"])),
        (head - 1, 1, Some(&["CheckpointCreated"])),
    ];
    for (since, limit, kinds) in cases {
        let mut a = json!({"since_seq": since, "limit": limit});
        if let Some(k) = kinds {
            a["kinds"] = json!(k);
        }
        assert_eq!(
            events_body(&k, a.clone()),
            events_reference(&log, since, limit, kinds),
            "{a}"
        );
    }
    // beyond the log: refused, as before
    let err = k
        .call("events", &json!({"since_seq": head + 1}), Caller::http("x"))
        .unwrap_err();
    assert!(format!("{err:#}").contains("beyond the log"), "{err:#}");
    // the end of a large log under the readers' lock: the lines before
    // `since_seq` are counted, not parsed (time recorded, not a threshold)
    let t = std::time::Instant::now();
    let p = k
        .run_events(&json!({"since_seq": head - 1, "limit": 1}))
        .unwrap();
    let tail_ms = t.elapsed().as_secs_f64() * 1e3;
    assert_eq!(p.lines.len(), 1);
    assert_eq!(p.next_seq, head);
    let t = std::time::Instant::now();
    k.run_events(&json!({"since_seq": 0, "limit": 1000}))
        .unwrap();
    let page_ms = t.elapsed().as_secs_f64() * 1e3;
    eprintln!(
        "events: head_seq {head}; tail page {tail_ms:.2} ms; first full page {page_ms:.2} ms"
    );
}

#[test]
fn c11_a_plan_whose_snapshot_is_not_its_location_record_writes_nothing() {
    let r = root("plan");
    std::fs::create_dir_all(&r).unwrap();
    let mut v = rootmarkcore::Vault::open(&r).unwrap();
    let m = rootmarkcore::MatcherVersion::V3;
    let prov = rootmarkcore::Provenance::default();
    v.ingest("a.md", "# A\n\nfirst\n", prov.clone(), m).unwrap();
    let log = std::fs::read(r.join(".rootmark/events.jsonl")).unwrap();
    let mut plan = v
        .plan_ingest("a.md", "# A\n\nfirst\n\nsecond\n", prov, m, None, None)
        .unwrap();
    let sc = plan
        .events
        .iter_mut()
        .rev()
        .find(|e| e.kind == rootmarkcore::EventKind::SnapshotCommitted)
        .unwrap();
    sc.snapshot_id = Some("0".repeat(32));
    let err = v.commit_plan(plan).unwrap_err();
    assert!(
        format!("{err:#}").contains("location record names"),
        "{err:#}"
    );
    assert_eq!(
        std::fs::read(r.join(".rootmark/events.jsonl")).unwrap(),
        log
    );
    // no location record of the refused plan either
    let recs: usize = std::fs::read_dir(r.join(".rootmark/loc"))
        .unwrap()
        .map(|d| std::fs::read_dir(d.unwrap().path()).unwrap().count())
        .sum();
    assert_eq!(recs, 1);
}
