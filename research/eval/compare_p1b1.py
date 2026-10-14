"""P1-B.1 regression against the P1-B official RUN (GOAL B1-6): every quality
field of the P1-B result set (`compare_p1b.RESULT_FILES_P1B`, frozen
comparison `compare_runs.compare`, same latency exclusions) and the governance
vault's `events.jsonl` and `checkpoints/` byte for byte. The only fields
allowed to differ are the ones GOAL B1-6 names as newly expected (K-B1.2,
spec 1.0.1 §8):

- C2-07 (`p1b_checks.json` B-1 entry of C2-07: `actual_hash`,
  `expected_hash`) and, through it, `fixtures_manifest_hash` (the manifest
  lists the C2-07 file);
- `traces.jsonl` (its byte comparison and `traces_sha256` in
  `governance/governance.json` and `p1b_checks.json`).

Every other difference, a missing file or a missing field fails.

    python -m eval.compare_p1b1 <p1b_run_dir> <run_dir> [--out path.json]

Exit code: 0 when nothing outside the exclusions differs, 3 when something
does (the comparison is still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1b import RESULT_FILES_P1B, VAULT, _tree
from eval.datasets.common import dump_json, file_blake3


def allowed_fields(run: Path) -> set[str]:
    """Field keys (`compare_runs` form `<file>:<a›b›c>`) GOAL B1-6 excludes."""
    checks = json.loads((run / "p1b_checks.json").read_text(encoding="utf-8"))
    ix = [i for i, t in enumerate(checks["B-1"]["tests"]) if t["id"] == "C2-07"]
    if len(ix) != 1:
        raise M.MetricError("p1b_checks.json: no single C2-07 entry")
    i = ix[0]
    return {
        f"p1b_checks.json:B-1›tests›{i}›actual_hash",
        f"p1b_checks.json:B-1›tests›{i}›expected_hash",
        "p1b_checks.json:fixtures_manifest_hash",
        "p1b_checks.json:governance›traces_sha256",
        "governance/governance.json:traces_sha256",
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("p1b_run", type=Path)
    ap.add_argument("run", type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    try:
        res = C.compare(a.p1b_run, a.run, RESULT_FILES_P1B)
        allowed = allowed_fields(a.p1b_run) & allowed_fields(a.run)
        byte_files = {}
        for f in (VAULT / "events.jsonl", VAULT / "traces.jsonl"):
            pa, pb = a.p1b_run / f, a.run / f
            for p in (pa, pb):
                if not p.is_file():
                    raise FileNotFoundError(f"{p} missing")
            byte_files[f.as_posix()] = {
                "blake3_p1b": file_blake3(pa),
                "blake3": file_blake3(pb),
                "equal": pa.read_bytes() == pb.read_bytes(),
                "excluded": f.name == "traces.jsonl",
            }
        d = VAULT / "checkpoints"
        ta, tb = _tree(a.p1b_run / d), _tree(a.run / d)
        if not ta:
            raise FileNotFoundError(f"{a.p1b_run / d} is empty")
        byte_files[d.as_posix() + "/"] = {"files": len(ta), "equal": ta == tb, "excluded": False}
    except (FileNotFoundError, M.MetricError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    diff = res["differing_fields"]
    res["allowed_fields"] = sorted(allowed)
    res["differing_excluded"] = sorted(k for k in diff if k in allowed)
    res["differing_outside_exclusions"] = sorted(k for k in diff if k not in allowed)
    res["bytes"] = byte_files
    res["bytes_equal_outside_exclusions"] = all(
        v["equal"] for v in byte_files.values() if not v["excluded"]
    )
    res["B1-6_equal"] = (
        not res["differing_outside_exclusions"] and res["bytes_equal_outside_exclusions"]
    )
    text = dump_json(C._finite(res))
    if a.out:
        a.out.write_text(text)
    print(text, end="")
    if not res["B1-6_equal"]:
        print(
            f"B1-6 FAILED: differing outside the exclusions {res['differing_outside_exclusions']}, "
            f"bytes equal {res['bytes_equal_outside_exclusions']}",
            file=sys.stderr,
        )
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
