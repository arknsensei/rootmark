//! Declarative promotion rules (P2-P/P1; GOAL P2-P 2, K-Q.1 (2)):
//! `[[promotion.rules]]` of `.rootmark/policy.toml`, loaded and checked by
//! the kernel's recipe loader (`rootmarkcore::PromotionRule`), matched here.
//!
//! The matcher runs only in `propose` (MCP, HTTP, `rootmark propose`): after
//! the call's `SnapshotCommitted`, every lineage the call wrote — its live
//! version is new in this call — that is a live, active `proposal` is
//! matched in lineage-id order; the first rule whose actor (exact), source
//! (exact, when set) and path prefix cover the call wins, and that lineage
//! gets one `Promoted` event through the kernel's hook (`RuleHook`):
//! `meta.actor = {kind: "policy", id: <rule id>}`, `meta.reason =
//! "rule:<rule id>"`, `meta.policy = "rules"`, `meta.policy_version` the
//! recipe's `[promotion] version`, `from = "proposal"`, `to = "canonical"`.
//! The kernel decides nothing (spec §7): it writes the event the hook
//! allows. Nothing else runs the matcher: an external change (the watcher,
//! attested or not), a late attestation and a manual `promote` never do, so
//! `unverified` content and an unattested file write are never promoted by a
//! rule.

use anyhow::{bail, Context, Result};
use rootmarkcore::{
    Actor, ActorKind, Decision, Event, Layer, PromotionPolicy, PromotionRequest, PromotionRule,
    TransitionKind, Vault,
};

/// The hook name of a rule's promotion (`Promoted.meta.policy`).
pub const RULES_POLICY: &str = "rules";

/// The kernel hook of one rule (spec §7: the kernel exposes a hook, the
/// policy decides). It allows exactly the request the matcher makes for
/// that rule — the rule's policy actor, its reason, `proposal → canonical`
/// — and nothing else (never `unverified`, never another actor).
pub struct RuleHook<'a> {
    pub rule: &'a PromotionRule,
    pub version: &'a str,
}

impl PromotionPolicy for RuleHook<'_> {
    fn name(&self) -> &str {
        RULES_POLICY
    }
    fn version(&self) -> &str {
        self.version
    }
    fn decide(&self, req: &PromotionRequest) -> Decision {
        let ok = req.kind == TransitionKind::Promote
            && req.actor.kind == ActorKind::Policy
            && req.actor.id == self.rule.id
            && req.reason == reason_of(self.rule)
            && req.from == Layer::Proposal
            && req.to == Layer::Canonical;
        if ok {
            Decision::Allow
        } else {
            Decision::Deny(format!(
                "rule {}: only its own proposal → canonical promotion",
                self.rule.id
            ))
        }
    }
}

/// `Promoted.meta.reason` of a rule's promotion.
pub fn reason_of(rule: &PromotionRule) -> String {
    format!("rule:{}", rule.id)
}

/// The first rule (recipe order) covering a `propose` by `actor` with
/// `source` into `doc_path`; only an agent's call can match.
pub fn first_match<'a>(
    rules: &'a [PromotionRule],
    actor: &Actor,
    source: &str,
    doc_path: &str,
) -> Option<&'a PromotionRule> {
    if actor.kind != ActorKind::Agent {
        return None;
    }
    rules.iter().find(|r| r.matches(actor, source, doc_path))
}

/// Promotes, by `rule`, the lineages `wrote` names (the call's own
/// lineages) that are live, active `proposal`s of `doc_path`, in lineage-id
/// order; returns their `Promoted` events.
pub fn promote_written(
    v: &mut Vault,
    rule: &PromotionRule,
    version: &str,
    doc_path: &str,
    mut wrote: Vec<String>,
) -> Result<Vec<Event>> {
    wrote.sort();
    wrote.dedup();
    let hook = RuleHook { rule, version };
    let actor = Actor::new(ActorKind::Policy, &rule.id)?;
    let reason = reason_of(rule);
    let mut out = Vec::new();
    for l in wrote {
        let live_here = v
            .doc_state(doc_path)
            .is_some_and(|s| s.live.contains_key(&l));
        if !live_here || v.layer_of(&l)? != Layer::Proposal || v.is_inactive(&l)? {
            continue;
        }
        let req = v.request(
            TransitionKind::Promote,
            &l,
            Layer::Canonical,
            actor.clone(),
            &reason,
        )?;
        let ev = v
            .promote(&req, &hook)
            .with_context(|| format!("rule {}: promote {l}", rule.id))?;
        out.push(ev);
    }
    if out.iter().any(|e| e.meta["policy_version"] != version) {
        bail!("rule {}: a promotion without the recipe's version", rule.id);
    }
    Ok(out)
}
