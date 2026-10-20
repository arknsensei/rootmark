//! P2-A/A0 (K-P2-02.5; GOAL P2-A A-7):
//! - (a) `rootmark serve` writes `[attestation] unattested = "canonical"`
//!   explicitly into a vault without `policy.toml`; `status.attestation`
//!   reports it and an unattested external change is the human's; choosing
//!   `unverified` gives P2-0.2's behaviour (`unverified`, actor
//!   `unattested_fs`); a policy without the table keeps the kernel's (and
//!   spec §7.1's) default `unverified`;
//! - (c) `--attest-log <file>` (outside the vault only): one line per
//!   attestation call (accepted, or refused with its code and reason) and
//!   per observed change (attested with its distance in `seq`; unverified
//!   with the accepted attestations of the document that did not match it,
//!   and why: other bytes, window expired).
//!
//! (b), the renamed note's own late link update, is in `p2_0_2_carry.rs`.
//! Needs the model files (`make models`).

use rootmarkserver::attest::{mac, mac_message};
use rootmarkserver::attest_log::AttestLog;
use rootmarkserver::error::{code_of, Code};
use rootmarkserver::tools::{Caller, Kernel};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

const NOTES: &str = "# Notes\n\nA human paragraph about roasting.\n";

/// A temp root with `vault/` holding one ingested note (no policy).
fn root(name: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("rootmark-p2a0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("notes.md"), NOTES).unwrap();
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    (root, v)
}

/// `rootmark serve --mcp stdio` until the (empty) MCP stream ends: writes
/// the default recipe and builds the index.
fn serve_once(v: &Path) {
    let st = Command::new(rootmark())
        .args(["serve", "--vault", v.to_str().unwrap(), "--mcp", "stdio"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        st.status.success(),
        "{}",
        String::from_utf8_lossy(&st.stderr)
    );
}

fn human() -> Caller<'static> {
    Caller::http("p2-a-test")
}

fn layer_of(k: &Kernel, doc: &str, needle: &str) -> String {
    let bl = k.call("blame", &json!({"doc_path": doc}), human()).unwrap();
    for b in bl["blocks"].as_array().unwrap() {
        let t = k
            .call(
                "read_block",
                &json!({"doc_path": doc, "lineage_id": b["lineage_id"]}),
                human(),
            )
            .unwrap();
        if t["block"]["block"]["text"]
            .as_str()
            .unwrap()
            .contains(needle)
        {
            return b["layer"].as_str().unwrap().to_string();
        }
    }
    panic!("no block with {needle:?} in {doc}")
}

/// Appends `line` to `doc` and runs the watcher's ingest.
fn edit(k: &Kernel, v: &Path, doc: &str, line: &str) {
    let t = std::fs::read_to_string(v.join(doc)).unwrap();
    std::fs::write(v.join(doc), format!("{t}\n{line}\n")).unwrap();
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
}

#[test]
fn serve_writes_the_canonical_policy_explicitly() {
    let (_r, v) = root("recipe");
    serve_once(&v);
    let p = std::fs::read_to_string(v.join(".rootmark/policy.toml")).unwrap();
    assert!(
        p.contains("\n[attestation]\nunattested = \"canonical\"\n"),
        "{p}"
    );
    // the comment says how to opt in
    assert!(p.contains("write unattested = \"unverified\""), "{p}");
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["attestation"]["unattested"], "canonical");
    edit(&k, &v, "notes.md", "An unattested edit in a new vault.");
    assert_eq!(layer_of(&k, "notes.md", "unattested edit"), "canonical");
}

#[test]
fn unverified_is_the_opt_in_and_behaves_as_in_p2_0_2() {
    let (_r, v) = root("optin");
    serve_once(&v);
    let p = v.join(".rootmark/policy.toml");
    let t = std::fs::read_to_string(&p).unwrap().replace(
        rootmarkserver::config::ATTESTATION_DEFAULT,
        "[attestation]\nunattested = \"unverified\"\n",
    );
    std::fs::write(&p, t).unwrap();
    let k = Kernel::open(&v).unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["attestation"]["unattested"], "unverified");
    edit(&k, &v, "notes.md", "An unattested edit, opted in.");
    assert_eq!(layer_of(&k, "notes.md", "unattested edit"), "unverified");
    let ev = std::fs::read_to_string(v.join(".rootmark/events.jsonl")).unwrap();
    assert!(ev.contains("unattested_fs"));
}

#[test]
fn a_policy_without_the_table_keeps_the_kernel_default() {
    // spec §7.1's default and `core`'s are unchanged: no `[attestation]`
    let (_r, v) = root("base");
    std::fs::create_dir_all(v.join(".rootmark")).unwrap();
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        rootmarkserver::config::base_recipe(),
    )
    .unwrap();
    let k = Kernel::open(&v).unwrap();
    k.ensure_index().unwrap();
    let st = k.call("status", &json!({}), human()).unwrap();
    assert_eq!(st["attestation"]["unattested"], "unverified");
}

struct Session {
    id: String,
    key: Vec<u8>,
}

fn session(k: &Kernel) -> Session {
    let s = k.call("attest_session", &json!({}), human()).unwrap();
    let key = s["key"].as_str().unwrap();
    Session {
        id: s["session_id"].as_str().unwrap().to_string(),
        key: (0..key.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&key[i..i + 2], 16).unwrap())
            .collect(),
    }
}

fn attest_bytes(
    k: &Kernel,
    s: &Session,
    doc: &str,
    bytes: &[u8],
    good_mac: bool,
) -> anyhow::Result<Value> {
    let h = rootmarkcore::attest::content_hash(bytes);
    let m = if good_mac {
        mac(&s.key, &mac_message(doc, &h, None))
    } else {
        "0".repeat(64)
    };
    k.call(
        "attest",
        &json!({"doc_path": doc, "content_hash": h, "session_id": s.id, "mac": m}),
        human(),
    )
}

fn lines(p: &Path) -> Vec<Value> {
    std::fs::read_to_string(p)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn the_attest_log_tells_refused_matched_and_unmatched_apart() {
    let (r, v) = root("log");
    serve_once(&v);
    // P2-0.2's behaviour, a short window
    let p = v.join(".rootmark/policy.toml");
    let t = std::fs::read_to_string(&p).unwrap().replace(
        rootmarkserver::config::ATTESTATION_DEFAULT,
        "[attestation]\nunattested = \"unverified\"\nwindow_s = 3.0\n",
    ) + "\n[identity]\nhuman_id = \"ada\"\n";
    std::fs::write(&p, t).unwrap();
    let logp = r.join("attest-diag.jsonl");
    let mut k = Kernel::open(&v).unwrap();
    k.set_attest_log(AttestLog::open(&logp, &v).unwrap());
    // the embedder loads on the first ingest (seconds on a CI runner):
    // loaded before the timed steps, so that only the window decides
    k.embedder().unwrap();
    let s = session(&k);
    let doc = "notes.md";

    // 1. a wrong MAC: refused, with its code and reason
    let e = attest_bytes(&k, &s, doc, b"whatever", false).unwrap_err();
    assert_eq!(code_of(&e), Code::Denied);
    // 2. an attested save: accepted, then observed as attested
    let saved = format!("{NOTES}\nAn attested human line.\n");
    std::fs::write(v.join(doc), &saved).unwrap();
    attest_bytes(&k, &s, doc, saved.as_bytes(), true).unwrap();
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
    // 3. an accepted attestation of other bytes than the ones observed (B)
    let other = format!("{saved}\nWhat the client said it saved.\n");
    attest_bytes(&k, &s, doc, other.as_bytes(), true).unwrap();
    let observed = format!("{saved}\nWhat the file holds instead.\n");
    std::fs::write(v.join(doc), &observed).unwrap();
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();
    // 4. the same bytes attested, observed after the window
    let late = format!("{observed}\nObserved after the window.\n");
    std::fs::write(v.join(doc), &late).unwrap();
    attest_bytes(&k, &s, doc, late.as_bytes(), true).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(3500));
    k.watch_doc(doc, "rootmark-serve-watch").unwrap();

    let l = lines(&logp);
    let attests: Vec<&Value> = l.iter().filter(|x| x["event"] == "attest").collect();
    let observes: Vec<&Value> = l.iter().filter(|x| x["event"] == "observe").collect();
    assert_eq!(attests.len(), 4, "{l:#?}");
    assert_eq!(observes.len(), 3, "{l:#?}");
    assert_eq!(attests[0]["result"], "refused");
    assert_eq!(attests[0]["code"], "denied");
    assert!(
        attests[0]["reason"].as_str().unwrap().contains("MAC"),
        "{}",
        attests[0]
    );
    assert_eq!(attests[1]["result"], "accepted");
    assert_eq!(attests[1]["kind"], "plain");
    assert_eq!(observes[0]["ingest"], "attested");
    assert_eq!(observes[0]["attestation_id"], attests[1]["attestation_id"]);
    assert!(observes[0]["seq_distance"].as_i64().unwrap() >= 0);
    assert_eq!(attests[2]["matched"], false, "file never held them");
    assert_eq!(observes[1]["ingest"], "unverified");
    let why: Vec<&str> = observes[1]["accepted_before"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["attestation_id"] == attests[2]["attestation_id"])
        .map(|a| a["why"].as_str().unwrap())
        .collect();
    assert_eq!(why, ["other_bytes"], "{}", observes[1]);
    assert_eq!(observes[2]["ingest"], "unverified");
    let why: Vec<&str> = observes[2]["accepted_before"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["attestation_id"] == attests[3]["attestation_id"])
        .map(|a| a["why"].as_str().unwrap())
        .collect();
    assert_eq!(why, ["expired"], "{}", observes[2]);
    // nothing of the log is in the vault
    assert!(!v.join("attest-diag.jsonl").exists());
}

#[test]
fn the_attest_log_must_lie_outside_the_vault() {
    let (_r, v) = root("inside");
    let e = AttestLog::open(&v.join(".rootmark/diag.jsonl"), &v)
        .err()
        .unwrap();
    assert!(format!("{e:#}").contains("inside the vault"), "{e:#}");
    // through `rootmark serve`: refused before anything is written
    let st = Command::new(rootmark())
        .args([
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--mcp",
            "stdio",
            "--attest-log",
            v.join("diag.jsonl").to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!st.status.success());
    assert!(!v.join("diag.jsonl").exists());
    assert!(!v.join(".rootmark/policy.toml").exists());
}

#[cfg(unix)]
#[test]
fn a_log_name_linking_into_the_vault_is_refused() {
    let (r, v) = root("link");
    std::os::unix::fs::symlink(v.join("notes.md"), r.join("diag.jsonl")).unwrap();
    let e = AttestLog::open(&r.join("diag.jsonl"), &v).err().unwrap();
    assert!(format!("{e:#}").contains("inside the vault"), "{e:#}");
    assert_eq!(std::fs::read_to_string(v.join("notes.md")).unwrap(), NOTES);
}
