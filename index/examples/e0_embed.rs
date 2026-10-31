//! P2-V.1/E0 (TASKS E0): embeds a fixed corpus with the current ONNX Runtime
//! embedder (`ort.f32`) and with the fixed-order forward pass of
//! `rootmarkindex::detbert` (`det.f32`), raw little-endian binary32 vectors
//! in record order, for the feasibility measurement across processors.
//! Measurement tool only: the product's embedder does not use `detbert`
//! before E1.
//!
//! `cargo run --release -p rootmark-index --example e0_embed -- <corpus dir> <out dir> [model]`

use anyhow::{Context, Result};
use rootmarkcore::{MatcherVersion, Vault};
use rootmarkindex::detbert::DetBert;
use rootmarkindex::embed::{Embedder, OnnxMiniLm, DIM};
use rootmarkindex::{models, store};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use tokenizers::{Tokenizer, TruncationParams, TruncationStrategy};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let corpus = PathBuf::from(args.get(1).context("corpus dir")?);
    let out = PathBuf::from(args.get(2).context("out dir")?);
    let model = args.get(3).map(String::as_str).unwrap_or("minilm-l6-v2");
    std::fs::create_dir_all(&out)?;
    let vroot = out.join("vault");
    // a corpus directory of markdown notes, or a JSON file with `chunks[].text`
    // (`research/data/embed_ref.json`, GOAL P1-C C-10)
    let (ids, owned): (Vec<String>, Vec<String>) = if corpus.is_file() {
        let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&corpus)?)?;
        j["chunks"]
            .as_array()
            .context("chunks")?
            .iter()
            .enumerate()
            .map(|(i, c)| {
                Ok((
                    format!("chunk{i}"),
                    c["text"].as_str().context("text")?.to_string(),
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .unzip()
    } else {
        let _ = std::fs::remove_dir_all(&vroot);
        std::fs::create_dir_all(&vroot)?;
        let mut v = Vault::open(&vroot)?;
        v.ingest_dir(&corpus, MatcherVersion::V3)?;
        let (records, _) = store::collect(&v)?;
        records.into_iter().map(|r| (r.record_id, r.text)).unzip()
    };
    let texts: Vec<&str> = owned.iter().map(String::as_str).collect();
    eprintln!("{} records", texts.len());

    let verified = models::verify_named(&models::models_root(), model)?;
    let t0 = Instant::now();
    let onnx = OnnxMiniLm::load(&verified)?;
    let ort = onnx.embed(&texts)?;
    let ort_s = t0.elapsed().as_secs_f64();

    let st: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        verified.path(models::ST_CONFIG_FILE),
    )?)?;
    let max_len = st["max_seq_length"].as_u64().context("max_seq_length")? as usize;
    let mut tok = Tokenizer::from_file(verified.path(models::TOKENIZER_FILE))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tok.with_padding(None);
    tok.with_truncation(Some(TruncationParams {
        max_length: max_len,
        strategy: TruncationStrategy::LongestFirst,
        stride: 0,
        ..Default::default()
    }))
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    let t1 = Instant::now();
    let bert = DetBert::load(&verified.path(models::MODEL_FILE))?;
    let mut det = Vec::with_capacity(texts.len());
    let mut tokens = Vec::with_capacity(texts.len());
    for t in &texts {
        let enc = tok.encode(*t, true).map_err(|e| anyhow::anyhow!("{e}"))?;
        let hs = bert.forward(enc.get_ids(), enc.get_type_ids())?;
        let n = enc.get_ids().len();
        tokens.push(n);
        // the pooling of `crate::embed` (mask all ones), unchanged
        let mut sum = vec![0f32; DIM];
        let mut weight = 0f32;
        for tk in 0..n {
            weight += 1.0;
            for (s, x) in sum.iter_mut().zip(&hs[tk * DIM..(tk + 1) * DIM]) {
                *s += x;
            }
        }
        let weight = weight.max(1e-9);
        let mean: Vec<f32> = sum.iter().map(|s| s / weight).collect();
        let norm = mean.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        det.push(mean.iter().map(|x| x / norm).collect::<Vec<f32>>());
    }
    let det_s = t1.elapsed().as_secs_f64();

    for (name, vecs) in [("ort.f32", &ort), ("det.f32", &det)] {
        let mut f = std::fs::File::create(out.join(name))?;
        for v in vecs {
            for x in v {
                f.write_all(&x.to_le_bytes())?;
            }
        }
    }
    let meta = serde_json::json!({
        "model": model,
        "embedder_id": onnx.id(),
        "layers": bert.layers(),
        "records": ids.len(),
        "record_ids": ids,
        "tokens": tokens,
        "ort_s": ort_s,
        "det_s": det_s,
    });
    std::fs::write(out.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    let _ = std::fs::remove_dir_all(&vroot);
    eprintln!("ort {ort_s:.1} s, det {det_s:.1} s");
    Ok(())
}
