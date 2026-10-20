//! P2-0/Z0 (P1-E.1 verification, recommendation 3): `rootmark serve` over an
//! index that is not `ok` stops cleanly. The shutdown compaction is skipped
//! with a note on stderr and the process exits 0 (CLIENT_CONTRACT §3), where
//! P1-E.1 wrote `compact (shutdown)` as an error and exited 1:
//! - `stale` (format `mdkernel-index/2`, `[index] auto_rebuild = false`);
//! - `damaged` (manifest not JSON);
//! - `missing` (index removed while the service runs).
//!
//! An `ok` index is still compacted at shutdown (control case).

use rootmarkserver::tools::Kernel;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn rootmark() -> &'static str {
    env!("CARGO_BIN_EXE_rootmark")
}

const SAFETY: &str = "# Safety\n\nIntro written by a human.\n\n## Fire\n\nKeep a class D extinguisher next to the roaster.\n";

fn vault(name: &str, extra: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rootmark-p2z0-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("safety.md"), SAFETY).unwrap();
    let st = Command::new(rootmark())
        .args(["ingest-dir", v.to_str().unwrap(), v.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(st.status.success());
    std::fs::write(
        v.join(".rootmark/policy.toml"),
        format!(
            "{}\n[identity]\nhuman_id = \"ada\"\n",
            rootmarkserver::config::base_recipe()
                .replace("[index]\n", &format!("[index]\n{extra}"))
        ),
    )
    .unwrap();
    Kernel::open(&v).unwrap().ensure_index().unwrap();
    v
}

fn manifest(v: &Path) -> PathBuf {
    v.join(".rootmark/index/manifest.json")
}

/// Starts `rootmark serve --http 127.0.0.1:0`, runs `during` once it
/// listens, sends SIGTERM and returns (exit code, every stderr line).
fn serve_and_stop(v: &Path, during: impl FnOnce()) -> (i32, Vec<String>) {
    let mut child = Command::new(rootmark())
        .args([
            "serve",
            "--vault",
            v.to_str().unwrap(),
            "--http",
            "127.0.0.1:0",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let err = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        for l in BufReader::new(err).lines().map_while(Result::ok) {
            if tx.send(l).is_err() {
                break;
            }
        }
    });
    let mut lines = Vec::new();
    let start = Instant::now();
    loop {
        let l = rx
            .recv_timeout(Duration::from_secs(120).saturating_sub(start.elapsed()))
            .expect("serve did not start");
        let up = l.starts_with("rootmark serve: http listening on");
        lines.push(l);
        if up {
            break;
        }
    }
    during();
    let st = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(st.success());
    let code = child.wait().unwrap().code().unwrap_or(-1);
    lines.extend(rx.iter());
    (code, lines)
}

#[test]
fn shutdown_skips_compaction_of_an_index_that_is_not_ok() {
    let mut records = Vec::new();
    type Case = (&'static str, &'static str, fn(&Path));
    let cases: [Case; 3] = [
        ("stale", "auto_rebuild = false\n", |v| {
            let p = manifest(v);
            let mut m: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            m["format"] = json!("mdkernel-index/2");
            std::fs::write(&p, serde_json::to_string(&m).unwrap()).unwrap();
        }),
        ("damaged", "", |v| {
            std::fs::write(manifest(v), "{ not json").unwrap()
        }),
        ("missing", "", |_| {}),
    ];
    for (state, extra, prepare) in cases {
        let v = vault(state, extra);
        prepare(&v);
        let ix = v.join(".rootmark/index");
        let (code, lines) = serve_and_stop(&v, || {
            if state == "missing" {
                std::fs::remove_dir_all(&ix).unwrap();
            }
        });
        let note = format!("rootmark serve: compact (shutdown): skipped, index {state}");
        let noted = lines.iter().any(|l| l.starts_with(&note));
        let errors: Vec<&String> = lines.iter().filter(|l| l.contains("error")).collect();
        records.push(json!({"state": state, "exit": code, "noted": noted, "errors": errors}));
        assert_eq!(code, 0, "{state}: {lines:#?}");
        assert!(noted, "{state}: {lines:#?}");
        assert!(errors.is_empty(), "{state}: {lines:#?}");
        assert!(
            lines.iter().any(|l| l == "rootmark serve: stopped"),
            "{state}"
        );
    }
    eprintln!("{}", serde_json::to_string_pretty(&records).unwrap());
}

#[test]
fn shutdown_still_compacts_an_ok_index() {
    let v = vault("ok", "");
    let (code, lines) = serve_and_stop(&v, || {});
    assert_eq!(code, 0, "{lines:#?}");
    assert!(
        lines.iter().any(
            |l| l.starts_with("rootmark serve: compact (shutdown): ") && !l.contains("skipped")
        ),
        "{lines:#?}"
    );
}
