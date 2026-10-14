//! P1-B/B6: end-to-end tests of every governance command of the `mdk`
//! binary, `blame --at`, and the six B-6 policy-bound cases (each an error,
//! none a silent fallback: the word "default" never appears in the output).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdk-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// (exit ok, stdout JSON or Null, stderr)
fn mdk(args: &[&str]) -> (bool, Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mdk"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let v = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    (out.status.success(), v, stderr)
}

fn ok(args: &[&str]) -> Value {
    let (s, v, e) = mdk(args);
    assert!(s, "mdk {args:?} failed: {e}");
    v
}

/// fails, with `pat` in stderr and no fallback wording anywhere
fn fails(args: &[&str], pat: &str) -> String {
    let (s, v, e) = mdk(args);
    assert!(!s, "mdk {args:?} succeeded: {v}");
    assert!(e.contains(pat), "mdk {args:?}: no {pat:?} in {e}");
    assert!(
        !e.to_lowercase().contains("default") && v.is_null(),
        "mdk {args:?}: fallback wording or output: {e} {v}"
    );
    e
}

fn write_policy(vault: &Path, text: &str) {
    fs::create_dir_all(vault.join(".mdkernel")).unwrap();
    fs::write(vault.join(".mdkernel/policy.toml"), text).unwrap();
}

const GOOD: &str = "promotion = \"human_only\"\n\n[version_trigger]\nidle_ms = 1000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 24\n";

#[test]
fn governance_commands_end_to_end() {
    let root = tmp("e2e");
    let vault = root.join("vault");
    fs::create_dir_all(&vault).unwrap();
    let vs = vault.to_str().unwrap();
    let draft = root.join("draft.md");
    fs::write(
        &draft,
        "# Plan\n\nDescale the machine every two weeks.\n\nBuy a new burr set.\n",
    )
    .unwrap();
    let df = draft.to_str().unwrap();

    // propose: the file lands in the vault, its blocks are proposals
    let p = ok(&[
        "propose",
        vs,
        "notes/plan.md",
        df,
        "--source",
        "claude-code",
    ]);
    assert_eq!(
        fs::read_to_string(vault.join("notes/plan.md")).unwrap(),
        fs::read_to_string(&draft).unwrap()
    );
    let ls: Vec<String> = p["layers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            assert_eq!(b["layer"], "proposal");
            b["lineage_id"].as_str().unwrap().to_string()
        })
        .collect();
    assert_eq!(ls.len(), 3);
    fails(
        &["propose", vs, "../x.md", df, "--source", "a"],
        "vault-relative",
    );

    // without policy.toml the built-in recipe denies every promotion
    let pol = ok(&["policy", "show", vs]);
    assert_eq!(pol["source"], "builtin");
    assert_eq!(pol["promotion"], "deny");
    fails(
        &[
            "promote",
            vs,
            &ls[1],
            "--actor",
            "human:alice",
            "--reason",
            "ok",
        ],
        "denied by policy deny",
    );
    write_policy(&vault, GOOD);
    assert_eq!(ok(&["policy", "show", vs])["promotion"], "human_only");
    fails(
        &[
            "promote",
            vs,
            &ls[1],
            "--actor",
            "agent:claude",
            "--reason",
            "ok",
        ],
        "is not human",
    );

    // checkpoint before the promotion
    let cp = ok(&["checkpoint", "create", vs, "--name", "before"]);
    let cid = cp["checkpoint_id"].as_str().unwrap().to_string();
    assert_eq!(ok(&["checkpoint", "create", vs, "--name", "before"]), cp);
    assert_eq!(ok(&["checkpoint", "show", vs, "before"]), cp);
    assert_eq!(ok(&["checkpoint", "show", vs, &cid]), cp);
    assert_eq!(ok(&["checkpoint", "list", vs])["checkpoints"][0], cp);
    fails(&["checkpoint", "show", vs, "nope"], "no checkpoint");

    // promote, demote, prune
    let pr = ok(&[
        "promote",
        vs,
        &ls[1],
        "--actor",
        "human:alice",
        "--reason",
        "reviewed",
    ]);
    assert_eq!(pr["event"]["kind"], "Promoted");
    assert_eq!(pr["layer"], "canonical");
    let de = ok(&[
        "demote",
        vs,
        &ls[1],
        "--actor",
        "human:alice",
        "--reason",
        "undo",
    ]);
    assert_eq!(de["layer"], "proposal");
    ok(&[
        "promote",
        vs,
        &ls[1],
        "--actor",
        "human:alice",
        "--reason",
        "again",
    ]);
    let pn = ok(&[
        "prune",
        vs,
        &ls[2],
        "--actor",
        "human:alice",
        "--reason",
        "stale",
    ]);
    assert_eq!(pn["inactive"], true);
    assert_eq!(pn["event"]["meta"]["prune"], true);

    // history
    let h = ok(&["history", vs, &ls[1]]);
    let kinds: Vec<&str> = h["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec!["BlockCreated", "Promoted", "Demoted", "Promoted"]
    );
    assert_eq!(h["layer"], "canonical");

    // blame now and as of the checkpoint (later events invisible)
    let now = ok(&["blame", vs, "notes/plan.md"]);
    let then = ok(&["blame", vs, "notes/plan.md", "--at", "before"]);
    let row = |b: &Value, i: usize| b["blocks"][i].clone();
    assert_eq!(row(&now, 1)["layer"], "canonical");
    assert_eq!(row(&then, 1)["layer"], "proposal");
    assert_eq!(row(&now, 2)["inactive"], true);
    assert_eq!(row(&then, 2)["inactive"], false);
    let before_seq = cp["event_seq"].as_u64().unwrap();
    for i in 0..3 {
        let r = row(&then, i);
        assert!(r["seq_last"].as_u64().unwrap() < before_seq);
        assert_eq!(r["author_kind"], "Agent");
        assert_eq!(r["source"], "claude-code");
        for k in [
            "lineage_id",
            "version_id",
            "layer",
            "seq_created",
            "seq_last",
        ] {
            assert!(!r[k].is_null(), "{k}");
        }
    }
    assert!(row(&now, 1)["seq_last"].as_u64().unwrap() > before_seq);

    // read at the checkpoint writes a trace; trace list
    let rd = ok(&[
        "read",
        vs,
        "--at",
        "before",
        "--consumer",
        "claude-code",
        &ls[1],
    ]);
    assert_eq!(rd["blocks"][0]["block"]["layer"], "proposal");
    assert_eq!(rd["trace"]["at_seq"], 0);
    ok(&[
        "read",
        vs,
        "--at",
        &cid,
        "--consumer",
        "claude-code",
        &ls[1],
        &ls[0],
    ]);
    let tl = ok(&["trace", "list", vs]);
    assert_eq!(tl["traces"].as_array().unwrap().len(), 2);
    assert_eq!(
        ok(&["trace", "list", vs, "--since", "1"])["traces"][0]["at_seq"],
        1
    );

    // journal: append (distance trigger), tick (idle trigger), list, prune
    fs::write(vault.join("log.md"), "# Log\n\nFirst save.\n").unwrap();
    let j = ok(&[
        "journal", "append", vs, "log.md", "--now-ms", "1000", "--author", "human", "--source",
        "editor",
    ]);
    assert_eq!(j["record"]["seq"], 0);
    assert!(
        !j["versioned"].is_null(),
        "first save: distance 1 from nothing"
    );
    fs::write(vault.join("log.md"), "# Log\n\nFirst save!\n").unwrap();
    let j = ok(&[
        "journal", "append", vs, "log.md", "--now-ms", "1500", "--author", "human", "--source",
        "editor",
    ]);
    assert!(j["versioned"].is_null(), "small change, not idle");
    let t = ok(&[
        "journal", "tick", vs, "log.md", "--now-ms", "1999", "--author", "human", "--source",
        "editor",
    ]);
    assert!(t["versioned"].is_null());
    let t = ok(&[
        "journal", "tick", vs, "log.md", "--now-ms", "2500", "--author", "human", "--source",
        "editor",
    ]);
    assert!(!t["versioned"].is_null(), "idle 1000 ms reached");
    assert_eq!(
        ok(&["journal", "list", vs, "log.md"])["records"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let pr = ok(&[
        "journal",
        "prune",
        vs,
        "--now-ms",
        &(1000 + 25 * 3_600_000u64).to_string(),
    ]);
    assert_eq!(pr["records_removed"], 1);
    fs::remove_dir_all(&root).unwrap();
}

/// GOAL B-6: six policy-bound cases, each an error, none a silent fallback.
#[test]
fn b6_six_bound_cases() {
    let root = tmp("b6");
    let vault = root.join("vault");
    fs::create_dir_all(&vault).unwrap();
    let vs = vault.to_str().unwrap();
    let draft = root.join("d.md");
    fs::write(&draft, "# D\n\nAgent text.\n").unwrap();
    let p = ok(&[
        "propose",
        vs,
        "d.md",
        draft.to_str().unwrap(),
        "--source",
        "agent",
    ]);
    let l = p["layers"][1]["lineage_id"].as_str().unwrap().to_string();
    let promote = [
        "promote",
        vs,
        &l,
        "--actor",
        "human:alice",
        "--reason",
        "ok",
    ];

    let bad = [
        (
            "retention < 24h",
            GOOD.replace("retention_hours = 24", "retention_hours = 23"),
            "kernel minimum is 24",
        ),
        (
            "idle_ms < 0",
            GOOD.replace("idle_ms = 1000", "idle_ms = -1"),
            "idle_ms = -1",
        ),
        (
            "min_sim_change > 1",
            GOOD.replace("min_sim_change = 0.2", "min_sim_change = 1.5"),
            "must lie in [0, 1]",
        ),
        (
            "min_sim_change < 0",
            GOOD.replace("min_sim_change = 0.2", "min_sim_change = -0.1"),
            "must lie in [0, 1]",
        ),
        (
            "unknown promotion",
            GOOD.replace("human_only", "implicit"),
            "unknown promotion policy",
        ),
        (
            "unknown field",
            format!("{GOOD}extra = 1\n"),
            "unknown field",
        ),
        (
            "missing field",
            GOOD.replace("retention_hours = 24\n", ""),
            "missing field",
        ),
    ];
    for (name, text, pat) in bad {
        write_policy(&vault, &text);
        for cmd in [
            &["policy", "show", vs][..],
            &promote[..],
            &["journal", "prune", vs, "--now-ms", "0"][..],
        ] {
            let e = fails(cmd, pat);
            assert!(e.contains("rejected"), "{name}: {e}");
        }
    }

    // hook'suz terfi: no recipe → built-in deny; an explicit deny recipe
    fs::remove_file(vault.join(".mdkernel/policy.toml")).unwrap();
    fails(&promote, "denied by policy deny");
    write_policy(&vault, &GOOD.replace("human_only", "deny"));
    fails(&promote, "denied by policy deny");

    // actor'sız Promoted: no --actor, an empty one, one without a kind
    write_policy(&vault, GOOD);
    let (s, _, e) = mdk(&["promote", vs, &l, "--reason", "ok"]);
    assert!(!s && e.contains("--actor"), "{e}");
    fails(
        &["promote", vs, &l, "--actor", "human:", "--reason", "ok"],
        "actor id is empty",
    );
    fails(
        &["promote", vs, &l, "--actor", "alice", "--reason", "ok"],
        "expected <human|agent|policy>:<id>",
    );
    fails(
        &["promote", vs, &l, "--actor", "human:alice", "--reason", " "],
        "requires a reason",
    );
    // the log holds no transition after all of this
    let log = fs::read_to_string(vault.join(".mdkernel/events.jsonl")).unwrap();
    assert!(!log.contains("Promoted"));
    // and the good recipe works
    ok(&promote);
    fs::remove_dir_all(&root).unwrap();
}
