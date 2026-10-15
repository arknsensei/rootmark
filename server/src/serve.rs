//! `mdk serve` (GOAL P1-D "Servis"): one process, one vault.
//!
//! Start: the vault must open; without `.mdkernel/policy.toml` the default
//! recipe (`config::default_recipe`, promotion `human_only`) is written; the
//! policy and its `[server]`/`[index]` tables must be valid; without an
//! index one is built. Then, as asked:
//! - `--mcp stdio`: MCP over stdin/stdout (`crate::mcp`); the end of stdin
//!   ends the service;
//! - `--http 127.0.0.1:PORT`: the HTTP API (`crate::http`), token in
//!   `.mdkernel/server.token`;
//! - `--watch`: Markdown files changed under the vault (outside
//!   `.mdkernel/`) are ingested as human content and indexed into the hot
//!   tier (`Kernel::watch_doc`);
//! - always: the hot tier is compacted every `[index] compact_every_s`.
//!
//! SIGTERM or SIGINT (or the end of the MCP stream) stops the service: the
//! HTTP server and the watcher stop, the hot tier is compacted, and the
//! process exits 0 — or 1 if that final compaction fails. Progress lines go
//! to stderr (`mdk serve: …`); stdout carries only MCP.

use crate::config::{self, ServerConfig};
use crate::tools::Kernel;
use anyhow::{bail, Context, Result};
use mdkcore::{MatcherVersion, Policy};
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
    eprintln!("mdk serve: {}", msg.as_ref());
}

/// The vault's policy, writing the default recipe first if there is none.
fn policy_of(vault: &Path) -> Result<(Policy, ServerConfig)> {
    let p = Policy::path(vault);
    if std::fs::symlink_metadata(&p).is_err() {
        std::fs::create_dir_all(vault.join(".mdkernel"))?;
        std::fs::write(&p, config::default_recipe())
            .with_context(|| format!("write {}", p.display()))?;
        log(format!(
            "no policy.toml: wrote the default recipe (promotion human_only) to {}",
            p.display()
        ));
    }
    let policy = Policy::load(vault)?;
    let cfg = ServerConfig::from_policy(&policy)
        .with_context(|| format!("policy {} rejected", p.display()))?;
    Ok((policy, cfg))
}

/// Vault-relative `/` path of a changed file, if it is a Markdown document
/// of the vault (not under `.mdkernel/`).
fn watched_doc(root: &Path, p: &Path) -> Option<String> {
    let rel = p.strip_prefix(root).ok()?;
    let mut parts = Vec::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(s) => parts.push(s.to_str()?.to_string()),
            _ => return None,
        }
    }
    if parts.first().map(String::as_str) == Some(".mdkernel") {
        return None;
    }
    let doc = parts.join("/");
    doc.ends_with(".md").then_some(doc)
}

/// The watcher thread: batches events for 50 ms, then ingests each changed
/// document once, in byte order. Returns the watcher (dropping it stops the
/// events, which ends the thread).
fn start_watcher(kernel: Arc<Kernel>) -> Result<notify::RecommendedWatcher> {
    use notify::{RecursiveMode, Watcher};
    let root = std::fs::canonicalize(kernel.vault())?;
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut w = notify::recommended_watcher(tx).context("start the file watcher")?;
    w.watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", root.display()))?;
    std::thread::spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut batch = vec![first];
            while let Ok(ev) = rx.recv_timeout(Duration::from_millis(50)) {
                batch.push(ev);
            }
            let mut docs = std::collections::BTreeSet::new();
            for ev in batch {
                match ev {
                    Ok(ev) => {
                        if matches!(ev.kind, notify::EventKind::Access(_)) {
                            continue;
                        }
                        for p in ev.paths {
                            // macOS reports /private/var/… for /var/…
                            let p = std::fs::canonicalize(&p).unwrap_or(p);
                            if let Some(d) = watched_doc(&root, &p) {
                                docs.insert(d);
                            }
                        }
                    }
                    Err(e) => log(format!("watch: error: {e}")),
                }
            }
            for d in docs {
                if !root.join(&d).is_file() {
                    log(format!(
                        "watch: {d} removed or not a file; removals are not ingested"
                    ));
                    continue;
                }
                match kernel.watch_doc(&d, "mdk-serve-watch") {
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
        Ok(Some(r)) => {
            log(format!(
                "compact ({why}): {} records, {} snapshots, {} moved from the hot tier",
                r.records, r.snapshots, r.moved_from_hot
            ));
            Ok(())
        }
        Ok(None) => {
            log(format!("compact ({why}): hot tier empty"));
            Ok(())
        }
        Err(e) => Err(e.context(format!("compact ({why})"))),
    }
}

#[cfg(unix)]
async fn signal() -> &'static str {
    use tokio::signal::unix::{signal, SignalKind};
    let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
    let mut int = signal(SignalKind::interrupt()).expect("SIGINT handler");
    tokio::select! {
        _ = term.recv() => "SIGTERM",
        _ = int.recv() => "SIGINT",
    }
}

#[cfg(not(unix))]
async fn signal() -> &'static str {
    let _ = tokio::signal::ctrl_c().await;
    "Ctrl-C"
}

pub fn run(o: ServeOptions) -> Result<()> {
    if !o.mcp_stdio && o.http.is_none() && !o.watch {
        bail!("mdk serve: nothing to serve (--mcp stdio, --http 127.0.0.1:PORT, --watch)");
    }
    let kernel = Arc::new(Kernel::open_with(&o.vault, o.matcher)?);
    let (_policy, cfg) = policy_of(&o.vault)?;
    // the HTTP address is checked before anything is built
    let http_addr = match &o.http {
        Some(s) => Some(crate::http::parse_addr(s, cfg.http_port)?),
        None => None,
    };
    if let Some(r) = kernel.ensure_index()? {
        log(format!(
            "no index: built one ({} records, {} snapshots, {})",
            r.records, r.snapshots, r.embedder_id
        ));
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    rt.block_on(async move {
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
                s = signal() => break s.to_string(),
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
