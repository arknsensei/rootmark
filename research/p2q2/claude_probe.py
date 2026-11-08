"""P2-Q.2 (TASKS R0, R1, R4): real Claude Code, headless, against rootmark.

Runs one `claude -p` session with only rootmark as an MCP server
(`--mcp-config <file> --strict-mcp-config`), started through `mcp_tap.py`,
on a vault that is a copy of the open D-NOTES corpus. The session is asked
to list rootmark's tools, run one `search` and one `propose` (a new file).
Afterwards this script reads, without writing any content or host value:
  - the tap's record (`requests.jsonl`: message shapes only);
  - Claude Code's debug log, for rootmark's MCP lines (status, errors);
  - the vault's event log, for the `propose` and its actor;
and writes `<out>/summary.json`. The debug log, the session's answer and
the tap's raw record stay in <work> (scratch, outside the repository).

    python3 claude_probe.py --rootmark <bin> --vault <vault> --work <dir>
        --out <summary dir> --label <name> [--models <dir>]
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TOOLS = [
    "blame",
    "checkpoint",
    "checkpoints",
    "events",
    "history",
    "promote",
    "propose",
    "read_block",
    "search",
    "status",
    "trace",
]
PROBE_PATH = "agent-probe/claude-code.md"
PROMPT = f"""You are checking an MCP server named rootmark. Do exactly these steps and nothing else:
1. Say how many tools the rootmark MCP server offers you and list their names (only names).
2. Call the rootmark `search` tool with the query "dataview" (k = 3).
3. Call the rootmark `propose` tool with doc_path "{PROBE_PATH}", mode "create", reason "P2-Q.2 MCP compatibility probe" and content "# Probe\\n\\nA probe note proposed by the agent over MCP.\\n".
4. Answer in one line per step: step number, SUCCEEDED or FAILED, and the error text if it failed. Do not quote search results."""


def sh(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, check=False, **kw)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--vault", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--label", required=True)
    ap.add_argument("--models", default=os.environ.get("ROOTMARK_MODELS", ""))
    ap.add_argument("--claude", default="claude")
    a = ap.parse_args(argv)
    work = a.work.resolve() / a.label
    work.mkdir(parents=True, exist_ok=True)
    vault = a.vault.resolve()
    tap_log = work / "requests.jsonl"
    tap_log.unlink(missing_ok=True)
    events = vault / ".rootmark" / "events.jsonl"
    events_before = events.read_bytes().count(b"\n") if events.exists() else 0
    cfg = {
        "mcpServers": {
            "rootmark": {
                "type": "stdio",
                "command": sys.executable,
                "args": [
                    str(HERE / "mcp_tap.py"),
                    "--log",
                    str(tap_log),
                    "--",
                    str(Path(a.rootmark).resolve()),
                    "serve",
                    "--vault",
                    str(vault),
                    "--mcp",
                    "stdio",
                ],
                "env": {"ROOTMARK_MODELS": a.models},
            }
        }
    }
    (work / "mcp.json").write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    debug = work / "debug.log"
    debug.unlink(missing_ok=True)
    version = sh([a.claude, "--version"]).stdout.strip()
    rm_version = sh([a.rootmark, "--version"]).stdout.strip()
    allowed = ",".join(f"mcp__rootmark__{t}" for t in ("search", "propose", "status"))
    cmd = [
        a.claude,
        "-p",
        PROMPT,
        "--mcp-config",
        str(work / "mcp.json"),
        "--strict-mcp-config",
        "--allowedTools",
        allowed,
        "--output-format",
        "json",
        "--debug-file",
        str(debug),
    ]
    r = sh(cmd, cwd=work, timeout=900)
    (work / "answer.json").write_text(r.stdout, encoding="utf-8")
    (work / "stderr.txt").write_text(r.stderr, encoding="utf-8")
    answer = ""
    try:
        answer = json.loads(r.stdout).get("result", "")
    except ValueError:
        pass

    shapes = [json.loads(l) for l in tap_log.read_text().splitlines()] if tap_log.exists() else []
    listed = next((s["tools"] for s in shapes if "tools" in s), None)
    list_errors = []
    by_id_method = {}
    for s in shapes:
        if s["dir"] == "c2s" and s.get("method"):
            by_id_method.setdefault(s["method"], 0)
            by_id_method[s["method"]] += 1
        if s["dir"] == "s2c" and "error" in s:
            list_errors.append(s["error"])
    calls = [s.get("tool") for s in shapes if s.get("method") == "tools/call"]

    dbg = debug.read_text(errors="replace") if debug.exists() else ""
    mcp_lines = []
    for line in dbg.splitlines():
        if 'MCP server "rootmark"' in line:
            # drop the timestamp and anything path-like
            t = re.sub(r"^\S+\s+\[[A-Z]+\]\s*", "", line)
            t = re.sub(r"(?<![\w.])(/[^\s\"',]+)+", "<path>", t)
            mcp_lines.append(t[:400])

    new_events = []
    if events.exists():
        for l in events.read_text(encoding="utf-8").splitlines()[events_before:]:
            e = json.loads(l)
            meta = e.get("meta", {})
            new_events.append(
                {
                    "kind": e.get("kind") or e.get("type"),
                    "doc": e.get("doc_path"),
                    "actor": meta.get("actor"),
                    "layer": meta.get("layer"),
                    "source": (meta.get("provenance") or {}).get("source"),
                    "author_kind": (meta.get("provenance") or {}).get("author_kind"),
                }
            )
    probe_file = (vault / PROBE_PATH).exists()
    # the proposal waits for a person: its blocks' layers (the session's
    # server has stopped; the CLI reads the vault)
    probe_layers = None
    if probe_file:
        b = sh([a.rootmark, "blame", str(vault), PROBE_PATH])
        if b.returncode == 0:
            probe_layers = [x.get("layer") for x in json.loads(b.stdout).get("blocks", [])]
    agent_events = [
        e
        for e in new_events
        if e["author_kind"] == "Agent" or (e["actor"] or {}).get("kind") == "agent"
    ]
    summary = {
        "label": a.label,
        "claude_code_version": version,
        "rootmark_version": rm_version,
        "command": "claude -p <PROMPT> --mcp-config <work>/mcp.json --strict-mcp-config "
        f"--allowedTools {allowed} --output-format json --debug-file <work>/debug.log",
        "server_command": "python3 research/p2q2/mcp_tap.py --log <work>/requests.jsonl -- "
        "<rootmark> serve --vault <vault> --mcp stdio",
        "exit": r.returncode,
        "requests": shapes,
        "methods_sent": by_id_method,
        "tools_listed": listed,
        "tools_listed_count": len(listed) if listed is not None else 0,
        "tools_match_0_3_1": listed == TOOLS,
        "tool_calls": calls,
        "server_errors": list_errors,
        "debug_rootmark_lines": mcp_lines,
        "answer_steps": [
            ln.strip()
            for ln in answer.splitlines()
            if re.match(r"^\s*\**\s*(step\s*)?[1-4]\b", ln, re.IGNORECASE)
        ][:8],
        "new_events": new_events,
        "agent_events": len(agent_events),
        "probe_file_written": probe_file,
        "probe_layers": probe_layers,
    }
    a.out.mkdir(parents=True, exist_ok=True)
    (a.out / f"{a.label}.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(
        json.dumps(
            {
                k: summary[k]
                for k in (
                    "label",
                    "exit",
                    "tools_listed_count",
                    "tool_calls",
                    "agent_events",
                    "server_errors",
                )
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
