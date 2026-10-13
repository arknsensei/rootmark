//! Deterministic vault directory walking.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// `path` relative to `base`, `/`-separated.
pub fn rel_slash(path: &Path, base: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(base)
        .with_context(|| format!("{} is not under {}", path.display(), base.display()))?;
    let parts: Vec<String> = rel
        .components()
        .map(|c| {
            c.as_os_str()
                .to_str()
                .map(str::to_string)
                .context("non-UTF-8 path")
        })
        .collect::<Result<_>>()?;
    Ok(parts.join("/"))
}

/// Every `*.md` under `dir` (hidden entries skipped) as `(doc_path, path)`,
/// sorted by the bytes of `doc_path` — the ingest order (EVAL §D3).
pub fn markdown_files(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).with_context(|| format!("read_dir {}", d.display()))? {
            let p = e?.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else if name.ends_with(".md") {
                out.push((rel_slash(&p, dir)?, p));
            }
        }
    }
    out.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    Ok(out)
}
