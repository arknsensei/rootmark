"""Historical FAILs (K-E1.4; P1-E.1 TASKS E1.4).

Frozen recorders of earlier rounds record FAILs in every chain because of
decisions taken after their round (`historical_fails.json`). This module
reads every `*_checks.json` of a RUN, lists every criterion whose `pass` is
false, and labels it

- `historical` when it is listed in `historical_fails.json` and its record
  equals the same record of the reference RUN (the P1-E official RUN),
  except the declared `changes` paths (each with its decision); it is shown
  with "tarihsel, karar K-… ile kapandı" and not counted;
- `historical` too when its record differs from the reference only under
  the subtrees `compare_p1e1.EXCLUDED` declares for that file and criterion
  (a difference a P1-E.1 decision causes, written before the official RUN);
  its decisions are those of the listing and of the exclusions;
- `fail` otherwise (a FAIL whose record differs from the reference in any
  other leaf is a new failure, never hidden).

A listed criterion (or its checks file) missing from the RUN is reported as
`missing` and counted; a `pass` that is not a boolean is counted. Nothing is
silent: an unreadable checks file is an error. Limit: only criteria that
carry a `pass` are seen (p1a A-1/A-5/A-6, p1b B-1/B-2/B-3/B-6/B-10 and p1c
C-6/C-8 carry none); their fields are compared by `compare_p1e1 regress`
(E1-6). An exclusion of `compare_p1e1.EXCLUDED` admits a difference only
strictly below a criterion and never with a wildcard.

    python -m eval.historical --run-dir <RUN_DIR> [--out <json>]

exits 0 when the counted FAILs are 0, 3 otherwise (a recorded result), 1
on an error.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p1e1
from eval.compare_runs import renamed
from eval.datasets.common import ROOT, die

LIST = Path(__file__).with_name("historical_fails.json")


def criteria(doc: dict) -> dict:
    """The criterion records of one checks file: `{id: record}` for every
    top-level (or `criteria`) entry that is an object with a `pass`."""
    base = doc.get("criteria", doc) if isinstance(doc.get("criteria"), dict) else doc
    return {k: v for k, v in base.items() if isinstance(v, dict) and "pass" in v}


def leaves(x, path: tuple = ()):  # (path tuple, value) of every leaf
    if isinstance(x, (dict, list)) and not x:
        yield path, type(x).__name__ + ":empty"  # an empty container is not an absent value
    elif isinstance(x, dict):
        if len({renamed(k) for k in x}) != len(x):  # never drop a key silently
            raise ValueError(f"{'/'.join(path)}: keys equal after the rename (K-GH.1)")
        for k in sorted(x):
            yield from leaves(x[k], path + (renamed(k),))
    elif isinstance(x, list):
        for i, v in enumerate(x):
            yield from leaves(v, path + (str(i),))
    else:
        # names modulo the project rename (K-GH.1, GH-0/F3)
        yield path, renamed(x) if isinstance(x, str) else x


def differing(a: dict, b: dict, allowed: list[str]) -> list[str]:
    """Paths (`a/b/c`) whose leaves differ between `a` and `b`, outside the
    allowed path prefixes."""
    la = {"/".join(p): v for p, v in leaves(a)}
    lb = {"/".join(p): v for p, v in leaves(b)}

    def ok(p: str) -> bool:
        return any(p == q or p.startswith(q + "/") for q in allowed)

    def emptied(p: str, here: dict, there: dict) -> bool:
        # an empty container on one side whose content on the other side is
        # all under declared changes (F7): the container itself is not a
        # difference; any undeclared leaf inside still is
        inside = [k for k in there if k.startswith(p + "/")]
        return (
            str(here.get(p, "")).endswith(":empty")
            and p not in there
            and bool(inside)
            and all(ok(k) for k in inside)
        )

    out = []
    for p in sorted(set(la) | set(lb)):
        if la.get(p, ("absent",)) == lb.get(p, ("absent",)):
            continue
        if ok(p) or emptied(p, la, lb) or emptied(p, lb, la):
            continue
        out.append(p)
    return out


def decided(f: str, cid: str, doc_has_criteria: bool) -> list[tuple[str, str]]:
    """`(path below the criterion, decision)` of every `compare_p1e1.EXCLUDED`
    subtree of checks file `f` under criterion `cid`."""
    head = ("criteria", cid) if doc_has_criteria else (cid,)
    out = []
    for e in compare_p1e1.EXCLUDED:
        path = tuple(map(str, e[1]))
        # only a subtree strictly below the criterion, without wildcards,
        # can excuse a difference (E1.4 review 1, finding 2)
        if e[0] == f and path[: len(head)] == head and len(path) > len(head) and "*" not in path:
            out.append(("/".join(path[len(head) :]), e[2]))
    return out


def load(path: Path) -> dict:
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        die(f"historical: cannot read {path}: {e}")


def classify(run_dir: Path, listing: dict | None = None) -> dict:
    listing = listing or load(LIST)
    ref_dir = ROOT.parent / listing["reference_run"]
    if not ref_dir.is_dir():
        die(f"historical: reference RUN {ref_dir} missing")
    entries = {(e["file"], e["criterion"]): e for e in listing["entries"]}
    rows = []
    files = sorted(p.name for p in run_dir.glob("*_checks.json"))
    if not files:
        die(f"historical: no *_checks.json in {run_dir}")
    for f in files:
        doc = load(run_dir / f)
        crit = criteria(doc)
        has_criteria = isinstance(doc.get("criteria"), dict)
        for cid in sorted(crit):
            rec = crit[cid]
            if rec.get("pass") is True:
                continue
            if not isinstance(rec.get("pass"), bool):
                rows.append(
                    {
                        "file": f,
                        "criterion": cid,
                        "label": "fail",
                        "why": f"pass is not a boolean ({rec.get('pass')!r})",
                    }
                )
                continue
            e = entries.get((f, cid))
            row = {"file": f, "criterion": cid}
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
            for _, d in dec:
                if d not in decisions:
                    decisions.append(d)
            rows.append(
                {
                    **row,
                    "label": "historical",
                    "decisions": decisions,
                    "why": (e or {}).get(
                        "why",
                        "a frozen recorder of an earlier round; its record changed only under the declared P1-E.1 decision exclusions",
                    ),
                    "declared_changes": (e or {}).get("changes", [])
                    + [{"path": p, "decision": d} for p, d in dec],
                }
            )
    # a listed criterion (or its checks file) absent from the RUN is not a pass
    for (f, cid), _e in sorted(entries.items()):
        if not (run_dir / f).is_file():
            why = "listed checks file absent from the RUN"
        elif cid not in criteria(load(run_dir / f)):
            why = "listed criterion absent from the RUN"
        else:
            continue
        rows.append({"file": f, "criterion": cid, "label": "missing", "why": why})
    counted = [r for r in rows if r["label"] != "historical"]
    return {
        "reference_run": listing["reference_run"],
        "fails": rows,
        "historical": sum(r["label"] == "historical" for r in rows),
        "counted_fails": len(counted),
        "counted": [f"{r['file']}:{r['criterion']}" for r in counted],
    }


def label(row: dict) -> str:
    """The report line of one FAIL (K-E1.4 wording)."""
    if row["label"] == "historical":
        return f"{row['criterion']} ({row['file']}): FAIL — tarihsel, karar {', '.join(row['decisions'])} ile kapandı"
    return f"{row['criterion']} ({row['file']}): FAIL — {row['why']}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run-dir", required=True, type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args()
    r = classify(a.run_dir)
    if a.out:
        a.out.write_text(json.dumps(r, indent=2, ensure_ascii=False) + "\n")
    for row in r["fails"]:
        print(f"historical: {label(row)}")
    print(
        f"historical: {r['historical']} historical FAIL(s) not counted; counted FAILs {r['counted_fails']}"
    )
    return 0 if r["counted_fails"] == 0 else 3


if __name__ == "__main__":
    sys.exit(main())
