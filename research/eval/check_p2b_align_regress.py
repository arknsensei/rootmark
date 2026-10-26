"""P2-R/R0 (c5; P2-B review finding 5): the chain checks the block-location
alignment against a reference instead of only recording it. A run's
`p2b/align.json` (`eval.p2b_align`) regresses when the error of the real
notes, of the Turkish notes or of any edge class is higher than the
reference's, or when the measured set differs (manifest seed, notes digest,
edge digests, block counts). Exit 1 on a regression, 0 otherwise; prints
JSON. The reference is R-3's official value (`reports/rounds/P2-R/r0/
align.json`, K-P2R.2).

    python -m eval.check_p2b_align_regress --run <RUN> --ref <align.json>
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import runinfo


def regressions(got: dict, ref: dict) -> list[str]:
    out = []
    for k in ("manifest_seed", "notes_digest", "edge_sha256"):
        if got.get(k) != ref.get(k):
            out.append(f"{k}: {got.get(k)!r} != reference {ref.get(k)!r}")
    sets = [
        ("notes", got["notes"], ref["notes"]),
        ("notes_turkish", got["notes_turkish"], ref["notes_turkish"]),
    ]
    if set(got["edge"]) != set(ref["edge"]):
        out.append(f"edge classes {sorted(got['edge'])} != reference {sorted(ref['edge'])}")
    sets += [
        (f"edge.{c}", got["edge"][c], ref["edge"][c])
        for c in sorted(ref["edge"])
        if c in got["edge"]
    ]
    for name, g, r in sets:
        if g.get("blocks") != r.get("blocks"):
            out.append(f"{name}: blocks {g.get('blocks')} != reference {r.get('blocks')}")
        if g["error"] > r["error"]:
            out.append(f"{name}: error {g['error']} > reference {r['error']}")
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--ref", required=True)
    a = ap.parse_args(argv)
    got = json.loads((runinfo.run_dir(a.run, create=False) / "p2b" / "align.json").read_text())
    ref = json.loads(Path(a.ref).read_text())
    bad = regressions(got, ref)
    print(json.dumps({"reference": a.ref, "regressions": bad, "pass": not bad}, indent=1))
    if bad:
        print("check_p2b_align_regress: REGRESSION: " + "; ".join(bad), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
