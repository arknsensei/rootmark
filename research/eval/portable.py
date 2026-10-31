"""P2-V/V3 (GOAL P2-V V3, V-5): no host-specific value in a record the
pair comparison reads — the chain's work directory, `$HOME`, `$TMPDIR`
(and the runner's temp and workspace directories), the user name in a
path, the host name. Every file of the full pair (`eval.compare_p2r.pair`:
its files and its byte-compared artefacts) or of the fast pair
(`compare_p2r.P2R_FAST_FILES`) is read as text and searched; a hit is red.

The host values are the chain's, not this process's: the chain job writes
them (`--host host.json`: `{"workdir", "home", "tmpdir", "user", "host",
"extra": [...]}`) beside its RUN; this process's own values are searched
too. Each path is searched as given and as its real path (`/var` →
`/private/var` on macOS). The user name counts only inside a path
(`/<user>/`, `~<user>`, `<user>@`): a word is not a host value. Values
shorter than 4 characters and `/tmp` itself are not searched (they are
not specific to a host).

P2-V.1/F4 (Ş2 of P2-V's verification): with `--all` every record of the
RUN is read too — each `*.json` and `*.jsonl` file under it, whether the
pair compares it or not (logs, `*.log`, are diagnostics, not records).

    python -m eval.portable <run_dir> [--host host.json …] [--fast] [--all] [--skip-missing] [--files-from compare.json] [--out path.json]

Exit 0 when nothing is found, 3 when something is (the record names the
file, the value's kind and the first match), 1 on a missing file.
"""

from __future__ import annotations

import argparse
import getpass
import json
import os
import re
import socket
import sys
from pathlib import Path

from eval import compare_p2r
from eval.datasets.common import dump_json

GENERIC = {"/", "/tmp", "/private/tmp", "/var", "/private/var", "/home", "/Users"}


def _variants(p: str) -> set[str]:
    p = p.rstrip("/")
    out = {p}
    try:
        out.add(os.path.realpath(p))
    except OSError:
        pass
    out |= {v[len("/private") :] for v in out if v.startswith("/private/")}
    return {v for v in out if len(v) >= 4 and v not in GENERIC}


def needles(hosts: list[dict]) -> list[tuple[str, re.Pattern]]:
    """`(kind, pattern)` of every host value to search."""
    vals: dict[str, set[str]] = {"path": set(), "user": set(), "host": set()}
    for h in hosts:
        for k in ("workdir", "home", "tmpdir", *(["extra"] if h.get("extra") else [])):
            for v in h.get(k) if k == "extra" else [h.get(k)]:
                if v:
                    vals["path"] |= _variants(v)
        if h.get("user"):
            vals["user"].add(h["user"])
        if h.get("host"):
            hn = h["host"]
            vals["host"] |= {hn, hn.split(".")[0]}
    out = [("path", re.compile(re.escape(v) + r"(?![A-Za-z0-9_])")) for v in sorted(vals["path"])]
    for u in sorted(vals["user"]):
        if len(u) >= 3:
            e = re.escape(u)
            out.append(
                (
                    "user",
                    re.compile(rf"/{e}(?=[/\"'\s])|~{e}(?![A-Za-z0-9_])|(?<![A-Za-z0-9_]){e}@"),
                )
            )
    for hn in sorted(vals["host"]):
        if len(hn) >= 4:
            out.append(
                ("host", re.compile(rf"(?<![A-Za-z0-9_.-]){re.escape(hn)}(?![A-Za-z0-9_-])"))
            )
    return out


def this_host() -> dict:
    return {
        "workdir": os.getcwd(),
        "home": os.environ.get("HOME"),
        "tmpdir": os.environ.get("TMPDIR"),
        "user": getpass.getuser(),
        "host": socket.gethostname(),
        "extra": [
            v for v in (os.environ.get("RUNNER_TEMP"), os.environ.get("GITHUB_WORKSPACE")) if v
        ],
    }


def all_records(run_dir: Path) -> list[str]:
    """Every record of the RUN (P2-V.1/F4): `*.json` and `*.jsonl` files."""
    return sorted(
        str(p.relative_to(run_dir))
        for p in run_dir.rglob("*")
        if p.is_file() and p.suffix in (".json", ".jsonl")
    )


def record_files(
    run_dir: Path,
    fast: bool,
    archived: bool = False,
    files_from: Path | None = None,
    every: bool = False,
) -> list[str]:
    if every:
        base = [] if fast else record_files(run_dir, False, archived, files_from)
        fast_files = list(compare_p2r.P2R_FAST_FILES) if fast else []
        return sorted(set(base) | set(fast_files) | set(all_records(run_dir)))
    if fast:
        return list(compare_p2r.P2R_FAST_FILES)
    if files_from is not None:
        # the frozen comparator's result of this pair (P2-V/F6): its files and
        # byte-compared artefacts, without comparing again
        res = json.loads(files_from.read_text(encoding="utf-8"))
        runs = {str(Path(res.get(k, "")).resolve()) for k in ("run_a", "run_b")}
        if str(run_dir.resolve()) not in runs:
            raise FileNotFoundError(f"{files_from} is not a comparison of {run_dir.name}")
    else:
        try:
            res = compare_p2r.pair(run_dir, run_dir)
        except FileNotFoundError:
            if not archived:
                raise
            # an archive without some artefacts: the files its own pair compared
            res = json.loads((run_dir / "pair_compare.json").read_text(encoding="utf-8"))
    files = list(res["files"])
    for p in res["bytes"]:
        q = run_dir / p
        if q.is_dir():
            files += sorted(str(x.relative_to(run_dir)) for x in q.rglob("*") if x.is_file())
        else:
            files.append(p)
    return files


def scan(
    run_dir: Path,
    files: list[str],
    pats: list[tuple[str, re.Pattern]],
    missing: list[str] | None = None,
) -> list[dict]:
    """Every hit; a missing file is an error unless `missing` collects it
    (an archived RUN that left large artefacts out names them)."""
    hits = []
    for f in files:
        p = run_dir / f
        if not p.is_file():
            if missing is None:
                raise FileNotFoundError(f"{p} missing")
            missing.append(f)
            continue
        text = p.read_text(encoding="utf-8", errors="replace")
        for kind, rx in pats:
            m = rx.search(text)
            if m:
                lo, hi = max(0, m.start() - 60), min(len(text), m.end() + 40)
                hits.append({"file": f, "kind": kind, "match": m.group(0), "context": text[lo:hi]})
    return hits


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("run_dir", type=Path)
    ap.add_argument("--host", type=Path, action="append", default=[])
    ap.add_argument("--fast", action="store_true")
    ap.add_argument(
        "--all",
        action="store_true",
        help="every record of the RUN too (*.json, *.jsonl; P2-V.1/F4)",
    )
    ap.add_argument(
        "--skip-missing", action="store_true", help="an archived RUN: list missing files"
    )
    ap.add_argument(
        "--files-from", type=Path, help="the frozen `compare_p2r pair --out` result naming this RUN"
    )
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    missing: list[str] | None = [] if a.skip_missing else None
    try:
        hosts = [json.loads(h.read_text(encoding="utf-8")) for h in a.host] + [this_host()]
        files = record_files(a.run_dir, a.fast, a.skip_missing, a.files_from, a.all)
        hits = scan(a.run_dir, files, needles(hosts), missing)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    res = {
        "run": a.run_dir.name,
        "mode": ("fast" if a.fast else "pair") + (" + all records" if a.all else ""),
        "files": len(files),
        "hosts": len(hosts),
        "missing": missing,
        "findings": hits,
        "pass": not hits,
    }
    text = dump_json(res)
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    print(text, end="")
    if hits:
        for h in hits:
            print(f"portable: FOUND {h['kind']} {h['match']!r} in {h['file']}", file=sys.stderr)
        sys.exit(3)


if __name__ == "__main__":
    main()
