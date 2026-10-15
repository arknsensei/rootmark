//! `mdk` CLI (SPEC §7–§9, §11). Every command prints JSON; `--pretty` indents
//! it. `mdk index …` and `mdk search …` run the `mdk-index` binary of the
//! `index` crate (see `delegate`). Governance commands read `<vault>/.mdkernel/policy.toml` (built-in
//! recipe without one) and fail on an invalid recipe. Moves to `server` in
//! P1-D.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use mdkcore::chunk::{chunk, Strategy};
use mdkcore::files::rel_slash;
use mdkcore::vault::IngestSummary;
use mdkcore::{
    Actor, AuthorKind, Layer, MatcherVersion, Policy, Provenance, TransitionKind, Vault,
};
use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "mdk",
    about = "mdkernel core CLI",
    after_help = "Index commands (run by mdk-index): mdk index build|status|compact|watch, mdk search <vault> <query> [--k] [--mode flat|lexical|hybrid] [--at <checkpoint>] [--include-proposal]"
)]
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
    /// Resolve a content-referenced truth to a lineage id
    Resolve {
        vault: PathBuf,
        doc: String,
        version_id: String,
        k: usize,
    },
    /// Agent write: copy <file> to <vault>/<doc> and ingest it as agent content
    /// (new lineages are proposals, spec §7)
    Propose {
        vault: PathBuf,
        doc: String,
        file: PathBuf,
        /// provenance source (e.g. the agent's name)
        #[arg(long)]
        source: String,
    },
    /// Promote a lineage proposal → canonical through the vault's promotion policy
    Promote {
        vault: PathBuf,
        lineage: String,
        /// <human|agent|policy>:<id>
        #[arg(long)]
        actor: String,
        #[arg(long)]
        reason: String,
    },
    /// Demote a lineage to a lower layer through the vault's promotion policy
    Demote {
        vault: PathBuf,
        lineage: String,
        #[arg(long)]
        actor: String,
        #[arg(long)]
        reason: String,
        /// target layer (canonical|proposal|derived); one layer lower when omitted
        #[arg(long)]
        to: Option<String>,
    },
    /// Mark a proposal lineage inactive (deletes nothing) through the policy
    Prune {
        vault: PathBuf,
        lineage: String,
        #[arg(long)]
        actor: String,
        #[arg(long)]
        reason: String,
    },
    /// Checkpoints (spec §8)
    #[command(subcommand)]
    Checkpoint(CheckpointCmd),
    /// Every event that names a lineage
    History { vault: PathBuf, lineage: String },
    /// Per block of a document: lineage, version, layer, author, source,
    /// creating and last seq (at the head, or as of a checkpoint)
    Blame {
        vault: PathBuf,
        doc: String,
        /// checkpoint id or name
        #[arg(long)]
        at: Option<String>,
    },
    /// Read blocks as of a checkpoint; records a consumption trace
    Read {
        vault: PathBuf,
        /// checkpoint id or name
        #[arg(long)]
        at: String,
        #[arg(long)]
        consumer: String,
        #[arg(required = true)]
        lineages: Vec<String>,
    },
    /// Consumption traces (spec §8)
    #[command(subcommand)]
    Trace(TraceCmd),
    /// Raw journal and version trigger (spec §11)
    #[command(subcommand)]
    Journal(JournalCmd),
    /// The vault's policy recipe
    #[command(subcommand)]
    Policy(PolicyCmd),
}

#[derive(Subcommand)]
enum CheckpointCmd {
    /// Checkpoint of the current state (same state → same id)
    Create {
        vault: PathBuf,
        #[arg(long)]
        name: Option<String>,
    },
    List {
        vault: PathBuf,
    },
    Show {
        vault: PathBuf,
        /// checkpoint id or name
        checkpoint: String,
    },
}

#[derive(Subcommand)]
enum TraceCmd {
    List {
        vault: PathBuf,
        /// only traces with at_seq >= since
        #[arg(long, default_value_t = 0)]
        since: u64,
    },
}

#[derive(Subcommand)]
enum JournalCmd {
    /// Append the current bytes of <vault>/<doc> to its journal, then version
    /// it if the policy's trigger says so
    Append {
        vault: PathBuf,
        doc: String,
        /// time of the save in ms since the epoch (default: the clock)
        #[arg(long)]
        now_ms: Option<u64>,
        /// author of the save
        #[arg(long, value_parser = ["human", "agent", "transcript", "synthetic"])]
        author: String,
        #[arg(long)]
        source: String,
    },
    /// Re-check the trigger (idle) for the latest save
    Tick {
        vault: PathBuf,
        doc: String,
        #[arg(long)]
        now_ms: Option<u64>,
        #[arg(long, value_parser = ["human", "agent", "transcript", "synthetic"])]
        author: String,
        #[arg(long)]
        source: String,
    },
    /// Journal records of a document
    List { vault: PathBuf, doc: String },
    /// Prune records older than the policy's retention (no event)
    Prune {
        vault: PathBuf,
        #[arg(long)]
        now_ms: Option<u64>,
    },
}

#[derive(Subcommand)]
enum PolicyCmd {
    /// Load and print the recipe (an invalid recipe is an error)
    Show { vault: PathBuf },
}

fn ingest_one(v: &mut Vault, doc: &str, path: &Path, m: MatcherVersion) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let r = v.ingest(doc, &text, Provenance::default(), m)?;
    Ok(serde_json::to_value(IngestSummary::new(doc, &r))?)
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

fn now_or(t: Option<u64>) -> Result<u64> {
    match t {
        Some(t) => Ok(t),
        None => Ok(u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis(),
        )?),
    }
}

/// `<vault>/<doc>` for a vault-relative, `/`-separated doc path. The path
/// must stay inside the vault: no `..`/absolute components, and no existing
/// component (directory or the file itself) may be a symbolic link, so a
/// read or write can never leave the vault through a link.
fn doc_file(vault: &Path, doc: &str) -> Result<PathBuf> {
    let rel = Path::new(doc);
    if doc.is_empty()
        || !doc.ends_with(".md")
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        bail!("doc {doc:?}: expected a vault-relative path to a .md file");
    }
    let root =
        std::fs::canonicalize(vault).with_context(|| format!("vault {}", vault.display()))?;
    let mut p = root.clone();
    for c in rel.components() {
        p.push(c);
        if let Ok(m) = std::fs::symlink_metadata(&p) {
            if m.file_type().is_symlink() {
                bail!("doc {doc:?}: {} is a symbolic link", p.display());
            }
        }
    }
    if !p.starts_with(&root) {
        bail!("doc {doc:?} resolves outside the vault");
    }
    Ok(p)
}

/// The vault and its policy; an invalid policy.toml fails before anything else.
fn open_governed(vault: &Path) -> Result<(Vault, Policy)> {
    let policy = Policy::load(vault)?;
    Ok((Vault::open(vault)?, policy))
}

fn transition(
    vault: &Path,
    kind: TransitionKind,
    lineage: &str,
    actor: &str,
    reason: &str,
    to: Option<&str>,
) -> Result<Value> {
    let (mut v, policy) = open_governed(vault)?;
    let actor = Actor::parse(actor)?;
    let hook = policy.promotion.hook();
    let ev = match kind {
        TransitionKind::Promote => {
            let req = v.request(kind, lineage, Layer::Canonical, actor, reason)?;
            v.promote(&req, hook)?
        }
        TransitionKind::Demote => {
            let to = match to {
                Some(t) => Layer::parse(t)?,
                None => match v.layer_of(lineage)? {
                    Layer::Canonical => Layer::Proposal,
                    Layer::Proposal => Layer::Derived,
                    Layer::Derived => bail!("lineage {lineage} is derived: no lower layer"),
                },
            };
            let req = v.request(kind, lineage, to, actor, reason)?;
            v.demote(&req, hook)?
        }
        TransitionKind::Prune => v.prune_inactive(lineage, actor, reason, hook)?,
    };
    Ok(json!({"event": ev, "layer": v.layer_of(lineage)?, "inactive": v.is_inactive(lineage)?}))
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
        Cmd::Resolve {
            vault,
            doc,
            version_id,
            k,
        } => {
            let v = Vault::open(&vault)?;
            json!({"lineage_id": v.resolve_truth(&doc, &version_id, k)?})
        }
        Cmd::Propose {
            vault,
            doc,
            file,
            source,
        } => {
            let (mut v, _) = open_governed(&vault)?;
            let text = std::fs::read_to_string(&file)
                .with_context(|| format!("read {}", file.display()))?;
            let target = doc_file(&vault, &doc)?;
            if let Some(d) = target.parent() {
                std::fs::create_dir_all(d)?;
            }
            std::fs::write(&target, &text)?;
            let prov = Provenance {
                author_kind: AuthorKind::Agent,
                source,
            };
            let r = v.ingest(&doc, &text, prov, m)?;
            let blocks = v.snapshot_blocks(&r.snapshot)?;
            json!({
                "ingested": IngestSummary::new(&doc, &r),
                "layers": blocks.iter().map(|b| json!({"lineage_id": b.lineage_id, "layer": b.layer})).collect::<Vec<_>>(),
                "events_hash": v.events_hash()?,
            })
        }
        Cmd::Promote {
            vault,
            lineage,
            actor,
            reason,
        } => transition(
            &vault,
            TransitionKind::Promote,
            &lineage,
            &actor,
            &reason,
            None,
        )?,
        Cmd::Demote {
            vault,
            lineage,
            actor,
            reason,
            to,
        } => transition(
            &vault,
            TransitionKind::Demote,
            &lineage,
            &actor,
            &reason,
            to.as_deref(),
        )?,
        Cmd::Prune {
            vault,
            lineage,
            actor,
            reason,
        } => transition(
            &vault,
            TransitionKind::Prune,
            &lineage,
            &actor,
            &reason,
            None,
        )?,
        Cmd::Checkpoint(c) => match c {
            CheckpointCmd::Create { vault, name } => {
                let mut v = Vault::open(&vault)?;
                serde_json::to_value(v.checkpoint(name.as_deref())?)?
            }
            CheckpointCmd::List { vault } => {
                json!({"checkpoints": Vault::open(&vault)?.checkpoints()})
            }
            CheckpointCmd::Show { vault, checkpoint } => {
                serde_json::to_value(Vault::open(&vault)?.find_checkpoint(&checkpoint)?)?
            }
        },
        Cmd::History { vault, lineage } => {
            let v = Vault::open(&vault)?;
            let layer = v.layer_of(&lineage).ok();
            json!({"lineage_id": lineage, "layer": layer, "events": v.history(&lineage)?})
        }
        Cmd::Blame { vault, doc, at } => {
            let v = Vault::open(&vault)?;
            let blocks = v.blame(&doc, at.as_deref())?;
            json!({"doc_path": doc, "at": at, "blocks": blocks})
        }
        Cmd::Read {
            vault,
            at,
            consumer,
            lineages,
        } => {
            let mut v = Vault::open(&vault)?;
            let (blocks, trace) = v.read_blocks(&lineages, &consumer, &at)?;
            json!({"blocks": blocks, "trace": trace})
        }
        Cmd::Trace(TraceCmd::List { vault, since }) => {
            json!({"traces": Vault::open(&vault)?.traces(since)?})
        }
        Cmd::Journal(j) => match j {
            JournalCmd::Append {
                vault,
                doc,
                now_ms,
                author: a,
                source,
            } => {
                let (mut v, policy) = open_governed(&vault)?;
                let now = now_or(now_ms)?;
                let path = doc_file(&vault, &doc)?;
                let bytes =
                    std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
                let rec = v.journal_append(&doc, &bytes, now)?;
                let prov = Provenance {
                    author_kind: author(&a)?,
                    source,
                };
                let r = v.maybe_version(&doc, &policy.trigger, now, prov, m)?;
                json!({
                    "record": rec,
                    "versioned": r.as_ref().map(|r| IngestSummary::new(&doc, r)),
                })
            }
            JournalCmd::Tick {
                vault,
                doc,
                now_ms,
                author: a,
                source,
            } => {
                let (mut v, policy) = open_governed(&vault)?;
                doc_file(&vault, &doc)?;
                let now = now_or(now_ms)?;
                let prov = Provenance {
                    author_kind: author(&a)?,
                    source,
                };
                let r = v.maybe_version(&doc, &policy.trigger, now, prov, m)?;
                json!({"versioned": r.as_ref().map(|r| IngestSummary::new(&doc, r))})
            }
            JournalCmd::List { vault, doc } => {
                json!({"doc_path": doc, "records": Vault::open(&vault)?.journal(&doc)?})
            }
            JournalCmd::Prune { vault, now_ms } => {
                let (mut v, policy) = open_governed(&vault)?;
                serde_json::to_value(v.journal_prune(now_or(now_ms)?, policy.retention)?)?
            }
        },
        Cmd::Policy(PolicyCmd::Show { vault }) => Policy::load(&vault)?.to_json(),
    })
}

/// `mdk index …` and `mdk search …` (GOAL P1-C "CLI") are served by the
/// `mdk-index` binary of the `index` crate: `core` cannot link `index`
/// (dependency direction core ← index; ASSUMPTIONS P1-C C0-1), so `mdk`
/// runs it with the same arguments (global `--pretty` and `--matcher`
/// included; `mdk-index` resolves `--matcher` like `mdk`) and exits with its
/// status. No index logic lives here. `$MDK_INDEX_BIN` overrides the binary next to `mdk`.
const DELEGATED: [&str; 2] = ["index", "search"];

fn index_bin() -> anyhow::Result<PathBuf> {
    if let Some(p) = std::env::var_os("MDK_INDEX_BIN").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().context("locate mdk")?;
    let p = exe.with_file_name(format!("mdk-index{}", std::env::consts::EXE_SUFFIX));
    if !p.is_file() {
        bail!(
            "{} not found: `mdk index`/`mdk search` need the mdk-index binary next to mdk (cargo build --release -p mdkernel-index)",
            p.display()
        );
    }
    Ok(p)
}

/// The exit status of `mdk-index` when the command is delegated.
fn delegate() -> Option<i32> {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    // the subcommand is the first argument that is neither a flag nor the
    // value of `--matcher`
    let mut skip_value = false;
    let first = args.iter().find(|a| {
        let s = a.to_string_lossy();
        if skip_value {
            skip_value = false;
            return false;
        }
        if s == "--matcher" {
            skip_value = true;
            return false;
        }
        !s.starts_with('-')
    })?;
    let delegated = |a: &std::ffi::OsString| DELEGATED.iter().any(|d| a == *d);
    // `mdk help index|search` is the delegated binary's help
    let is_help = first == "help"
        && args
            .iter()
            .skip_while(|a| *a != first)
            .nth(1)
            .is_some_and(delegated);
    if !delegated(first) && !is_help {
        return None;
    }
    let status = index_bin().and_then(|bin| {
        std::process::Command::new(&bin)
            .args(&args)
            .status()
            .with_context(|| format!("run {}", bin.display()))
    });
    Some(match status {
        Ok(st) => st.code().unwrap_or(1),
        Err(e) => {
            eprintln!("error: {e:#}");
            1
        }
    })
}

fn main() {
    if let Some(code) = delegate() {
        std::process::exit(code);
    }
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
