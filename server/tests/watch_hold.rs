//! P2-Q.2/R2 (PQ1-4; `docs/rounds/P2-Q.2/progress/TESTS.md` H1–H5,
//! DESIGN §4): a removal's hold starts after its batch's ingest, so the old
//! path of a rename is not taken before the new path's late-attestation
//! window is over — even when the new path's ingest is late (load). The
//! load is stood in for by the test setting
//! `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` (only this file's servers get it;
//! the product's default is unchanged). Runs on macOS and Linux (`make
//! test`, CI). Needs the model files (`make models`).

mod watchkit;

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use watchkit::{events, filled_snapshots, of_kind, rootmark, settle, tmp, wait_for, write};

const LIMIT: Duration = Duration::from_secs(90);
const SETTLE: Duration = Duration::from_millis(800);
const DELAY_MS: u64 = 3000;

/// `rootmark serve --http 127.0.0.1:<port> --watch` on `vault` under
/// `unattested = "unverified"` and `window_s`, each ingest of the watcher
/// delayed by `delay_ms` (0: the setting is not given).
struct Held {
    vault: PathBuf,
    port: u16,
    token: String,
    child: Child,
    rest: Option<std::thread::JoinHandle<String>>,
    head: String,
}

fn serve(vault: &Path, window_s: f64, delay_ms: u64) -> Held {
    std::fs::create_dir_all(vault.join(".rootmark")).unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let recipe = rootmarkserver::config::base_recipe().replacen(
        "[server]\n",
        &format!("[server]\nhttp_port = {port}\n"),
        1,
    );
    std::fs::write(
        vault.join(".rootmark/policy.toml"),
        recipe
            + &format!(
                "\n[identity]\nhuman_id = \"tester\"\n\n[attestation]\nunattested = \"unverified\"\nwindow_s = {window_s:.1}\n"
            ),
    )
    .unwrap();
    let mut cmd = Command::new(rootmark());
    cmd.args([
        "serve",
        "--vault",
        vault.to_str().unwrap(),
        "--http",
        &format!("127.0.0.1:{port}"),
        "--watch",
    ])
    .stderr(Stdio::piped())
    .env_remove("ROOTMARK_WATCH_TEST_INGEST_DELAY_MS");
    if delay_ms > 0 {
        cmd.env("ROOTMARK_WATCH_TEST_INGEST_DELAY_MS", delay_ms.to_string());
    }
    let mut child = cmd.spawn().unwrap();
    let mut err = BufReader::new(child.stderr.take().unwrap());
    let mut head = String::new();
    loop {
        let mut l = String::new();
        assert!(err.read_line(&mut l).unwrap() > 0, "server exited: {head}");
        head.push_str(&l);
        if l.contains("rootmark serve: watching") {
            break;
        }
    }
    if delay_ms > 0 {
        assert!(
            head.contains("test setting ROOTMARK_WATCH_TEST_INGEST_DELAY_MS"),
            "the delay is not announced: {head}"
        );
    }
    let rest = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    let token = std::fs::read_to_string(vault.join(".rootmark/server.token"))
        .unwrap()
        .trim()
        .to_string();
    Held {
        vault: vault.to_path_buf(),
        port,
        token,
        child,
        rest: Some(rest),
        head,
    }
}

impl Held {
    fn stop(mut self) -> (bool, String) {
        let st = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(st.success());
        let code = self.child.wait().unwrap();
        let tail = self.rest.take().unwrap().join().unwrap();
        (code.success(), format!("{}{}", self.head, tail))
    }

    fn blame(&self, doc: &str) -> Vec<Value> {
        let body = json!({"doc_path": doc}).to_string();
        let mut s = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            s,
            "POST /v1/blame HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\n\
             X-Consumer: p2q2-test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            self.token,
            body.len()
        )
        .unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        let text = String::from_utf8(resp).unwrap();
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        let st: u16 = head.split(' ').nth(1).unwrap().parse().unwrap();
        let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
        if st == 404 && v["error"]["code"] == "not_found" {
            return Vec::new();
        }
        assert_eq!(st, 200, "{v}");
        v["blocks"].as_array().cloned().unwrap_or_default()
    }

    fn layers(&self, doc: &str) -> Vec<String> {
        self.blame(doc)
            .iter()
            .map(|b| b["layer"].as_str().unwrap().to_string())
            .collect()
    }

    fn lineages(&self, doc: &str) -> Vec<String> {
        self.blame(doc)
            .iter()
            .map(|b| b["lineage_id"].as_str().unwrap().to_string())
            .collect()
    }

    fn attest(&self, doc: &str, renamed_from: Option<&str>) -> (bool, String) {
        let mut c = Command::new(rootmark());
        c.args(["attest", self.vault.to_str().unwrap(), doc]);
        if let Some(f) = renamed_from {
            c.args(["--renamed-from", f]);
        }
        let o = c.output().unwrap();
        (
            o.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            ),
        )
    }

    fn events(&self) -> Vec<Value> {
        events(&self.vault)
    }
}

fn note(word: &str) -> String {
    format!(
        "# {word}\n\nFirst paragraph about {word} beans.\n\nSecond paragraph about {word} grinders.\n"
    )
}

fn all(layers: &[String], layer: &str) -> bool {
    layers.len() == 3 && layers.iter().all(|l| l == layer)
}

/// A human note, `canonical` by a late attestation (TESTS.md R2 eki: an
/// attestation recorded more than `W` before the delayed observation would
/// match nothing); its lineages.
fn human(sv: &Held, doc: &str, word: &str) -> Vec<String> {
    write(&sv.vault.join(doc), &note(word));
    wait_for(LIMIT, &format!("{doc} ingested"), || {
        sv.blame(doc).len() == 3
    });
    let (ok, out) = sv.attest(doc, None);
    assert!(ok, "{doc}: late attestation refused: {out}");
    wait_for(LIMIT, &format!("{doc} canonical"), || {
        all(&sv.layers(doc), "canonical")
    });
    sv.lineages(doc)
}

fn deleted(evs: &[Value], doc: &str) -> Vec<String> {
    of_kind(evs, "BlockDeleted", doc)
        .iter()
        .flat_map(|e| e["lineage_ids"].as_array().unwrap().iter())
        .map(|l| l.as_str().unwrap().to_string())
        .collect()
}

fn wait_deleted(sv: &Held, doc: &str, lin: &[String]) {
    wait_for(LIMIT, &format!("{doc}'s removal ingested"), || {
        let d = deleted(&sv.events(), doc);
        lin.iter().all(|l| d.contains(l))
    });
}

/// Each lineage deleted exactly once, by `unattested_fs`.
fn assert_deleted_once(evs: &[Value], doc: &str, lin: &[String]) {
    let mut d = deleted(evs, doc);
    d.sort();
    let mut want = lin.to_vec();
    want.sort();
    assert_eq!(d, want, "{doc}: BlockDeleted once per lineage");
    for e in of_kind(evs, "BlockDeleted", doc) {
        assert_eq!(e["meta"]["actor"]["kind"], "unattested_fs", "{e}");
    }
}

fn pos(evs: &[Value], f: impl Fn(&Value) -> bool) -> usize {
    evs.iter().position(f).unwrap()
}

/// One rename `old → new` attested late (after the watcher took `new`
/// unattested): the attestation is accepted, `new` is raised, `old` is
/// removed once, after the raise.
fn late_rename(sv: &Held, old: &str, new: &str, word: &str) {
    let lin = human(sv, old, word);
    std::fs::rename(sv.vault.join(old), sv.vault.join(new)).unwrap();
    wait_for(LIMIT, &format!("{new} ingested"), || {
        sv.blame(new).len() == 3
    });
    let (ok, out) = sv.attest(new, Some(old));
    assert!(ok, "{old} → {new}: late rename attestation refused: {out}");
    wait_for(LIMIT, &format!("{new} canonical"), || {
        all(&sv.layers(new), "canonical")
    });
    wait_deleted(sv, old, &lin);
    let evs = sv.events();
    assert_deleted_once(&evs, old, &lin);
    assert_eq!(
        filled_snapshots(&evs, new),
        1,
        "{new} ingested more than once"
    );
    let raise = pos(&evs, |e| {
        e["kind"] == "LayerChanged"
            && e["doc_path"] == new
            && e["meta"]["cause"] == "late_attestation"
    });
    let del = pos(&evs, |e| {
        e["kind"] == "BlockDeleted" && e["doc_path"] == old
    });
    assert!(raise < del, "{old} removed before the late attestation");
}

fn vault(name: &str) -> (PathBuf, PathBuf) {
    let root = tmp(&format!("p2q2-{name}"));
    let v = root.join("vault");
    std::fs::create_dir_all(&v).unwrap();
    (root, v)
}

fn done(sv: Held, root: &Path) {
    let (ok, log) = sv.stop();
    assert!(ok, "{log}");
    std::fs::remove_dir_all(root).unwrap();
}

/// H1: 20 renames, each new path's ingest delayed past the hold, each
/// attested late.
#[test]
fn h1_late_attested_rename_under_load_20_times() {
    let (root, v) = vault("h1");
    let sv = serve(&v, 1.0, DELAY_MS);
    for i in 0..20 {
        late_rename(
            &sv,
            &format!("h{i:02}.md"),
            &format!("h{i:02}-r.md"),
            &format!("hold{i:02}word"),
        );
    }
    settle(&v, SETTLE, LIMIT);
    done(sv, &root);
}

/// H2: an on-time attested rename under the delay (window 5 s > delay):
/// accepted, the old path removed once, after the new path's ingest.
#[test]
fn h2_on_time_attested_rename_under_load() {
    let (root, v) = vault("h2");
    let sv = serve(&v, 5.0, DELAY_MS);
    let lin = human(&sv, "o.md", "ontimeword");
    std::fs::rename(v.join("o.md"), v.join("o-r.md")).unwrap();
    let (ok, out) = sv.attest("o-r.md", Some("o.md"));
    assert!(ok, "on-time rename attestation refused: {out}");
    wait_for(LIMIT, "o-r.md canonical", || {
        all(&sv.layers("o-r.md"), "canonical")
    });
    wait_deleted(&sv, "o.md", &lin);
    settle(&v, SETTLE, LIMIT);
    let evs = sv.events();
    assert_deleted_once(&evs, "o.md", &lin);
    assert_eq!(filled_snapshots(&evs, "o-r.md"), 1);
    let new = pos(&evs, |e| e["doc_path"] == "o-r.md");
    let del = pos(&evs, |e| {
        e["kind"] == "BlockDeleted" && e["doc_path"] == "o.md"
    });
    assert!(new < del, "o.md removed before o-r.md's ingest");
    done(sv, &root);
}

/// H3: an unattested deletion under the delay is still taken, once.
#[test]
fn h3_deletion_under_load_is_taken() {
    let (root, v) = vault("h3");
    let sv = serve(&v, 1.0, DELAY_MS);
    let lin = human(&sv, "d.md", "deletedword");
    std::fs::remove_file(v.join("d.md")).unwrap();
    wait_deleted(&sv, "d.md", &lin);
    settle(&v, SETTLE, LIMIT);
    assert_deleted_once(&sv.events(), "d.md", &lin);
    assert!(sv.blame("d.md").is_empty());
    done(sv, &root);
}

/// H4: a stream of writes to another note does not hold a removal back
/// for ever: taken within 2 × hold + 3 s of the deletion, while the stream
/// (15 s) still runs (DESIGN §4.3 (2); TESTS.md R2 eki).
#[test]
fn h4_write_stream_does_not_starve_a_removal() {
    let (root, v) = vault("h4");
    let sv = serve(&v, 1.0, 0);
    let lin = human(&sv, "gone.md", "goneword");
    write(&v.join("busy.md"), &note("busyword"));
    wait_for(LIMIT, "busy.md ingested", || sv.blame("busy.md").len() == 3);
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let busy = v.join("busy.md");
    let s2 = stop.clone();
    let writer = std::thread::spawn(move || {
        let mut i = 0u32;
        while !s2.load(std::sync::atomic::Ordering::Relaxed) {
            write(
                &busy,
                &format!("# busy\n\nLine {i} about busy beans.\n\nSecond busy paragraph.\n"),
            );
            i += 1;
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    std::thread::sleep(Duration::from_millis(500));
    let t0 = Instant::now();
    std::fs::remove_file(v.join("gone.md")).unwrap();
    let bound = Duration::from_millis(2 * 1500 + 3000);
    wait_for(bound, "gone.md removed while busy.md is written", || {
        let d = deleted(&sv.events(), "gone.md");
        lin.iter().all(|l| d.contains(l))
    });
    let took = t0.elapsed();
    // the stream still runs when the removal is taken (it lasts 15 s)
    if let Some(rest) = Duration::from_secs(15).checked_sub(t0.elapsed()) {
        std::thread::sleep(rest);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    writer.join().unwrap();
    assert!(took < bound, "took {took:?}");
    settle(&v, SETTLE, LIMIT);
    assert_deleted_once(&sv.events(), "gone.md", &lin);
    done(sv, &root);
}

/// H5: the same late-attested rename under the delay in two fresh vaults
/// gives the same event log.
#[test]
fn h5_same_steps_same_log() {
    let mut logs = Vec::new();
    for n in ["h5a", "h5b"] {
        let (root, v) = vault(n);
        let sv = serve(&v, 1.0, DELAY_MS);
        late_rename(&sv, "same.md", "same-r.md", "sameword");
        settle(&v, SETTLE, LIMIT);
        logs.push(std::fs::read(v.join(".rootmark/events.jsonl")).unwrap());
        done(sv, &root);
    }
    assert!(logs[0] == logs[1], "the two event logs differ");
}
