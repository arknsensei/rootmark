//! The vault's metadata directory, `<vault>/.rootmark/` (K-GH.1). A vault
//! written before the rename keeps it under `.mdkernel/`: it is opened only
//! after `rootmark migrate --rename-store` renamed the directory, never by
//! creating a fresh empty store beside it (GH-0 ASSUMPTIONS G0-6). The
//! rename moves the directory as it is; `events.jsonl` and every object keep
//! their bytes, so `events_hash` is unchanged.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// The metadata directory under the vault root.
pub const META_DIR: &str = ".rootmark";
/// Its name before K-GH.1.
pub const LEGACY_META_DIR: &str = ".mdkernel";

/// `<vault>/.rootmark`.
pub fn path(vault_root: &Path) -> PathBuf {
    vault_root.join(META_DIR)
}

fn present(p: &Path) -> bool {
    std::fs::symlink_metadata(p).is_ok()
}

/// Fails when the vault has the pre-rename directory and no `.rootmark/`.
pub fn check(vault_root: &Path) -> Result<()> {
    let legacy = vault_root.join(LEGACY_META_DIR);
    if present(&legacy) && !present(&path(vault_root)) {
        bail!(
            "{} holds a store written before the rename to rootmark ({LEGACY_META_DIR}/); \
             run `rootmark migrate --rename-store {}` to rename it to {META_DIR}/",
            vault_root.display(),
            vault_root.display()
        );
    }
    Ok(())
}

/// Renames `<vault>/.mdkernel` to `<vault>/.rootmark`; returns both paths.
/// An error if there is no `.mdkernel/` or if `.rootmark/` already exists.
pub fn rename_store(vault_root: &Path) -> Result<(PathBuf, PathBuf)> {
    let from = vault_root.join(LEGACY_META_DIR);
    let to = path(vault_root);
    if present(&to) {
        bail!(
            "{} already exists; nothing renamed{}",
            to.display(),
            if present(&from) {
                format!(
                    " ({} is still there: move one of them away by hand)",
                    from.display()
                )
            } else {
                String::new()
            }
        );
    }
    if !std::fs::symlink_metadata(&from)
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        bail!("{} is not a directory: nothing to migrate", from.display());
    }
    std::fs::rename(&from, &to)
        .with_context(|| format!("rename {} to {}", from.display(), to.display()))?;
    Ok((from, to))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system temp dir, removed on drop.
    struct Dir(PathBuf);
    impl Dir {
        fn new(name: &str) -> Dir {
            let p =
                std::env::temp_dir().join(format!("rootmark-meta-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Dir(p)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn legacy_only_is_an_error_and_rename_moves_it() {
        let d = Dir::new("rename");
        let v = d.0.as_path();
        check(v).unwrap();
        std::fs::create_dir_all(v.join(LEGACY_META_DIR)).unwrap();
        std::fs::write(v.join(LEGACY_META_DIR).join("events.jsonl"), b"x\n").unwrap();
        let e = check(v).unwrap_err().to_string();
        assert!(e.contains("rootmark migrate --rename-store"), "{e}");
        rename_store(v).unwrap();
        check(v).unwrap();
        assert_eq!(
            std::fs::read(v.join(META_DIR).join("events.jsonl")).unwrap(),
            b"x\n"
        );
        assert!(!v.join(LEGACY_META_DIR).exists());
        // a second rename has nothing to do
        assert!(rename_store(v).is_err());
    }

    #[test]
    fn both_present_is_refused() {
        let d = Dir::new("both");
        let v = d.0.as_path();
        std::fs::create_dir_all(v.join(LEGACY_META_DIR)).unwrap();
        std::fs::create_dir_all(v.join(META_DIR)).unwrap();
        check(v).unwrap();
        let e = rename_store(v).unwrap_err().to_string();
        assert!(e.contains("move one of them away"), "{e}");
    }
}
