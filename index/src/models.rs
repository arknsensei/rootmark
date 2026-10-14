//! Embedder model files pinned by `models/MANIFEST.json` (GOAL P1-C
//! "Embedder"): every file is checked against its manifest size and blake3
//! before use; any mismatch is an error. `embedder_id` is derived from the
//! manifest-verified `model.onnx` hash.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MODEL_FILE: &str = "model.onnx";
pub const TOKENIZER_FILE: &str = "tokenizer.json";
pub const ST_CONFIG_FILE: &str = "sentence_bert_config.json";
pub const POOLING_FILE: &str = "1_Pooling/config.json";

#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    pub remote_path: String,
    pub size: u64,
    pub sha256: String,
    pub blake3: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub source: String,
    pub revision: String,
    pub files: BTreeMap<String, FileEntry>,
}

/// A model directory whose every file matched the manifest.
#[derive(Debug, Clone)]
pub struct VerifiedModel {
    pub dir: PathBuf,
    pub manifest: Manifest,
}

impl VerifiedModel {
    pub fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    /// `"<name>/" + blake3(model.onnx)[:8]` (TASKS C1).
    pub fn embedder_id(&self) -> String {
        format!(
            "{}/{}",
            self.manifest.name,
            &self.manifest.files[MODEL_FILE].blake3[..8]
        )
    }
}

/// The models root: `$MDK_MODELS`, else this source tree's `models/`.
pub fn models_root() -> PathBuf {
    match std::env::var_os("MDK_MODELS") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("models"),
    }
}

pub fn read_manifest(root: &Path) -> Result<Manifest> {
    let p = root.join("MANIFEST.json");
    let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    let m: Manifest =
        serde_json::from_str(&text).with_context(|| format!("parse {}", p.display()))?;
    for f in [MODEL_FILE, TOKENIZER_FILE, ST_CONFIG_FILE, POOLING_FILE] {
        if !m.files.contains_key(f) {
            bail!("{}: no entry for {f}", p.display());
        }
    }
    for (name, e) in &m.files {
        let hex = |s: &str, n: usize| {
            s.len() == n && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        };
        if !hex(&e.blake3, 64) || !hex(&e.sha256, 64) {
            bail!(
                "{}: {name}: blake3/sha256 must be 64 lowercase hex",
                p.display()
            );
        }
    }
    Ok(m)
}

/// Checks every manifest file (size, blake3) under `<root>/<name>/`.
pub fn verify(root: &Path) -> Result<VerifiedModel> {
    let manifest = read_manifest(root)?;
    let dir = root.join(&manifest.name);
    for (name, e) in &manifest.files {
        let p = dir.join(name);
        let bytes = std::fs::read(&p)
            .with_context(|| format!("read {} (fetch with scripts/fetch_model.sh)", p.display()))?;
        if bytes.len() as u64 != e.size {
            bail!(
                "{}: size {} != manifest {}",
                p.display(),
                bytes.len(),
                e.size
            );
        }
        let got = blake3::hash(&bytes).to_hex().to_string();
        if got != e.blake3 {
            bail!("{}: blake3 {got} != manifest {}", p.display(), e.blake3);
        }
    }
    Ok(VerifiedModel { dir, manifest })
}
