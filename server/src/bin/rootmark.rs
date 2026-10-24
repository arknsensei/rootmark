//! `rootmark` CLI (SPEC §7–§9, §11). Every command prints JSON; `--pretty` indents
//! it. `rootmark index …` and `rootmark search …` are `rootmarkindex::cli` (linked, since
//! P1-D/D0 no second process; K-C6.1). Governance commands read
//! `<vault>/.rootmark/policy.toml` (built-in recipe without one) and fail on
//! an invalid recipe. Moved from `core` to `server` in P1-D/D0.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rootmarkcore::chunk::{chunk, Strategy};
use rootmarkcore::files::rel_slash;
use rootmarkcore::vault::IngestSummary;
use rootmarkcore::{
    Actor, AuthorKind, Layer, MatcherVersion, Policy, Provenance, TransitionKind, Vault,
};
use rootmarkserver::paths::doc_file;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rootmark", about = "rootmark CLI", version)]
struct Cli {
    /// Print the spec version this rootmark implements (spec/SPEC.md) and exit
    #[arg(long)]
    spec_version: bool,
    /// Indent JSON output
    #[arg(long, global = true)]
    pretty: bool,
    /// Lineage matcher version for ingest (v2|v3; default: $ROOTMARK_MATCHER, else v3).
    /// A vault is locked to the version of its first event.
    #[arg(long, global = true, value_parser = ["v2", "v3"])]
    matcher: Option<String>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the conformance suite (spec/CONFORMANCE.md, Levels 1–3) on the
    /// fixtures embedded in this binary; prints the report, exit 1 unless
    /// level 3 with every test passing
    Conformance {
        /// another fixture tree (default: the embedded spec/fixtures)
        #[arg(long)]
        fixtures: Option<PathBuf>,
        /// the embedder model directory (MANIFEST.json + model; default
        /// $ROOTMARK_MODELS, else the build tree's models/)
        #[arg(long)]
        models: Option<PathBuf>,
        /// also write the report to this file
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Ingest files; doc_path is the file path relative to the vault root
    Ingest { vault: PathBuf, files: Vec<PathBuf> },
    /// Ingest every *.md under dir in byte-wise path order; doc_path is relative to dir
    IngestDir { vault: PathBuf, dir: PathBuf },
    /// Rebuild the last snapshot of a document from the log
    Replay { vault: PathBuf, doc: String },
    /// blake3 of events.jsonl
    EventsHash { vault: PathBuf },
    /// Migrate a vault's stored artifacts to this rootmark's formats
    /// (P2-A): every registered migration that applies, in order, each with
    /// a rollback record under `.rootmark/migrations/<id>/`; never changes
    /// `events_hash`. `--rename-store` runs only the rename of
    /// `<vault>/.mdkernel/` to `<vault>/.rootmark/` (K-GH.1)
    Migrate {
        /// only the rename of the metadata directory
        #[arg(long, conflicts_with_all = ["dry_run", "rollback"])]
        rename_store: bool,
        /// print the version matrix and the plan; change nothing
        #[arg(long, conflicts_with = "rollback")]
        dry_run: bool,
        /// undo migration <id> (the newest record) and check every restored
        /// file is byte-equal to the one before
        #[arg(long, value_name = "ID")]
        rollback: Option<String>,
        vault: PathBuf,
    },
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
    /// Agent write: the text of <file> into <vault>/<doc> as agent content
    /// (new lineages are proposals, spec §7), through the same checks and
    /// edit policy as the MCP/HTTP `propose` tool (K-D1.1; actor agent:<source>)
    Propose {
        vault: PathBuf,
        doc: String,
        file: PathBuf,
        /// provenance source and actor id (e.g. the agent's name)
        #[arg(long)]
        source: String,
        /// create (default; never overwrites), append or replace_section
        #[arg(long, value_parser = ["create", "append", "replace_section"])]
        mode: Option<String>,
        /// the section heading (replace_section only)
        #[arg(long)]
        heading: Option<String>,
        /// why (recorded in .rootmark/proposals.jsonl)
        #[arg(long, default_value = "rootmark propose (CLI)")]
        reason: String,
    },
    /// Attest the current bytes of documents to the vault's running server
    /// (spec 2.0.0 §7.1; HTTP with the token, as a client does after a save;
    /// the window applies: run it right after saving)
    Attest {
        vault: PathBuf,
        /// vault-relative Markdown paths
        docs: Vec<String>,
        /// every Markdown document of the vault; raises no change already
        /// ingested (only later observations of these bytes count)
        #[arg(long)]
        all: bool,
        /// the one document was renamed from this path (`mv`): the server
        /// accepts it only as a rename of that path's content and carries
        /// each block's layer over as it is (P2-0.1 T3, P2-0.2 T2)
        #[arg(long = "renamed-from")]
        renamed_from: Option<String>,
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
        /// the channel's label, recorded as the event's `meta.source` (spec
        /// 2.0.3 §7; e.g. the bot that relayed the approval); none: no field
        #[arg(long)]
        source: Option<String>,
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
        /// the channel's label, recorded as the event's `meta.source` (spec
        /// 2.0.3 §7); none: no field
        #[arg(long)]
        source: Option<String>,
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
    /// Index commands (spec §9)
    #[command(subcommand)]
    Index(rootmarkindex::cli::IndexCmd),
    /// Ranked chunks as of a checkpoint; records one consumption trace
    Search(rootmarkindex::cli::SearchArgs),
    /// Serve one vault (GOAL P1-D): MCP over stdio, local HTTP API, file
    /// watcher, hot-tier compaction; until SIGTERM/SIGINT or the end of the
    /// MCP stream. stdout carries only MCP; progress goes to stderr.
    Serve(ServeArgs),
}

#[derive(clap::Args)]
struct ServeArgs {
    #[arg(long)]
    vault: PathBuf,
    /// MCP transport (only `stdio`)
    #[arg(long, value_parser = ["stdio"])]
    mcp: Option<String>,
    /// HTTP API address: 127.0.0.1:PORT (or 127.0.0.1 with [server] http_port)
    #[arg(long)]
    http: Option<String>,
    /// ingest Markdown files changed under the vault as human content
    #[arg(long)]
    watch: bool,
    /// append the attestation diagnostic log (one JSON line per attestation
    /// and per observed change: accepted/refused and why, match distance) to
    /// this file, which must lie outside the vault (P2-A/A0)
    #[arg(long)]
    attest_log: Option<PathBuf>,
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
    /// Check the recipe as `rootmark serve` loads it — the kernel's fields,
    /// the server's tables, the promotion rules (P2-P/P1) — and list the
    /// rules; exit 1 with the field named on any error
    Lint { vault: PathBuf },
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
    source: Option<&str>,
) -> Result<Value> {
    let (mut v, policy) = open_governed(vault)?;
    let actor = Actor::parse(actor)?;
    let hook = policy.promotion.hook();
    let ev = match kind {
        TransitionKind::Promote => {
            let req = v.request(kind, lineage, Layer::Canonical, actor, reason)?;
            v.promote_from(&req, hook, source)?
        }
        TransitionKind::Demote => {
            let to = match to {
                Some(t) => Layer::parse(t)?,
                None => match v.layer_of(lineage)? {
                    Layer::Canonical => Layer::Proposal,
                    // the 1.0.3 defaults; `unverified` (2.0.0) has `derived` below it
                    Layer::Proposal | Layer::Unverified => Layer::Derived,
                    Layer::Derived => bail!("lineage {lineage} is derived: no lower layer"),
                },
            };
            let req = v.request(kind, lineage, to, actor, reason)?;
            v.demote_from(&req, hook, source)?
        }
        TransitionKind::Prune => v.prune_inactive(lineage, actor, reason, hook)?,
    };
    Ok(json!({"event": ev, "layer": v.layer_of(lineage)?, "inactive": v.is_inactive(lineage)?}))
}

/// `matcher`: `--matcher`, else `$ROOTMARK_MATCHER`. Index commands pass it on
/// as given (`index watch` falls back to the vault's lock, like
/// `rootmark-index`); every other command defaults to v3.
fn run(cmd: Cmd, matcher: Option<MatcherVersion>) -> Result<Value> {
    let m = matcher.unwrap_or(MatcherVersion::V3);
    Ok(match cmd {
        Cmd::Serve(_) => bail!("rootmark serve is not a JSON command"),
        Cmd::Conformance { .. } => bail!("rootmark conformance is handled in main"),
        Cmd::Index(c) => rootmarkindex::cli::run(rootmarkindex::cli::Cmd::Index(c), matcher)?,
        Cmd::Search(a) => rootmarkindex::cli::run(rootmarkindex::cli::Cmd::Search(a), matcher)?,
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
        Cmd::Migrate {
            rename_store,
            dry_run,
            rollback,
            vault,
        } => {
            use rootmarkserver::migrate;
            if let Some(id) = rollback {
                migrate::rollback(&vault, &id)?
            } else if dry_run {
                migrate::dry_run(&vault, None)?
            } else if rename_store {
                let mut r = migrate::migrate(&vault, Some("rename_store"))?;
                r["renamed"] = json!({
                    "from": vault.join(rootmarkcore::meta::LEGACY_META_DIR).display().to_string(),
                    "to": rootmarkcore::meta::path(&vault).display().to_string(),
                });
                r
            } else {
                migrate::migrate(&vault, None)?
            }
        }
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
            mode,
            heading,
            reason,
        } => {
            let text = std::fs::read_to_string(&file)
                .with_context(|| format!("read {}", file.display()))?;
            let k = rootmarkserver::tools::Kernel::open_with(&vault, matcher)?;
            let mut args = json!({"doc_path": doc, "content": text, "reason": reason});
            if let Some(md) = mode {
                args["mode"] = json!(md);
            }
            if let Some(h) = heading {
                args["heading"] = json!(h);
            }
            let caller = rootmarkserver::tools::Caller {
                channel: rootmarkserver::tools::Channel::Cli,
                consumer: Some(&source),
            };
            let mut out = k.call("propose", &args, caller)?;
            // the P1-B output fields: every block's layer
            let layers: Vec<Value> = out["blocks"]
                .as_array()
                .map(|bs| {
                    bs.iter()
                        .map(|b| json!({"lineage_id": b["lineage_id"], "layer": b["layer"]}))
                        .collect()
                })
                .unwrap_or_default();
            out["layers"] = Value::Array(layers);
            out
        }
        Cmd::Attest {
            vault,
            docs,
            all,
            renamed_from,
        } => {
            if renamed_from.is_some() && (all || docs.len() != 1) {
                bail!("rootmark attest: --renamed-from goes with exactly one document");
            }
            let docs = match (all, docs.is_empty()) {
                (true, true) => rootmarkcore::files::markdown_files(&vault)?
                    .into_iter()
                    .map(|(d, _)| d)
                    .collect(),
                (false, false) => docs,
                (true, false) => bail!("rootmark attest: give documents or --all, not both"),
                (false, true) => bail!("rootmark attest: no document (give paths or --all)"),
            };
            // `--all` never raises an earlier unattested change (late: false)
            rootmarkserver::attest::cli_attest(&vault, &docs, !all, renamed_from.as_deref())?
        }
        Cmd::Promote {
            vault,
            lineage,
            actor,
            reason,
            source,
        } => transition(
            &vault,
            TransitionKind::Promote,
            &lineage,
            &actor,
            &reason,
            None,
            source.as_deref(),
        )?,
        Cmd::Demote {
            vault,
            lineage,
            actor,
            reason,
            to,
            source,
        } => transition(
            &vault,
            TransitionKind::Demote,
            &lineage,
            &actor,
            &reason,
            to.as_deref(),
            source.as_deref(),
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
        Cmd::Policy(PolicyCmd::Lint { vault }) => {
            let p = Policy::load(&vault)?;
            rootmarkserver::config::ServerConfig::from_policy(&p)
                .with_context(|| format!("policy {} rejected", Policy::path(&vault).display()))?;
            json!({
                "ok": true,
                "source": p.to_json()["source"],
                "promotion": p.promotion.hook().name(),
                "promotion_version": p.promotion_version,
                "rules": p.promotion_rules,
            })
        }
    })
}

fn main() {
    let cli = Cli::parse();
    if cli.spec_version {
        println!("{}", rootmarkserver::tools::SPEC_VERSION);
        return;
    }
    let Some(cmd) = cli.cmd else {
        eprintln!("error: a subcommand is required (rootmark --help)");
        std::process::exit(2);
    };
    if let Cmd::Conformance {
        fixtures,
        models,
        report,
    } = cmd
    {
        let res = rootmarkserver::conformance::command(fixtures, models).and_then(|r| {
            let mut text = serde_json::to_string_pretty(&r)?;
            text.push('\n');
            if let Some(p) = &report {
                std::fs::write(p, &text)
                    .map_err(|e| anyhow::anyhow!("write {}: {e}", p.display()))?;
            }
            Ok((r, text))
        });
        match res {
            Ok((r, text)) => {
                print!("{text}");
                let ok =
                    r["level_claimed"] == 3 && r["passed"] == rootmarkserver::conformance::TOTAL;
                eprintln!(
                    "rootmark conformance: level {}, {}/{} (spec {})",
                    r["level_claimed"],
                    r["passed"],
                    r["total"],
                    r["spec_version"].as_str().unwrap_or("")
                );
                std::process::exit(if ok { 0 } else { 1 });
            }
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(1);
            }
        }
    }
    if let Cmd::Serve(a) = cmd {
        // serve prints no JSON: --pretty would be silently ignored
        let res = if cli.pretty {
            Err(anyhow::anyhow!("rootmark serve: --pretty does not apply"))
        } else {
            cli.matcher
                .clone()
                .or_else(|| std::env::var("ROOTMARK_MATCHER").ok())
                .map(|n| MatcherVersion::parse(&n))
                .transpose()
                .and_then(|matcher| {
                    rootmarkserver::serve::run(rootmarkserver::serve::ServeOptions {
                        vault: a.vault,
                        mcp_stdio: a.mcp.is_some(),
                        http: a.http,
                        watch: a.watch,
                        attest_log: a.attest_log,
                        matcher,
                    })
                })
        };
        if let Err(e) = res {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
        return;
    }
    let name = cli
        .matcher
        .or_else(|| std::env::var("ROOTMARK_MATCHER").ok());
    let res = name
        .map(|n| MatcherVersion::parse(&n))
        .transpose()
        .and_then(|m| run(cmd, m));
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
