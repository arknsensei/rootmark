"""P2-Q.2/R0 (TASKS R0): a stdio tap between an MCP client and a server.

Started by the client in place of the server; starts the server, copies the
client's stdin to it and its stdout back, byte for byte, and appends one
record per JSON-RPC message to <log> giving only the message's *shape*:
direction, method, whether it has an id, the keys of `params` and of
`params._meta`, the protocol versions it names, the client's name/version
and capability keys, the keys of a result and an error's code and message.
Tool arguments, tool results, texts, paths and host values are never
written (the record holds no content). Python standard library only.

    python3 mcp_tap.py --log <file.jsonl> -- <server command…>
"""

from __future__ import annotations

import json
import subprocess
import sys
import threading

PV = "io.modelcontextprotocol/protocolVersion"
CI = "io.modelcontextprotocol/clientInfo"
CC = "io.modelcontextprotocol/clientCapabilities"


def keys(x) -> list[str] | None:
    return sorted(x) if isinstance(x, dict) else None


def shape(direction: str, line: bytes) -> dict:
    try:
        m = json.loads(line)
    except ValueError:
        return {"dir": direction, "json": False, "bytes": len(line)}
    if not isinstance(m, dict):
        return {"dir": direction, "json": True, "batch": isinstance(m, list)}
    out: dict = {"dir": direction, "jsonrpc": m.get("jsonrpc")}
    if "method" in m:
        out["method"] = m["method"]
    out["has_id"] = "id" in m
    if "id" in m:
        out["id_type"] = type(m["id"]).__name__
    p = m.get("params")
    if isinstance(p, dict):
        out["params_keys"] = keys(p)
        meta = p.get("_meta")
        if isinstance(meta, dict):
            out["meta_keys"] = keys(meta)
            if PV in meta:
                out["meta_protocolVersion"] = meta[PV]
            if isinstance(meta.get(CI), dict):
                out["meta_clientInfo"] = {k: meta[CI].get(k) for k in ("name", "version")}
            if CC in meta:
                out["meta_clientCapabilities_keys"] = keys(meta[CC])
        if m.get("method") == "initialize":
            out["protocolVersion"] = p.get("protocolVersion")
            ci = p.get("clientInfo")
            if isinstance(ci, dict):
                out["clientInfo"] = {k: ci.get(k) for k in ("name", "version")}
            out["capabilities_keys"] = keys(p.get("capabilities"))
        if m.get("method") == "tools/call":
            out["tool"] = p.get("name")
    r = m.get("result")
    if isinstance(r, dict):
        out["result_keys"] = keys(r)
        if "protocolVersion" in r:
            out["result_protocolVersion"] = r["protocolVersion"]
        if isinstance(r.get("tools"), list):
            out["tools"] = sorted(t.get("name") for t in r["tools"] if isinstance(t, dict))
        if "isError" in r:
            out["isError"] = r["isError"]
    e = m.get("error")
    if isinstance(e, dict):
        out["error"] = {"code": e.get("code"), "message": e.get("message")}
    return out


def main() -> int:
    argv = sys.argv[1:]
    if len(argv) < 4 or argv[0] != "--log" or argv[2] != "--":
        print("usage: mcp_tap.py --log <file.jsonl> -- <server command…>", file=sys.stderr)
        return 2
    log_path, cmd = argv[1], argv[3:]
    lock = threading.Lock()
    log = open(log_path, "a", encoding="utf-8")  # noqa: SIM115 (lives as long as the tap)

    def record(direction: str, line: bytes) -> None:
        s = line.strip()
        if not s:
            return
        with lock:
            log.write(json.dumps(shape(direction, s), sort_keys=True) + "\n")
            log.flush()

    child = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)

    def up() -> None:  # client → server
        for line in sys.stdin.buffer:
            record("c2s", line)
            try:
                child.stdin.write(line)
                child.stdin.flush()
            except BrokenPipeError:
                break
        try:
            child.stdin.close()
        except BrokenPipeError:
            pass

    t = threading.Thread(target=up, daemon=True)
    t.start()
    for line in child.stdout:  # server → client
        record("s2c", line)
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()
    return child.wait()


if __name__ == "__main__":
    sys.exit(main())
