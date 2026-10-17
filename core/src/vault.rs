//! Vault: object store + event log + per-document heads (SPEC §2–4, §7).

use crate::events::{append_events, read_events, Event, EventKind};
use crate::governance::{checkpoint_id, Actor, ActorKind, Checkpoint, EditEffect, TouchedLineage};
use crate::lineage::{match_blocks, MatchParams, MatcherVersion, PrevBlock};
use crate::model::{hash32, AuthorKind, Block, Layer, LayerSource, Provenance};
use crate::parse::heading_paths;
use crate::parse::parse_document;
use crate::store::{self, BlockObject, SnapshotObject};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub doc_path: String,
    pub blocks: Vec<(String, String)>,
    pub frontmatter: Option<String>,
    pub parent_snapshot: Option<String>,
}

/// `snapshot_id = blake3(version_ids joined by "\n")`.
pub fn snapshot_id<'a, I: IntoIterator<Item = &'a str>>(vids: I) -> String {
    hash32(&vids.into_iter().collect::<Vec<_>>().join("\n"))
}

/// Replay state of one document.
#[derive(Debug, Clone, Default)]
pub struct DocState {
    /// live lineage → version
    pub live: BTreeMap<String, String>,
    /// lineage → parents of its current version
    pub parents: HashMap<String, Vec<String>>,
    /// lineage → provenance of its current version
    pub provenance: HashMap<String, Provenance>,
    /// every lineage id ever assigned in this document
    pub used: HashSet<String>,
    pub head: Option<Snapshot>,
    /// live lineage → layer (spec §7)
    pub layer: HashMap<String, Layer>,
    /// live lineages pruned by policy (spec §7: marked inactive, never deleted)
    pub inactive: HashSet<String>,
    /// live lineage → seq of the event that created it
    pub created_seq: HashMap<String, u64>,
    /// live lineage → seq of the last event that named it
    pub last_seq: HashMap<String, u64>,
    /// inherited lineages whose layer an ingest lowered: the `LayerChanged`
    /// events that must follow before the next `SnapshotCommitted`, in order
    pub pending: Vec<(String, Layer)>,
    /// live lineages whose layer replay derived without an event (K-B1.1)
    pub derived: HashSet<String>,
    /// the log carries layer information at this point (`carries_layer`):
    /// a drop the layer rule implies must then be a `LayerChanged` event;
    /// before it (a v3-final-format prefix) replay derives the drop (K-B1.1)
    pub layered: bool,
}

impl DocState {
    fn forget(&mut self, l: &str) {
        self.parents.remove(l);
        self.provenance.remove(l);
        self.layer.remove(l);
        self.inactive.remove(l);
        self.created_seq.remove(l);
        self.last_seq.remove(l);
        self.derived.remove(l);
    }

    /// Where the layer of live lineage `l` comes from (`blame`).
    pub fn layer_source(&self, l: &str) -> LayerSource {
        if self.derived.contains(l) {
            LayerSource::Derived
        } else {
            LayerSource::Event
        }
    }

    fn layer_of(&self, l: &str, ev: &Event) -> Result<Layer> {
        self.layer
            .get(l)
            .copied()
            .with_context(|| format!("event seq {}: lineage {l} has no layer", ev.seq))
    }
}

fn meta_str(ev: &Event, key: &str) -> Option<String> {
    ev.meta
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn meta_list(ev: &Event, key: &str) -> Vec<String> {
    ev.meta
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn one<'a>(v: &'a [String], what: &str, ev: &Event) -> Result<&'a str> {
    if v.len() != 1 {
        bail!("event seq {}: {:?} expects one {what}", ev.seq, ev.kind);
    }
    Ok(&v[0])
}

fn meta_layer(ev: &Event, key: &str) -> Result<Layer> {
    let s = meta_str(ev, key)
        .with_context(|| format!("event seq {}: {:?} without meta.{key}", ev.seq, ev.kind))?;
    Layer::parse(&s).with_context(|| format!("event seq {}: meta.{key}", ev.seq))
}

/// Whether an event carries layer information (K-B1.1): a kind the v3-final
/// core never writes (`LayerChanged`, `Promoted`, `Demoted`,
/// `CheckpointCreated`) or a block event with a declared `meta.layer`. A log
/// is in v3-final format up to its first such event; from it on (vault-wide)
/// every layer drop must be logged.
pub fn carries_layer(ev: &Event) -> bool {
    matches!(
        ev.kind,
        EventKind::LayerChanged
            | EventKind::Promoted
            | EventKind::Demoted
            | EventKind::CheckpointCreated
    ) || ev.meta.get("layer").is_some()
}

/// Layer a block event's author gives its content: `meta.layer` when the
/// ingest declared one (synthetic content only, spec §7 "as declared"),
/// else `Layer::default_for(author_kind)`.
fn author_layer(ev: &Event, prov: &Provenance) -> Result<Layer> {
    match ev.meta.get("layer") {
        None => Ok(Layer::default_for(prov.author_kind)),
        Some(_) => {
            if prov.author_kind != AuthorKind::Synthetic {
                bail!(
                    "event seq {}: meta.layer declared for {:?} content (synthetic only)",
                    ev.seq,
                    prov.author_kind
                );
            }
            meta_layer(ev, "layer")
        }
    }
}

/// The governance fields every `Promoted` / `Demoted` event must carry
/// (spec §7, C2-02): `actor`, `reason`, `policy`, `policy_version`, `from`, `to`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionMeta {
    pub actor: Actor,
    pub reason: String,
    pub policy: String,
    pub policy_version: String,
    pub from: Layer,
    pub to: Layer,
    pub prune: bool,
}

/// Built-in policies whose decision replay can re-check from the event
/// alone: `deny` never allows, `human_only` only for a human actor.
fn check_builtin_policy(ev: &Event, m: &TransitionMeta) -> Result<()> {
    match m.policy.as_str() {
        "deny" => bail!(
            "event seq {}: policy deny cannot have allowed {:?}",
            ev.seq,
            ev.kind
        ),
        "human_only" if m.actor.kind != ActorKind::Human => bail!(
            "event seq {}: policy human_only cannot have allowed actor {:?}",
            ev.seq,
            m.actor.id
        ),
        _ => Ok(()),
    }
}

pub fn transition_meta(ev: &Event) -> Result<TransitionMeta> {
    let req = |k: &str| -> Result<String> {
        let s = meta_str(ev, k)
            .with_context(|| format!("event seq {}: {:?} requires meta.{k}", ev.seq, ev.kind))?;
        if s.trim().is_empty() {
            bail!("event seq {}: {:?} meta.{k} is empty", ev.seq, ev.kind);
        }
        Ok(s)
    };
    let actor: Actor = ev
        .meta
        .get("actor")
        .with_context(|| format!("event seq {}: {:?} requires meta.actor", ev.seq, ev.kind))
        .and_then(|a| {
            serde_json::from_value(a.clone())
                .with_context(|| format!("event seq {}: malformed meta.actor", ev.seq))
        })?;
    if actor.id.trim().is_empty() {
        bail!("event seq {}: {:?} meta.actor.id is empty", ev.seq, ev.kind);
    }
    let prune = match ev.meta.get("prune") {
        None => false,
        Some(v) => v
            .as_bool()
            .with_context(|| format!("event seq {}: meta.prune is not a boolean", ev.seq))?,
    };
    Ok(TransitionMeta {
        actor,
        reason: req("reason")?,
        policy: req("policy")?,
        policy_version: req("policy_version")?,
        from: meta_layer(ev, "from")?,
        to: meta_layer(ev, "to")?,
        prune,
    })
}

/// Applies one event to a document state. `SnapshotCommitted` is verified
/// against the state built from the preceding block events.
///
/// K-D1.2: `meta.actor` (`{kind, id}`, id not empty) and `meta.source` (a
/// string) of a block event, when present, must be well formed; absent is
/// the reading of logs written before 1.0.3 or by the unattributed API.
fn check_event_actor(ev: &Event) -> Result<()> {
    // `Promoted`/`Demoted` carry their own `meta.actor` (`transition_meta`)
    if matches!(ev.kind, EventKind::Promoted | EventKind::Demoted) {
        return Ok(());
    }
    let (a, s) = (ev.meta.get("actor"), ev.meta.get("source"));
    if !carries_actor(ev.kind) && (a.is_some() || s.is_some()) {
        bail!(
            "event seq {}: {:?} carries no meta.actor/meta.source (only change events do)",
            ev.seq,
            ev.kind
        );
    }
    if a.is_some() != s.is_some() {
        bail!(
            "event seq {}: meta.actor and meta.source go together",
            ev.seq
        );
    }
    if let Some(a) = ev.meta.get("actor") {
        let a: Actor = serde_json::from_value(a.clone())
            .with_context(|| format!("event seq {}: malformed meta.actor", ev.seq))?;
        if a.id.trim().is_empty() {
            bail!("event seq {}: meta.actor.id is empty", ev.seq);
        }
    }
    if let Some(s) = ev.meta.get("source") {
        if !s.is_string() {
            bail!("event seq {}: meta.source is not a string", ev.seq);
        }
    }
    Ok(())
}

/// The block events that carry the ingest's actor (K-D1.2).
fn carries_actor(k: EventKind) -> bool {
    matches!(
        k,
        EventKind::BlockUpdated
            | EventKind::BlockMoved
            | EventKind::BlockSplit
            | EventKind::BlockMerged
            | EventKind::BlockDeleted
    )
}

/// A checkpoint name (spec §8): not empty, not padded, and not of the form
/// of a checkpoint id (ids and names share `at`).
fn check_checkpoint_name(n: &str) -> Result<()> {
    if n.trim().is_empty() || n != n.trim() || n.chars().any(char::is_control) {
        bail!("checkpoint name {n:?}: empty, padded with whitespace or holding control characters");
    }
    if n.len() == 32 && n.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("checkpoint name {n:?} has the form of a checkpoint id");
    }
    Ok(())
}

/// Layers (spec §7): a new lineage gets the lowest of its author's layer and
/// its parents' layers; an inherited lineage (update, dominant split piece,
/// merge target) keeps its layer, and when the same rule gives a lower one
/// the ingest must record it with a `LayerChanged` event before its
/// `SnapshotCommitted`. Ingest never raises a layer; only `Promoted` does.
/// In a v3-final-format prefix of the log (`!st.layered`, K-B1.1) no such
/// event exists: `SnapshotCommitted` applies the pending drops itself and
/// marks those lineages `LayerSource::Derived`.
pub fn apply_event(st: &mut DocState, ev: &Event) -> Result<()> {
    // provenance of the version a block event creates (spec §4.3). Absent:
    // the v3-final reading (`Provenance::default()`, human) — logs built
    // through the low-level `commit` API had none (ASSUMPTIONS B2-1).
    // Present but malformed: an error, never a default.
    let prov = || -> Result<Provenance> {
        match ev.meta.get("provenance") {
            None => Ok(Provenance::default()),
            Some(p) => serde_json::from_value(p.clone())
                .with_context(|| format!("event seq {}: malformed meta.provenance", ev.seq)),
        }
    };
    check_event_actor(ev)?;
    let lower = |st: &mut DocState, l: &str, target: Layer| -> Result<()> {
        let cur = st.layer_of(l, ev)?;
        if target < cur {
            st.pending.retain(|(p, _)| p != l);
            st.pending.push((l.to_string(), target));
        }
        Ok(())
    };
    match ev.kind {
        EventKind::BlockCreated => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let v = one(&ev.new_version_ids, "new version", ev)?;
            if st.live.contains_key(l) {
                bail!("event seq {}: lineage {l} already live", ev.seq);
            }
            let p = prov()?;
            st.layer.insert(l.into(), author_layer(ev, &p)?);
            st.live.insert(l.into(), v.into());
            st.parents.insert(l.into(), Vec::new());
            st.provenance.insert(l.into(), p);
            st.used.insert(l.into());
            st.created_seq.insert(l.into(), ev.seq);
            st.last_seq.insert(l.into(), ev.seq);
        }
        EventKind::BlockUpdated => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let old = one(&ev.old_version_ids, "old version", ev)?;
            let v = one(&ev.new_version_ids, "new version", ev)?;
            if st.live.get(l).map(String::as_str) != Some(old) {
                bail!(
                    "event seq {}: update of {l} does not match live version",
                    ev.seq
                );
            }
            let p = prov()?;
            lower(st, l, author_layer(ev, &p)?)?;
            st.live.insert(l.into(), v.into());
            st.parents.insert(l.into(), vec![old.into()]);
            st.provenance.insert(l.into(), p);
            st.last_seq.insert(l.into(), ev.seq);
        }
        EventKind::BlockMoved => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            if !st.live.contains_key(l) {
                bail!("event seq {}: move of unknown lineage {l}", ev.seq);
            }
            st.last_seq.insert(l.into(), ev.seq);
        }
        EventKind::BlockSplit | EventKind::BlockMerged => {
            let from = meta_list(ev, "from_lineages");
            if from.len() != ev.old_version_ids.len()
                || ev.lineage_ids.len() != ev.new_version_ids.len()
            {
                bail!("event seq {}: malformed {:?}", ev.seq, ev.kind);
            }
            let p = prov()?;
            // lowest layer among the sources and the author
            let mut target = author_layer(ev, &p)?;
            let mut kept: HashMap<String, (Layer, bool, u64, bool)> = HashMap::new();
            for (l, old) in from.iter().zip(&ev.old_version_ids) {
                if st.live.remove(l).as_deref() != Some(old.as_str()) {
                    bail!("event seq {}: source lineage {l} not live at {old}", ev.seq);
                }
                let layer = st.layer_of(l, ev)?;
                target = target.min(layer);
                let created = st.created_seq.get(l).copied().unwrap_or(ev.seq);
                kept.insert(
                    l.clone(),
                    (
                        layer,
                        st.inactive.contains(l),
                        created,
                        st.derived.contains(l),
                    ),
                );
                st.pending.retain(|(q, _)| q != l);
                st.forget(l);
            }
            for (l, v) in ev.lineage_ids.iter().zip(&ev.new_version_ids) {
                if st.live.contains_key(l) {
                    bail!("event seq {}: lineage {l} already live", ev.seq);
                }
                st.live.insert(l.clone(), v.clone());
                st.parents.insert(l.clone(), ev.old_version_ids.clone());
                st.provenance.insert(l.clone(), p.clone());
                st.used.insert(l.clone());
                st.last_seq.insert(l.clone(), ev.seq);
                match kept.get(l) {
                    // inherited lineage (dominant piece / merge target)
                    Some(&(layer, inactive, created, derived)) => {
                        st.layer.insert(l.clone(), layer);
                        if inactive {
                            st.inactive.insert(l.clone());
                        }
                        if derived {
                            st.derived.insert(l.clone());
                        }
                        st.created_seq.insert(l.clone(), created);
                        lower(st, l, target)?;
                    }
                    None => {
                        st.layer.insert(l.clone(), target);
                        st.created_seq.insert(l.clone(), ev.seq);
                    }
                }
            }
        }
        EventKind::BlockDeleted => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let old = one(&ev.old_version_ids, "old version", ev)?;
            if st.live.remove(l).as_deref() != Some(old) {
                bail!(
                    "event seq {}: delete of {l} does not match live version",
                    ev.seq
                );
            }
            st.pending.retain(|(q, _)| q != l);
            st.forget(l);
        }
        EventKind::LayerChanged => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let (from, to) = (meta_layer(ev, "from")?, meta_layer(ev, "to")?);
            let cur = st.layer_of(l, ev)?;
            let due = st.pending.iter().position(|(q, _)| q == l);
            match due {
                // in the order the block events implied them
                Some(0) if cur == from && st.pending[0].1 == to => {
                    st.pending.remove(0);
                }
                _ => bail!(
                    "event seq {}: LayerChanged {l} {}→{} does not match the replayed layer rule",
                    ev.seq,
                    from.as_str(),
                    to.as_str()
                ),
            }
            st.layer.insert(l.into(), to);
            st.derived.remove(l);
            st.last_seq.insert(l.into(), ev.seq);
        }
        EventKind::Promoted | EventKind::Demoted => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let v = one(&ev.new_version_ids, "version", ev)?;
            if ev.old_version_ids != ev.new_version_ids {
                bail!("event seq {}: {:?} changes no version", ev.seq, ev.kind);
            }
            if st.live.get(l).map(String::as_str) != Some(v) {
                bail!(
                    "event seq {}: {:?} of {l} at a version that is not live",
                    ev.seq,
                    ev.kind
                );
            }
            if !st.pending.is_empty() {
                bail!(
                    "event seq {}: {:?} inside an unfinished ingest",
                    ev.seq,
                    ev.kind
                );
            }
            let m = transition_meta(ev)?;
            check_builtin_policy(ev, &m)?;
            let cur = st.layer_of(l, ev)?;
            check_transition(ev.kind, &m, cur, st.inactive.contains(l))
                .with_context(|| format!("event seq {}: {l}", ev.seq))?;
            st.layer.insert(l.into(), m.to);
            st.derived.remove(l);
            if m.prune {
                st.inactive.insert(l.into());
            }
            st.last_seq.insert(l.into(), ev.seq);
        }
        EventKind::CheckpointCreated | EventKind::CheckpointNamed => {
            bail!(
                "event seq {}: {:?} is not a document event",
                ev.seq,
                ev.kind
            );
        }
        EventKind::SnapshotCommitted => {
            let sid = ev
                .snapshot_id
                .clone()
                .context("SnapshotCommitted without snapshot_id")?;
            if !st.pending.is_empty() && !st.layered {
                // v3-final-format log: the drops are derived, not logged
                for (l, to) in std::mem::take(&mut st.pending) {
                    st.layer.insert(l.clone(), to);
                    st.derived.insert(l);
                }
            }
            if !st.pending.is_empty() {
                bail!(
                    "event seq {}: snapshot committed before LayerChanged of {}",
                    ev.seq,
                    st.pending
                        .iter()
                        .map(|(l, _)| l.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if ev.lineage_ids.len() != ev.new_version_ids.len() {
                bail!(
                    "event seq {}: snapshot lineage/version length mismatch",
                    ev.seq
                );
            }
            let committed: BTreeMap<String, String> = ev
                .lineage_ids
                .iter()
                .cloned()
                .zip(ev.new_version_ids.iter().cloned())
                .collect();
            if committed.len() != ev.lineage_ids.len() || committed != st.live {
                bail!(
                    "event seq {}: snapshot does not match replayed block events",
                    ev.seq
                );
            }
            let computed = snapshot_id(ev.new_version_ids.iter().map(String::as_str));
            if computed != sid {
                bail!(
                    "event seq {}: snapshot_id {sid} != computed {computed}",
                    ev.seq
                );
            }
            let parent = st.head.as_ref().map(|s| s.snapshot_id.clone());
            if meta_str(ev, "parent_snapshot") != parent {
                bail!("event seq {}: parent_snapshot mismatch", ev.seq);
            }
            st.head = Some(Snapshot {
                snapshot_id: sid,
                doc_path: ev.doc_path.clone(),
                blocks: committed_pairs(ev),
                frontmatter: meta_str(ev, "frontmatter"),
                parent_snapshot: parent,
            });
        }
    }
    Ok(())
}

/// The transitions a policy may allow (spec §7): `Promoted` proposal →
/// canonical; `Demoted` to a lower layer; prune (a `Demoted` with
/// `meta.prune`) keeps a proposal's or derived lineage's layer and marks it
/// inactive. Inactive lineages take no further transition.
fn check_transition(kind: EventKind, m: &TransitionMeta, cur: Layer, inactive: bool) -> Result<()> {
    if m.from != cur {
        bail!(
            "requested from {} but the lineage is {}",
            m.from.as_str(),
            cur.as_str()
        );
    }
    if inactive {
        bail!("lineage is inactive (pruned)");
    }
    match (kind, m.prune) {
        (EventKind::Promoted, false) => {
            if (m.from, m.to) != (Layer::Proposal, Layer::Canonical) {
                bail!(
                    "promotion is proposal → canonical only, not {} → {}",
                    m.from.as_str(),
                    m.to.as_str()
                );
            }
        }
        (EventKind::Demoted, false) => {
            if m.to >= m.from {
                bail!(
                    "demotion must lower the layer, not {} → {}",
                    m.from.as_str(),
                    m.to.as_str()
                );
            }
        }
        (EventKind::Demoted, true) => {
            if m.to != m.from || m.from == Layer::Canonical {
                bail!(
                    "prune keeps a proposal or derived layer, not {} → {}",
                    m.from.as_str(),
                    m.to.as_str()
                );
            }
        }
        _ => bail!("{kind:?} cannot prune"),
    }
    Ok(())
}

fn committed_pairs(ev: &Event) -> Vec<(String, String)> {
    ev.lineage_ids
        .iter()
        .cloned()
        .zip(ev.new_version_ids.iter().cloned())
        .collect()
}

/// A `CheckpointCreated` event replayed: the checkpoint it must match.
#[derive(Debug, Clone)]
pub struct ReplayedCheckpoint {
    pub seq: u64,
    /// the checkpoint, with the name a later `CheckpointNamed` gave it
    pub checkpoint: Checkpoint,
    /// the name its `CheckpointCreated` recorded (its write-once file's)
    pub created_name: Option<String>,
}

/// Whole-log replay: document states plus every checkpoint, each verified
/// against the document heads at its offset.
#[derive(Debug, Clone, Default)]
pub struct Replayed {
    pub docs: BTreeMap<String, DocState>,
    pub checkpoints: Vec<ReplayedCheckpoint>,
    /// 1 + seq of the last non-checkpoint event (0 for none): the offset a
    /// new checkpoint records as `event_seq`
    pub content_offset: u64,
    /// an event carrying layer information has been applied (K-B1.1)
    pub layered: bool,
}

impl Replayed {
    pub fn heads(&self) -> BTreeMap<String, String> {
        self.docs
            .iter()
            .filter_map(|(d, s)| s.head.as_ref().map(|h| (d.clone(), h.snapshot_id.clone())))
            .collect()
    }

    pub fn apply(&mut self, ev: &Event) -> Result<()> {
        self.layered |= carries_layer(ev);
        if ev.kind == EventKind::CheckpointNamed {
            return self.apply_named(ev);
        }
        if ev.kind != EventKind::CheckpointCreated {
            if ev.doc_path.is_empty() {
                bail!("event seq {}: {:?} without doc_path", ev.seq, ev.kind);
            }
            let st = self.docs.entry(ev.doc_path.clone()).or_default();
            st.layered = self.layered;
            apply_event(st, ev)?;
            self.content_offset = ev.seq + 1;
            return Ok(());
        }
        if !ev.doc_path.is_empty() || !ev.lineage_ids.is_empty() {
            bail!("event seq {}: CheckpointCreated is vault-wide", ev.seq);
        }
        if let Some((d, _)) = self.docs.iter().find(|(_, s)| !s.pending.is_empty()) {
            bail!(
                "event seq {}: checkpoint inside an unfinished ingest of {d}",
                ev.seq
            );
        }
        let cid = meta_str(ev, "checkpoint_id")
            .with_context(|| format!("event seq {}: no meta.checkpoint_id", ev.seq))?;
        let event_seq = ev
            .meta
            .get("event_seq")
            .and_then(|v| v.as_u64())
            .with_context(|| format!("event seq {}: no meta.event_seq", ev.seq))?;
        if event_seq != self.content_offset {
            bail!(
                "event seq {}: checkpoint event_seq {event_seq} != log offset {}",
                ev.seq,
                self.content_offset
            );
        }
        let name = match ev.meta.get("name") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(n)) if !n.trim().is_empty() => Some(n.clone()),
            Some(other) => bail!(
                "event seq {}: checkpoint meta.name {other} is not a name",
                ev.seq
            ),
        };
        if let Some(n) = &name {
            if let Some(c) = self
                .checkpoints
                .iter()
                .find(|c| c.checkpoint.name.as_deref() == Some(n.as_str()))
            {
                bail!(
                    "event seq {}: checkpoint name {n:?} is taken by {}",
                    ev.seq,
                    c.checkpoint.checkpoint_id
                );
            }
        }
        let docs = self.heads();
        let computed = checkpoint_id(&docs, event_seq);
        if computed != cid {
            bail!(
                "event seq {}: checkpoint_id {cid} != computed {computed}",
                ev.seq
            );
        }
        self.checkpoints.push(ReplayedCheckpoint {
            seq: ev.seq,
            created_name: name.clone(),
            checkpoint: Checkpoint {
                checkpoint_id: cid,
                name,
                event_seq,
                docs,
            },
        });
        Ok(())
    }

    /// `CheckpointNamed` (K-D1.6): names an existing unnamed checkpoint with
    /// a name no checkpoint has; the id, offset and file stay as created.
    /// It is not a content event (the next checkpoint's `event_seq` ignores it).
    fn apply_named(&mut self, ev: &Event) -> Result<()> {
        if !ev.doc_path.is_empty() || !ev.lineage_ids.is_empty() {
            bail!("event seq {}: CheckpointNamed is vault-wide", ev.seq);
        }
        let cid = meta_str(ev, "checkpoint_id")
            .with_context(|| format!("event seq {}: no meta.checkpoint_id", ev.seq))?;
        let name = meta_str(ev, "name")
            .with_context(|| format!("event seq {}: CheckpointNamed without meta.name", ev.seq))?;
        check_checkpoint_name(&name).with_context(|| format!("event seq {}", ev.seq))?;
        if let Some(c) = self
            .checkpoints
            .iter()
            .find(|c| c.checkpoint.name.as_deref() == Some(name.as_str()))
        {
            bail!(
                "event seq {}: checkpoint name {name:?} is taken by {}",
                ev.seq,
                c.checkpoint.checkpoint_id
            );
        }
        let c = self
            .checkpoints
            .iter_mut()
            .find(|c| c.checkpoint.checkpoint_id == cid)
            .with_context(|| format!("event seq {}: no checkpoint {cid} to name", ev.seq))?;
        if let Some(n) = &c.checkpoint.name {
            bail!(
                "event seq {}: checkpoint {cid} is already named {n:?}",
                ev.seq
            );
        }
        c.checkpoint.name = Some(name);
        Ok(())
    }
}

/// Applies a sequence of events grouped by document.
pub fn replay_all(events: &[Event]) -> Result<Replayed> {
    let mut r = Replayed::default();
    for ev in events {
        r.apply(ev)?;
    }
    Ok(r)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestResult {
    pub snapshot: Snapshot,
    pub events: Vec<Event>,
    /// lineages touched by block events (created, updated, moved, split,
    /// merged, deleted, including split/merge sources), in event order
    pub changed_lineages: Vec<String>,
}

/// An ingest computed but not written (`Vault::plan_ingest`,
/// `Vault::commit_plan`).
#[derive(Debug, Clone)]
pub struct IngestPlan {
    pub doc_path: String,
    matcher: MatcherVersion,
    /// the events in log order (`seq` assigned on commit); empty when the
    /// document does not change
    pub events: Vec<Event>,
    objects: Vec<BlockObject>,
    changed: Vec<String>,
    /// the document's blocks before the ingest (head, current layers)
    pub blocks_before: Vec<Block>,
    /// the document's blocks after it, with the layers they would have
    pub blocks_after: Vec<Block>,
    /// `Some(head)`: nothing changes
    unchanged: Option<Snapshot>,
    /// an existing document's front matter changes
    frontmatter_changed: bool,
}

/// Compact per-document ingest report (CLI / `ingest_dir`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestSummary {
    pub doc_path: String,
    pub snapshot_id: String,
    pub blocks: usize,
    pub events: usize,
}

impl IngestSummary {
    pub fn new(doc_path: &str, r: &IngestResult) -> Self {
        IngestSummary {
            doc_path: doc_path.to_string(),
            snapshot_id: r.snapshot.snapshot_id.clone(),
            blocks: r.snapshot.blocks.len(),
            events: r.events.len(),
        }
    }
}

pub struct Vault {
    root: PathBuf,
    meta_dir: PathBuf,
    next_seq: u64,
    /// replayed log: document states, checkpoints, content offset
    state: Replayed,
    /// number of lines in `traces.jsonl` (the next `at_seq`)
    next_trace: u64,
    /// THETA / THETA_SPLIT override; `None` → the compiled defaults of the
    /// matcher version passed to `ingest`.
    pub params: Option<MatchParams>,
    /// Matcher version of the first logged event (SPEC_DELTA §7): every later
    /// ingest must use the same version.
    lock: Option<MatcherVersion>,
}

impl Vault {
    /// Opens (or initialises) `<root>/.rootmark` and replays the log into memory.
    pub fn open(root: impl AsRef<Path>) -> Result<Vault> {
        Self::open_inner(root, None)
    }

    /// Like `open`, with THETA / THETA_SPLIT fixed for every ingest.
    pub fn open_with(root: impl AsRef<Path>, params: MatchParams) -> Result<Vault> {
        Self::open_inner(root, Some(params))
    }

    fn open_inner(root: impl AsRef<Path>, params: Option<MatchParams>) -> Result<Vault> {
        let root = root.as_ref().to_path_buf();
        crate::meta::check(&root)?;
        let meta_dir = crate::meta::path(&root);
        fs::create_dir_all(meta_dir.join("objects"))?;
        fs::create_dir_all(meta_dir.join("snapshots"))?;
        let events = read_events(&meta_dir.join("events.jsonl"))?;
        let state = replay_all(&events)?;
        // the first block event fixes the matcher (governance events carry none)
        let lock = match events.iter().find(|e| is_block_event(e.kind)) {
            None => None,
            Some(ev) => {
                let v = meta_str(ev, "matcher_version")
                    .with_context(|| format!("event seq {}: no meta.matcher_version", ev.seq))?;
                Some(MatcherVersion::parse(&v)?)
            }
        };
        // every logged checkpoint has its write-once file, byte for byte
        for c in &state.checkpoints {
            let p = checkpoint_path(&meta_dir, &c.checkpoint.checkpoint_id);
            let want = checkpoint_bytes(&Checkpoint {
                name: c.created_name.clone(),
                ..c.checkpoint.clone()
            })?;
            let got = fs::read(&p).with_context(|| {
                format!(
                    "checkpoint {} (event seq {}) has no file",
                    c.checkpoint.checkpoint_id, c.seq
                )
            })?;
            if got != want {
                bail!("{} does not match its CheckpointCreated event", p.display());
            }
        }
        let mut v = Vault {
            root,
            meta_dir,
            next_seq: events.len() as u64,
            state,
            next_trace: 0,
            params,
            lock,
        };
        v.next_trace = v.traces(0)?.len() as u64;
        Ok(v)
    }

    /// Matcher version this vault is locked to (`None` while the log is empty).
    pub fn matcher_lock(&self) -> Option<MatcherVersion> {
        self.lock
    }

    /// THETA / THETA_SPLIT an ingest with `matcher` uses.
    pub fn params_for(&self, matcher: MatcherVersion) -> MatchParams {
        self.params.unwrap_or_else(|| matcher.defaults())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn events_path(&self) -> PathBuf {
        self.meta_dir.join("events.jsonl")
    }

    pub fn store_dir(&self) -> &Path {
        &self.meta_dir
    }

    pub fn doc_paths(&self) -> Vec<String> {
        self.state
            .docs
            .iter()
            .filter(|(_, s)| s.head.is_some())
            .map(|(d, _)| d.clone())
            .collect()
    }

    pub fn doc_state(&self, doc_path: &str) -> Option<&DocState> {
        self.state.docs.get(doc_path)
    }

    /// Head snapshot of a document (in-memory state built from the log).
    pub fn snapshot(&self, doc_path: &str) -> Result<Option<Snapshot>> {
        Ok(self.state.docs.get(doc_path).and_then(|s| s.head.clone()))
    }

    pub fn object(&self, version_id: &str) -> Result<BlockObject> {
        store::get_object(&self.meta_dir, version_id)
    }

    /// Content of a block version. Occurrence fields (`lineage_id`,
    /// `heading_path`, `parents`, `provenance`) are empty because one version
    /// can occur in several places; use `snapshot_blocks` for occurrences.
    pub fn block(&self, version_id: &str) -> Result<Block> {
        let o = self.object(version_id)?;
        Ok(Block {
            version_id: o.version_id,
            lineage_id: String::new(),
            block_type: o.block_type,
            text: o.normalized_text.clone(),
            normalized_text: o.normalized_text,
            heading_path: Vec::new(),
            heading_level: o.heading_level,
            parents: Vec::new(),
            provenance: Provenance::default(),
            layer: Layer::default_for(Provenance::default().author_kind),
        })
    }

    /// Full block occurrences of a snapshot, in document order.
    pub fn snapshot_blocks(&self, snap: &Snapshot) -> Result<Vec<Block>> {
        self.blocks_of(self.state.docs.get(&snap.doc_path), snap)
    }

    /// Block occurrences of `snap` with the occurrence data (provenance,
    /// parents, layer) of the document state `st`.
    pub(crate) fn blocks_of(&self, st: Option<&DocState>, snap: &Snapshot) -> Result<Vec<Block>> {
        let objs = snap
            .blocks
            .iter()
            .map(|(_, v)| self.object(v))
            .collect::<Result<Vec<_>>>()?;
        let paths = heading_paths(
            objs.iter()
                .map(|o| (o.block_type, o.heading_level, o.normalized_text.as_str())),
        );
        Ok(snap
            .blocks
            .iter()
            .zip(objs)
            .zip(paths)
            .map(|(((l, _), o), hp)| {
                let provenance: Provenance = st
                    .and_then(|s| s.provenance.get(l).cloned())
                    .unwrap_or_default();
                Block {
                    version_id: o.version_id,
                    lineage_id: l.clone(),
                    block_type: o.block_type,
                    text: o.normalized_text.clone(),
                    normalized_text: o.normalized_text,
                    heading_path: hp,
                    heading_level: o.heading_level,
                    parents: st
                        .and_then(|s| s.parents.get(l).cloned())
                        .unwrap_or_default(),
                    layer: st
                        .and_then(|s| s.layer.get(l).copied())
                        .unwrap_or_else(|| Layer::default_for(provenance.author_kind)),
                    provenance,
                }
            })
            .collect())
    }

    /// Rebuilds the last snapshot of `doc_path` from the on-disk log alone and
    /// verifies every `snapshot_id` and the stored snapshot object.
    pub fn replay(&self, doc_path: &str) -> Result<Option<Snapshot>> {
        let events = read_events(&self.events_path())?;
        let mut st = DocState::default();
        for ev in &events {
            // the log's format is vault-wide (K-B1.1)
            st.layered |= carries_layer(ev);
            if ev.doc_path == doc_path {
                apply_event(&mut st, ev)?;
            }
        }
        if let Some(s) = &st.head {
            let obj = store::get_snapshot(&self.meta_dir, &s.snapshot_id)?;
            let vids: Vec<String> = s.blocks.iter().map(|(_, v)| v.clone()).collect();
            if obj.version_ids != vids {
                bail!("snapshot object {} does not match replay", s.snapshot_id);
            }
        }
        Ok(st.head)
    }

    /// blake3 of the whole `events.jsonl` (first 32 hex chars); H4-1.
    pub fn events_hash(&self) -> Result<String> {
        let p = self.events_path();
        let bytes = if p.exists() {
            fs::read(&p)?
        } else {
            Vec::new()
        };
        let mut h = blake3::hash(&bytes).to_hex().to_string();
        h.truncate(32);
        Ok(h)
    }

    /// Low-level commit: stores objects, assigns `seq`, appends the events,
    /// then applies them to the in-memory state (which re-verifies them).
    /// Layer transitions (`Promoted`, `Demoted`) are refused here: they are
    /// written only by `promote` / `demote` / `prune_inactive`, after the
    /// policy hook allowed them (C2-03; B2 review).
    pub fn commit(&mut self, objects: &[BlockObject], events: Vec<Event>) -> Result<Vec<Event>> {
        if let Some(e) = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::Promoted | EventKind::Demoted))
        {
            bail!(
                "{:?} is written only through the policy hook (promote/demote/prune_inactive)",
                e.kind
            );
        }
        self.commit_inner(objects, events)
    }

    fn commit_inner(
        &mut self,
        objects: &[BlockObject],
        mut events: Vec<Event>,
    ) -> Result<Vec<Event>> {
        // verify on a copy before anything is written
        let mut staged: BTreeMap<String, DocState> = BTreeMap::new();
        let mut layered = self.state.layered;
        for (i, ev) in events.iter_mut().enumerate() {
            ev.seq = self.next_seq + i as u64;
            if ev.kind == EventKind::CheckpointCreated || ev.doc_path.is_empty() {
                bail!("commit takes document events only ({:?})", ev.kind);
            }
            let st = staged.entry(ev.doc_path.clone()).or_insert_with(|| {
                self.state
                    .docs
                    .get(&ev.doc_path)
                    .cloned()
                    .unwrap_or_default()
            });
            layered |= carries_layer(ev);
            st.layered = layered;
            apply_event(st, ev)?;
        }
        for o in objects {
            store::put_object(&self.meta_dir, o)?;
        }
        for ev in &events {
            if ev.kind == EventKind::SnapshotCommitted {
                let sid = ev
                    .snapshot_id
                    .clone()
                    .context("SnapshotCommitted without snapshot_id")?;
                store::put_snapshot(
                    &self.meta_dir,
                    &SnapshotObject {
                        snapshot_id: sid,
                        version_ids: ev.new_version_ids.clone(),
                    },
                )?;
            }
        }
        append_events(&self.events_path(), &events)?;
        self.next_seq += events.len() as u64;
        self.state.docs.extend(staged);
        self.state.layered = layered;
        if !events.is_empty() {
            self.state.content_offset = self.next_seq;
        }
        Ok(events)
    }

    /// parse → match → store → log (SPEC §7). Re-ingesting identical block
    /// content and frontmatter produces no events.
    /// A vault is locked to the matcher version of its first event; ingesting
    /// with another version is an error (SPEC_DELTA §7).
    pub fn ingest(
        &mut self,
        doc_path: &str,
        text: &str,
        prov: Provenance,
        matcher: MatcherVersion,
    ) -> Result<IngestResult> {
        self.ingest_declared(doc_path, text, prov, matcher, None)
    }

    /// `ingest` with a declared layer for synthetic content (spec §7:
    /// "`synthetic` → as declared"; recorded as `meta.layer` of its block
    /// events). Declaring a layer for any other author is an error.
    /// Layers (spec §7): new lineages get the author's layer (lowered to the
    /// lowest parent layer for split/merge pieces); when an edit implies a
    /// lower layer for an inherited lineage a `LayerChanged` event follows the
    /// block events. Ingest never raises a layer.
    pub fn ingest_declared(
        &mut self,
        doc_path: &str,
        text: &str,
        prov: Provenance,
        matcher: MatcherVersion,
        declared: Option<Layer>,
    ) -> Result<IngestResult> {
        let plan = self.plan_ingest(doc_path, text, prov, matcher, declared, None)?;
        self.commit_plan(plan)
    }

    /// An attributed ingest (K-D1.2): like `ingest`, and every
    /// `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged` and
    /// `BlockDeleted` it writes carries `meta.actor` (who caused the ingest)
    /// and `meta.source` (`prov.source`). Every write path of the server
    /// (MCP, HTTP, watcher, `rootmark propose`) ingests through this.
    pub fn ingest_by(
        &mut self,
        doc_path: &str,
        text: &str,
        prov: Provenance,
        matcher: MatcherVersion,
        actor: &Actor,
    ) -> Result<IngestResult> {
        let plan = self.plan_ingest(doc_path, text, prov, matcher, None, Some(actor))?;
        self.commit_plan(plan)
    }

    /// The ingest of `text` as `doc_path`, computed and verified against the
    /// replayed state but not written (`commit_plan` writes it). With
    /// `actor`, the block events that change or remove existing blocks carry
    /// it (`ingest_by`).
    pub fn plan_ingest(
        &self,
        doc_path: &str,
        text: &str,
        prov: Provenance,
        matcher: MatcherVersion,
        declared: Option<Layer>,
        actor: Option<&Actor>,
    ) -> Result<IngestPlan> {
        if declared.is_some() && prov.author_kind != AuthorKind::Synthetic {
            bail!(
                "a layer can be declared for synthetic content only, not {:?}",
                prov.author_kind
            );
        }
        if doc_path.is_empty() {
            bail!("empty doc_path");
        }
        if let Some(lock) = self.lock {
            if lock != matcher {
                bail!(
                    "vault is locked to matcher {}; ingest with {} refused",
                    lock.as_str(),
                    matcher.as_str()
                );
            }
        }
        if let Some(a) = actor {
            if a.id.trim().is_empty() {
                bail!("actor id is empty");
            }
        }
        let (drafts, frontmatter) = parse_document(text, doc_path)?;
        let st = self.state.docs.get(doc_path);
        let head = st.and_then(|s| s.head.clone());
        let before = match &head {
            Some(h) => self.snapshot_blocks(h)?,
            None => Vec::new(),
        };
        let prev: Vec<PrevBlock> = before.iter().map(PrevBlock::from).collect();
        let used = st.map(|s| s.used.clone()).unwrap_or_default();
        let params = self.params_for(matcher);
        let r = match_blocks(doc_path, &prev, &drafts, &used, params, &prov, matcher);
        let fm_changed = head.as_ref().is_none_or(|h| h.frontmatter != frontmatter);
        if r.events.is_empty() && !fm_changed {
            let snapshot = head.expect("head exists when frontmatter is unchanged");
            return Ok(IngestPlan {
                doc_path: doc_path.to_string(),
                matcher,
                events: Vec::new(),
                objects: Vec::new(),
                changed: Vec::new(),
                blocks_before: before.clone(),
                blocks_after: before,
                unchanged: Some(snapshot),
                frontmatter_changed: false,
            });
        }
        let mut changed: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for ev in &r.events {
            let from = ev
                .meta
                .get("from_lineages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let from = from.iter().filter_map(|x| x.as_str().map(str::to_string));
            for l in from.chain(ev.lineage_ids.iter().cloned()) {
                if seen.insert(l.clone()) {
                    changed.push(l);
                }
            }
        }
        let parent = head.as_ref().map(|h| h.snapshot_id.clone());
        let sid = snapshot_id(r.blocks.iter().map(|b| b.version_id.as_str()));
        let mut events = r.events;
        if let Some(layer) = declared {
            for ev in events.iter_mut().filter(|e| {
                matches!(
                    e.kind,
                    EventKind::BlockCreated
                        | EventKind::BlockUpdated
                        | EventKind::BlockSplit
                        | EventKind::BlockMerged
                )
            }) {
                ev.meta["layer"] = serde_json::json!(layer.as_str());
            }
        }
        if let Some(a) = actor {
            for ev in events.iter_mut().filter(|e| carries_actor(e.kind)) {
                ev.meta["actor"] = serde_json::to_value(a)?;
                ev.meta["source"] = serde_json::json!(prov.source);
            }
        }
        // layer transitions the edit implies, recorded before the snapshot
        let mut probe = self.state.docs.get(doc_path).cloned().unwrap_or_default();
        for ev in &events {
            apply_event(&mut probe, ev)?;
        }
        for (l, to) in &probe.pending {
            let from = probe.layer[l];
            let v = probe.live[l].clone();
            events.push(Event {
                seq: 0,
                doc_path: doc_path.to_string(),
                kind: EventKind::LayerChanged,
                lineage_ids: vec![l.clone()],
                old_version_ids: vec![v.clone()],
                new_version_ids: vec![v],
                snapshot_id: None,
                meta: serde_json::json!({
                    "from": from.as_str(),
                    "to": to.as_str(),
                    "cause": "ingest",
                    "author_kind": prov.author_kind,
                }),
            });
        }
        // the layers after the ingest: the pending drops applied
        for (l, to) in std::mem::take(&mut probe.pending) {
            probe.layer.insert(l, to);
        }
        events.push(Event {
            seq: 0,
            doc_path: doc_path.to_string(),
            kind: EventKind::SnapshotCommitted,
            lineage_ids: r.blocks.iter().map(|b| b.lineage_id.clone()).collect(),
            old_version_ids: Vec::new(),
            new_version_ids: r.blocks.iter().map(|b| b.version_id.clone()).collect(),
            snapshot_id: Some(sid),
            meta: serde_json::json!({
                "matcher_version": matcher.as_str(),
                "frontmatter": frontmatter,
                "parent_snapshot": parent,
            }),
        });
        let objects: Vec<BlockObject> = r
            .blocks
            .iter()
            .map(|b| BlockObject {
                version_id: b.version_id.clone(),
                block_type: b.block_type,
                normalized_text: b.normalized_text.clone(),
                heading_level: b.heading_level,
            })
            .collect();
        let blocks_after: Vec<Block> = r
            .blocks
            .iter()
            .map(|b| {
                let mut b = b.clone();
                if let Some(l) = probe.layer.get(&b.lineage_id) {
                    b.layer = *l;
                }
                if let Some(p) = probe.provenance.get(&b.lineage_id) {
                    b.provenance = p.clone();
                }
                b
            })
            .collect();
        Ok(IngestPlan {
            doc_path: doc_path.to_string(),
            matcher,
            events,
            objects,
            changed,
            blocks_before: before,
            blocks_after,
            unchanged: None,
            frontmatter_changed: head.is_some() && fm_changed,
        })
    }

    /// Writes a plan of `plan_ingest` (objects, snapshot, events). The plan
    /// must be applied to the state it was computed on: the events are
    /// verified again, and a plan of an earlier state fails.
    pub fn commit_plan(&mut self, plan: IngestPlan) -> Result<IngestResult> {
        if let Some(snapshot) = plan.unchanged {
            return Ok(IngestResult {
                snapshot,
                events: Vec::new(),
                changed_lineages: Vec::new(),
            });
        }
        let events = self.commit(&plan.objects, plan.events)?;
        self.lock.get_or_insert(plan.matcher);
        let snapshot = self
            .snapshot(&plan.doc_path)?
            .context("snapshot after commit")?;
        Ok(IngestResult {
            snapshot,
            events,
            changed_lineages: plan.changed,
        })
    }

    /// What a planned ingest would do to the document's existing content
    /// (K-D1.1, `EditPolicy`): the live lineages its events change or remove,
    /// with their layer before; and the structural chunks (spec §6) made only
    /// of `canonical` blocks that it would remove or add.
    pub fn edit_effect(&self, plan: &IngestPlan) -> Result<EditEffect> {
        let layer_before: HashMap<&str, Layer> = plan
            .blocks_before
            .iter()
            .map(|b| (b.lineage_id.as_str(), b.layer))
            .collect();
        let mut touched: Vec<TouchedLineage> = Vec::new();
        for ev in &plan.events {
            let names: Vec<String> = match ev.kind {
                EventKind::BlockUpdated
                | EventKind::BlockMoved
                | EventKind::BlockDeleted
                | EventKind::LayerChanged => ev.lineage_ids.clone(),
                EventKind::BlockSplit | EventKind::BlockMerged => meta_list(ev, "from_lineages"),
                _ => Vec::new(),
            };
            for l in names {
                let Some(&layer) = layer_before.get(l.as_str()) else {
                    // a lineage this ingest itself created (e.g. a layer
                    // drop of a new split piece): not existing content
                    continue;
                };
                match touched.iter_mut().find(|t| t.lineage_id == l) {
                    Some(t) => t.kinds.push(ev.kind),
                    None => touched.push(TouchedLineage {
                        lineage_id: l,
                        layer,
                        kinds: vec![ev.kind],
                    }),
                }
            }
        }
        let canonical = |blocks: &[Block]| -> BTreeSet<(String, String)> {
            let chunks = crate::chunk::structural(&plan.doc_path, blocks);
            chunks
                .into_iter()
                .filter(|c| {
                    c.lineage_ids.iter().all(|l| {
                        blocks
                            .iter()
                            .find(|b| &b.lineage_id == l)
                            .is_some_and(|b| b.layer == Layer::Canonical)
                    })
                })
                .map(|c| (c.chunk_id, c.text))
                .collect()
        };
        let before = canonical(&plan.blocks_before);
        let after = canonical(&plan.blocks_after);
        Ok(EditEffect {
            touched,
            canonical_chunks_removed: before.difference(&after).map(|c| c.0.clone()).collect(),
            canonical_chunks_added: after.difference(&before).map(|c| c.0.clone()).collect(),
            frontmatter_changed: plan.frontmatter_changed,
        })
    }

    /// Content-referenced truth → lineage: the `k`-th block (document order)
    /// with `version_id` in the head snapshot of `doc_path`. Never empty: an
    /// unresolvable reference is an error.
    pub fn resolve_truth(&self, doc_path: &str, version_id: &str, k: usize) -> Result<String> {
        let snap = self
            .snapshot(doc_path)?
            .with_context(|| format!("resolve_truth: no snapshot for {doc_path}"))?;
        snap.blocks
            .iter()
            .filter(|(_, v)| v == version_id)
            .nth(k)
            .map(|(l, _)| l.clone())
            .with_context(|| {
                format!("resolve_truth: {doc_path} has no occurrence {k} of {version_id}")
            })
    }

    /// Ingests every `*.md` under `dir` in byte-wise `doc_path` order
    /// (deterministic); `doc_path` is relative to `dir`.
    pub fn ingest_dir(
        &mut self,
        dir: &Path,
        matcher: MatcherVersion,
    ) -> Result<Vec<IngestSummary>> {
        let mut out = Vec::new();
        for (doc, p) in crate::files::markdown_files(dir)? {
            let text = fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
            let r = self.ingest(&doc, &text, Provenance::default(), matcher)?;
            out.push(IngestSummary::new(&doc, &r));
        }
        Ok(out)
    }
}

fn is_block_event(k: EventKind) -> bool {
    matches!(
        k,
        EventKind::BlockCreated
            | EventKind::BlockUpdated
            | EventKind::BlockMoved
            | EventKind::BlockSplit
            | EventKind::BlockMerged
            | EventKind::BlockDeleted
            | EventKind::SnapshotCommitted
    )
}

pub fn checkpoint_path(meta_dir: &Path, cid: &str) -> PathBuf {
    meta_dir.join("checkpoints").join(format!("{cid}.json"))
}

/// The write-once bytes of a checkpoint file (compact JSON, `docs` sorted).
pub fn checkpoint_bytes(c: &Checkpoint) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(c)?)
}

/// A block a consumer read (`Vault::read_blocks`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBlock {
    pub doc_path: String,
    pub block: Block,
    /// pruned by policy (spec §7)
    pub inactive: bool,
}

/// Governance (spec §7, §8): layers, transitions through a policy hook,
/// checkpoints, consumption traces.
impl Vault {
    /// `(doc_path, live version)` of a lineage at the head.
    fn locate(&self, lineage: &str) -> Result<(String, String)> {
        self.state
            .docs
            .iter()
            .find_map(|(d, s)| s.live.get(lineage).map(|v| (d.clone(), v.clone())))
            .with_context(|| format!("lineage {lineage} is not live in this vault"))
    }

    /// Current layer of a live lineage (spec §7).
    pub fn layer_of(&self, lineage: &str) -> Result<Layer> {
        let (doc, _) = self.locate(lineage)?;
        Ok(self.state.docs[&doc].layer[lineage])
    }

    /// Whether a live lineage was pruned (marked inactive).
    pub fn is_inactive(&self, lineage: &str) -> Result<bool> {
        let (doc, _) = self.locate(lineage)?;
        Ok(self.state.docs[&doc].inactive.contains(lineage))
    }

    /// A request for `lineage` from its current layer.
    pub fn request(
        &self,
        kind: crate::governance::TransitionKind,
        lineage: &str,
        to: Layer,
        actor: Actor,
        reason: &str,
    ) -> Result<crate::governance::PromotionRequest> {
        Ok(crate::governance::PromotionRequest {
            kind,
            lineage: lineage.to_string(),
            from: self.layer_of(lineage)?,
            to,
            actor,
            reason: reason.to_string(),
        })
    }

    /// `Promoted` (proposal → canonical) if `policy` allows `req`.
    pub fn promote(
        &mut self,
        req: &crate::governance::PromotionRequest,
        policy: &dyn crate::governance::PromotionPolicy,
    ) -> Result<Event> {
        if req.kind != crate::governance::TransitionKind::Promote {
            bail!("promote takes a promote request, not {:?}", req.kind);
        }
        self.transition(EventKind::Promoted, req, policy)
    }

    /// `Demoted` to a lower layer if `policy` allows `req`.
    pub fn demote(
        &mut self,
        req: &crate::governance::PromotionRequest,
        policy: &dyn crate::governance::PromotionPolicy,
    ) -> Result<Event> {
        if req.kind != crate::governance::TransitionKind::Demote {
            bail!("demote takes a demote request, not {:?}", req.kind);
        }
        self.transition(EventKind::Demoted, req, policy)
    }

    /// Marks a proposal (or derived) lineage inactive if `policy` allows it: a
    /// `Demoted` event with `meta.prune`, layer unchanged. Nothing is deleted
    /// (spec §7, C2-04).
    pub fn prune_inactive(
        &mut self,
        lineage: &str,
        actor: Actor,
        reason: &str,
        policy: &dyn crate::governance::PromotionPolicy,
    ) -> Result<Event> {
        let cur = self.layer_of(lineage)?;
        let req = self.request(
            crate::governance::TransitionKind::Prune,
            lineage,
            cur,
            actor,
            reason,
        )?;
        self.transition(EventKind::Demoted, &req, policy)
    }

    fn transition(
        &mut self,
        kind: EventKind,
        req: &crate::governance::PromotionRequest,
        policy: &dyn crate::governance::PromotionPolicy,
    ) -> Result<Event> {
        use crate::governance::{Decision, TransitionKind};
        if req.reason.trim().is_empty() {
            bail!("a layer transition requires a reason");
        }
        if req.actor.id.trim().is_empty() {
            bail!("a layer transition requires an actor");
        }
        if policy.name().trim().is_empty() || policy.version().trim().is_empty() {
            bail!("policy without name or version");
        }
        let (doc, v) = self.locate(&req.lineage)?;
        let prune = req.kind == TransitionKind::Prune;
        let mut meta = serde_json::json!({
            "actor": req.actor,
            "reason": req.reason,
            "policy": policy.name(),
            "policy_version": policy.version(),
            "from": req.from.as_str(),
            "to": req.to.as_str(),
        });
        if prune {
            meta["prune"] = serde_json::json!(true);
        }
        let ev = Event {
            seq: 0,
            doc_path: doc.clone(),
            kind,
            lineage_ids: vec![req.lineage.clone()],
            old_version_ids: vec![v.clone()],
            new_version_ids: vec![v],
            snapshot_id: None,
            meta,
        };
        // shape and state first (spec §7 rules), then the hook decides
        {
            let st = &self.state.docs[&doc];
            let m = transition_meta(&ev)?;
            check_transition(
                kind,
                &m,
                st.layer[&req.lineage],
                st.inactive.contains(&req.lineage),
            )
            .with_context(|| format!("{:?} of {}", req.kind, req.lineage))?;
        }
        match policy.decide(req) {
            Decision::Allow => {}
            Decision::Deny(why) => bail!(
                "{:?} of {} denied by policy {} v{}: {why}",
                req.kind,
                req.lineage,
                policy.name(),
                policy.version()
            ),
        }
        let mut out = self.commit_inner(&[], vec![ev])?;
        Ok(out.remove(0))
    }

    /// Every checkpoint, in log order.
    pub fn checkpoints(&self) -> Vec<Checkpoint> {
        self.state
            .checkpoints
            .iter()
            .map(|c| c.checkpoint.clone())
            .collect()
    }

    /// A checkpoint by its full id or its name.
    pub fn find_checkpoint(&self, id_or_name: &str) -> Result<Checkpoint> {
        self.state
            .checkpoints
            .iter()
            .map(|c| &c.checkpoint)
            .find(|c| c.checkpoint_id == id_or_name || c.name.as_deref() == Some(id_or_name))
            .cloned()
            .with_context(|| format!("no checkpoint {id_or_name:?}"))
    }

    /// Creates (or returns) the checkpoint of the current vault state
    /// (spec §8). `event_seq` is the log offset after the last
    /// non-checkpoint event, so calling this twice on the same state yields
    /// the same `checkpoint_id` and writes nothing the second time. The
    /// file `.rootmark/checkpoints/<cid>.json` is write-once. If this state
    /// already has an unnamed checkpoint and `name` is given, the checkpoint
    /// is named (`CheckpointNamed`, K-D1.6; same id); a state whose
    /// checkpoint has another name, or a name another checkpoint has, is an
    /// error.
    pub fn checkpoint(&mut self, name: Option<&str>) -> Result<Checkpoint> {
        if let Some(n) = name {
            check_checkpoint_name(n)?;
        }
        let event_seq = self.state.content_offset;
        let docs = self.state.heads();
        let cid = checkpoint_id(&docs, event_seq);
        if let Some(c) = self
            .checkpoints()
            .into_iter()
            .find(|c| c.checkpoint_id == cid)
        {
            return match (c.name.as_deref(), name) {
                (a, b) if a == b => Ok(c),
                (None, Some(n)) => self.name_checkpoint(&cid, n),
                (a, _) => bail!(
                    "checkpoint {cid} of this state already exists with name {:?}",
                    a
                ),
            };
        }
        if let Some(n) = name {
            if let Some(c) = self
                .checkpoints()
                .iter()
                .find(|c| c.name.as_deref() == Some(n))
            {
                bail!("checkpoint name {n:?} is taken by {}", c.checkpoint_id);
            }
        }
        let cp = Checkpoint {
            checkpoint_id: cid.clone(),
            name: name.map(str::to_string),
            event_seq,
            docs,
        };
        store::write_immutable(
            &checkpoint_path(&self.meta_dir, &cid),
            &checkpoint_bytes(&cp)?,
        )?;
        let ev = Event {
            seq: self.next_seq,
            doc_path: String::new(),
            kind: EventKind::CheckpointCreated,
            lineage_ids: Vec::new(),
            old_version_ids: Vec::new(),
            new_version_ids: Vec::new(),
            snapshot_id: None,
            meta: serde_json::json!({
                "checkpoint_id": cid,
                "name": name,
                "event_seq": event_seq,
            }),
        };
        let mut probe = self.state.clone();
        probe.apply(&ev)?;
        append_events(&self.events_path(), std::slice::from_ref(&ev))?;
        self.next_seq += 1;
        self.state = probe;
        Ok(cp)
    }

    /// The id the checkpoint of the current state has (or would have), by
    /// the spec §3 formula; nothing is written.
    pub fn current_checkpoint_id(&self) -> String {
        checkpoint_id(&self.state.heads(), self.state.content_offset)
    }

    /// The checkpoint of the current state: the existing one (named or not)
    /// if this state already has one, else a new unnamed checkpoint (spec §8).
    /// Searches without an explicit checkpoint run, and trace, at this one.
    pub fn head_checkpoint(&mut self) -> Result<Checkpoint> {
        let cid = checkpoint_id(&self.state.heads(), self.state.content_offset);
        if let Some(c) = self
            .checkpoints()
            .into_iter()
            .find(|c| c.checkpoint_id == cid)
        {
            return Ok(c);
        }
        self.checkpoint(None)
    }

    /// Names the unnamed checkpoint `cid` (K-D1.6): one `CheckpointNamed`
    /// event; the checkpoint id and file do not change.
    pub fn name_checkpoint(&mut self, cid: &str, name: &str) -> Result<Checkpoint> {
        check_checkpoint_name(name)?;
        let ev = Event {
            seq: self.next_seq,
            doc_path: String::new(),
            kind: EventKind::CheckpointNamed,
            lineage_ids: Vec::new(),
            old_version_ids: Vec::new(),
            new_version_ids: Vec::new(),
            snapshot_id: None,
            meta: serde_json::json!({"checkpoint_id": cid, "name": name}),
        };
        let mut probe = self.state.clone();
        probe.apply(&ev)?;
        append_events(&self.events_path(), std::slice::from_ref(&ev))?;
        self.next_seq += 1;
        self.state = probe;
        self.find_checkpoint(cid)
    }

    /// The vault as of a checkpoint: the log replayed up to its offset.
    pub fn state_at(&self, checkpoint: &str) -> Result<(Checkpoint, Replayed)> {
        let cp = self.find_checkpoint(checkpoint)?;
        let events = read_events(&self.events_path())?;
        let upto = usize::try_from(cp.event_seq)?;
        if upto > events.len() {
            bail!("checkpoint {} lies beyond the log", cp.checkpoint_id);
        }
        let st = replay_all(&events[..upto])?;
        if st.heads() != cp.docs {
            bail!(
                "checkpoint {} does not match the replayed log",
                cp.checkpoint_id
            );
        }
        Ok((cp, st))
    }

    /// Snapshot of `doc_path` as of a checkpoint (`None`: not yet ingested).
    pub fn snapshot_at(&self, doc_path: &str, checkpoint: &str) -> Result<Option<Snapshot>> {
        let (_, st) = self.state_at(checkpoint)?;
        Ok(st.docs.get(doc_path).and_then(|s| s.head.clone()))
    }

    /// Blocks of `doc_path` as of a checkpoint, with their layers then.
    pub fn blocks_at(&self, doc_path: &str, checkpoint: &str) -> Result<Vec<Block>> {
        let (_, st) = self.state_at(checkpoint)?;
        let ds = st
            .docs
            .get(doc_path)
            .with_context(|| format!("{doc_path} has no snapshot at checkpoint {checkpoint}"))?;
        let snap = ds.head.clone().context("no head")?;
        self.blocks_of(Some(ds), &snap)
    }

    /// The block of `lineage` as of a checkpoint (layer as of then).
    pub fn block_at(&self, lineage: &str, checkpoint: &str) -> Result<Block> {
        let (_, st) = self.state_at(checkpoint)?;
        self.find_block(&st, lineage).map(|(_, b, _)| b)
    }

    fn find_block(&self, st: &Replayed, lineage: &str) -> Result<(String, Block, bool)> {
        for (d, ds) in &st.docs {
            if ds.live.contains_key(lineage) {
                let snap = ds.head.clone().context("live lineage without head")?;
                let b = self
                    .blocks_of(Some(ds), &snap)?
                    .into_iter()
                    .find(|b| b.lineage_id == lineage)
                    .context("live lineage missing from its head snapshot")?;
                return Ok((d.clone(), b, ds.inactive.contains(lineage)));
            }
        }
        bail!("lineage {lineage} is not live at this checkpoint")
    }

    pub fn traces_path(&self) -> PathBuf {
        self.meta_dir.join("traces.jsonl")
    }

    /// Appends a consumption trace (spec §8) for a checkpoint of this vault.
    pub fn record_trace(
        &mut self,
        checkpoint_id: &str,
        consumer: &str,
        query_hash: &str,
        returned: Vec<crate::governance::TraceEntry>,
    ) -> Result<crate::governance::Trace> {
        if consumer.trim().is_empty() || query_hash.trim().is_empty() {
            bail!("a trace requires a consumer and a query hash");
        }
        let cp = self.find_checkpoint(checkpoint_id)?;
        if cp.checkpoint_id != checkpoint_id {
            bail!("record_trace takes a checkpoint id, not the name {checkpoint_id:?}");
        }
        // every returned version must be live at the checkpoint (B4 review)
        if !returned.is_empty() {
            let (_, st) = self.state_at(checkpoint_id)?;
            for e in &returned {
                let live = st
                    .docs
                    .get(&e.doc_path)
                    .and_then(|d| d.live.get(&e.lineage_id));
                if live != Some(&e.version_id) {
                    bail!(
                        "trace entry {}/{}@{} is not live at checkpoint {checkpoint_id}",
                        e.doc_path,
                        e.lineage_id,
                        e.version_id
                    );
                }
            }
        }
        crate::governance::check_trace_input(consumer, query_hash)?;
        let at_seq = self.next_trace;
        let t = crate::governance::Trace {
            trace_id: crate::governance::trace_id(checkpoint_id, consumer, query_hash, at_seq),
            checkpoint_id: checkpoint_id.to_string(),
            consumer: consumer.to_string(),
            query_hash: query_hash.to_string(),
            returned,
            at_seq,
        };
        let mut line = serde_json::to_vec(&t)?;
        line.push(b'\n');
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.traces_path())?;
        std::io::Write::write_all(&mut f, &line)?;
        f.sync_data()?;
        self.next_trace += 1;
        Ok(t)
    }

    /// Reads blocks by lineage as of a checkpoint and records one trace
    /// (spec §8, C2-07). `derived` content is never returned (spec §7):
    /// asking for it is an error; proposals are returned labelled.
    pub fn read_blocks(
        &mut self,
        refs: &[String],
        consumer: &str,
        checkpoint: &str,
    ) -> Result<(Vec<ReadBlock>, crate::governance::Trace)> {
        if refs.is_empty() {
            bail!("read_blocks: no lineage requested");
        }
        let (cp, st) = self.state_at(checkpoint)?;
        let mut out = Vec::new();
        for l in refs {
            let (doc, block, inactive) = self.find_block(&st, l)?;
            if block.layer == Layer::Derived {
                bail!("lineage {l} is derived: never returned as content (spec §7)");
            }
            out.push(ReadBlock {
                doc_path: doc,
                block,
                inactive,
            });
        }
        let query_hash = hash32(&format!("read_blocks\n{}", refs.join("\n")));
        let returned = out
            .iter()
            .map(|r| crate::governance::TraceEntry {
                doc_path: r.doc_path.clone(),
                lineage_id: r.block.lineage_id.clone(),
                version_id: r.block.version_id.clone(),
            })
            .collect();
        let t = self.record_trace(&cp.checkpoint_id, consumer, &query_hash, returned)?;
        Ok((out, t))
    }

    /// Traces with `at_seq >= since`; every line is verified (contiguous
    /// `at_seq`, `trace_id` formula, known checkpoint).
    pub fn traces(&self, since: u64) -> Result<Vec<crate::governance::Trace>> {
        let p = self.traces_path();
        if !p.exists() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let t: crate::governance::Trace = serde_json::from_str(line)
                .with_context(|| format!("traces.jsonl line {}", i + 1))?;
            if t.at_seq != i as u64 {
                bail!("traces.jsonl line {}: at_seq {} != {i}", i + 1, t.at_seq);
            }
            crate::governance::check_trace_input(&t.consumer, &t.query_hash)
                .with_context(|| format!("traces.jsonl line {}", i + 1))?;
            let want =
                crate::governance::trace_id(&t.checkpoint_id, &t.consumer, &t.query_hash, t.at_seq);
            if t.trace_id != want {
                bail!(
                    "traces.jsonl line {}: trace_id {} != {want}",
                    i + 1,
                    t.trace_id
                );
            }
            if !self
                .state
                .checkpoints
                .iter()
                .any(|c| c.checkpoint.checkpoint_id == t.checkpoint_id)
            {
                bail!(
                    "traces.jsonl line {}: unknown checkpoint {}",
                    i + 1,
                    t.checkpoint_id
                );
            }
            if t.at_seq >= since {
                out.push(t);
            }
        }
        Ok(out)
    }

    /// Every event that names `lineage` (in `lineage_ids` or as a split/merge
    /// source), in log order.
    pub fn history(&self, lineage: &str) -> Result<Vec<Event>> {
        let out: Vec<Event> = read_events(&self.events_path())?
            .into_iter()
            .filter(|e| {
                e.kind != EventKind::SnapshotCommitted
                    && (e.lineage_ids.iter().any(|l| l == lineage)
                        || meta_list(e, "from_lineages").iter().any(|l| l == lineage))
            })
            .collect();
        if out.is_empty() {
            bail!("no event names lineage {lineage}");
        }
        Ok(out)
    }

    /// Per block of `doc_path` (document order, at the head or as of a
    /// checkpoint): lineage, version, layer, author, source, the seq that
    /// created the lineage and the last seq that named it.
    pub fn blame(&self, doc_path: &str, checkpoint: Option<&str>) -> Result<Vec<BlameLine>> {
        let at;
        let st = match checkpoint {
            Some(c) => {
                at = self.state_at(c)?.1;
                &at
            }
            None => &self.state,
        };
        let ds = st
            .docs
            .get(doc_path)
            .with_context(|| format!("no snapshot for {doc_path}"))?;
        let snap = ds
            .head
            .clone()
            .with_context(|| format!("no snapshot for {doc_path}"))?;
        self.blocks_of(Some(ds), &snap)?
            .into_iter()
            .map(|b| {
                let l = &b.lineage_id;
                Ok(BlameLine {
                    lineage_id: l.clone(),
                    version_id: b.version_id.clone(),
                    layer: b.layer,
                    layer_source: ds.layer_source(l),
                    inactive: ds.inactive.contains(l),
                    author_kind: b.provenance.author_kind,
                    source: b.provenance.source.clone(),
                    seq_created: *ds
                        .created_seq
                        .get(l)
                        .context("lineage without creation seq")?,
                    seq_last: *ds.last_seq.get(l).context("lineage without last seq")?,
                })
            })
            .collect()
    }
}

/// One `blame` row (TASKS B6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlameLine {
    pub lineage_id: String,
    pub version_id: String,
    pub layer: Layer,
    /// `event` or `derived` (K-B1.1: a drop replay derived for a log without
    /// layer information)
    pub layer_source: LayerSource,
    pub inactive: bool,
    pub author_kind: AuthorKind,
    pub source: String,
    pub seq_created: u64,
    pub seq_last: u64,
}
