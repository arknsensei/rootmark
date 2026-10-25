//! P2-B/B1 (GOAL P2-B 2; B-2): block locations in the contract (v2).
//! - `blame`, `read_block` and `search` hits carry per block `loc
//!   {line_start, line_end, byte_start, byte_end, content_hash}`: the slice
//!   `file[byte_start..byte_end]` re-parses into the block, the lines are
//!   the bytes' lines, `content_hash` is the hash of the bytes;
//! - a file changed and not yet ingested: `loc_stale: true` and the
//!   locations of the ingested version (its `content_hash`), never the old
//!   offsets on the new bytes; after the ingest, fresh again; a version with
//!   no recorded locations gives none;
//! - a location is derived: `events.jsonl`, `traces.jsonl` (beyond the
//!   search's own trace) and the markdown bytes are unchanged; the v1 fields
//!   are as before; P2-A's six old vaults keep their `events_hash`.
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
    let r = std::env::temp_dir().join(format!("rootmark-p2b1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    r
}

fn recipe() -> String {
    format!(
        "{}\n[identity]\nhuman_id = \"ada\"\n",
        rootmarkserver::config::default_recipe()
    )
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
    std::fs::write(v.join(".rootmark/policy.toml"), recipe()).unwrap();
    v
}

fn reader() -> Caller<'static> {
    Caller::http("p2-b-b1")
}

const EDGE: &[u8] = "\u{feff}---\r\ntitle: kenar\r\ntags: [a]\r\n---\r\n# Başlık 🎉 ve çay\r\n\r\nBir paragraf\r\niki satır.\r\n\r\n- madde bir\r\n- madde iki\r\n  - iç içe\r\n\r\n> [!note] Not\r\n> > iç içe callout\r\n\r\n<div>\r\n\r\nhtml içinde boşluk</div>\r\n\r\n| a | b |\r\n|---|---|\r\n| 1 | 2 |\r\n\r\n```rust\r\nfn main() {}\r\n```\r\n\r\n![[resim.png]] ve ![[diğer.png]]\r\n\r\nDipnot[^1].\r\n\r\n[^1]: belge sonunda tanım\r\n".as_bytes();

const PLAIN: &[u8] = b"---\ntitle: plain\n---\n# Plain\n\nFirst paragraph.\n\n## Second\n\nSecond paragraph with words.\n";

/// The line (1-based) of a byte offset.
fn line_of(bytes: &[u8], off: usize) -> u64 {
    1 + bytes[..off].iter().filter(|&&b| b == b'\n').count() as u64
}

/// Every `loc` of `rows` (snapshot order, with `version_id`) describes
/// `bytes`: the slice re-parses into the block's version, the lines match.
fn check_locs(bytes: &[u8], rows: &[Value]) {
    let text = std::str::from_utf8(bytes).unwrap();
    for r in rows {
        let l = &r["loc"];
        let (bs, be) = (
            l["byte_start"].as_u64().unwrap() as usize,
            l["byte_end"].as_u64().unwrap() as usize,
        );
        assert!(bs < be && be <= bytes.len(), "{r}");
        let slice = &text[bs..be];
        let (drafts, _) = rootmarkcore::parse_document(slice, "x.md").unwrap();
        assert_eq!(drafts.len(), 1, "{slice:?}");
        assert_eq!(
            drafts[0].version_id,
            r["version_id"].as_str().unwrap(),
            "{slice:?}"
        );
        assert_eq!(l["line_start"].as_u64().unwrap(), line_of(bytes, bs), "{r}");
        assert_eq!(
            l["line_end"].as_u64().unwrap(),
            line_of(bytes, be - 1),
            "{r}"
        );
        assert_eq!(l["content_hash"], rootmarkcore::hash32(text), "{r}");
    }
}

fn blame(k: &Kernel, doc: &str, at: Option<&str>) -> Value {
    let mut a = json!({ "doc_path": doc });
    if let Some(at) = at {
        a["at"] = json!(at);
    }
    k.call("blame", &a, reader()).unwrap()
}

/// `blame` and `read_block` on a document with every edge class (BOM, CRLF,
/// front matter, Turkish and emoji heading, nested list and callout, HTML
/// with a blank line, table, code fence, two embeds on one line, a footnote
/// defined at the end).
#[test]
fn blame_and_read_block_carry_locations_that_slice_the_blocks() {
    let v = vault("edges", &[("kenar.md", EDGE), ("plain.md", PLAIN)]);
    let k = Kernel::open(&v).unwrap();
    for (doc, bytes) in [("kenar.md", EDGE), ("plain.md", PLAIN)] {
        let b = blame(&k, doc, None);
        assert_eq!(b["loc_stale"], false, "{b}");
        let rows = b["blocks"].as_array().unwrap();
        assert!(rows.len() >= 4, "{b}");
        check_locs(bytes, rows);
        // ordered, not overlapping
        let mut end = 0;
        for r in rows {
            let bs = r["loc"]["byte_start"].as_u64().unwrap();
            assert!(bs >= end, "{r}");
            end = r["loc"]["byte_end"].as_u64().unwrap();
        }
        // read_block: the same location
        let r0 = &rows[1];
        let rb = k
            .call(
                "read_block",
                &json!({"doc_path": doc, "lineage_id": r0["lineage_id"]}),
                reader(),
            )
            .unwrap();
        assert_eq!(rb["block"]["loc"], r0["loc"], "{rb}");
        assert_eq!(rb["block"]["loc_stale"], false);
    }
    // the CLI's `--loc` gives the same locations; without it the output is
    // as before
    let vs = v.to_str().unwrap();
    let cli = ok_json(&["blame", "--loc", vs, "kenar.md"]);
    assert_eq!(cli["blocks"], blame(&k, "kenar.md", None)["blocks"]);
    let plain_cli = ok_json(&["blame", vs, "kenar.md"]);
    assert!(plain_cli.get("loc_stale").is_none());
    assert!(plain_cli["blocks"][0].get("loc").is_none());
    let rep = ok_json(&["replay", "--loc", vs, "kenar.md"]);
    assert_eq!(rep["loc_stale"], false);
    for (d, b) in rep["blocks_detail"]
        .as_array()
        .unwrap()
        .iter()
        .zip(cli["blocks"].as_array().unwrap())
    {
        assert_eq!(d["loc"], b["loc"]);
        assert_eq!(d["version_id"], b["version_id"]);
    }
    let rep_plain = ok_json(&["replay", vs, "kenar.md"]);
    assert!(rep_plain.get("blocks_detail").is_none());
}

/// With `loc: true` a search hit carries one location per lineage, in
/// `lineage_ids` order; every other field is the v1 hit. Without it the
/// response is the v1 response (the same hits, byte for byte).
#[test]
fn search_hits_carry_locations() {
    let v = vault("search", &[("plain.md", PLAIN), ("kenar.md", EDGE)]);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let r = k
        .call(
            "search",
            &json!({"query": "paragraph words", "mode": "lexical", "k": 5, "loc": true}),
            reader(),
        )
        .unwrap();
    let hits = r["hits"].as_array().unwrap();
    assert!(!hits.is_empty(), "{r}");
    let b = blame(&k, "plain.md", None);
    for h in hits {
        let keys: Vec<&str> = h.as_object().unwrap().keys().map(|s| s.as_str()).collect();
        assert_eq!(
            keys,
            [
                "chunk_id",
                "doc_path",
                "layer",
                "lineage_ids",
                "loc",
                "loc_stale",
                "score"
            ]
        );
        assert_eq!(h["loc_stale"], false);
        let ls = h["lineage_ids"].as_array().unwrap();
        let locs = h["loc"].as_array().unwrap();
        assert_eq!(ls.len(), locs.len());
        if h["doc_path"] == "plain.md" {
            for (l, loc) in ls.iter().zip(locs) {
                let row = b["blocks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["lineage_id"] == *l)
                    .unwrap();
                assert_eq!(row["loc"], *loc);
            }
        }
    }
    // the v1 hits: the CLI's search at the same checkpoint
    let cli = ok_json(&[
        "search",
        v.to_str().unwrap(),
        "paragraph words",
        "--mode",
        "lexical",
        "--k",
        "5",
        "--consumer",
        "cli",
    ]);
    let strip = |hs: &Value| -> Vec<Value> {
        hs.as_array()
            .unwrap()
            .iter()
            .map(|h| {
                let mut h = h.clone();
                let o = h.as_object_mut().unwrap();
                o.remove("loc");
                o.remove("loc_stale");
                h
            })
            .collect()
    };
    assert_eq!(strip(&r["hits"]), strip(&cli["hits"]));
    // without `loc`: the v1 hits as they are, same query_hash
    let v1 = k
        .call(
            "search",
            &json!({"query": "paragraph words", "mode": "lexical", "k": 5}),
            reader(),
        )
        .unwrap();
    assert_eq!(v1["hits"], json!(strip(&r["hits"])));
    assert_eq!(v1["query_hash"], r["query_hash"]);
}

/// The file changes on disk and is not ingested: `loc_stale: true`, and
/// the locations are the ingested version's (its bytes' `content_hash`),
/// never offsets applied to the new bytes. After the ingest: fresh. A
/// version whose locations were never recorded gives none.
#[test]
fn a_file_changed_before_its_ingest_gives_stale_locations() {
    let v = vault("stale", &[("plain.md", PLAIN)]);
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let fresh = blame(&k, "plain.md", None);
    let changed = b"---\ntitle: plain\n---\nA new first line pushes everything down.\n\n# Plain\n\nFirst paragraph.\n\n## Second\n\nSecond paragraph with words, edited.\n";
    std::fs::write(v.join("plain.md"), changed).unwrap();
    let stale = blame(&k, "plain.md", None);
    assert_eq!(stale["loc_stale"], true);
    // the same (ingested) blocks, the same locations: the old bytes'
    assert_eq!(stale["blocks"], fresh["blocks"]);
    check_locs(PLAIN, stale["blocks"].as_array().unwrap());
    assert_ne!(
        stale["blocks"][0]["loc"]["content_hash"],
        rootmarkcore::hash32(std::str::from_utf8(changed).unwrap())
    );
    // search: the hit says so too
    let s = k
        .call(
            "search",
            &json!({"query": "paragraph", "mode": "lexical", "loc": true}),
            reader(),
        )
        .unwrap();
    assert!(s["hits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["loc_stale"] == true));
    // a version with no recorded locations: no `loc`, still stale
    let locdir = v.join(".rootmark/loc");
    let aside = root("stale-aside");
    std::fs::rename(&locdir, &aside).unwrap();
    let none = blame(&k, "plain.md", None);
    assert_eq!(none["loc_stale"], true);
    assert!(none["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|b| b.get("loc").is_none()));
    std::fs::rename(&aside, &locdir).unwrap();
    // the ingest: fresh again, the new bytes' locations
    k.watch_doc("plain.md", "p2-b-b1").unwrap();
    let again = blame(&k, "plain.md", None);
    assert_eq!(again["loc_stale"], false);
    check_locs(changed, again["blocks"].as_array().unwrap());
    // blame at the checkpoint of the first version: stale (another
    // version), its recorded locations
    let cps = k.call("checkpoints", &json!({}), reader()).unwrap();
    let first = cps["checkpoints"][0]["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();
    let old = blame(&k, "plain.md", Some(&first));
    assert_eq!(old["loc_stale"], true);
    check_locs(PLAIN, old["blocks"].as_array().unwrap());
}

/// Locations write nothing: the log, the traces (beyond the search's own
/// line) and the markdown bytes are unchanged by `blame` and `read_block`;
/// the location records of two vaults ingested alike are byte-equal.
#[test]
fn locations_are_derived_and_deterministic() {
    let a = vault("det-a", &[("kenar.md", EDGE), ("plain.md", PLAIN)]);
    let b = vault("det-b", &[("kenar.md", EDGE), ("plain.md", PLAIN)]);
    let snap = |v: &Path| {
        (
            std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
            std::fs::read(v.join(".rootmark/traces.jsonl")).ok(),
            std::fs::read(v.join("kenar.md")).unwrap(),
        )
    };
    let before = snap(&a);
    let k = Kernel::open(&a).unwrap();
    for doc in ["kenar.md", "plain.md"] {
        blame(&k, doc, None);
    }
    assert_eq!(snap(&a), before);
    let files = |v: &Path| -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        let d = v.join(".rootmark/loc");
        for e in walk(&d) {
            out.push((
                e.strip_prefix(&d).unwrap().display().to_string(),
                std::fs::read(&e).unwrap(),
            ));
        }
        out.sort();
        out
    };
    let (fa, fb) = (files(&a), files(&b));
    assert_eq!(fa.len(), 2);
    assert_eq!(fa, fb);
    assert_eq!(
        ok_json(&["events-hash", a.to_str().unwrap()]),
        ok_json(&["events-hash", b.to_str().unwrap()])
    );
}

fn walk(d: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(d).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

/// P2-A's six old vaults (P2-0's three `v0.1.0` vaults, P2-0.1's two,
/// P2-0.2's one): `blame` with locations on every document, then
/// `events_hash` is the recorded one and the log is byte-equal. Their
/// versions were ingested before P2-B: a document whose file still holds
/// its version gets the file's locations; one whose file differs, none.
#[test]
fn the_six_old_vaults_keep_their_events_hash() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/data");
    for (set, name) in [
        ("p2_0_old_vaults", "e2e"),
        ("p2_0_old_vaults", "mixed"),
        ("p2_0_old_vaults", "scene"),
        ("p2_0_1_old_vaults", "attested"),
        ("p2_0_1_old_vaults", "bulk"),
        ("p2_0_2_old_vaults", "partial"),
    ] {
        let src = data.join(set).join(name);
        let expected: Value =
            serde_json::from_slice(&std::fs::read(src.join("expected.json")).unwrap()).unwrap();
        let v = root(&format!("old-{name}")).join("vault");
        copy_dir(&src.join("vault"), &v);
        copy_dir(&src.join("store"), &v.join(".rootmark"));
        let log = std::fs::read(v.join(".rootmark/events.jsonl")).unwrap();
        let vault = rootmarkcore::Vault::open(&v).unwrap();
        let docs = vault.doc_paths();
        drop(vault);
        let k = Kernel::open(&v).unwrap();
        let mut located = 0;
        for d in &docs {
            let Ok(b) = k.call("blame", &json!({"doc_path": d}), reader()) else {
                continue;
            };
            if b["loc_stale"] == false {
                located += 1;
                let bytes = std::fs::read(v.join(d)).unwrap();
                check_locs(&bytes, b["blocks"].as_array().unwrap());
            } else {
                assert!(b["blocks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|r| r.get("loc").is_none()));
            }
        }
        assert!(located > 0, "{name}");
        assert_eq!(
            ok_json(&["events-hash", v.to_str().unwrap()])["events_hash"],
            expected["events_hash"],
            "{name}"
        );
        assert_eq!(
            std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
            log
        );
        assert!(
            !v.join(".rootmark/loc").exists(),
            "{name}: blame wrote locations"
        );
    }
}
