//! Retention of consumption traces, checkpoint files and location records
//! (P2-R/R2; GOAL P2-R 3, K-Q.1 (5)).
//!
//! rootmark deletes no governance record: a trace line or a checkpoint
//! file behind the retention bound is **moved**, byte for byte and in the
//! same order, to an archive inside `.rootmark/` that only a human deletes;
//! the only deletion is of derived location records (`crate::loc`), which
//! enter no event and no id. `events.jsonl`, `objects/`, `snapshots/` and
//! `attestations.jsonl` are never touched here.
//!
//! - **Bound** (`[retention] keep_checkpoints = K`): the oldest of the last
//!   `K` checkpoints in creation order; without the table, or with at most
//!   `K` checkpoints, there is none and nothing is archived.
//! - **Traces:** the longest prefix of `traces.jsonl` whose lines read a
//!   checkpoint whose `event_seq` lies before the bound's moves to
//!   `traces/archive/<first at_seq>-<last at_seq>.jsonl` (with, beside it,
//!   the lines of `trace_filters.jsonl` of those traces as
//!   `<first>-<last>.filters.jsonl`). `at_seq` stays absolute: the first live
//!   line's `at_seq` is the **base**, kept in `traces/base` (decimal), and
//!   a reader checks `at_seq = base + line`; `trace_id`s do not change and
//!   new traces continue the count (K18-T). The live lines and the archive
//!   files, joined in order, are the former file byte for byte.
//! - **Checkpoints:** a checkpoint behind the bound (created before it),
//!   unnamed, and read by no live trace moves to
//!   `checkpoints/archive/<id>.json`. A named checkpoint is never archived.
//!   A query `at` an archived checkpoint is refused (`Vault::state_at`).
//! - **Location records:** the record of each document's head and those of
//!   the versions the live checkpoints name are kept; the others are
//!   deleted.
//!
//! The decision reads the log, the checkpoint order and the recipe only,
//! never the clock. `plan` writes nothing; `apply` writes the plan of the
//! state it was computed on.

use crate::governance::Trace;
use crate::vault::{checkpoint_path, Replayed, Vault};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn traces_dir(meta: &Path) -> PathBuf {
    meta.join("traces")
}

/// `.rootmark/traces/base`: the `at_seq` of the first live trace line.
pub fn base_path(meta: &Path) -> PathBuf {
    traces_dir(meta).join("base")
}

/// `.rootmark/traces/archive/`
pub fn trace_archive_dir(meta: &Path) -> PathBuf {
    traces_dir(meta).join("archive")
}

/// `.rootmark/checkpoints/archive/<cid>.json`
pub fn archived_checkpoint_path(meta: &Path, cid: &str) -> PathBuf {
    meta.join("checkpoints")
        .join("archive")
        .join(format!("{cid}.json"))
}

/// One trace archive file: `<first>-<last>.jsonl` (and its filters).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchiveFile {
    first: u64,
    last: u64,
    filters: bool,
}

fn archive_name(first: u64, last: u64) -> String {
    format!("{first}-{last}.jsonl")
}

fn filters_name(first: u64, last: u64) -> String {
    format!("{first}-{last}.filters.jsonl")
}

fn decimal(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || (s.len() > 1 && s.starts_with('0'))
    {
        return None;
    }
    s.parse().ok()
}

/// The trace archive files, checked: names `<first>-<last>.jsonl` (and
/// optional `<first>-<last>.filters.jsonl` of the same range), contiguous
/// from 0; anything else in the directory is an error.
fn archive_files(meta: &Path) -> Result<Vec<ArchiveFile>> {
    let dir = trace_archive_dir(meta);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut ranges: BTreeMap<u64, ArchiveFile> = BTreeMap::new();
    let mut filters: BTreeSet<(u64, u64)> = BTreeSet::new();
    for e in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let e = e?;
        let name = e.file_name().to_string_lossy().to_string();
        let (stem, is_filters) = match name.strip_suffix(".filters.jsonl") {
            Some(s) => (s, true),
            None => match name.strip_suffix(".jsonl") {
                Some(s) => (s, false),
                None => bail!(
                    "{}: {name} is not a trace archive file (<first>-<last>.jsonl)",
                    dir.display()
                ),
            },
        };
        let Some((a, b)) = stem
            .split_once('-')
            .and_then(|(a, b)| Some((decimal(a)?, decimal(b)?)))
            .filter(|(a, b)| a <= b)
        else {
            bail!(
                "{}: {name} is not a trace archive file (<first>-<last>.jsonl)",
                dir.display()
            );
        };
        if is_filters {
            filters.insert((a, b));
        } else if ranges
            .insert(
                a,
                ArchiveFile {
                    first: a,
                    last: b,
                    filters: false,
                },
            )
            .is_some()
        {
            bail!("{}: two archive files start at {a}", dir.display());
        }
    }
    let mut out: Vec<ArchiveFile> = ranges.into_values().collect();
    for (a, b) in filters {
        match out.iter_mut().find(|f| f.first == a && f.last == b) {
            Some(f) => f.filters = true,
            None => bail!(
                "{}: {} has no trace archive file {}",
                dir.display(),
                filters_name(a, b),
                archive_name(a, b)
            ),
        }
    }
    let mut next = 0;
    for f in &out {
        if f.first != next {
            bail!(
                "{}: the trace archive is not contiguous: {} follows at_seq {} (a file is missing or extra)",
                dir.display(),
                archive_name(f.first, f.last),
                next.max(1) - 1
            );
        }
        next = f.last + 1;
    }
    Ok(out)
}

/// The trace base (`traces/base`, absent: 0), checked against the archive:
/// the archive files cover `0..base` exactly. A malformed base, or one the
/// archive does not cover, is an error naming the files (never a guess).
pub fn read_base(meta: &Path) -> Result<u64> {
    let p = base_path(meta);
    let base = match fs::read_to_string(&p) {
        Ok(t) => {
            let Some(n) = t.strip_suffix('\n').and_then(decimal) else {
                bail!(
                    "{}: not a trace base (a decimal at_seq and a newline): {t:?}",
                    p.display()
                );
            };
            n
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => return Err(e).with_context(|| format!("read {}", p.display())),
    };
    let files = archive_files(meta)?;
    let covered = files.last().map_or(0, |f| f.last + 1);
    if covered != base {
        bail!(
            "trace base {base} ({}) does not match the trace archive, which covers at_seq 0..{covered} ({})",
            p.display(),
            trace_archive_dir(meta).display()
        );
    }
    Ok(base)
}

/// Every file of `checkpoints/archive/` is `<cid>.json` of a logged
/// checkpoint (its bytes are checked by `Vault::open`).
pub fn check_checkpoint_archive(meta: &Path, st: &Replayed) -> Result<()> {
    let dir = meta.join("checkpoints").join("archive");
    if !dir.exists() {
        return Ok(());
    }
    let known: BTreeSet<&str> = st
        .checkpoints
        .iter()
        .map(|c| c.checkpoint.checkpoint_id.as_str())
        .collect();
    for e in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let name = e?.file_name().to_string_lossy().to_string();
        match name.strip_suffix(".json") {
            Some(cid) if known.contains(cid) => {}
            _ => bail!(
                "{}: {name} is not the file of a logged checkpoint",
                dir.display()
            ),
        }
    }
    Ok(())
}

/// The retention bound: the oldest of the last `K` checkpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bound {
    pub checkpoint_id: String,
    pub event_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TracePlan {
    /// the base before and after
    pub base_before: u64,
    pub base: u64,
    /// trace lines moved (`base - base_before`)
    pub lines: u64,
    /// the archive file they move to (vault-relative), when any
    pub archive: Option<String>,
    /// lines of `trace_filters.jsonl` moved beside it
    pub filter_lines: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocRemoval {
    pub doc_path: String,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocPlan {
    /// the records deleted, by document path and snapshot
    pub removed: Vec<LocRemoval>,
    /// the records kept
    pub kept: u64,
}

/// What a retention prune moves and deletes (`Vault::retention_plan`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPlan {
    pub keep_checkpoints: Option<u64>,
    pub bound: Option<Bound>,
    pub traces: TracePlan,
    /// checkpoint files moved to `checkpoints/archive/`, in creation order
    pub checkpoints: Vec<String>,
    pub loc: LocPlan,
}

fn rel(meta: &Path, p: &Path) -> String {
    let r = p.strip_prefix(meta).unwrap_or(p);
    format!(
        "{}/{}",
        crate::meta::META_DIR,
        r.to_string_lossy().replace('\\', "/")
    )
}

/// The lines of a JSONL file with their `\n` (a last line without one is
/// an error: these files are written line by line).
fn lines_of(p: &Path) -> Result<Vec<Vec<u8>>> {
    let bytes = match fs::read(p) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("read {}", p.display())),
    };
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            out.push(bytes[start..=i].to_vec());
            start = i + 1;
        }
    }
    if start != bytes.len() {
        bail!("{}: the last line has no newline", p.display());
    }
    Ok(out)
}

/// Writes `bytes` to `p` through a temporary file and a rename.
fn replace_file(p: &Path, bytes: &[u8]) -> Result<()> {
    let name = p
        .file_name()
        .context("a file path")?
        .to_string_lossy()
        .to_string();
    let tmp = p.with_file_name(format!(".{name}.tmp"));
    // synced before the rename (P2-R review finding 2): an interrupted prune
    // leaves either the old file or the new one
    let mut f = fs::File::create(&tmp).with_context(|| format!("write {}", tmp.display()))?;
    std::io::Write::write_all(&mut f, bytes)
        .and_then(|_| f.sync_all())
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, p).with_context(|| format!("rename to {}", p.display()))?;
    Ok(())
}

/// A write-once archive file, synced before the live files change.
fn write_archive(p: &Path, bytes: &[u8]) -> Result<()> {
    crate::store::write_immutable(p, bytes)?;
    fs::File::open(p)
        .and_then(|f| f.sync_all())
        .with_context(|| format!("sync {}", p.display()))
}

impl Vault {
    /// `at_seq` of the first live trace line (P2-R/R2; 0 when nothing was
    /// archived).
    pub fn trace_base(&self) -> u64 {
        self.trace_base
    }

    /// The checkpoints whose file is archived (P2-R/R2).
    pub fn archived_checkpoints(&self) -> &BTreeSet<String> {
        &self.archived
    }

    pub fn is_archived(&self, cid: &str) -> bool {
        self.archived.contains(cid)
    }

    /// An error naming the archive file when `cid` is archived.
    pub fn check_not_archived(&self, cid: &str) -> Result<()> {
        if self.archived.contains(cid) {
            bail!(
                "checkpoint {cid} is archived ({}): a query as of it is refused",
                rel(
                    &self.meta_dir,
                    &archived_checkpoint_path(&self.meta_dir, cid)
                )
            );
        }
        Ok(())
    }

    /// The archived traces (`rootmark trace list --archived`), in `at_seq`
    /// order from 0, each line verified (contiguous `at_seq`, `trace_id`
    /// formula, logged checkpoint).
    pub fn archived_traces(&self) -> Result<Vec<Trace>> {
        let mut out = Vec::new();
        for f in archive_files(&self.meta_dir)? {
            let p = trace_archive_dir(&self.meta_dir).join(archive_name(f.first, f.last));
            let lines = lines_of(&p)?;
            if lines.len() as u64 != f.last - f.first + 1 {
                bail!(
                    "{}: {} lines for at_seq {}..={}",
                    p.display(),
                    lines.len(),
                    f.first,
                    f.last
                );
            }
            for (i, l) in lines.iter().enumerate() {
                let t: Trace = serde_json::from_slice(l)
                    .with_context(|| format!("{} line {}", p.display(), i + 1))?;
                let want = f.first + i as u64;
                if t.at_seq != want {
                    bail!(
                        "{} line {}: at_seq {} != {want}",
                        p.display(),
                        i + 1,
                        t.at_seq
                    );
                }
                crate::governance::check_trace_input(&t.consumer, &t.query_hash)?;
                let id = crate::governance::trace_id(
                    &t.checkpoint_id,
                    &t.consumer,
                    &t.query_hash,
                    t.at_seq,
                );
                if t.trace_id != id {
                    bail!(
                        "{} line {}: trace_id {} != {id}",
                        p.display(),
                        i + 1,
                        t.trace_id
                    );
                }
                if !self
                    .state
                    .checkpoints
                    .iter()
                    .any(|c| c.checkpoint.checkpoint_id == t.checkpoint_id)
                {
                    bail!(
                        "{} line {}: unknown checkpoint {}",
                        p.display(),
                        i + 1,
                        t.checkpoint_id
                    );
                }
                out.push(t);
            }
        }
        Ok(out)
    }

    /// The retention prune of this state for `keep` (`[retention]
    /// keep_checkpoints`; `None`: no bound — traces and checkpoints stay,
    /// only location records are pruned). Writes nothing.
    pub fn retention_plan(&self, keep: Option<u64>) -> Result<RetentionPlan> {
        if keep == Some(0) {
            bail!("[retention] keep_checkpoints = 0: the kernel minimum is 1");
        }
        let cps = &self.state.checkpoints;
        let event_seq: BTreeMap<&str, u64> = cps
            .iter()
            .map(|c| (c.checkpoint.checkpoint_id.as_str(), c.checkpoint.event_seq))
            .collect();
        let bound_ix = match keep {
            Some(k) if cps.len() as u64 > k => Some(cps.len() - k as usize),
            _ => None,
        };
        let bound = bound_ix.map(|i| Bound {
            checkpoint_id: cps[i].checkpoint.checkpoint_id.clone(),
            event_seq: cps[i].checkpoint.event_seq,
        });
        let live = self.traces(0)?;
        let mut moved = 0u64;
        if let Some(b) = &bound {
            for t in &live {
                let e = *event_seq
                    .get(t.checkpoint_id.as_str())
                    .context("trace of an unknown checkpoint")?;
                if e >= b.event_seq {
                    break;
                }
                moved += 1;
            }
        }
        let base = self.trace_base + moved;
        let filter_lines = if moved == 0 {
            0
        } else {
            self.trace_filters(&live)?;
            let ids: BTreeSet<&str> = live[..moved as usize]
                .iter()
                .map(|t| t.trace_id.as_str())
                .collect();
            let mut n = 0;
            for l in lines_of(&self.trace_filters_path())? {
                let v: serde_json::Value = serde_json::from_slice(&l)?;
                if v["trace_id"].as_str().is_some_and(|id| ids.contains(id)) {
                    n += 1;
                }
            }
            n
        };
        let traces = TracePlan {
            base_before: self.trace_base,
            base,
            lines: moved,
            archive: (moved > 0).then(|| {
                rel(
                    &self.meta_dir,
                    &trace_archive_dir(&self.meta_dir)
                        .join(archive_name(self.trace_base, base - 1)),
                )
            }),
            filter_lines,
        };
        // checkpoints: behind the bound, unnamed, not yet archived, read by
        // no live trace (those at or after the new base)
        let read: BTreeSet<&str> = live[moved as usize..]
            .iter()
            .map(|t| t.checkpoint_id.as_str())
            .collect();
        let mut checkpoints = Vec::new();
        if let Some(bi) = bound_ix {
            for c in &cps[..bi] {
                let id = c.checkpoint.checkpoint_id.as_str();
                if c.checkpoint.name.is_none() && !self.archived.contains(id) && !read.contains(id)
                {
                    checkpoints.push(id.to_string());
                }
            }
        }
        // location records: each document's head and the versions the live
        // checkpoints (after this plan) name are kept
        let gone: BTreeSet<&str> = checkpoints.iter().map(String::as_str).collect();
        let mut keep_loc: BTreeSet<(String, String)> = BTreeSet::new();
        for (d, ds) in &self.state.docs {
            if let Some(h) = &ds.head {
                keep_loc.insert((d.clone(), h.snapshot_id.clone()));
            }
        }
        for c in cps {
            let id = c.checkpoint.checkpoint_id.as_str();
            if self.archived.contains(id) || gone.contains(id) {
                continue;
            }
            for (d, s) in &c.checkpoint.docs {
                keep_loc.insert((d.clone(), s.clone()));
            }
        }
        let by_hash: BTreeMap<String, &String> = self
            .state
            .docs
            .keys()
            .map(|d| (crate::model::hash32(d), d))
            .collect();
        let mut removed = Vec::new();
        let mut kept = 0u64;
        let loc_dir = self.meta_dir.join("loc");
        if loc_dir.exists() {
            let mut dirs: Vec<PathBuf> = fs::read_dir(&loc_dir)?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<_>>()?;
            dirs.sort();
            for d in dirs {
                let h = d
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let Some(doc) = by_hash.get(&h) else {
                    bail!(
                        "{}: not the location records of a document of this vault",
                        d.display()
                    );
                };
                let mut files: Vec<String> = fs::read_dir(&d)?
                    .map(|e| e.map(|e| e.file_name().to_string_lossy().to_string()))
                    .collect::<std::io::Result<_>>()?;
                files.sort();
                for f in files {
                    // an interrupted write's temporary file is not a record
                    let Some(sid) = f.strip_suffix(".json").filter(|_| !f.starts_with('.')) else {
                        continue;
                    };
                    if keep_loc.contains(&((*doc).clone(), sid.to_string())) {
                        kept += 1;
                    } else {
                        removed.push(LocRemoval {
                            doc_path: (*doc).clone(),
                            snapshot_id: sid.to_string(),
                        });
                    }
                }
            }
        }
        removed.sort_by(|a, b| (&a.doc_path, &a.snapshot_id).cmp(&(&b.doc_path, &b.snapshot_id)));
        Ok(RetentionPlan {
            keep_checkpoints: keep,
            bound,
            traces,
            checkpoints,
            loc: LocPlan { removed, kept },
        })
    }

    /// Carries out a plan of `retention_plan` computed on this state (a plan
    /// of another state — another base or checkpoint list — is an error):
    /// the archive files are written first (write-once), then the live
    /// files are replaced and the base recorded, then checkpoint files are
    /// moved and location records deleted.
    pub fn apply_retention(&mut self, plan: &RetentionPlan) -> Result<()> {
        if plan.traces.base_before != self.trace_base {
            bail!(
                "retention plan of trace base {} applied at base {}",
                plan.traces.base_before,
                self.trace_base
            );
        }
        let meta = self.meta_dir.clone();
        if plan.traces.lines > 0 {
            let (first, base) = (plan.traces.base_before, plan.traces.base);
            let tp = self.traces_path();
            let lines = lines_of(&tp)?;
            let k = plan.traces.lines as usize;
            if k > lines.len() {
                bail!(
                    "retention plan moves {k} trace lines, traces.jsonl has {}",
                    lines.len()
                );
            }
            // the moved lines are those of the plan: their at_seq
            for (i, l) in lines[..k].iter().enumerate() {
                let t: Trace = serde_json::from_slice(l)?;
                if t.at_seq != first + i as u64 {
                    bail!("traces.jsonl changed since the retention plan");
                }
            }
            let dir = trace_archive_dir(&meta);
            fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
            write_archive(
                &dir.join(archive_name(first, base - 1)),
                &lines[..k].concat(),
            )?;
            let ids: BTreeSet<String> = lines[..k]
                .iter()
                .map(|l| serde_json::from_slice::<Trace>(l).map(|t| t.trace_id))
                .collect::<serde_json::Result<_>>()?;
            let fp = self.trace_filters_path();
            let flines = lines_of(&fp)?;
            let mut fk = 0;
            for l in &flines {
                let v: serde_json::Value = serde_json::from_slice(l)?;
                if v["trace_id"].as_str().is_some_and(|id| ids.contains(id)) {
                    fk += 1;
                } else {
                    break;
                }
            }
            if fk as u64 != plan.traces.filter_lines {
                bail!("trace_filters.jsonl changed since the retention plan");
            }
            if fk > 0 {
                write_archive(
                    &dir.join(filters_name(first, base - 1)),
                    &flines[..fk].concat(),
                )?;
            }
            replace_file(&tp, &lines[k..].concat())?;
            if fk > 0 {
                replace_file(&fp, &flines[fk..].concat())?;
            }
            fs::create_dir_all(traces_dir(&meta))?;
            replace_file(&base_path(&meta), format!("{base}\n").as_bytes())?;
            self.trace_base = base;
        }
        for cid in &plan.checkpoints {
            if !self
                .state
                .checkpoints
                .iter()
                .any(|c| c.checkpoint.checkpoint_id == *cid)
            {
                bail!("retention plan archives unknown checkpoint {cid}");
            }
            let from = checkpoint_path(&meta, cid);
            let to = archived_checkpoint_path(&meta, cid);
            fs::create_dir_all(to.parent().expect("parent"))?;
            if to.exists() {
                bail!("{} exists already", to.display());
            }
            fs::rename(&from, &to)
                .with_context(|| format!("move {} to {}", from.display(), to.display()))?;
            self.archived.insert(cid.clone());
        }
        for r in &plan.loc.removed {
            let p = crate::loc::record_path(&meta, &r.doc_path, &r.snapshot_id);
            fs::remove_file(&p).with_context(|| format!("remove {}", p.display()))?;
        }
        Ok(())
    }
}
