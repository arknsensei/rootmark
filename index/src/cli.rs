//! The index commands of the `rootmark` CLI (`rootmark index …`, `rootmark search …`;
//! GOAL P1-C "CLI"), shared by the `rootmark` binary of the `server` crate and
//! this crate's `rootmark-index` binary (P1-D/D0: `rootmark` calls `run` directly, no
//! longer through a second process; K-C6.1). Every command returns JSON; a
//! failure is an `Err`.

use crate::models;
use crate::search::{Mode, Searcher};
use crate::store::{self, Store};
use crate::OnnxMiniLm;
use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use rootmarkcore::{AuthorKind, MatcherVersion, Provenance, Vault};
use serde_json::json;
use std::path::{Component, Path, PathBuf};

#[derive(Subcommand)]
pub enum Cmd {
    /// Index commands
    #[command(subcommand)]
    Index(IndexCmd),
    /// Ranked chunks as of a checkpoint; records one consumption trace
    Search(SearchArgs),
    /// Model files (models/MANIFEST.json)
    #[command(subcommand)]
    Models(ModelsCmd),
}

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    pub vault: PathBuf,
    pub query: String,
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    #[arg(long, default_value = "hybrid", value_parser = ["flat", "lexical", "hybrid"])]
    pub mode: String,
    /// checkpoint id or name (default: the checkpoint of the current
    /// state, created if the state has none)
    #[arg(long)]
    pub at: Option<String>,
    /// also return up to k proposal chunks, ranked apart with the canonical
    /// statistics: labelled, score ×0.5, after the unchanged canonical hits
    #[arg(long)]
    pub include_proposal: bool,
    /// consumer recorded in the trace
    #[arg(long, default_value = "rootmark-cli")]
    pub consumer: String,
}

#[derive(Subcommand)]
pub enum IndexCmd {
    /// Rebuild <vault>/.rootmark/index/ (every head and every checkpoint's
    /// snapshots; the hot tier is dropped)
    Build {
        vault: PathBuf,
        /// the embedder: the vault's `[index] embedder` (default
        /// minilm-l6-v2); given, it must be that one (P2-L/L2)
        #[arg(long)]
        embedder: Option<String>,
    },
    /// Cold and hot record counts; documents whose head is not indexed
    Status { vault: PathBuf },
    /// Move the hot tier into a new cold index (rankings unchanged)
    Compact { vault: PathBuf },
    /// Ingest <vault>/<doc> once and add its new chunks to the hot tier
    Watch {
        vault: PathBuf,
        doc: String,
        #[arg(long, default_value = "human", value_parser = ["human", "agent", "transcript", "synthetic"])]
        author: String,
        #[arg(long, default_value = "")]
        source: String,
    },
}

#[derive(Subcommand)]
pub enum ModelsCmd {
    /// Check every model file against the manifest (size, blake3)
    Verify {
        /// the model (P2-L/L2): minilm-l6-v2 (default) or
        /// multilingual-minilm-l12-v2
        #[arg(long, default_value = crate::config::EMBEDDER_DEFAULT)]
        embedder: String,
    },
}

/// The vault's embedder (`[index] embedder`, P2-L/L2); a name given on the
/// command line must be that one.
fn embedder(vault: &std::path::Path, name: Option<&str>) -> Result<OnnxMiniLm> {
    let want = crate::config::IndexConfig::of_vault(vault)?.embedder;
    if let Some(n) = name {
        if !crate::config::EMBEDDERS.contains(&n) {
            bail!(
                "unknown embedder {n:?} (one of {:?})",
                crate::config::EMBEDDERS
            );
        }
        if n != want {
            bail!("--embedder {n:?}: the vault's [index] embedder is {want:?} (set it in .rootmark/policy.toml)");
        }
    }
    OnnxMiniLm::load_named(&want)
}

/// `<vault>/<doc>` for a vault-relative `/`-separated document path.
fn doc_file(vault: &Path, doc: &str) -> Result<PathBuf> {
    let p = Path::new(doc);
    if doc.is_empty()
        || p.is_absolute()
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        bail!("{doc:?} is not a vault-relative document path");
    }
    Ok(vault.join(p))
}

fn author(s: &str) -> Result<AuthorKind> {
    Ok(match s {
        "human" => AuthorKind::Human,
        "agent" => AuthorKind::Agent,
        "transcript" => AuthorKind::Transcript,
        "synthetic" => AuthorKind::Synthetic,
        _ => bail!("unknown author {s:?}"),
    })
}

pub fn run(cmd: Cmd, matcher: Option<MatcherVersion>) -> Result<serde_json::Value> {
    Ok(match cmd {
        Cmd::Index(IndexCmd::Build {
            vault,
            embedder: name,
        }) => {
            let v = Vault::open(&vault)?;
            serde_json::to_value(store::build(&v, &embedder(&vault, name.as_deref())?)?)?
        }
        Cmd::Index(IndexCmd::Status { vault }) => {
            let v = Vault::open(&vault)?;
            let s = Store::open(&v, &embedder(&vault, None)?)?;
            serde_json::to_value(s.status(&v)?)?
        }
        Cmd::Index(IndexCmd::Compact { vault }) => {
            let v = Vault::open(&vault)?;
            serde_json::to_value(store::compact(&v, &embedder(&vault, None)?)?)?
        }
        Cmd::Index(IndexCmd::Watch {
            vault,
            doc,
            author: a,
            source,
        }) => {
            let mut v = Vault::open(&vault)?;
            let e = embedder(&vault, None)?;
            let mut s = Store::open(&v, &e)?;
            let p = doc_file(&vault, &doc)?;
            let text =
                std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
            let prov = Provenance {
                author_kind: author(&a)?,
                source,
            };
            let m = match matcher {
                Some(m) => m,
                None => v.matcher_lock().unwrap_or(MatcherVersion::V3),
            };
            serde_json::to_value(s.watch_with(&mut v, &e, &doc, &text, prov, m)?)?
        }
        Cmd::Search(SearchArgs {
            vault,
            query,
            k,
            mode,
            at,
            include_proposal,
            consumer,
        }) => {
            // before the vault is opened: an empty query touches nothing (K-C1.5)
            crate::search::check_query(&query)?;
            let mut v = Vault::open(&vault)?;
            let e = embedder(&vault, None)?;
            let s = Store::open(&v, &e)?;
            let mut se = Searcher::new(&s, &e, &consumer)?;
            serde_json::to_value(se.search(
                &mut v,
                Mode::parse(&mode)?,
                &query,
                k,
                at.as_deref(),
                include_proposal,
            )?)?
        }
        Cmd::Models(ModelsCmd::Verify { embedder }) => {
            let root = models::models_root();
            let m = models::verify_named(&root, &embedder)?;
            json!({
                "ok": true,
                "root": root.display().to_string(),
                "name": m.manifest().name,
                "revision": m.manifest().revision,
                "files": m.manifest().files.len(),
                "embedder_id": m.embedder_id(),
            })
        }
    })
}
