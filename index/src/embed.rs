//! Embedders (GOAL P1-C "Embedder"). One embedder per index: every vector
//! carries the `embedder_id` it was made with, and mixing ids is an error.
//!
//! `OnnxMiniLm`: `all-MiniLM-L6-v2` as ONNX (`models/MANIFEST.json`), CPU,
//! one intra-op and one inter-op thread, sequential execution, deterministic
//! compute. Texts are tokenized with the model's own `tokenizer.json`,
//! truncated to `max_seq_length` of `sentence_bert_config.json` (256, as
//! `sentence-transformers` does) and never padded; each text runs alone, so
//! a batch gives bit-identical vectors to one-by-one calls. Pooling is the
//! attention-masked mean of `last_hidden_state`, then L2 normalization
//! (`1_Pooling/config.json`: mean tokens only).

use crate::models::{self, VerifiedModel};
use anyhow::{anyhow, bail, Context, Result};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
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

pub struct OnnxMiniLm {
    id: String,
    tokenizer: Tokenizer,
    session: Mutex<Session>,
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
        let ort_err = |e: ort::Error| anyhow!("onnx runtime: {e}");
        let session = Session::builder()
            .map_err(ort_err)?
            .with_intra_threads(1)
            .map_err(|e| anyhow!("{e}"))?
            .with_inter_threads(1)
            .map_err(|e| anyhow!("{e}"))?
            .with_parallel_execution(false)
            .map_err(|e| anyhow!("{e}"))?
            .with_deterministic_compute(true)
            .map_err(|e| anyhow!("{e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow!("{e}"))?
            .commit_from_file(model.path(models::MODEL_FILE))
            .map_err(ort_err)?;
        Ok(OnnxMiniLm {
            id: model.embedder_id(),
            tokenizer,
            session: Mutex::new(session),
            count: AtomicU64::new(0),
        })
    }

    fn embed_one(&self, session: &mut Session, text: &str) -> Result<Vec<f32>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("tokenize: {e}"))?;
        let n = enc.get_ids().len();
        let col = |v: &[u32]| -> Result<ndarray::Array2<i64>> {
            Ok(ndarray::Array2::from_shape_vec(
                (1, n),
                v.iter().map(|&x| i64::from(x)).collect(),
            )?)
        };
        let ids = col(enc.get_ids())?;
        let mask = col(enc.get_attention_mask())?;
        let types = col(enc.get_type_ids())?;
        let out = session
            .run(ort::inputs![
                "input_ids" => TensorRef::from_array_view(&ids)?,
                "attention_mask" => TensorRef::from_array_view(&mask)?,
                "token_type_ids" => TensorRef::from_array_view(&types)?,
            ])
            .map_err(|e| anyhow!("onnx run: {e}"))?;
        let (shape, data) = out["last_hidden_state"]
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("last_hidden_state: {e}"))?;
        if shape.len() != 3 || shape[0] != 1 || shape[1] as usize != n || shape[2] as usize != DIM {
            bail!("last_hidden_state shape {shape:?}, expected [1, {n}, {DIM}]");
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
        let mut session = self
            .session
            .lock()
            .map_err(|_| anyhow!("embedder session poisoned"))?;
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed_one(&mut session, t)?);
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        Ok(out)
    }

    fn embedded_count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }
}
