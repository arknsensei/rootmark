//! Raw journal and version triggers (spec §11).
//!
//! Every save of a document is appended to `.mdkernel/journal/<doc_hash>.jsonl`
//! (`doc_hash = H(doc_path)`) as `(seq, at_ms, content_hash, bytes_len)`; the
//! bytes are kept content-addressed under `.mdkernel/journal/blobs/`. A
//! version (an ingest) is created only when a `VersionTrigger` says so, and
//! always from the latest journal content. Kernel bounds:
//! - minimum journal retention 24 h (`JournalRetention::new`);
//! - a version never carries less than the journal: `maybe_version` ingests
//!   the latest save, and pruning never drops a save that is not versioned;
//! - unversioned content is never indexed: `Vault::indexable_snapshot`
//!   returns versioned snapshots only.
//!
//! Parameters outside the bounds are errors, never clamped. Pruning writes
//! no event (the journal is not part of the event log).

use crate::lineage::MatcherVersion;
use crate::model::{hash32, Provenance};
use crate::parse::parse_document;
use crate::store::write_immutable;
use crate::vault::{snapshot_id, IngestResult, Snapshot, Vault};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Kernel bound (spec §11): no policy may keep the journal shorter.
pub const MIN_RETENTION_HOURS: u64 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRecord {
    pub seq: u64,
    pub at_ms: u64,
    pub content_hash: String,
    pub bytes_len: u64,
}

/// What a trigger sees: the time, the last save, and the normalized block
/// text of the last version and of the latest save.
#[derive(Debug, Clone)]
pub struct TriggerInput<'a> {
    pub now_ms: u64,
    pub last_save_ms: u64,
    /// `None`: the document has no version yet
    pub previous: Option<&'a str>,
    pub current: &'a str,
}

/// Version trigger policy (spec §11, ARCHITECTURE §4.4 `on_version_trigger`).
pub trait VersionTrigger {
    fn name(&self) -> &str;
    fn should_version(&self, input: &TriggerInput) -> bool;
}

/// Version when the document has been idle for `idle_ms` or when the latest
/// save differs from the last version by at least `min_sim_change`
/// (1 − normalized Levenshtein similarity of the block texts).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdleOrDistance {
    idle_ms: u64,
    min_sim_change: f64,
}

impl IdleOrDistance {
    /// `idle_ms ≥ 0`, `min_sim_change ∈ [0, 1]`; anything else is an error.
    pub fn new(idle_ms: i64, min_sim_change: f64) -> Result<IdleOrDistance> {
        if idle_ms < 0 {
            bail!("version_trigger.idle_ms = {idle_ms}: must be >= 0");
        }
        if !(0.0..=1.0).contains(&min_sim_change) {
            bail!("version_trigger.min_sim_change = {min_sim_change}: must lie in [0, 1]");
        }
        Ok(IdleOrDistance {
            idle_ms: idle_ms as u64,
            min_sim_change,
        })
    }

    pub fn idle_ms(&self) -> u64 {
        self.idle_ms
    }

    pub fn min_sim_change(&self) -> f64 {
        self.min_sim_change
    }

    pub fn distance(previous: &str, current: &str) -> f64 {
        1.0 - strsim::normalized_levenshtein(previous, current)
    }
}

impl VersionTrigger for IdleOrDistance {
    fn name(&self) -> &str {
        "idle_or_distance"
    }

    fn should_version(&self, i: &TriggerInput) -> bool {
        let idle = i.now_ms.saturating_sub(i.last_save_ms) >= self.idle_ms;
        let far = match i.previous {
            None => true,
            Some(p) => Self::distance(p, i.current) >= self.min_sim_change,
        };
        idle || far
    }
}

/// Journal retention within the kernel bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JournalRetention {
    hours: u64,
}

impl JournalRetention {
    pub fn new(hours: i64) -> Result<JournalRetention> {
        if hours < MIN_RETENTION_HOURS as i64 {
            bail!("journal.retention_hours = {hours}: the kernel minimum is {MIN_RETENTION_HOURS}");
        }
        if (hours as u64).checked_mul(3_600_000).is_none() {
            bail!("journal.retention_hours = {hours}: too large to express in milliseconds");
        }
        Ok(JournalRetention {
            hours: hours as u64,
        })
    }

    pub fn hours(&self) -> u64 {
        self.hours
    }

    pub fn millis(&self) -> u64 {
        self.hours * 3_600_000
    }
}

/// Result of `Vault::journal_prune`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneReport {
    pub records_removed: u64,
    pub records_kept: u64,
    pub blobs_removed: u64,
}

fn journal_dir(v: &Vault) -> PathBuf {
    v.store_dir().join("journal")
}

fn journal_file(v: &Vault, doc: &str) -> PathBuf {
    journal_dir(v).join(format!("{}.jsonl", hash32(doc)))
}

fn blob_path(v: &Vault, h: &str) -> PathBuf {
    journal_dir(v).join("blobs").join(&h[..2]).join(h)
}

fn content_hash(bytes: &[u8]) -> String {
    let mut h = blake3::hash(bytes).to_hex().to_string();
    h.truncate(32);
    h
}

fn read_records(p: &Path) -> Result<Vec<JournalRecord>> {
    if !p.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(p).with_context(|| format!("read {}", p.display()))?;
    let mut out: Vec<JournalRecord> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let r: JournalRecord = serde_json::from_str(line)
            .with_context(|| format!("{} line {}", p.display(), i + 1))?;
        if let Some(prev) = out.last() {
            if r.seq != prev.seq + 1 || r.at_ms < prev.at_ms {
                bail!("{} line {}: seq/time out of order", p.display(), i + 1);
            }
        }
        out.push(r);
    }
    Ok(out)
}

fn write_records(p: &Path, recs: &[JournalRecord]) -> Result<()> {
    let mut buf = Vec::new();
    for r in recs {
        serde_json::to_writer(&mut buf, r)?;
        buf.push(b'\n');
    }
    let tmp = p.with_extension("jsonl.tmp");
    fs::write(&tmp, buf)?;
    fs::rename(&tmp, p)?;
    Ok(())
}

/// Normalized block texts of a document text, one per line.
fn block_text(doc: &str, text: &str) -> Result<(String, String)> {
    let (drafts, _) = parse_document(text, doc)?;
    let sid = snapshot_id(drafts.iter().map(|d| d.version_id.as_str()));
    let joined = drafts
        .iter()
        .map(|d| d.normalized_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Ok((sid, joined))
}

impl Vault {
    /// The journal of a document, oldest first.
    pub fn journal(&self, doc: &str) -> Result<Vec<JournalRecord>> {
        read_records(&journal_file(self, doc))
    }

    /// Bytes of a journal record.
    pub fn journal_bytes(&self, r: &JournalRecord) -> Result<Vec<u8>> {
        let b = fs::read(blob_path(self, &r.content_hash))
            .with_context(|| format!("journal blob {} missing", r.content_hash))?;
        if content_hash(&b) != r.content_hash || b.len() as u64 != r.bytes_len {
            bail!("journal blob {} does not match its record", r.content_hash);
        }
        Ok(b)
    }

    /// Appends one save of `doc` (UTF-8 Markdown) at `now_ms`.
    pub fn journal_append(
        &mut self,
        doc: &str,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<JournalRecord> {
        if doc.is_empty() {
            bail!("empty doc_path");
        }
        std::str::from_utf8(bytes).with_context(|| format!("journal {doc}: not UTF-8"))?;
        let p = journal_file(self, doc);
        let recs = read_records(&p)?;
        if let Some(last) = recs.last() {
            if now_ms < last.at_ms {
                bail!(
                    "journal {doc}: time {now_ms} before the last save {}",
                    last.at_ms
                );
            }
        }
        let h = content_hash(bytes);
        write_immutable(&blob_path(self, &h), bytes)?;
        let r = JournalRecord {
            seq: recs.last().map_or(0, |l| l.seq + 1),
            at_ms: now_ms,
            content_hash: h,
            bytes_len: bytes.len() as u64,
        };
        let mut line = serde_json::to_vec(&r)?;
        line.push(b'\n');
        fs::create_dir_all(journal_dir(self))?;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&p)?;
        std::io::Write::write_all(&mut f, &line)?;
        f.sync_data()?;
        Ok(r)
    }

    /// Whether the latest save of `doc` is the document's current version.
    pub fn journal_versioned(&self, doc: &str) -> Result<bool> {
        let Some(last) = self.journal(doc)?.pop() else {
            return Ok(true);
        };
        let text = String::from_utf8(self.journal_bytes(&last)?)?;
        let (sid, _) = block_text(doc, &text)?;
        Ok(self.snapshot(doc)?.is_some_and(|h| h.snapshot_id == sid))
    }

    /// Creates a version of `doc` from its latest save when `trigger` asks
    /// for one; `None` when there is no save, the latest save is already the
    /// current version, or the trigger declines.
    pub fn maybe_version(
        &mut self,
        doc: &str,
        trigger: &dyn VersionTrigger,
        now_ms: u64,
        prov: Provenance,
        matcher: MatcherVersion,
    ) -> Result<Option<IngestResult>> {
        let Some(last) = self.journal(doc)?.pop() else {
            return Ok(None);
        };
        if now_ms < last.at_ms {
            bail!(
                "maybe_version {doc}: time {now_ms} before the last save {}",
                last.at_ms
            );
        }
        let text = String::from_utf8(self.journal_bytes(&last)?)?;
        let (sid, current) = block_text(doc, &text)?;
        let head = self.snapshot(doc)?;
        if head.as_ref().is_some_and(|h| h.snapshot_id == sid) {
            return Ok(None);
        }
        let previous = match &head {
            None => None,
            Some(h) => Some(
                self.snapshot_blocks(h)?
                    .iter()
                    .map(|b| b.normalized_text.as_str().to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        };
        let input = TriggerInput {
            now_ms,
            last_save_ms: last.at_ms,
            previous: previous.as_deref(),
            current: &current,
        };
        if !trigger.should_version(&input) {
            return Ok(None);
        }
        Ok(Some(self.ingest(doc, &text, prov, matcher)?))
    }

    /// Prunes journal records older than the retention window. The latest
    /// save of a document is always kept, and nothing is pruned from a
    /// document whose latest save is not versioned (a version never carries
    /// less than the journal). Unreferenced blobs are removed. No event.
    pub fn journal_prune(
        &mut self,
        now_ms: u64,
        retention: JournalRetention,
    ) -> Result<PruneReport> {
        let dir = journal_dir(self);
        let mut rep = PruneReport {
            records_removed: 0,
            records_kept: 0,
            blobs_removed: 0,
        };
        if !dir.exists() {
            return Ok(rep);
        }
        let cutoff = now_ms.saturating_sub(retention.millis());
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<_>>()?;
        files.retain(|p| p.extension().is_some_and(|x| x == "jsonl"));
        files.sort();
        // the doc_path of a journal file is not stored; find it among the
        // vault's documents (a journal of a never-versioned doc is kept whole)
        let docs: Vec<(String, PathBuf)> = self
            .doc_paths()
            .into_iter()
            .map(|d| {
                let p = journal_file(self, &d);
                (d, p)
            })
            .collect();
        let mut referenced: BTreeSet<String> = BTreeSet::new();
        for f in &files {
            let recs = read_records(f)?;
            let doc = docs.iter().find(|(_, p)| p == f).map(|(d, _)| d.clone());
            let prunable = match &doc {
                Some(d) => self.journal_versioned(d)?,
                None => false,
            };
            let n = recs.len();
            let keep: Vec<JournalRecord> = recs
                .into_iter()
                .enumerate()
                .filter(|(i, r)| !prunable || *i + 1 == n || r.at_ms >= cutoff)
                .map(|(_, r)| r)
                .collect();
            rep.records_removed += (n - keep.len()) as u64;
            rep.records_kept += keep.len() as u64;
            referenced.extend(keep.iter().map(|r| r.content_hash.clone()));
            if keep.len() != n {
                write_records(f, &keep)?;
            }
        }
        let blobs = dir.join("blobs");
        if blobs.exists() {
            let mut sub: Vec<PathBuf> = fs::read_dir(&blobs)?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<_>>()?;
            sub.sort();
            for d in sub {
                let mut entries: Vec<PathBuf> = fs::read_dir(&d)?
                    .map(|e| e.map(|e| e.path()))
                    .collect::<std::io::Result<_>>()?;
                entries.sort();
                for b in entries {
                    let name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !referenced.contains(name) {
                        fs::remove_file(&b)?;
                        rep.blobs_removed += 1;
                    }
                }
            }
        }
        Ok(rep)
    }

    /// The snapshot an index may hold for `doc`: its latest **version**
    /// (spec §11: unversioned content is never indexed); journal saves that
    /// were not versioned are never visible here.
    pub fn indexable_snapshot(&self, doc: &str) -> Result<Option<Snapshot>> {
        self.snapshot(doc)
    }
}
