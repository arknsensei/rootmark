//! Policy recipe `.rootmark/policy.toml` (PACKAGES: phase-1 policy plugins
//! are declarative recipes with parameters inside the kernel bounds).
//!
//! ```toml
//! promotion = "human_only"          # "deny" | "human_only"; or the table
//!                                   # [promotion] policy = "human_only"
//!
//! [version_trigger]
//! idle_ms = 30000                   # >= 0
//! min_sim_change = 0.2              # in [0, 1]
//!
//! [journal]
//! retention_hours = 168             # >= 24
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

use crate::governance::{AppendOnly, Deny, EditPolicy, HumanOnly, PromotionPolicy, Unrestricted};
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

#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub promotion: Promotion,
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
}

/// `[attestation]` (spec 2.0.0 §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attestation {
    /// `unattested = "unverified"` (true, the default) or `"canonical"`
    pub unverified: bool,
    /// `window_s` in milliseconds
    pub window_ms: u64,
}

impl Default for Attestation {
    fn default() -> Self {
        Attestation {
            unverified: true,
            window_ms: crate::attest::DEFAULT_WINDOW_MS,
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

impl RawPromotion {
    fn name(&self) -> Result<&str> {
        match self {
            RawPromotion::Name(n) => Ok(n),
            RawPromotion::Table(t) => {
                if let Some(k) = t.keys().find(|k| k.as_str() != "policy") {
                    bail!("[promotion]: unknown field `{k}`, expected `policy`");
                }
                match t.get("policy") {
                    Some(toml::Value::String(n)) => Ok(n),
                    Some(_) => bail!("[promotion] policy must be a string"),
                    None => bail!("[promotion]: missing field `policy`"),
                }
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
            source: PolicySource::Builtin,
            server: None,
            index: None,
            edit: Edit::AppendOnly,
            identity: None,
            attestation: Attestation::default(),
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
        Ok(Policy {
            promotion: Promotion::parse(raw.promotion.name()?)?,
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
        serde_json::json!({
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
        })
    }
}
