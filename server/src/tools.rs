//! The tools of the MCP server and the HTTP API (GOAL P1-D "MCP araçları"),
//! independent of the transport: a tool takes a JSON object of arguments and
//! the caller (channel and consumer) and returns JSON, or an error with a
//! stable code (`crate::error`, K-D1.5). Every tool opens the vault afresh
//! (`Vault::open` replays the log), so a change made by another process —
//! `rootmark promote` from the command line — is seen by the next call. Writing
//! calls (content, governance or a consumption trace) run one at a time in
//! arrival order through the vault's writer queue; read-only calls run in
//! parallel (`crate::queue`, K-D1.4). The channel fixes the actor (K-D1.3):
//! MCP is an agent, HTTP the vault's human (`[identity] human_id`).
//!
//! The tools expose existing primitives only (spec §7–§9): no tool decides a
//! promotion, no tool other than `propose` writes Markdown, and the output of
//! a tool that has a CLI twin is the JSON that command prints.

use crate::attest::{Attesting, Ingest};
use crate::bail_code;
use crate::error::{Code, CodeExt};
use crate::queue::{Queue, Ticket};
use anyhow::{bail, Context, Result};
use rootmarkcore::{
    Actor, ActorKind, AuthorKind, Decision, EditMode, EditRequest, MatcherVersion, Policy,
    Provenance, Vault,
};
use rootmarkindex::search::{Mode, Searcher};
use rootmarkindex::store::{self, Store};
use rootmarkindex::OnnxMiniLm;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Spec version this build implements (spec/SPEC.md title).
pub const SPEC_VERSION: &str = "2.0.1";
/// Conformance level the reference implementation claims (spec §12;
/// `make conformance`).
pub const LEVEL_CLAIMED: u8 = 3;
/// `rootmark` version (one SemVer for every crate, PACKAGES).
pub const ROOTMARK_VERSION: &str = env!("CARGO_PKG_VERSION");

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
            description: "State of the served vault: its path, the spec and rootmark versions, the \
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
                promotes them. The vault's edit policy decides first (`[edit]`, default \
                `append_only`): a write that would change or remove a `canonical` block, or a \
                canonical retrieval unit (text appended to a section of canonical blocks joins \
                their chunk), is refused with code `denied` and writes nothing. Agent guideline \
                (K-E0.4): text appended to a document of human blocks is accepted only as a section \
                of its own, so start it with a heading (`## …`); an append without one that the \
                policy refuses answers `policy_violation` with the hint \"start the appended text \
                with a heading\"; `replace_section` is allowed only on a section made of \
                proposals. Change and delete events name the actor (spec §4.3). Nothing but the \
                agent's text is written to the file (no ids, no metadata). `reason` is recorded \
                beside the vault (`.rootmark/proposals.jsonl`). \
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
                the channel gives (it cannot be chosen): over MCP the agent agent:<client>, over \
                HTTP the vault's human human:<[identity] human_id>. Under `human_only` (the \
                default) an agent's request is denied: the answer is `decision: denied` with the \
                policy and its reason, not an error, and no event is written; a human's request \
                writes a `Promoted` event (`decision: promoted`). `deny` refuses everyone.",
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
                at`, `blame at`, `read_block at`). Returns the checkpoint. A state has one \
                checkpoint: if a search or read already created its unnamed checkpoint, `name` \
                names that one (same id, a `CheckpointNamed` event). A name another checkpoint \
                holds, or another name for an already named state, is an error (`exists`).",
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

/// The transport a call came through; it fixes the actor's kind (K-D1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// MCP stdio: the agent channel; the actor is `agent:<client>`
    Mcp,
    /// HTTP with the token: the human channel; the actor is
    /// `human:<[identity] human_id>`
    Http,
    /// the `rootmark` command line (`rootmark propose`): the actor is `agent:<--source>`
    Cli,
}

/// Who calls: the channel and the consumer identity (the MCP client
/// `<name>/<version>`, the HTTP `X-Consumer`, the CLI `--source`). The
/// consumer labels traces; it never changes the actor's kind.
#[derive(Debug, Clone, Copy)]
pub struct Caller<'a> {
    pub channel: Channel,
    pub consumer: Option<&'a str>,
}

impl<'a> Caller<'a> {
    pub fn mcp(consumer: Option<&'a str>) -> Self {
        Caller {
            channel: Channel::Mcp,
            consumer,
        }
    }
    pub fn http(consumer: &'a str) -> Self {
        Caller {
            channel: Channel::Http,
            consumer: Some(consumer),
        }
    }
}

/// Tools that write the vault or a consumption trace: they run through the
/// vault's writer queue (K-D1.4). The others only read.
pub fn is_writer(tool: &str) -> bool {
    matches!(
        tool,
        "search" | "read_block" | "propose" | "promote" | "checkpoint" | "attest"
    )
}

/// Tools an HTTP caller may use only with `[identity] human_id` (K-D1.3):
/// those that change the vault's content or governance state (and, P2-0,
/// the attestation endpoints).
pub fn needs_human_id(tool: &str) -> bool {
    matches!(
        tool,
        "propose" | "promote" | "checkpoint" | "attest_session" | "attest"
    )
}

/// Endpoints of the human channel only (CLIENT_CONTRACT §2.6; spec 2.0.0
/// §7.1): `POST /v1/attest_session`, `POST /v1/attest`. They are not tools:
/// MCP lists and serves neither (an agent's channel never attests).
pub const HTTP_ONLY: [&str; 2] = ["attest_session", "attest"];

/// Whether `name` is an HTTP-only endpoint.
pub fn is_http_only(name: &str) -> bool {
    HTTP_ONLY.contains(&name)
}

/// What `Kernel::compact` did.
#[derive(Debug)]
pub enum Compacted {
    /// the hot tier moved into the cold index
    Done(store::CompactReport),
    /// the hot tier is empty
    Empty,
    /// the index is not `ok` (its `status.index.state`): nothing to compact
    Skipped(&'static str),
}

/// What `Kernel::ensure_index` found and did at the start of `rootmark serve`.
#[derive(Debug)]
pub enum IndexAtStart {
    /// the index is of this format and lexical record
    Current,
    /// there was none: built
    Built(store::BuildReport),
    /// it was stale (the reason): rebuilt (`[index] auto_rebuild`)
    Rebuilt(String, store::BuildReport),
    /// it is stale (the reason) and `auto_rebuild = false`: left as it is
    Stale(String),
    /// its manifest cannot be read (the error): left as it is, never
    /// rebuilt over (damage, not staleness); the index's tools answer
    /// `index_stale` until `rootmark index build` (K-E1.2)
    Damaged(String),
}

/// The served vault and its writer queue.
pub struct Kernel {
    /// the vault path as given (`rootmark serve --vault`); `status` reports it
    vault: PathBuf,
    /// `--matcher` for ingests the server makes (watcher, `propose`); `None`:
    /// the vault's lock, else v3 (as `rootmark index watch`)
    matcher: Option<MatcherVersion>,
    queue: Arc<Queue>,
    /// loaded on first use, then shared (ONNX session behind a mutex)
    embedder: Mutex<Option<Arc<OnnxMiniLm>>>,
    /// attestation sessions and window (spec 2.0.0 §7.1), in memory
    attesting: Attesting,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttestArgs {
    doc_path: String,
    content_hash: String,
    saved_at_seq: Option<u64>,
    session_id: String,
    mac: String,
    /// `false`: record the attestation for later observations only, never
    /// raise an earlier unattested change (`rootmark attest --all`)
    late: Option<bool>,
    /// a rename by the client: accepted only when the file at `doc_path`
    /// holds exactly the canonical head of this document (contract §2.6)
    renamed_from: Option<String>,
    /// a link update Obsidian wrote while renaming the note now at this
    /// path (P2-0.1 T1): accepted only when the bytes differ from the
    /// document's head in link targets naming that note, in canonical
    /// blocks (contract §2.6)
    link_update_of: Option<String>,
    /// with `link_update_of`: the renamed note's old path as the client
    /// knows it (used when no accepted rename attestation gave it)
    link_update_from: Option<String>,
    /// the saved bytes (UTF-8 text; P2-0.1 T2): they must hash to
    /// `content_hash`; kept in memory for a partial match, never stored
    content: Option<String>,
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
pub struct ProposeArgs {
    pub doc_path: String,
    pub content: String,
    pub reason: String,
    pub mode: Option<String>,
    pub heading: Option<String>,
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
        bail_code!(
            Code::InvalidArguments,
            "{tool}: arguments must be a JSON object"
        );
    }
    serde_json::from_value(a)
        .with_context(|| format!("{tool}: invalid arguments"))
        .code(Code::InvalidArguments)
}

/// The consumer a trace-recording tool records; none → error (an MCP client
/// that has not identified itself in `initialize`, or no `X-Consumer`).
fn need_consumer<'c>(tool: &str, consumer: Option<&'c str>) -> Result<&'c str> {
    match consumer {
        Some(c) if !c.trim().is_empty() => Ok(c),
        _ => bail_code!(
            Code::InvalidArguments,
            "{tool}: records a consumption trace and needs a consumer identity (MCP initialize clientInfo, or HTTP X-Consumer)"
        ),
    }
}

fn non_empty(tool: &str, field: &str, v: &str) -> Result<()> {
    if v.is_empty() {
        bail_code!(Code::InvalidArguments, "{tool}: {field} must not be empty");
    }
    Ok(())
}

fn open_vault(vault: &Path) -> Result<Vault> {
    Vault::open(vault).code(Code::Internal)
}

/// A checkpoint `at` names (id or name); unknown → `not_found`.
fn checkpoint_at(v: &Vault, at: &str) -> Result<rootmarkcore::Checkpoint> {
    v.find_checkpoint(at).code(Code::NotFound)
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
            queue: Queue::new(),
            embedder: Mutex::new(None),
            attesting: Attesting::new(),
        })
    }

    /// The vault's writer queue (K-D1.4): transports take a ticket when a
    /// writing call arrives (`ticket`) and pass it to `run`.
    pub fn ticket(&self) -> Ticket {
        self.queue.ticket()
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

    /// The vault's recipe and server settings; an invalid `policy.toml` is
    /// `policy_violation`.
    fn policy(&self) -> Result<(Policy, crate::config::ServerConfig)> {
        let p = Policy::load(&self.vault).code(Code::PolicyViolation)?;
        let c = crate::config::ServerConfig::from_policy(&p)
            .with_context(|| format!("policy {} rejected", Policy::path(&self.vault).display()))
            .code(Code::PolicyViolation)?;
        Ok((p, c))
    }

    /// The actor of a call (K-D1.3): MCP and CLI → `agent:<consumer>`;
    /// HTTP → `human:<[identity] human_id>`, and without `human_id` a
    /// writing tool is `denied`.
    fn actor(&self, tool: &str, caller: &Caller) -> Result<Actor> {
        match caller.channel {
            Channel::Mcp | Channel::Cli => {
                let c = need_consumer(tool, caller.consumer)?;
                Actor::new(ActorKind::Agent, c).code(Code::InvalidArguments)
            }
            Channel::Http => {
                let (_, cfg) = self.policy()?;
                match cfg.human_id {
                    Some(id) => Actor::new(ActorKind::Human, &id).code(Code::PolicyViolation),
                    None => bail_code!(
                        Code::Denied,
                        "{tool}: the HTTP channel acts for the human of [identity] human_id, and policy.toml sets none; add `[identity] human_id = \"<id>\"`"
                    ),
                }
            }
        }
    }

    /// The actor of a watched edit (spec §7 `human`): the vault's human
    /// (`[identity] human_id`), or `human:unidentified` without one.
    fn watch_actor(&self) -> Result<Actor> {
        let (_, cfg) = self.policy()?;
        let id = cfg.human_id.unwrap_or_else(|| "unidentified".to_string());
        Actor::new(ActorKind::Human, &id)
    }

    /// Builds the index if the vault has none (`rootmark index build`); `None`
    /// if one exists. The served index must exist before searches and
    /// watched edits.
    pub fn ensure_index(&self) -> Result<Option<store::BuildReport>> {
        self.ticket().run(|| {
            let v = Vault::open(&self.vault)?;
            if store::index_dir(&v).join("manifest.json").exists() {
                return Ok(None);
            }
            let e = self.embedder()?;
            Ok(Some(store::build(&v, e.as_ref())?))
        })
    }

    /// `rootmark serve`'s start: `ensure_index`, and an index of another format
    /// is rebuilt when `auto_rebuild` (`[index] auto_rebuild`, K-E0.3); see
    /// `IndexAtStart`. A stale index left in place makes searches answer
    /// `index_stale`.
    pub fn start_index(&self, auto_rebuild: bool) -> Result<IndexAtStart> {
        self.ticket().run(|| {
            let v = Vault::open(&self.vault)?;
            let missing = !store::index_dir(&v).join("manifest.json").exists();
            let stale = match missing {
                true => None,
                // a damaged manifest is not staleness: the service starts as
                // before (P1-D.1), nothing is rebuilt over it, and the
                // index's tools answer `index_stale` (K-E1.2)
                false => match store::staleness(&v) {
                    Ok(None) => return Ok(IndexAtStart::Current),
                    Ok(Some(s)) => Some(s),
                    Err(e) => return Ok(IndexAtStart::Damaged(format!("{e:#}"))),
                },
            };
            if let (Some(s), false) = (&stale, auto_rebuild) {
                return Ok(IndexAtStart::Stale(s.0.clone()));
            }
            let e = self.embedder()?;
            let r = store::build(&v, e.as_ref())?;
            Ok(match stale {
                None => IndexAtStart::Built(r),
                Some(s) => IndexAtStart::Rebuilt(s.0, r),
            })
        })
    }

    /// A document edited outside the server (`--watch`): an external change
    /// (spec 2.0.0 §7.1, `external_plan`), new chunks to the hot tier. An
    /// unchanged document writes nothing. Runs in the writer queue.
    pub fn watch_doc(&self, doc: &str, source: &str) -> Result<store::WatchReport> {
        self.ticket().run(|| {
            let path = crate::paths::doc_file(&self.vault, doc)?;
            let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
            let mut v = Vault::open(&self.vault)?;
            let e = self.embedder()?;
            let mut s = Store::open(&v, e.as_ref())?;
            let (policy, _) = self.policy()?;
            let (_, r) = self.ingest_external(
                &mut v,
                Some((&mut s, e.as_ref())),
                &policy,
                doc,
                &bytes,
                source,
            )?;
            Ok(r.expect("a store was given"))
        })
    }

    /// Ingests an external change of `doc` (`external_plan`) and commits it
    /// (with `store`: the hot tier too). A partial match (P2-0.1 T2) is two
    /// commits: the attested bytes as an attested change, the line
    /// `{doc_path, content_hash, at_seq, partial: true}` in the attestation
    /// log, then the observed bytes as an unattested change. The number of
    /// events and, with a store, its report (the two merged).
    fn ingest_external(
        &self,
        v: &mut Vault,
        mut store: Option<(&mut Store, &dyn rootmarkindex::Embedder)>,
        policy: &Policy,
        doc: &str,
        bytes: &[u8],
        source: &str,
    ) -> Result<(usize, Option<store::WatchReport>)> {
        let mut events = 0;
        let mut report: Option<store::WatchReport> = None;
        let mut force_unattested = false;
        loop {
            let (plan, after) =
                self.external_plan(v, policy, doc, bytes, source, force_unattested)?;
            let n = plan.events.len();
            match &mut store {
                Some((s, e)) => {
                    let r = s.watch_plan(v, *e, plan)?;
                    report = Some(match report.take() {
                        None => r,
                        Some(mut old) => {
                            old.events += r.events;
                            old.embedded += r.embedded;
                            old.snapshot_id = r.snapshot_id;
                            old.chunks = r.chunks;
                            old
                        }
                    });
                }
                None => {
                    v.commit_plan(plan)?;
                }
            }
            events += n;
            let partial = after.partial_hash();
            after.done(self, v, policy)?;
            let Some(saved_hash) = partial else {
                return Ok((events, report));
            };
            // the rest of the observed bytes: unattested
            let mut line =
                rootmarkcore::attest::Attestation::new(doc, &saved_hash, v.next_seq(), None)?;
            line.partial = Some(true);
            v.record_attestation_line(line)?;
            force_unattested = true;
        }
    }

    /// The ingest of an external change of `doc` (the watcher, and
    /// `propose`'s sync of an edit made outside the server): under
    /// `[attestation] unattested = "canonical"`, or when a human-channel
    /// client attested these bytes within the window, human content
    /// (spec §7: `canonical`, the vault's human, `source`); otherwise
    /// `unverified` with the actor `unattested_fs` (spec 2.0.0 §7.1). The
    /// returned `External` must be told when the plan is committed.
    fn external_plan(
        &self,
        v: &Vault,
        policy: &Policy,
        doc: &str,
        bytes: &[u8],
        source: &str,
        force_unattested: bool,
    ) -> Result<(rootmarkcore::IngestPlan, External)> {
        let text = std::str::from_utf8(bytes)
            .with_context(|| format!("{doc} is not UTF-8"))?
            .to_string();
        let hash = rootmarkcore::attest::content_hash(bytes);
        let m = self.matcher_for(v);
        // an unchanged document is no change: nothing to decide
        let probe = v.plan_ingest(
            doc,
            &text,
            Provenance {
                author_kind: AuthorKind::Human,
                source: source.to_string(),
            },
            m,
            None,
            Some(&self.watch_actor()?),
        )?;
        if probe.events.is_empty() {
            return Ok((probe, External::nothing(doc, &hash)));
        }
        let (ingest, observed_ms) = if force_unattested {
            (Ingest::Unverified, self.attesting.now_ms())
        } else {
            self.attesting.decide(policy, doc, &hash, &text)?
        };
        if let Ingest::Partial(_, saved) = &ingest {
            // the attested bytes first, as an attested change (P2-0.1 T2)
            let plan = v.plan_ingest(
                doc,
                saved,
                Provenance {
                    author_kind: AuthorKind::Human,
                    source: source.to_string(),
                },
                m,
                None,
                Some(&self.watch_actor()?),
            )?;
            return Ok((
                plan,
                External {
                    doc: doc.to_string(),
                    hash: rootmarkcore::attest::content_hash(saved.as_bytes()),
                    ingest,
                    observed_ms,
                    named: Vec::new(),
                },
            ));
        }
        // the layer an attested ingest gives each lineage: what a late
        // attestation restores (spec 2.0.0 §7.1)
        let attested_layers: std::collections::HashMap<String, rootmarkcore::Layer> = probe
            .blocks_after
            .iter()
            .map(|b| (b.lineage_id.clone(), b.layer))
            .collect();
        if ingest == Ingest::Unverified {
            self.attesting.unattested_base(
                policy,
                doc,
                &hash,
                observed_ms,
                rootmarkcore::attest::LinkBase::head(v, doc)?,
                rootmarkcore::attest::block_versions(&text, doc)?,
            )?;
        }
        let plan = match &ingest {
            Ingest::Canonical | Ingest::Attested(_) | Ingest::Partial(..) => probe,
            Ingest::Unverified => v.plan_ingest(
                doc,
                &text,
                Provenance {
                    author_kind: AuthorKind::Human,
                    source: crate::attest::UNATTESTED_SOURCE.to_string(),
                },
                m,
                Some(rootmarkcore::Layer::Unverified),
                Some(&crate::attest::unattested_actor()),
            )?,
        };
        // the version this ingest gives each lineage: a late attestation
        // raises the lineage only while it is still there (Z2 review 1)
        let versions: std::collections::HashMap<&str, &str> = plan
            .blocks_after
            .iter()
            .map(|b| (b.lineage_id.as_str(), b.version_id.as_str()))
            .collect();
        let named: Vec<rootmarkcore::attest::Raise> = plan
            .events
            .iter()
            .flat_map(|e| e.lineage_ids.iter().cloned())
            .filter_map(|l| {
                let to = *attested_layers.get(&l)?;
                let version_id = (*versions.get(l.as_str())?).to_string();
                Some(rootmarkcore::attest::Raise {
                    lineage_id: l,
                    version_id,
                    to,
                })
            })
            .collect();
        Ok((
            plan,
            External {
                doc: doc.to_string(),
                hash,
                ingest,
                observed_ms,
                named,
            },
        ))
    }

    /// A rename attestation (contract §2.6): the file now at `to` must hold
    /// bytes hashing to `content_hash` whose blocks are, one for one, the
    /// head snapshot of `from`, every one of them live and `canonical` —
    /// a rename moves canonical content and nothing else (an agent's bytes
    /// that reached `from` unobserved, or a proposal, make it `denied`).
    fn check_rename(&self, v: &Vault, from: &str, to: &str, text: &str) -> Result<()> {
        crate::propose::check_doc_path(from)?;
        let (drafts, _) = rootmarkcore::parse_document(text, to)?;
        let Some(head) = v.snapshot(from)? else {
            bail_code!(Code::Denied, "attest: {from} is not a document of the log");
        };
        let blocks = v.snapshot_blocks(&head)?;
        let st = v.doc_state(from).context("document state")?;
        let same = drafts.len() == blocks.len()
            && drafts
                .iter()
                .zip(&blocks)
                .all(|(d, b)| d.version_id == b.version_id);
        let canonical = blocks.iter().all(|b| {
            b.layer == rootmarkcore::Layer::Canonical && !st.inactive.contains(&b.lineage_id)
        });
        if !same || !canonical {
            bail_code!(
                Code::Denied,
                "attest: {to} is not a rename of the canonical content of {from}"
            );
        }
        Ok(())
    }

    /// A link-update attestation (P2-0.1 T1, contract §2.6): the file at
    /// `doc` holds bytes hashing to `content_hash` that are a link update
    /// (`rootmarkcore::attest::check_link_update`) for the note now at
    /// `renamed` of: the document's head before an unattested ingest of
    /// these very bytes, when the watcher already ingested them (a late
    /// attestation must not raise what preceded it, Y1 review 1); else its
    /// head now — or, for a document the log does not know yet because it
    /// was just renamed (a note linking to itself), the head of the path it
    /// was renamed from. Changed link targets must name `renamed`, and its
    /// old path when an accepted rename attestation gave it. Anything else
    /// is `denied`: Obsidian's rewrite of a note during a rename never
    /// vouches for bytes an agent wrote there.
    fn check_link_update(
        &self,
        v: &Vault,
        policy: &Policy,
        x: &AttestArgs,
        text: &str,
    ) -> Result<()> {
        let (doc, content_hash) = (x.doc_path.as_str(), x.content_hash.as_str());
        let renamed = x
            .link_update_of
            .as_deref()
            .context("a link update names the renamed note")?;
        let from_hint = x.link_update_from.as_deref();
        crate::propose::check_doc_path(renamed)?;
        rootmarkcore::attest::check_doc_path(renamed).code(Code::InvalidArguments)?;
        if !crate::paths::doc_file(&self.vault, renamed)?.is_file() {
            bail_code!(
                Code::Denied,
                "attest: link_update_of {renamed} is not a note of the vault"
            );
        }
        let base = match self
            .attesting
            .pending_base(policy, doc, content_hash)
            .code(Code::Denied)?
        {
            Some(b) => b,
            None => {
                let d = self
                    .attesting
                    .base_of(doc, |d| v.snapshot(d).ok().flatten().is_some())
                    .code(Code::Denied)?;
                let b = rootmarkcore::attest::LinkBase::head(v, &d)?
                    .with_context(|| format!("attest: {d} is not a document of the log"))
                    .code(Code::Denied)?;
                // the head of another path (the log does not know `doc` yet):
                // its content became `doc`'s by a rename, which moves only
                // canonical content — whatever the old path holds now must be
                // that, all of it (automated security review of Y1)
                if d != doc && !b.blocks.iter().all(|x| x.canonical) {
                    bail_code!(
                        Code::Denied,
                        "attest: {d}, the path {doc} was renamed from, no longer holds only canonical content"
                    );
                }
                b
            }
        };
        // the renamed note's old path: from an accepted rename attestation,
        // else as the client states it (`link_update_from`)
        let was = match self.attesting.renamed_from_of(renamed)? {
            Some(w) => Some(w),
            None => from_hint.map(str::to_string),
        };
        let changed = rootmarkcore::attest::check_link_update(&base, text, renamed, was.as_deref())
            .map_err(|e| {
                anyhow::anyhow!(
                    "attest: {doc} is not a link update of {} for {renamed}: {e:#}",
                    base.doc_path
                )
            })
            .code(Code::Denied)?;
        if changed == 0 {
            // a link update changes a link (Y1 security review B2)
            bail_code!(
                Code::Denied,
                "attest: {doc} changes no link target naming {renamed}"
            );
        }
        Ok(())
    }

    /// Opens the vault's attestation session (stage 1; CLIENT_CONTRACT §2.6).
    fn attest_session(&self) -> Result<Value> {
        let (sid, key, ttl) = self.attesting.open_session()?;
        Ok(json!({"session_id": sid, "key": key, "ttl_s": ttl, "stage": 1}))
    }

    /// Records an attestation (spec 2.0.0 §7.1, §8) after checking its MAC,
    /// and raises the unattested ingests it matches late. In the writer
    /// queue.
    fn attest(&self, x: AttestArgs) -> Result<Value> {
        crate::propose::check_doc_path(&x.doc_path)?;
        rootmarkcore::attest::check_doc_path(&x.doc_path).code(Code::InvalidArguments)?;
        if x.content_hash.len() != 32
            || !x
                .content_hash
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            bail_code!(
                Code::InvalidArguments,
                "attest: content_hash must be 32 lowercase hexadecimal characters"
            );
        }
        self.attesting
            .verify(
                &x.session_id,
                &x.mac,
                &x.doc_path,
                &x.content_hash,
                x.saved_at_seq,
            )
            .code(Code::Denied)?;
        let (policy, _) = self.policy()?;
        let mut v = open_vault(&self.vault)?;
        if x.renamed_from.is_some() && x.link_update_of.is_some() {
            bail_code!(
                Code::InvalidArguments,
                "attest: renamed_from and link_update_of exclude each other"
            );
        }
        if x.link_update_from.is_some() && x.link_update_of.is_none() {
            bail_code!(
                Code::InvalidArguments,
                "attest: link_update_from goes with link_update_of"
            );
        }
        // the saved bytes the client sent (P2-0.1 T2): they must be the
        // attested ones; kept in memory for a partial match only
        if let Some(c) = &x.content {
            if rootmarkcore::attest::content_hash(c.as_bytes()) != x.content_hash {
                bail_code!(
                    Code::InvalidArguments,
                    "attest: content does not hash to content_hash"
                );
            }
        }
        // the bytes a rename or a link update is checked on: the file on
        // disk, which must hold the attested bytes (also when `content` is
        // sent: Y2 review 2)
        let checked = |s: &Self| -> Result<String> {
            let path = crate::paths::doc_file(&s.vault, &x.doc_path)?;
            let bytes = std::fs::read(&path)
                .with_context(|| format!("attest: read {}", path.display()))
                .code(Code::Denied)?;
            if rootmarkcore::attest::content_hash(&bytes) != x.content_hash {
                bail_code!(
                    Code::Denied,
                    "attest: {} on disk does not hold the attested bytes",
                    x.doc_path
                );
            }
            String::from_utf8(bytes)
                .with_context(|| format!("attest: {} is not UTF-8", x.doc_path))
                .code(Code::Denied)
        };
        if let Some(from) = &x.renamed_from {
            let text = checked(self)?;
            self.check_rename(&v, from, &x.doc_path, &text)?;
            self.attesting.renamed(from, &x.doc_path)?;
        }
        if x.link_update_of.is_some() {
            let text = checked(self)?;
            self.check_link_update(&v, &policy, &x, &text)?;
            self.attesting.link_update();
        }
        let mut a = rootmarkcore::attest::Attestation::new(
            &x.doc_path,
            &x.content_hash,
            v.next_seq(),
            x.saved_at_seq,
        )
        .code(Code::InvalidArguments)?;
        let late_on = x.late != Some(false);
        let (late, ambiguous) = self
            .attesting
            .attested(&policy, &a, late_on, x.content.clone())?;
        // a late partial match (P2-0.1 T2): the one pending unattested
        // change whose blocks contain the attested ones — never while the
        // file holds the attested bytes (their own observation is coming:
        // Y2 review 1 a), nor when the exact late match was ambiguous
        let mut partial = None;
        if late.is_empty() && !ambiguous && late_on {
            if let Some(c) = &x.content {
                let on_disk = crate::paths::doc_file(&self.vault, &x.doc_path)
                    .ok()
                    .and_then(|p| std::fs::read(p).ok())
                    .is_some_and(|b| rootmarkcore::attest::content_hash(&b) == x.content_hash);
                if !on_disk {
                    let av = rootmarkcore::attest::block_versions(c, &x.doc_path)?;
                    if let Some(rs) =
                        self.attesting
                            .late_partial(&policy, &x.doc_path, &x.content_hash, &av)?
                    {
                        a.partial = Some(true);
                        self.attesting.forget(&policy, &a.attestation_id)?;
                        partial = Some(rs);
                    }
                }
            }
        }
        let (a, recorded) = v.record_attestation_line(a)?;
        self.attesting.drop_bases(&a.doc_path, &a.content_hash)?;
        let mut raised: Vec<String> = Vec::new();
        for u in late {
            for ev in v.raise_late(&u.doc_path, &u.lineages, &a.attestation_id)? {
                raised.extend(ev.lineage_ids);
            }
        }
        if let Some(rs) = partial {
            for ev in v.raise_late(&a.doc_path, &rs, &a.attestation_id)? {
                raised.extend(ev.lineage_ids);
            }
        }
        Ok(json!({
            "attestation_id": a.attestation_id,
            "doc_path": a.doc_path,
            "content_hash": a.content_hash,
            "at_seq": a.at_seq,
            "recorded": recorded,
            "raised": raised,
        }))
    }

    /// Moves the hot tier into the cold index if it holds anything
    /// (`rootmark index compact`). Runs in the writer queue. An index that is
    /// not `ok` (`stale`, `missing`, `damaged`: `status.index.state`) has no
    /// hot tier this process could have filled — its tools answer
    /// `index_stale` — so there is nothing to compact: `Skipped` with the
    /// state, not an error (P1-E.1 verification, recommendation 3; P2-0/Z0).
    pub fn compact(&self) -> Result<Compacted> {
        self.ticket().run(|| {
            let v = Vault::open(&self.vault)?;
            let ix = index_state(&v);
            if ix.state != "ok" {
                return Ok(Compacted::Skipped(ix.state));
            }
            let e = self.embedder()?;
            let st = Store::open(&v, e.as_ref())?.status(&v)?;
            if st.hot_records == 0 && st.hot_snapshots == 0 {
                return Ok(Compacted::Empty);
            }
            Ok(Compacted::Done(store::compact(&v, e.as_ref())?))
        })
    }

    pub fn vault(&self) -> &Path {
        &self.vault
    }

    /// Calls tool `name` for `caller`; a writing tool waits for its turn in
    /// the writer queue (taken now), a reading tool runs at once (in
    /// parallel with other readers). An unknown tool is `not_found`. Every
    /// error carries a code (`crate::error`).
    pub fn call(&self, name: &str, a: &Value, caller: Caller) -> Result<Value> {
        let t = is_writer(name).then(|| self.ticket());
        self.run(t, name, a, caller)
    }

    /// `call` with the ticket the transport took when the call arrived
    /// (writing tools; `None` for the others).
    pub fn run(
        &self,
        ticket: Option<Ticket>,
        name: &str,
        a: &Value,
        caller: Caller,
    ) -> Result<Value> {
        let http_only = is_http_only(name) && caller.channel == Channel::Http;
        if !http_only && !specs().iter().any(|s| s.name == name) {
            bail_code!(Code::NotFound, "unknown tool {name:?}");
        }
        match (is_writer(name), ticket) {
            (true, Some(t)) => t.run(|| self.dispatch(name, a, caller)),
            (true, None) => self.ticket().run(|| self.dispatch(name, a, caller)),
            (false, t) => {
                drop(t);
                let _r = self.queue.read();
                self.dispatch(name, a, caller)
            }
        }
    }

    fn dispatch(&self, name: &str, a: &Value, caller: Caller) -> Result<Value> {
        let consumer = caller.consumer;
        if caller.channel == Channel::Http && needs_human_id(name) {
            // an HTTP write needs the human id before anything else
            self.actor(name, &caller)?;
        }
        match name {
            "search" => {
                let x: SearchArgs = args(name, a)?;
                // before anything is opened: an empty query records nothing (K-C1.5)
                rootmarkindex::search::check_query(&x.query).code(Code::EmptyQuery)?;
                let consumer = need_consumer(name, consumer)?;
                rootmarkcore::check_trace_input(consumer, &rootmarkcore::hash32(""))
                    .code(Code::InvalidArguments)?;
                let k = x.k.unwrap_or(10);
                if !(1..=1000).contains(&k) {
                    bail_code!(Code::InvalidArguments, "search: k = {k} outside [1, 1000]");
                }
                let mode = Mode::parse(x.mode.as_deref().unwrap_or("hybrid"))
                    .code(Code::InvalidArguments)?;
                let mut v = open_vault(&self.vault)?;
                if let Some(at) = &x.at {
                    checkpoint_at(&v, at)?;
                }
                index_readable(&v)?;
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
                rootmarkcore::check_trace_input(consumer, &rootmarkcore::hash32(""))
                    .code(Code::InvalidArguments)?;
                let mut v = open_vault(&self.vault)?;
                let cp = match &x.at {
                    Some(a) => checkpoint_at(&v, a)?,
                    None => v.head_checkpoint()?,
                };
                // the lineage must be live in this document at the
                // checkpoint, checked before the trace is written
                let in_doc = match v.blocks_at(&x.doc_path, &cp.checkpoint_id) {
                    Ok(bs) => bs.iter().any(|b| b.lineage_id == x.lineage_id),
                    Err(_) => false,
                };
                if !in_doc {
                    bail_code!(
                        Code::NotFound,
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
                let actor = self.actor(name, &caller)?;
                let source = need_consumer(name, consumer)?;
                self.propose(x, source, &actor)
            }
            "promote" => {
                let x: PromoteArgs = args(name, a)?;
                non_empty(name, "lineage_id", &x.lineage_id)?;
                if x.reason.trim().is_empty() {
                    bail_code!(Code::InvalidArguments, "promote: reason must not be empty");
                }
                // the actor comes from the channel (K-D1.3): MCP → agent,
                // HTTP → the vault's human
                let actor = self.actor(name, &caller)?;
                let (policy, _) = self.policy()?;
                let hook = policy.promotion.hook();
                let mut v = open_vault(&self.vault)?;
                let from = v.layer_of(&x.lineage_id).code(Code::NotFound)?;
                let req = v
                    .request(
                        rootmarkcore::TransitionKind::Promote,
                        &x.lineage_id,
                        rootmarkcore::Layer::Canonical,
                        actor.clone(),
                        &x.reason,
                    )
                    .code(Code::InvalidArguments)?;
                match hook.decide(&req) {
                    rootmarkcore::Decision::Deny(why) => Ok(json!({
                        "decision": "denied",
                        "policy": hook.name(),
                        "policy_version": hook.version(),
                        "reason": why,
                        "lineage_id": x.lineage_id,
                        "layer": req.from,
                        "actor": actor,
                    })),
                    rootmarkcore::Decision::Allow => {
                        // an allowed request for a lineage that is not a
                        // live proposal is the caller's error (after the
                        // policy: an agent's request stays a denial)
                        // spec 2.0.0 §7: `unverified` is promoted like `proposal`
                        if !matches!(
                            from,
                            rootmarkcore::Layer::Proposal | rootmarkcore::Layer::Unverified
                        ) {
                            bail_code!(
                                Code::InvalidArguments,
                                "promote: lineage {} is {}; only a proposal (or unverified content) is promoted",
                                x.lineage_id,
                                from.as_str()
                            );
                        }
                        if v.is_inactive(&x.lineage_id).code(Code::Internal)? {
                            bail_code!(
                                Code::InvalidArguments,
                                "promote: lineage {} is inactive (pruned)",
                                x.lineage_id
                            );
                        }
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
                let mut v = open_vault(&self.vault)?;
                if let Some(n) = &x.name {
                    non_empty(name, "name", n)?;
                    if n != n.trim() || n.chars().any(char::is_control) {
                        bail_code!(
                            Code::InvalidArguments,
                            "checkpoint: name {n:?} is padded with whitespace or holds control characters"
                        );
                    }
                    if n.len() == 32 && n.bytes().all(|b| b.is_ascii_hexdigit()) {
                        bail_code!(
                            Code::InvalidArguments,
                            "checkpoint: name {n:?} has the form of a checkpoint id"
                        );
                    }
                    let cur = v.current_checkpoint_id();
                    if let Some(c) = v
                        .checkpoints()
                        .into_iter()
                        .find(|c| c.name.as_deref() == Some(n.as_str()) && c.checkpoint_id != cur)
                    {
                        bail_code!(
                            Code::Exists,
                            "checkpoint: name {n:?} is taken by {}",
                            c.checkpoint_id
                        );
                    }
                }
                let cur = v.current_checkpoint_id();
                if let Some(c) = v.checkpoints().into_iter().find(|c| c.checkpoint_id == cur) {
                    if c.name.is_some() && c.name != x.name {
                        bail_code!(
                            Code::Exists,
                            "checkpoint {cur} of this state already exists with name {:?}",
                            c.name.as_deref().unwrap_or_default()
                        );
                    }
                }
                Ok(serde_json::to_value(v.checkpoint(x.name.as_deref())?)?)
            }
            "history" => {
                let x: HistoryArgs = args(name, a)?;
                non_empty(name, "lineage_id", &x.lineage_id)?;
                let v = open_vault(&self.vault)?;
                // the CLI's `history` output (an unknown lineage is an error)
                let layer = v.layer_of(&x.lineage_id).ok();
                let events = v.history(&x.lineage_id).code(Code::NotFound)?;
                Ok(json!({"lineage_id": x.lineage_id, "layer": layer, "events": events}))
            }
            "blame" => {
                let x: BlameArgs = args(name, a)?;
                non_empty(name, "doc_path", &x.doc_path)?;
                let v = open_vault(&self.vault)?;
                match &x.at {
                    Some(at) => {
                        let cp = checkpoint_at(&v, at)?;
                        if !cp.docs.contains_key(&x.doc_path) {
                            bail_code!(
                                Code::NotFound,
                                "blame: {} has no snapshot at checkpoint {}",
                                x.doc_path,
                                cp.checkpoint_id
                            );
                        }
                    }
                    None => {
                        if v.doc_state(&x.doc_path)
                            .and_then(|s| s.head.as_ref())
                            .is_none()
                        {
                            bail_code!(Code::NotFound, "blame: no snapshot for {}", x.doc_path);
                        }
                    }
                }
                let blocks = v.blame(&x.doc_path, x.at.as_deref())?;
                Ok(json!({"doc_path": x.doc_path, "at": x.at, "blocks": blocks}))
            }
            "checkpoints" => {
                args::<NoArgs>(name, a)?;
                Ok(json!({"checkpoints": open_vault(&self.vault)?.checkpoints()}))
            }
            "attest_session" if caller.channel == Channel::Http => {
                args::<NoArgs>(name, a)?;
                self.attest_session()
            }
            "attest" if caller.channel == Channel::Http => {
                let x: AttestArgs = args(name, a)?;
                self.attest(x)
            }
            "trace" => {
                let x: TraceArgs = args(name, a)?;
                let v = open_vault(&self.vault)?;
                let mut traces = v.traces(x.since_seq.unwrap_or(0))?;
                if let Some(c) = &x.consumer {
                    non_empty(name, "consumer", c)?;
                    traces.retain(|t| &t.consumer == c);
                }
                Ok(json!({"traces": traces}))
            }
            _ => bail_code!(Code::NotFound, "unknown tool {name:?}"),
        }
    }

    /// `propose` (in the writer queue): checks, sync of external edits,
    /// the edit policy (K-D1.1), write, attributed agent ingest (K-D1.2),
    /// record. See the tool description. `source` is the provenance source
    /// (the consumer), `actor` the channel's actor.
    fn propose(&self, x: ProposeArgs, source: &str, actor: &Actor) -> Result<Value> {
        use crate::propose::{self, Mode as PMode};
        let mode =
            PMode::parse(x.mode.as_deref().unwrap_or("create")).code(Code::InvalidArguments)?;
        if x.reason.trim().is_empty() {
            bail_code!(Code::InvalidArguments, "propose: reason must not be empty");
        }
        match (mode, &x.heading) {
            (PMode::ReplaceSection, None) => {
                bail_code!(
                    Code::InvalidArguments,
                    "propose: replace_section needs `heading`"
                )
            }
            (PMode::Create | PMode::Append, Some(_)) => {
                bail_code!(
                    Code::InvalidArguments,
                    "propose: `heading` applies to replace_section only"
                )
            }
            _ => {}
        }
        let (policy, cfg) = self.policy()?;
        let size = x.content.len() as u64;
        if size > cfg.max_propose_bytes {
            bail_code!(
                Code::TooLarge,
                "propose: content is {size} bytes; the limit is [server] max_propose_bytes = {}",
                cfg.max_propose_bytes
            );
        }
        propose::check_doc_path(&x.doc_path)?;
        let path = crate::paths::doc_file(&self.vault, &x.doc_path).code(Code::PathOutsideVault)?;
        // on a case- or normalization-insensitive file system another
        // spelling of an existing name would write into a document the log
        // knows under another doc_path
        crate::paths::exact_on_disk(&self.vault, &x.doc_path).code(Code::Exists)?;
        let mut v = open_vault(&self.vault)?;
        // the index, when the vault has one (the server always does; the
        // CLI may run before `rootmark index build`)
        let e = if store::index_dir(&v).join("manifest.json").exists() {
            index_readable(&v)?;
            Some(self.embedder()?)
        } else {
            None
        };
        let mut s = match &e {
            Some(e) => Some(Store::open(&v, e.as_ref())?),
            None => None,
        };
        let m = self.matcher_for(&v);
        let exists = std::fs::symlink_metadata(&path).is_ok();
        if exists && !path.is_file() {
            bail_code!(
                Code::InvalidArguments,
                "propose: {} is not a file",
                x.doc_path
            );
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
            (PMode::Create, Some(_)) => bail_code!(
                Code::Exists,
                "propose: {} exists; propose never overwrites (use mode append or replace_section)",
                x.doc_path
            ),
            (PMode::Create, None) => {
                if v.doc_state(&x.doc_path).is_some() {
                    bail_code!(
                        Code::Exists,
                        "propose: {} is in the vault's log but its file is missing",
                        x.doc_path
                    );
                }
                propose::append("", &x.content)
            }
            (PMode::Append | PMode::ReplaceSection, None) => bail_code!(
                Code::NotFound,
                "propose: {} does not exist ({} needs an existing document)",
                x.doc_path,
                mode.as_str()
            ),
            (PMode::Append | PMode::ReplaceSection, Some(_))
                if v.doc_state(&x.doc_path).is_none() =>
            {
                bail_code!(
                    Code::NotFound,
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
            bail_code!(
                Code::InvalidArguments,
                "propose: the resulting document would be empty"
            );
        }
        // an edit made outside the server and not yet ingested is human
        // content (the watcher's rule); it must not be attributed to the agent
        // (spec 2.0.0 §7.1: an external change, attested or `unverified`)
        let synced = match &old {
            Some(o) => {
                let store = match (&mut s, &e) {
                    (Some(s), Some(e)) => Some((s, e.as_ref() as &dyn rootmarkindex::Embedder)),
                    _ => None,
                };
                self.ingest_external(
                    &mut v,
                    store,
                    &policy,
                    &x.doc_path,
                    o.as_bytes(),
                    "rootmark-serve-watch",
                )?
                .0
            }
            None => 0,
        };
        // the edit policy (K-D1.1) decides on what the ingest would do
        let prov = Provenance {
            author_kind: AuthorKind::Agent,
            source: source.to_string(),
        };
        let plan = v.plan_ingest(&x.doc_path, &new, prov, m, None, Some(actor))?;
        let section = match (mode, &old) {
            (PMode::ReplaceSection, Some(o)) => section_blocks(
                &v,
                &x.doc_path,
                o,
                x.heading.as_deref().expect("checked above"),
            )?,
            _ => Vec::new(),
        };
        let edit = policy.edit.hook();
        let req = EditRequest {
            doc_path: x.doc_path.clone(),
            mode: match mode {
                PMode::Create => EditMode::Create,
                PMode::Append => EditMode::Append,
                PMode::ReplaceSection => EditMode::ReplaceSection,
            },
            actor: actor.clone(),
            section,
            effect: v.edit_effect(&plan)?,
        };
        if let Decision::Deny(why) = edit.decide(&req) {
            // K-E0.4: an append to human blocks is accepted only as a section
            // of its own; without a leading heading the agent is told so
            if mode == PMode::Append && !propose::starts_with_heading(&x.content) {
                return Err(crate::error::coded_hint(
                    Code::PolicyViolation,
                    format!("propose: denied by edit policy {}: {why}", edit.name()),
                    propose::APPEND_HINT,
                ));
            }
            bail_code!(
                Code::Denied,
                "propose: denied by edit policy {}: {why}",
                edit.name()
            );
        }
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
        let committed = match (&mut s, &e) {
            (Some(s), Some(e)) => s
                .watch_plan(&mut v, e.as_ref(), plan)
                .map(|r| (r.snapshot_id, r.events, r.embedded)),
            _ => {
                let n = plan.events.len();
                v.commit_plan(plan).map(|r| {
                    (
                        r.snapshot.snapshot_id,
                        if r.events.is_empty() { 0 } else { n },
                        0,
                    )
                })
            }
        };
        let (snapshot_id, events, embedded) = match committed {
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
            "consumer": source,
            "actor": actor,
            "reason": x.reason,
            "snapshot_id": snapshot_id,
            "events": events,
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
                    snapshot_id,
                    rp.display()
                )
            })?;
        let blocks = v.blame(&x.doc_path, None)?;
        Ok(json!({
            "doc_path": x.doc_path,
            "mode": mode.as_str(),
            "bytes": new.len(),
            "synced_external_events": synced,
            "edit_policy": edit.name(),
            "actor": actor,
            "snapshot_id": snapshot_id,
            "events": events,
            "chunks_embedded": embedded,
            "blocks": blocks,
            "events_hash": v.events_hash()?,
        }))
    }

    fn status(&self) -> Result<Value> {
        let v = open_vault(&self.vault)?;
        let (policy, cfg) = self.policy()?;
        // the index never fails `status` (K-E1.2): its state is a field
        let ix = index_state(&v);
        Ok(json!({
            "vault": self.vault.display().to_string(),
            "spec_version": SPEC_VERSION,
            "mdk_version": ROOTMARK_VERSION,
            "level_claimed": LEVEL_CLAIMED,
            "embedder": ix.embedder,
            "index": {
                "state": ix.state,
                "format": ix.format,
                "expected_format": store::FORMAT,
                "auto_rebuild": cfg.auto_rebuild,
            },
            "last_checkpoint": v.checkpoints().last(),
            "policy": policy.to_json(),
            "server_policy": cfg.to_json(),
            // spec 2.0.0 §7.1 (P2-0): the policy in force and the key stage
            "attestation": {
                "unattested": policy.attestation.unattested(),
                "window_s": policy.attestation.window_s(),
                "stage": 1,
                "pending": self.attesting.pending()?,
                // P2-0.1 T1: link updates attested since the server started
                "link_updates": self.attesting.link_updates(),
            },
        }))
    }
}

/// The vault's index as `status` reports it (K-E1.2), from `manifest.json`
/// alone: `ok` (this format and lexical record), `stale` (another format or
/// lexical record: an older `rootmark`'s index), `missing` (no manifest),
/// `damaged` (the manifest cannot be read or decoded; `damage` says why).
/// `stale` and `missing` are `index_stale` (K-E0.3).
struct IndexState {
    state: &'static str,
    /// `format` of the manifest on disk
    format: Option<String>,
    /// `embedder_id` of the manifest on disk (not of a damaged one)
    embedder: Option<String>,
    damage: Option<String>,
}

fn index_state(v: &Vault) -> IndexState {
    let p = store::index_dir(v).join("manifest.json");
    let mut ix = IndexState {
        state: "missing",
        format: None,
        embedder: None,
        damage: None,
    };
    if !p.is_file() {
        return ix;
    }
    let raw = std::fs::read(&p)
        .with_context(|| format!("read {}", p.display()))
        .and_then(|b| {
            serde_json::from_slice::<Value>(&b)
                .with_context(|| format!("{} is not JSON", p.display()))
        });
    if let Ok(raw) = &raw {
        ix.format = raw["format"].as_str().map(str::to_string);
    }
    match raw.and_then(|raw| store::staleness(v).map(|s| (raw, s))) {
        Ok((raw, s)) => {
            ix.state = if s.is_some() { "stale" } else { "ok" };
            ix.embedder = raw["embedder_id"].as_str().map(str::to_string);
        }
        Err(e) => {
            ix.state = "damaged";
            ix.damage = Some(format!("{e:#}"));
        }
    }
    ix
}

/// A call that needs the index refuses a damaged manifest as `index_stale`
/// (K-E1.2, GOAL P1-E.1 E1-2): only a rebuild (`rootmark index build`) recovers
/// it. Any other damage of the index files stays `internal`.
fn index_readable(v: &Vault) -> Result<()> {
    let ix = index_state(v);
    if let Some(why) = ix.damage {
        bail_code!(
            Code::IndexStale,
            "index damaged ({why}); rebuild it with `rootmark index build`"
        );
    }
    Ok(())
}

/// The blocks of the section `replace_section` would replace in `old` (the
/// document's head text, after any external edit was synced): the heading
/// and its body, `(lineage_id, layer)`. The text's blocks must be the head
/// snapshot's, one for one.
fn section_blocks(
    v: &Vault,
    doc: &str,
    old: &str,
    heading: &str,
) -> Result<Vec<(String, rootmarkcore::Layer)>> {
    let (at, end) = crate::propose::section_lines(old, heading)?;
    let (drafts, _) = rootmarkcore::parse_document(old, doc)?;
    let head = v
        .snapshot(doc)?
        .with_context(|| format!("{doc}: no head snapshot"))?;
    let blocks = v.snapshot_blocks(&head)?;
    if drafts.len() != blocks.len()
        || drafts
            .iter()
            .zip(&blocks)
            .any(|(d, b)| d.version_id != b.version_id)
    {
        bail!("propose: {doc} on disk differs from its head snapshot after the sync");
    }
    Ok(drafts
        .iter()
        .zip(&blocks)
        .filter(|(d, _)| {
            let line = d.line_start as usize - 1;
            line >= at && line < end
        })
        .map(|(_, b)| (b.lineage_id.clone(), b.layer))
        .collect())
}

/// An external ingest's decision, finished once its plan is committed:
/// an unattested ingest is remembered for a late attestation (spec 2.0.0
/// §7.1).
struct External {
    doc: String,
    hash: String,
    ingest: Ingest,
    /// when the change was observed (the decision)
    observed_ms: u64,
    /// the lineages its events name, with their new version and the layer
    /// an attested ingest gives each
    named: Vec<rootmarkcore::attest::Raise>,
}

impl External {
    /// A partial match: the attested bytes' hash (the rest of the observed
    /// change is ingested next, unattested).
    fn partial_hash(&self) -> Option<String> {
        match &self.ingest {
            Ingest::Partial(..) => Some(self.hash.clone()),
            _ => None,
        }
    }

    fn nothing(doc: &str, hash: &str) -> External {
        External {
            doc: doc.to_string(),
            hash: hash.to_string(),
            ingest: Ingest::Canonical,
            observed_ms: 0,
            named: Vec::new(),
        }
    }

    /// After the commit: the lineages the unattested ingest left
    /// `unverified` wait for a late attestation.
    fn done(self, k: &Kernel, v: &Vault, policy: &Policy) -> Result<()> {
        if self.ingest != Ingest::Unverified {
            return Ok(());
        }
        let Some(st) = v.doc_state(&self.doc) else {
            return Ok(());
        };
        let mut seen = std::collections::HashSet::new();
        let lineages: Vec<rootmarkcore::attest::Raise> = self
            .named
            .into_iter()
            .filter(|r| st.layer.get(&r.lineage_id) == Some(&rootmarkcore::Layer::Unverified))
            .filter(|r| {
                matches!(
                    r.to,
                    rootmarkcore::Layer::Proposal | rootmarkcore::Layer::Canonical
                )
            })
            .filter(|r| seen.insert(r.lineage_id.clone()))
            .collect();
        k.attesting
            .unattested(policy, &self.doc, &self.hash, self.observed_ms, lineages)
    }
}
