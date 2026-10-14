//! Vector store (GOAL P1-C "Vektör"): `vectors/vectors.f32` holds one
//! L2-normalized f32 vector per record (storage order, little-endian, `dim`
//! values each); `vectors/embedders.jsonl` gives every record's
//! `embedder_id`. Search is exact cosine over the eligible records (no ANN).
//! Vectors of different `embedder_id`s never meet: mixing them, or querying
//! them with another embedder, is an error.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorMeta {
    pub record_id: String,
    pub embedder_id: String,
}

pub fn encode(vecs: &[Vec<f32>], dim: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(vecs.len() * dim * 4);
    for v in vecs {
        if v.len() != dim {
            bail!("vector of length {} != {dim}", v.len());
        }
        for x in v {
            if !x.is_finite() {
                bail!("non-finite vector component");
            }
            out.extend_from_slice(&x.to_le_bytes());
        }
    }
    Ok(out)
}

pub fn decode(bytes: &[u8], dim: usize) -> Result<Vec<Vec<f32>>> {
    if dim == 0 || !bytes.len().is_multiple_of(dim * 4) {
        bail!(
            "vector file of {} bytes is not a multiple of {dim} f32",
            bytes.len()
        );
    }
    Ok(bytes
        .chunks_exact(dim * 4)
        .map(|c| {
            c.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect()
        })
        .collect())
}

/// The single `embedder_id` of `metas`; none or several is an error.
pub fn single_embedder(metas: &[VectorMeta]) -> Result<Option<String>> {
    let mut ids: Vec<&str> = metas.iter().map(|m| m.embedder_id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    match ids.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some(one.to_string())),
        many => bail!("vector store mixes embedder ids {many:?}"),
    }
}

/// Cosine of unit vectors, accumulated in f64 in component order.
pub fn cosine(q: &[f32], v: &[f32]) -> f64 {
    q.iter()
        .zip(v)
        .map(|(a, b)| f64::from(*a) * f64::from(*b))
        .sum()
}

pub fn read(dir: &Path, dim: usize) -> Result<(Vec<Vec<f32>>, Vec<VectorMeta>)> {
    let bytes = fs::read(dir.join("vectors.f32")).context("vectors.f32")?;
    let vecs = decode(&bytes, dim)?;
    let text = fs::read_to_string(dir.join("embedders.jsonl")).context("embedders.jsonl")?;
    let metas = text
        .lines()
        .map(|l| Ok(serde_json::from_str(l)?))
        .collect::<Result<Vec<VectorMeta>>>()?;
    if metas.len() != vecs.len() {
        bail!("{} vectors, {} embedder rows", vecs.len(), metas.len());
    }
    Ok((vecs, metas))
}
