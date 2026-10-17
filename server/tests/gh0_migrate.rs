//! GH-0 GH-3 (K-GH.1, ASSUMPTIONS G0-6): a vault written before the rename
//! (`server/tests/fixtures/index_v2/vault`, written by the `p1d-final`
//! binary, metadata under `.mdkernel/`, index format `mdkernel-index/2`).
//! - Without migration `rootmark` refuses it and names the migration; no
//!   `.rootmark/` is created beside it.
//! - `rootmark migrate --rename-store` renames the directory and nothing
//!   else: every file in it keeps its bytes (events, objects, snapshots, the
//!   index and its manifest: the format string is a data identifier and kept
//!   its value, G0-2); `events_hash`, `replay` and `blame` equal the values
//!   the `p1d-final` binary prints for the fixture (measured once, below); a
//!   second migration is refused.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `p1d-final` `mdk events-hash` / `replay` / `blame` on the fixture vault
/// (sha256 of the printed JSON for replay and blame; GH-0 G0.md).
const OLD_EVENTS_HASH: &str = "3fdac071e50f0e70f2bb4585ebdbca8c";
const OLD_REPLAY_ESPRESSO: &str =
    "928aa0fbec5301448673fccc2f91919f5df9932bfc3324ea7bb0ea1ae491a000";
const OLD_REPLAY_SAFETY: &str = "1804a0940c9e6f251a9a0a76ae99e333dbb6fd7014a01d5882fc543009e2a716";
const OLD_BLAME_ESPRESSO: &str = "e931499d2c8bab6f9851b483678e6c8202bf4965feb0c0eaf02a359c2c1d1e9e";

fn rootmark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rootmark"))
        .args(args)
        .output()
        .unwrap()
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/index_v2/vault")
}

fn copy_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-gh0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let st = Command::new("cp")
        .args(["-R", fixture().to_str().unwrap(), root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(st.success());
    root.join("vault")
}

fn sha256(b: &[u8]) -> String {
    let out = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            c.stdin.take().unwrap().write_all(b)?;
            c.wait_with_output()
        })
        .unwrap();
    String::from_utf8(out.stdout).unwrap()[..64].to_string()
}

#[test]
fn legacy_vault_is_refused_until_migrated() {
    let v = copy_fixture("refused");
    let vs = v.to_str().unwrap();
    for args in [vec!["events-hash", vs], vec!["blame", vs, "espresso.md"]] {
        let o = rootmark(&args);
        assert!(!o.status.success(), "{args:?} opened a legacy vault");
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(err.contains("rootmark migrate --rename-store"), "{err}");
    }
    assert!(
        !v.join(".rootmark").exists(),
        "a .rootmark/ was created beside .mdkernel/"
    );
    assert!(v.join(".mdkernel/events.jsonl").is_file());
}

/// Every file under `dir`, by path relative to it, with its bytes.
fn tree(dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
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

#[test]
fn migrate_rename_store_keeps_every_byte() {
    let v = copy_fixture("migrate");
    let vs = v.to_str().unwrap();
    let o = rootmark(&["migrate", "--rename-store", vs]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["events_hash"], OLD_EVENTS_HASH);
    assert!(!v.join(".mdkernel").exists());

    // the store as the p1d-final binary wrote it, byte for byte
    let old = tree(&fixture().join(".mdkernel"));
    assert!(old.contains_key("events.jsonl") && old.contains_key("index/manifest.json"));
    assert_eq!(tree(&v.join(".rootmark")), old);

    // the same answers as the p1d-final binary
    let eh: serde_json::Value =
        serde_json::from_slice(&rootmark(&["events-hash", vs]).stdout).unwrap();
    assert_eq!(eh["events_hash"], OLD_EVENTS_HASH);
    assert_eq!(
        sha256(&rootmark(&["replay", vs, "espresso.md"]).stdout),
        OLD_REPLAY_ESPRESSO
    );
    assert_eq!(
        sha256(&rootmark(&["replay", vs, "safety.md"]).stdout),
        OLD_REPLAY_SAFETY
    );
    assert_eq!(
        sha256(&rootmark(&["blame", vs, "espresso.md"]).stdout),
        OLD_BLAME_ESPRESSO
    );

    // nothing left to migrate
    let again = rootmark(&["migrate", "--rename-store", vs]);
    assert!(!again.status.success());
    let none = rootmark(&["migrate", vs]);
    assert!(!none.status.success());
}

/// A `/3` index written under the old name (what `p1e-final` writes: the
/// same bytes as today's index under `.mdkernel/`; GH-0 G0.md compares it
/// with a real `p1e-final` index) is usable after the migration without a
/// rebuild: `index status` answers and a search returns the same hits (G0
/// review finding 1). Needs the model files.
#[test]
fn migrated_format_3_index_is_read_as_it_is() {
    let v = copy_fixture("format3");
    let vs = v.to_str().unwrap();
    assert!(rootmark(&["migrate", "--rename-store", vs])
        .status
        .success());
    let b = rootmark(&["index", "build", vs]);
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    let hits = |o: Output| -> serde_json::Value {
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let r: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        r["hits"].clone()
    };
    let before = hits(rootmark(&["search", vs, "extinguisher roaster"]));
    assert!(before.as_array().is_some_and(|h| !h.is_empty()));
    let store = tree(&v.join(".rootmark"));
    assert!(String::from_utf8_lossy(&store["index/manifest.json"]).contains("\"mdkernel-index/3\""));

    // the pre-rename layout, then the migration
    std::fs::rename(v.join(".rootmark"), v.join(".mdkernel")).unwrap();
    let o = rootmark(&["migrate", "--rename-store", vs]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(tree(&v.join(".rootmark")), store);
    let st = rootmark(&["index", "status", vs]);
    assert!(
        st.status.success(),
        "{}",
        String::from_utf8_lossy(&st.stderr)
    );
    assert_eq!(
        hits(rootmark(&["search", vs, "extinguisher roaster"])),
        before
    );
}
