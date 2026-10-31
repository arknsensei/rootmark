//! Embedders (GOAL P1-C "Embedder"). One embedder per index: every vector
//! carries the `embedder_id` it was made with, and mixing ids is an error.
//!
//! `OnnxMiniLm`: `all-MiniLM-L6-v2` as ONNX (`models/MANIFEST.json`); the
//! graph's forward pass is computed by `crate::detbert` in fixed-order Rust
//! (P2-V.1/E1; K-P2V.3 T1 (a), `reports/rounds/P2-V.1/e0/feasibility.md`):
//! every vector component is one defined sequence of IEEE 754 operations,
//! the same bytes on every processor (x86_64, aarch64, Apple silicon).
//! Until P2-V.1 the graph ran on ONNX Runtime, whose kernels depend on the
//! processor. Texts are tokenized with the model's own `tokenizer.json`,
//! truncated to `max_seq_length` of `sentence_bert_config.json` (256, as
//! `sentence-transformers` does) and never padded; each text runs alone, so
//! a batch gives bit-identical vectors to one-by-one calls. Pooling is the
//! attention-masked mean of `last_hidden_state`, then L2 normalization
//! (`1_Pooling/config.json`: mean tokens only).

use crate::detbert::DetBert;
use crate::models::{self, VerifiedModel};
use anyhow::{anyhow, bail, Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use tokenizers::{Tokenizer, TruncationParams, TruncationStrategy};

/// Text → unit vectors. Implementations MUST be deterministic (spec §10).
pub trait Embedder {
    /// Recorded with every vector; queries and stored vectors must agree.
    fn id(&self) -> &str;
    fn dim(&self) -> usize;
    /// One vector per text, input order; an error, never a partial result.
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    /// Texts embedded since construction (TASKS C5: only affected chunks).
    fn embedded_count(&self) -> u64;
}

pub const DIM: usize = 384;

/// The numerics of the vectors, the last part of every manifest model's
/// `embedder_id` (`crate::models`): `fixed1` = `crate::detbert` (P2-V.1).
pub const NUMERICS: &str = "fixed1";

/// The `embedder_id`s of the two manifest models as an older rootmark
/// computed them, with ONNX Runtime (processor-dependent vector bytes;
/// P2-V.1, K-P2V.3 T1 (a)). An index holding one is stale: `rootmark
/// migrate` (`index_rebuild`) rebuilds it.
pub const ONNX_RUNTIME_IDS: [&str; 2] = [
    "minilm-l6-v2/e73168f9",
    "multilingual-minilm-l12-v2/ca55a866",
];

/// Whether `embedder_id` is one of [`ONNX_RUNTIME_IDS`].
pub fn onnx_runtime_numerics(embedder_id: &str) -> bool {
    ONNX_RUNTIME_IDS.contains(&embedder_id)
}

pub struct OnnxMiniLm {
    id: String,
    tokenizer: Tokenizer,
    bert: DetBert,
    count: AtomicU64,
}

fn read_json(model: &VerifiedModel, file: &str) -> Result<serde_json::Value> {
    let p = model.path(file);
    let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", p.display()))
}

impl OnnxMiniLm {
    /// Loads the manifest-verified model under `models::models_root()`.
    pub fn load_default() -> Result<OnnxMiniLm> {
        OnnxMiniLm::load(&models::verify(&models::models_root())?)
    }

    /// Loads the manifest-verified model `name` (`[index] embedder`; P2-L/L2):
    /// `minilm-l6-v2` (the default, `models/MANIFEST.json`) or
    /// `multilingual-minilm-l12-v2` (`models/MANIFEST.multilingual-minilm-l12-v2.json`;
    /// `make models-multilingual`). Both are 384-d mean-pooled BERT models
    /// with the same inputs, run the same way.
    pub fn load_named(name: &str) -> Result<OnnxMiniLm> {
        OnnxMiniLm::load(&models::verify_named(&models::models_root(), name)?)
    }

    /// The embedder the vault at `root` names (`crate::config`).
    pub fn for_vault(root: &std::path::Path) -> Result<OnnxMiniLm> {
        OnnxMiniLm::load_named(&crate::config::IndexConfig::of_vault(root)?.embedder)
    }

    pub fn load(model: &VerifiedModel) -> Result<OnnxMiniLm> {
        let st = read_json(model, models::ST_CONFIG_FILE)?;
        let max_len = st["max_seq_length"]
            .as_u64()
            .context("sentence_bert_config.json: max_seq_length")? as usize;
        let pool = read_json(model, models::POOLING_FILE)?;
        let flag = |k: &str| pool[k].as_bool().unwrap_or(false);
        if pool["word_embedding_dimension"].as_u64() != Some(DIM as u64)
            || !flag("pooling_mode_mean_tokens")
            || flag("pooling_mode_cls_token")
            || flag("pooling_mode_max_tokens")
            || flag("pooling_mode_mean_sqrt_len_tokens")
        {
            bail!("1_Pooling/config.json: only {DIM}-d mean-token pooling is supported");
        }
        let mut tokenizer = Tokenizer::from_file(model.path(models::TOKENIZER_FILE))
            .map_err(|e| anyhow!("tokenizer.json: {e}"))?;
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: max_len,
                strategy: TruncationStrategy::LongestFirst,
                stride: 0,
                ..Default::default()
            }))
            .map_err(|e| anyhow!("tokenizer truncation: {e}"))?;
        let bert = DetBert::load(&model.path(models::MODEL_FILE))?;
        Ok(OnnxMiniLm {
            id: model.embedder_id(),
            tokenizer,
            bert,
            count: AtomicU64::new(0),
        })
    }

    fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("tokenize: {e}"))?;
        let n = enc.get_ids().len();
        let data = self.bert.forward(enc.get_ids(), enc.get_type_ids())?;
        if data.len() != n * DIM {
            bail!(
                "last_hidden_state of {} values, expected {n} × {DIM}",
                data.len()
            );
        }
        // attention-masked mean (no padding: every mask value is 1)
        let mut sum = vec![0f32; DIM];
        let mut weight = 0f32;
        for (t, &m) in enc.get_attention_mask().iter().enumerate() {
            if m == 0 {
                continue;
            }
            weight += 1.0;
            for (s, x) in sum.iter_mut().zip(&data[t * DIM..(t + 1) * DIM]) {
                *s += x;
            }
        }
        let weight = weight.max(1e-9);
        let mean: Vec<f32> = sum.iter().map(|s| s / weight).collect();
        let norm = mean.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        Ok(mean.iter().map(|x| x / norm).collect())
    }
}

impl Embedder for OnnxMiniLm {
    fn id(&self) -> &str {
        &self.id
    }

    fn dim(&self) -> usize {
        DIM
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed_one(t)?);
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        Ok(out)
    }

    fn embedded_count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }
}
