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
pub fn finalize(store: &Store, ranked: Vec<Scored>, k: usize) -> Vec<Hit> {
    finalize_ix(store, ranked, k)
        .into_iter()
        .map(|(_, h)| h)
        .collect()
}

/// `finalize` with each hit's storage position.
pub fn finalize_ix(store: &Store, mut ranked: Vec<Scored>, k: usize) -> Vec<(usize, Hit)> {
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
            (
                i,
                Hit {
                    doc_path: r.doc_path.clone(),
                    chunk_id: r.chunk_id.clone(),
                    lineage_ids: r.lineage_ids.clone(),
                    layer,
                    score,
                },
            )
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

/// Reciprocal rank fusion constant and the candidates taken from each side
/// (GOAL: `1/(60 + rank)`, top 50; prototype `RRF_K`, `CANDIDATES`).
pub const RRF_K: f64 = 60.0;
pub const CANDIDATES: usize = 50;

/// Exact cosine against every eligible record, best first.
pub fn cosine_ranking(store: &Store, eligible: &[(usize, Layer)], q: &[f32]) -> Vec<Scored> {
    let mut out: Vec<Scored> = eligible
        .iter()
        .map(|&(i, l)| (i, l, crate::vectors::cosine(q, &store.vectors[i])))
        .collect();
    sort_scored(store, &mut out);
    out
}

/// `Σ 1/(RRF_K + rank)` over the lists (rank from 1), best first.
pub fn rrf(store: &Store, lists: &[&[Scored]]) -> Vec<Scored> {
    let mut fused: BTreeMap<usize, (Layer, f64)> = BTreeMap::new();
    for list in lists {
        for (r, &(i, l, _)) in list.iter().enumerate() {
            fused.entry(i).or_insert((l, 0.0)).1 += 1.0 / (RRF_K + (r + 1) as f64);
        }
    }
    let mut out: Vec<Scored> = fused.into_iter().map(|(i, (l, s))| (i, l, s)).collect();
    sort_scored(store, &mut out);
    out
}

fn query_vector(
    embedder: &dyn crate::embed::Embedder,
    store: &Store,
    query: &str,
) -> Result<Vec<f32>> {
    if embedder.id() != store.manifest.embedder_id {
        bail!(
            "query embedder {} != index embedder {}",
            embedder.id(),
            store.manifest.embedder_id
        );
    }
    embedder
        .embed(&[query])?
        .pop()
        .context("embedder returned no vector")
}

/// Unweighted ranking of the eligible records of `view` for `mode` (the one
/// implementation behind `flat`, `lexical`, `hybrid` and `Searcher`).
pub fn ranked(
    store: &Store,
    embedder: &dyn crate::embed::Embedder,
    view: &View,
    mode: Mode,
    query: &str,
    include_proposal: bool,
) -> Result<Vec<Scored>> {
    let eligible = view.eligible(include_proposal);
    Ok(match mode {
        Mode::Lexical => bm25(store, &eligible, query)?,
        Mode::Flat => cosine_ranking(store, &eligible, &query_vector(embedder, store, query)?),
        Mode::Hybrid => {
            let q = query_vector(embedder, store, query)?;
            let mut dense = cosine_ranking(store, &eligible, &q);
            dense.truncate(CANDIDATES);
            let mut sparse = bm25(store, &eligible, query)?;
            sparse.truncate(CANDIDATES);
            rrf(store, &[&dense, &sparse])
        }
    })
}

/// Vector retrieval (exact cosine) as of `view`'s checkpoint.
pub fn flat(
    store: &Store,
    embedder: &dyn crate::embed::Embedder,
    view: &View,
    query: &str,
    k: usize,
    include_proposal: bool,
) -> Result<Vec<Hit>> {
    let r = ranked(store, embedder, view, Mode::Flat, query, include_proposal)?;
    Ok(finalize(store, r, k))
}

/// Hybrid retrieval: RRF of the top `CANDIDATES` of the cosine and BM25
/// rankings (BM25 lists only records with a query term).
pub fn hybrid(
    store: &Store,
    embedder: &dyn crate::embed::Embedder,
    view: &View,
    query: &str,
    k: usize,
    include_proposal: bool,
) -> Result<Vec<Hit>> {
    let r = ranked(store, embedder, view, Mode::Hybrid, query, include_proposal)?;
    Ok(finalize(store, r, k))
}

/// Retrieval mode (GOAL: `flat`, `lexical`, `hybrid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Flat,
    Lexical,
    Hybrid,
}

impl Mode {
    pub fn parse(s: &str) -> Result<Mode> {
        Ok(match s {
            "flat" => Mode::Flat,
            "lexical" => Mode::Lexical,
            "hybrid" => Mode::Hybrid,
            _ => bail!("unknown search mode {s:?} (flat|lexical|hybrid)"),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Flat => "flat",
            Mode::Lexical => "lexical",
            Mode::Hybrid => "hybrid",
        }
    }
}

/// One search: its checkpoint, hits and consumption trace (spec §8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub checkpoint_id: String,
    pub mode: Mode,
    pub include_proposal: bool,
    pub query_hash: String,
    pub trace_id: String,
    pub hits: Vec<Hit>,
}

/// `query_hash = H(query)` (spec §3 hash: blake3, 32 lowercase hex).
pub fn query_hash(query: &str) -> String {
    mdkcore::hash32(query)
}

/// Searches with one store and embedder for one consumer; views are cached
/// per checkpoint (a checkpoint is immutable, spec §8).
pub struct Searcher<'a> {
    pub store: &'a Store,
    pub embedder: &'a dyn crate::embed::Embedder,
    pub consumer: String,
    views: HashMap<String, View>,
}

impl<'a> Searcher<'a> {
    pub fn new(
        store: &'a Store,
        embedder: &'a dyn crate::embed::Embedder,
        consumer: &str,
    ) -> Result<Searcher<'a>> {
        mdkcore::check_trace_input(consumer, &query_hash(""))?;
        if consumer.trim().is_empty() {
            bail!("a search requires a consumer");
        }
        Ok(Searcher {
            store,
            embedder,
            consumer: consumer.to_string(),
            views: HashMap::new(),
        })
    }

    /// Ranked hits as of checkpoint `at` (id or name; `None`: the checkpoint
    /// of the vault's current state, created if missing), then one trace
    /// line: every block of every returned chunk (`record_trace`, spec §8).
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &mut self,
        vault: &mut Vault,
        mode: Mode,
        query: &str,
        k: usize,
        at: Option<&str>,
        include_proposal: bool,
    ) -> Result<SearchResult> {
        let cp = match at {
            Some(a) => vault.find_checkpoint(a)?,
            None => vault.head_checkpoint()?,
        };
        if !self.views.contains_key(&cp.checkpoint_id) {
            let v = View::new(vault, self.store, &cp.checkpoint_id)?;
            self.views.insert(cp.checkpoint_id.clone(), v);
        }
        let view = &self.views[&cp.checkpoint_id];
        let ranked = ranked(
            self.store,
            self.embedder,
            view,
            mode,
            query,
            include_proposal,
        )?;
        let picked = finalize_ix(self.store, ranked, k);
        let returned = picked
            .iter()
            .flat_map(|(i, _)| {
                let r = &self.store.records[*i];
                r.blocks.iter().map(|b| mdkcore::TraceEntry {
                    doc_path: r.doc_path.clone(),
                    lineage_id: b.lineage_id.clone(),
                    version_id: b.version_id.clone(),
                })
            })
            .collect();
        let qh = query_hash(query);
        let trace = vault.record_trace(&cp.checkpoint_id, &self.consumer, &qh, returned)?;
        Ok(SearchResult {
            checkpoint_id: cp.checkpoint_id,
            mode,
            include_proposal,
            query_hash: qh,
            trace_id: trace.trace_id,
            hits: picked.into_iter().map(|(_, h)| h).collect(),
        })
    }
}
