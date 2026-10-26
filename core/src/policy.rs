//! Policy recipe `.rootmark/policy.toml` (PACKAGES: phase-1 policy plugins
//! are declarative recipes with parameters inside the kernel bounds).
//!
//! ```toml
//! promotion = "human_only"          # "deny" | "human_only"; or the table
//!                                   # [promotion] policy = "human_only"
//!
//! # P2-P/P1: declarative promotion rules (only with the table form)
//! # [promotion]
//! # policy = "human_only"            # manual promotion, unchanged
//! # version = "2026-10-25"           # required with rules: Promoted.meta.policy_version
//! # [[promotion.rules]]
//! # id = "official"                  # [a-z0-9_-]{1,64}, unique
//! # actor = "agent:collector"        # exact match; agent:<id> only
//! # source = "collector"             # optional; propose's source, exact match
//! # path_prefix = "sources/official" # vault-relative (spec §8's filter rule)
//! # to = "canonical"                 # the only value
//!
//! [version_trigger]
//! idle_ms = 30000                   # >= 0
//! min_sim_change = 0.2              # in [0, 1]
//!
//! [journal]
//! retention_hours = 168             # >= 24
//!
//! # P2-R/R1: proposal decay rules (optional; run only by `rootmark prune`
//! # and POST /v1/prune, never by `serve` on its own)
//! # [prune]
//! # version = "2026-10-26"           # required with rules: Demoted.meta.policy_version
//! # [[prune.rules]]
//! # id = "stale"                     # [a-z0-9_-]{1,64}, unique
//! # layer = "proposal"               # "proposal" | "unverified" (never canonical)
//! # idle_checkpoints = 10            # >= 1: checkpoints created since the lineage's last event
//! # path_prefix = ["agents/"]        # optional; vault-relative, NFC (an NFD spelling is refused)
//!
//! # P2-R/R2: retention of traces and checkpoints (optional; without it
//! # nothing is archived). The bound is the oldest of the last K checkpoints
//! # [retention]
//! # keep_checkpoints = 5             # >= 1
//! ```
//!
//! The optional `[server]`, `[index]` (P1-D) and `[identity]` (P1-D.1,
//! K-D1.3) tables belong to the `server` package: the kernel keeps them as
//! written (`Policy::server`, `Policy::index`, `Policy::identity`) and does
//! not interpret them; `server` validates them.
//!
//! The optional `[edit]` table (P1-D.1, K-D1.1) chooses the edit policy of
//! agent writes: `policy = "append_only"` (also when the table is absent)
//! or `"unrestricted"`, which only an explicit table gives.
//!
//! The optional `[attestation]` table (spec 2.0.0 §7.1, P2-0) chooses how
//! an external change no human-channel client attested is ingested:
//! `unattested = "unverified"` (also when absent) or `"canonical"` (the
//! 1.0.3 behaviour), and the window `window_s` (seconds, 0.1..=60, default
//! 2.0) within which an attestation matches a change.
//!
//! A present file must hold every required field (`[attestation]`'s two
//! fields are optional, with the defaults above); unknown fields, unknown policies
//! and values outside the kernel bounds are load errors — the loader never
//! falls back to the built-in values for a file that exists. Without a file
//! the built-in recipe applies (`Policy::builtin`, promotion `deny`), and
//! `Policy::source` says so.

use crate::governance::{
    Actor, ActorKind, AppendOnly, Deny, EditPolicy, HumanOnly, PromotionPolicy, Unrestricted,
};
use crate::journal::{IdleOrDistance, JournalRetention};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Deny,
    HumanOnly,
}

impl Promotion {
    pub fn parse(s: &str) -> Result<Promotion> {
        Ok(match s {
            "deny" => Promotion::Deny,
            "human_only" => Promotion::HumanOnly,
            _ => bail!("unknown promotion policy {s:?} (deny|human_only)"),
        })
    }

    pub fn hook(self) -> &'static dyn PromotionPolicy {
        match self {
            Promotion::Deny => &Deny,
            Promotion::HumanOnly => &HumanOnly,
        }
    }
}

/// The edit policy of agent writes (K-D1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    AppendOnly,
    Unrestricted,
}

impl Edit {
    pub fn parse(s: &str) -> Result<Edit> {
        Ok(match s {
            "append_only" => Edit::AppendOnly,
            "unrestricted" => Edit::Unrestricted,
            _ => bail!("unknown edit policy {s:?} (append_only|unrestricted)"),
        })
    }

    pub fn hook(self) -> &'static dyn EditPolicy {
        match self {
            Edit::AppendOnly => &AppendOnly,
            Edit::Unrestricted => &Unrestricted,
        }
    }
}

/// Where a policy came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicySource {
    File(PathBuf),
    /// no `policy.toml` in the vault
    Builtin,
}

/// One declarative promotion rule (P2-P/P1, `[[promotion.rules]]`): the
/// `server` matches it against the lineages a `propose` call wrote; the
/// kernel only loads and checks it (it decides nothing, spec §7).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PromotionRule {
    /// `[a-z0-9_-]{1,64}`, unique in the recipe
    pub id: String,
    /// `agent:<id>`, matched exactly against the caller's actor
    pub actor: Actor,
    /// matched exactly against `propose`'s source, when given
    pub source: Option<String>,
    /// normalized (`crate::pathprefix`)
    pub path_prefix: String,
    /// always `canonical`
    pub to: crate::model::Layer,
}

impl PromotionRule {
    /// Whether the rule covers a `propose` by `actor` with `source` into
    /// `doc_path` (exact actor, exact source when set, path under the prefix).
    /// The agent's name is its identity, not its version: an MCP client's
    /// actor id and source are `<clientInfo.name>/<clientInfo.version>`
    /// (K-D1.3), so `agent:collector` covers `agent:collector` (CLI
    /// `--source collector`) and `agent:collector/<version>` — one more
    /// component without `/` — and nothing else (`collector/x/1`,
    /// `collector2/1`, `collectorx` do not match). P2-P/P4, ASSUMPTIONS P4-1.
    pub fn matches(&self, actor: &Actor, source: &str, doc_path: &str) -> bool {
        actor.kind == self.actor.kind
            && same_agent(&self.actor.id, &actor.id)
            && self.source.as_deref().is_none_or(|s| same_agent(s, source))
            && crate::pathprefix::matches(&self.path_prefix, doc_path)
    }
}

/// `got` is `want`, or — when `want` is a bare name (no `/`) —
/// `want/<version>` with a version that is not empty and holds no `/` (an
/// MCP client's `<name>/<version>`). A rule written with a version
/// (`agent:x/1.0`) matches exactly (P5 review 10).
fn same_agent(want: &str, got: &str) -> bool {
    got == want
        || (!want.contains('/')
            && got
                .strip_prefix(want)
                .and_then(|r| r.strip_prefix('/'))
                .is_some_and(|v| !v.is_empty() && !v.contains('/')))
}

/// The rule id form: `[a-z0-9_-]{1,64}`.
fn rule_id_ok(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// `[[promotion.rules]]` entry `i` (1-based in messages).
fn rule_of(i: usize, v: &toml::Value) -> Result<PromotionRule> {
    let at = format!("[[promotion.rules]] #{i}");
    let Some(t) = v.as_table() else {
        bail!("{at}: must be a table");
    };
    let at = match t.get("id").and_then(|v| v.as_str()) {
        Some(id) => format!("{at} (id {id:?})"),
        None => at,
    };
    if let Some(k) = t
        .keys()
        .find(|k| !matches!(k.as_str(), "id" | "actor" | "source" | "path_prefix" | "to"))
    {
        bail!("{at}: unknown field `{k}` (id, actor, source, path_prefix, to)");
    }
    let s = |k: &str| -> Result<Option<String>> {
        match t.get(k) {
            None => Ok(None),
            Some(toml::Value::String(x)) => Ok(Some(x.clone())),
            Some(_) => bail!("{at}: {k} must be a string"),
        }
    };
    let need =
        |k: &str| -> Result<String> { s(k)?.with_context(|| format!("{at}: missing field `{k}`")) };
    let id = need("id")?;
    if !rule_id_ok(&id) {
        bail!("{at}: id {id:?} must match [a-z0-9_-]{{1,64}}");
    }
    let actor_s = need("actor")?;
    let actor = match Actor::parse(&actor_s) {
        Ok(a) if a.kind == ActorKind::Agent && a.id == a.id.trim() => a,
        _ => bail!(
            "{at}: actor {actor_s:?} must be agent:<id> (a rule never promotes for a human, a policy or unattested_fs)"
        ),
    };
    let source = s("source")?;
    if source.as_deref().is_some_and(|x| x.trim().is_empty()) {
        bail!("{at}: source must not be empty (omit it to match any source)");
    }
    let raw = need("path_prefix")?;
    // a recipe is written once: an NFD (or any non-NFC) spelling of the
    // prefix is refused, not folded (GOAL P2-P 2: "NFD varyantı reddedilir";
    // the search filter of P2-L/L3 folds it, a query being typed each time)
    let path_prefix = recipe_prefix(&at, &raw)?;
    let to = need("to")?;
    if to != "canonical" {
        bail!("{at}: to {to:?} must be \"canonical\" (the only value)");
    }
    Ok(PromotionRule {
        id,
        actor,
        source,
        path_prefix,
        to: crate::model::Layer::Canonical,
    })
}

/// One proposal decay rule (P2-R/R1, `[[prune.rules]]`; K19-T, K-Q.1 (5)):
/// the `server` matches it in `rootmark prune` / `POST /v1/prune` only; the
/// kernel loads and checks it and writes the `Demoted` (`meta.prune`) its
/// hook allows (it decides nothing, spec §7). `canonical` is never a rule's
/// layer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PruneRule {
    /// `[a-z0-9_-]{1,64}`, unique in the recipe
    pub id: String,
    /// `proposal` or `unverified`
    pub layer: crate::model::Layer,
    /// checkpoints created after the lineage's last event, at least (>= 1)
    pub idle_checkpoints: u64,
    /// normalized prefixes (`crate::pathprefix`); empty: every document
    pub path_prefix: Vec<String>,
}

impl PruneRule {
    /// Whether the rule covers a live lineage of `layer` in `doc_path`.
    pub fn covers(&self, layer: crate::model::Layer, doc_path: &str) -> bool {
        layer == self.layer
            && (self.path_prefix.is_empty()
                || self
                    .path_prefix
                    .iter()
                    .any(|p| crate::pathprefix::matches(p, doc_path)))
    }
}

/// A recipe path prefix: NFC (an NFD spelling is refused, P2-B rule of
/// recipes), then normalized.
fn recipe_prefix(at: &str, raw: &str) -> Result<String> {
    use unicode_normalization::UnicodeNormalization;
    if raw.nfc().collect::<String>() != raw {
        bail!("{at}: path_prefix {raw:?} is not in Unicode NFC (an NFD variant is refused; write the prefix in NFC)");
    }
    crate::pathprefix::normalize(raw).with_context(|| at.to_string())
}

/// `[[prune.rules]]` entry `i` (1-based in messages).
fn prune_rule_of(i: usize, v: &toml::Value) -> Result<PruneRule> {
    let at = format!("[[prune.rules]] #{i}");
    let Some(t) = v.as_table() else {
        bail!("{at}: must be a table");
    };
    let at = match t.get("id").and_then(|v| v.as_str()) {
        Some(id) => format!("{at} (id {id:?})"),
        None => at,
    };
    if let Some(k) = t.keys().find(|k| {
        !matches!(
            k.as_str(),
            "id" | "layer" | "idle_checkpoints" | "path_prefix"
        )
    }) {
        bail!("{at}: unknown field `{k}` (id, layer, idle_checkpoints, path_prefix)");
    }
    let id = match t.get("id") {
        Some(toml::Value::String(x)) => x.clone(),
        Some(_) => bail!("{at}: id must be a string"),
        None => bail!("{at}: missing field `id`"),
    };
    if !rule_id_ok(&id) {
        bail!("{at}: id {id:?} must match [a-z0-9_-]{{1,64}}");
    }
    let layer = match t.get("layer") {
        Some(toml::Value::String(x)) => match x.as_str() {
            "proposal" => crate::model::Layer::Proposal,
            "unverified" => crate::model::Layer::Unverified,
            "canonical" => bail!("{at}: layer \"canonical\" is never pruned (a rule prunes proposal or unverified lineages)"),
            other => bail!("{at}: layer {other:?} must be \"proposal\" or \"unverified\""),
        },
        Some(_) => bail!("{at}: layer must be a string"),
        None => bail!("{at}: missing field `layer`"),
    };
    let idle_checkpoints = match t.get("idle_checkpoints") {
        Some(toml::Value::Integer(n)) if *n >= 1 => *n as u64,
        Some(toml::Value::Integer(n)) => {
            bail!("{at}: idle_checkpoints = {n}: the kernel minimum is 1")
        }
        Some(_) => bail!("{at}: idle_checkpoints must be an integer"),
        None => bail!("{at}: missing field `idle_checkpoints`"),
    };
    let path_prefix = match t.get("path_prefix") {
        None => Vec::new(),
        Some(toml::Value::Array(a)) if !a.is_empty() => {
            let mut out = Vec::new();
            for x in a {
                let Some(x) = x.as_str() else {
                    bail!("{at}: path_prefix must be a list of strings");
                };
                out.push(recipe_prefix(&at, x)?);
            }
            out
        }
        Some(toml::Value::Array(_)) => {
            bail!("{at}: path_prefix is empty (omit it for every document)")
        }
        Some(_) => bail!("{at}: path_prefix must be a list of strings"),
    };
    Ok(PruneRule {
        id,
        layer,
        idle_checkpoints,
        path_prefix,
    })
}

/// `[prune]`: `version` and `rules` (P2-R/R1).
fn prune_of(t: Option<&toml::Table>) -> Result<(Option<String>, Vec<PruneRule>)> {
    let Some(t) = t else {
        return Ok((None, Vec::new()));
    };
    if let Some(k) = t
        .keys()
        .find(|k| !matches!(k.as_str(), "version" | "rules"))
    {
        bail!("[prune]: unknown field `{k}` (version, rules)");
    }
    let version = match t.get("version") {
        None => None,
        Some(toml::Value::String(v)) if !v.trim().is_empty() => Some(v.clone()),
        Some(toml::Value::String(_)) => bail!("[prune] version must not be empty"),
        Some(_) => bail!("[prune] version must be a string"),
    };
    let rules = match t.get("rules") {
        None => Vec::new(),
        Some(toml::Value::Array(a)) => a
            .iter()
            .enumerate()
            .map(|(i, v)| prune_rule_of(i + 1, v))
            .collect::<Result<Vec<_>>>()?,
        Some(_) => bail!("[prune] rules must be an array of tables ([[prune.rules]])"),
    };
    let mut seen = std::collections::BTreeSet::new();
    for r in &rules {
        if !seen.insert(r.id.as_str()) {
            bail!(
                "[[prune.rules]]: id {:?} is given twice (ids are unique)",
                r.id
            );
        }
    }
    if !rules.is_empty() && version.is_none() {
        bail!("[prune]: missing field `version` (required with [[prune.rules]]; the policy_version of a rule's prune)");
    }
    Ok((version, rules))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub promotion: Promotion,
    /// `[promotion] version` (P2-P/P1): required with rules, the
    /// `policy_version` of a rule's `Promoted`
    pub promotion_version: Option<String>,
    /// `[[promotion.rules]]` (P2-P/P1), in recipe order (the first match wins)
    pub promotion_rules: Vec<PromotionRule>,
    pub trigger: IdleOrDistance,
    pub retention: JournalRetention,
    pub source: PolicySource,
    /// `[server]` as written, for the `server` package (P1-D)
    pub server: Option<toml::Table>,
    /// `[index]` as written, for the `server` package (P1-D)
    pub index: Option<toml::Table>,
    /// `[edit] policy` (K-D1.1); `AppendOnly` when absent
    pub edit: Edit,
    /// `[identity]` as written, for the `server` package (K-D1.3)
    pub identity: Option<toml::Table>,
    /// `[attestation]` (spec 2.0.0 §7.1)
    pub attestation: Attestation,
    /// `[prune] version` (P2-R/R1): required with rules, the
    /// `policy_version` of a rule's `Demoted`
    pub prune_version: Option<String>,
    /// `[[prune.rules]]` (P2-R/R1), in recipe order (the first covering
    /// rule decides)
    pub prune_rules: Vec<PruneRule>,
    /// `[retention] keep_checkpoints` (P2-R/R2): `None` without the table —
    /// traces and checkpoints are kept without bound
    pub keep_checkpoints: Option<u64>,
}

/// `[retention] keep_checkpoints = K` (P2-R/R2; K >= 1, the only field).
fn retention_of(t: Option<&toml::Table>) -> Result<Option<u64>> {
    let Some(t) = t else {
        return Ok(None);
    };
    if let Some(k) = t.keys().find(|k| k.as_str() != "keep_checkpoints") {
        bail!("[retention]: unknown field `{k}` (keep_checkpoints)");
    }
    match t.get("keep_checkpoints") {
        Some(toml::Value::Integer(n)) if *n >= 1 => Ok(Some(*n as u64)),
        Some(toml::Value::Integer(n)) => {
            bail!("[retention] keep_checkpoints = {n}: the kernel minimum is 1")
        }
        Some(_) => bail!("[retention] keep_checkpoints must be an integer"),
        None => bail!("[retention]: missing field `keep_checkpoints`"),
    }
}

/// `[attestation]` (spec 2.0.0 §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attestation {
    /// `unattested = "unverified"` (true, the default) or `"canonical"`
    pub unverified: bool,
    /// `window_s` in milliseconds
    pub window_ms: u64,
    /// no `[attestation]` table (or no recipe): `unattested` is the kernel's
    /// reading of a recipe without the setting, not a choice (K-P2L-T1;
    /// `rootmark migrate` adds the table, migration `policy_attestation_1`)
    pub implicit: bool,
}

impl Default for Attestation {
    fn default() -> Self {
        Attestation {
            unverified: true,
            window_ms: crate::attest::DEFAULT_WINDOW_MS,
            implicit: true,
        }
    }
}

impl Attestation {
    pub fn unattested(&self) -> &'static str {
        if self.unverified {
            "unverified"
        } else {
            "canonical"
        }
    }

    pub fn window_s(&self) -> f64 {
        self.window_ms as f64 / 1000.0
    }
}

/// `[attestation] unattested = "unverified"|"canonical"`, `window_s` in
/// 0.1..=60 (each optional).
fn attestation_of(t: Option<&toml::Table>) -> Result<Attestation> {
    let mut a = Attestation::default();
    let Some(t) = t else {
        return Ok(a);
    };
    a.implicit = false;
    for (k, v) in t {
        match k.as_str() {
            "unattested" => {
                a.unverified = match v.as_str() {
                    Some("unverified") => true,
                    Some("canonical") => false,
                    _ => bail!("[attestation] unattested must be \"unverified\" or \"canonical\""),
                }
            }
            "window_s" => {
                let w = match v {
                    toml::Value::Float(f) => *f,
                    toml::Value::Integer(i) => *i as f64,
                    _ => bail!("[attestation] window_s must be a number"),
                };
                if !(0.1..=60.0).contains(&w) {
                    bail!("[attestation] window_s = {w} outside [0.1, 60]");
                }
                a.window_ms = (w * 1000.0).round() as u64;
            }
            _ => bail!("[attestation]: unknown field `{k}` (unattested, window_s)"),
        }
    }
    Ok(a)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    promotion: RawPromotion,
    version_trigger: RawTrigger,
    journal: RawJournal,
    server: Option<toml::Table>,
    index: Option<toml::Table>,
    edit: Option<toml::Table>,
    identity: Option<toml::Table>,
    attestation: Option<toml::Table>,
    prune: Option<toml::Table>,
    retention: Option<toml::Table>,
}

/// `[edit] policy = "<policy>"` (the only field).
fn edit_of(t: Option<&toml::Table>) -> Result<Edit> {
    let Some(t) = t else {
        return Ok(Edit::AppendOnly);
    };
    if let Some(k) = t.keys().find(|k| k.as_str() != "policy") {
        bail!("[edit]: unknown field `{k}`, expected `policy`");
    }
    match t.get("policy") {
        Some(toml::Value::String(n)) => Edit::parse(n),
        Some(_) => bail!("[edit] policy must be a string"),
        None => bail!("[edit]: missing field `policy`"),
    }
}

/// `promotion = "<policy>"` or `[promotion] policy = "<policy>"` (P1-D).
#[derive(Deserialize)]
#[serde(untagged)]
enum RawPromotion {
    Name(String),
    Table(toml::Table),
}

/// `[promotion]`: the manual policy's name, `version` and `rules` (P2-P/P1).
type PromotionParts<'a> = (&'a str, Option<String>, Vec<PromotionRule>);

impl RawPromotion {
    fn parts(&self) -> Result<PromotionParts<'_>> {
        match self {
            RawPromotion::Name(n) => Ok((n, None, Vec::new())),
            RawPromotion::Table(t) => {
                if let Some(k) = t
                    .keys()
                    .find(|k| !matches!(k.as_str(), "policy" | "version" | "rules"))
                {
                    bail!("[promotion]: unknown field `{k}` (policy, version, rules)");
                }
                let name = match t.get("policy") {
                    Some(toml::Value::String(n)) => n.as_str(),
                    Some(_) => bail!("[promotion] policy must be a string"),
                    None => bail!("[promotion]: missing field `policy`"),
                };
                let version = match t.get("version") {
                    None => None,
                    Some(toml::Value::String(v)) if !v.trim().is_empty() => Some(v.clone()),
                    Some(toml::Value::String(_)) => bail!("[promotion] version must not be empty"),
                    Some(_) => bail!("[promotion] version must be a string"),
                };
                let rules = match t.get("rules") {
                    None => Vec::new(),
                    Some(toml::Value::Array(a)) => a
                        .iter()
                        .enumerate()
                        .map(|(i, v)| rule_of(i + 1, v))
                        .collect::<Result<Vec<_>>>()?,
                    Some(_) => {
                        bail!("[promotion] rules must be an array of tables ([[promotion.rules]])")
                    }
                };
                let mut seen = std::collections::BTreeSet::new();
                for r in &rules {
                    if !seen.insert(r.id.as_str()) {
                        bail!(
                            "[[promotion.rules]]: id {:?} is given twice (ids are unique)",
                            r.id
                        );
                    }
                }
                // P5 review 2 (K-P2P.5): under `deny` no one can demote, so a
                // rule's promotion could never be taken back
                if !rules.is_empty() && name == "deny" {
                    bail!("[promotion]: policy = \"deny\" with [[promotion.rules]]: a rule's promotion could not be demoted (deny refuses every transition); use policy = \"human_only\"");
                }
                if !rules.is_empty() && version.is_none() {
                    bail!("[promotion]: missing field `version` (required with [[promotion.rules]]; the policy_version of a rule's promotion)");
                }
                Ok((name, version, rules))
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTrigger {
    idle_ms: i64,
    min_sim_change: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJournal {
    retention_hours: i64,
}

impl Policy {
    /// The recipe without a file: promotion `deny`, version after 30 s idle
    /// or a 0.2 change, journal kept 168 h.
    pub fn builtin() -> Policy {
        Policy {
            promotion: Promotion::Deny,
            trigger: IdleOrDistance::new(30_000, 0.2).expect("built-in trigger within bounds"),
            retention: JournalRetention::new(168).expect("built-in retention within bounds"),
            promotion_version: None,
            promotion_rules: Vec::new(),
            source: PolicySource::Builtin,
            server: None,
            index: None,
            edit: Edit::AppendOnly,
            identity: None,
            attestation: Attestation::default(),
            prune_version: None,
            prune_rules: Vec::new(),
            keep_checkpoints: None,
        }
    }

    pub fn path(vault_root: &Path) -> PathBuf {
        crate::meta::path(vault_root).join("policy.toml")
    }

    /// `<vault>/.rootmark/policy.toml`, or the built-in recipe if absent.
    pub fn load(vault_root: &Path) -> Result<Policy> {
        crate::meta::check(vault_root)?;
        let p = Self::path(vault_root);
        // only a path that is absent altogether means "no recipe"; a broken
        // link or unreadable entry is an error (B6 review)
        if std::fs::symlink_metadata(&p).is_err() {
            return Ok(Self::builtin());
        }
        let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        Self::parse(&text, PolicySource::File(p.clone()))
            .with_context(|| format!("policy {} rejected", p.display()))
    }

    pub fn parse(text: &str, source: PolicySource) -> Result<Policy> {
        let raw: RawPolicy = toml::from_str(text).context("invalid policy.toml")?;
        let (name, promotion_version, promotion_rules) = raw.promotion.parts()?;
        let (prune_version, prune_rules) = prune_of(raw.prune.as_ref())?;
        Ok(Policy {
            keep_checkpoints: retention_of(raw.retention.as_ref())?,
            prune_version,
            prune_rules,
            promotion: Promotion::parse(name)?,
            promotion_version,
            promotion_rules,
            trigger: IdleOrDistance::new(
                raw.version_trigger.idle_ms,
                raw.version_trigger.min_sim_change,
            )?,
            retention: JournalRetention::new(raw.journal.retention_hours)?,
            source,
            edit: edit_of(raw.edit.as_ref())?,
            attestation: attestation_of(raw.attestation.as_ref())?,
            server: raw.server,
            index: raw.index,
            identity: raw.identity,
        })
    }

    pub fn to_json(&self) -> serde_json::Value {
        let hook = self.promotion.hook();
        let mut out = serde_json::json!({
            "source": match &self.source {
                PolicySource::File(p) => p.display().to_string(),
                PolicySource::Builtin => "builtin".to_string(),
            },
            "promotion": hook.name(),
            "promotion_version": hook.version(),
            "version_trigger": {
                "idle_ms": self.trigger.idle_ms(),
                "min_sim_change": self.trigger.min_sim_change(),
            },
            "journal": {"retention_hours": self.retention.hours()},
            "edit": self.edit.hook().name(),
            "edit_version": self.edit.hook().version(),
        });
        // P2-P/P1: only a recipe with rules shows them (the output of every
        // other recipe is unchanged)
        if !self.promotion_rules.is_empty() {
            out["promotion_rules"] = serde_json::json!({
                "version": self.promotion_version,
                "rules": self.promotion_rules.iter().map(|r| serde_json::json!({
                    "id": r.id,
                    "actor": format!("agent:{}", r.actor.id),
                    "source": r.source,
                    "path_prefix": r.path_prefix,
                    "to": r.to.as_str(),
                })).collect::<Vec<_>>(),
            });
        }
        // P2-R/R2: likewise only a recipe with `[retention]` shows it
        if let Some(k) = self.keep_checkpoints {
            out["retention"] = serde_json::json!({"keep_checkpoints": k});
        }
        // P2-R/R1: likewise only a recipe with prune rules shows them
        if !self.prune_rules.is_empty() {
            out["prune_rules"] = serde_json::json!({
                "version": self.prune_version,
                "rules": self.prune_rules,
            });
        }
        out
    }
}
