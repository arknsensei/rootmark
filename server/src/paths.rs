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

/// Every component of `doc` that exists under `vault` has a directory
/// entry with exactly that name (byte for byte), and no component is
/// another spelling of an existing entry — the same name up to case or
/// Unicode form (`rootmarkcore::normalize::fold_name`). On a case- or
/// normalization-insensitive file system (macOS) `ESPRESSO.md` or the NFD
/// form of `café.md` open the file the vault knows as `espresso.md` or the
/// NFC name; on Linux `SUB/b.md` would create a second directory beside
/// `sub/`. Such a spelling is refused on every platform (K-GH.6: a vault
/// stays portable between macOS and Linux), so no write reaches a document
/// or a directory under a name other than its own.
pub fn exact_on_disk(vault: &Path, doc: &str) -> Result<()> {
    let mut dir =
        std::fs::canonicalize(vault).with_context(|| format!("vault {}", vault.display()))?;
    for c in doc.split('/') {
        if !dir.is_dir() {
            return Ok(()); // from here on nothing exists
        }
        let fold = rootmarkcore::normalize::fold_name(c);
        let mut exact = false;
        let mut other: Option<String> = None;
        for e in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let name = e?.file_name();
            if name.as_encoded_bytes() == c.as_bytes() {
                exact = true;
            } else if let Some(n) = name.to_str() {
                if rootmarkcore::normalize::fold_name(n) == fold {
                    other = Some(n.to_string());
                }
            }
        }
        if let (false, Some(n)) = (exact, &other) {
            bail!(
                "doc {doc:?}: {c:?} is another spelling (case or Unicode form) of the existing entry {n:?} of {}; use the name as it is on disk",
                dir.display()
            );
        }
        // fail closed (automated security review): whatever the file system
        // folds that this fold does not, a name the OS resolves to an entry
        // of another spelling is refused too
        if !exact && std::fs::symlink_metadata(dir.join(c)).is_ok() {
            bail!(
                "doc {doc:?}: {c:?} matches an existing entry of {} only by case or Unicode form; use the name as it is on disk",
                dir.display()
            );
        }
        if !exact {
            return Ok(()); // a new name: nothing below it exists
        }
        dir = dir.join(c);
    }
    Ok(())
}

/// Replaces `path` with `bytes` through a fresh hidden temporary file in
/// the same directory (`create_new`: an existing entry there, a symbolic
/// link included, is removed first and never followed) and a rename, so
/// the write never leaves the directory and never half-writes the file.
pub fn replace_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let dir = path.parent().context("file without a directory")?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("file name is not UTF-8")?;
    let tmp = dir.join(format!(".{name}.rootmark-propose.tmp"));
    if std::fs::symlink_metadata(&tmp).is_ok() {
        std::fs::remove_file(&tmp).with_context(|| format!("remove {}", tmp.display()))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .with_context(|| format!("create {}", tmp.display()))?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}
