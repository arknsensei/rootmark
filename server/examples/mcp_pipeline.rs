//! Pipelining MCP stdio driver (P1-D.1, GOAL D1-5/D1-6; K-D1.4). Test
//! tooling, not shipped: unlike `mcp_client` (lock-step), it writes **every**
//! message of the script to the server at once, without waiting for any
//! response, then reads responses (which may come in any order) until every
//! request (a message with an `id`) is answered.
//!
//!   cargo run --example mcp_pipeline -- <script.jsonl> <transcript.jsonl> [<elapsed.json>] -- <server> [args…]
//!
//! The transcript holds one line per request, in script order:
//! `{"request": …, "response": …}`. `<elapsed.json>` (optional) receives
//! `{"requests": n, "elapsed_ms": t, "out_of_order": m}` — wall time from the
//! first byte written to the last response, and how many responses arrived
//! before a response to an earlier request (timing is information, not part
//! of the transcript). Exit 1 if the server does not answer within 300 s,
//! writes a line that is not JSON, answers an unknown id or exits non-zero
//! after stdin is closed.

use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn fail(msg: String) -> ! {
    eprintln!("mcp_pipeline: error: {msg}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: mcp_pipeline <script> <transcript> [<elapsed.json>] -- <server…>";
    let sep = args
        .iter()
        .position(|a| a == "--")
        .unwrap_or_else(|| fail(usage.into()));
    if !(2..=3).contains(&sep) || args.len() < sep + 2 {
        fail(usage.into());
    }
    let script = std::fs::read_to_string(&args[0])
        .unwrap_or_else(|e| fail(format!("read {}: {e}", args[0])));
    let elapsed_out = (sep == 3).then(|| args[2].clone());
    let msgs: Vec<Value> = script
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| {
            serde_json::from_str(l).unwrap_or_else(|e| fail(format!("script line {}: {e}", i + 1)))
        })
        .collect();
    let ids: Vec<Value> = msgs.iter().filter_map(|m| m.get("id").cloned()).collect();
    let mut child = Command::new(&args[sep + 1])
        .args(&args[sep + 2..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| fail(format!("start {}: {e}", args[sep + 1])));
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut buf = Vec::new();
    for m in &msgs {
        serde_json::to_writer(&mut buf, m).expect("JSON");
        buf.push(b'\n');
    }
    let start = Instant::now();
    stdin
        .write_all(&buf)
        .and_then(|_| stdin.flush())
        .unwrap_or_else(|e| fail(format!("write to the server: {e}")));
    let pos: HashMap<String, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.to_string(), i))
        .collect();
    let mut answers: Vec<Option<Value>> = vec![None; ids.len()];
    let (mut got, mut out_of_order, mut highest) = (0usize, 0usize, None::<usize>);
    while got < ids.len() {
        let line = rx
            .recv_timeout(Duration::from_secs(300))
            .unwrap_or_else(|_| fail(format!("{} of {} responses after 300 s", got, ids.len())));
        let r: Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| fail(format!("server wrote a non-JSON line ({e}): {line}")));
        let Some(id) = r.get("id") else {
            continue; // a notification
        };
        let i = *pos
            .get(&id.to_string())
            .unwrap_or_else(|| fail(format!("response to an unknown id {id}")));
        if answers[i].is_some() {
            fail(format!("two responses to id {id}"));
        }
        if highest.is_some_and(|h| i < h) {
            out_of_order += 1;
        }
        highest = Some(highest.map_or(i, |h| h.max(i)));
        answers[i] = Some(r);
        got += 1;
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    drop(stdin);
    let st = child
        .wait()
        .unwrap_or_else(|e| fail(format!("wait for the server: {e}")));
    if !st.success() {
        fail(format!("server exited with {st}"));
    }
    let mut out = std::fs::File::create(&args[1])
        .unwrap_or_else(|e| fail(format!("create {}: {e}", args[1])));
    let requests = msgs.iter().filter(|m| m.get("id").is_some());
    for (req, resp) in requests.zip(answers) {
        let line = serde_json::json!({"request": req, "response": resp});
        writeln!(out, "{line}").unwrap_or_else(|e| fail(format!("write transcript: {e}")));
    }
    if let Some(p) = elapsed_out {
        let t = serde_json::json!({"requests": ids.len(), "elapsed_ms": elapsed_ms, "out_of_order": out_of_order});
        std::fs::write(&p, format!("{t}\n")).unwrap_or_else(|e| fail(format!("write {p}: {e}")));
    }
}
