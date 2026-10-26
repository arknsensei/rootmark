//! Proposal decay (P2-R/R1; GOAL P2-R 2, K19-T, K-Q.1 (5)): the
//! `[[prune.rules]]` of `.rootmark/policy.toml`, loaded and checked by the
//! kernel's recipe loader (`rootmarkcore::PruneRule`), matched here.
//!
//! The matcher runs only in an explicit prune (`rootmark prune`, `POST
//! /v1/prune`); `rootmark serve` never runs it on its own and MCP has no
//! prune. It decides from the log alone — never from the clock: for every
//! live, active lineage (documents in path order, lineages in id order) the
//! first rule (recipe order) whose layer is the lineage's layer and whose
//! path prefixes cover its document decides; the lineage is pruned when at
//! least `idle_checkpoints` checkpoints were created after its last event.
//! Each prune is one `Demoted` event through the kernel's hook
//! (`PruneHook`, `Vault::prune_inactive`): `meta.prune = true`, the layer
//! unchanged (`from = to`), `meta.actor = {kind: "policy", id: <rule id>}`,
//! `meta.reason = "rule:<rule id>"`, `meta.policy = "prune_rules"`,
//! `meta.policy_version` the recipe's `[prune] version`. `canonical` is
//! never a rule's layer (the loader refuses it) and the kernel refuses a
//! prune of it. A recipe without rules writes no event.

use anyhow::{bail, Context, Result};
use rootmarkcore::{
    Actor, ActorKind, Decision, Event, Layer, PromotionPolicy, PromotionRequest, PruneRule,
    TransitionKind, Vault,
};
use serde::Serialize;

/// The hook name of a rule's prune (`Demoted.meta.policy`).
pub const PRUNE_POLICY: &str = "prune_rules";

/// The kernel hook of one decay rule (spec §7: the kernel exposes a hook,
/// the policy decides). It allows exactly the request the matcher makes for
/// that rule — a prune by the rule's policy actor, its reason, the rule's
/// layer kept — and nothing else.
pub struct PruneHook<'a> {
    pub rule: &'a PruneRule,
    pub version: &'a str,
}

impl PromotionPolicy for PruneHook<'_> {
    fn name(&self) -> &str {
        PRUNE_POLICY
    }
    fn version(&self) -> &str {
        self.version
    }
    fn decide(&self, req: &PromotionRequest) -> Decision {
        let ok = req.kind == TransitionKind::Prune
            && req.actor.kind == ActorKind::Policy
            && req.actor.id == self.rule.id
            && req.reason == reason_of(self.rule)
            && req.from == self.rule.layer
            && req.to == self.rule.layer
            && req.from != Layer::Canonical;
        if ok {
            Decision::Allow
        } else {
            Decision::Deny(format!(
                "prune rule {}: only its own prune of a {} lineage",
                self.rule.id,
                self.rule.layer.as_str()
            ))
        }
    }
}

/// `Demoted.meta.reason` of a rule's prune.
pub fn reason_of(rule: &PruneRule) -> String {
    format!("rule:{}", rule.id)
}

/// One lineage a rule prunes (the plan, and what was done).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decay {
    pub doc_path: String,
    pub lineage_id: String,
    pub layer: Layer,
    pub rule: String,
    /// seq of the lineage's last event
    pub last_seq: u64,
    /// checkpoints created after it
    pub idle_checkpoints: u64,
}

/// The lineages the recipe's rules prune now, in documents' path order and
/// lineage-id order; nothing is written.
pub fn plan(v: &Vault, rules: &[PruneRule]) -> Result<Vec<Decay>> {
    let mut out = Vec::new();
    if rules.is_empty() {
        return Ok(out);
    }
    let cps = v.checkpoint_seqs();
    for doc in v.doc_paths() {
        let st = v.doc_state(&doc).context("document without state")?;
        let mut lineages: Vec<&String> = st.live.keys().collect();
        lineages.sort();
        for l in lineages {
            if st.inactive.contains(l) {
                continue;
            }
            let layer = *st.layer.get(l).context("live lineage without layer")?;
            if layer == Layer::Canonical {
                continue;
            }
            let Some(rule) = rules.iter().find(|r| r.covers(layer, &doc)) else {
                continue;
            };
            let last = *st
                .last_seq
                .get(l)
                .context("live lineage without last seq")?;
            let idle = cps.iter().filter(|&&s| s > last).count() as u64;
            if idle >= rule.idle_checkpoints {
                out.push(Decay {
                    doc_path: doc.clone(),
                    lineage_id: l.clone(),
                    layer,
                    rule: rule.id.clone(),
                    last_seq: last,
                    idle_checkpoints: idle,
                });
            }
        }
    }
    Ok(out)
}

/// Writes the plan: one `Demoted` (`meta.prune`) per lineage through the
/// rule's hook (`source`: the channel's label, `meta.source`); returns the
/// events in order.
pub fn apply(
    v: &mut Vault,
    rules: &[PruneRule],
    version: Option<&str>,
    plan: &[Decay],
    source: Option<&str>,
) -> Result<Vec<Event>> {
    let mut out = Vec::new();
    if plan.is_empty() {
        return Ok(out);
    }
    let Some(version) = version else {
        bail!("[prune]: rules without a version (the recipe loader requires one)");
    };
    for d in plan {
        let rule = rules
            .iter()
            .find(|r| r.id == d.rule)
            .with_context(|| format!("prune rule {} is not in the recipe", d.rule))?;
        let hook = PruneHook { rule, version };
        let actor = Actor::new(ActorKind::Policy, &rule.id)?;
        let ev = v
            .prune_inactive_from(&d.lineage_id, actor, &reason_of(rule), &hook, source)
            .with_context(|| format!("prune rule {}: prune {}", rule.id, d.lineage_id))?;
        out.push(ev);
    }
    Ok(out)
}

/// What `rootmark prune` says when the recipe has no `[retention]`.
pub const NO_RETENTION: &str = "no [retention] in policy.toml: traces and checkpoints are kept without bound (nothing archived); location records of versions no checkpoint names are pruned";

/// The vault's explicit prune (`rootmark prune <vault>`, `POST /v1/prune`;
/// GOAL P2-R 3), in this order: the recipe's decay rules (`Demoted` with
/// `meta.prune`, `source` its `meta.source`), the retention of traces,
/// checkpoint files and location records (`[retention]`,
/// `rootmarkcore::retention`), the raw journal (`journal prune` at
/// `now_ms`, default the clock — the only part that reads a time).
/// `dry_run`: the same plan, no file changes. The same vault, recipe and
/// call give the same bytes.
pub fn run(
    vault: &std::path::Path,
    dry_run: bool,
    now_ms: Option<u64>,
    source: Option<&str>,
) -> Result<serde_json::Value> {
    let policy = rootmarkcore::Policy::load(vault)?;
    let now = match now_ms {
        Some(t) => t,
        None => u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis(),
        )?,
    };
    let mut v = Vault::open(vault)?;
    let before = v.events_hash()?;
    let decays = plan(&v, &policy.prune_rules)?;
    let (events, retention, journal) = if dry_run {
        (
            Vec::new(),
            v.retention_plan(policy.keep_checkpoints)?,
            v.journal_prune_plan(now, policy.retention)?,
        )
    } else {
        let events = apply(
            &mut v,
            &policy.prune_rules,
            policy.prune_version.as_deref(),
            &decays,
            source,
        )?;
        let r = v.retention_plan(policy.keep_checkpoints)?;
        v.apply_retention(&r)?;
        let j = v.journal_prune(now, policy.retention)?;
        (events, r, j)
    };
    let mut ret = serde_json::to_value(&retention)?;
    if policy.keep_checkpoints.is_none() {
        ret["note"] = serde_json::json!(NO_RETENTION);
    }
    Ok(serde_json::json!({
        "dry_run": dry_run,
        "events_hash_before": before,
        "events_hash": v.events_hash()?,
        "rules": {
            "version": policy.prune_version,
            "rules": policy.prune_rules.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            "pruned": decays,
            "events": events.iter().map(|e| e.seq).collect::<Vec<_>>(),
        },
        "retention": ret,
        "journal": {
            "now_ms": now,
            "records_removed": journal.records_removed,
            "records_kept": journal.records_kept,
            "blobs_removed": journal.blobs_removed,
        },
    }))
}
