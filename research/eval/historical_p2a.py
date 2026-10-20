"""Historical FAILs of P2-A (K-E1.4 carried to P2-A; GOAL P2-A 2, K-P2A.2):
the round's classifier, which `scripts/verify.sh` prints as its FAIL
summary, against the **P2-0.2 official RUN**.

Three labels:

- `historical` — as in `eval.historical_p2_0_2`: the failing criterion is
  listed in `historical_fails_p2a.json` (the K-E1.4 kind: a decision taken
  after its round) and its record equals the reference RUN's record except
  the declared `changes` paths (each with its decision) or the subtrees
  `compare_p2a.EXCLUDED` declares strictly below that criterion;
- `migration_required` ("göç gerekli", new in P2-A) — the failing criterion
  is listed in `migration_required_fails.json`: its recorder measured the
  behaviour of an older format version (an older rootmark's index read as
  `index_stale`, …) that P2-A's migration framework replaced; its record
  equals the reference RUN's record except the listed `paths`, and **every
  listed path that differs holds the listed new value** (`expect`, e.g.
  `migration_required`) in this RUN. The K-E1.4 listing does not grow: the
  category has its own data file;
- a listed criterion with `not_measured: true` whose record is `{measured:
  false, skipped: true, pass: null}` — or `pass: null` with every other
  difference from the reference under its declared `changes` (partly
  measured) — is `historical` too (a set not run by a decision, K-P2.3);
- `fail` — anything else (absent from the reference, listed in neither,
  differing elsewhere, or a listed path not holding the expected value): a
  new failure, never hidden; a listed criterion whose checks file is
  missing counts too.

    python -m eval.historical_p2a --run-dir <RUN_DIR> [--out <json>] [--summary]

Exit 0 when the counted FAILs are 0, 3 otherwise (a recorded result), 1 on
an error.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.historical import criteria, differing, leaves, load

LIST = Path(__file__).with_name("historical_fails_p2a.json")
MIGRATION = Path(__file__).with_name("migration_required_fails.json")


def excluded() -> list:
    from eval import compare_p2a

    return compare_p2a.EXCLUDED


def decided(f: str, cid: str, doc_has_criteria: bool, excl: list) -> list[tuple[str, str]]:
    """`(path below the criterion, decision)` of every excluded subtree of
    checks file `f` strictly below criterion `cid`, no wildcard."""
    head = ("criteria", cid) if doc_has_criteria else (cid,)
    out = []
    for e in excl:
        path = tuple(map(str, e[1]))
        if e[0] == f and path[: len(head)] == head and len(path) > len(head) and "*" not in path:
            out.append(("/".join(path[len(head) :]), e[2]))
    return out


def holds(rec: dict, spec: dict) -> bool:
    """Whether the leaf at `spec["path"]` of `rec` (or every leaf below it)
    is the value `spec["expect"]` (A2 review: equality, not a substring),
    or — `spec["prefix"]`, for a message that names this run's temporary
    directory — a string starting with it."""
    path = spec["path"]
    vals = [v for p, v in leaves(rec) if "/".join(p) == path or "/".join(p).startswith(path + "/")]
    if "prefix" in spec:
        return bool(vals) and all(isinstance(v, str) and v.startswith(spec["prefix"]) for v in vals)
    return bool(vals) and all(v == spec["expect"] for v in vals)


def differs_under(a: dict, b: dict, path: str) -> bool:
    """Whether any leaf at or below `path` differs between `a` and `b`."""

    def under(x: dict) -> dict:
        return {
            "/".join(p): v
            for p, v in leaves(x)
            if "/".join(p) == path or "/".join(p).startswith(path + "/")
        }

    return under(a) != under(b)


def classify(
    run_dir: Path,
    listing: dict | None = None,
    migration: dict | None = None,
    excl: list | None = None,
) -> dict:
    listing = listing or load(LIST)
    migration = migration or load(MIGRATION)
    excl = excluded() if excl is None else excl
    if migration["reference_run"] != listing["reference_run"]:
        die("historical_p2a: the two listings name different reference RUNs")
    ref_dir = ROOT.parent / listing["reference_run"]
    if not ref_dir.is_dir():
        die(f"historical_p2a: reference RUN {ref_dir} missing")
    entries = {(e["file"], e["criterion"]): e for e in listing["entries"]}
    moved = {(e["file"], e["criterion"]): e for e in migration["entries"]}
    if set(entries) & set(moved):
        die("historical_p2a: a criterion is in both listings")
    rows = []
    # this round's own record is not an earlier recorder
    files = sorted(p.name for p in run_dir.glob("*_checks.json") if p.name != "p2a_checks.json")
    if not files:
        die(f"historical_p2a: no *_checks.json in {run_dir}")
    for f in files:
        doc = load(run_dir / f)
        crit = criteria(doc)
        has_criteria = isinstance(doc.get("criteria"), dict)
        for cid in sorted(crit):
            rec = crit[cid]
            if rec.get("pass") is True:
                continue
            row = {"file": f, "criterion": cid}
            e0 = entries.get((f, cid))
            ref_path0 = ref_dir / f
            ref0 = criteria(load(ref_path0)).get(cid) if ref_path0.is_file() else None
            if (
                rec.get("pass") is None
                and e0 is not None
                and e0.get("not_measured") is True
                and (
                    (rec.get("measured") is False and rec.get("skipped") is True)
                    # partly measured: what ran equals the reference, what did
                    # not run is a declared change
                    or (
                        ref0 is not None
                        and not differing(
                            rec, ref0, ["pass"] + [c["path"] for c in e0.get("changes", [])]
                        )
                    )
                )
            ):
                # not measured in this chain by a decision (K-P2.3: the real
                # Obsidian sets of a hypothesis round are not a development
                # round's): listed, its record says so
                rows.append(
                    {**row, "label": "historical", "decisions": e0["decisions"], "why": e0["why"]}
                )
                continue
            if not isinstance(rec.get("pass"), bool):
                rows.append(
                    {**row, "label": "fail", "why": f"pass is not a boolean ({rec.get('pass')!r})"}
                )
                continue
            ref_path = ref_dir / f
            ref = criteria(load(ref_path)).get(cid) if ref_path.is_file() else None
            if ref is None:
                rows.append({**row, "label": "fail", "why": "absent from the reference RUN"})
                continue
            dec = decided(f, cid, has_criteria, excl)
            m = moved.get((f, cid))
            if m is not None:
                paths = [p["path"] for p in m["paths"]]
                # `pass` flips with the listed paths: that is the FAIL
                diff = differing(rec, ref, ["pass"] + paths + [p for p, _ in dec])
                # every listed path that differs from the reference holds
                # the expected new value
                changed = [p for p in m["paths"] if differs_under(rec, ref, p["path"])]
                wrong = [p["path"] for p in changed if not holds(rec, p)]
                if diff or wrong:
                    rows.append(
                        {
                            **row,
                            "label": "fail",
                            "why": "listed as migration required, but its record differs elsewhere"
                            if diff
                            else "listed as migration required, but a listed path does not hold the expected value",
                            "differing": (diff or wrong)[:20],
                        }
                    )
                    continue
                rows.append(
                    {
                        **row,
                        "label": "migration_required",
                        "decisions": m["decisions"],
                        "why": m["why"],
                    }
                )
                continue
            e = entries.get((f, cid))
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
                    "why": (e or {}).get("why", "a P2-A decision (compare_p2a.EXCLUDED)"),
                }
            )
    for f, cid in sorted(set(entries) | set(moved)):
        if not (run_dir / f).is_file():
            rows.append(
                {"file": f, "criterion": cid, "label": "missing", "why": "checks file missing"}
            )
    counted = [
        f"{r['file']}:{r['criterion']}"
        for r in rows
        if r["label"] not in ("historical", "migration_required")
    ]
    return {
        "reference_run": listing["reference_run"],
        "fails": rows,
        "historical": sum(1 for r in rows if r["label"] == "historical"),
        "migration_required": sum(1 for r in rows if r["label"] == "migration_required"),
        "counted_fails": len(counted),
        "counted": counted,
    }


def label(row: dict) -> str:
    if row["label"] == "historical":
        return f"`{row['file']}` {row['criterion']}: tarihsel, karar {', '.join(row['decisions'])} ile kapandı — {row['why']}"
    if row["label"] == "migration_required":
        return f"`{row['file']}` {row['criterion']}: göç gerekli, karar {', '.join(row['decisions'])} — {row['why']}"
    return f"`{row['file']}` {row['criterion']}: **sayılan FAIL** ({row['label']}) — {row['why']}"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--summary", action="store_true", help="print as the chain's FAIL summary")
    a = ap.parse_args(argv)
    res = classify(a.run_dir)
    text = json.dumps(res, indent=1, ensure_ascii=False) + "\n"
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    tag = "[verify summary]" if a.summary else "historical_p2a:"
    for r in res["fails"]:
        print(f"{tag} {label(r)}")
    print(
        f"{tag} {res['historical']} historical and {res['migration_required']} "
        f"migration-required FAIL(s) not counted; counted FAILs {res['counted_fails']} "
        f"(eval.historical_p2a against {res['reference_run']})"
    )
    return 0 if res["counted_fails"] == 0 else 3


if __name__ == "__main__":
    sys.exit(main())
