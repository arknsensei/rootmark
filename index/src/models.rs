//! Embedder model files pinned by `models/MANIFEST.json` (GOAL P1-C
//! "Embedder"): every file is checked against its manifest size and blake3
//! before use; any mismatch is an error. `embedder_id` is derived from the
//! manifest-verified `model.onnx` hash and the numerics tag (P2-V.1).

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

/// A model directory whose every file matched the manifest; only `verify`
/// constructs one.
#[derive(Debug, Clone)]
pub struct VerifiedModel {
    dir: PathBuf,
    manifest: Manifest,
}

impl VerifiedModel {
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    /// `"<name>/" + blake3(model.onnx)[:8] + "+" + NUMERICS` (TASKS C1;
    /// P2-V.1/E1 adds the numerics tag: the same model computed by
    /// `crate::detbert` gives other vector bytes than the ONNX Runtime
    /// vectors stored under the id without it, so the two never mix).
    pub fn embedder_id(&self) -> String {
        format!(
            "{}/{}+{}",
            self.manifest.name,
            &self.manifest.files[MODEL_FILE].blake3[..8],
            crate::embed::NUMERICS
        )
    }
}

/// The models root: `$ROOTMARK_MODELS`, else this source tree's `models/`.
pub fn models_root() -> PathBuf {
    match std::env::var_os("ROOTMARK_MODELS") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("models"),
    }
}

pub fn read_manifest(root: &Path) -> Result<Manifest> {
    read_manifest_file(&root.join("MANIFEST.json"))
}

/// The manifest file of the model `name` (P2-L/L2, `[index] embedder`):
/// `MANIFEST.json` for the default model, `MANIFEST.<name>.json` for another.
pub fn manifest_file(name: &str) -> String {
    if name == crate::config::EMBEDDER_DEFAULT {
        "MANIFEST.json".to_string()
    } else {
        format!("MANIFEST.{name}.json")
    }
}

fn read_manifest_file(p: &Path) -> Result<Manifest> {
    let p = p.to_path_buf();
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
    verify_manifest(root, read_manifest(root)?)
}

/// `verify` for the model `name` (one of `crate::config::EMBEDDERS`): its
/// own manifest (`manifest_file`), whose `name` must be `name`.
pub fn verify_named(root: &Path, name: &str) -> Result<VerifiedModel> {
    if !crate::config::EMBEDDERS.contains(&name) {
        bail!(
            "unknown embedder {name:?}: one of {:?}",
            crate::config::EMBEDDERS
        );
    }
    let p = root.join(manifest_file(name));
    let m = read_manifest_file(&p)?;
    if m.name != name {
        bail!("{}: name {:?} != {name:?}", p.display(), m.name);
    }
    verify_manifest(root, m)
}

fn verify_manifest(root: &Path, manifest: Manifest) -> Result<VerifiedModel> {
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
