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
        let _ = consumer;
        match name {
            "status" => {
                args::<NoArgs>(name, a)?;
                self.status()
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
