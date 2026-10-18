"""P2-0 deliberate cases (TASKS Z7; GOAL P2-0 "doğrulama zinciri"): each
corruption must be rejected, each untouched control must pass.

- control: a save attested with its own bytes, inside the window →
  `canonical`; the untouched vault opens (`events-hash`);
- wrong hash: an attestation of other bytes → the change stays
  `unverified`;
- no token: `attest_session` without `Authorization` → 401;
- wrong MAC: `attest` with a MAC over other bytes → 403;
- unknown session: `attest` with a session the server never opened → 403;
- MCP: `tools/list` has no attestation tool, `tools/call attest` and
  `attest_session` → JSON-RPC -32602 (the agent's channel cannot attest);
- forged log line: an attestation line whose `attestation_id` is not the
  formula's, appended to `.rootmark/attestations.jsonl` → the vault does not
  open;
- removed log line: the line a late `LayerChanged` (cause
  `late_attestation`) refers to, removed → the vault does not open.

Writes `<out.json>`: {cases, rejected, rejected_total, controls_ok,
controls_total, pass} (the shape of scripts/test_tamper_p1e1.sh); exit 1
unless pass.

    python -m eval.tamper_p2_0 --rootmark <bin> --out <out.json>
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

from eval.run_p2_0 import Served, content_hash

WINDOW_S = 2.0


def _case(group: str, case: str, expected: str, got: str, detail=None) -> dict:
    return {
        "group": group,
        "case": case,
        "expected": expected,
        "got": got,
        "detail": detail,
        "as_expected": expected == got,
    }


def _layers(sv: Served, doc: str, old: set[str]) -> list[str]:
    return [b["layer"] for b in sv.blame(doc) or [] if b["version_id"] not in old]


def _http(sv: Served, tool: str, body: dict, token: bool = True) -> int:
    headers = {"X-Consumer": "p2-0-tamper", "Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {sv.token}"
    req = urllib.request.Request(
        f"http://127.0.0.1:{sv.port}/v1/{tool}",
        data=json.dumps(body).encode(),
        headers=headers,
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status
    except urllib.error.HTTPError as e:
        return e.code


def _opens(rootmark: str, vault: Path) -> bool:
    r = subprocess.run(
        [rootmark, "events-hash", str(vault)], capture_output=True, text=True, check=False
    )
    return r.returncode == 0


def _save(sv: Served, doc: str, text: str) -> tuple[bytes, set[str]]:
    old = {b["version_id"] for b in sv.blame(doc) or []}
    p = sv.vault / doc
    data = p.read_bytes().rstrip(b"\n") + f"\n\n{text}\n".encode()
    p.write_bytes(data)
    return data, old


def _settle(sv: Served) -> None:
    sv.quiesce()
    time.sleep(WINDOW_S + 0.5)
    sv.quiesce()


def mcp_cases(rootmark: str, vault: Path) -> list[dict]:
    p = subprocess.Popen(
        [rootmark, "serve", "--vault", str(vault), "--mcp", "stdio"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    assert p.stdin is not None and p.stdout is not None
    msgs = [
        {
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "p2-0-tamper", "version": "1.0"},
            },
        },
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 1, "method": "tools/list"},
    ]
    for i, name in [(2, "attest"), (3, "attest_session")]:
        msgs.append(
            {
                "jsonrpc": "2.0",
                "id": i,
                "method": "tools/call",
                "params": {"name": name, "arguments": {"doc_path": "roasting.md"}},
            }
        )
    for m in msgs:
        p.stdin.write(json.dumps(m) + "\n")
    p.stdin.flush()
    seen: dict[int, dict] = {}
    deadline = time.monotonic() + 60
    while len(seen) < 4:
        if time.monotonic() > deadline:
            p.kill()
            raise RuntimeError("MCP server did not answer")
        line = p.stdout.readline()
        if line == "":
            raise RuntimeError("MCP server closed its output")
        m = json.loads(line)
        if isinstance(m.get("id"), int):
            seen[m["id"]] = m
    p.kill()
    p.wait()
    tools = [t["name"] for t in seen[1]["result"]["tools"]]
    out = [
        _case(
            "mcp",
            "tools/list holds no attestation tool",
            "rejected",
            "accepted" if any("attest" in t for t in tools) else "rejected",
            {"tools": len(tools)},
        )
    ]
    for i, name in [(2, "attest"), (3, "attest_session")]:
        err = seen[i].get("error") or {}
        out.append(
            _case(
                "mcp",
                f"tools/call {name}",
                "rejected",
                "rejected" if err.get("code") == -32602 else "accepted",
                {"code": err.get("code")},
            )
        )
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    cases: list[dict] = []
    with tempfile.TemporaryDirectory(prefix="rootmark-tamper-p2-0-") as w:
        work = Path(w)
        sv = Served(rootmark, work / "served", WINDOW_S)
        try:
            # control: the plugin's protocol with the file's own bytes
            data, old = _save(sv, "roasting.md", "A human control paragraph about the drum.")
            sv.attest("roasting.md", data)
            _settle(sv)
            lay = _layers(sv, "roasting.md", old)
            cases.append(
                _case(
                    "attest",
                    "save attested with its own bytes (control)",
                    "accepted",
                    "accepted" if lay and all(x == "canonical" for x in lay) else "rejected",
                    {"layers": lay},
                )
            )
            # wrong hash
            data, old = _save(sv, "filter.md", "A paragraph whose attestation names other bytes.")
            sv.attest("filter.md", data + b"x")
            _settle(sv)
            lay = _layers(sv, "filter.md", old)
            cases.append(
                _case(
                    "attest",
                    "attestation of other bytes",
                    "rejected",
                    "rejected" if lay and all(x == "unverified" for x in lay) else "accepted",
                    {"layers": lay},
                )
            )
            # no token, wrong MAC, unknown session
            st = _http(sv, "attest_session", {}, token=False)
            cases.append(
                _case(
                    "http",
                    "attest_session without a token",
                    "rejected",
                    "rejected" if st == 401 else "accepted",
                    {"status": st},
                )
            )
            assert sv.session is not None
            sid, key = sv.session
            h = content_hash((sv.vault / "espresso.md").read_bytes())
            wrong = hmac.new(
                key, b"rootmark-attest/1\nespresso.md\n" + b"0" * 32 + b"\n", hashlib.sha256
            ).hexdigest()
            st = _http(
                sv,
                "attest",
                {"doc_path": "espresso.md", "content_hash": h, "session_id": sid, "mac": wrong},
            )
            cases.append(
                _case(
                    "http",
                    "attest with a MAC over other bytes",
                    "rejected",
                    "rejected" if st == 403 else "accepted",
                    {"status": st},
                )
            )
            mac = hmac.new(
                key, f"rootmark-attest/1\nespresso.md\n{h}\n".encode(), hashlib.sha256
            ).hexdigest()
            st = _http(
                sv,
                "attest",
                {"doc_path": "espresso.md", "content_hash": h, "session_id": "nope", "mac": mac},
            )
            cases.append(
                _case(
                    "http",
                    "attest with a session the server never opened",
                    "rejected",
                    "rejected" if st == 403 else "accepted",
                    {"status": st},
                )
            )
            # a late attestation (for the removed-line case below)
            data, old = _save(sv, "cupping.md", "A human paragraph attested late.")
            sv.quiesce()
            r = sv.attest("cupping.md", data)
            late_id = r["attestation_id"]
            late_raised = len(r.get("raised") or [])
            sv.quiesce()
        finally:
            sv.stop()
        v = sv.vault
        cases.append(
            _case(
                "log",
                "untouched vault opens (control)",
                "accepted",
                "accepted" if _opens(rootmark, v) else "rejected",
            )
        )
        log = v / ".rootmark/attestations.jsonl"
        keep = log.read_bytes()
        lines = [json.loads(x) for x in keep.decode().splitlines()]
        forged = dict(lines[0])
        forged["attestation_id"] = "0" * 32
        log.write_bytes(keep + (json.dumps(forged) + "\n").encode())
        cases.append(
            _case(
                "log",
                "forged attestation line appended",
                "rejected",
                "accepted" if _opens(rootmark, v) else "rejected",
            )
        )
        log.write_bytes(
            "".join(json.dumps(x) + "\n" for x in lines if x["attestation_id"] != late_id).encode()
        )
        cases.append(
            _case(
                "log",
                "line of a late attestation removed",
                "rejected",
                "accepted" if _opens(rootmark, v) else "rejected",
                {"raised": late_raised, "as_expected_needs_raise": late_raised > 0},
            )
        )
        if late_raised == 0:
            cases[-1]["as_expected"] = False
        log.write_bytes(keep)
        mv = work / "mcp"
        shutil.copytree(v, mv)
        cases += mcp_cases(rootmark, mv)
    res = {
        "cases": cases,
        "rejected": sum(1 for c in cases if c["expected"] == "rejected" and c["as_expected"]),
        "rejected_total": sum(1 for c in cases if c["expected"] == "rejected"),
        "controls_ok": sum(1 for c in cases if c["expected"] == "accepted" and c["as_expected"]),
        "controls_total": sum(1 for c in cases if c["expected"] == "accepted"),
    }
    res["pass"] = (
        res["rejected"] == res["rejected_total"] and res["controls_ok"] == res["controls_total"]
    )
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(res, indent=1) + "\n")
    print(
        f"tamper_p2_0: rejected {res['rejected']}/{res['rejected_total']}, "
        f"controls {res['controls_ok']}/{res['controls_total']}",
        file=sys.stderr,
    )
    return 0 if res["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
