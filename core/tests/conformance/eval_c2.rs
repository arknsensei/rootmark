//! Conformance Level 2 evaluator (spec/CONFORMANCE.md C2-01..C2-13).
//!
//! A C2 case (`spec/fixtures/C2-xx/case.c2.json` — not `case.json`, which the
//! frozen v3-final C1 generator walks) is a script of governance operations
//! on one fresh vault; the output is one observation per operation. Expected
//! outputs are written from the current core by `scripts/gen_fixtures_c2.sh`
//! (`conformance_c2_gen`), compared by `core/tests/conformance.rs`.
//!
//! Operations (`op`):
//! - `ingest {doc, file, author, source, declared?, actor?}` — with `actor`
//!   (`<kind>:<id>`, C2-10, 1.0.3) the attributed ingest (`Vault::ingest_by`),
//!   observed with `attributed`: `[kind, meta.actor, meta.source]` per event
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
//! - `actor_tamper {kind, change, expect_open_err?}` (C2-10, 1.0.3) — reopen
//!   with the last event of `kind` changed (`remove_both`, `remove_actor`,
//!   `actor_kind`, `actor_id_empty`, `actor_extra_field`, `source_number`,
//!   `add_both`), then restore; observed as `{opened}` or `{open_err}`
//!
//! - (C2-13; spec 2.0.1 §7.1 "Partial match", §8; P2-0.1/F1) `attest
//!   {content: true | content_file}` sends the saved bytes (a
//!   `content_file` whose bytes are not the file's is refused); `external`
//!   (`on_disk: true`: the file holds the attested bytes when it is
//!   recorded — no late partial match) decides `partial` when the bytes
//!   contain an attestation's blocks (the
//!   attested bytes ingested first, observed as `attested`, the line
//!   `partial: true`, then the rest unattested); a late attestation with
//!   content raises the attested lineages of the one pending change that
//!   contains its blocks; `attestation_tamper {change: "partial_false"}`.
//! - (C2-11, C2-12; spec 2.0.0 §7.1, §8; P2-0/F7) `restore_store {from}` —
//!   the case directory `from` becomes the vault's `.rootmark/` (a store
//!   written by another implementation, e.g. 0.1.0: spec 1.0.3), then
//!   `fingerprint` (`events_hash`); `unattested_ingest {doc, file}` — the
//!   ingest of an unattested external change (`meta.layer = "unverified"`,
//!   actor `unattested_fs:fs`, source `unattested_fs`), observed as
//!   `ingest` with `attributed` and `declared` per event; `external {doc,
//!   file, at_ms}` — an external change observed at `at_ms` (explicit
//!   milliseconds) through the case's window (`window_ms`): attested →
//!   human ingest (`human:conformance`), else an unattested ingest
//!   remembered for a late attestation; `attest {doc, file, at_ms, late?}` —
//!   record an attestation of the file's bytes, match it in the window,
//!   raise late (`raise_late`); `attestations` — `attestations.jsonl` as
//!   written; `meta_tamper {kind, key, value, expect_open_err}` — reopen with
//!   `meta.<key>` of the last event of `kind` set to `value`, then restore;
//!   `attestation_tamper {change, expect_open_err}` — reopen with the
//!   attestation log changed (`id`, `content_hash`, `doc_path`, `at_seq`,
//!   `duplicate`, `remove`), then restore.
//!
//! `expect_err` (substring) marks an operation that must fail; an error is
//! recorded as `{"err": true}` only (messages may hold temp paths).

use anyhow::{anyhow, bail, Context, Result};
use rootmarkcore::attest::{content_hash, Raise, Unattested, Window};
use rootmarkcore::store::write_immutable;
use rootmarkcore::{
    Actor, ActorKind, AuthorKind, Deny, HumanOnly, Layer, MatchParams, MatcherVersion,
    PromotionPolicy, Provenance, TransitionKind, Vault,
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
        let d =
            std::env::temp_dir().join(format!("rootmark-conf2-{}-{n}-{tag}", std::process::id()));
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
    /// the case's attestation window (C2-12), explicit times
    window: Window,
    /// unattested changes in the window as the reference server remembers
    /// them beside the matcher (C2-13, P2-0.1): (doc, content_hash,
    /// observed ms, the observed blocks' versions, the lineages the change
    /// named with their raise)
    pending: Vec<Pending>,
}

/// One remembered unattested change (C2-13): doc, content_hash, observed
/// ms, observed versions, named lineages.
type Pending = (String, String, u64, Vec<String>, Vec<Raise>);

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)? {
        let p = e?.path();
        let t = to.join(p.file_name().context("name")?);
        if p.is_dir() {
            copy_dir(&p, &t)?;
        } else {
            fs::copy(&p, &t)?;
        }
    }
    Ok(())
}

/// `[kind, meta.actor, meta.source, meta.layer]` per event of an ingest.
fn attributed(events: &[rootmarkcore::Event]) -> Result<Value> {
    Ok(events
        .iter()
        .map(|e| {
            Ok(json!([
                serde_json::to_value(e.kind)?,
                e.meta.get("actor").cloned().unwrap_or(Value::Null),
                e.meta.get("source").cloned().unwrap_or(Value::Null),
                e.meta.get("layer").cloned().unwrap_or(Value::Null)
            ]))
        })
        .collect::<serde_json::Result<Vec<_>>>()?
        .into())
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
                let actor = op.get("actor").and_then(Value::as_str);
                let r = match actor {
                    None => self.v()?.ingest_declared(&doc, &text, prov, m, declared)?,
                    Some(a) => {
                        if declared.is_some() {
                            bail!("`declared` and `actor` together are not a case");
                        }
                        let a = Actor::parse(a)?;
                        self.v()?.ingest_by(&doc, &text, prov, m, &a)?
                    }
                };
                let mut o = json!({
                    "snapshot_id": r.snapshot.snapshot_id,
                    "events": r.events.iter().map(|e| serde_json::to_value(e.kind)).collect::<serde_json::Result<Vec<_>>>()?,
                });
                if actor.is_some() {
                    o["attributed"] = r
                        .events
                        .iter()
                        .map(|e| {
                            Ok(json!([
                                serde_json::to_value(e.kind)?,
                                e.meta.get("actor").cloned().unwrap_or(Value::Null),
                                e.meta.get("source").cloned().unwrap_or(Value::Null)
                            ]))
                        })
                        .collect::<serde_json::Result<Vec<_>>>()?
                        .into();
                }
                o
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
                    .join(".rootmark/checkpoints")
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
                let p = self.root.join(".rootmark/traces.jsonl");
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
                let md = self.root.join(".rootmark");
                json!({"objects": count(&md.join("objects"))?, "snapshots": count(&md.join("snapshots"))?})
            }
            "events" => {
                let log = fs::read_to_string(self.root.join(".rootmark/events.jsonl"))?;
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
                let p = self.root.join(".rootmark/events.jsonl");
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
            "actor_tamper" => {
                let kind = s(op, "kind")?;
                let change = s(op, "change")?;
                self.v = None;
                let p = self.root.join(".rootmark/events.jsonl");
                let log = fs::read_to_string(&p)?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let i = lines
                    .iter()
                    .rposition(|l| l.contains(&format!("\"kind\":\"{kind}\"")))
                    .context("no event of that kind")?;
                let mut ev: Value = serde_json::from_str(&lines[i])?;
                let meta = ev["meta"].as_object_mut().context("meta")?;
                match change {
                    "remove_both" => {
                        meta.remove("actor").context("no meta.actor")?;
                        meta.remove("source").context("no meta.source")?;
                    }
                    "remove_actor" => {
                        meta.remove("actor").context("no meta.actor")?;
                    }
                    "actor_kind" => meta["actor"]["kind"] = json!("robot"),
                    "actor_id_empty" => meta["actor"]["id"] = json!(""),
                    "actor_extra_field" => meta["actor"]["extra"] = json!(1),
                    "source_number" => meta["source"] = json!(7),
                    "add_both" => {
                        meta.insert("actor".into(), json!({"kind": "agent", "id": "x"}));
                        meta.insert("source".into(), json!("x"));
                    }
                    o => bail!("unknown change {o:?}"),
                }
                lines[i] = serde_json::to_string(&ev)?;
                fs::write(&p, lines.join("\n") + "\n")?;
                let res = Vault::open_with(&self.root, self.params);
                fs::write(&p, &log)?;
                match (res, op.get("expect_open_err").and_then(Value::as_str)) {
                    (Ok(_), None) => json!({"opened": true}),
                    (Ok(_), Some(_)) => bail!("tampered log opened"),
                    (Err(e), Some(want)) => {
                        let msg = format!("{e:#}");
                        if !msg.contains(want) {
                            bail!("actor_tamper: {msg:?} lacks {want:?}");
                        }
                        json!({"open_err": want})
                    }
                    (Err(e), None) => bail!("actor_tamper {change}: open failed: {e:#}"),
                }
            }
            "trace_tamper" => {
                let field = s(op, "field")?;
                if field != "query_hash" && field != "consumer" {
                    bail!("trace_tamper: field {field:?} (query_hash|consumer)");
                }
                let value = s(op, "value")?;
                self.v = None;
                let p = self.root.join(".rootmark/traces.jsonl");
                let log = fs::read_to_string(&p).context("traces.jsonl")?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let last = lines.last_mut().context("traces.jsonl is empty")?;
                let mut t: Value = serde_json::from_str(last)?;
                t[field] = json!(value);
                let at_seq = t["at_seq"].as_u64().context("at_seq")?;
                t["trace_id"] = json!(rootmarkcore::trace_id(
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
            "restore_store" => {
                self.v = None;
                let md = self.root.join(".rootmark");
                if md.exists() {
                    fs::remove_dir_all(&md)?;
                }
                copy_dir(&self.dir.join(s(op, "from")?), &md)?;
                json!({"restored": true})
            }
            "fingerprint" => json!({"events_hash": self.v()?.events_hash()?}),
            "unattested_ingest" => {
                let doc = s(op, "doc")?.to_string();
                let text = fs::read_to_string(self.dir.join(s(op, "file")?))?;
                let r = self.unattested(&doc, &text)?.0;
                json!({
                    "snapshot_id": r.snapshot.snapshot_id,
                    "events": r.events.iter().map(|e| serde_json::to_value(e.kind)).collect::<serde_json::Result<Vec<_>>>()?,
                    "attributed": attributed(&r.events)?,
                })
            }
            "external" => {
                let doc = s(op, "doc")?.to_string();
                let bytes = fs::read(self.dir.join(s(op, "file")?))?;
                let at = op.get("at_ms").and_then(Value::as_u64).context("at_ms")?;
                let h = content_hash(&bytes);
                let text = String::from_utf8(bytes)?;
                match self.window.observe(&doc, &h, at) {
                    Some(_) => {
                        let a = Actor::new(ActorKind::Human, "conformance")?;
                        let prov = Provenance {
                            author_kind: AuthorKind::Human,
                            source: "watch".into(),
                        };
                        let r = self.v()?.ingest_by(&doc, &text, prov, m, &a)?;
                        json!({"decision": "attested", "attributed": attributed(&r.events)?})
                    }
                    None => {
                        // (C2-13, P2-0.1 T2) a partial match: the attested
                        // bytes as an attested change, the line `partial`,
                        // then the rest unattested
                        let partial = self.window.observe_partial(&doc, &text, at)?;
                        let mut first = None;
                        if let Some((_, saved)) = &partial {
                            let a = Actor::new(ActorKind::Human, "conformance")?;
                            let prov = Provenance {
                                author_kind: AuthorKind::Human,
                                source: "watch".into(),
                            };
                            let r = self.v()?.ingest_by(&doc, saved, prov, m, &a)?;
                            first = Some(attributed(&r.events)?);
                            let v = self.v()?;
                            let mut line = rootmarkcore::attest::Attestation::new(
                                &doc,
                                &content_hash(saved.as_bytes()),
                                v.next_seq(),
                                None,
                            )?;
                            line.partial = Some(true);
                            v.record_attestation_line(line)?;
                        }
                        let (r, raises) = self.unattested(&doc, &text)?;
                        let named: Vec<Raise> = raises
                            .iter()
                            .filter(|x| {
                                r.events
                                    .iter()
                                    .any(|e| e.lineage_ids.contains(&x.lineage_id))
                            })
                            .cloned()
                            .collect();
                        self.pending
                            .retain(|p| at.saturating_sub(p.2) <= self.window.window_ms);
                        self.pending.push((
                            doc.clone(),
                            h.clone(),
                            at,
                            rootmarkcore::attest::block_versions(&text, &doc)?,
                            named,
                        ));
                        self.window.unattested(Unattested {
                            doc_path: doc,
                            content_hash: h,
                            observed_ms: at,
                            lineages: raises,
                        });
                        match first {
                            Some(f) => {
                                json!({"decision": "partial", "attested": f, "attributed": attributed(&r.events)?})
                            }
                            None => {
                                json!({"decision": "unverified", "attributed": attributed(&r.events)?})
                            }
                        }
                    }
                }
            }
            "attest" => {
                let doc = s(op, "doc")?.to_string();
                let bytes = fs::read(self.dir.join(s(op, "file")?))?;
                let at = op.get("at_ms").and_then(Value::as_u64).context("at_ms")?;
                let late = op.get("late").and_then(Value::as_bool).unwrap_or(true);
                let h = content_hash(&bytes);
                // (C2-13, P2-0.1 T2) the saved bytes sent with it: `content:
                // true` the file's, `content_file` another file's (which
                // must then be refused)
                let content = match (
                    op.get("content").and_then(Value::as_bool),
                    op.get("content_file").and_then(Value::as_str),
                ) {
                    (Some(true), _) => Some(String::from_utf8(bytes.clone())?),
                    (_, Some(f)) => Some(fs::read_to_string(self.dir.join(f))?),
                    _ => None,
                };
                if let Some(c) = &content {
                    if content_hash(c.as_bytes()) != h {
                        bail!("attest: content does not hash to content_hash");
                    }
                }
                let v = self.v()?;
                let mut a = rootmarkcore::attest::Attestation::new(&doc, &h, v.next_seq(), None)?;
                let (late_list, ambiguous) =
                    self.window.attest_content(&a, at, late, content.clone());
                // the file holds the attested bytes when it is recorded
                // (`on_disk`): their own observation is coming, no late
                // partial match (spec 2.0.1 §7.1)
                let on_disk = op.get("on_disk").and_then(Value::as_bool).unwrap_or(false);
                let mut partial = None;
                if late_list.is_empty() && !ambiguous && late && !on_disk {
                    if let Some(c) = &content {
                        let av = rootmarkcore::attest::block_versions(c, &doc)?;
                        let w = self.window.window_ms;
                        let hits: Vec<usize> = (0..self.pending.len())
                            .filter(|&i| {
                                let p = &self.pending[i];
                                p.0 == doc
                                    && p.1 != h
                                    && at.saturating_sub(p.2) <= w
                                    && rootmarkcore::attest::covers(&p.3, &av)
                            })
                            .collect();
                        if hits.len() == 1 {
                            let p = self.pending[hits[0]].clone();
                            // the lineages the change named (as the server
                            // remembers them), while it is still in the window
                            let rs = self
                                .window
                                .pending_lineages(&p.0, &p.1, p.2)
                                .and_then(|_| {
                                    rootmarkcore::attest::late_partial_raises(&p.3, &av, &p.4)
                                })
                                .filter(|r| !r.is_empty());
                            if let Some(rs) = rs {
                                self.window.take_unattested(&p.0, &p.1, p.2);
                                self.pending.remove(hits[0]);
                                a.partial = Some(true);
                                self.window.forget(&a.attestation_id);
                                partial = Some(rs);
                            }
                        }
                    }
                }
                self.pending.retain(|p| !(p.0 == doc && p.1 == h));
                let (a, recorded) = self.v()?.record_attestation_line(a)?;
                let mut raised = Vec::new();
                for u in late_list {
                    for e in self
                        .v()?
                        .raise_late(&u.doc_path, &u.lineages, &a.attestation_id)?
                    {
                        raised.push(json!([e.lineage_ids[0], e.meta["to"]]));
                    }
                }
                if let Some(rs) = partial {
                    for e in self.v()?.raise_late(&doc, &rs, &a.attestation_id)? {
                        raised.push(json!([e.lineage_ids[0], e.meta["to"]]));
                    }
                }
                json!({"attestation_id": a.attestation_id, "at_seq": a.at_seq, "recorded": recorded, "raised": raised})
            }
            "attestations" => {
                let p = self.root.join(".rootmark/attestations.jsonl");
                json!(fs::read_to_string(&p).unwrap_or_default())
            }
            "meta_tamper" => {
                let kind = s(op, "kind")?;
                let key = s(op, "key")?;
                let value = op.get("value").cloned().context("value")?;
                self.v = None;
                let p = self.root.join(".rootmark/events.jsonl");
                let log = fs::read_to_string(&p)?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let i = lines
                    .iter()
                    .rposition(|l| l.contains(&format!("\"kind\":\"{kind}\"")))
                    .context("no event of that kind")?;
                let mut ev: Value = serde_json::from_str(&lines[i])?;
                ev["meta"][key] = value;
                lines[i] = serde_json::to_string(&ev)?;
                fs::write(&p, lines.join("\n") + "\n")?;
                let res = Vault::open_with(&self.root, self.params);
                fs::write(&p, &log)?;
                self.open_err(res.err(), op)?
            }
            "attestation_tamper" => {
                let change = s(op, "change")?;
                self.v = None;
                let p = self.root.join(".rootmark/attestations.jsonl");
                let log = fs::read_to_string(&p).context("attestations.jsonl")?;
                let mut lines: Vec<String> = log.lines().map(str::to_string).collect();
                let last = lines.last().context("attestations.jsonl is empty")?.clone();
                let mut a: Value = serde_json::from_str(&last)?;
                match change {
                    "id" => a["attestation_id"] = json!("0".repeat(32)),
                    "content_hash" => a["content_hash"] = json!("ABC"),
                    "doc_path" => a["doc_path"] = json!("../outside.md"),
                    "at_seq" => a["at_seq"] = json!(1_000_000),
                    // C2-13: `partial` is written only as true
                    "partial_false" => a["partial"] = json!(false),
                    "duplicate" => {}
                    "remove" => {}
                    o => bail!("unknown change {o:?}"),
                }
                match change {
                    "duplicate" => lines.push(last),
                    "remove" => lines.clear(),
                    _ => *lines.last_mut().expect("a line") = serde_json::to_string(&a)?,
                }
                fs::write(
                    &p,
                    lines.iter().map(|l| format!("{l}\n")).collect::<String>(),
                )?;
                let res = Vault::open_with(&self.root, self.params);
                fs::write(&p, &log)?;
                self.open_err(res.err(), op)?
            }
            o => bail!("unknown op {o:?}"),
        })
    }

    /// An unattested external change (spec 2.0.0 §7.1): `unverified`, actor
    /// `unattested_fs:fs`, source `unattested_fs`; with the raises a late
    /// attestation may make (the layer an attested ingest would give each
    /// lineage, at the version this ingest gives it).
    fn unattested(
        &mut self,
        doc: &str,
        text: &str,
    ) -> Result<(rootmarkcore::IngestResult, Vec<Raise>)> {
        let m = self.m;
        let v = self.v()?;
        let probe = v.plan_ingest(
            doc,
            text,
            Provenance {
                author_kind: AuthorKind::Human,
                source: "watch".into(),
            },
            m,
            None,
            Some(&Actor::new(ActorKind::Human, "conformance")?),
        )?;
        let plan = v.plan_ingest(
            doc,
            text,
            Provenance {
                author_kind: AuthorKind::Human,
                source: "unattested_fs".into(),
            },
            m,
            Some(Layer::Unverified),
            Some(&Actor::new(ActorKind::UnattestedFs, "fs")?),
        )?;
        let raises: Vec<Raise> = plan
            .blocks_after
            .iter()
            .filter(|b| b.layer == Layer::Unverified)
            .filter_map(|b| {
                let to = probe
                    .blocks_after
                    .iter()
                    .find(|p| p.lineage_id == b.lineage_id)?
                    .layer;
                matches!(to, Layer::Proposal | Layer::Canonical).then(|| Raise {
                    lineage_id: b.lineage_id.clone(),
                    version_id: b.version_id.clone(),
                    to,
                })
            })
            .collect();
        let r = v.commit_plan(plan)?;
        Ok((r, raises))
    }

    /// The observation of a tamper: the vault must not have opened, with an
    /// error holding `expect_open_err`.
    fn open_err(&self, err: Option<anyhow::Error>, op: &Value) -> Result<Value> {
        let want = s(op, "expect_open_err")?;
        match err {
            None => bail!("tampered store opened"),
            Some(e) => {
                let msg = format!("{e:#}");
                if !msg.contains(want) {
                    bail!("{msg:?} lacks {want:?}");
                }
                Ok(json!({"open_err": want}))
            }
        }
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
        window: Window::new(
            case.get("window_ms")
                .and_then(Value::as_u64)
                .unwrap_or(rootmarkcore::attest::DEFAULT_WINDOW_MS),
        ),
        pending: Vec::new(),
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
