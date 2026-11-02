"""P2-V.2/Y3 (TASKS Y3 summary; GOAL V2-2, V2-3): the difference between a
reading with P2-V.1's inventory ("önce") and with P2-V.2's ("sonra"), leaf
by leaf, per output file, and whether each difference is one of K-P2V1.2's
four names (E1-W, `tamper·det` historical, H5, `reported_only`) or the
reading's own bookkeeping (the inventory read, the new reading's labels).

    python -m p2v2.name_diff <reread dir> --out <json>   (the run's `<dir>` and `<dir>/before_p2v1`)
    python -m p2v2.name_diff --mac <after.json> <before.json> --out <json>

A difference that no rule names is listed under `unexplained`; exit 3 when
there is one, 0 otherwise.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

FILES = (
    "tier2_pair.json",
    "tier2_fails_A.json",
    "tier2_fails_B.json",
    "portable_A.json",
    "portable_B.json",
    "criteria_A.json",
    "criteria_B.json",
    "equiv_v0m.json",
)


def leaves(x, p=()):
    if isinstance(x, dict):
        if not x:
            yield p, {}
        for k, v in x.items():
            yield from leaves(v, (*p, k))
    elif isinstance(x, list):
        if not x:
            yield p, []
        for i, v in enumerate(x):
            yield from leaves(v, (*p, i))
    else:
        yield p, x


def _diff(a, b) -> list[tuple]:
    x, y = dict(leaves(a)), dict(leaves(b))
    return sorted((k for k in set(x) | set(y) if x.get(k, KeyError) != y.get(k, KeyError)), key=str)


def explain(f: str, k: tuple, before: dict, after: dict) -> str | None:
    """The name a differing leaf comes from, or None."""
    top = k[0] if k else None
    if f == "tier2_pair.json":
        if k in (("inventory",), ("frozen_result_read",)):
            return "okunan envanter / girdi yolu (yankı)"
        return None
    if f.startswith("tier2_fails_"):
        if top in ("counted", "counted_fails", "pass"):
            return "T3 tamper·det tarihsel (sayılan → tarihsel)"
        if top == "historical":
            return "T3 tamper·det tarihsel okuma"
        if top == "derived_criteria" and k[-1] in ("reading", "record_equal"):
            return "T3 tamper·det tarihsel okuma"
        return None
    if f.startswith("portable_"):
        if top in ("findings", "pass"):
            return "T4 H5 adıyla atlandı (conformance.json fixtures_root)"
        if top == "skipped_by_name":
            return "T4 H5 adıyla atlama raporu"
        return None
    if f == "equiv_v0m.json":
        if top in ("d_fields_differing", "pass"):
            return "T2 E1-W (R7 detail adlandırıldı)"
        if k == ("named_counts", "E1"):
            return "T2 E1-W (E1 grubunda +1)"
        return None
    return None


def run_dir(d: Path) -> dict:
    out, bad = {}, []
    for f in FILES:
        a, b = d / "before_p2v1" / f, d / f
        if not a.is_file() and not b.is_file():
            continue
        x = json.loads(a.read_text(encoding="utf-8"))
        y = json.loads(b.read_text(encoding="utf-8"))
        rows = []
        for k in _diff(x, y):
            why = explain(f, k, x, y)
            rows.append({"leaf": "›".join(map(str, k)), "why": why})
            if why is None:
                bad.append(f"{f}:{'›'.join(map(str, k))}")
        bad += _content(f, x, y)
        out[f] = {"differing_leaves": len(rows), "leaves": rows}
    return {"files": out, "unexplained": bad}


def _content(f: str, x: dict, y: dict) -> list[str]:
    """The named leaves differ only in the names' own way: what the new
    reading drops from the old one is exactly the four names' entries, and
    it adds nothing counted."""
    bad = []
    if f.startswith("tier2_fails_"):
        old = {c["criterion"] for c in x["counted"]}
        new = {c["criterion"] for c in y["counted"]}
        if not new <= old or not old - new <= {"p1e1_checks.json:tamper·det"}:
            bad.append(f"{f}: counted {sorted(old)} → {sorted(new)}")
    if f.startswith("portable_"):
        key = lambda h: (h["file"], h["kind"], h["match"])
        old, new = {key(h) for h in x["findings"]}, {key(h) for h in y["findings"]}
        gone = [h for h in x["findings"] if key(h) not in new]
        if not new <= old or any(
            h["file"] != "conformance.json" or '"fixtures_root"' not in h["context"] for h in gone
        ):
            bad.append(f"{f}: findings other than H5 changed")
    if f == "equiv_v0m.json":
        gone = set(x["d_fields_differing"]) - set(y["d_fields_differing"])
        if set(y["d_fields_differing"]) - set(x["d_fields_differing"]) or gone - {
            "p1e1/release/release_check.json:items›7›detail"
        }:
            bad.append(f"{f}: unnamed differences other than R7 changed")
    return bad


MAC_LABELS = ("in_verdict", "reported_only", "reading")


def mac(after: Path, before: Path) -> dict:
    x = json.loads(before.read_text(encoding="utf-8"))
    y = json.loads(after.read_text(encoding="utf-8"))
    ro = {r["criterion"] for r in y.get("reported_only", [])}
    rows, bad = [], []
    for k in _diff(x, y):
        why = None
        if k[0] in MAC_LABELS:
            why = "P2-V.2 okumasının etiketi (T5)"
        elif k[0] == "sets" and len(k) >= 4 and k[2] == "criteria":
            crit = y["sets"][k[1]]["criteria"][k[3]] if isinstance(k[3], int) else None
            name = crit["criterion"] if crit else None
            if len(k) >= 5 and k[4] == "values":
                why = None  # a recorded value must never differ
            elif len(k) >= 5 and k[4] in ("in_verdict", "reported_only"):
                why = "P2-V.2 okumasının etiketi (T5)"
            elif len(k) >= 5 and k[4] == "pass" and name not in ro:
                why = "öteki O kriteri 3/3 ile hükümde (T5: muaf tutulan yalnız beş)"
        elif k[0] == "sets" and len(k) == 3 and k[2] == "pass":
            why = "kümenin hükmü: öteki O kriterleri 3/3 ile hükümde (T5)"
        rows.append({"leaf": "›".join(map(str, k)), "why": why})
        if why is None:
            bad.append("›".join(map(str, k)))
    return {"differing_leaves": len(rows), "leaves": rows, "unexplained": bad}


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("dir", type=Path, nargs="?")
    ap.add_argument("--mac", nargs=2, type=Path)
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args(argv)
    res = mac(*a.mac) if a.mac else run_dir(a.dir)
    a.out.write_text(json.dumps(res, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{a.out}: unexplained {len(res['unexplained'])}")
    sys.exit(3 if res["unexplained"] else 0)


if __name__ == "__main__":
    main()
