//! P2-L/L3 (GOAL P2-L 4, L-7; CLIENT_CONTRACT §2.2): `search.path_prefix`.
//! - only documents under the prefixes are ranked; their order is the
//!   unfiltered ranking's among them (`lexical`, `flat`: the statistics stay
//!   the checkpoint's); `include_proposal` keeps its meaning (the canonical
//!   part equal to the same filtered search without it);
//! - normalized (NFC, trailing `/`, sorted, once): equal filters give equal
//!   `query_hash`es; a prefix matches a document or a folder, never a
//!   neighbour sharing its first letters (`notes` ≠ `notes2/`, `notesx.md`);
//! - `query_hash` of an unfiltered search is `H(query)` as before; of a
//!   filtered one `H(query + "\n" + {"path_prefix": […]})`;
//! - the trace of a filtered search carries the filter (`trace`), an
//!   unfiltered one none;
//! - security: an absolute path, `..`, `.`, an empty component, `\`, a
//!   drive, a control character, an empty list or prefix are
//!   `invalid_arguments` and record nothing — a prefix never addresses
//!   anything outside the vault.
//!
//! Needs the model files (`make models`).

use rootmarkserver::error::{code_of, Code};
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

const NOTES: &[(&str, &str)] = &[
    (
        "notes/espresso.md",
        "# Espresso\n\nEspresso grind and roast notes for the kalimba beans.\n",
    ),
    (
        "notes/sub/filter.md",
        "# Filter\n\nA filter roast, ground coarser than espresso.\n",
    ),
    (
        "notes2/espresso.md",
        "# Espresso two\n\nAnother espresso grind, in the other folder.\n",
    ),
    (
        "notesx.md",
        "# Notes X\n\nEspresso roast beside the folder.\n",
    ),
    (
        "kahve/çay.md",
        "# Çay\n\nEspresso yerine çay demleme notu.\n",
    ),
    ("top.md", "# Top\n\nEspresso at the top of the vault.\n"),
];

fn vault(name: &str) -> (PathBuf, Kernel) {
    let root = std::env::temp_dir().join(format!("rootmark-p2l3-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    for (p, t) in NOTES {
        let f = v.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["ingest-dir", vs, vs]).status.success());
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::default_recipe() + "\n[identity]\nhuman_id = \"ada\"\n",
    )
    .unwrap();
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    (v, k)
}

fn reader() -> Caller<'static> {
    Caller::mcp(Some("p2l3-reader/1.0"))
}

fn search(k: &Kernel, extra: Value) -> anyhow::Result<Value> {
    let mut a = json!({"query": "espresso roast", "mode": "lexical", "k": 20});
    for (key, val) in extra.as_object().unwrap() {
        a[key] = val.clone();
    }
    k.call("search", &a, reader())
}

fn docs(r: &Value) -> Vec<String> {
    r["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["doc_path"].as_str().unwrap().to_string())
        .collect()
}

fn traces(k: &Kernel) -> Vec<Value> {
    k.call("trace", &json!({}), reader()).unwrap()["traces"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn only_documents_under_the_prefixes_in_the_unfiltered_order() {
    let (_v, k) = vault("order");
    for mode in ["lexical", "flat", "hybrid"] {
        let all = search(&k, json!({"mode": mode})).unwrap();
        let f = search(&k, json!({"mode": mode, "path_prefix": ["notes"]})).unwrap();
        let got = docs(&f);
        assert!(!got.is_empty(), "{mode}");
        for d in &got {
            assert!(d.starts_with("notes/"), "{mode}: {d}");
        }
        assert!(got.contains(&"notes/sub/filter.md".to_string()) || mode != "lexical");
        if mode != "hybrid" {
            let restricted: Vec<String> = docs(&all)
                .into_iter()
                .filter(|d| d.starts_with("notes/"))
                .collect();
            assert_eq!(got, restricted, "{mode}");
            // the same scores: the statistics are the checkpoint's
            let score = |r: &Value, d: &str| {
                r["hits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|h| h["doc_path"] == d)
                    .map(|h| h["score"].clone())
            };
            for d in &got {
                assert_eq!(score(&f, d), score(&all, d), "{mode}: {d}");
            }
        }
    }
    // a document prefix, two prefixes, a non-ASCII folder
    assert_eq!(
        docs(&search(&k, json!({"path_prefix": ["top.md"]})).unwrap()),
        ["top.md"]
    );
    let two = docs(&search(&k, json!({"path_prefix": ["notes2", "kahve"]})).unwrap());
    assert!(
        two.iter()
            .all(|d| d.starts_with("notes2/") || d.starts_with("kahve/")),
        "{two:?}"
    );
    assert_eq!(two.len(), 2, "{two:?}");
    // no neighbour sharing the first letters
    let n = docs(&search(&k, json!({"path_prefix": ["notes"]})).unwrap());
    assert!(
        !n.iter()
            .any(|d| d.starts_with("notes2/") || d == "notesx.md"),
        "{n:?}"
    );
    // a prefix nothing lies under: no hit, a trace all the same
    let before = traces(&k).len();
    let none = search(&k, json!({"path_prefix": ["nowhere"]})).unwrap();
    assert!(docs(&none).is_empty());
    assert_eq!(traces(&k).len(), before + 1);
}

#[test]
fn normalized_filters_give_one_query_hash_and_the_unfiltered_hash_is_unchanged() {
    let (_v, k) = vault("hash");
    let plain = search(&k, json!({})).unwrap();
    assert_eq!(plain["query_hash"], rootmarkcore::hash32("espresso roast"));
    assert!(plain.get("filter").is_none(), "{plain:#}");
    let a = search(&k, json!({"path_prefix": ["notes", "kahve"]})).unwrap();
    let want = rootmarkcore::hash32(&format!(
        "espresso roast\n{}",
        json!({"path_prefix": ["kahve", "notes"]})
    ));
    assert_eq!(a["query_hash"], want);
    assert_eq!(a["filter"], json!({"path_prefix": ["kahve", "notes"]}));
    // trailing '/', order, duplicates and NFD (ç as c + U+0327) normalize away
    for same in [
        json!(["kahve/", "notes"]),
        json!(["notes", "kahve", "notes/"]),
        json!(["notes", "kahve"]),
    ] {
        let b = search(&k, json!({"path_prefix": same})).unwrap();
        assert_eq!(b["query_hash"], want, "{same}");
        assert_eq!(docs(&b), docs(&a), "{same}");
    }
    let nfc = search(&k, json!({"path_prefix": ["kahve/çay.md"]})).unwrap();
    let nfd = search(&k, json!({"path_prefix": ["kahve/c\u{327}ay.md"]})).unwrap();
    assert_eq!(nfc["query_hash"], nfd["query_hash"]);
    assert_eq!(docs(&nfd), ["kahve/çay.md"]);
}

#[test]
fn the_trace_carries_the_filter() {
    let (_v, k) = vault("trace");
    let plain = search(&k, json!({})).unwrap();
    let f = search(&k, json!({"path_prefix": ["notes/sub"]})).unwrap();
    let ts = traces(&k);
    let by = |id: &Value| ts.iter().find(|t| &t["trace_id"] == id).unwrap().clone();
    assert!(by(&plain["trace_id"]).get("filter").is_none());
    let t = by(&f["trace_id"]);
    assert_eq!(t["filter"], json!({"path_prefix": ["notes/sub"]}));
    assert_eq!(t["query_hash"], f["query_hash"]);
    // since_seq and consumer still select
    let later = k
        .call("trace", &json!({"since_seq": t["at_seq"]}), reader())
        .unwrap();
    assert_eq!(later["traces"].as_array().unwrap().len(), 1);
    assert_eq!(later["traces"][0]["filter"], t["filter"]);
}

#[test]
fn a_damaged_filter_record_is_an_error() {
    let (v, k) = vault("damaged");
    search(&k, json!({"path_prefix": ["notes"]})).unwrap();
    let p = v.join(".rootmark/trace_filters.jsonl");
    let line = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, line.replace("\"at_seq\":", "\"at_seq\":9")).unwrap();
    assert!(k.call("trace", &json!({}), reader()).is_err());
}

#[test]
fn include_proposal_keeps_its_meaning_under_a_filter() {
    let (_v, k) = vault("proposal");
    k.call(
        "propose",
        &json!({"doc_path": "notes/agent.md", "content": "# Agent\n\nAgent espresso roast advice.", "reason": "p2l3"}),
        Caller::mcp(Some("p2l3-agent/1.0")),
    )
    .unwrap();
    k.call(
        "propose",
        &json!({"doc_path": "other/agent.md", "content": "# Other\n\nAgent espresso roast elsewhere.", "reason": "p2l3"}),
        Caller::mcp(Some("p2l3-agent/1.0")),
    )
    .unwrap();
    let without = search(&k, json!({"path_prefix": ["notes"]})).unwrap();
    let with = search(
        &k,
        json!({"path_prefix": ["notes"], "include_proposal": true}),
    )
    .unwrap();
    let canon: Vec<Value> = with["hits"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["layer"] == "canonical")
        .cloned()
        .collect();
    assert_eq!(&canon, without["hits"].as_array().unwrap());
    let props: Vec<String> = with["hits"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["layer"] != "canonical")
        .map(|h| h["doc_path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(props, ["notes/agent.md"]);
}

#[test]
fn a_prefix_outside_the_vault_is_refused_and_records_nothing() {
    let (v, k) = vault("security");
    search(&k, json!({})).unwrap();
    let before = std::fs::read(v.join(".rootmark/traces.jsonl")).unwrap();
    let events = std::fs::read(v.join(".rootmark/events.jsonl")).unwrap();
    for bad in [
        json!([]),
        json!([""]),
        json!(["/"]),
        json!(["/etc"]),
        json!(["../vault"]),
        json!(["notes/../.."]),
        json!(["notes/./sub"]),
        json!(["./notes"]),
        json!(["notes//sub"]),
        json!(["notes\\sub"]),
        json!(["\\\\server\\share"]),
        json!(["C:/Windows"]),
        json!(["c:"]),
        json!(["notes\u{0}"]),
        json!(["notes\n"]),
        json!(["notes", ".."]),
        json!([".rootmark/../.."]),
        json!("notes"),
        json!([1]),
    ] {
        let e = search(&k, json!({"path_prefix": bad})).unwrap_err();
        assert!(
            matches!(code_of(&e), Code::InvalidArguments),
            "{bad}: {:?} {e:#}",
            code_of(&e)
        );
    }
    let many: Vec<String> = (0..65).map(|i| format!("p{i}")).collect();
    assert_eq!(
        code_of(&search(&k, json!({"path_prefix": many})).unwrap_err()),
        Code::InvalidArguments
    );
    assert_eq!(
        std::fs::read(v.join(".rootmark/traces.jsonl")).unwrap(),
        before
    );
    assert_eq!(
        std::fs::read(v.join(".rootmark/events.jsonl")).unwrap(),
        events
    );
    assert!(!v.join(".rootmark/trace_filters.jsonl").exists());
}
