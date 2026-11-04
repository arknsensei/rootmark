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
    /// `--attest-log <file>` (P2-A/A0 c): outside the vault
    pub attest_log: Option<PathBuf>,
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

/// `p` with symbolic links resolved where it exists: the longest existing
/// ancestor is canonicalized and the rest appended, so a removed path
/// reads like the existing ones (macOS reports /private/var/… for /var/…).
fn canon(p: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(p) {
        return c;
    }
    let mut rest = Vec::new();
    let mut cur = p;
    while let (Some(parent), Some(name)) = (cur.parent(), cur.file_name()) {
        rest.push(name.to_os_string());
        if let Ok(mut c) = std::fs::canonicalize(parent) {
            for n in rest.iter().rev() {
                c.push(n);
            }
            return c;
        }
        cur = parent;
    }
    p.to_path_buf()
}

/// Vault-relative `/` path of `p` under `root` if none of its components is
/// hidden (starts with `.`, so `.rootmark/`, `.obsidian/`, `.x.md` are
/// skipped) — the rule of `rootmarkcore::files::markdown_files`, which
/// `ingest-dir` uses. The root itself is no path.
fn watched_rel(root: &Path, p: &Path) -> Option<String> {
    let rel = p.strip_prefix(root).ok()?;
    let mut parts = Vec::new();
    for c in rel.components() {
        match c {
            std::path::Component::Normal(s) => parts.push(s.to_str()?.to_string()),
            _ => return None,
        }
    }
    if parts.is_empty() || parts.iter().any(|c| c.starts_with('.')) {
        return None;
    }
    Some(parts.join("/"))
}

/// What an event path asks of the watcher (P2-Q.1 DESIGN §2–§3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Target {
    /// a Markdown document of the vault (whether its file is there or not)
    Doc(String),
    /// a directory created in or moved into the vault: its Markdown files
    /// are read (PQ-22: inotify watches a new directory only after its
    /// creation is read, so a file written into it before is never named)
    Scan(String),
}

/// The targets an event names: none for access events, `.rootmark/` writes,
/// hidden or non-Markdown paths.
fn targets_of(ev: notify::Result<notify::Event>, root: &Path) -> Vec<Target> {
    use notify::event::{EventKind, ModifyKind};
    let ev = match ev {
        Ok(ev) => ev,
        Err(e) => {
            log(format!("watch: error: {e}"));
            return Vec::new();
        }
    };
    if matches!(ev.kind, EventKind::Access(_)) {
        return Vec::new();
    }
    let arrives = matches!(
        ev.kind,
        EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(_))
    );
    let mut out = Vec::new();
    for p in ev.paths {
        let c = canon(&p);
        let Some(rel) = watched_rel(root, &c) else {
            continue;
        };
        let dir = std::fs::symlink_metadata(&c).is_ok_and(|m| m.is_dir());
        if dir && arrives {
            out.push(Target::Scan(rel));
        } else if rel.ends_with(".md") {
            out.push(Target::Doc(rel));
        }
    }
    out
}

/// The Markdown documents under the vault directory `rel`, by the rule of
/// `watched_rel` (hidden components skipped); symbolic links are not
/// followed (a note says so). A directory gone meanwhile has none.
fn scan_dir(root: &Path, rel: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![rel.to_string()];
    while let Some(d) = stack.pop() {
        let entries = match std::fs::read_dir(root.join(&d)) {
            Ok(e) => e,
            Err(e) => {
                log(format!("watch: {d}/: not read ({e})"));
                continue;
            }
        };
        for e in entries.flatten() {
            let Some(name) = e.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let r = format!("{d}/{name}");
            match e.file_type() {
                Ok(t) if t.is_symlink() => {
                    log(format!("watch: {r}: a symbolic link, not followed"))
                }
                Ok(t) if t.is_dir() => stack.push(r),
                Ok(t) if t.is_file() && name.ends_with(".md") => out.push(r),
                _ => {}
            }
        }
    }
    out
}

/// The watcher thread: batches the events that name a document or a new
/// directory (50 ms of quiet, at most 250 ms; other events are dropped and
/// never extend a batch), reads each new directory's Markdown files, then
/// ingests each changed document once, in byte order (a document named
/// twice — by a directory's scan and by its own event — is ingested once;
/// an unchanged one writes nothing). Returns the watcher (dropping it stops
/// the events, which ends the thread).
fn start_watcher(kernel: Arc<Kernel>) -> Result<notify::RecommendedWatcher> {
    use notify::{RecursiveMode, Watcher};
    let root = std::fs::canonicalize(kernel.vault())?;
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut w = notify::recommended_watcher(tx).context("start the file watcher")?;
    w.watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", root.display()))?;
    std::thread::spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut targets: std::collections::BTreeSet<Target> =
                targets_of(first, &root).into_iter().collect();
            if targets.is_empty() {
                continue; // the server's own writes under .rootmark/ etc.
            }
            // gather the rest of the batch: until 50 ms pass without an
            // event naming a target, and at most 250 ms in all, so a
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
                        let t = targets_of(ev, &root);
                        if !t.is_empty() {
                            last = std::time::Instant::now();
                            targets.extend(t);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut docs = std::collections::BTreeSet::new();
            for t in targets {
                match t {
                    Target::Doc(d) => {
                        docs.insert(d);
                    }
                    Target::Scan(d) => docs.extend(scan_dir(&root, &d)),
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
    let mut kernel = Kernel::open_with(&o.vault, o.matcher)?;
    if let Some(p) = &o.attest_log {
        let l = crate::attest_log::AttestLog::open(p, &o.vault)?;
        log(format!(
            "attestation diagnostic log: {}",
            l.path().display()
        ));
        kernel.set_attest_log(l);
    }
    let kernel = Arc::new(kernel);
    let (policy, cfg, write_policy) = policy_of(&o.vault)?;
    // the HTTP address is checked before anything is written or built
    let http_addr = match &o.http {
        Some(s) => Some(crate::http::parse_addr(s, cfg.http_port)?),
        None => None,
    };
    if write_policy {
        write_default_policy(&o.vault)?;
    } else if policy.attestation.implicit {
        // P2-P/P0 (K-P2L-T1): a recipe without `[attestation]` (an older
        // rootmark's) is read as `unverified`, not the default `canonical`
        log(format!(
            "warning: {} has no [attestation] table: unattested changes are read as \"unverified\" (an older rootmark's recipe; the default is \"canonical\"); `rootmark migrate {}` adds `unattested = \"canonical\"` (rollback record kept), or write the table yourself",
            Policy::path(&o.vault).display(),
            o.vault.display()
        ));
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
        IndexAtStart::Stale(why) => {
            let migrate = crate::migrate::matrix(&o.vault)
                .is_ok_and(|m| m["required"].as_array().is_some_and(|r| r.iter().any(|x| x == "index_rebuild")));
            log(if migrate {
                format!(
                    "index stale ({why}); [index] auto_rebuild = false: an older rootmark's index — searches answer migration_required until `rootmark migrate {}` (rollback record kept) or `rootmark index build`",
                    o.vault.display()
                )
            } else {
                format!(
                    "index stale ({why}); [index] auto_rebuild = false: searches answer index_stale until `rootmark index build`"
                )
            })
        }
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
