//! `mdk` CLI (SPEC §7). Every command prints JSON; `--pretty` indents it.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use mdkcore::chunk::{chunk, Strategy};
use mdkcore::files::rel_slash;
use mdkcore::vault::IngestSummary;
use mdkcore::{MatcherVersion, Provenance, Vault};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "mdk", about = "mdkernel core CLI")]
struct Cli {
    /// Indent JSON output
    #[arg(long, global = true)]
    pretty: bool,
    /// Lineage matcher version for ingest (v2|v3; default: $MDK_MATCHER, else v3).
    /// A vault is locked to the version of its first event.
    #[arg(long, global = true, value_parser = ["v2", "v3"])]
    matcher: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Ingest files; doc_path is the file path relative to the vault root
    Ingest { vault: PathBuf, files: Vec<PathBuf> },
    /// Ingest every *.md under dir in byte-wise path order; doc_path is relative to dir
    IngestDir { vault: PathBuf, dir: PathBuf },
    /// Rebuild the last snapshot of a document from the log
    Replay { vault: PathBuf, doc: String },
    /// blake3 of events.jsonl
    EventsHash { vault: PathBuf },
    /// Chunks of the head snapshot of a document
    Chunk {
        vault: PathBuf,
        doc: String,
        #[arg(long, default_value = "structural")]
        strategy: String,
    },
    /// Per block of a document: lineage, version, layer, author, source,
    /// creating and last seq (at the head, or as of a checkpoint)
    Blame {
        vault: PathBuf,
        doc: String,
        /// checkpoint id or name
        #[arg(long)]
        at: Option<String>,
    },
    /// Resolve a content-referenced truth to a lineage id
    Resolve {
        vault: PathBuf,
        doc: String,
        version_id: String,
        k: usize,
    },
}

fn ingest_one(v: &mut Vault, doc: &str, path: &Path, m: MatcherVersion) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let r = v.ingest(doc, &text, Provenance::default(), m)?;
    Ok(serde_json::to_value(IngestSummary::new(doc, &r))?)
}

fn run(cmd: Cmd, m: MatcherVersion) -> Result<Value> {
    Ok(match cmd {
        Cmd::Ingest { vault, files } => {
            let mut v = Vault::open(&vault)?;
            let root = std::fs::canonicalize(&vault)?;
            let mut out = Vec::new();
            for f in files {
                let p = std::fs::canonicalize(&f).with_context(|| format!("{}", f.display()))?;
                let doc = rel_slash(&p, &root)?;
                out.push(ingest_one(&mut v, &doc, &p, m)?);
            }
            json!({"ingested": out, "events_hash": v.events_hash()?})
        }
        Cmd::IngestDir { vault, dir } => {
            let mut v = Vault::open(&vault)?;
            let out = v.ingest_dir(&dir, m)?;
            json!({"ingested": out, "events_hash": v.events_hash()?})
        }
        Cmd::Replay { vault, doc } => {
            let v = Vault::open(&vault)?;
            match v.replay(&doc)? {
                Some(s) => serde_json::to_value(s)?,
                None => bail!("no snapshot for {doc}"),
            }
        }
        Cmd::EventsHash { vault } => json!({"events_hash": Vault::open(&vault)?.events_hash()?}),
        Cmd::Chunk {
            vault,
            doc,
            strategy,
        } => {
            let v = Vault::open(&vault)?;
            let snap = v
                .snapshot(&doc)?
                .with_context(|| format!("no snapshot for {doc}"))?;
            serde_json::to_value(chunk(&snap, &v, Strategy::parse(&strategy)?)?)?
        }
        Cmd::Blame { vault, doc, at } => {
            let v = Vault::open(&vault)?;
            let blocks = v.blame(&doc, at.as_deref())?;
            json!({"doc_path": doc, "at": at, "blocks": blocks})
        }
        Cmd::Resolve {
            vault,
            doc,
            version_id,
            k,
        } => {
            let v = Vault::open(&vault)?;
            json!({"lineage_id": v.resolve_truth(&doc, &version_id, k)?})
        }
    })
}

fn main() {
    let cli = Cli::parse();
    let name = cli
        .matcher
        .or_else(|| std::env::var("MDK_MATCHER").ok())
        .unwrap_or_else(|| "v3".to_string());
    let res = MatcherVersion::parse(&name).and_then(|m| run(cli.cmd, m));
    match res {
        Ok(v) => {
            let s = if cli.pretty {
                serde_json::to_string_pretty(&v)
            } else {
                serde_json::to_string(&v)
            };
            println!("{}", s.expect("JSON serialisation"));
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}
