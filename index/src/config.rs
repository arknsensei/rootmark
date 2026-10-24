//! The index settings of a vault (P2-L/L1, L2; GOAL P2-L 2, 3): `[index]
//! tokenizer` and `[index] embedder` of `.rootmark/policy.toml`, read through
//! the kernel's recipe loader (`rootmarkcore::Policy::index`, the table as
//! written; the server validates its own keys of it). Each is optional; an
//! unknown value is an error, never a silent default.
//!
//! ```toml
//! [index]
//! tokenizer = "mdk_lower_alnum"   # or "uni_fold_tr" (P2-L/L1)
//! embedder = "minilm-l6-v2"       # or "multilingual-minilm-l12-v2" (P2-L/L2)
//! ```

use crate::tokenize::Variant;
use anyhow::{bail, Result};
use std::path::Path;

/// The conformance default of the lexical variant (P2-L/L1, L-10; P2-P/P0
/// a1): consulted only when the recipe names no `[index] tokenizer`, so that
/// the frozen C3 evaluator (whose scratch vaults have no recipe) can run its
/// cases with `uni_fold_tr` (`index/tests/p2_l_conformance.rs`). It is set
/// only through `set_conformance_default_tokenizer`, a library call: no
/// environment variable or setting of a released binary reaches it (P2-L
/// review finding (a); until P2-P it was `ROOTMARK_DEFAULT_TOKENIZER`). The
/// index manifest records the variant used, and a vault is opened against
/// the same rule, so it never mixes variants silently.
static CONFORMANCE_DEFAULT: std::sync::Mutex<Option<Variant>> = std::sync::Mutex::new(None);

/// Sets (`Some(name)`) or clears (`None`) the conformance default of the
/// lexical variant for this process; an unknown name is an error and
/// changes nothing. For conformance tests only.
#[doc(hidden)]
pub fn set_conformance_default_tokenizer(name: Option<&str>) -> Result<()> {
    let v = match name {
        Some(n) => Some(Variant::of_name(n).ok_or_else(|| {
            anyhow::anyhow!(
                "conformance tokenizer {n:?}: one of \"mdk_lower_alnum\", \"uni_fold_tr\""
            )
        })?),
        None => None,
    };
    *CONFORMANCE_DEFAULT
        .lock()
        .map_err(|_| anyhow::anyhow!("conformance tokenizer: poisoned lock"))? = v;
    Ok(())
}

fn conformance_default() -> Result<Option<Variant>> {
    Ok(*CONFORMANCE_DEFAULT
        .lock()
        .map_err(|_| anyhow::anyhow!("conformance tokenizer: poisoned lock"))?)
}

/// `[index] embedder`: the default model.
pub const EMBEDDER_DEFAULT: &str = "minilm-l6-v2";
/// `[index] embedder`: the multilingual option (P2-L/L2).
pub const EMBEDDER_MULTILINGUAL: &str = "multilingual-minilm-l12-v2";
pub const EMBEDDERS: [&str; 2] = [EMBEDDER_DEFAULT, EMBEDDER_MULTILINGUAL];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexConfig {
    pub tokenizer: Variant,
    /// one of `EMBEDDERS`
    pub embedder: String,
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig {
            tokenizer: Variant::default(),
            embedder: EMBEDDER_DEFAULT.to_string(),
        }
    }
}

impl IndexConfig {
    /// The settings of an `[index]` table (other keys are the server's).
    pub fn of_table(t: Option<&toml::Table>) -> Result<IndexConfig> {
        let mut c = IndexConfig::default();
        if t.is_none_or(|t| !t.contains_key("tokenizer")) {
            if let Some(v) = conformance_default()? {
                c.tokenizer = v;
            }
        }
        let Some(t) = t else {
            return Ok(c);
        };
        if let Some(v) = t.get("tokenizer") {
            let Some(name) = v.as_str() else {
                bail!("[index] tokenizer must be a string");
            };
            let Some(tok) = Variant::of_name(name) else {
                bail!("[index] tokenizer {name:?}: one of \"mdk_lower_alnum\", \"uni_fold_tr\"");
            };
            c.tokenizer = tok;
        }
        if let Some(v) = t.get("embedder") {
            let Some(name) = v.as_str() else {
                bail!("[index] embedder must be a string");
            };
            if !EMBEDDERS.contains(&name) {
                bail!("[index] embedder {name:?}: one of {EMBEDDERS:?}");
            }
            c.embedder = name.to_string();
        }
        Ok(c)
    }

    /// The settings of the vault at `root` (its `policy.toml`; none: the
    /// defaults).
    pub fn of_vault(root: &Path) -> Result<IndexConfig> {
        let p = rootmarkcore::Policy::load(root)?;
        IndexConfig::of_table(p.index.as_ref())
    }
}
