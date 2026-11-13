"""P2-Q.3 R4 (f), TESTS.md R4: a note deleted in the last seconds of a real
Claude Code session is gone from search in the next session.

Two headless `claude -p` sessions, each with only rootmark as its MCP server
(`--mcp-config <file> --strict-mcp-config`; the server is `rootmark serve
--vault <v> --mcp stdio --watch`, its stderr kept in <work>), on a vault that
is a copy of the open D-NOTES corpus plus one synthetic note
(`zz-probe/session-end.md`, unique word `zqsessionend`), ingested and indexed
before. The vault's recipe sets `window_s = 10` so that the watcher still
holds the removal when the session ends (the hold is `window_s` + 0.5 s).

Session 1: `status`, then, as its very last action, the agent deletes the
probe note with `rm` and ends. Claude Code then stops its server (SIGINT,
SIGTERM, ~0.5 s, SIGKILL). Session 2: the agent calls `search` for the word.
Afterwards (no content or host value is written):
  - whether session 1's server took the removal before it ended (its stderr),
  - whether session 2's server held it at its start (the reconciliation line),
  - the probe note's `BlockDeleted` events in the log, and the paths session
    2's `search` returned for the word (from Claude Code's answer and from the
    MCP tap's shapes are not needed: the answer lists paths only).

    python3 research/p2q3/claude_session_end.py --rootmark <bin> --notes <d_notes/notes> \
        --work <dir> --out <summary.json> [--models <dir>]
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

PROBE = "zz-probe/session-end.md"
WORD = "zqsessionend"
NOTE = f"# Session end probe\n\nA synthetic note with the word {WORD} for the P2-Q.3 probe.\n"
P1 = f"""You are checking a notes vault served by an MCP server named rootmark. Do exactly these steps and nothing else:
1. Call the rootmark `status` tool once.
2. As your very last action, delete the file {PROBE} with the Bash command `rm {PROBE}`.
3. Then answer with the single word DONE. Do not call any tool after step 2."""
P2 = f"""You are checking a notes vault served by an MCP server named rootmark. Do exactly this:
1. Call the rootmark `search` tool with the query "{WORD}" (k = 5, include_proposal = true).
2. Answer with one line per hit: only its doc_path. If there is no hit, answer NO HITS. Do not quote any text."""


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, check=False, **kw)


def ok(cmd, **kw):
    r = sh(cmd, stdin=subprocess.DEVNULL, **kw)
    if r.returncode != 0:
        raise SystemExit(
            f"failed ({r.returncode}): {' '.join(map(str, cmd[:2]))}: {r.stderr[-400:]}"
        )
    return r.stdout


def deletes(vault: Path) -> int:
    n = 0
    for line in (vault / ".rootmark/events.jsonl").read_text(encoding="utf-8").splitlines():
        e = json.loads(line)
        if e.get("kind") == "BlockDeleted" and e.get("doc_path") == PROBE:
            n += 1
    return n


def session(a, vault: Path, work: Path, label: str, prompt: str, allowed: str) -> dict:
    d = work / label
    d.mkdir(parents=True, exist_ok=True)
    log = d / "serve.err"
    log.write_text("")
    cfg = {
        "mcpServers": {
            "rootmark": {
                "type": "stdio",
                "command": "/bin/sh",
                "args": [
                    "-c",
                    f'exec "{Path(a.rootmark).resolve()}" serve --vault "{vault}" --mcp stdio --watch 2>>"{log}"',
                ],
                "env": {"ROOTMARK_MODELS": a.models},
            }
        }
    }
    (d / "mcp.json").write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    t0 = time.monotonic()
    r = sh(
        [
            a.claude,
            "-p",
            prompt,
            "--mcp-config",
            str(d / "mcp.json"),
            "--strict-mcp-config",
            "--allowedTools",
            allowed,
            "--output-format",
            "json",
        ],
        cwd=vault,
        timeout=900,
    )
    secs = round(time.monotonic() - t0, 1)
    (d / "answer.json").write_text(r.stdout, encoding="utf-8")
    answer = ""
    try:
        answer = json.loads(r.stdout).get("result", "")
    except ValueError:
        pass
    time.sleep(2.0)  # the server's last lines
    err = log.read_text(errors="replace")
    stop = re.findall(r"rootmark serve: stopping \(([^)]+)\)", err)
    rec = re.search(
        r"watch: start: reconciled (\d+) files with the log \((\d+) changed, "
        r"(\d+) removals held, (\d+) errors\)",
        err,
    )
    return {
        "exit": r.returncode,
        "seconds": secs,
        "answer_lines": [x.strip() for x in answer.splitlines() if x.strip()][:6],
        "server_stopping": stop,
        "server_stopped_line": "rootmark serve: stopped" in err,
        "server_took_probe_removal": f"watch: {PROBE} removed" in err,
        "reconciled": (
            dict(zip(("files", "changed", "removals_held", "errors"), map(int, rec.groups())))
            if rec
            else None
        ),
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--notes", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--models", default=os.environ.get("ROOTMARK_MODELS", ""))
    ap.add_argument("--claude", default="claude")
    a = ap.parse_args(argv)
    work = a.work.resolve()
    vault = work / "vault"
    if vault.exists():
        shutil.rmtree(vault)
    shutil.copytree(a.notes, vault)
    (vault / PROBE).parent.mkdir(parents=True, exist_ok=True)
    (vault / PROBE).write_text(NOTE, encoding="utf-8")
    rm = a.rootmark
    env = dict(os.environ, ROOTMARK_MODELS=a.models)
    ok([rm, "ingest-dir", str(vault), str(vault)], env=env)
    # the default recipe and the index (`serve` writes and builds them, then
    # ends at the end of stdin), then the recipe's window
    ok([rm, "serve", "--vault", str(vault), "--mcp", "stdio"], env=env)
    pol = vault / ".rootmark/policy.toml"
    text = pol.read_text(encoding="utf-8")
    assert '[attestation]\nunattested = "canonical"\n' in text
    pol.write_text(
        text.replace(
            '[attestation]\nunattested = "canonical"\n',
            '[attestation]\nunattested = "canonical"\nwindow_s = 10.0\n',
        ),
        encoding="utf-8",
    )
    before = deletes(vault)
    s1 = session(a, vault, work, "s1", P1, "mcp__rootmark__status,Bash(rm zz-probe/session-end.md)")
    file_gone = not (vault / PROBE).exists()
    after_s1 = deletes(vault)
    s2 = session(a, vault, work, "s2", P2, "mcp__rootmark__search")
    after_s2 = deletes(vault)
    hits_probe = any(PROBE in x for x in s2["answer_lines"])
    rec = {
        "probe": "P2-Q.3 R4 (f): a note deleted at the end of a real Claude Code session",
        "claude_code_version": sh([a.claude, "--version"]).stdout.strip(),
        "rootmark_version": ok([rm, "--version"]).strip(),
        "recipe": {"unattested": "canonical", "window_s": 10.0},
        "session_1": s1,
        "probe_file_deleted_by_session_1": file_gone,
        "probe_block_deleted_events": {
            "before": before,
            "after_session_1": after_s1,
            "after_session_2": after_s2,
        },
        "removal_taken_at": (
            "session 1's stop"
            if after_s1 > before
            else ("session 2's start" if after_s2 > before else "never")
        ),
        "session_2": s2,
        "search_returned_probe": hits_probe,
        "pass": file_gone and after_s2 > before and not hits_probe and s2["exit"] == 0,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(rec, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: rec[k] for k in ("removal_taken_at", "search_returned_probe", "pass")}))
    return 0 if rec["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
