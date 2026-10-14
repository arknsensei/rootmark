//! Conformance Level 2 evaluator (spec/CONFORMANCE.md C2-01..C2-09).
//!
//! A C2 case (`spec/fixtures/C2-xx/case.c2.json` — not `case.json`, which the
//! frozen v3-final C1 generator walks) is a script of governance operations
//! on one fresh vault; the output is one observation per operation. Expected
//! outputs are written from the current core by `scripts/gen_fixtures_c2.sh`
//! (`conformance_c2_gen`), compared by `core/tests/conformance.rs`.
//!
//! Operations (`op`):
//! - `ingest {doc, file, author, source, declared?}`
//! - `label {name, doc, block}` — remember the lineage of a head block
//! - `promote|demote|prune {lineage, actor, reason, policy, to?}`
//! - `checkpoint {name?}`; `checkpoint_file_overwrite {at}`
//! - `read {at, consumer, lineages}`; `traces`
//! - `layers` — every head block's layer, in memory and after reopening
//! - `snapshot_at {doc, at}`; `block_at {lineage, at}`; `head {doc}`
//! - `objects` — object/snapshot file counts
//! - `events` — kinds of the whole log
//! - `tamper {kind, remove_meta}` — reopen with one meta key removed from the
//!   last event of `kind`, then restore
//! - `trace_tamper {field, value, expect_open_err}` (C2-09, K-C0.2) — reopen
//!   with `field` (`query_hash` or `consumer`) of the last `traces.jsonl` line
//!   set to `value` and its `trace_id` recomputed with the §8 formula (so
//!   only the input rule can reject it), then restore
//!
//! `expect_err` (substring) marks an operation that must fail; an error is
//! recorded as `{"err": true}` only (messages may hold temp paths).

use anyhow::{anyhow, bail, Context, Result};
use mdkcore::store::write_immutable;
use mdkcore::{
    Actor, AuthorKind, Deny, HumanOnly, Layer, MatchParams, MatcherVersion, PromotionPolicy,
    Provenance, TransitionKind, Vault,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Result<Scratch> {
        let n = SCRATCH.fetch_add(1, Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("mdk-conf2-{}-{n}-{tag}", std::process::id()));
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

fn matcher_of(case: &Value) -> Result<(MatcherVersion, MatchParams)> {
    let m = case.get("matcher").context("case has no matcher block")?;
    let f = |k: &str| {
        m.get(k)
            .and_then(Value::as_f64)
            .ok_or_else(|| anyhow!("matcher.{k} missing"))
    };
    Ok((
        MatcherVersion::parse(s(m, "version")?)?,
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
        _ => bail!("unknown author {kind:?}"),
    })
}

fn policy(name: &str) -> Result<&'static dyn PromotionPolicy> {
    Ok(match name {
        "deny" => &Deny,
        "human_only" => &HumanOnly,
        _ => bail!("unknown policy {name:?}"),
    })
}

fn count(dir: &Path) -> Result<u64> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut n = 0;
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        n += if p.is_dir() { count(&p)? } else { 1 };
    }
    Ok(n)
}

fn layers(v: &Vault) -> Result<Value> {
    let mut out = BTreeMap::new();
    for d in v.doc_paths() {
        let snap = v.snapshot(&d)?.context("head")?;
        let mut rows = Vec::new();
        for b in v.snapshot_blocks(&snap)? {
            rows.push(json!({
                "lineage_id": b.lineage_id,
                "layer": b.layer,
                "inactive": v.is_inactive(&b.lineage_id)?,
            }));
        }
        out.insert(d, rows);
    }
    Ok(serde_json::to_value(out)?)
}

struct Run<'a> {
    dir: &'a Path,
    root: PathBuf,
    v: Option<Vault>,
    m: MatcherVersion,
    params: MatchParams,
    labels: BTreeMap<String, String>,
}

impl Run<'_> {
    fn v(&mut self) -> Result<&mut Vault> {
        if self.v.is_none() {
            self.v = Some(Vault::open_with(&self.root, self.params)?);
        }
        Ok(self.v.as_mut().expect("vault"))
    }

    fn reopen(&mut self) -> Result<Vault> {
        self.v = None;
        Vault::open_with(&self.root, self.params)
    }

    fn lineage(&self, op: &Value) -> Result<String> {
        let name = s(op, "lineage")?;
        self.labels
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("unknown label {name:?}"))
    }

    fn apply(&mut self, op: &Value) -> Result<Value> {
        let m = self.m;
        Ok(match s(op, "op")? {
            "ingest" => {
                let text = fs::read_to_string(self.dir.join(s(op, "file")?))?;
                let prov = Provenance {
                    author_kind: author(s(op, "author")?)?,
                    source: s(op, "source")?.to_string(),
                };
                let declared = op
                    .get("declared")
                    .and_then(Value::as_str)
                    .map(Layer::parse)
                    .transpose()?;
                let doc = s(op, "doc")?.to_string();
                let r = self.v()?.ingest_declared(&doc, &text, prov, m, declared)?;
                json!({
                    "snapshot_id": r.snapshot.snapshot_id,
                    "events": r.events.iter().map(|e| serde_json::to_value(e.kind)).collect::<serde_json::Result<Vec<_>>>()?,
                })
            }
            "label" => {
                let doc = s(op, "doc")?;
                let i = op
                    .get("block")
                    .and_then(Value::as_u64)
                    .context("block index")? as usize;
                let snap = self.v()?.snapshot(doc)?.context("no head")?;
                let l = snap
                    .blocks
                    .get(i)
                    .context("block index out of range")?
                    .0
                    .clone();
                self.labels.insert(s(op, "name")?.to_string(), l.clone());
                json!({"lineage_id": l})
            }
            kind @ ("promote" | "demote" | "prune") => {
                let l = self.lineage(op)?;
                let actor = Actor::parse(s(op, "actor")?)?;
                let reason = s(op, "reason")?.to_string();
                let pol = policy(s(op, "policy")?)?;
                let v = self.v()?;
                let ev = match kind {
                    "promote" => {
                        let req = v.request(
                            TransitionKind::Promote,
                            &l,
                            Layer::Canonical,
                            actor,
                            &reason,
                        )?;
                        v.promote(&req, pol)?
                    }
                    "demote" => {
                        let req = v.request(
                            TransitionKind::Demote,
                            &l,
                            Layer::parse(s(op, "to")?)?,
                            actor,
                            &reason,
                        )?;
                        v.demote(&req, pol)?
                    }
                    _ => v.prune_inactive(&l, actor, &reason, pol)?,
                };
                serde_json::to_value(ev)?
            }
            "checkpoint" => {
                let name = op.get("name").and_then(Value::as_str);
                serde_json::to_value(self.v()?.checkpoint(name)?)?
            }
            "checkpoint_file_overwrite" => {
                let cp = self.v()?.find_checkpoint(s(op, "at")?)?;
                let p = self
                    .root
                    .join(".mdkernel/checkpoints")
                    .join(format!("{}.json", cp.checkpoint_id));
                let same = fs::read(&p)?;
                write_immutable(&p, &same)?;
                write_immutable(&p, b"{\"tampered\":true}")?;
                json!(null)
            }
            "read" => {
                let ls = op
                    .get("lineages")
                    .and_then(Value::as_array)
                    .context("lineages")?
                    .iter()
                    .map(|x| {
                        let n = x.as_str().context("label")?;
                        self.labels
                            .get(n)
                            .cloned()
                            .ok_or_else(|| anyhow!("unknown label {n:?}"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let (at, consumer) = (s(op, "at")?.to_string(), s(op, "consumer")?.to_string());
                let (blocks, trace) = self.v()?.read_blocks(&ls, &consumer, &at)?;
                json!({"blocks": blocks, "trace": trace})
            }
            "traces" => {
                let p = self.root.join(".mdkernel/traces.jsonl");
                json!(fs::read_to_string(&p).with_context(|| "traces.jsonl")?)
            }
            "layers" => {
                let mem = layers(self.v()?)?;
                let replayed = layers(&self.reopen()?)?;
                json!({"layers": mem, "replay_equal": mem == replayed})
            }
            "snapshot_at" => {
                let (doc, at) = (s(op, "doc")?.to_string(), s(op, "at")?.to_string());
                serde_json::to_value(self.v()?.snapshot_at(&doc, &at)?)?
            }
            "block_at" => {
                let l = self.lineage(op)?;
                let at = s(op, "at")?.to_string();
                let b = self.v()?.block_at(&l, &at)?;
                json!({"lineage_id": b.lineage_id, "version_id": b.version_id, "layer": b.layer, "text": b.normalized_text})
            }
            "head" => {
                let doc = s(op, "doc")?.to_string();
                serde_json::to_value(self.v()?.snapshot(&doc)?)?
            }
            "objects" => {
                let md = self.root.join(".mdkernel");
                json!({"objects": count(&md.join("objects"))?, "snapshots": count(&md.join("snapshots"))?})
            }
            "events" => {
                let log = fs::read_to_string(self.root.join(".mdkernel/events.jsonl"))?;
                let kinds = log
                    .lines()
                    .map(|l| {
                        let e: Value = serde_json::from_str(l)?;
                        Ok(e["kind"].clone())
                    })
                    .collect::<Result<Vec<_>>>()?;
                json!(kinds)
            }
            "tamper" => {
                let kind = s(op, "kind")?;
                let key = s(op, "remove_meta")?;
                self.v = None;
                let p = self.root.join(".mdkernel/events.jsonl");
                let log = fs::read_to_string(&p)?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let i = lines
                    .iter()
                    .rposition(|l| l.contains(&format!("\"kind\":\"{kind}\"")))
                    .context("no event of that kind")?;
                let mut ev: Value = serde_json::from_str(&lines[i])?;
                ev["meta"]
                    .as_object_mut()
                    .context("meta")?
                    .remove(key)
                    .context("meta key absent")?;
                lines[i] = serde_json::to_string(&ev)?;
                fs::write(&p, lines.join("\n") + "\n")?;
                let res = Vault::open_with(&self.root, self.params);
                fs::write(&p, &log)?;
                match res {
                    Ok(_) => bail!("tampered log opened"),
                    Err(e) => {
                        let msg = format!("{e:#}");
                        let want = s(op, "expect_open_err")?;
                        if !msg.contains(want) {
                            bail!("tamper: {msg:?} lacks {want:?}");
                        }
                        json!({"open_err": want})
                    }
                }
            }
            "trace_tamper" => {
                let field = s(op, "field")?;
                if field != "query_hash" && field != "consumer" {
                    bail!("trace_tamper: field {field:?} (query_hash|consumer)");
                }
                let value = s(op, "value")?;
                self.v = None;
                let p = self.root.join(".mdkernel/traces.jsonl");
                let log = fs::read_to_string(&p).context("traces.jsonl")?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let last = lines.last_mut().context("traces.jsonl is empty")?;
                let mut t: Value = serde_json::from_str(last)?;
                t[field] = json!(value);
                let at_seq = t["at_seq"].as_u64().context("at_seq")?;
                t["trace_id"] = json!(mdkcore::trace_id(
                    s(&t, "checkpoint_id")?,
                    s(&t, "consumer")?,
                    s(&t, "query_hash")?,
                    at_seq
                ));
                *last = serde_json::to_string(&t)?;
                fs::write(&p, lines.join("\n") + "\n")?;
                let res = Vault::open_with(&self.root, self.params);
                fs::write(&p, &log)?;
                match res {
                    Ok(_) => bail!("tampered traces.jsonl opened"),
                    Err(e) => {
                        let msg = format!("{e:#}");
                        let want = s(op, "expect_open_err")?;
                        if !msg.contains(want) {
                            bail!("trace_tamper: {msg:?} lacks {want:?}");
                        }
                        json!({"open_err": want})
                    }
                }
            }
            o => bail!("unknown op {o:?}"),
        })
    }
}

/// Evaluates one C2 case directory.
pub fn evaluate_c2(dir: &Path) -> Result<Value> {
    let case: Value = serde_json::from_str(&fs::read_to_string(dir.join("case.c2.json"))?)
        .with_context(|| format!("{}/case.c2.json", dir.display()))?;
    let (m, params) = matcher_of(&case)?;
    let id = s(&case, "id")?.to_string();
    let scratch = Scratch::new(&id)?;
    let mut run = Run {
        dir,
        root: scratch.0.clone(),
        v: None,
        m,
        params,
        labels: BTreeMap::new(),
    };
    let mut out = Vec::new();
    for (i, op) in case
        .get("ops")
        .and_then(Value::as_array)
        .context("ops")?
        .iter()
        .enumerate()
    {
        let res = run.apply(op);
        let obs = match (op.get("expect_err").and_then(Value::as_str), res) {
            (None, Ok(v)) => v,
            (None, Err(e)) => bail!("op {i} ({}): {e:#}", s(op, "op")?),
            (Some(want), Err(e)) => {
                let msg = format!("{e:#}");
                if !msg.contains(want) {
                    bail!("op {i}: error {msg:?} lacks {want:?}");
                }
                // a failed operation must leave the vault openable
                run.v = None;
                json!({"err": true})
            }
            (Some(want), Ok(_)) => bail!("op {i}: expected an error containing {want:?}"),
        };
        out.push(json!({"op": s(op, "op")?, "result": obs}));
    }
    Ok(json!({"id": id, "output": out}))
}

/// `C2-*` directories holding a `case.c2.json`, sorted.
pub fn c2_case_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("case.c2.json").is_file())
        .collect();
    out.sort();
    Ok(out)
}
