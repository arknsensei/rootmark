"""P2-Q.2/R1 (Q2-3): the `tools/list` result a generic MCP client gets.

Starts `<rootmark> serve --vault <vault> --mcp stdio`, sends `initialize`
(protocol version as given, no `_meta`), `notifications/initialized` and
`tools/list`, and writes the `result` of `tools/list` to <out> as compact
JSON (the server's bytes re-serialized with sorted keys, so two servers'
lists compare byte for byte). Python standard library only.

    python3 mcp_list.py --rootmark <bin> --vault <vault> --out <file>
        [--protocol 2025-06-18]
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--vault", required=True)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--protocol", default="2025-06-18")
    a = ap.parse_args(argv)
    p = subprocess.Popen(
        [a.rootmark, "serve", "--vault", a.vault, "--mcp", "stdio"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )

    def send(m: dict) -> None:
        p.stdin.write(json.dumps(m) + "\n")
        p.stdin.flush()

    def answer(i: int) -> dict:
        for line in p.stdout:
            m = json.loads(line)
            if m.get("id") == i:
                return m
        raise SystemExit(f"no answer to request {i}")

    send(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": a.protocol,
                "capabilities": {},
                "clientInfo": {"name": "p2q2-list", "version": "1"},
            },
        }
    )
    init = answer(1)
    if "result" not in init:
        raise SystemExit(f"initialize failed: {init}")
    send({"jsonrpc": "2.0", "method": "notifications/initialized"})
    send({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
    r = answer(2)
    p.stdin.close()
    p.wait(timeout=120)
    if "result" not in r:
        raise SystemExit(f"tools/list failed: {r}")
    a.out.write_text(
        json.dumps(r["result"], sort_keys=True, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(f"{len(r['result']['tools'])} tools, negotiated {init['result'].get('protocolVersion')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
