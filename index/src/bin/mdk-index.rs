//! `mdk-index`: the index half of the `mdk` CLI (GOAL P1-C "CLI"). `core`'s
//! `mdk` cannot link `index` (dependency direction core ← index), so `mdk
//! index …` and `mdk search …` run this binary, installed next to `mdk`,
//! with the same arguments (ASSUMPTIONS P1-C C0-1). Every command prints
//! JSON (`--pretty` indents it); any failure is a non-zero exit with the
//! error on stderr. Moves to `server` with `mdk` in P1-D.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use mdkcore::{AuthorKind, Provenance, Vault};
use mdkindex::embed::Embedder;
use mdkindex::models;
use mdkindex::search::{Mode, Searcher};
use mdkindex::store::{self, Store};
use mdkindex::OnnxMiniLm;
use serde_json::json;
use std::path::{Component, Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "mdk-index",
    about = "mdkernel index CLI (`mdk index …`, `mdk search …`)"
)]
struct Cli {
    /// Indent JSON output
    #[arg(long, global = true)]
    pretty: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Index commands
    #[command(subcommand)]
    Index(IndexCmd),
    /// Ranked chunks as of a checkpoint; records one consumption trace
    Search {
        vault: PathBuf,
        query: String,
        #[arg(long, default_value_t = 10)]
        k: usize,
        #[arg(long, default_value = "hybrid", value_parser = ["flat", "lexical", "hybrid"])]
        mode: String,
        /// checkpoint id or name (default: the checkpoint of the current
        /// state, created if the state has none)
        #[arg(long)]
        at: Option<String>,
        /// also return proposal chunks: labelled, score ×0.5, after canonical
        #[arg(long)]
        include_proposal: bool,
        /// consumer recorded in the trace
        #[arg(long, default_value = "mdk-cli")]
        consumer: String,
    },
    /// Model files (models/MANIFEST.json)
    #[command(subcommand)]
    Models(ModelsCmd),
}

#[derive(Subcommand)]
enum IndexCmd {
    /// Rebuild <vault>/.mdkernel/index/ (every head and every checkpoint's
    /// snapshots; the hot tier is dropped)
    Build {
        vault: PathBuf,
        /// the embedder (only one exists: minilm-l6-v2)
        #[arg(long, default_value = "minilm-l6-v2")]
        embedder: String,
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
enum ModelsCmd {
    /// Check every model file against the manifest (size, blake3)
    Verify,
}

fn main() {
    let cli = Cli::parse();
    match run(cli.cmd) {
        Ok(v) => {
            let s = if cli.pretty {
                serde_json::to_string_pretty(&v)
            } else {
                serde_json::to_string(&v)
            };
            println!("{}", s.expect("JSON output"));
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

fn embedder(name: &str) -> Result<OnnxMiniLm> {
    let e = OnnxMiniLm::load_default()?;
    if !e.id().starts_with(&format!("{name}/")) {
        bail!(
            "unknown embedder {name:?} (the only embedder is {})",
            e.id()
        );
    }
    Ok(e)
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

fn run(cmd: Cmd) -> Result<serde_json::Value> {
    Ok(match cmd {
        Cmd::Index(IndexCmd::Build {
            vault,
            embedder: name,
        }) => {
            let v = Vault::open(&vault)?;
            serde_json::to_value(store::build(&v, &embedder(&name)?)?)?
        }
        Cmd::Index(IndexCmd::Status { vault }) => {
            let v = Vault::open(&vault)?;
            let s = Store::open(&v, &embedder("minilm-l6-v2")?)?;
            serde_json::to_value(s.status(&v)?)?
        }
        Cmd::Index(IndexCmd::Compact { vault }) => {
            let v = Vault::open(&vault)?;
            serde_json::to_value(store::compact(&v, &embedder("minilm-l6-v2")?)?)?
        }
        Cmd::Index(IndexCmd::Watch {
            vault,
            doc,
            author: a,
            source,
        }) => {
            let mut v = Vault::open(&vault)?;
            let e = embedder("minilm-l6-v2")?;
            let mut s = Store::open(&v, &e)?;
            let p = doc_file(&vault, &doc)?;
            let text =
                std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
            let prov = Provenance {
                author_kind: author(&a)?,
                source,
            };
            serde_json::to_value(s.watch(&mut v, &e, &doc, &text, prov)?)?
        }
        Cmd::Search {
            vault,
            query,
            k,
            mode,
            at,
            include_proposal,
            consumer,
        } => {
            let mut v = Vault::open(&vault)?;
            let e = embedder("minilm-l6-v2")?;
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
        Cmd::Models(ModelsCmd::Verify) => {
            let root = models::models_root();
            let m = models::verify(&root)?;
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
