//! Retrieval as of a checkpoint (GOAL P1-C "Hibrit", "Kontrol noktasına göre
//! sorgu"; spec §7, §9).
//!
//! A `View` is the set of records a checkpoint names (the snapshots of its
//! `docs` map) with each record's layer **as of that checkpoint**: the lowest
//! layer of the blocks it covers (`derived` < `proposal` < `canonical`).
//! Layers come from the replayed log at the checkpoint, not from the index,
//! because a layer transition (spec §7) changes no `snapshot_id`.
//!
//! Eligible records: `canonical`; `proposal` only with `include_proposal`;
//! `derived` never. Ranking statistics (BM25 `N`, `df`, `avgdl`) are those
//! of the eligible records, so the result depends only on the checkpoint,
//! the query and the index configuration (spec §9) — not on which other
//! chunk versions the index holds.
//!
//! Every ranking is total: score descending, then `(doc_path, chunk_id)`
//! ascending (unique within a view). With `include_proposal`, a proposal
//! hit's score is multiplied by `PROPOSAL_WEIGHT`, the top `k` are taken,
//! and the proposal hits are placed after the canonical ones (labelled by
//! `layer`).

use crate::store::{Store, BM25_B, BM25_K1};
use crate::tokenize::tokens;
use anyhow::{bail, Context, Result};
use mdkcore::{Checkpoint, Layer, Vault};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub const PROPOSAL_WEIGHT: f64 = 0.5;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub doc_path: String,
    pub chunk_id: String,
    pub lineage_ids: Vec<String>,
    pub layer: Layer,
    pub score: f64,
}

fn rank(l: Layer) -> u8 {
    match l {
        Layer::Derived => 0,
        Layer::Proposal => 1,
        Layer::Canonical => 2,
    }
}

/// The records of a checkpoint, with their layers then.
pub struct View {
    pub checkpoint: Checkpoint,
    /// `(storage position, layer)`, in `(doc_path, chunk_id)` order
    pub entries: Vec<(usize, Layer)>,
}

impl View {
    pub fn new(vault: &Vault, store: &Store, checkpoint: &str) -> Result<View> {
        let (cp, st) = vault.state_at(checkpoint)?;
        let mut entries = Vec::new();
        for (doc, sid) in &cp.docs {
            let ds = st
                .docs
                .get(doc)
                .with_context(|| format!("{doc}: no state at checkpoint"))?;
            for &i in store.snapshot(doc, sid)? {
                let r = &store.records[i];
                let mut layer = Layer::Canonical;
                for b in &r.blocks {
                    let l = *ds.layer.get(&b.lineage_id).with_context(|| {
                        format!("{doc}: lineage {} has no layer at checkpoint", b.lineage_id)
                    })?;
                    if rank(l) < rank(layer) {
                        layer = l;
                    }
                }
                entries.push((i, layer));
            }
        }
        entries.sort_by(|a, b| {
            let (ra, rb) = (&store.records[a.0], &store.records[b.0]);
            (&ra.doc_path, &ra.chunk_id).cmp(&(&rb.doc_path, &rb.chunk_id))
        });
        if entries.windows(2).any(|w| {
            let (ra, rb) = (&store.records[w[0].0], &store.records[w[1].0]);
            (&ra.doc_path, &ra.chunk_id) == (&rb.doc_path, &rb.chunk_id)
        }) {
            bail!(
                "checkpoint {}: a (doc_path, chunk_id) occurs twice",
                cp.checkpoint_id
            );
        }
        Ok(View {
            checkpoint: cp,
            entries,
        })
    }

    /// Eligible `(storage position, layer)` (never `derived`).
    pub fn eligible(&self, include_proposal: bool) -> Vec<(usize, Layer)> {
        self.entries
            .iter()
            .copied()
            .filter(|(_, l)| match l {
                Layer::Canonical => true,
                Layer::Proposal => include_proposal,
                Layer::Derived => false,
            })
            .collect()
    }
}

/// A scored candidate: `(storage position, layer, score)`.
pub type Scored = (usize, Layer, f64);

/// Score descending, then `(doc_path, chunk_id)`.
pub fn sort_scored(store: &Store, v: &mut [Scored]) {
    v.sort_by(|a, b| {
        b.2.total_cmp(&a.2).then_with(|| {
            let (ra, rb) = (&store.records[a.0], &store.records[b.0]);
            (&ra.doc_path, &ra.chunk_id).cmp(&(&rb.doc_path, &rb.chunk_id))
        })
    });
}

/// BM25 over the eligible records (tantivy's IDF `ln(1 + (N − n + 0.5) /
/// (n + 0.5))`, `k1` 1.2, `b` 0.75, document length = token count); every
/// record with a query term, best first. Repeated query tokens count again
/// (as the prototype's `BM25Okapi.get_scores`).
pub fn bm25(store: &Store, eligible: &[(usize, Layer)], query: &str) -> Result<Vec<Scored>> {
    let q = tokens(query);
    if q.is_empty() || eligible.is_empty() {
        return Ok(Vec::new());
    }
    let pos: HashMap<usize, usize> = eligible
        .iter()
        .enumerate()
        .map(|(j, (i, _))| (*i, j))
        .collect();
    let n = eligible.len() as f64;
    let total: u64 = eligible
        .iter()
        .map(|(i, _)| u64::from(store.records[*i].lexical_len))
        .sum();
    let avgdl = total as f64 / n;
    let mut postings: BTreeMap<&str, Vec<(usize, u32)>> = BTreeMap::new();
    for t in &q {
        if postings.contains_key(t.as_str()) {
            continue;
        }
        let mut p = Vec::new();
        for (ord, tf) in store.lexical.postings(t)? {
            if let Some(&j) = pos.get(&usize::try_from(ord)?) {
                p.push((j, tf));
            }
        }
        postings.insert(t, p);
    }
    let mut score = vec![0f64; eligible.len()];
    let mut hit = vec![false; eligible.len()];
    for t in &q {
        let p = &postings[t.as_str()];
        if p.is_empty() {
            continue;
        }
        let df = p.len() as f64;
        let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
        for &(j, tf) in p {
            let tf = f64::from(tf);
            let dl = f64::from(store.records[eligible[j].0].lexical_len);
            score[j] +=
                idf * tf * (BM25_K1 + 1.0) / (tf + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / avgdl));
            hit[j] = true;
        }
    }
    let mut out: Vec<Scored> = (0..eligible.len())
        .filter(|&j| hit[j])
        .map(|j| (eligible[j].0, eligible[j].1, score[j]))
        .collect();
    sort_scored(store, &mut out);
    Ok(out)
}

/// The returned list: proposals down-weighted, top `k`, canonical first.
pub fn finalize(store: &Store, mut ranked: Vec<Scored>, k: usize) -> Vec<Hit> {
    for s in &mut ranked {
        if s.1 == Layer::Proposal {
            s.2 *= PROPOSAL_WEIGHT;
        }
    }
    sort_scored(store, &mut ranked);
    ranked.truncate(k);
    let (canon, prop): (Vec<Scored>, Vec<Scored>) =
        ranked.into_iter().partition(|s| s.1 == Layer::Canonical);
    canon
        .into_iter()
        .chain(prop)
        .map(|(i, layer, score)| {
            let r = &store.records[i];
            Hit {
                doc_path: r.doc_path.clone(),
                chunk_id: r.chunk_id.clone(),
                lineage_ids: r.lineage_ids.clone(),
                layer,
                score,
            }
        })
        .collect()
}

/// Lexical retrieval as of `view`'s checkpoint.
pub fn lexical(
    store: &Store,
    view: &View,
    query: &str,
    k: usize,
    include_proposal: bool,
) -> Result<Vec<Hit>> {
    let ranked = bm25(store, &view.eligible(include_proposal), query)?;
    Ok(finalize(store, ranked, k))
}
