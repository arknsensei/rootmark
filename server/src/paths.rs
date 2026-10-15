//! Vault-relative document paths (shared by the CLI, the watcher and
//! `propose`).

use anyhow::{bail, Context, Result};
use std::path::{Component, Path, PathBuf};

/// `<vault>/<doc>` for a vault-relative, `/`-separated doc path. The path
/// must stay inside the vault: no `..`/absolute components, and no existing
/// component (directory or the file itself) may be a symbolic link, so a
/// read or write can never leave the vault through a link.
pub fn doc_file(vault: &Path, doc: &str) -> Result<PathBuf> {
    let rel = Path::new(doc);
    if doc.is_empty()
        || !doc.ends_with(".md")
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        bail!("doc {doc:?}: expected a vault-relative path to a .md file");
    }
    let root =
        std::fs::canonicalize(vault).with_context(|| format!("vault {}", vault.display()))?;
    let mut p = root.clone();
    for c in rel.components() {
        p.push(c);
        if let Ok(m) = std::fs::symlink_metadata(&p) {
            if m.file_type().is_symlink() {
                bail!("doc {doc:?}: {} is a symbolic link", p.display());
            }
        }
    }
    if !p.starts_with(&root) {
        bail!("doc {doc:?} resolves outside the vault");
    }
    Ok(p)
}
