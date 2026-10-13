//! Vault: object store + event log + per-document heads (SPEC §2–4, §7).

use crate::events::{append_events, read_events, Event, EventKind};
use crate::model::{hash32, Block, Provenance};
use crate::parse::heading_paths;
use crate::store::{self, BlockObject, SnapshotObject};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub doc_path: String,
    pub blocks: Vec<(String, String)>,
    pub frontmatter: Option<String>,
    pub parent_snapshot: Option<String>,
}

/// `snapshot_id = blake3(version_ids joined by "\n")`.
pub fn snapshot_id<'a, I: IntoIterator<Item = &'a str>>(vids: I) -> String {
    hash32(&vids.into_iter().collect::<Vec<_>>().join("\n"))
}

/// Replay state of one document.
#[derive(Debug, Clone, Default)]
pub struct DocState {
    /// live lineage → version
    pub live: BTreeMap<String, String>,
    /// lineage → parents of its current version
    pub parents: HashMap<String, Vec<String>>,
    /// lineage → provenance of its current version
    pub provenance: HashMap<String, Provenance>,
    /// every lineage id ever assigned in this document
    pub used: HashSet<String>,
    pub head: Option<Snapshot>,
}

fn meta_str(ev: &Event, key: &str) -> Option<String> {
    ev.meta
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn meta_list(ev: &Event, key: &str) -> Vec<String> {
    ev.meta
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn one<'a>(v: &'a [String], what: &str, ev: &Event) -> Result<&'a str> {
    if v.len() != 1 {
        bail!("event seq {}: {:?} expects one {what}", ev.seq, ev.kind);
    }
    Ok(&v[0])
}

/// Applies one event to a document state. `SnapshotCommitted` is verified
/// against the state built from the preceding block events.
pub fn apply_event(st: &mut DocState, ev: &Event) -> Result<()> {
    let prov = || -> Provenance {
        ev.meta
            .get("provenance")
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .unwrap_or_default()
    };
    match ev.kind {
        EventKind::BlockCreated => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let v = one(&ev.new_version_ids, "new version", ev)?;
            if st.live.contains_key(l) {
                bail!("event seq {}: lineage {l} already live", ev.seq);
            }
            st.live.insert(l.into(), v.into());
            st.parents.insert(l.into(), Vec::new());
            st.provenance.insert(l.into(), prov());
            st.used.insert(l.into());
        }
        EventKind::BlockUpdated => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let old = one(&ev.old_version_ids, "old version", ev)?;
            let v = one(&ev.new_version_ids, "new version", ev)?;
            if st.live.get(l).map(String::as_str) != Some(old) {
                bail!(
                    "event seq {}: update of {l} does not match live version",
                    ev.seq
                );
            }
            st.live.insert(l.into(), v.into());
            st.parents.insert(l.into(), vec![old.into()]);
            st.provenance.insert(l.into(), prov());
        }
        EventKind::BlockMoved => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            if !st.live.contains_key(l) {
                bail!("event seq {}: move of unknown lineage {l}", ev.seq);
            }
        }
        EventKind::BlockSplit | EventKind::BlockMerged => {
            let from = meta_list(ev, "from_lineages");
            if from.len() != ev.old_version_ids.len()
                || ev.lineage_ids.len() != ev.new_version_ids.len()
            {
                bail!("event seq {}: malformed {:?}", ev.seq, ev.kind);
            }
            for (l, old) in from.iter().zip(&ev.old_version_ids) {
                if st.live.remove(l).as_deref() != Some(old.as_str()) {
                    bail!("event seq {}: source lineage {l} not live at {old}", ev.seq);
                }
                st.parents.remove(l);
                st.provenance.remove(l);
            }
            for (l, v) in ev.lineage_ids.iter().zip(&ev.new_version_ids) {
                if st.live.contains_key(l) {
                    bail!("event seq {}: lineage {l} already live", ev.seq);
                }
                st.live.insert(l.clone(), v.clone());
                st.parents.insert(l.clone(), ev.old_version_ids.clone());
                st.provenance.insert(l.clone(), prov());
                st.used.insert(l.clone());
            }
        }
        EventKind::BlockDeleted => {
            let l = one(&ev.lineage_ids, "lineage", ev)?;
            let old = one(&ev.old_version_ids, "old version", ev)?;
            if st.live.remove(l).as_deref() != Some(old) {
                bail!(
                    "event seq {}: delete of {l} does not match live version",
                    ev.seq
                );
            }
            st.parents.remove(l);
            st.provenance.remove(l);
        }
        EventKind::SnapshotCommitted => {
            let sid = ev
                .snapshot_id
                .clone()
                .context("SnapshotCommitted without snapshot_id")?;
            if ev.lineage_ids.len() != ev.new_version_ids.len() {
                bail!(
                    "event seq {}: snapshot lineage/version length mismatch",
                    ev.seq
                );
            }
            let committed: BTreeMap<String, String> = ev
                .lineage_ids
                .iter()
                .cloned()
                .zip(ev.new_version_ids.iter().cloned())
                .collect();
            if committed.len() != ev.lineage_ids.len() || committed != st.live {
                bail!(
                    "event seq {}: snapshot does not match replayed block events",
                    ev.seq
                );
            }
            let computed = snapshot_id(ev.new_version_ids.iter().map(String::as_str));
            if computed != sid {
                bail!(
                    "event seq {}: snapshot_id {sid} != computed {computed}",
                    ev.seq
                );
            }
            let parent = st.head.as_ref().map(|s| s.snapshot_id.clone());
            if meta_str(ev, "parent_snapshot") != parent {
                bail!("event seq {}: parent_snapshot mismatch", ev.seq);
            }
            st.head = Some(Snapshot {
                snapshot_id: sid,
                doc_path: ev.doc_path.clone(),
                blocks: committed_pairs(ev),
                frontmatter: meta_str(ev, "frontmatter"),
                parent_snapshot: parent,
            });
        }
    }
    Ok(())
}

fn committed_pairs(ev: &Event) -> Vec<(String, String)> {
    ev.lineage_ids
        .iter()
        .cloned()
        .zip(ev.new_version_ids.iter().cloned())
        .collect()
}

/// Applies a sequence of events grouped by document.
pub fn replay_all(events: &[Event]) -> Result<BTreeMap<String, DocState>> {
    let mut docs: BTreeMap<String, DocState> = BTreeMap::new();
    for ev in events {
        apply_event(docs.entry(ev.doc_path.clone()).or_default(), ev)?;
    }
    Ok(docs)
}

pub struct Vault {
    root: PathBuf,
    meta_dir: PathBuf,
    next_seq: u64,
    docs: BTreeMap<String, DocState>,
    pub params: crate::lineage::MatchParams,
}

impl Vault {
    /// Opens (or initialises) `<root>/.mdkernel` and replays the log into memory.
    pub fn open(root: impl AsRef<Path>) -> Result<Vault> {
        Self::open_with(root, crate::lineage::MatchParams::default())
    }

    pub fn open_with(root: impl AsRef<Path>, params: crate::lineage::MatchParams) -> Result<Vault> {
        let root = root.as_ref().to_path_buf();
        let meta_dir = root.join(".mdkernel");
        fs::create_dir_all(meta_dir.join("objects"))?;
        fs::create_dir_all(meta_dir.join("snapshots"))?;
        let events = read_events(&meta_dir.join("events.jsonl"))?;
        let docs = replay_all(&events)?;
        Ok(Vault {
            root,
            meta_dir,
            next_seq: events.len() as u64,
            docs,
            params,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn events_path(&self) -> PathBuf {
        self.meta_dir.join("events.jsonl")
    }

    pub fn store_dir(&self) -> &Path {
        &self.meta_dir
    }

    pub fn doc_paths(&self) -> Vec<String> {
        self.docs
            .iter()
            .filter(|(_, s)| s.head.is_some())
            .map(|(d, _)| d.clone())
            .collect()
    }

    pub fn doc_state(&self, doc_path: &str) -> Option<&DocState> {
        self.docs.get(doc_path)
    }

    /// Head snapshot of a document (in-memory state built from the log).
    pub fn snapshot(&self, doc_path: &str) -> Result<Option<Snapshot>> {
        Ok(self.docs.get(doc_path).and_then(|s| s.head.clone()))
    }

    pub fn object(&self, version_id: &str) -> Result<BlockObject> {
        store::get_object(&self.meta_dir, version_id)
    }

    /// Content of a block version. Occurrence fields (`lineage_id`,
    /// `heading_path`, `parents`, `provenance`) are empty because one version
    /// can occur in several places; use `snapshot_blocks` for occurrences.
    pub fn block(&self, version_id: &str) -> Result<Block> {
        let o = self.object(version_id)?;
        Ok(Block {
            version_id: o.version_id,
            lineage_id: String::new(),
            block_type: o.block_type,
            text: o.normalized_text.clone(),
            normalized_text: o.normalized_text,
            heading_path: Vec::new(),
            heading_level: o.heading_level,
            parents: Vec::new(),
            provenance: Provenance::default(),
        })
    }

    /// Full block occurrences of a snapshot, in document order.
    pub fn snapshot_blocks(&self, snap: &Snapshot) -> Result<Vec<Block>> {
        let st = self.docs.get(&snap.doc_path);
        let objs = snap
            .blocks
            .iter()
            .map(|(_, v)| self.object(v))
            .collect::<Result<Vec<_>>>()?;
        let paths = heading_paths(
            objs.iter()
                .map(|o| (o.block_type, o.heading_level, o.normalized_text.as_str())),
        );
        Ok(snap
            .blocks
            .iter()
            .zip(objs)
            .zip(paths)
            .map(|(((l, _), o), hp)| Block {
                version_id: o.version_id,
                lineage_id: l.clone(),
                block_type: o.block_type,
                text: o.normalized_text.clone(),
                normalized_text: o.normalized_text,
                heading_path: hp,
                heading_level: o.heading_level,
                parents: st
                    .and_then(|s| s.parents.get(l).cloned())
                    .unwrap_or_default(),
                provenance: st
                    .and_then(|s| s.provenance.get(l).cloned())
                    .unwrap_or_default(),
            })
            .collect())
    }

    /// Rebuilds the last snapshot of `doc_path` from the on-disk log alone and
    /// verifies every `snapshot_id` and the stored snapshot object.
    pub fn replay(&self, doc_path: &str) -> Result<Option<Snapshot>> {
        let events = read_events(&self.events_path())?;
        let mut st = DocState::default();
        for ev in events.iter().filter(|e| e.doc_path == doc_path) {
            apply_event(&mut st, ev)?;
        }
        if let Some(s) = &st.head {
            let obj = store::get_snapshot(&self.meta_dir, &s.snapshot_id)?;
            let vids: Vec<String> = s.blocks.iter().map(|(_, v)| v.clone()).collect();
            if obj.version_ids != vids {
                bail!("snapshot object {} does not match replay", s.snapshot_id);
            }
        }
        Ok(st.head)
    }

    /// blake3 of the whole `events.jsonl` (first 32 hex chars); H4-1.
    pub fn events_hash(&self) -> Result<String> {
        let p = self.events_path();
        let bytes = if p.exists() {
            fs::read(&p)?
        } else {
            Vec::new()
        };
        let mut h = blake3::hash(&bytes).to_hex().to_string();
        h.truncate(32);
        Ok(h)
    }

    /// Low-level commit: stores objects, assigns `seq`, appends the events,
    /// then applies them to the in-memory state (which re-verifies them).
    pub fn commit(
        &mut self,
        objects: &[BlockObject],
        mut events: Vec<Event>,
    ) -> Result<Vec<Event>> {
        // verify on a copy before anything is written
        let mut staged: BTreeMap<String, DocState> = BTreeMap::new();
        for (i, ev) in events.iter_mut().enumerate() {
            ev.seq = self.next_seq + i as u64;
            let st = staged
                .entry(ev.doc_path.clone())
                .or_insert_with(|| self.docs.get(&ev.doc_path).cloned().unwrap_or_default());
            apply_event(st, ev)?;
        }
        for o in objects {
            store::put_object(&self.meta_dir, o)?;
        }
        for ev in &events {
            if ev.kind == EventKind::SnapshotCommitted {
                let sid = ev
                    .snapshot_id
                    .clone()
                    .context("SnapshotCommitted without snapshot_id")?;
                store::put_snapshot(
                    &self.meta_dir,
                    &SnapshotObject {
                        snapshot_id: sid,
                        version_ids: ev.new_version_ids.clone(),
                    },
                )?;
            }
        }
        append_events(&self.events_path(), &events)?;
        self.next_seq += events.len() as u64;
        self.docs.extend(staged);
        Ok(events)
    }
}
