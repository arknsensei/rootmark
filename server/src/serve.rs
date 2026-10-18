//! `rootmark serve` (GOAL P1-D "Servis"): one process, one vault.
//!
//! Start: the vault must open; without `.rootmark/policy.toml` the default
//! recipe (`config::default_recipe`, promotion `human_only`) is written; the
//! policy and its `[server]`/`[index]` tables must be valid; without an
//! index one is built, and an index of another format is rebuilt unless
//! `[index] auto_rebuild = false` (then searches answer `index_stale`,
//! K-E0.3). Then, as asked:
//! - `--mcp stdio`: MCP over stdin/stdout (`crate::mcp`); the end of stdin
//!   ends the service;
//! - `--http 127.0.0.1:PORT`: the HTTP API (`crate::http`), token in
//!   `.rootmark/server.token`;
//! - `--watch`: Markdown files changed under the vault (outside
//!   `.rootmark/`) are ingested as human content and indexed into the hot
//!   tier (`Kernel::watch_doc`);
//! - always: the hot tier is compacted every `[index] compact_every_s`.
//!
//! SIGTERM or SIGINT (or the end of the MCP stream) stops the service: the
//! HTTP server and the watcher stop, the hot tier is compacted, and the
//! process exits 0 — or 1 if that final compaction fails. An index that is
//! not `ok` (`stale`, `missing`, `damaged`) is not compacted: a note, exit 0
//! (CLIENT_CONTRACT §3; P2-0/Z0). Progress lines go
//! to stderr (`rootmark serve: …`); stdout carries only MCP.

use crate::config::{self, ServerConfig};
use crate::tools::{Compacted, IndexAtStart, Kernel};
use anyhow::{bail, Context, Result};
use rootmarkcore::{MatcherVersion, Policy, PolicySource};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub struct ServeOptions {
    pub vault: PathBuf,
    pub mcp_stdio: bool,
    pub http: Option<String>,
    pub watch: bool,
    pub matcher: Option<MatcherVersion>,
}

fn log(msg: impl AsRef<str>) {
    eprintln!("rootmark serve: {}", msg.as_ref());
}

/// The vault's policy and server settings. Without `policy.toml` the
/// default recipe is used and returned for writing (`true`); nothing is
/// written here, so a start that fails later leaves the vault untouched.
fn policy_of(vault: &Path) -> Result<(Policy, ServerConfig, bool)> {
    rootmarkcore::meta::check(vault)?;
    let p = Policy::path(vault);
    let missing = std::fs::symlink_metadata(&p).is_err();
    let policy = if missing {
        Policy::parse(&config::default_recipe(), PolicySource::File(p.clone()))?
    } else {
        Policy::load(vault)?
    };
    let cfg = ServerConfig::from_policy(&policy)
        .with_context(|| format!("policy {} rejected", p.display()))?;
    Ok((policy, cfg, missing))
}

fn write_default_policy(vault: &Path) -> Result<()> {
    let p = Policy::path(vault);
    std::fs::create_dir_all(rootmarkcore::meta::path(vault))?;
    std::fs::write(&p, config::default_recipe())
        .with_context(|| format!("write {}", p.display()))?;
    log(format!(
        "no policy.toml: wrote the default recipe (promotion human_only) to {}",
        p.display()
    ));
    Ok(())
}

/// Vault-relative `/` path of a changed file, if it is a Markdown document
/// of the vault: a `.md` file none of whose path components is hidden
/// (starts with `.`, so `.rootmark/`, `.obsidian/`, `.x.md` are skipped) —
/// the rule of `rootmarkcore::files::markdown_files`, which `ingest-dir` uses.
fn watched_doc(root: &Path, p: &Path) -> Option<String> {
    let rel = p.strip_prefix(root).ok()?;
    let mut parts = Vec::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(s) => parts.push(s.to_str()?.to_string()),
            _ => return None,
        }
    }
    if parts.iter().any(|c| c.starts_with('.')) {
        return None;
    }
    let doc = parts.join("/");
    doc.ends_with(".md").then_some(doc)
}

/// The watcher thread: batches the events that name a document (50 ms of
/// quiet, at most 250 ms; other events are dropped and never extend a
/// batch), then ingests each changed document once, in byte order. Returns the watcher (dropping it stops the
/// events, which ends the thread).
fn start_watcher(kernel: Arc<Kernel>) -> Result<notify::RecommendedWatcher> {
    use notify::{RecursiveMode, Watcher};
    let root = std::fs::canonicalize(kernel.vault())?;
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut w = notify::recommended_watcher(tx).context("start the file watcher")?;
    w.watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", root.display()))?;
    // the documents an event names (none for `.rootmark/` writes, hidden
    // or non-Markdown paths, and access events)
    let docs_of = move |ev: notify::Result<notify::Event>, root: &Path| -> Vec<String> {
        match ev {
            Ok(ev) if !matches!(ev.kind, notify::EventKind::Access(_)) => ev
                .paths
                .into_iter()
                // macOS reports /private/var/… for /var/…
                .filter_map(|p| watched_doc(root, &std::fs::canonicalize(&p).unwrap_or(p)))
                .collect(),
            Ok(_) => Vec::new(),
            Err(e) => {
                log(format!("watch: error: {e}"));
                Vec::new()
            }
        }
    };
    std::thread::spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut docs: std::collections::BTreeSet<String> =
                docs_of(first, &root).into_iter().collect();
            if docs.is_empty() {
                continue; // the server's own writes under .rootmark/ etc.
            }
            // gather the rest of the batch: until 50 ms pass without an
            // event naming a document, and at most 250 ms in all, so a
            // stream of unrelated events (searches writing traces) can
            // never hold a change back
            let start = std::time::Instant::now();
            let mut last = start;
            loop {
                let quiet = Duration::from_millis(50).saturating_sub(last.elapsed());
                let left = Duration::from_millis(250).saturating_sub(start.elapsed());
                let wait = quiet.min(left);
                if wait.is_zero() {
                    break;
                }
                match rx.recv_timeout(wait) {
                    Ok(ev) => {
                        let d = docs_of(ev, &root);
                        if !d.is_empty() {
                            last = std::time::Instant::now();
                            docs.extend(d);
                        }
                    }
                    Err(_) => break,
                }
            }
            for d in docs {
                if !root.join(&d).is_file() {
                    log(format!(
                        "watch: {d} removed or not a file; removals are not ingested"
                    ));
                    continue;
                }
                match kernel.watch_doc(&d, "rootmark-serve-watch") {
                    Ok(r) if r.events == 0 && r.embedded == 0 => {}
                    Ok(r) => log(format!(
                        "watch: {d} ingested ({} events, {} chunks embedded, snapshot {})",
                        r.events, r.embedded, r.snapshot_id
                    )),
                    Err(e) => log(format!("watch: {d}: error: {e:#}")),
                }
            }
        }
    });
    Ok(w)
}

async fn compact(kernel: &Arc<Kernel>, why: &str) -> Result<()> {
    let k = kernel.clone();
    match tokio::task::spawn_blocking(move || k.compact()).await? {
        Ok(Compacted::Done(r)) => {
            log(format!(
                "compact ({why}): {} records, {} snapshots, {} moved from the hot tier",
                r.records, r.snapshots, r.moved_from_hot
            ));
            Ok(())
        }
        Ok(Compacted::Empty) => {
            log(format!("compact ({why}): hot tier empty"));
            Ok(())
        }
        Ok(Compacted::Skipped(state)) => {
            log(format!(
                "compact ({why}): skipped, index {state} (rebuild it with `rootmark index build`)"
            ));
            Ok(())
        }
        Err(e) => Err(e.context(format!("compact ({why})"))),
    }
}

/// SIGTERM and SIGINT, registered once for the whole service: a signal
/// that arrives while the loop is busy (a timer compaction) stays pending
/// in its stream and is seen at the next poll — none is lost.
struct Signals {
    #[cfg(unix)]
    term: tokio::signal::unix::Signal,
    #[cfg(unix)]
    int: tokio::signal::unix::Signal,
}

impl Signals {
    fn new() -> Result<Signals> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            Ok(Signals {
                term: signal(SignalKind::terminate()).context("SIGTERM handler")?,
                int: signal(SignalKind::interrupt()).context("SIGINT handler")?,
            })
        }
        #[cfg(not(unix))]
        Ok(Signals {})
    }

    async fn next(&mut self) -> &'static str {
        #[cfg(unix)]
        {
            tokio::select! {
                _ = self.term.recv() => "SIGTERM",
                _ = self.int.recv() => "SIGINT",
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
            "Ctrl-C"
        }
    }
}

pub fn run(o: ServeOptions) -> Result<()> {
    if !o.mcp_stdio && o.http.is_none() && !o.watch {
        bail!("rootmark serve: nothing to serve (--mcp stdio, --http 127.0.0.1:PORT, --watch)");
    }
    let kernel = Arc::new(Kernel::open_with(&o.vault, o.matcher)?);
    let (_policy, cfg, write_policy) = policy_of(&o.vault)?;
    // the HTTP address is checked before anything is written or built
    let http_addr = match &o.http {
        Some(s) => Some(crate::http::parse_addr(s, cfg.http_port)?),
        None => None,
    };
    if write_policy {
        write_default_policy(&o.vault)?;
    }
    match kernel.start_index(cfg.auto_rebuild)? {
        IndexAtStart::Current => {}
        IndexAtStart::Built(r) => log(format!(
            "no index: built one ({} records, {} snapshots, {})",
            r.records, r.snapshots, r.embedder_id
        )),
        IndexAtStart::Rebuilt(why, r) => log(format!(
            "index stale ({why}): rebuilt ({} records, {} snapshots, {})",
            r.records, r.snapshots, r.embedder_id
        )),
        IndexAtStart::Damaged(why) => log(format!(
            "index damaged ({why}); left as it is (never rebuilt over): index tools answer index_stale until `rootmark index build`"
        )),
        IndexAtStart::Stale(why) => log(format!(
            "index stale ({why}); [index] auto_rebuild = false: searches answer index_stale until `rootmark index build`"
        )),
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    rt.block_on(async move {
        let mut signals = Signals::new()?;
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let mut http_task = None;
        if let Some(addr) = http_addr {
            let (token, state) = crate::token::ensure(&o.vault, cfg.token_rotate_days)?;
            log(format!(
                "token {state:?} ({})",
                crate::token::token_path(&o.vault).display()
            ));
            let (listener, bound) = crate::http::bind(addr).await?;
            let limit = usize::try_from(cfg.max_propose_bytes)? + 1_048_576;
            let app = crate::http::router(kernel.clone(), token, limit);
            let mut rx = stop_rx.clone();
            http_task = Some(tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        let _ = rx.wait_for(|s| *s).await;
                    })
                    .await
            }));
            log(format!("http listening on {bound}"));
        }
        let watcher = if o.watch {
            let w = start_watcher(kernel.clone())?;
            log(format!("watching {}", o.vault.display()));
            Some(w)
        } else {
            None
        };
        let mcp = async {
            if o.mcp_stdio {
                crate::mcp::serve_stdio(kernel.clone()).await
            } else {
                std::future::pending::<Result<()>>().await
            }
        };
        let every = Duration::from_secs(cfg.compact_every_s);
        let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
        tokio::pin!(mcp);
        let why = loop {
            tokio::select! {
                s = signals.next() => break s.to_string(),
                r = &mut mcp => {
                    if let Err(e) = r {
                        log(format!("mcp: error: {e:#}"));
                    }
                    break "end of the MCP stream".to_string();
                }
                _ = timer.tick() => {
                    if let Err(e) = compact(&kernel, "timer").await {
                        log(format!("error: {e:#}"));
                    }
                }
            }
        };
        log(format!("stopping ({why})"));
        let _ = stop_tx.send(true);
        drop(watcher);
        if let Some(t) = http_task {
            t.await.context("http task")?.context("http server")?;
        }
        compact(&kernel, "shutdown").await?;
        log("stopped");
        Ok(())
    })
}
