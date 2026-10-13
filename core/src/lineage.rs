//! Lineage matcher (SPEC §5; v3: SPEC_DELTA §5.3). `MatcherVersion::V2`
//! reproduces the v2-final matcher byte for byte; `V3` adds K4 (the dominant
//! part keeps the lineage), drops the minimum part ratio and adds the
//! merge-direction residual check.

use crate::events::{Event, EventKind};
use crate::model::{hash32, Block, BlockDraft, BlockType, Provenance};
use anyhow::{bail, Result};
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

/// V2 similarity-assignment threshold (SPEC §5.3 step 4). Set by T6 tuning; frozen.
pub const THETA: f64 = 0.60;
/// V2 split/merge threshold (SPEC §5.3 steps 2–3). Set by T6 tuning; frozen.
pub const THETA_SPLIT: f64 = 0.70;
/// V3 similarity-assignment threshold. Written only by the T24 tuning phase.
pub const THETA_V3: f64 = 0.50;
/// V3 split/merge threshold. Written only by the T24 tuning phase.
pub const THETA_SPLIT_V3: f64 = 0.65;
/// V2 only: minimum part length relative to the whole for split/merge candidates.
pub const MIN_PART_RATIO: f64 = 0.15;
/// Residual check margin (SPEC §5.3 steps 5 and, in V3, 6).
pub const RESIDUAL_MARGIN: f64 = 0.10;
/// Largest split/merge window (SPEC §5.3 steps 2–3).
pub const MAX_WINDOW: usize = 4;
/// V3: largest old-block window of the merge-direction residual check (step 6).
pub const MAX_RESIDUAL_MERGE: usize = 3;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum MatcherVersion {
    V2,
    #[default]
    V3,
}

impl MatcherVersion {
    /// The value written to `meta.matcher_version`.
    pub fn as_str(self) -> &'static str {
        match self {
            MatcherVersion::V2 => "v2",
            MatcherVersion::V3 => "v3",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "v2" => Ok(MatcherVersion::V2),
            "v3" => Ok(MatcherVersion::V3),
            _ => bail!("unknown matcher version {s:?} (expected v2 or v3)"),
        }
    }

    /// Compiled THETA / THETA_SPLIT of this version.
    pub fn defaults(self) -> MatchParams {
        match self {
            MatcherVersion::V2 => MatchParams {
                theta: THETA,
                theta_split: THETA_SPLIT,
            },
            MatcherVersion::V3 => MatchParams {
                theta: THETA_V3,
                theta_split: THETA_SPLIT_V3,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchParams {
    pub theta: f64,
    pub theta_split: f64,
}

/// The V2 constants (unchanged from v2-final).
impl Default for MatchParams {
    fn default() -> Self {
        MatcherVersion::V2.defaults()
    }
}

/// A block of the previous snapshot, as the matcher sees it.
#[derive(Debug, Clone)]
pub struct PrevBlock {
    pub lineage_id: String,
    pub version_id: String,
    pub block_type: BlockType,
    pub normalized_text: String,
    pub heading_path: Vec<String>,
    pub parents: Vec<String>,
    pub provenance: Provenance,
}

impl From<&Block> for PrevBlock {
    fn from(b: &Block) -> Self {
        PrevBlock {
            lineage_id: b.lineage_id.clone(),
            version_id: b.version_id.clone(),
            block_type: b.block_type,
            normalized_text: b.normalized_text.clone(),
            heading_path: b.heading_path.clone(),
            parents: b.parents.clone(),
            provenance: b.provenance.clone(),
        }
    }
}

/// `lineage_id = blake3("lineage\n" + doc_path + "\n" + version_id + "\n" + k)` (SPEC §5.1).
pub fn lineage_formula(doc_path: &str, version_id: &str, k: usize) -> String {
    hash32(&format!("lineage\n{doc_path}\n{version_id}\n{k}"))
}

/// Context anchor (SPEC §5.2).
pub fn ctx_anchor(prev: Option<&str>, vid: &str, next: Option<&str>) -> String {
    hash32(&format!(
        "{}\n{}\n{}",
        prev.unwrap_or(""),
        vid,
        next.unwrap_or("")
    ))
}

/// `strsim::normalized_levenshtein` (0–1).
pub fn sim(a: &str, b: &str) -> f64 {
    strsim::normalized_levenshtein(a, b)
}

/// Upper bound of `sim` from lengths alone (Levenshtein ≥ length difference).
fn sim_bound(la: usize, lb: usize) -> f64 {
    let (lo, hi) = if la < lb { (la, lb) } else { (lb, la) };
    if hi == 0 {
        1.0
    } else {
        lo as f64 / hi as f64
    }
}

fn round6(x: f64) -> f64 {
    (x * 1e6).round() / 1e6
}

#[derive(Debug, Clone, PartialEq)]
enum NewKind {
    Exact {
        old: usize,
        anchor: &'static str,
    },
    Updated {
        old: usize,
        score: f64,
    },
    /// part of a split of `old`; `group` lists all parts (new indices)
    Split {
        old: usize,
        group: Vec<usize>,
        score: f64,
        residual: bool,
    },
    Merged {
        olds: Vec<usize>,
        score: f64,
        residual: bool,
    },
    Created,
}

pub struct MatchResult {
    pub blocks: Vec<Block>,
    /// events without `seq` and without the trailing `SnapshotCommitted`
    pub events: Vec<Event>,
}

fn event(
    doc: &str,
    kind: EventKind,
    l: Vec<String>,
    old: Vec<String>,
    new: Vec<String>,
    meta: Value,
) -> Event {
    Event {
        seq: 0,
        doc_path: doc.to_string(),
        kind,
        lineage_ids: l,
        old_version_ids: old,
        new_version_ids: new,
        snapshot_id: None,
        meta,
    }
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// V3 split/merge part condition: at least one word.
fn has_word(s: &str) -> bool {
    s.split_whitespace().next().is_some()
}

/// Index of the longest element (ties → first).
fn longest(idx: &[usize], len: &[usize]) -> usize {
    let mut best = idx[0];
    for &k in &idx[1..] {
        if len[k] > len[best] {
            best = k;
        }
    }
    best
}

/// Longest strictly increasing subsequence; returns a membership mask.
fn lis_mask(seq: &[usize]) -> Vec<bool> {
    let n = seq.len();
    let mut tails: Vec<usize> = Vec::new(); // indices into seq
    let mut prev: Vec<Option<usize>> = vec![None; n];
    for i in 0..n {
        let pos = tails.partition_point(|&t| seq[t] < seq[i]);
        if pos > 0 {
            prev[i] = Some(tails[pos - 1]);
        }
        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }
    }
    let mut mask = vec![false; n];
    let mut cur = tails.last().copied();
    while let Some(i) = cur {
        mask[i] = true;
        cur = prev[i];
    }
    mask
}

/// SPEC §5.3: matches `drafts` against the previous snapshot of `doc_path`.
/// `used` holds every lineage id ever assigned in this document; new lineages
/// never reuse one of them (see ASSUMPTIONS, T3).
pub fn match_blocks(
    doc_path: &str,
    prev: &[PrevBlock],
    drafts: &[BlockDraft],
    used: &HashSet<String>,
    params: MatchParams,
    provenance: &Provenance,
    matcher: MatcherVersion,
) -> MatchResult {
    let v3 = matcher == MatcherVersion::V3;
    let mv = matcher.as_str();
    let n_old = prev.len();
    let n_new = drafts.len();
    let mut new_kind: Vec<Option<NewKind>> = vec![None; n_new];
    let mut old_taken = vec![false; n_old];
    let old_norm: Vec<&str> = prev.iter().map(|b| b.normalized_text.as_str()).collect();
    let new_norm: Vec<&str> = drafts.iter().map(|b| b.normalized_text.as_str()).collect();
    let old_len: Vec<usize> = old_norm.iter().map(|s| char_len(s)).collect();
    let new_len: Vec<usize> = new_norm.iter().map(|s| char_len(s)).collect();

    // ---- step 1: exact matches (with ctx / ordinal anchors for duplicates)
    let old_vid = |i: usize| prev[i].version_id.as_str();
    let new_vid = |i: usize| drafts[i].version_id.as_str();
    let old_ctx: Vec<String> = (0..n_old)
        .map(|i| {
            ctx_anchor(
                i.checked_sub(1).map(old_vid),
                old_vid(i),
                (i + 1 < n_old).then(|| old_vid(i + 1)),
            )
        })
        .collect();
    let new_ctx: Vec<String> = (0..n_new)
        .map(|i| {
            ctx_anchor(
                i.checked_sub(1).map(new_vid),
                new_vid(i),
                (i + 1 < n_new).then(|| new_vid(i + 1)),
            )
        })
        .collect();
    let mut old_by_vid: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for i in 0..n_old {
        old_by_vid.entry(old_vid(i)).or_default().push(i);
    }
    let mut new_by_vid: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for i in 0..n_new {
        new_by_vid.entry(new_vid(i)).or_default().push(i);
    }
    for (vid, news) in &new_by_vid {
        let Some(olds) = old_by_vid.get(vid) else {
            continue;
        };
        if olds.len() == 1 && news.len() == 1 {
            new_kind[news[0]] = Some(NewKind::Exact {
                old: olds[0],
                anchor: "exact",
            });
            old_taken[olds[0]] = true;
            continue;
        }
        for &n in news {
            if let Some(&o) = olds
                .iter()
                .find(|&&o| !old_taken[o] && old_ctx[o] == new_ctx[n])
            {
                new_kind[n] = Some(NewKind::Exact {
                    old: o,
                    anchor: "ctx",
                });
                old_taken[o] = true;
            }
        }
        let rest_old: Vec<usize> = olds.iter().copied().filter(|&o| !old_taken[o]).collect();
        let rest_new: Vec<usize> = news
            .iter()
            .copied()
            .filter(|&n| new_kind[n].is_none())
            .collect();
        for (&o, &n) in rest_old.iter().zip(&rest_new) {
            new_kind[n] = Some(NewKind::Exact {
                old: o,
                anchor: "ordinal",
            });
            old_taken[o] = true;
        }
    }

    // ---- step 2: split candidates (one old ↔ 2..4 consecutive unmatched new)
    {
        let mut cands: Vec<(f64, usize, usize, usize)> = Vec::new(); // (sim, old, i, j)
        for o in (0..n_old).filter(|&o| !old_taken[o]) {
            for i in 0..n_new {
                let mut parts = String::new();
                let mut plen = 0usize;
                for j in i..(i + MAX_WINDOW).min(n_new) {
                    let small = if v3 {
                        !has_word(new_norm[j])
                    } else {
                        (new_len[j] as f64) < MIN_PART_RATIO * old_len[o] as f64
                    };
                    if new_kind[j].is_some() || small {
                        break;
                    }
                    if j > i {
                        parts.push(' ');
                        plen += 1;
                    }
                    parts.push_str(new_norm[j]);
                    plen += new_len[j];
                    if j == i || sim_bound(old_len[o], plen) < params.theta_split {
                        continue;
                    }
                    let s = sim(old_norm[o], &parts);
                    if s >= params.theta_split {
                        cands.push((s, o, i, j));
                    }
                }
            }
        }
        cands.sort_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
                .then(a.3.cmp(&b.3))
        });
        for (s, o, i, j) in cands {
            if old_taken[o] || (i..=j).any(|k| new_kind[k].is_some()) {
                continue;
            }
            old_taken[o] = true;
            let group: Vec<usize> = (i..=j).collect();
            for slot in &mut new_kind[i..=j] {
                *slot = Some(NewKind::Split {
                    old: o,
                    group: group.clone(),
                    score: s,
                    residual: false,
                });
            }
        }
    }

    // ---- step 3: merge candidates (2..4 consecutive unmatched old ↔ one new)
    {
        let mut cands: Vec<(f64, usize, usize, usize)> = Vec::new(); // (sim, new, i, j)
        for n in (0..n_new).filter(|&n| new_kind[n].is_none()) {
            for i in 0..n_old {
                let mut parts = String::new();
                let mut plen = 0usize;
                for j in i..(i + MAX_WINDOW).min(n_old) {
                    let small = if v3 {
                        !has_word(old_norm[j])
                    } else {
                        (old_len[j] as f64) < MIN_PART_RATIO * new_len[n] as f64
                    };
                    if old_taken[j] || small {
                        break;
                    }
                    if j > i {
                        parts.push(' ');
                        plen += 1;
                    }
                    parts.push_str(old_norm[j]);
                    plen += old_len[j];
                    if j == i || sim_bound(new_len[n], plen) < params.theta_split {
                        continue;
                    }
                    let s = sim(&parts, new_norm[n]);
                    if s >= params.theta_split {
                        cands.push((s, n, i, j));
                    }
                }
            }
        }
        cands.sort_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
                .then(a.3.cmp(&b.3))
        });
        for (s, n, i, j) in cands {
            if new_kind[n].is_some() || (i..=j).any(|k| old_taken[k]) {
                continue;
            }
            old_taken[i..=j].fill(true);
            new_kind[n] = Some(NewKind::Merged {
                olds: (i..=j).collect(),
                score: s,
                residual: false,
            });
        }
    }

    // ---- step 4: similarity assignment (Hungarian per block type)
    let denom = n_old.max(n_new).max(1) as f64;
    let mut updated: Vec<(usize, usize, f64)> = Vec::new(); // (old, new, s)
    {
        let mut types: Vec<BlockType> = drafts.iter().map(|d| d.block_type).collect();
        types.sort();
        types.dedup();
        for bt in types {
            let olds: Vec<usize> = (0..n_old)
                .filter(|&o| !old_taken[o] && prev[o].block_type == bt)
                .collect();
            let news: Vec<usize> = (0..n_new)
                .filter(|&n| new_kind[n].is_none() && drafts[n].block_type == bt)
                .collect();
            if olds.is_empty() || news.is_empty() {
                continue;
            }
            // s ≥ THETA pairs only; below-threshold pairs carry weight 0 so
            // they never steer the assignment.
            let mut score: HashMap<(usize, usize), f64> = HashMap::new();
            for &o in &olds {
                for &n in &news {
                    let hp = if prev[o].heading_path == drafts[n].heading_path {
                        0.15
                    } else {
                        0.0
                    };
                    let pos = 0.15 * (1.0 - (o as f64 - n as f64).abs() / denom).max(0.0);
                    if 0.70 * sim_bound(old_len[o], new_len[n]) + hp + pos < params.theta {
                        continue;
                    }
                    let s = 0.70 * sim(old_norm[o], new_norm[n]) + hp + pos;
                    if s >= params.theta {
                        score.insert((o, n), s);
                    }
                }
            }
            if score.is_empty() {
                continue;
            }
            let rows: Vec<usize> = olds
                .iter()
                .copied()
                .filter(|o| news.iter().any(|n| score.contains_key(&(*o, *n))))
                .collect();
            let cols: Vec<usize> = news
                .iter()
                .copied()
                .filter(|n| olds.iter().any(|o| score.contains_key(&(*o, *n))))
                .collect();
            let w = |o: usize, n: usize| {
                score
                    .get(&(o, n))
                    .map_or(0i64, |s| (s * 1e6).round() as i64)
            };
            let pairs: Vec<(usize, usize)> = if rows.len() <= cols.len() {
                let m = Matrix::from_fn(rows.len(), cols.len(), |(r, c)| w(rows[r], cols[c]));
                let (_, asg) = kuhn_munkres(&m);
                asg.iter()
                    .enumerate()
                    .map(|(r, &c)| (rows[r], cols[c]))
                    .collect()
            } else {
                let m = Matrix::from_fn(cols.len(), rows.len(), |(c, r)| w(rows[r], cols[c]));
                let (_, asg) = kuhn_munkres(&m);
                asg.iter()
                    .enumerate()
                    .map(|(c, &r)| (rows[r], cols[c]))
                    .collect()
            };
            for (o, n) in pairs {
                if let Some(&s) = score.get(&(o, n)) {
                    updated.push((o, n, s));
                }
            }
        }
        updated.sort_by_key(|&(_, n, _)| n);
        for &(o, n, s) in &updated {
            old_taken[o] = true;
            new_kind[n] = Some(NewKind::Updated { old: o, score: s });
        }
    }

    // ---- step 5: residual check (matched part + adjacent unmatched new → split)
    for &(o, n, _) in &updated {
        if !matches!(new_kind[n], Some(NewKind::Updated { .. })) {
            continue;
        }
        let free = |k: usize, nk: &Vec<Option<NewKind>>| nk[k].is_none();
        let mut lo = n;
        while lo > 0 && n - (lo - 1) < MAX_WINDOW && free(lo - 1, &new_kind) {
            lo -= 1;
        }
        let mut hi = n;
        while hi + 1 < n_new && hi + 1 - n < MAX_WINDOW && free(hi + 1, &new_kind) {
            hi += 1;
        }
        if lo == n && hi == n {
            continue;
        }
        let base = sim(old_norm[o], new_norm[n]);
        let mut best: Option<(f64, usize, usize)> = None;
        for i in lo..=n {
            for j in n..=hi {
                if j == i || j - i + 1 > MAX_WINDOW {
                    continue;
                }
                let concat = new_norm[i..=j].join(" ");
                let s = sim(old_norm[o], &concat);
                if s > base + RESIDUAL_MARGIN && best.is_none_or(|(bs, _, _)| s > bs) {
                    best = Some((s, i, j));
                }
            }
        }
        if let Some((s, i, j)) = best {
            let group: Vec<usize> = (i..=j).collect();
            for slot in &mut new_kind[i..=j] {
                *slot = Some(NewKind::Split {
                    old: o,
                    group: group.clone(),
                    score: s,
                    residual: true,
                });
            }
        }
    }

    // ---- V3 step 6: residual check, merge direction (matched old + adjacent
    // unmatched old → merge). Windows of 2..=3 consecutive old blocks around `o`.
    if v3 {
        for &(o, n, _) in &updated {
            if !matches!(new_kind[n], Some(NewKind::Updated { .. })) {
                continue;
            }
            let mut lo = o;
            while lo > 0 && o - (lo - 1) < MAX_RESIDUAL_MERGE && !old_taken[lo - 1] {
                lo -= 1;
            }
            let mut hi = o;
            while hi + 1 < n_old && hi + 1 - o < MAX_RESIDUAL_MERGE && !old_taken[hi + 1] {
                hi += 1;
            }
            if lo == o && hi == o {
                continue;
            }
            let base = sim(old_norm[o], new_norm[n]);
            let mut best: Option<(f64, usize, usize)> = None;
            for i in lo..=o {
                for j in o..=hi {
                    if j == i || j - i + 1 > MAX_RESIDUAL_MERGE {
                        continue;
                    }
                    let concat = old_norm[i..=j].join(" ");
                    let s = sim(&concat, new_norm[n]);
                    if s > base + RESIDUAL_MARGIN && best.is_none_or(|(bs, _, _)| s > bs) {
                        best = Some((s, i, j));
                    }
                }
            }
            if let Some((s, i, j)) = best {
                old_taken[i..=j].fill(true);
                new_kind[n] = Some(NewKind::Merged {
                    olds: (i..=j).collect(),
                    score: s,
                    residual: true,
                });
            }
        }
    }

    // ---- step 6 (V3: step 7): remaining → created / deleted
    for k in new_kind.iter_mut() {
        if k.is_none() {
            *k = Some(NewKind::Created);
        }
    }
    let new_kind: Vec<NewKind> = new_kind.into_iter().map(|k| k.expect("assigned")).collect();

    // ---- lineage ids (SPEC §5.1; never reuse a lineage used in this document)
    let mut taken: HashSet<String> = used.clone();
    let mut next_k: HashMap<&str, usize> = HashMap::new();
    let mut lineage: Vec<String> = Vec::with_capacity(n_new);
    for (n, kind) in new_kind.iter().enumerate() {
        let l = match kind {
            NewKind::Exact { old, .. } | NewKind::Updated { old, .. } => {
                prev[*old].lineage_id.clone()
            }
            // K4: the longest part of a split keeps the old lineage
            NewKind::Split { old, group, .. } if v3 && longest(group, &new_len) == n => {
                prev[*old].lineage_id.clone()
            }
            // K4: a merge result keeps the lineage of the longest old block
            NewKind::Merged { olds, .. } if v3 => prev[longest(olds, &old_len)].lineage_id.clone(),
            _ => {
                let vid = new_vid(n);
                let k = next_k.entry(vid).or_insert(0);
                loop {
                    let cand = lineage_formula(doc_path, vid, *k);
                    *k += 1;
                    if taken.insert(cand.clone()) {
                        break cand;
                    }
                }
            }
        };
        lineage.push(l);
    }

    // ---- moves: exact matches outside the LIS of old positions
    let exact: Vec<usize> = (0..n_new)
        .filter(|&n| matches!(new_kind[n], NewKind::Exact { .. }))
        .collect();
    let seq: Vec<usize> = exact
        .iter()
        .map(|&n| match new_kind[n] {
            NewKind::Exact { old, .. } => old,
            _ => unreachable!(),
        })
        .collect();
    let in_lis = lis_mask(&seq);
    let moved: HashSet<usize> = exact
        .iter()
        .zip(&in_lis)
        .filter(|(_, &m)| !m)
        .map(|(&n, _)| n)
        .collect();

    // ---- blocks and events in document order
    let prov_json = serde_json::to_value(provenance).expect("provenance serialises");
    let mut blocks = Vec::with_capacity(n_new);
    let mut events = Vec::new();
    for (n, kind) in new_kind.iter().enumerate() {
        let d = &drafts[n];
        let (parents, prov) = match kind {
            NewKind::Exact { old, .. } => {
                (prev[*old].parents.clone(), prev[*old].provenance.clone())
            }
            NewKind::Updated { old, .. } => {
                (vec![prev[*old].version_id.clone()], provenance.clone())
            }
            NewKind::Split { old, .. } => (vec![prev[*old].version_id.clone()], provenance.clone()),
            NewKind::Merged { olds, .. } => (
                olds.iter().map(|&o| prev[o].version_id.clone()).collect(),
                provenance.clone(),
            ),
            NewKind::Created => (Vec::new(), provenance.clone()),
        };
        blocks.push(Block {
            version_id: d.version_id.clone(),
            lineage_id: lineage[n].clone(),
            block_type: d.block_type,
            text: d.text.clone(),
            normalized_text: d.normalized_text.clone(),
            heading_path: d.heading_path.clone(),
            heading_level: d.heading_level,
            parents,
            provenance: prov,
        });
        let v = d.version_id.clone();
        match kind {
            NewKind::Exact { old, anchor } => {
                if moved.contains(&n) {
                    events.push(event(
                        doc_path,
                        EventKind::BlockMoved,
                        vec![lineage[n].clone()],
                        vec![v.clone()],
                        vec![v],
                        json!({"matcher_version": mv, "anchor_kind": anchor, "from_pos": old, "to_pos": n}),
                    ));
                }
            }
            NewKind::Updated { old, score } => events.push(event(
                doc_path,
                EventKind::BlockUpdated,
                vec![lineage[n].clone()],
                vec![prev[*old].version_id.clone()],
                vec![v],
                json!({"matcher_version": mv, "score": round6(*score), "provenance": prov_json}),
            )),
            NewKind::Split {
                old,
                group,
                score,
                residual,
            } => {
                if group[0] == n {
                    let mut meta = json!({"matcher_version": mv, "score": round6(*score),
                               "from_lineages": [prev[*old].lineage_id], "residual": residual,
                               "provenance": prov_json});
                    if v3 {
                        meta["dominant"] = json!(lineage[longest(group, &new_len)]);
                    }
                    events.push(event(
                        doc_path,
                        EventKind::BlockSplit,
                        group.iter().map(|&k| lineage[k].clone()).collect(),
                        vec![prev[*old].version_id.clone()],
                        group
                            .iter()
                            .map(|&k| drafts[k].version_id.clone())
                            .collect(),
                        meta,
                    ));
                }
            }
            NewKind::Merged {
                olds,
                score,
                residual,
            } => {
                let mut meta = json!({"matcher_version": mv, "score": round6(*score),
                       "from_lineages": olds.iter().map(|&o| prev[o].lineage_id.clone()).collect::<Vec<_>>(),
                       "provenance": prov_json});
                if v3 {
                    meta["dominant"] = json!(lineage[n]);
                    meta["residual"] = json!(residual);
                }
                events.push(event(
                    doc_path,
                    EventKind::BlockMerged,
                    vec![lineage[n].clone()],
                    olds.iter().map(|&o| prev[o].version_id.clone()).collect(),
                    vec![v],
                    meta,
                ))
            }
            NewKind::Created => events.push(event(
                doc_path,
                EventKind::BlockCreated,
                vec![lineage[n].clone()],
                vec![],
                vec![v],
                json!({"matcher_version": mv, "provenance": prov_json}),
            )),
        }
    }
    for o in (0..n_old).filter(|&o| !old_taken[o]) {
        events.push(event(
            doc_path,
            EventKind::BlockDeleted,
            vec![prev[o].lineage_id.clone()],
            vec![prev[o].version_id.clone()],
            vec![],
            json!({"matcher_version": mv}),
        ));
    }
    MatchResult { blocks, events }
}
