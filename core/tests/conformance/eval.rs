//! Conformance case evaluator (spec/CONFORMANCE.md, Level 1).
//!
//! One source, compiled twice:
//! - by `scripts/gen_fixtures.sh` against the **v3-final** core (prototype sha
//!   in docs/rounds/P1-A/progress/SOURCE.md) to write `expected.json`;
//! - by `core/tests/conformance.rs` against the current core to compare.
//!
//! It therefore uses only the core API that exists in v3-final, and builds
//! every output field explicitly (never serializes a core struct whose shape
//! may grow, e.g. `Block`). Every case carries its own matcher version and
//! parameters (`case.json: matcher`); nothing reads compiled defaults.

use anyhow::{anyhow, bail, Context, Result};
use rootmarkcore::lineage::lineage_formula;
use rootmarkcore::store::{self, BlockObject};
use rootmarkcore::{
    chunk, extract_wikilinks, normalize, parse_document, version_id, AuthorKind, BlockType, Event,
    MatchParams, MatcherVersion, Provenance, Strategy, Vault,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// `(doc_path, [(lineage_id, version_id)])` of every head snapshot.
type Heads = Vec<(String, Vec<(String, String)>)>;

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

/// Fresh empty scratch directory (removed by `Scratch::drop`).
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Result<Scratch> {
        let n = SCRATCH.fetch_add(1, Ordering::SeqCst);
        let d =
            std::env::temp_dir().join(format!("rootmark-conf-{}-{n}-{tag}", std::process::id()));
        if d.exists() {
            fs::remove_dir_all(&d)?;
        }
        fs::create_dir_all(&d)?;
        Ok(Scratch(d))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn s<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("case field {key:?} missing or not a string"))
}

fn arr<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("case field {key:?} missing or not an array"))
}

fn read(dir: &Path, rel: &str) -> Result<String> {
    let p = dir.join(rel);
    fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))
}

/// `case.json: matcher = {version, theta, theta_split}` — all three required.
fn matcher_of(case: &Value) -> Result<(MatcherVersion, MatchParams)> {
    let m = case
        .get("matcher")
        .ok_or_else(|| anyhow!("case has no matcher block"))?;
    let version = MatcherVersion::parse(s(m, "version")?)?;
    let f = |k: &str| {
        m.get(k)
            .and_then(Value::as_f64)
            .ok_or_else(|| anyhow!("matcher.{k} missing"))
    };
    Ok((
        version,
        MatchParams {
            theta: f("theta")?,
            theta_split: f("theta_split")?,
        },
    ))
}

fn author(kind: &str) -> Result<AuthorKind> {
    Ok(match kind {
        "human" => AuthorKind::Human,
        "agent" => AuthorKind::Agent,
        "transcript" => AuthorKind::Transcript,
        "synthetic" => AuthorKind::Synthetic,
        _ => bail!("unknown author_kind {kind:?}"),
    })
}

fn event_json(e: &Event) -> Result<Value> {
    Ok(serde_json::to_value(e)?)
}

fn kind_str(e: &Event) -> Result<String> {
    Ok(serde_json::to_value(e.kind)?
        .as_str()
        .ok_or_else(|| anyhow!("event kind is not a string"))?
        .to_string())
}

fn events_bytes(root: &Path) -> Result<Vec<u8>> {
    let p = root.join(".rootmark").join("events.jsonl");
    fs::read(&p).with_context(|| format!("read {}", p.display()))
}

fn blake32(bytes: &[u8]) -> String {
    let mut h = blake3::hash(bytes).to_hex().to_string();
    h.truncate(32);
    h
}

/// Evaluates one case directory and returns its output document.
pub fn evaluate(case_dir: &Path) -> Result<Value> {
    let case: Value = serde_json::from_str(&read(case_dir, "case.json")?)
        .with_context(|| format!("{}/case.json", case_dir.display()))?;
    let out = match s(&case, "kind")? {
        "normalize" => normalize_case(&case)?,
        "parse" => parse_case(case_dir, &case)?,
        "wikilinks" => wikilinks_case(&case)?,
        "version_id" => version_id_case(&case)?,
        "lineage_formula" => lineage_formula_case(case_dir, &case)?,
        "object_store" => object_store_case()?,
        "sequence" => sequence_case(case_dir, &case)?,
        "mixed_versions" => mixed_versions_case(case_dir, &case)?,
        "chunk_rebuild" => chunk_rebuild_case(case_dir, &case)?,
        "vault_determinism" => vault_determinism_case(case_dir, &case)?,
        k => bail!("unknown case kind {k:?}"),
    };
    Ok(json!({"id": s(&case, "id")?, "output": out}))
}

// C1-01
fn normalize_case(case: &Value) -> Result<Value> {
    let mut out = Vec::new();
    for inp in arr(case, "inputs")? {
        let text = s(inp, "text")?;
        let is_code = inp.get("is_code").and_then(Value::as_bool).unwrap_or(false);
        out.push(
            json!({"text": text, "is_code": is_code, "normalized_text": normalize(text, is_code)}),
        );
    }
    Ok(Value::Array(out))
}

// C1-02
fn parse_case(dir: &Path, case: &Value) -> Result<Value> {
    let mut docs = Vec::new();
    for f in arr(case, "documents")? {
        let f = f
            .as_str()
            .ok_or_else(|| anyhow!("documents: not a string"))?;
        let (blocks, frontmatter) = parse_document(&read(dir, f)?, f)?;
        let blocks: Vec<Value> = blocks
            .iter()
            .map(|b| {
                json!({
                    "block_type": b.block_type.as_str(),
                    "normalized_text": b.normalized_text,
                    "heading_path": b.heading_path,
                    "heading_level": b.heading_level,
                    "version_id": b.version_id,
                    "line_start": b.line_start,
                    "line_end": b.line_end,
                })
            })
            .collect();
        docs.push(json!({"document": f, "frontmatter": frontmatter, "blocks": blocks}));
    }
    Ok(Value::Array(docs))
}

// C1-03
fn wikilinks_case(case: &Value) -> Result<Value> {
    let mut out = Vec::new();
    for t in arr(case, "inputs")? {
        let t = t.as_str().ok_or_else(|| anyhow!("inputs: not a string"))?;
        let links: Vec<Value> = extract_wikilinks(t)
            .iter()
            .map(|l| json!({"target": l.target, "heading": l.heading}))
            .collect();
        out.push(json!({"text": t, "links": links}));
    }
    Ok(Value::Array(out))
}

// C1-04
fn version_id_case(case: &Value) -> Result<Value> {
    let mut out = Vec::new();
    for inp in arr(case, "inputs")? {
        let bt: BlockType = serde_json::from_value(json!(s(inp, "block_type")?))?;
        let text = s(inp, "normalized_text")?;
        // spec §3 formula computed independently of the core helper
        let formula = blake32(format!("{}\n{}", bt.as_str(), text).as_bytes());
        let vid = version_id(bt, text);
        if vid != formula {
            bail!("version_id {vid} != H(block_type + \"\\n\" + text) {formula}");
        }
        out.push(json!({"block_type": bt.as_str(), "normalized_text": text, "version_id": vid}));
    }
    Ok(Value::Array(out))
}

// C1-05
fn lineage_formula_case(dir: &Path, case: &Value) -> Result<Value> {
    let (m, params) = matcher_of(case)?;
    let doc = s(case, "doc_path")?;
    let sc = Scratch::new("lin")?;
    let mut v = Vault::open_with(&sc.0, params)?;
    let r = v.ingest(doc, &read(dir, s(case, "file")?)?, Provenance::default(), m)?;
    let blocks = v.snapshot_blocks(&r.snapshot)?;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for b in &blocks {
        let k = *seen.get(&b.version_id).unwrap_or(&0);
        seen.insert(b.version_id.clone(), k + 1);
        // spec §3 formula computed independently of the core helper
        let formula = blake32(format!("lineage\n{doc}\n{}\n{k}", b.version_id).as_bytes());
        if b.lineage_id != formula || lineage_formula(doc, &b.version_id, k) != formula {
            bail!(
                "lineage_id of {} (k={k}) does not follow spec §3",
                b.version_id
            );
        }
        out.push(json!({"version_id": b.version_id, "k": k, "lineage_id": b.lineage_id}));
    }
    Ok(json!({"doc_path": doc, "blocks": out, "snapshot_id": r.snapshot.snapshot_id}))
}

// C1-06
fn object_store_case() -> Result<Value> {
    let sc = Scratch::new("store")?;
    let text = "Content addressed block.".to_string();
    let obj = BlockObject {
        version_id: version_id(BlockType::Paragraph, &text),
        block_type: BlockType::Paragraph,
        normalized_text: text,
        heading_level: None,
    };
    let path = store::object_path(&sc.0, &obj.version_id);
    let outcome = |r: Result<()>| if r.is_ok() { "ok" } else { "error" };
    let first = outcome(store::put_object(&sc.0, &obj));
    let bytes1 = fs::read(&path)?;
    let identical = outcome(store::put_object(&sc.0, &obj));
    let unchanged_after_identical = fs::read(&path)? == bytes1;
    let mut other = bytes1.clone();
    other.extend_from_slice(b" ");
    let conflicting = outcome(store::write_immutable(&path, &other));
    let unchanged_after_conflict = fs::read(&path)? == bytes1;
    let mut bad = obj.clone();
    bad.normalized_text.push('!');
    let wrong_hash = outcome(store::put_object(&sc.0, &bad));
    Ok(json!({
        "version_id": obj.version_id,
        "object_bytes_blake3": blake32(&bytes1),
        "first_write": first,
        "identical_write": identical,
        "unchanged_after_identical": unchanged_after_identical,
        "conflicting_write": conflicting,
        "unchanged_after_conflict": unchanged_after_conflict,
        "id_not_matching_content": wrong_hash,
    }))
}

/// Ingests `case.steps` = [{doc_path, file, author_kind?}] into a fresh vault.
fn run_steps(dir: &Path, case: &Value, v: &mut Vault, m: MatcherVersion) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    const STEP_KEYS: [&str; 7] = [
        "doc_path",
        "file",
        "author_kind",
        "source",
        "must_contain",
        "must_not_contain",
        "note",
    ];
    for (i, st) in arr(case, "steps")?.iter().enumerate() {
        let obj = st
            .as_object()
            .ok_or_else(|| anyhow!("step {i}: not an object"))?;
        if let Some(k) = obj.keys().find(|k| !STEP_KEYS.contains(&k.as_str())) {
            bail!("step {i}: unknown key {k:?}");
        }
        let doc = s(st, "doc_path")?;
        let prov = Provenance {
            author_kind: author(
                st.get("author_kind")
                    .and_then(Value::as_str)
                    .unwrap_or("human"),
            )?,
            source: st
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        };
        let r = v.ingest(doc, &read(dir, s(st, "file")?)?, prov, m)?;
        let kinds = r.events.iter().map(kind_str).collect::<Result<Vec<_>>>()?;
        if let Some(req) = st
            .get("must_contain")
            .map(|v| {
                v.as_array()
                    .ok_or_else(|| anyhow!("step {i}: must_contain must be an array"))
            })
            .transpose()?
        {
            for k in req {
                let k = k
                    .as_str()
                    .ok_or_else(|| anyhow!("must_contain: not a string"))?;
                if !kinds.iter().any(|x| x == k) {
                    bail!("step {i}: fixture intent {k} not produced (got {kinds:?})");
                }
            }
        }
        if let Some(forbid) = st
            .get("must_not_contain")
            .map(|v| {
                v.as_array()
                    .ok_or_else(|| anyhow!("step {i}: must_not_contain must be an array"))
            })
            .transpose()?
        {
            for k in forbid {
                let k = k
                    .as_str()
                    .ok_or_else(|| anyhow!("must_not_contain: not a string"))?;
                if kinds.iter().any(|x| x == k) {
                    bail!("step {i}: fixture forbids {k} (got {kinds:?})");
                }
            }
        }
        let events = r
            .events
            .iter()
            .map(event_json)
            .collect::<Result<Vec<_>>>()?;
        out.push(json!({
            "step": i,
            "doc_path": doc,
            "snapshot_id": r.snapshot.snapshot_id,
            "lineage_ids": r.snapshot.blocks.iter().map(|(l, _)| l).collect::<Vec<_>>(),
            "version_ids": r.snapshot.blocks.iter().map(|(_, v)| v).collect::<Vec<_>>(),
            "event_kinds": kinds,
            "events": events,
        }));
    }
    Ok(out)
}

/// Every document's head `snapshot_id` and the one rebuilt from the log.
fn replay_check(v: &Vault, root: &Path) -> Result<Value> {
    let mut out = serde_json::Map::new();
    // a fresh open replays the on-disk log alone
    let reopened = Vault::open(root)?;
    for doc in v.doc_paths() {
        let head = v
            .snapshot(&doc)?
            .ok_or_else(|| anyhow!("{doc}: no head"))?
            .snapshot_id;
        let replayed = reopened
            .replay(&doc)?
            .ok_or_else(|| anyhow!("{doc}: replay produced nothing"))?
            .snapshot_id;
        if replayed != head {
            bail!("{doc}: replayed snapshot {replayed} != head {head}");
        }
        out.insert(doc, json!({"head": head, "replayed": replayed}));
    }
    Ok(Value::Object(out))
}

// C1-07 .. C1-14
fn sequence_case(dir: &Path, case: &Value) -> Result<Value> {
    let (m, params) = matcher_of(case)?;
    let sc = Scratch::new("seq")?;
    let mut v = Vault::open_with(&sc.0, params)?;
    let steps = run_steps(dir, case, &mut v, m)?;
    Ok(json!({
        "steps": steps,
        "replay": replay_check(&v, &sc.0)?,
        "events_hash": v.events_hash()?,
    }))
}

// C1-15
fn mixed_versions_case(dir: &Path, case: &Value) -> Result<Value> {
    let (first, params) = matcher_of(case)?;
    let second = MatcherVersion::parse(s(case, "second_version")?)?;
    let text = read(dir, s(case, "file")?)?;
    let sc = Scratch::new("mixed")?;
    let mut v = Vault::open_with(&sc.0, params)?;
    v.ingest("a.md", &text, Provenance::default(), first)?;
    let before = events_bytes(&sc.0)?;
    let same_handle = v
        .ingest("b.md", &text, Provenance::default(), second)
        .err()
        .map(|e| e.to_string());
    let mut reopened = Vault::open_with(&sc.0, params)?;
    let after_reopen = reopened
        .ingest("b.md", &text, Provenance::default(), second)
        .err()
        .map(|e| e.to_string());
    let log_unchanged = events_bytes(&sc.0)? == before;
    if same_handle.is_none() || after_reopen.is_none() {
        bail!("mixed matcher versions were accepted");
    }
    Ok(json!({
        "first_version": first.as_str(),
        "second_version": second.as_str(),
        "error_same_handle": same_handle,
        "error_after_reopen": after_reopen,
        "log_unchanged": log_unchanged,
    }))
}

fn ingest_dir_steps(dir: &Path, case: &Value, v: &mut Vault, m: MatcherVersion) -> Result<()> {
    for d in arr(case, "ingest_dirs")? {
        let d = d
            .as_str()
            .ok_or_else(|| anyhow!("ingest_dirs: not a string"))?;
        v.ingest_dir(&dir.join(d), m)?;
    }
    Ok(())
}

// C1-16
fn chunk_rebuild_case(dir: &Path, case: &Value) -> Result<Value> {
    let (m, params) = matcher_of(case)?;
    let sc = Scratch::new("chunk")?;
    let mut v = Vault::open_with(&sc.0, params)?;
    ingest_dir_steps(dir, case, &mut v, m)?;
    let log_before = events_bytes(&sc.0)?;
    let heads = |v: &Vault| -> Result<Heads> {
        v.doc_paths()
            .into_iter()
            .map(|d| {
                let snap = v.snapshot(&d)?.ok_or_else(|| anyhow!("{d}: no head"))?;
                Ok((d, snap.blocks))
            })
            .collect()
    };
    let lineages_before = heads(&v)?;
    let mut per_strategy = serde_json::Map::new();
    // structural → fixed → structural: rebuilding with another strategy and back
    for (pass, name) in ["structural", "fixed", "structural"].iter().enumerate() {
        let strategy = Strategy::parse(name)?;
        let mut docs = serde_json::Map::new();
        for d in v.doc_paths() {
            let snap = v.snapshot(&d)?.ok_or_else(|| anyhow!("{d}: no head"))?;
            let chunks: Vec<Value> = chunk(&snap, &v, strategy)?
                .iter()
                .map(|c| {
                    json!({
                        "chunk_id": c.chunk_id,
                        "lineage_ids": c.lineage_ids,
                        "version_ids": c.version_ids,
                        "heading_path": c.heading_path,
                        "token_count": c.token_count,
                        "text_blake3": blake32(c.text.as_bytes()),
                    })
                })
                .collect();
            docs.insert(d, Value::Array(chunks));
        }
        per_strategy.insert(format!("{pass}:{name}"), Value::Object(docs));
    }
    let reopened = Vault::open(&sc.0)?;
    Ok(json!({
        "chunks": per_strategy,
        "events_unchanged": events_bytes(&sc.0)? == log_before,
        "lineages_unchanged": heads(&v)? == lineages_before && heads(&reopened)? == lineages_before,
        "events_hash": v.events_hash()?,
    }))
}

// C1-17
fn vault_determinism_case(dir: &Path, case: &Value) -> Result<Value> {
    let (m, params) = matcher_of(case)?;
    let runs = 2;
    let mut logs = Vec::new();
    for i in 0..runs {
        let sc = Scratch::new(&format!("det{i}"))?;
        let mut v = Vault::open_with(&sc.0, params)?;
        ingest_dir_steps(dir, case, &mut v, m)?;
        logs.push(events_bytes(&sc.0)?);
    }
    let identical = logs.windows(2).all(|w| w[0] == w[1]);
    if !identical {
        bail!("two ingests of the same vault produced different events.jsonl bytes");
    }
    Ok(json!({
        "runs": runs,
        "identical_bytes": identical,
        "events_jsonl_blake3": blake32(&logs[0]),
        "event_lines": logs[0].iter().filter(|&&b| b == b'\n').count(),
    }))
}

/// blake3[:32] of every file under the fixture root (`/`-separated relative
/// paths, sorted), except `MANIFEST.json` itself. `expected.json` files are
/// included: the manifest freezes inputs and expected values alike.
pub fn fixture_manifest(root: &Path) -> Result<BTreeMap<String, String>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name == ".rootmark" || name == ".DS_Store" {
                continue;
            }
            if e.file_type()?.is_dir() {
                walk(base, &p, out)?;
            } else {
                let rel = p
                    .strip_prefix(base)?
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("/");
                if rel != "MANIFEST.json" {
                    out.insert(rel, blake32(&fs::read(&p)?));
                }
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out)?;
    Ok(out)
}

/// The C1 case directories under the fixture root, in id order.
pub fn case_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("case.json").is_file())
        .collect();
    out.sort();
    Ok(out)
}
