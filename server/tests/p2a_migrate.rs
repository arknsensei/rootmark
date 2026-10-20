//! P2-A/A1 (GOAL P2-A A-1): the migration framework.
//! - every registered migration forward and back: `rename_store` and
//!   `index_rebuild` on the pre-rename `/2` index fixture
//!   (`server/tests/fixtures/index_v2`), `log_readers` and `checkpoints` on
//!   a vault with an attestation log and a checkpoint; after `--rollback`
//!   the vault is byte-equal to the one before (every file, the record
//!   gone);
//! - `--dry-run` changes no byte and prints the plan;
//! - a second `migrate` is a no-op (no record);
//! - `events_hash`, replay and blame never change;
//! - a migration that would change `events_hash` is undone and stops with
//!   an error (no record); a damaged record or a vault changed since the
//!   migration is refused by `--rollback` (nothing restored).
//!
//! `index_rebuild` needs the model files (`make models`).

use rootmarkserver::migrate::{self, Migration, Plan, Saver};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn rootmark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(args)
        .output()
        .unwrap()
}

fn ok_json(o: Output) -> Value {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2a1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// A copy of the pre-rename `/2` index fixture.
fn legacy(name: &str) -> PathBuf {
    let root = tmp(name);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_v2/vault");
    assert!(Command::new("cp")
        .args(["-R", src.to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    root.join("vault")
}

/// Every file under `dir` with its bytes (relative paths).
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// Every directory under `dir` (empty ones included).
fn dirs(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.push(p.strip_prefix(dir).unwrap().to_string_lossy().into_owned());
                stack.push(p);
            }
        }
    }
    out.sort();
    out
}

fn ids(v: &Path) -> Vec<String> {
    migrate::records(v)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect()
}

fn answers(v: &Path, docs: &[&str]) -> Vec<Value> {
    let vs = v.to_str().unwrap();
    let mut out = vec![ok_json(rootmark(&["events-hash", vs]))];
    for d in docs {
        out.push(ok_json(rootmark(&["replay", vs, d])));
        out.push(ok_json(rootmark(&["blame", vs, d])));
    }
    out
}

#[test]
fn dry_run_changes_nothing_and_names_the_plan() {
    let v = legacy("dry");
    let before = tree(&v);
    let r = ok_json(rootmark(&["migrate", "--dry-run", v.to_str().unwrap()]));
    assert_eq!(tree(&v), before, "--dry-run wrote something");
    assert_eq!(dirs(&v), dirs(&legacy("dry-ref")));
    let plan: Vec<&str> = r["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["migration"].as_str().unwrap())
        .collect();
    // the index migration is planned after the rename: from the legacy
    // store it is only named once the store has its new name
    assert_eq!(plan, ["rename_store"], "{r:#}");
    assert_eq!(
        r["matrix"]["artifacts"]["store"]["state"],
        "migration_required"
    );
    assert_eq!(
        r["matrix"]["artifacts"]["index"]["format"],
        "mdkernel-index/2"
    );
    assert_eq!(
        r["matrix"]["artifacts"]["index"]["state"],
        "migration_required"
    );
    assert!(r["plan"][0]["bytes"].as_u64().unwrap() > 0);
}

#[test]
fn rename_store_and_index_rebuild_forward_and_back() {
    let v = legacy("both");
    let vs = v.to_str().unwrap();
    let orig = tree(&v);
    let orig_dirs = dirs(&v);
    let r = ok_json(rootmark(&["migrate", vs]));
    let applied: Vec<&str> = r["applied"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["migration"].as_str().unwrap())
        .collect();
    assert_eq!(
        applied,
        [
            "rename_store",
            "index_rebuild",
            "log_readers",
            "checkpoints"
        ],
        "{r:#}"
    );
    assert_eq!(
        ids(&v),
        [
            "0001-rename_store",
            "0002-index_rebuild",
            "0003-log_readers",
            "0004-checkpoints"
        ]
    );
    assert!(v.join(".rootmark").is_dir() && !v.join(".mdkernel").exists());
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(v.join(".rootmark/index/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["format"], rootmarkindex::store::FORMAT);
    assert_eq!(r["matrix"]["required"], serde_json::json!([]));
    assert_eq!(r["matrix"]["artifacts"]["index"]["state"], "ok");
    // the log, its replay and blame as before (gh0_migrate's values)
    assert_eq!(r["events_hash"], "3fdac071e50f0e70f2bb4585ebdbca8c");
    let migrated = answers(&v, &["espresso.md", "safety.md"]);
    // the rebuilt index opens (a search would write a checkpoint event)
    ok_json(rootmark(&["index", "status", vs]));

    // idempotent: nothing left
    let again = ok_json(rootmark(&["migrate", vs]));
    assert_eq!(again["applied"], serde_json::json!([]));
    assert_eq!(ids(&v).len(), 4);

    // back, newest first; an older record first is refused
    let e = rootmark(&["migrate", "--rollback", "0001-rename_store", vs]);
    assert!(!e.status.success());
    assert!(String::from_utf8_lossy(&e.stderr).contains("first"));
    for id in ["0004-checkpoints", "0003-log_readers", "0002-index_rebuild"] {
        let b = ok_json(rootmark(&["migrate", "--rollback", id, vs]));
        assert_eq!(b["byte_equal"], true);
        assert_eq!(answers(&v, &["espresso.md", "safety.md"]), migrated);
    }
    let idx: Value =
        serde_json::from_slice(&std::fs::read(v.join(".rootmark/index/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(idx["format"], "mdkernel-index/2", "the /2 index is back");
    ok_json(rootmark(&[
        "migrate",
        "--rollback",
        "0001-rename_store",
        vs,
    ]));
    assert_eq!(tree(&v), orig, "not byte-equal after the rollback");
    assert_eq!(dirs(&v), orig_dirs, "a directory was left behind");
}

/// A vault of the current format with an attestation log and a checkpoint.
fn current(name: &str) -> PathBuf {
    let v = tmp(name).join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("a.md"), "# A\n\nHuman text.\n").unwrap();
    let vs = v.to_str().unwrap();
    ok_json(rootmark(&["ingest-dir", vs, vs]));
    ok_json(rootmark(&["checkpoint", "create", vs]));
    // a 2.0.1 line (`matched: false`): bytes the file never held
    let mut a =
        rootmarkcore::attest::Attestation::new("a.md", "0123456789abcdef0123456789abcdef", 0, None)
            .unwrap();
    a.matched = Some(false);
    std::fs::write(
        v.join(".rootmark/attestations.jsonl"),
        format!("{}\n", serde_json::to_string(&a).unwrap()),
    )
    .unwrap();
    v
}

#[test]
fn validations_record_and_roll_back_without_a_byte_changed() {
    let v = current("valid");
    let vs = v.to_str().unwrap();
    let orig = tree(&v);
    let orig_dirs = dirs(&v);
    let d = ok_json(rootmark(&["migrate", "--dry-run", vs]));
    assert_eq!(tree(&v), orig);
    assert_eq!(
        d["matrix"]["artifacts"]["attestations"]["format"],
        "attestations/2.0.1"
    );
    assert_eq!(d["matrix"]["required"], serde_json::json!([]));
    let r = ok_json(rootmark(&["migrate", vs]));
    assert_eq!(ids(&v), ["0001-log_readers", "0002-checkpoints"], "{r:#}");
    // the record is the only new file
    let mut now = tree(&v);
    now.retain(|k, _| !k.starts_with(".rootmark/migrations"));
    assert_eq!(now, orig);
    for id in ["0002-checkpoints", "0001-log_readers"] {
        ok_json(rootmark(&["migrate", "--rollback", id, vs]));
    }
    assert_eq!(tree(&v), orig);
    assert_eq!(dirs(&v), orig_dirs);
}

#[test]
fn a_rollback_refuses_a_damaged_record_or_a_changed_vault() {
    let v = legacy("tamper");
    let vs = v.to_str().unwrap();
    ok_json(rootmark(&["migrate", vs]));
    for id in ["0004-checkpoints", "0003-log_readers"] {
        ok_json(rootmark(&["migrate", "--rollback", id, vs]));
    }
    // a copy in the record no longer matches its SHA-256
    let rec = v.join(".rootmark/migrations/0002-index_rebuild/files/.rootmark/index/manifest.json");
    let good = std::fs::read(&rec).unwrap();
    std::fs::write(&rec, [&good[..], b" "].concat()).unwrap();
    let before = tree(&v);
    let e = rootmark(&["migrate", "--rollback", "0002-index_rebuild", vs]);
    assert!(!e.status.success());
    assert!(
        String::from_utf8_lossy(&e.stderr).contains("does not match its SHA-256"),
        "{}",
        String::from_utf8_lossy(&e.stderr)
    );
    assert_eq!(tree(&v), before, "a refused rollback changed something");
    std::fs::write(&rec, &good).unwrap();
    // the index changed since the migration
    let m = v.join(".rootmark/index/manifest.json");
    let mb = std::fs::read(&m).unwrap();
    std::fs::write(&m, [&mb[..], b"\n"].concat()).unwrap();
    let before = tree(&v);
    let e = rootmark(&["migrate", "--rollback", "0002-index_rebuild", vs]);
    assert!(!e.status.success());
    assert!(String::from_utf8_lossy(&e.stderr).contains("changed since"));
    assert_eq!(tree(&v), before);
    std::fs::write(&m, &mb).unwrap();
    ok_json(rootmark(&[
        "migrate",
        "--rollback",
        "0002-index_rebuild",
        vs,
    ]));
    // the log changed since the rename (an appended event): refused
    let ev = v.join(".rootmark/events.jsonl");
    let eb = std::fs::read(&ev).unwrap();
    std::fs::write(&ev, [&eb[..], b"{}\n"].concat()).unwrap();
    let e = rootmark(&["migrate", "--rollback", "0001-rename_store", vs]);
    assert!(!e.status.success());
    assert!(v.join(".rootmark").is_dir());
}

/// A migration that rewrites the log.
struct Forger;

impl Migration for Forger {
    fn id(&self) -> &'static str {
        "forger"
    }
    fn required(&self) -> bool {
        true
    }
    fn applies(&self, _: &Path, _: &[migrate::Record]) -> anyhow::Result<bool> {
        Ok(true)
    }
    fn plan(&self, _: &Path) -> anyhow::Result<Plan> {
        Ok(Plan {
            migration: "forger",
            required: true,
            from: "x".into(),
            to: "y".into(),
            steps: vec!["append an event".into()],
            touches: Vec::new(),
            bytes: 0,
            scope: None,
        })
    }
    fn apply(&self, v: &Path, _: &mut Saver) -> anyhow::Result<()> {
        let p = v.join(".rootmark/events.jsonl");
        let mut b = std::fs::read(&p)?;
        b.extend_from_slice(b"{\"forged\":true}\n");
        std::fs::write(p, b)?;
        Ok(())
    }
}

#[test]
fn a_migration_that_would_change_events_hash_stops() {
    let v = current("forge");
    let orig = tree(&v);
    let e = migrate::run_one(&v, &Forger).unwrap_err();
    let msg = format!("{e:#}");
    assert!(msg.contains("would change events_hash"), "{msg}");
    assert!(msg.contains("the vault is as it was"), "{msg}");
    assert_eq!(tree(&v), orig, "not restored");
    assert!(ids(&v).is_empty(), "a record was written");
}

#[test]
fn the_legacy_flag_still_renames_only() {
    let v = legacy("flag");
    let vs = v.to_str().unwrap();
    let r = ok_json(rootmark(&["migrate", "--rename-store", vs]));
    assert_eq!(r["events_hash"], "3fdac071e50f0e70f2bb4585ebdbca8c");
    assert_eq!(
        r["renamed"]["to"],
        v.join(".rootmark").display().to_string()
    );
    assert_eq!(ids(&v), ["0001-rename_store"]);
    // the index stays as it was: migrated with the next `rootmark migrate`
    let st = ok_json(rootmark(&["migrate", "--dry-run", vs]));
    assert_eq!(
        st["matrix"]["required"],
        serde_json::json!(["index_rebuild"])
    );
    assert!(!rootmark(&["migrate", "--rename-store", vs])
        .status
        .success());
}

#[test]
fn a_record_naming_paths_outside_its_scope_is_refused() {
    // the record lies in the vault: an edited one must not make a rollback
    // remove or write anything else (a markdown file, a path outside)
    let v = legacy("paths");
    let vs = v.to_str().unwrap();
    ok_json(rootmark(&["migrate", vs]));
    for id in ["0004-checkpoints", "0003-log_readers"] {
        ok_json(rootmark(&["migrate", "--rollback", id, vs]));
    }
    let m = v.join(".rootmark/migrations/0002-index_rebuild/manifest.json");
    let good = std::fs::read(&m).unwrap();
    for bad in [
        "espresso.md",
        "../outside.md",
        ".rootmark/index/../../espresso.md",
    ] {
        let mut r: Value = serde_json::from_slice(&good).unwrap();
        r["after"][bad] = Value::String("0".repeat(64));
        std::fs::write(&m, serde_json::to_vec(&r).unwrap()).unwrap();
        let before = tree(&v);
        let e = rootmark(&["migrate", "--rollback", "0002-index_rebuild", vs]);
        assert!(!e.status.success(), "{bad}");
        assert!(
            String::from_utf8_lossy(&e.stderr).contains("outside"),
            "{bad}: {}",
            String::from_utf8_lossy(&e.stderr)
        );
        assert_eq!(tree(&v), before, "{bad}");
    }
    std::fs::write(&m, &good).unwrap();
    ok_json(rootmark(&[
        "migrate",
        "--rollback",
        "0002-index_rebuild",
        vs,
    ]));
}

#[test]
fn a_record_cannot_widen_its_migration_s_scope() {
    // A1 review O1: a hand-written record undoing "files" over the whole
    // store (what no registered migration writes) is refused, the store kept
    let v = current("widen");
    let vs = v.to_str().unwrap();
    ok_json(rootmark(&["migrate", vs]));
    let all: BTreeMap<String, String> = tree(&v.join(".rootmark"))
        .into_iter()
        .filter(|(k, _)| !k.starts_with("migrations"))
        .map(|(k, b)| {
            let h = hmac_sha256::Hash::hash(&b)
                .iter()
                .map(|x| format!("{x:02x}"))
                .collect::<String>();
            (format!(".rootmark/{k}"), h)
        })
        .collect();
    let m = v.join(".rootmark/migrations/0002-checkpoints/manifest.json");
    let mut r: Value = serde_json::from_slice(&std::fs::read(&m).unwrap()).unwrap();
    r["scope"] = Value::String(".rootmark".into());
    r["after"] = serde_json::to_value(&all).unwrap();
    std::fs::write(&m, serde_json::to_vec(&r).unwrap()).unwrap();
    let before = tree(&v);
    let e = rootmark(&["migrate", "--rollback", "0002-checkpoints", vs]);
    assert!(!e.status.success());
    assert_eq!(tree(&v), before, "the store was touched");
}
