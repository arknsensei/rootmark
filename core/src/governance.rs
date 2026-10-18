//! Governance (spec §7, §8): layer transitions requested through a policy
//! hook, checkpoints and consumption traces.
//!
//! The kernel never decides a transition: `Vault::promote`, `Vault::demote`
//! and `Vault::prune_inactive` take a `&dyn PromotionPolicy` (there is no
//! overload without one) and write an event only when it returns `Allow`.
//! A promotion without a hook does not compile (C2-03):
//!
//! ```compile_fail
//! # use rootmarkcore::{Actor, ActorKind, Layer, TransitionKind, Vault};
//! # fn f(v: &mut Vault, l: &str) -> anyhow::Result<()> {
//! let actor = Actor::new(ActorKind::Human, "alice")?;
//! let req = v.request(TransitionKind::Promote, l, Layer::Canonical, actor, "ok")?;
//! v.promote(&req)?; // error: a PromotionPolicy argument is required
//! # Ok(()) }
//! ```
//!
//! The same call with a hook compiles:
//!
//! ```
//! # use rootmarkcore::{Actor, ActorKind, HumanOnly, Layer, TransitionKind, Vault};
//! # fn f(v: &mut Vault, l: &str) -> anyhow::Result<()> {
//! let actor = Actor::new(ActorKind::Human, "alice")?;
//! let req = v.request(TransitionKind::Promote, l, Layer::Canonical, actor, "ok")?;
//! v.promote(&req, &HumanOnly)?;
//! # Ok(()) }
//! ```

use crate::model::{hash32, Layer};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Who requests a transition (spec §7: "human id or policy name").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    Human,
    Agent,
    Policy,
    /// the writer of an external change no human-channel client attested
    /// (spec 2.0.0 §7.1); only on the events of such an ingest
    #[serde(rename = "unattested_fs")]
    UnattestedFs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub kind: ActorKind,
    pub id: String,
}

impl Actor {
    pub fn new(kind: ActorKind, id: &str) -> Result<Actor> {
        if id.trim().is_empty() {
            bail!("actor id is empty");
        }
        Ok(Actor {
            kind,
            id: id.to_string(),
        })
    }

    /// `"<kind>:<id>"`, kind ∈ {human, agent, policy}.
    pub fn parse(s: &str) -> Result<Actor> {
        let Some((kind, id)) = s.split_once(':') else {
            bail!("actor {s:?}: expected <human|agent|policy>:<id>");
        };
        let kind = match kind {
            "human" => ActorKind::Human,
            "agent" => ActorKind::Agent,
            "policy" => ActorKind::Policy,
            _ => bail!("actor {s:?}: unknown kind {kind:?} (human|agent|policy)"),
        };
        Actor::new(kind, id)
    }
}

/// What a request asks for. `Prune` keeps the layer and marks the lineage
/// inactive (spec §7: pruning never deletes objects).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransitionKind {
    Promote,
    Demote,
    Prune,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionRequest {
    pub kind: TransitionKind,
    pub lineage: String,
    pub from: Layer,
    pub to: Layer,
    pub actor: Actor,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(String),
}

/// The promotion hook (spec §7, ARCHITECTURE §4.4 `on_promotion_request`).
pub trait PromotionPolicy {
    /// Recorded as `meta.policy` of the transition event.
    fn name(&self) -> &str;
    /// Recorded as `meta.policy_version`; never empty.
    fn version(&self) -> &str;
    fn decide(&self, req: &PromotionRequest) -> Decision;
}

/// Refuses every transition (the kernel's default policy).
pub struct Deny;

impl PromotionPolicy for Deny {
    fn name(&self) -> &str {
        "deny"
    }
    fn version(&self) -> &str {
        "1"
    }
    fn decide(&self, _req: &PromotionRequest) -> Decision {
        Decision::Deny("policy deny refuses every layer transition".into())
    }
}

/// Allows a transition only when the actor is a human (K12, K16).
pub struct HumanOnly;

impl PromotionPolicy for HumanOnly {
    fn name(&self) -> &str {
        "human_only"
    }
    fn version(&self) -> &str {
        "1"
    }
    fn decide(&self, req: &PromotionRequest) -> Decision {
        if req.actor.kind == ActorKind::Human {
            Decision::Allow
        } else {
            Decision::Deny(format!(
                "policy human_only: actor {:?} is not human",
                req.actor.id
            ))
        }
    }
}

/// How an agent's text enters a document (`propose`; K-D1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditMode {
    Create,
    Append,
    ReplaceSection,
}

impl EditMode {
    pub fn as_str(self) -> &'static str {
        match self {
            EditMode::Create => "create",
            EditMode::Append => "append",
            EditMode::ReplaceSection => "replace_section",
        }
    }
}

/// A lineage live before an edit that the edit's events would change
/// (update, move, split or merge source, delete, layer drop).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchedLineage {
    pub lineage_id: String,
    /// its layer before the edit
    pub layer: Layer,
    /// the event kinds that name it, in log order
    pub kinds: Vec<crate::events::EventKind>,
}

/// What an ingest would do to a document's existing content, computed
/// before anything is written (`Vault::edit_effect`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditEffect {
    pub touched: Vec<TouchedLineage>,
    /// structural chunks (spec §6) made only of `canonical` blocks that the
    /// edit would remove or add, by `chunk_id`: such a chunk is a unit of
    /// default retrieval (spec §9), so either changes canonical results
    pub canonical_chunks_removed: Vec<String>,
    pub canonical_chunks_added: Vec<String>,
    /// an existing document's YAML front matter would change
    pub frontmatter_changed: bool,
}

/// An agent's request to change a document (K-D1.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRequest {
    pub doc_path: String,
    pub mode: EditMode,
    pub actor: Actor,
    /// `replace_section` only: the blocks of the target section (its
    /// heading and body) before the edit, `(lineage_id, layer)`
    pub section: Vec<(String, Layer)>,
    pub effect: EditEffect,
}

/// The edit hook (K-D1.1): decides whether an agent's write may change a
/// document. The kernel computes the effect; the policy decides.
pub trait EditPolicy {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn decide(&self, req: &EditRequest) -> Decision;
}

/// The default (K-D1.1): an agent's write may add content and change its
/// own proposals, never a `canonical` (or `derived`) block, nor a canonical
/// retrieval unit. `replace_section` is allowed only when every block of
/// the section is a `proposal`.
pub struct AppendOnly;

impl EditPolicy for AppendOnly {
    fn name(&self) -> &str {
        "append_only"
    }
    fn version(&self) -> &str {
        "1"
    }
    fn decide(&self, req: &EditRequest) -> Decision {
        let protected = |l: Layer| l != Layer::Proposal;
        if req.mode == EditMode::ReplaceSection {
            let held: Vec<String> = req
                .section
                .iter()
                .filter(|(_, l)| protected(*l))
                .map(|(id, l)| format!("{id} ({})", l.as_str()))
                .collect();
            if !held.is_empty() {
                return Decision::Deny(format!(
                    "policy append_only: replace_section only replaces a section made of proposal blocks; this section holds {}",
                    held.join(", ")
                ));
            }
        }
        let touched: Vec<String> = req
            .effect
            .touched
            .iter()
            .filter(|t| protected(t.layer))
            .map(|t| format!("{} ({})", t.lineage_id, t.layer.as_str()))
            .collect();
        if !touched.is_empty() {
            return Decision::Deny(format!(
                "policy append_only: the edit would change or remove blocks that are not proposals: {}",
                touched.join(", ")
            ));
        }
        let e = &req.effect;
        if e.frontmatter_changed {
            return Decision::Deny(
                "policy append_only: the edit would change the document's front matter".into(),
            );
        }
        if !e.canonical_chunks_removed.is_empty() || !e.canonical_chunks_added.is_empty() {
            return Decision::Deny(format!(
                "policy append_only: the edit would change canonical retrieval units (chunks removed: {}; added: {}); text added to a section of canonical blocks joins their chunk — start it with a heading of its own",
                e.canonical_chunks_removed.len(),
                e.canonical_chunks_added.len()
            ));
        }
        Decision::Allow
    }
}

/// Allows every edit (K-D1.1; only when `policy.toml` says so). Deletions
/// and changes still carry their actor (K-D1.2).
pub struct Unrestricted;

impl EditPolicy for Unrestricted {
    fn name(&self) -> &str {
        "unrestricted"
    }
    fn version(&self) -> &str {
        "1"
    }
    fn decide(&self, _req: &EditRequest) -> Decision {
        Decision::Allow
    }
}

/// Immutable vault-wide map `doc_path → snapshot_id` at an event-log offset
/// (spec §1, §8). `checkpoint_id` follows spec §3 (`checkpoint_id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub checkpoint_id: String,
    pub name: Option<String>,
    pub event_seq: u64,
    pub docs: BTreeMap<String, String>,
}

/// `checkpoint_id = H(sorted "doc_path\tsnapshot_id" lines + "\n" + event_seq)`
/// (spec §3): the lines joined by `"\n"`, then `"\n"` and the decimal offset.
pub fn checkpoint_id(docs: &BTreeMap<String, String>, event_seq: u64) -> String {
    let mut lines: Vec<String> = docs.iter().map(|(d, s)| format!("{d}\t{s}")).collect();
    lines.sort();
    hash32(&format!("{}\n{event_seq}", lines.join("\n")))
}

/// One block version a consumer received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEntry {
    pub doc_path: String,
    pub lineage_id: String,
    pub version_id: String,
}

/// Consumption trace line (spec §8): which block versions a consumer
/// received for a query at a checkpoint. `at_seq` is the 0-based position of
/// the line in `traces.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub checkpoint_id: String,
    pub consumer: String,
    pub query_hash: String,
    pub returned: Vec<TraceEntry>,
    pub at_seq: u64,
}

/// `trace_id = H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" +
/// at_seq)` (spec 1.0.1 §8, K-B1.2), `at_seq` in decimal. The inputs are
/// valid only as `check_trace_input` accepts them.
pub fn trace_id(checkpoint_id: &str, consumer: &str, query_hash: &str, at_seq: u64) -> String {
    hash32(&format!(
        "{checkpoint_id}\n{consumer}\n{query_hash}\n{at_seq}"
    ))
}

/// Spec 1.0.1 §8 (K-B1.2): `query_hash` is exactly 32 lowercase hex
/// characters (the form of every §3 identifier) and `consumer` holds no
/// `"\n"`, so the separated fields of `trace_id` cannot shift.
pub fn check_trace_input(consumer: &str, query_hash: &str) -> anyhow::Result<()> {
    if consumer.contains('\n') {
        anyhow::bail!("trace consumer {consumer:?} contains a newline");
    }
    if query_hash.len() != 32
        || !query_hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        anyhow::bail!("trace query_hash {query_hash:?} is not 32 lowercase hex characters");
    }
    Ok(())
}
