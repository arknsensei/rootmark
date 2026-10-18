"""Historical FAILs of P2-0 (K-E1.4 carried to P2-0; TASKS Z7).

The frozen recorders of earlier rounds (P1-A … P1-E.1) record FAILs in
every chain because of decisions taken after their round. `eval.historical`
(P1-E.1) classifies them against the P1-E official RUN; it cannot see the
P1-E.1 recorders (they are absent from its reference) nor the P2-0
decisions. This module does the same against the **P1-E.1 official RUN**,
with its own listing `historical_fails_p2_0.json` (written from a
development RUN before the official RUN, like `compare_p2_0.EXCLUDED`):

- `historical` when the failing criterion is listed and its record equals
  the same record of the reference RUN, except the declared `changes`
  paths (each with its decision), or the subtrees `compare_p2_0.EXCLUDED`
  declares strictly below that criterion (no wildcard);
- `fail` otherwise (absent from the reference, not listed, or differing
  elsewhere: a new failure, never hidden); a listed criterion whose checks
  file is missing, or a `pass` that is not a boolean, counts.

The record comparison is `eval.historical.differing` (frozen).

    python -m eval.historical_p2_0 --run-dir <RUN_DIR> [--out <json>]

Exit 0 when the counted FAILs are 0, 3 otherwise (a recorded result), 1 on
an error.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2_0
from eval.datasets.common import ROOT, die
from eval.historical import criteria, differing, load

LIST = Path(__file__).with_name("historical_fails_p2_0.json")


def decided(f: str, cid: str, doc_has_criteria: bool) -> list[tuple[str, str]]:
    """`(path below the criterion, decision)` of every `compare_p2_0.EXCLUDED`
    subtree of checks file `f` strictly below criterion `cid`, no wildcard."""
    head = ("criteria", cid) if doc_has_criteria else (cid,)
    out = []
    for e in compare_p2_0.EXCLUDED:
        path = tuple(map(str, e[1]))
        if e[0] == f and path[: len(head)] == head and len(path) > len(head) and "*" not in path:
            out.append(("/".join(path[len(head) :]), e[2]))
    return out


def classify(run_dir: Path, listing: dict | None = None) -> dict:
    listing = listing or load(LIST)
    ref_dir = ROOT.parent / listing["reference_run"]
    if not ref_dir.is_dir():
        die(f"historical_p2_0: reference RUN {ref_dir} missing")
    entries = {(e["file"], e["criterion"]): e for e in listing["entries"]}
    rows = []
    # this round's own record (p2_0_checks.json) is the hypothesis table, not
    # an earlier recorder
    files = sorted(p.name for p in run_dir.glob("*_checks.json") if p.name != "p2_0_checks.json")
    if not files:
        die(f"historical_p2_0: no *_checks.json in {run_dir}")
    seen = set()
    for f in files:
        doc = load(run_dir / f)
        crit = criteria(doc)
        has_criteria = isinstance(doc.get("criteria"), dict)
        for cid in sorted(crit):
            rec = crit[cid]
            if rec.get("pass") is True:
                continue
            row = {"file": f, "criterion": cid}
            seen.add((f, cid))
            if not isinstance(rec.get("pass"), bool):
                rows.append(
                    {**row, "label": "fail", "why": f"pass is not a boolean ({rec.get('pass')!r})"}
                )
                continue
            e = entries.get((f, cid))
            ref_path = ref_dir / f
            ref = criteria(load(ref_path)).get(cid) if ref_path.is_file() else None
            dec = decided(f, cid, has_criteria)
            if ref is None:
                rows.append({**row, "label": "fail", "why": "absent from the reference RUN"})
                continue
            if e is None and not dec:
                rows.append({**row, "label": "fail", "why": "not a listed historical FAIL"})
                continue
            allowed = [c["path"] for c in (e or {}).get("changes", [])] + [p for p, _ in dec]
            diff = differing(rec, ref, allowed)
            if diff:
                rows.append(
                    {
                        **row,
                        "label": "fail",
                        "why": "its record differs from the reference RUN outside the declared changes",
                        "differing": diff[:20],
                    }
                )
                continue
            decisions = list((e or {}).get("decisions", []))
            for c in (e or {}).get("changes", []):
                if c["decision"] not in decisions:
                    decisions.append(c["decision"])
            for _, d in dec:
                if d not in decisions:
                    decisions.append(d)
            rows.append(
                {
                    **row,
                    "label": "historical",
                    "decisions": decisions,
                    "why": (e or {}).get("why", "a P2-0 decision (compare_p2_0.EXCLUDED)"),
                }
            )
    # a listed criterion whose checks file is missing from the RUN counts
    for f, cid in sorted(entries):
        if not (run_dir / f).is_file():
            rows.append(
                {"file": f, "criterion": cid, "label": "missing", "why": "checks file missing"}
            )
    counted = [f"{r['file']}:{r['criterion']}" for r in rows if r["label"] != "historical"]
    return {
        "reference_run": listing["reference_run"],
        "fails": rows,
        "historical": sum(1 for r in rows if r["label"] == "historical"),
        "counted_fails": len(counted),
        "counted": counted,
    }


def label(row: dict) -> str:
    if row["label"] == "historical":
        return f"`{row['file']}` {row['criterion']}: tarihsel, karar {', '.join(row['decisions'])} ile kapandı — {row['why']}"
    return f"`{row['file']}` {row['criterion']}: **sayılan FAIL** ({row['label']}) — {row['why']}"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    res = classify(a.run_dir)
    text = json.dumps(res, indent=1, ensure_ascii=False) + "\n"
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    for r in res["fails"]:
        print(f"historical_p2_0: {label(r)}")
    print(f"historical_p2_0: {res['historical']} historical, {res['counted_fails']} counted")
    return 0 if res["counted_fails"] == 0 else 3


if __name__ == "__main__":
    sys.exit(main())
