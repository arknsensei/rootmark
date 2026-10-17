//! The HTTP API token, `<vault>/.rootmark/server.token` (GOAL P1-D "HTTP
//! API"): 32 random bytes as 64 lowercase hex characters and a newline,
//! created on the first start with mode 0600. A token file readable by group
//! or others, or not 64 hex characters, is an error (not repaired). A token
//! older than `[server] token_rotate_days` (by modification time) is replaced
//! at start. The token is a secret, never an identifier: it enters no event,
//! trace or log.

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub fn token_path(vault: &Path) -> PathBuf {
    rootmarkcore::meta::path(vault).join("server.token")
}

/// What `ensure` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenState {
    Created,
    Rotated,
    Kept,
}

fn fresh() -> Result<String> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).map_err(|e| anyhow::anyhow!("random token: {e}"))?;
    Ok(b.iter().map(|x| format!("{x:02x}")).collect())
}

fn write_0600(p: &Path, token: &str) -> Result<()> {
    let tmp = p.with_extension("token.tmp");
    let _ = std::fs::remove_file(&tmp);
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o
        .open(&tmp)
        .with_context(|| format!("create {}", tmp.display()))?;
    f.write_all(format!("{token}\n").as_bytes())?;
    f.sync_all()?;
    std::fs::rename(&tmp, p).with_context(|| format!("write {}", p.display()))?;
    Ok(())
}

fn check_mode(p: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let m = std::fs::metadata(p)?.permissions().mode() & 0o777;
        if m != 0o600 {
            bail!(
                "{} has mode {m:o}; it must be 600 (chmod 600, or delete it to get a new token)",
                p.display()
            );
        }
    }
    Ok(())
}

/// Reads the token, creating or rotating it first if needed.
pub fn ensure(vault: &Path, rotate_days: u32) -> Result<(String, TokenState)> {
    let p = token_path(vault);
    if std::fs::symlink_metadata(&p).is_err() {
        let t = fresh()?;
        write_0600(&p, &t)?;
        return Ok((t, TokenState::Created));
    }
    if std::fs::symlink_metadata(&p)?.file_type().is_symlink() {
        bail!("{} is a symbolic link", p.display());
    }
    check_mode(&p)?;
    let age = SystemTime::now()
        .duration_since(std::fs::metadata(&p)?.modified()?)
        .unwrap_or(Duration::ZERO);
    if age > Duration::from_secs(u64::from(rotate_days) * 86_400) {
        let t = fresh()?;
        write_0600(&p, &t)?;
        return Ok((t, TokenState::Rotated));
    }
    let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    let t = text.strip_suffix('\n').unwrap_or(&text);
    if t.len() != 64 || !t.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        bail!(
            "{} is not a token (64 lowercase hex characters); delete it to get a new one",
            p.display()
        );
    }
    Ok((t.to_string(), TokenState::Kept))
}
