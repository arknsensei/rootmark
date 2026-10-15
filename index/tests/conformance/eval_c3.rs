//! Conformance Level 3 evaluator (spec/CONFORMANCE.md C3-01..C3-04; spec §9).
//!
//! A C3 case (`spec/fixtures/C3-xx/case.c3.json`) is a script on one fresh
//! vault; the output is one observation per operation. Expected outputs are
//! written from the current implementation by `scripts/gen_fixtures_c3.sh`
//! (`conformance_c3_gen`) and compared by `index/tests/conformance_c3.rs`.
//! A ranking is observed as `[doc_path, chunk_id, layer]` per hit, in order
//! — scores are not part of the observation (rankings are the normative
//! result, spec §9, CONFORMANCE "ranking equals fixture").
//!
//! Operations (`op`):
//! - `ingest_dir {dir}` — every `*.md` under `dir` (relative to the case),
//!   human-authored, byte-wise path order
//! - `ingest {doc, file, author, source}`
//! - `demote {doc, block, to, reason}` — human actor, human-only policy
//! - `checkpoint {name}`
//! - `build` — `index build`; `rebuild` — delete the index, then build
//! - `search {at, mode, k, include_proposal, queries, repeat?}` — one
//!   ranking per query (`repeat` n: every run must give the same ranking;
//!   observed as `runs_equal`), with the number of `derived` and the
//!   positions of `proposal` hits; every search records a trace
//! - `view {at}` — number of chunks per layer at the checkpoint (so that a
//!   "never returned" observation is not vacuous)
//! - `traces` — number of trace lines
//!
//! `label` on a search stores its rankings; `same_as` on a later search
//! records whether its rankings equal that label's.

use anyhow::{anyhow, bail, Context, Result};
use mdkcore::{
    Actor, ActorKind, AuthorKind, HumanOnly, Layer, MatcherVersion, Provenance, TransitionKind,
    Vault,
};
use mdkindex::embed::OnnxMiniLm;
use mdkindex::search::{Mode, Searcher};
use mdkindex::store::{self, Store};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

fn embedder() -> Result<&'static OnnxMiniLm> {
    static E: OnceLock<std::result::Result<OnnxMiniLm, String>> = OnceLock::new();
    E.get_or_init(|| OnnxMiniLm::load_default().map_err(|e| format!("{e:#}")))
        .as_ref()
        .map_err(|e| anyhow!("embedder: {e} (run `make models`)"))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Result<Scratch> {
        let n = SCRATCH.fetch_add(1, Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("mdk-conf3-{}-{n}-{tag}", std::process::id()));
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
        .ok_or_else(|| anyhow!("field {key:?} missing or not a string"))
}

fn author(kind: &str) -> Result<AuthorKind> {
    Ok(match kind {
        "human" => AuthorKind::Human,
        "agent" => AuthorKind::Agent,
        "transcript" => AuthorKind::Transcript,
        "synthetic" => AuthorKind::Synthetic,
        _ => bail!("unknown author {kind:?}"),
    })
}

fn matcher_of(case: &Value) -> Result<MatcherVersion> {
    MatcherVersion::parse(s(case.get("matcher").context("matcher")?, "version")?)
}

fn layer_name(l: Layer) -> &'static str {
    l.as_str()
}

/// Evaluates one C3 case directory.
pub fn evaluate_c3(dir: &Path) -> Result<Value> {
    let case: Value = serde_json::from_str(&fs::read_to_string(dir.join("case.c3.json"))?)
        .with_context(|| format!("{}/case.c3.json", dir.display()))?;
    let id = s(&case, "id")?.to_string();
    let m = matcher_of(&case)?;
    let scratch = Scratch::new(&id)?;
    let root = scratch.0.clone();
    let mut v = Vault::open(&root)?;
    let e = embedder()?;
    let mut labels: BTreeMap<String, Value> = BTreeMap::new();
    let mut out = Vec::new();
    for (i, op) in case
        .get("ops")
        .and_then(Value::as_array)
        .context("ops")?
        .iter()
        .enumerate()
    {
        let name = s(op, "op")?;
        let obs = (|| -> Result<Value> {
            Ok(match name {
                "ingest_dir" => {
                    let d = dir.join(s(op, "dir")?);
                    let r = v.ingest_dir(&d, m)?;
                    json!({"documents": r.len()})
                }
                "ingest" => {
                    let text = fs::read_to_string(dir.join(s(op, "file")?))?;
                    let prov = Provenance {
                        author_kind: author(s(op, "author")?)?,
                        source: s(op, "source")?.to_string(),
                    };
                    let r = v.ingest(s(op, "doc")?, &text, prov, m)?;
                    json!({"snapshot_id": r.snapshot.snapshot_id})
                }
                "demote" => {
                    let doc = s(op, "doc")?;
                    let b = op.get("block").and_then(Value::as_u64).context("block")? as usize;
                    let snap = v.snapshot(doc)?.context("head")?;
                    let lin = snap.blocks.get(b).context("block index")?.0.clone();
                    let req = v.request(
                        TransitionKind::Demote,
                        &lin,
                        Layer::parse(s(op, "to")?)?,
                        Actor::new(ActorKind::Human, "conformance")?,
                        s(op, "reason")?,
                    )?;
                    v.demote(&req, &HumanOnly)?;
                    json!({"lineage_id": lin})
                }
                "checkpoint" => {
                    let c = v.checkpoint(Some(s(op, "name")?))?;
                    json!({"checkpoint_id": c.checkpoint_id})
                }
                "build" => {
                    let r = store::build(&v, e)?;
                    json!({"records": r.records, "snapshots": r.snapshots})
                }
                "rebuild" => {
                    fs::remove_dir_all(store::index_dir(&v))?;
                    let r = store::build(&v, e)?;
                    json!({"records": r.records, "snapshots": r.snapshots, "embedded": r.embedded})
                }
                "search" => {
                    let st = Store::open(&v, e)?;
                    let mut se = Searcher::new(&st, e, "conformance")?;
                    let at = s(op, "at")?;
                    let mode = Mode::parse(s(op, "mode")?)?;
                    let k = op.get("k").and_then(Value::as_u64).context("k")? as usize;
                    let inc = op
                        .get("include_proposal")
                        .and_then(Value::as_bool)
                        .context("include_proposal")?;
                    let repeat = op.get("repeat").and_then(Value::as_u64).unwrap_or(1).max(1);
                    let mut rankings = Vec::new();
                    let mut runs_equal = true;
                    let mut derived = 0usize;
                    let mut proposal_after_canonical = true;
                    for q in op
                        .get("queries")
                        .and_then(Value::as_array)
                        .context("queries")?
                    {
                        let q = q.as_str().context("query")?;
                        let mut first: Option<Value> = None;
                        for _ in 0..repeat {
                            let r = se.search(&mut v, mode, q, k, Some(at), inc)?;
                            let mut seen_prop = false;
                            let ranking: Vec<Value> = r
                                .hits
                                .iter()
                                .map(|h| {
                                    match h.layer {
                                        Layer::Derived => derived += 1,
                                        Layer::Proposal => seen_prop = true,
                                        Layer::Canonical => {
                                            if seen_prop {
                                                proposal_after_canonical = false;
                                            }
                                        }
                                    }
                                    json!([h.doc_path, h.chunk_id, layer_name(h.layer)])
                                })
                                .collect();
                            let ranking = Value::Array(ranking);
                            match &first {
                                None => first = Some(ranking),
                                Some(f) => runs_equal &= *f == ranking,
                            }
                        }
                        rankings.push(json!({"query": q, "ranking": first}));
                    }
                    let rankings = Value::Array(rankings);
                    let mut obs = json!({
                        "mode": mode.as_str(),
                        "include_proposal": inc,
                        "rankings": rankings,
                        "runs": repeat,
                        "runs_equal": runs_equal,
                        "derived_hits": derived,
                        "proposals_after_canonical": proposal_after_canonical,
                    });
                    if let Some(l) = op.get("same_as").and_then(Value::as_str) {
                        let other = labels
                            .get(l)
                            .with_context(|| format!("unknown label {l:?}"))?;
                        obs["same_as"] = json!({"label": l, "equal": *other == obs["rankings"]});
                    }
                    if let Some(l) = op.get("label").and_then(Value::as_str) {
                        labels.insert(l.to_string(), obs["rankings"].clone());
                    }
                    obs
                }
                "view" => {
                    let st = Store::open(&v, e)?;
                    let view = mdkindex::search::View::new(&v, &st, s(op, "at")?)?;
                    let mut n: BTreeMap<&str, usize> = BTreeMap::new();
                    for (_, l) in &view.entries {
                        *n.entry(layer_name(*l)).or_insert(0) += 1;
                    }
                    json!(n)
                }
                "traces" => json!({"lines": v.traces(0)?.len()}),
                o => bail!("unknown op {o:?}"),
            })
        })()
        .with_context(|| format!("{id} op {i} ({name})"))?;
        out.push(json!({"op": name, "result": obs}));
    }
    drop(v);
    Ok(json!({"id": id, "output": out}))
}

/// `C3-*` directories holding a `case.c3.json`, sorted.
pub fn c3_case_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("case.c3.json").is_file())
        .collect();
    out.sort();
    Ok(out)
}
