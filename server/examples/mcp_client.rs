//! Lock-step MCP stdio driver for the end-to-end scripts (`scripts/mcp_e2e.sh`,
//! `scripts/scene_agent.sh`). Test tooling, not shipped: it starts the
//! server command, sends the JSON-RPC messages of a script one by one and,
//! for every request (a message with an `id`), waits for the response with
//! that `id` before sending the next, so stateful calls (`propose` then
//! `search`) never race. Notifications are sent without waiting.
//!
//!   cargo run --example mcp_client -- <script.jsonl> <transcript.jsonl> -- <server> [args…]
//!
//! The transcript holds one line per request: `{"request": …, "response": …}`.
//! Exit 1 if the server does not answer within 120 s, writes a line that is
//! not JSON, answers out of order, or exits non-zero after stdin is closed.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn fail(msg: String) -> ! {
    eprintln!("mcp_client: error: {msg}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sep = args
        .iter()
        .position(|a| a == "--")
        .unwrap_or_else(|| fail("usage: mcp_client <script> <transcript> -- <server…>".into()));
    if sep != 2 || args.len() < 4 {
        fail("usage: mcp_client <script> <transcript> -- <server…>".into());
    }
    let script = std::fs::read_to_string(&args[0])
        .unwrap_or_else(|e| fail(format!("read {}: {e}", args[0])));
    let mut out = std::fs::File::create(&args[1])
        .unwrap_or_else(|e| fail(format!("create {}: {e}", args[1])));
    let mut child = Command::new(&args[3])
        .args(&args[4..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| fail(format!("start {}: {e}", args[3])));
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
    for (i, line) in script.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let req: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| fail(format!("script line {}: {e}", i + 1)));
        writeln!(stdin, "{}", serde_json::to_string(&req).unwrap())
            .and_then(|_| stdin.flush())
            .unwrap_or_else(|e| fail(format!("write to server: {e}")));
        let Some(id) = req.get("id").cloned() else {
            continue; // notification
        };
        let resp = loop {
            let l = rx
                .recv_timeout(Duration::from_secs(120))
                .unwrap_or_else(|_| fail(format!("no response to request id {id}")));
            let v: serde_json::Value = serde_json::from_str(&l)
                .unwrap_or_else(|e| fail(format!("server wrote a non-JSON line ({e}): {l}")));
            if v.get("id").is_none() {
                continue; // a server notification (e.g. logging)
            }
            if v["id"] != id {
                fail(format!("response id {} while waiting for {id}", v["id"]));
            }
            break v;
        };
        let rec = serde_json::json!({"request": req, "response": resp});
        writeln!(out, "{}", serde_json::to_string(&rec).unwrap())
            .unwrap_or_else(|e| fail(format!("write transcript: {e}")));
    }
    drop(stdin);
    let st = child
        .wait()
        .unwrap_or_else(|e| fail(format!("wait for server: {e}")));
    if !st.success() {
        fail(format!("server exited with {st}"));
    }
}
