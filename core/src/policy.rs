//! Policy recipe `.mdkernel/policy.toml` (PACKAGES: phase-1 policy plugins
//! are declarative recipes with parameters inside the kernel bounds).
//!
//! ```toml
//! promotion = "human_only"          # "deny" | "human_only"
//!
//! [version_trigger]
//! idle_ms = 30000                   # >= 0
//! min_sim_change = 0.2              # in [0, 1]
//!
//! [journal]
//! retention_hours = 168             # >= 24
//! ```
//!
//! A present file must hold every field; unknown fields, unknown policies
//! and values outside the kernel bounds are load errors — the loader never
//! falls back to the built-in values for a file that exists. Without a file
//! the built-in recipe applies (`Policy::builtin`, promotion `deny`), and
//! `Policy::source` says so.

use crate::governance::{Deny, HumanOnly, PromotionPolicy};
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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    promotion: String,
    version_trigger: RawTrigger,
    journal: RawJournal,
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
        }
    }

    pub fn path(vault_root: &Path) -> PathBuf {
        vault_root.join(".mdkernel").join("policy.toml")
    }

    /// `<vault>/.mdkernel/policy.toml`, or the built-in recipe if absent.
    pub fn load(vault_root: &Path) -> Result<Policy> {
        let p = Self::path(vault_root);
        if !p.exists() {
            return Ok(Self::builtin());
        }
        let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        Self::parse(&text, PolicySource::File(p.clone()))
            .with_context(|| format!("policy {} rejected", p.display()))
    }

    pub fn parse(text: &str, source: PolicySource) -> Result<Policy> {
        let raw: RawPolicy = toml::from_str(text).context("invalid policy.toml")?;
        Ok(Policy {
            promotion: Promotion::parse(&raw.promotion)?,
            trigger: IdleOrDistance::new(
                raw.version_trigger.idle_ms,
                raw.version_trigger.min_sim_change,
            )?,
            retention: JournalRetention::new(raw.journal.retention_hours)?,
            source,
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
        })
    }
}
