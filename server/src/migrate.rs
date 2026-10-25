//! Migrations of a vault's stored artifacts (P2-A/A1, A2; GOAL P2-A 1–2).
//!
//! A registry of ordered migrations, each with an id, `applies`, `plan` and
//! `apply`; `rootmark migrate [--dry-run] [--rollback <id>] <vault>`:
//! - `--dry-run` prints the version matrix and the plan (which migration,
//!   from which format to which, which files and how many bytes) and
//!   changes nothing;
//! - a migration writes its rollback record under
//!   `.rootmark/migrations/<id>/` (`<id>` = `<NNNN>-<name>`, `NNNN` the
//!   record's ordinal: no time, no randomness): `manifest.json` with the
//!   SHA-256 of every file it touched before and after, and under `files/`
//!   a copy of every file it replaced or removed;
//! - `--rollback <id>` (the newest record only) checks the record (each
//!   copy against its SHA-256) and the vault (each touched file still as
//!   the migration left it), restores the files, removes the record and
//!   checks that every restored file is byte-equal to the one before;
//! - a migration never changes `events_hash`: one that would is undone and
//!   stops with an error (no record is written).
//!
//! The registered migrations, in order: `rename_store` (`.mdkernel/` →
//! `.rootmark/`, K-GH.1: the directory is renamed, every byte kept),
//! `index_rebuild` (an index of an older format or lexical record → this
//! format, rebuilt; the old index is kept in the record),
//! `policy_attestation_1` (P2-P/P0, K-P2L-T1: a recipe without an
//! `[attestation]` table gets `[attestation] unattested = "canonical"`
//! appended, every other byte kept; recommended, not required), `log_readers`
//! (the event and attestation logs read by this reader, spec 2.0.2: no
//! byte changes, a validation) and `checkpoints` (their format is unchanged:
//! a record only). Only the first two change bytes; they are the ones a
//! vault *requires* (`status.migrations.required`), the last two are
//! recorded validations. `policy_attestation_1` changes bytes but a vault
//! does not require it (`status.migrations.recommended`): a recipe without
//! the table is read as the kernel always read it (`unverified`), and never
//! answers `migration_required`.

use anyhow::{bail, Context, Result};
use rootmarkcore::meta;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The record's format.
pub const RECORD_FORMAT: &str = "rootmark-migration/1";
/// `.rootmark/migrations/`, relative to the store.
pub const MIGRATIONS_DIR: &str = "migrations";
/// Index formats this implementation can migrate from (older than
/// `rootmarkindex::store::FORMAT`; `/1` before P1-C.1, `/2` before P1-D.1).
pub const OLD_INDEX_FORMATS: [&str; 2] = ["mdkernel-index/1", "mdkernel-index/2"];
/// The store formats (`status.migrations.artifacts.store`).
pub const STORE_FORMAT: &str = "rootmark-store/1";
pub const LEGACY_STORE_FORMAT: &str = "mdkernel-store/1";
/// The event log's line format (spec §4.2; every value since 1.0.0 is read
/// by this reader).
pub const EVENTS_FORMAT: &str = "rootmark-events/1";
/// The attestation log's reading rule (spec 2.0.2 §8).
pub const ATTESTATIONS_FORMAT: &str = "attestations/2.0.2";
/// Checkpoints (spec §8; unchanged since 1.0.0).
pub const CHECKPOINTS_FORMAT: &str = "rootmark-checkpoints/1";

fn sha256(b: &[u8]) -> String {
    hmac_sha256::Hash::hash(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

/// `events_hash` of the log under `store` (`Vault::events_hash`'s value,
/// computed on the file so that a store not yet opened can be checked).
fn events_hash_at(store: &Path) -> Result<String> {
    let p = store.join("events.jsonl");
    let bytes = if p.exists() {
        std::fs::read(&p).with_context(|| format!("read {}", p.display()))?
    } else {
        Vec::new()
    };
    let mut h = blake3::hash(&bytes).to_hex().to_string();
    h.truncate(32);
    Ok(h)
}

/// The vault's store directory as it is now: `.rootmark/`, else the
/// pre-rename `.mdkernel/`.
fn store_of(vault: &Path) -> PathBuf {
    let s = meta::path(vault);
    if std::fs::symlink_metadata(&s).is_ok() {
        s
    } else {
        vault.join(meta::LEGACY_META_DIR)
    }
}

/// Every file under `dir` (relative `/` paths, sorted), skipping
/// `skip` (relative to `dir`). Symbolic links are refused: a store holds
/// none.
fn files_under(dir: &Path, skip: Option<&str>) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if std::fs::symlink_metadata(dir).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("{}: a symbolic link in the store", dir.display());
    }
    if !dir.exists() {
        return Ok(out);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).with_context(|| format!("read {}", d.display()))? {
            let p = e?.path();
            let rel = p
                .strip_prefix(dir)
                .expect("under dir")
                .to_string_lossy()
                .replace('\\', "/");
            if skip.is_some_and(|s| rel == s || rel.starts_with(&format!("{s}/"))) {
                continue;
            }
            let m = std::fs::symlink_metadata(&p)?;
            if m.file_type().is_symlink() {
                bail!("{}: a symbolic link in the store", p.display());
            } else if m.is_dir() {
                stack.push(p);
            } else {
                out.push(rel);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Every component of `vault/<rel>` up to (not including) the last is a
/// real directory, not a symbolic link (or absent): a record's path cannot
/// be bent out of the store through a link (automated security review).
fn no_links(vault: &Path, rel: &str) -> Result<()> {
    let mut cur = vault.to_path_buf();
    let parts: Vec<_> = Path::new(rel).components().collect();
    for c in &parts[..parts.len().saturating_sub(1)] {
        cur.push(c);
        match std::fs::symlink_metadata(&cur) {
            Ok(m) if m.file_type().is_symlink() => {
                bail!("{}: a symbolic link in the store", cur.display())
            }
            Ok(m) if !m.is_dir() => bail!("{}: not a directory", cur.display()),
            _ => {}
        }
    }
    // the file itself: never a link
    if std::fs::symlink_metadata(vault.join(rel)).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("{rel}: a symbolic link in the store");
    }
    Ok(())
}

/// SHA-256 of every file under `vault/<prefix>` keyed by its vault-relative
/// path (`skip` relative to the prefix).
fn hashes(vault: &Path, prefix: &str, skip: Option<&str>) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    // a scope that is one file (`policy_attestation_1`): that file, if any
    let p = vault.join(prefix);
    match std::fs::symlink_metadata(&p) {
        Ok(m) if m.file_type().is_symlink() => {
            bail!("{}: a symbolic link in the store", p.display())
        }
        Ok(m) if m.is_file() => {
            let b = std::fs::read(&p).with_context(|| format!("read {}", p.display()))?;
            out.insert(prefix.to_string(), sha256(&b));
            return Ok(out);
        }
        _ => {}
    }
    for rel in files_under(&vault.join(prefix), skip)? {
        let p = vault.join(prefix).join(&rel);
        let b = std::fs::read(&p).with_context(|| format!("read {}", p.display()))?;
        out.insert(format!("{prefix}/{rel}"), sha256(&b));
    }
    Ok(out)
}

fn bytes_of(vault: &Path, paths: &BTreeMap<String, String>) -> u64 {
    paths
        .keys()
        .filter_map(|p| std::fs::metadata(vault.join(p)).ok())
        .map(|m| m.len())
        .sum()
}

/// What a migration would do (`--dry-run`).
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub migration: &'static str,
    /// changes bytes (`false`: a recorded validation)
    pub required: bool,
    pub from: String,
    pub to: String,
    /// what is done, in words
    pub steps: Vec<String>,
    /// files replaced or removed (vault-relative) and their total bytes
    pub touches: Vec<String>,
    pub bytes: u64,
    /// the directory (vault-relative) the migration rewrites: every file in
    /// it after the migration is recorded (`None`: a validation)
    #[serde(skip)]
    pub scope: Option<String>,
}

/// A registered migration.
pub trait Migration {
    fn id(&self) -> &'static str;
    /// whether it changes bytes (a vault in its `from` format requires it,
    /// unless it is `recommended`)
    fn required(&self) -> bool;
    /// a migration that changes bytes but that no vault requires (P2-P/P0):
    /// listed in `status.migrations.recommended`, never `migration_required`
    fn recommended(&self) -> bool {
        false
    }
    /// whether the vault is in the format this migration starts from (and,
    /// for a validation, not yet validated)
    fn applies(&self, vault: &Path, done: &[Record]) -> Result<bool>;
    fn plan(&self, vault: &Path) -> Result<Plan>;
    /// Performs it. Before touching anything it saves what it will replace
    /// through `save`; returns the vault-relative files it changed, removed
    /// or created (`before` is computed by the framework from `plan`).
    fn apply(&self, vault: &Path, save: &mut Saver) -> Result<()>;
}

/// Copies the files a migration replaces into the record being written.
pub struct Saver {
    staging: PathBuf,
    saved: BTreeMap<String, String>,
}

impl Saver {
    /// Copies `vault/<rel>` into the record (`files/<rel>`).
    pub fn save(&mut self, vault: &Path, rel: &str) -> Result<()> {
        let b = std::fs::read(vault.join(rel)).with_context(|| format!("read {rel}"))?;
        let to = self.staging.join("files").join(rel);
        std::fs::create_dir_all(to.parent().expect("a file"))?;
        std::fs::write(&to, &b).with_context(|| format!("write {}", to.display()))?;
        self.saved.insert(rel.to_string(), sha256(&b));
        Ok(())
    }
}

/// One rollback record (`manifest.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub format: String,
    pub id: String,
    pub migration: String,
    pub from: String,
    pub to: String,
    pub events_hash: String,
    /// SHA-256 of every touched file before (vault-relative paths)
    pub before: BTreeMap<String, String>,
    /// SHA-256 of every touched file after
    pub after: BTreeMap<String, String>,
    /// SHA-256 of every copy under `files/` (the replaced or removed files)
    pub saved: BTreeMap<String, String>,
    /// `rename` (the store directory, `rename_store`), else `files`
    pub undo: String,
    /// the directory the migration rewrote (`after` is all of it; `None`
    /// for a validation)
    pub scope: Option<String>,
    /// what the validation checked (validations only)
    pub checked: Vec<String>,
}

// ---- the migrations -------------------------------------------------------------

pub struct RenameStore;
pub struct IndexRebuild;
pub struct PolicyAttestation;
pub struct LogReaders;
pub struct CheckpointsRecord;

impl Migration for RenameStore {
    fn id(&self) -> &'static str {
        "rename_store"
    }
    fn required(&self) -> bool {
        true
    }
    fn applies(&self, vault: &Path, _: &[Record]) -> Result<bool> {
        Ok(meta::check(vault).is_err())
    }
    fn plan(&self, vault: &Path) -> Result<Plan> {
        let before = hashes(vault, meta::LEGACY_META_DIR, None)?;
        Ok(Plan {
            migration: self.id(),
            required: true,
            from: format!("{LEGACY_STORE_FORMAT} ({}/)", meta::LEGACY_META_DIR),
            to: format!("{STORE_FORMAT} ({}/)", meta::META_DIR),
            steps: vec![format!(
                "rename {}/ to {}/; every file keeps its bytes (K-GH.1)",
                meta::LEGACY_META_DIR,
                meta::META_DIR
            )],
            bytes: bytes_of(vault, &before),
            touches: before.into_keys().collect(),
            scope: Some(meta::META_DIR.into()),
        })
    }
    fn apply(&self, vault: &Path, _: &mut Saver) -> Result<()> {
        meta::rename_store(vault)?;
        Ok(())
    }
}

/// The index manifest's `format` and whether its lexical record is the one
/// this implementation builds for the vault — its `[index] tokenizer`
/// (P2-L/L1: an index of the other variant is migrated by `index_rebuild`;
/// a recipe that cannot be read counts as not current, and the rebuild then
/// fails on it) (`None`: no manifest or not readable).
fn index_manifest(vault: &Path) -> Option<(String, bool)> {
    let p = store_of(vault).join("index/manifest.json");
    let raw: Value = serde_json::from_slice(&std::fs::read(p).ok()?).ok()?;
    let format = raw["format"].as_str()?.to_string();
    let want = rootmarkindex::config::IndexConfig::of_vault(vault)
        .ok()
        .map(|c| rootmarkindex::store::LexicalParams::of(c.tokenizer));
    let lexical =
        serde_json::from_value::<rootmarkindex::store::LexicalParams>(raw["lexical"].clone())
            .is_ok_and(|l| Some(l) == want);
    Some((format, lexical))
}

/// Whether an index manifest of `format` (lexical record current or not)
/// is one this implementation migrates (an older `rootmark`'s index).
pub fn index_needs_migration(format: &str, lexical_current: bool) -> bool {
    OLD_INDEX_FORMATS.contains(&format)
        || (format == rootmarkindex::store::FORMAT && !lexical_current)
}

impl Migration for IndexRebuild {
    fn id(&self) -> &'static str {
        "index_rebuild"
    }
    fn required(&self) -> bool {
        true
    }
    fn applies(&self, vault: &Path, _: &[Record]) -> Result<bool> {
        // after the store has its name (`rename_store` comes first)
        Ok(meta::check(vault).is_ok()
            && index_manifest(vault).is_some_and(|(f, l)| index_needs_migration(&f, l)))
    }
    fn plan(&self, vault: &Path) -> Result<Plan> {
        let (format, _) = index_manifest(vault).context("no index manifest")?;
        let before = hashes(vault, &format!("{}/index", meta::META_DIR), None)?;
        Ok(Plan {
            migration: self.id(),
            required: true,
            from: format,
            to: rootmarkindex::store::FORMAT.to_string(),
            steps: vec![
                "keep the index files in the record".into(),
                "rebuild .rootmark/index/ from the log (`rootmark index build`; the log is not touched)".into(),
            ],
            bytes: bytes_of(vault, &before),
            touches: before.into_keys().collect(),
            scope: Some(format!("{}/index", meta::META_DIR)),
        })
    }
    fn apply(&self, vault: &Path, save: &mut Saver) -> Result<()> {
        let rel = format!("{}/index", meta::META_DIR);
        for f in hashes(vault, &rel, None)?.keys() {
            save.save(vault, f)?;
        }
        let v = rootmarkcore::Vault::open(vault)?;
        let e = rootmarkindex::embed::OnnxMiniLm::for_vault(vault)?;
        rootmarkindex::store::build(&v, &e)?;
        Ok(())
    }
}

/// The attestation log's writer level, as far as its lines tell: `None`
/// without a log, `2.0.1` when a line carries `partial` or `matched`, else
/// `2.0.0` (a plain 2.0.2 line is the same bytes).
fn attestations_level(store: &Path) -> Result<Option<String>> {
    let p = store.join("attestations.jsonl");
    if !p.exists() {
        return Ok(None);
    }
    let t = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    let newer = t.lines().any(|l| {
        serde_json::from_str::<Value>(l)
            .is_ok_and(|v| v.get("partial").is_some() || v.get("matched").is_some())
    });
    Ok(Some(
        if newer {
            "attestations/2.0.1"
        } else {
            "attestations/2.0.0"
        }
        .to_string(),
    ))
}

fn validated(done: &[Record], id: &str) -> bool {
    done.iter().any(|r| r.migration == id)
}

/// The recipe, vault-relative (`policy_attestation_1`'s scope).
pub fn policy_rel() -> String {
    format!("{}/policy.toml", meta::META_DIR)
}

/// The recipe's state for `policy_attestation_1`: `None` without a recipe,
/// else whether it has an `[attestation]` table. A recipe that is not TOML
/// is an error (never read as "no table").
fn policy_has_attestation(vault: &Path) -> Result<Option<bool>> {
    let p = vault.join(policy_rel());
    if std::fs::symlink_metadata(&p).is_err() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
    let t: toml::Table =
        toml::from_str(&text).with_context(|| format!("{}: not a TOML table", p.display()))?;
    Ok(Some(t.contains_key("attestation")))
}

/// The table `policy_attestation_1` appends (K-P2L-T1, option (a)).
pub const POLICY_ATTESTATION_TABLE: &str = crate::config::ATTESTATION_DEFAULT;

impl Migration for PolicyAttestation {
    fn id(&self) -> &'static str {
        "policy_attestation_1"
    }
    fn required(&self) -> bool {
        true
    }
    fn recommended(&self) -> bool {
        true
    }
    fn applies(&self, vault: &Path, _: &[Record]) -> Result<bool> {
        Ok(meta::check(vault).is_ok() && policy_has_attestation(vault)? == Some(false))
    }
    fn plan(&self, vault: &Path) -> Result<Plan> {
        let rel = policy_rel();
        let before = hashes(vault, &rel, None)?;
        Ok(Plan {
            migration: self.id(),
            required: true,
            from: "recipe without [attestation] (read as unattested = \"unverified\")".into(),
            to: "recipe with [attestation] unattested = \"canonical\" (K-P2L-T1)".into(),
            steps: vec![
                "keep policy.toml in the record".into(),
                "append `[attestation]` / `unattested = \"canonical\"` to policy.toml; every other byte is kept".into(),
            ],
            bytes: bytes_of(vault, &before),
            touches: before.into_keys().collect(),
            scope: Some(rel),
        })
    }
    fn apply(&self, vault: &Path, save: &mut Saver) -> Result<()> {
        let rel = policy_rel();
        no_links(vault, &rel)?;
        save.save(vault, &rel)?;
        let p = vault.join(&rel);
        let old = std::fs::read(&p).with_context(|| format!("read {}", p.display()))?;
        let mut new = old.clone();
        if !new.is_empty() && !new.ends_with(b"\n") {
            new.push(b'\n');
        }
        new.push(b'\n');
        new.extend_from_slice(POLICY_ATTESTATION_TABLE.as_bytes());
        // the result must be a recipe the kernel loads, with the table
        let text = String::from_utf8(new.clone()).context("policy.toml is not UTF-8")?;
        let pol = rootmarkcore::Policy::parse(&text, rootmarkcore::PolicySource::File(p.clone()))
            .context("policy_attestation_1: the recipe with the table does not load")?;
        if pol.attestation.unverified || pol.attestation.implicit {
            bail!("policy_attestation_1: the appended table is not read as unattested = \"canonical\"");
        }
        crate::paths::replace_file(&p, &new)?;
        Ok(())
    }
}

impl Migration for LogReaders {
    fn id(&self) -> &'static str {
        "log_readers"
    }
    fn required(&self) -> bool {
        false
    }
    fn applies(&self, vault: &Path, done: &[Record]) -> Result<bool> {
        Ok(meta::check(vault).is_ok()
            && meta::path(vault).join("events.jsonl").exists()
            && !validated(done, self.id()))
    }
    fn plan(&self, vault: &Path) -> Result<Plan> {
        let level = attestations_level(&meta::path(vault))?;
        Ok(Plan {
            migration: self.id(),
            required: false,
            from: format!(
                "{EVENTS_FORMAT}; {}",
                level.as_deref().unwrap_or("no attestation log")
            ),
            to: format!("{EVENTS_FORMAT}; {ATTESTATIONS_FORMAT} (reading rule)"),
            steps: vec![
                "read events.jsonl with this reader (every document replays)".into(),
                "read attestations.jsonl with the 2.0.2 reading rule (spec §8)".into(),
                "no byte changes".into(),
            ],
            touches: Vec::new(),
            bytes: 0,
            scope: None,
        })
    }
    fn apply(&self, vault: &Path, _: &mut Saver) -> Result<()> {
        let v = rootmarkcore::Vault::open(vault)?;
        for d in v.doc_paths() {
            v.replay(&d)
                .with_context(|| format!("log_readers: replay {d}"))?;
        }
        v.attestations()
            .context("log_readers: attestations.jsonl")?;
        Ok(())
    }
}

impl Migration for CheckpointsRecord {
    fn id(&self) -> &'static str {
        "checkpoints"
    }
    fn required(&self) -> bool {
        false
    }
    fn applies(&self, vault: &Path, done: &[Record]) -> Result<bool> {
        Ok(meta::check(vault).is_ok()
            && meta::path(vault).join("events.jsonl").exists()
            && !validated(done, self.id()))
    }
    fn plan(&self, _: &Path) -> Result<Plan> {
        Ok(Plan {
            migration: self.id(),
            required: false,
            from: CHECKPOINTS_FORMAT.into(),
            to: CHECKPOINTS_FORMAT.into(),
            steps: vec![
                "read every checkpoint (spec §8): the format is unchanged; a record only".into(),
            ],
            touches: Vec::new(),
            bytes: 0,
            scope: None,
        })
    }
    fn apply(&self, vault: &Path, _: &mut Saver) -> Result<()> {
        let v = rootmarkcore::Vault::open(vault)?;
        for c in v.checkpoints() {
            for (doc, snap) in &c.docs {
                rootmarkcore::store::get_snapshot(v.store_dir(), snap).with_context(|| {
                    format!("checkpoints: {} {doc}: snapshot {snap}", c.checkpoint_id)
                })?;
            }
        }
        Ok(())
    }
}

/// The registry, in order.
pub fn registry() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(RenameStore),
        Box::new(IndexRebuild),
        Box::new(PolicyAttestation),
        Box::new(LogReaders),
        Box::new(CheckpointsRecord),
    ]
}

// ---- records ------------------------------------------------------------------

fn records_dir(vault: &Path) -> PathBuf {
    meta::path(vault).join(MIGRATIONS_DIR)
}

/// The vault's rollback records, oldest first (those of a store moved back
/// under its old name too: their ordinals go on).
pub fn records(vault: &Path) -> Result<Vec<Record>> {
    let d = store_of(vault).join(MIGRATIONS_DIR);
    if !d.is_dir() {
        return Ok(Vec::new());
    }
    let mut ids: Vec<String> = std::fs::read_dir(&d)?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<_>>()?;
    ids.sort();
    let mut out = Vec::new();
    for id in ids {
        let p = d.join(&id).join("manifest.json");
        let r: Record = serde_json::from_slice(
            &std::fs::read(&p).with_context(|| format!("read {}", p.display()))?,
        )
        .with_context(|| format!("{}: not a migration record", p.display()))?;
        if r.id != id || r.format != RECORD_FORMAT {
            bail!("{}: record id or format does not match", p.display());
        }
        out.push(r);
    }
    Ok(out)
}

// ---- the version matrix (status.migrations, --dry-run) ---------------------------------

fn row(format: Option<String>, expected: &str, state: &str, note: Option<String>) -> Value {
    let mut r = json!({"format": format, "expected": expected, "state": state});
    if let Some(n) = note {
        r["note"] = json!(n);
    }
    r
}

/// The version matrix: per artifact the format on disk, the expected one
/// and the state (`ok` / `migration_required` / `unknown`); the required
/// migrations that apply, the records applied, and how to migrate. Never
/// an error for a vault this reader can describe: what it cannot read is
/// `unknown`.
pub fn matrix(vault: &Path) -> Result<Value> {
    let legacy = meta::check(vault).is_err();
    // P2-P/P0 a4: a record or a chain this reader cannot build is an error
    // (`status.migrations.error`), never an empty list
    let done = records(vault)?;
    let store = if legacy {
        row(
            Some(LEGACY_STORE_FORMAT.into()),
            STORE_FORMAT,
            "migration_required",
            Some(format!("{}/ (K-GH.1)", meta::LEGACY_META_DIR)),
        )
    } else {
        row(Some(STORE_FORMAT.into()), STORE_FORMAT, "ok", None)
    };
    let s = store_of(vault);
    let events = if s.join("events.jsonl").exists() {
        row(Some(EVENTS_FORMAT.into()), EVENTS_FORMAT, "ok", None)
    } else {
        row(None, EVENTS_FORMAT, "ok", Some("no log yet".into()))
    };
    let attestations = match attestations_level(&s) {
        Ok(None) => row(
            None,
            ATTESTATIONS_FORMAT,
            "ok",
            Some("no attestation log".into()),
        ),
        Ok(Some(l)) => row(
            Some(l),
            ATTESTATIONS_FORMAT,
            "ok",
            Some("read by the 2.0.2 reading rule (spec §8)".into()),
        ),
        Err(e) => row(None, ATTESTATIONS_FORMAT, "unknown", Some(format!("{e:#}"))),
    };
    let index = match index_manifest(vault) {
        None if !s.join("index/manifest.json").exists() => row(
            None,
            rootmarkindex::store::FORMAT,
            "unknown",
            Some("no index (`rootmark index build`)".into()),
        ),
        None => row(
            None,
            rootmarkindex::store::FORMAT,
            "unknown",
            Some("index manifest not readable".into()),
        ),
        Some((f, l)) if f == rootmarkindex::store::FORMAT && l => {
            row(Some(f), rootmarkindex::store::FORMAT, "ok", None)
        }
        Some((f, l)) if index_needs_migration(&f, l) => {
            let note = if f == rootmarkindex::store::FORMAT {
                // P2-L/L1: this format, another lexical variant
                "the index's lexical variant is not the vault's [index] tokenizer (migration index_rebuild)"
            } else {
                "an older rootmark's index (migration index_rebuild)"
            };
            row(
                Some(f),
                rootmarkindex::store::FORMAT,
                "migration_required",
                Some(note.into()),
            )
        }
        Some((f, _)) => row(
            Some(f),
            rootmarkindex::store::FORMAT,
            "unknown",
            Some("a format this rootmark does not know".into()),
        ),
    };
    let checkpoints = row(
        Some(CHECKPOINTS_FORMAT.into()),
        CHECKPOINTS_FORMAT,
        "ok",
        None,
    );
    // P2-P/P0 (K-P2L-T1): the recipe's `[attestation]` table
    let policy = match policy_has_attestation(vault) {
        _ if legacy => row(
            None,
            "recipe/attestation",
            "ok",
            Some("read after rename_store".into()),
        ),
        Ok(None) => row(None, "recipe/attestation", "ok", Some("no policy.toml".into())),
        Ok(Some(true)) => row(
            Some("recipe/attestation".into()),
            "recipe/attestation",
            "ok",
            None,
        ),
        Ok(Some(false)) => row(
            Some("recipe/implicit".into()),
            "recipe/attestation",
            "ok",
            Some("no [attestation] table: read as unattested = \"unverified\"; `rootmark migrate` adds unattested = \"canonical\" (policy_attestation_1, recommended; K-P2L-T1)".into()),
        ),
        // P2-B/B0 (review finding 8): a recipe this reader cannot read is
        // an error (`status.migrations.error`), never a silent `unknown`
        Err(e) => return Err(e),
    };
    let mut required = Vec::new();
    let mut recommended = Vec::new();
    let mut validations = Vec::new();
    for (m, _) in chain(vault, None, false)? {
        if m.recommended() {
            recommended.push(m.id());
        } else if m.required() {
            required.push(m.id());
        } else {
            validations.push(m.id());
        }
    }
    Ok(json!({
        "artifacts": {
            "store": store, "events": events, "attestations": attestations,
            "index": index, "checkpoints": checkpoints, "policy": policy,
        },
        "required": required,
        "recommended": recommended,
        "validations": validations,
        "applied": done.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        "hint": if required.is_empty() { Value::Null } else {
            // stable, like the error's hint (`status.vault` names the path)
            json!("rootmark migrate <vault>")
        },
    }))
}

/// A migration of the chain and, when asked, its plan.
type Planned = (Box<dyn Migration>, Option<Plan>);

/// The migrations `rootmark migrate` would run, in order, with their plans:
/// each evaluated on the vault as the ones before it leave it, not as it is
/// now (P2-L/L0 c; P2-A verification F1: a store under its old name only
/// listed `rename_store`, though `migrate` then rebuilt its index too).
/// With `plans`, each one's plan (it hashes the files it touches: not for
/// `status`). Read-only: after `rename_store` the later migrations are evaluated on a
/// view of the vault whose `.rootmark/` is the old store (a symbolic link,
/// or a copy where links need privileges), outside the vault, removed after.
fn chain(vault: &Path, only: Option<&str>, plans: bool) -> Result<Vec<Planned>> {
    let mut out = Vec::new();
    let mut view: Option<View> = None;
    for m in registry() {
        let at = view.as_ref().map_or(vault, |v| v.root.as_path());
        let done = records(at)?;
        if !m.applies(at, &done)? {
            continue;
        }
        let renames = m.id() == "rename_store";
        if only.is_none_or(|o| o == m.id()) {
            let plan = if plans { Some(m.plan(at)?) } else { None };
            out.push((m, plan));
        }
        if renames && view.is_none() {
            view = Some(View::renamed(vault)?);
        }
    }
    Ok(out)
}

/// A temporary vault whose `.rootmark/` is a vault's `.mdkernel/` (what
/// `rename_store` leaves), for planning only; removed when dropped.
struct View {
    root: PathBuf,
}

impl View {
    fn renamed(vault: &Path) -> Result<View> {
        // a new directory of an unpredictable name, owner-only, that must
        // not exist yet: a path another user created first is never used,
        // nor removed (a shared temporary directory)
        let mut root = None;
        for _ in 0..8 {
            let mut r = [0u8; 16];
            getrandom::fill(&mut r).map_err(|e| anyhow::anyhow!("getrandom: {e}"))?;
            let name: String = r.iter().map(|b| format!("{b:02x}")).collect();
            let p = std::env::temp_dir().join(format!("rootmark-migrate-plan-{name}"));
            let mut b = std::fs::DirBuilder::new();
            #[cfg(unix)]
            std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
            match b.create(&p) {
                Ok(()) => {
                    root = Some(p);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e).with_context(|| format!("{}", p.display())),
            }
        }
        let root = root.context("no free temporary directory for the migration plan")?;
        let view = View { root };
        let old = std::fs::canonicalize(vault.join(meta::LEGACY_META_DIR))
            .with_context(|| format!("{}/{}", vault.display(), meta::LEGACY_META_DIR))?;
        let new = view.root.join(meta::META_DIR);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&old, &new)
            .with_context(|| format!("link {} → {}", new.display(), old.display()))?;
        #[cfg(not(unix))]
        copy_tree(&old, &new)?;
        Ok(view)
    }
}

impl Drop for View {
    fn drop(&mut self) {
        // the link (or the copy) only: the vault's store is never touched
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[cfg(not(unix))]
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let t = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&e.path(), &t)?;
        } else {
            std::fs::copy(e.path(), &t)?;
        }
    }
    Ok(())
}

// ---- running ------------------------------------------------------------------

/// `rootmark migrate --dry-run`: the matrix and the plan; nothing is written.
pub fn dry_run(vault: &Path, only: Option<&str>) -> Result<Value> {
    let plans = chain(vault, only, true)?
        .into_iter()
        .map(|(m, p)| {
            let mut v = serde_json::to_value(p.expect("planned"))?;
            // P2-P/P0: a migration no vault requires says so (only then);
            // P2-B/B0 (review finding 7): and its plan does not say
            // `required` (`status.migrations` never lists it there)
            if m.recommended() {
                v["recommended"] = json!(true);
                if let Some(o) = v.as_object_mut() {
                    o.remove("required");
                }
            }
            Ok(v)
        })
        .collect::<serde_json::Result<Vec<_>>>()?;
    Ok(json!({
        "vault": vault.display().to_string(),
        "dry_run": true,
        "matrix": matrix(vault)?,
        "plan": plans,
        "events_hash": events_hash_at(&store_of(vault))?,
    }))
}

/// Runs one migration under the framework's guard and writes its record.
pub fn run_one(vault: &Path, m: &dyn Migration) -> Result<Record> {
    let done = records(vault)?;
    if !m.applies(vault, &done)? {
        bail!("migration {} does not apply to {}", m.id(), vault.display());
    }
    let plan = m.plan(vault)?;
    let hash0 = events_hash_at(&store_of(vault))?;
    // before: every file of the rewritten directory (the touched files for
    // a validation: none)
    let skip_of = |d: &str| (d == meta::META_DIR).then_some(MIGRATIONS_DIR);
    let before: BTreeMap<String, String> = match &plan.scope {
        Some(d) if m.id() == "rename_store" => hashes(vault, meta::LEGACY_META_DIR, None)?,
        Some(d) => hashes(vault, d, skip_of(d))?,
        None => plan
            .touches
            .iter()
            .map(|p| Ok((p.clone(), sha256(&std::fs::read(vault.join(p))?))))
            .collect::<Result<_>>()?,
    };
    // the record is staged outside the store (the store itself may move)
    let n = done.len() + 1;
    let id = format!("{n:04}-{}", m.id());
    let staging = vault.join(format!(".rootmark-migration-{id}"));
    if staging.exists() {
        bail!(
            "{}: a staging directory of an interrupted migration is in the way",
            staging.display()
        );
    }
    std::fs::create_dir_all(&staging)?;
    let mut save = Saver {
        staging: staging.clone(),
        saved: BTreeMap::new(),
    };
    // the log's bytes, whatever the scope: a migration that changes them is
    // undone whatever it touched (A1 review O2)
    let log0 = std::fs::read(store_of(vault).join("events.jsonl")).ok();
    let res = m.apply(vault, &mut save);
    let hash1 = events_hash_at(&store_of(vault));
    let undo = if m.id() == "rename_store" {
        "rename"
    } else {
        "files"
    };
    // after: every file now in the rewritten directory (the store minus
    // the records for a rename)
    let after = match &plan.scope {
        Some(d) => hashes(vault, d, skip_of(d)),
        None => Ok(BTreeMap::new()),
    };
    let mut failed = match (&res, &hash1, &after) {
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Some(format!("{e:#}")),
        (Ok(()), Ok(h), _) if *h != hash0 => Some(format!(
            "migration {} would change events_hash ({hash0} → {h}); a migration never does",
            m.id()
        )),
        _ => None,
    };
    // every file it replaced or removed must have its copy in the record
    // (else the rollback could not restore it)
    if let (None, "files", Ok(a)) = (&failed, undo, &after) {
        if let Some(rel) = before
            .iter()
            .find(|(r, h)| a.get(*r) != Some(*h) && save.saved.get(*r) != Some(*h))
            .map(|(r, _)| r)
        {
            failed = Some(format!(
                "migration {} changed {rel} without keeping its copy",
                m.id()
            ));
        }
    }
    if let Some(why) = failed {
        // undo what was done, from the staged copies
        let mut undone = undo_partial(
            vault,
            undo,
            plan.scope.as_deref(),
            &before,
            &save.saved,
            &staging,
        );
        let _ = std::fs::remove_dir_all(&staging);
        // a failed index build leaves its work directory (A1 review K2)
        let _ = std::fs::remove_dir_all(meta::path(vault).join("index.build"));
        if undone.is_ok() {
            let log = store_of(vault).join("events.jsonl");
            if let Some(b) = &log0 {
                if std::fs::read(&log).ok().as_ref() != Some(b) {
                    undone = std::fs::write(&log, b).map_err(anyhow::Error::from);
                }
            }
            if undone.is_ok()
                && events_hash_at(&store_of(vault)).ok().as_deref() != Some(hash0.as_str())
            {
                undone = Err(anyhow::anyhow!("events_hash still differs after the undo"));
            }
        }
        match undone {
            Ok(()) => bail!("{why}; stopped, the vault is as it was"),
            Err(e) => bail!("{why}; and the undo failed: {e:#} (the vault needs a manual check)"),
        }
    }
    let after = after?;
    let rec = Record {
        format: RECORD_FORMAT.into(),
        id: id.clone(),
        migration: m.id().into(),
        from: plan.from.clone(),
        to: plan.to.clone(),
        events_hash: hash0,
        before,
        after,
        saved: save.saved,
        undo: undo.into(),
        scope: plan.scope.clone(),
        checked: if plan.touches.is_empty() {
            plan.steps.clone()
        } else {
            Vec::new()
        },
    };
    std::fs::write(
        staging.join("manifest.json"),
        serde_json::to_vec_pretty(&rec)?,
    )?;
    let dir = records_dir(vault);
    std::fs::create_dir_all(&dir)?;
    std::fs::rename(&staging, dir.join(&id)).with_context(|| format!("install the record {id}"))?;
    Ok(rec)
}

/// Puts the touched files back after a failed or refused migration.
fn undo_partial(
    vault: &Path,
    undo: &str,
    scope: Option<&str>,
    before: &BTreeMap<String, String>,
    saved: &BTreeMap<String, String>,
    staging: &Path,
) -> Result<()> {
    if undo == "rename" {
        let (from, to) = (
            vault.join(meta::META_DIR),
            vault.join(meta::LEGACY_META_DIR),
        );
        if from.exists() && !to.exists() {
            std::fs::rename(&from, &to)?;
        }
    } else if let Some(d) = scope {
        let now = hashes(vault, d, (d == meta::META_DIR).then_some(MIGRATIONS_DIR))?;
        for rel in now.keys().filter(|r| !before.contains_key(*r)) {
            std::fs::remove_file(vault.join(rel))?;
        }
        restore_files(vault, before, saved, &now, &staging.join("files"))?;
        remove_empty_dirs(vault, now.keys())?;
    }
    check_equal(vault, before)
}

/// Restores the files of `before` from `copies` (and removes the files the
/// migration created under the same directories).
fn restore_files(
    vault: &Path,
    before: &BTreeMap<String, String>,
    saved: &BTreeMap<String, String>,
    now: &BTreeMap<String, String>,
    copies: &Path,
) -> Result<()> {
    for (rel, want) in before {
        if now.get(rel) == Some(want) {
            continue; // untouched
        }
        let src = copies.join(rel);
        let b = std::fs::read(&src).with_context(|| format!("record copy of {rel} missing"))?;
        if sha256(&b) != *want || saved.get(rel) != Some(want) {
            bail!("record copy of {rel} does not match its SHA-256");
        }
        no_links(vault, rel)?;
        let to = vault.join(rel);
        std::fs::create_dir_all(to.parent().expect("a file"))?;
        no_links(vault, rel)?;
        std::fs::write(&to, &b)?;
    }
    Ok(())
}

/// Every file of `want` byte-equal (by SHA-256) on disk.
fn check_equal(vault: &Path, want: &BTreeMap<String, String>) -> Result<()> {
    for (rel, h) in want {
        let b = std::fs::read(vault.join(rel)).with_context(|| format!("{rel} missing"))?;
        if sha256(&b) != *h {
            bail!("{rel} differs from the record");
        }
    }
    Ok(())
}

/// `rootmark migrate [--rename-store]`: every migration that applies (or
/// only `only`), in order, each with its record.
pub fn migrate(vault: &Path, only: Option<&str>) -> Result<Value> {
    let mut applied = Vec::new();
    for m in registry() {
        if only.is_some_and(|o| o != m.id()) {
            continue;
        }
        let done = records(vault)?;
        if !m.applies(vault, &done)? {
            continue;
        }
        let r = run_one(vault, m.as_ref())?;
        applied.push(json!({
            "id": r.id, "migration": r.migration, "from": r.from, "to": r.to,
            "files": r.after.len().max(r.before.len()),
        }));
    }
    if let Some(o) = only {
        if applied.is_empty() {
            bail!("migration {o} does not apply to {}", vault.display());
        }
    }
    Ok(json!({
        "vault": vault.display().to_string(),
        "applied": applied,
        "matrix": matrix(vault)?,
        "events_hash": events_hash_at(&store_of(vault))?,
    }))
}

/// `rootmark migrate --rollback <id>`: the newest record only.
pub fn rollback(vault: &Path, id: &str) -> Result<Value> {
    let done = records(vault)?;
    let Some(rec) = done.last() else {
        bail!("{}: no migration record to roll back", vault.display());
    };
    if rec.id != id {
        if done.iter().any(|r| r.id == id) {
            bail!(
                "roll back {} first (records are undone newest first)",
                rec.id
            );
        }
        bail!("no migration record {id}");
    }
    check_record_paths(rec)?;
    for p in rec.before.keys().chain(rec.after.keys()) {
        no_links(vault, p)?;
    }
    let dir = records_dir(vault).join(id);
    // the record: every copy as its SHA-256 says
    for (rel, h) in &rec.saved {
        let b = std::fs::read(dir.join("files").join(rel))
            .with_context(|| format!("record {id}: copy of {rel} missing"))?;
        if sha256(&b) != *h || rec.before.get(rel) != Some(h) {
            bail!("record {id}: the copy of {rel} does not match its SHA-256; nothing rolled back");
        }
    }
    if rec.undo == "files"
        && rec
            .before
            .iter()
            .any(|(r, h)| rec.after.get(r) != Some(h) && rec.saved.get(r) != Some(h))
    {
        bail!("record {id}: a copy is missing; nothing rolled back");
    }
    // the vault: the rewritten directory exactly as the migration left it
    // (no file changed, removed or added since)
    if let Some(d) = &rec.scope {
        let skip = (d == meta::META_DIR).then_some(MIGRATIONS_DIR);
        if hashes(vault, d, skip)? != rec.after {
            bail!("{d}/ changed since migration {id}; nothing rolled back");
        }
    }
    let hash = events_hash_at(&meta::path(vault))?;
    if hash != rec.events_hash {
        bail!(
            "events_hash changed since migration {id} ({} → {hash}); nothing rolled back",
            rec.events_hash
        );
    }
    // the copies leave the store before it changes
    let tmp = vault.join(format!(".rootmark-rollback-{id}"));
    if tmp.exists() {
        bail!("{}: in the way", tmp.display());
    }
    std::fs::rename(&dir, &tmp).with_context(|| format!("take the record {id} out"))?;
    let records = records_dir(vault);
    if std::fs::read_dir(&records)?.next().is_none() {
        std::fs::remove_dir(&records)?;
    }
    let res = (|| -> Result<()> {
        match rec.undo.as_str() {
            "rename" => {
                let (from, to) = (meta::path(vault), vault.join(meta::LEGACY_META_DIR));
                if to.exists() {
                    bail!("{} exists", to.display());
                }
                std::fs::rename(&from, &to)?;
            }
            _ => {
                for rel in rec.after.keys() {
                    if !rec.before.contains_key(rel) {
                        no_links(vault, rel)?;
                        std::fs::remove_file(vault.join(rel))?;
                    }
                }
                restore_files(
                    vault,
                    &rec.before,
                    &rec.saved,
                    &rec.after,
                    &tmp.join("files"),
                )?;
                remove_empty_dirs(vault, rec.after.keys())?;
            }
        }
        check_equal(vault, &rec.before)?;
        let h = events_hash_at(&store_of(vault))?;
        if h != rec.events_hash {
            bail!(
                "events_hash after the rollback is {h}, not {}",
                rec.events_hash
            );
        }
        Ok(())
    })();
    match res {
        Ok(()) => {
            std::fs::remove_dir_all(&tmp)?;
            Ok(json!({
                "vault": vault.display().to_string(),
                "rolled_back": rec.id, "migration": rec.migration,
                "files": rec.before.len(), "byte_equal": true,
                "events_hash": rec.events_hash,
            }))
        }
        Err(e) => bail!(
            "rollback of {id} failed: {e:#}; the record is kept at {} (check the vault by hand; to retry, move it back to {})",
            tmp.display(),
            dir.display()
        ),
    }
}

/// A record lies in the vault and can be edited: every path it names must
/// be a plain relative path inside the directory the migration rewrote —
/// the store, its index, or (a rename's `before`) the pre-rename store —
/// and the scope one of those; nothing else is ever removed or written by a
/// rollback (automated security review of A1).
fn check_record_paths(rec: &Record) -> Result<()> {
    use std::path::Component;
    let index = format!("{}/index", meta::META_DIR);
    // each registered migration's own undo and scope (A1 review O1): a
    // record cannot widen them (no `files` undo of the whole store)
    let (scope, before_root) = match (
        rec.migration.as_str(),
        rec.undo.as_str(),
        rec.scope.as_deref(),
    ) {
        ("rename_store", "rename", Some(s)) if s == meta::META_DIR => {
            (s.to_string(), meta::LEGACY_META_DIR.to_string())
        }
        ("index_rebuild", "files", Some(s)) if s == index => (s.to_string(), s.to_string()),
        // one file, the recipe (P2-P/P0): every path is that file
        ("policy_attestation_1", "files", Some(s)) if s == policy_rel() => {
            if rec
                .before
                .keys()
                .chain(rec.saved.keys())
                .chain(rec.after.keys())
                .any(|p| *p != s)
            {
                bail!(
                    "record {}: a path other than {s}; nothing rolled back",
                    rec.id
                );
            }
            return Ok(());
        }
        ("log_readers" | "checkpoints", "files", None)
            if rec.before.is_empty() && rec.after.is_empty() && rec.saved.is_empty() =>
        {
            return Ok(());
        }
        _ => bail!("record {}: undo {:?} with scope {:?} is not one this rootmark writes; nothing rolled back", rec.id, rec.undo, rec.scope),
    };
    let plain = |p: &str, root: &str| {
        let path = Path::new(p);
        path.components().all(|c| matches!(c, Component::Normal(_)))
            && path.starts_with(root)
            && path != Path::new(root)
            && !path.starts_with(format!("{}/{MIGRATIONS_DIR}", meta::META_DIR))
    };
    for p in rec.before.keys().chain(rec.saved.keys()) {
        if !plain(p, &before_root) {
            bail!(
                "record {}: path {p:?} outside {before_root}/; nothing rolled back",
                rec.id
            );
        }
    }
    for p in rec.after.keys() {
        if !plain(p, &scope) {
            bail!(
                "record {}: path {p:?} outside {scope}/; nothing rolled back",
                rec.id
            );
        }
    }
    Ok(())
}

/// Removes the directories the migration created that are now empty.
fn remove_empty_dirs<'a>(vault: &Path, paths: impl Iterator<Item = &'a String>) -> Result<()> {
    let mut dirs: Vec<PathBuf> = paths
        .filter_map(|p| vault.join(p).parent().map(Path::to_path_buf))
        .collect();
    dirs.sort();
    dirs.dedup();
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let mut cur = d;
        while cur.starts_with(meta::path(vault)) && cur != meta::path(vault) {
            if cur.is_dir() && std::fs::read_dir(&cur)?.next().is_none() {
                std::fs::remove_dir(&cur)?;
            } else {
                break;
            }
            match cur.parent() {
                Some(p) => cur = p.to_path_buf(),
                None => break,
            }
        }
    }
    Ok(())
}
