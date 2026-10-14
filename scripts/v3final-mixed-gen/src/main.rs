//! Writes `core/tests/fixtures/v3_final_mixed/<scenario>/` from the
//! **v3-final** core library (P1-B.1 TASKS B1.1; GOAL B1-1): every scenario of
//! `scenarios.json` is ingested step by step through the v3-final public API
//! (`Vault::ingest(doc, text, Provenance { author_kind, source }, V3)`), the
//! vault's store is copied to `<scenario>/store/` (the `.mdkernel/` directory,
//! renamed so that it is committed and never opened in place), and
//! `<scenario>/expected.json` records what v3-final itself reports: every
//! document's `replay` snapshot_id and the `events_hash` of `events.jsonl`.
//!
//! Built and run by scripts/gen_v3final_mixed.sh next to
//! `git archive v3-final core`; never part of this repository's workspace.
//!
//!   mdk-v3final-mixed-gen <scenarios.json> <out-dir> <v3-final-sha>

use anyhow::{bail, Context, Result};
use mdkcore::{AuthorKind, MatcherVersion, Provenance, Vault};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn author(s: &str) -> Result<AuthorKind> {
    Ok(match s {
        "Human" => AuthorKind::Human,
        "Agent" => AuthorKind::Agent,
        "Transcript" => AuthorKind::Transcript,
        "Synthetic" => AuthorKind::Synthetic,
        _ => bail!("unknown author {s:?}"),
    })
}

fn str_of<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string {k:?}"))
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    let mut entries: Vec<_> = fs::read_dir(from)?.collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type()?.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        bail!("usage: mdk-v3final-mixed-gen <scenarios.json> <out-dir> <v3-final-sha>");
    }
    let spec: Value = serde_json::from_str(&fs::read_to_string(&args[1])?)?;
    let out = Path::new(&args[2]);
    let sha = &args[3];
    if str_of(&spec, "matcher")? != "v3" {
        bail!("scenarios.json: matcher v3 expected");
    }
    let scenarios = spec["scenarios"]
        .as_array()
        .context("scenarios.json: no scenarios")?;
    for sc in scenarios {
        let name = str_of(sc, "name")?;
        let dir = out.join(name);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        let work = out.join(format!(".work-{name}"));
        if work.exists() {
            fs::remove_dir_all(&work)?;
        }
        let mut v = Vault::open(&work)?;
        let mut steps = Vec::new();
        for st in sc["steps"].as_array().context("no steps")? {
            let (doc, a, src) = (str_of(st, "doc")?, str_of(st, "author")?, str_of(st, "source")?);
            let prov = Provenance {
                author_kind: author(a)?,
                source: src.to_string(),
            };
            let r = v.ingest(doc, str_of(st, "text")?, prov, MatcherVersion::V3)?;
            steps.push(json!({"doc": doc, "author": a, "events": r.events.len()}));
        }
        // what v3-final reports for the vault it wrote (a fresh open = replay)
        let v = Vault::open(&work)?;
        let mut docs = BTreeMap::new();
        for d in v.doc_paths() {
            let s = v.replay(&d)?.with_context(|| format!("{name}: {d} has no snapshot"))?;
            docs.insert(d, s.snapshot_id);
        }
        let log = fs::read(v.events_path())?;
        let expected = json!({
            "scenario": name,
            "title": str_of(sc, "title")?,
            "v3_final_sha": sha,
            "matcher": "v3",
            "steps": steps,
            "events": log.iter().filter(|&&b| b == b'\n').count(),
            "events_hash": v.events_hash()?,
            "replay_snapshot_ids": docs,
        });
        copy_tree(&work.join(".mdkernel"), &dir.join("store"))?;
        fs::write(
            dir.join("expected.json"),
            serde_json::to_string_pretty(&expected)? + "\n",
        )?;
        fs::remove_dir_all(&work)?;
        println!("{name}: {} events", expected["events"]);
    }
    Ok(())
}
