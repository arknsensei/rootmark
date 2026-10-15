//! The tools of the MCP server and the HTTP API (GOAL P1-D "MCP araçları"),
//! independent of the transport: a tool takes a JSON object of arguments and
//! the consumer's identity and returns JSON, or an error. Every tool opens
//! the vault afresh (`Vault::open` replays the log), so a change made by
//! another process — `mdk promote` from the command line — is seen by the
//! next call; calls are serialised by one lock.
//!
//! The tools expose existing primitives only (spec §7–§9): no tool decides a
//! promotion, no tool other than `propose` writes Markdown, and the output of
//! a tool that has a CLI twin is the JSON that command prints.

use anyhow::{bail, Context, Result};
use mdkcore::{AuthorKind, MatcherVersion, Policy, Provenance, Vault};
use mdkindex::search::{Mode, Searcher};
use mdkindex::store::{self, Store};
use mdkindex::OnnxMiniLm;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Spec version this build implements (spec/SPEC.md title).
pub const SPEC_VERSION: &str = "1.0.2";
/// Conformance level the reference implementation claims (spec §12;
/// `make conformance`).
pub const LEVEL_CLAIMED: u8 = 3;
/// `mdk` version (one SemVer for every crate, PACKAGES).
pub const MDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A tool: name, description in spec terms, JSON Schema of its arguments.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

/// Every tool this server exposes, in a fixed order.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "status",
            description: "State of the served vault: its path, the spec and mdk versions, the \
                claimed conformance level (spec §12), the embedder of its index (spec §9), its \
                last checkpoint (spec §8) and the active policy recipe (promotion policy, spec §7). \
                Writes nothing.",
            input_schema: object_schema(json!({}), &[]),
        },
        ToolSpec {
            name: "search",
            description: "Retrieval as of a checkpoint (spec §9): ranked chunks for a query, lexical \
                (BM25), flat (exact cosine) or hybrid (reciprocal rank fusion, the default), at the \
                checkpoint `at` or at the checkpoint of the current state. Only `canonical` content \
                is returned unless `include_proposal` is set; then up to `k` `proposal` chunks, \
                ranked apart with the canonical statistics, labelled `proposal` with their score \
                multiplied by 0.5, follow the unchanged canonical hits (spec §7, §9). `derived` is \
                never returned. Records one consumption trace for the calling consumer (spec §8). \
                An empty or whitespace-only query is an error and records nothing.",
            input_schema: object_schema(
                json!({
                    "query": {"type": "string", "minLength": 1,
                        "description": "the query text (not empty, not only whitespace)"},
                    "k": {"type": "integer", "minimum": 1, "maximum": 1000,
                        "description": "canonical hits to return (default 10)"},
                    "mode": {"type": "string", "enum": ["hybrid", "lexical", "flat"],
                        "description": "retrieval mode (default hybrid)"},
                    "at": {"type": "string", "minLength": 1,
                        "description": "checkpoint id or name (default: the current state's)"},
                    "include_proposal": {"type": "boolean",
                        "description": "also return labelled proposal chunks (default false)"},
                }),
                &["query"],
            ),
        },
        ToolSpec {
            name: "read_block",
            description: "One block as of a checkpoint (spec §8): its version (version id, type, \
                text, heading path, parents), layer (spec §7) and provenance (author kind, \
                source), for lineage `lineage_id` of document `doc_path`; at checkpoint `at` or at \
                the checkpoint of the current state. Records one consumption trace for the calling \
                consumer. A `derived` block is never returned; a lineage that is not live in that \
                document at that checkpoint is an error and records nothing.",
            input_schema: object_schema(
                json!({
                    "doc_path": {"type": "string", "minLength": 1,
                        "description": "vault-relative path of the Markdown document"},
                    "lineage_id": {"type": "string", "minLength": 1,
                        "description": "lineage id (32 lowercase hex)"},
                    "at": {"type": "string", "minLength": 1,
                        "description": "checkpoint id or name (default: the current state's)"},
                }),
                &["doc_path", "lineage_id"],
            ),
        },
        ToolSpec {
            name: "history",
            description: "Version chain of one lineage (spec §4–§5): every event of the log that \
                names it (created, updated, moved, split, merged, deleted, promoted, demoted, \
                layer changed), in log order, and its current layer (spec §7). Writes nothing.",
            input_schema: object_schema(
                json!({"lineage_id": {"type": "string", "minLength": 1,
                    "description": "lineage id (32 lowercase hex)"}}),
                &["lineage_id"],
            ),
        },
        ToolSpec {
            name: "blame",
            description: "Provenance of a document block by block, in document order (spec §4.3, \
                §7): per block its lineage id, version id, layer (canonical, proposal, derived), \
                author kind, source and the log offsets that created and last changed it; at the \
                head, or as of a checkpoint (spec §8). Writes nothing.",
            input_schema: object_schema(
                json!({
                    "doc_path": {"type": "string", "minLength": 1,
                        "description": "vault-relative path of a Markdown document"},
                    "at": {"type": "string", "minLength": 1,
                        "description": "checkpoint id or name (default: the head)"},
                }),
                &["doc_path"],
            ),
        },
        ToolSpec {
            name: "propose",
            description: "The only way an agent writes Markdown (spec §7, K8): the agent's text is \
                written into the vault as plain Markdown — a new document (`mode` create, the \
                default; an existing document is never overwritten), appended to an existing one \
                (`append`), or as the body of the section under `heading` (`replace_section`) — \
                and ingested with `author_kind` agent and the consumer as source, so new and \
                changed blocks are `proposal`, excluded from default retrieval until a human \
                promotes them. Nothing but the agent's text is written to the file (no ids, no \
                metadata). `reason` is recorded beside the vault (`.mdkernel/proposals.jsonl`). \
                The path must stay inside the vault (no `..`, no absolute path, no hidden \
                directory) and the text may not exceed the server's `max_propose_bytes`.",
            input_schema: object_schema(
                json!({
                    "doc_path": {"type": "string", "minLength": 1,
                        "description": "vault-relative path of a .md document"},
                    "content": {"type": "string",
                        "description": "the agent's Markdown text"},
                    "reason": {"type": "string", "minLength": 1,
                        "description": "why the agent proposes this (recorded, not written to the document)"},
                    "mode": {"type": "string", "enum": ["create", "append", "replace_section"],
                        "description": "create (default), append or replace_section"},
                    "heading": {"type": "string", "minLength": 1,
                        "description": "the section heading text (replace_section only)"},
                }),
                &["doc_path", "content", "reason"],
            ),
        },
        ToolSpec {
            name: "promote",
            description: "Request the promotion of a lineage from `proposal` to `canonical` (spec §7). \
                The kernel does not decide: the vault's promotion policy hook does, for the actor \
                agent:<consumer> (the MCP client or HTTP consumer; it cannot be chosen). Under the \
                phase-1 policies (`human_only`, the default, or `deny`) an agent's request is \
                denied: the answer is `decision: denied` with the policy and its reason, not an \
                error, and no event is written. A human promotes through the CLI (`mdk promote \
                --actor human:<id>`). An allowed request writes a `Promoted` event.",
            input_schema: object_schema(
                json!({
                    "lineage_id": {"type": "string", "minLength": 1,
                        "description": "lineage id (32 lowercase hex)"},
                    "reason": {"type": "string", "minLength": 1,
                        "description": "why the promotion is requested"},
                }),
                &["lineage_id", "reason"],
            ),
        },
        ToolSpec {
            name: "checkpoint",
            description: "Create a checkpoint of the vault's current state (spec §8): an immutable \
                map doc path → snapshot at an event offset, optionally named; the same state \
                gives the same checkpoint id. Queries can then be evaluated as of it (`search \
                at`, `blame at`, `read_block at`). Returns the checkpoint.",
            input_schema: object_schema(
                json!({"name": {"type": "string", "minLength": 1,
                    "description": "optional checkpoint name"}}),
                &[],
            ),
        },
        ToolSpec {
            name: "checkpoints",
            description: "Every checkpoint of the vault (spec §8: checkpoint id, optional name, \
                event offset, snapshot per document), in creation order. Writes nothing.",
            input_schema: object_schema(json!({}), &[]),
        },
        ToolSpec {
            name: "trace",
            description: "Consumption traces (spec §8): which block versions (doc path, lineage \
                id, version id) each retrieval returned, with its checkpoint, consumer and query \
                hash; from trace offset `since_seq`, optionally for one consumer. Writes nothing.",
            input_schema: object_schema(
                json!({
                    "since_seq": {"type": "integer", "minimum": 0,
                        "description": "only traces with at_seq >= since_seq (default 0)"},
                    "consumer": {"type": "string", "minLength": 1,
                        "description": "only traces of this consumer"},
                }),
                &[],
            ),
        },
    ]
}

/// The served vault and the lock that serialises tool calls.
pub struct Kernel {
    /// the vault path as given (`mdk serve --vault`); `status` reports it
    vault: PathBuf,
    /// `--matcher` for ingests the server makes (watcher, `propose`); `None`:
    /// the vault's lock, else v3 (as `mdk index watch`)
    matcher: Option<MatcherVersion>,
    lock: Mutex<()>,
    /// loaded on first use, then shared (ONNX session behind a mutex)
    embedder: Mutex<Option<Arc<OnnxMiniLm>>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoArgs {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    query: String,
    k: Option<usize>,
    mode: Option<String>,
    at: Option<String>,
    include_proposal: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadBlockArgs {
    doc_path: String,
    lineage_id: String,
    at: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposeArgs {
    doc_path: String,
    content: String,
    reason: String,
    mode: Option<String>,
    heading: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PromoteArgs {
    lineage_id: String,
    reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointArgs {
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryArgs {
    lineage_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BlameArgs {
    doc_path: String,
    at: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceArgs {
    since_seq: Option<u64>,
    consumer: Option<String>,
}

/// Arguments of `tool`: an absent argument object is `{}`; anything else
/// must be an object matching the tool's schema (unknown fields rejected).
fn args<T: DeserializeOwned>(tool: &str, a: &Value) -> Result<T> {
    let a = if a.is_null() { json!({}) } else { a.clone() };
    if !a.is_object() {
        bail!("{tool}: arguments must be a JSON object");
    }
    serde_json::from_value(a).with_context(|| format!("{tool}: invalid arguments"))
}

/// The consumer a trace-recording tool records; none → error (an MCP client
/// that has not identified itself in `initialize`, or no `X-Consumer`).
fn need_consumer<'c>(tool: &str, consumer: Option<&'c str>) -> Result<&'c str> {
    match consumer {
        Some(c) if !c.trim().is_empty() => Ok(c),
        _ => bail!("{tool}: records a consumption trace and needs a consumer identity (MCP initialize clientInfo, or HTTP X-Consumer)"),
    }
}

fn non_empty(tool: &str, field: &str, v: &str) -> Result<()> {
    if v.is_empty() {
        bail!("{tool}: {field} must not be empty");
    }
    Ok(())
}

impl Kernel {
    /// A kernel serving `vault`, which must be an existing vault directory
    /// whose log opens.
    pub fn open(vault: &Path) -> Result<Kernel> {
        Self::open_with(vault, None)
    }

    pub fn open_with(vault: &Path, matcher: Option<MatcherVersion>) -> Result<Kernel> {
        if !vault.is_dir() {
            bail!("vault {} is not a directory", vault.display());
        }
        Vault::open(vault).with_context(|| format!("open vault {}", vault.display()))?;
        Ok(Kernel {
            vault: vault.to_path_buf(),
            matcher,
            lock: Mutex::new(()),
            embedder: Mutex::new(None),
        })
    }

    fn guard(&self) -> Result<std::sync::MutexGuard<'_, ()>> {
        self.lock
            .lock()
            .map_err(|_| anyhow::anyhow!("an earlier tool call panicked; restart the server"))
    }

    /// The embedder (the only one, `minilm-l6-v2`; models/MANIFEST.json).
    pub fn embedder(&self) -> Result<Arc<OnnxMiniLm>> {
        let mut e = self
            .embedder
            .lock()
            .map_err(|_| anyhow::anyhow!("embedder lock poisoned"))?;
        if e.is_none() {
            *e = Some(Arc::new(OnnxMiniLm::load_default()?));
        }
        Ok(e.as_ref().expect("set above").clone())
    }

    fn matcher_for(&self, v: &Vault) -> MatcherVersion {
        self.matcher
            .or_else(|| v.matcher_lock())
            .unwrap_or(MatcherVersion::V3)
    }

    /// Builds the index if the vault has none (`mdk index build`); `None`
    /// if one exists. The served index must exist before searches and
    /// watched edits.
    pub fn ensure_index(&self) -> Result<Option<store::BuildReport>> {
        let _g = self.guard()?;
        let v = Vault::open(&self.vault)?;
        if store::index_dir(&v).join("manifest.json").exists() {
            return Ok(None);
        }
        let e = self.embedder()?;
        Ok(Some(store::build(&v, e.as_ref())?))
    }

    /// A document edited outside the server (`--watch`): ingested as human
    /// content (spec §7: `human` → `canonical`), new chunks to the hot tier
    /// (`Store::watch_with`). An unchanged document writes nothing.
    pub fn watch_doc(&self, doc: &str, source: &str) -> Result<store::WatchReport> {
        let _g = self.guard()?;
        let path = crate::paths::doc_file(&self.vault, doc)?;
        let text =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let mut v = Vault::open(&self.vault)?;
        let e = self.embedder()?;
        let mut s = Store::open(&v, e.as_ref())?;
        let m = self.matcher_for(&v);
        let prov = Provenance {
            author_kind: AuthorKind::Human,
            source: source.to_string(),
        };
        s.watch_with(&mut v, e.as_ref(), doc, &text, prov, m)
    }

    /// Moves the hot tier into the cold index if it holds anything
    /// (`mdk index compact`); `None` when the hot tier is empty.
    pub fn compact(&self) -> Result<Option<store::CompactReport>> {
        let _g = self.guard()?;
        let v = Vault::open(&self.vault)?;
        let e = self.embedder()?;
        let st = Store::open(&v, e.as_ref())?.status(&v)?;
        if st.hot_records == 0 && st.hot_snapshots == 0 {
            return Ok(None);
        }
        Ok(Some(store::compact(&v, e.as_ref())?))
    }

    pub fn vault(&self) -> &Path {
        &self.vault
    }

    /// Calls tool `name` for `consumer` (the MCP client or the HTTP
    /// `X-Consumer`; `None`: unidentified, refused by tools that record a
    /// trace); an unknown tool is an error.
    pub fn call(&self, name: &str, a: &Value, consumer: Option<&str>) -> Result<Value> {
        let _g = self.guard()?;
        match name {
            "search" => {
                let x: SearchArgs = args(name, a)?;
                // before anything is opened: an empty query records nothing (K-C1.5)
                mdkindex::search::check_query(&x.query)?;
                let consumer = need_consumer(name, consumer)?;
                let k = x.k.unwrap_or(10);
                if !(1..=1000).contains(&k) {
                    bail!("search: k = {k} outside [1, 1000]");
                }
                let mode = Mode::parse(x.mode.as_deref().unwrap_or("hybrid"))?;
                let mut v = Vault::open(&self.vault)?;
                let e = self.embedder()?;
                let s = Store::open(&v, e.as_ref())?;
                let mut se = Searcher::new(&s, e.as_ref(), consumer)?;
                let r = se.search(
                    &mut v,
                    mode,
                    &x.query,
                    k,
                    x.at.as_deref(),
                    x.include_proposal.unwrap_or(false),
                )?;
                Ok(serde_json::to_value(r)?)
            }
            "read_block" => {
                let x: ReadBlockArgs = args(name, a)?;
                non_empty(name, "doc_path", &x.doc_path)?;
                non_empty(name, "lineage_id", &x.lineage_id)?;
                let consumer = need_consumer(name, consumer)?;
                mdkcore::check_trace_input(consumer, &mdkcore::hash32(""))?;
                let mut v = Vault::open(&self.vault)?;
                let cp = match &x.at {
                    Some(a) => v.find_checkpoint(a)?,
                    None => v.head_checkpoint()?,
                };
                // the lineage must be live in this document at the
                // checkpoint, checked before the trace is written
                let in_doc = v
                    .blocks_at(&x.doc_path, &cp.checkpoint_id)?
                    .iter()
                    .any(|b| b.lineage_id == x.lineage_id);
                if !in_doc {
                    bail!(
                        "read_block: lineage {} is not live in {} at checkpoint {}",
                        x.lineage_id,
                        x.doc_path,
                        cp.checkpoint_id
                    );
                }
                let (mut blocks, trace) = v.read_blocks(
                    std::slice::from_ref(&x.lineage_id),
                    consumer,
                    &cp.checkpoint_id,
                )?;
                let rb = blocks.pop().context("read_blocks returned no block")?;
                Ok(json!({"checkpoint_id": cp.checkpoint_id, "block": rb, "trace": trace}))
            }
            "status" => {
                args::<NoArgs>(name, a)?;
                self.status()
            }
            "propose" => {
                let x: ProposeArgs = args(name, a)?;
                let consumer = need_consumer(name, consumer)?;
                self.propose(x, consumer)
            }
            "promote" => {
                let x: PromoteArgs = args(name, a)?;
                non_empty(name, "lineage_id", &x.lineage_id)?;
                if x.reason.trim().is_empty() {
                    bail!("promote: reason must not be empty");
                }
                let consumer = need_consumer(name, consumer)?;
                // the actor is the consumer, always an agent (K12, K16)
                let actor = mdkcore::Actor::new(mdkcore::ActorKind::Agent, consumer)?;
                let policy = Policy::load(&self.vault)?;
                let hook = policy.promotion.hook();
                let mut v = Vault::open(&self.vault)?;
                let req = v.request(
                    mdkcore::TransitionKind::Promote,
                    &x.lineage_id,
                    mdkcore::Layer::Canonical,
                    actor.clone(),
                    &x.reason,
                )?;
                match hook.decide(&req) {
                    mdkcore::Decision::Deny(why) => Ok(json!({
                        "decision": "denied",
                        "policy": hook.name(),
                        "policy_version": hook.version(),
                        "reason": why,
                        "lineage_id": x.lineage_id,
                        "layer": req.from,
                        "actor": actor,
                    })),
                    mdkcore::Decision::Allow => {
                        let ev = v.promote(&req, hook)?;
                        Ok(json!({
                            "decision": "promoted",
                            "policy": hook.name(),
                            "policy_version": hook.version(),
                            "lineage_id": x.lineage_id,
                            "layer": v.layer_of(&x.lineage_id)?,
                            "event": ev,
                        }))
                    }
                }
            }
            "checkpoint" => {
                let x: CheckpointArgs = args(name, a)?;
                if let Some(n) = &x.name {
                    non_empty(name, "name", n)?;
                }
                let mut v = Vault::open(&self.vault)?;
                Ok(serde_json::to_value(v.checkpoint(x.name.as_deref())?)?)
            }
            "history" => {
                let x: HistoryArgs = args(name, a)?;
                non_empty(name, "lineage_id", &x.lineage_id)?;
                let v = Vault::open(&self.vault)?;
                // the CLI's `history` output (an unknown lineage is an error)
                let layer = v.layer_of(&x.lineage_id).ok();
                let events = v.history(&x.lineage_id)?;
                Ok(json!({"lineage_id": x.lineage_id, "layer": layer, "events": events}))
            }
            "blame" => {
                let x: BlameArgs = args(name, a)?;
                non_empty(name, "doc_path", &x.doc_path)?;
                let v = Vault::open(&self.vault)?;
                let blocks = v.blame(&x.doc_path, x.at.as_deref())?;
                Ok(json!({"doc_path": x.doc_path, "at": x.at, "blocks": blocks}))
            }
            "checkpoints" => {
                args::<NoArgs>(name, a)?;
                Ok(json!({"checkpoints": Vault::open(&self.vault)?.checkpoints()}))
            }
            "trace" => {
                let x: TraceArgs = args(name, a)?;
                let v = Vault::open(&self.vault)?;
                let mut traces = v.traces(x.since_seq.unwrap_or(0))?;
                if let Some(c) = &x.consumer {
                    non_empty(name, "consumer", c)?;
                    traces.retain(|t| &t.consumer == c);
                }
                Ok(json!({"traces": traces}))
            }
            _ => bail!("unknown tool {name:?}"),
        }
    }

    /// `propose` (under the call lock): checks, sync of external edits,
    /// write, agent ingest, record. See the tool description.
    fn propose(&self, x: ProposeArgs, consumer: &str) -> Result<Value> {
        use crate::propose::{self, Mode as PMode};
        let mode = PMode::parse(x.mode.as_deref().unwrap_or("create"))?;
        if x.reason.trim().is_empty() {
            bail!("propose: reason must not be empty");
        }
        match (mode, &x.heading) {
            (PMode::ReplaceSection, None) => bail!("propose: replace_section needs `heading`"),
            (PMode::Create | PMode::Append, Some(_)) => {
                bail!("propose: `heading` applies to replace_section only")
            }
            _ => {}
        }
        let cfg = crate::config::ServerConfig::from_policy(&Policy::load(&self.vault)?)?;
        let size = x.content.len() as u64;
        if size > cfg.max_propose_bytes {
            bail!(
                "propose: content is {size} bytes; the limit is [server] max_propose_bytes = {}",
                cfg.max_propose_bytes
            );
        }
        propose::check_doc_path(&x.doc_path)?;
        let path = crate::paths::doc_file(&self.vault, &x.doc_path)?;
        // on a case- or normalization-insensitive file system another
        // spelling of an existing name would write into a document the log
        // knows under another doc_path
        crate::paths::exact_on_disk(&self.vault, &x.doc_path)?;
        let mut v = Vault::open(&self.vault)?;
        let e = self.embedder()?;
        let mut s = Store::open(&v, e.as_ref())?;
        let m = self.matcher_for(&v);
        let exists = std::fs::symlink_metadata(&path).is_ok();
        if exists && !path.is_file() {
            bail!("propose: {} is not a file", x.doc_path);
        }
        let old = if exists {
            Some(
                std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?,
            )
        } else {
            None
        };
        let new = match (mode, &old) {
            (PMode::Create, Some(_)) => bail!(
                "propose: {} exists; propose never overwrites (use mode append or replace_section)",
                x.doc_path
            ),
            (PMode::Create, None) => {
                if v.doc_state(&x.doc_path).is_some() {
                    bail!(
                        "propose: {} is in the vault's log but its file is missing",
                        x.doc_path
                    );
                }
                propose::append("", &x.content)
            }
            (PMode::Append | PMode::ReplaceSection, None) => bail!(
                "propose: {} does not exist ({} needs an existing document)",
                x.doc_path,
                mode.as_str()
            ),
            (PMode::Append | PMode::ReplaceSection, Some(_))
                if v.doc_state(&x.doc_path).is_none() =>
            {
                bail!(
                    "propose: {} is not a document of the vault's log ({} needs one)",
                    x.doc_path,
                    mode.as_str()
                )
            }
            (PMode::Append, Some(o)) => propose::append(o, &x.content),
            (PMode::ReplaceSection, Some(o)) => propose::replace_section(
                o,
                x.heading.as_deref().expect("checked above"),
                &x.content,
            )?,
        };
        if new.trim().is_empty() {
            bail!("propose: the resulting document would be empty");
        }
        // an edit made outside the server and not yet ingested is human
        // content (the watcher's rule); it must not be attributed to the agent
        let synced = match &old {
            Some(o) => {
                let prov = Provenance {
                    author_kind: AuthorKind::Human,
                    source: "mdk-serve-watch".to_string(),
                };
                s.watch_with(&mut v, e.as_ref(), &x.doc_path, o, prov, m)?
                    .events
            }
            None => 0,
        };
        // write: directories this call creates are remembered (and removed
        // again if the ingest fails); the file goes through a fresh hidden
        // temporary file (never a symbolic link) and a rename
        let dir = path.parent().context("document without a directory")?;
        let mut created: Vec<std::path::PathBuf> = Vec::new();
        for d in dir.ancestors().collect::<Vec<_>>().into_iter().rev() {
            if std::fs::symlink_metadata(d).is_err() {
                std::fs::create_dir(d).with_context(|| format!("create {}", d.display()))?;
                created.push(d.to_path_buf());
            }
        }
        crate::paths::replace_file(&path, new.as_bytes())?;
        let prov = Provenance {
            author_kind: AuthorKind::Agent,
            source: consumer.to_string(),
        };
        let report = match s.watch_with(&mut v, e.as_ref(), &x.doc_path, &new, prov, m) {
            Ok(r) => r,
            Err(err) => {
                // the file is restored: it never holds text the vault did not take
                let undo = match &old {
                    Some(o) => crate::paths::replace_file(&path, o.as_bytes()),
                    // and the directories this call created go again
                    None => std::fs::remove_file(&path)
                        .map_err(anyhow::Error::from)
                        .and_then(|_| {
                            for d in created.iter().rev() {
                                std::fs::remove_dir(d)
                                    .with_context(|| format!("remove {}", d.display()))?;
                            }
                            Ok(())
                        }),
                };
                return Err(match undo {
                    Ok(()) => err.context("propose: ingest failed; the file was restored"),
                    Err(u) => err.context(format!(
                        "propose: ingest failed and restoring {} failed too: {u:#}",
                        path.display()
                    )),
                });
            }
        };
        let record = json!({
            "doc_path": x.doc_path,
            "mode": mode.as_str(),
            "heading": x.heading,
            "consumer": consumer,
            "reason": x.reason,
            "snapshot_id": report.snapshot_id,
            "events": report.events,
        });
        let mut line = serde_json::to_vec(&record)?;
        line.push(b'\n');
        let rp = v.store_dir().join("proposals.jsonl");
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rp)
            .and_then(|mut f| {
                std::io::Write::write_all(&mut f, &line)?;
                f.sync_data()
            })
            .with_context(|| {
                format!(
                    "propose: {} was written and ingested (snapshot {}), but recording its reason in {} failed",
                    x.doc_path,
                    report.snapshot_id,
                    rp.display()
                )
            })?;
        let blocks = v.blame(&x.doc_path, None)?;
        Ok(json!({
            "doc_path": x.doc_path,
            "mode": mode.as_str(),
            "bytes": new.len(),
            "synced_external_events": synced,
            "snapshot_id": report.snapshot_id,
            "events": report.events,
            "chunks_embedded": report.embedded,
            "blocks": blocks,
            "events_hash": v.events_hash()?,
        }))
    }

    fn status(&self) -> Result<Value> {
        let v = Vault::open(&self.vault)?;
        let policy = Policy::load(&self.vault)?;
        let manifest = mdkindex::store::index_dir(&v).join("manifest.json");
        let embedder = if manifest.exists() {
            let m: mdkindex::store::Manifest = serde_json::from_slice(
                &std::fs::read(&manifest)
                    .with_context(|| format!("read {}", manifest.display()))?,
            )
            .with_context(|| format!("{} is not an index manifest", manifest.display()))?;
            Value::String(m.embedder_id)
        } else {
            Value::Null
        };
        Ok(json!({
            "vault": self.vault.display().to_string(),
            "spec_version": SPEC_VERSION,
            "mdk_version": MDK_VERSION,
            "level_claimed": LEVEL_CLAIMED,
            "embedder": embedder,
            "index": if embedder.is_null() { "absent" } else { "present" },
            "last_checkpoint": v.checkpoints().last(),
            "policy": policy.to_json(),
            "server_policy": crate::config::ServerConfig::from_policy(&policy)?.to_json(),
        }))
    }
}
