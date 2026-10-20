"""Deterministic-field audit of the run comparators (P1-E/E0, K-E0.2;
`scripts/check_compare_fields.sh`, `make check-compare-fields`).

`research/eval/DETERMINISTIC_FIELDS.md` declares, for every comparator
`research/eval/compare_*.py`, the result files it compares (files table)
and every field it does *not* compare, with its class and reason (fields
table). Every other leaf of a compared file is compared, i.e. declared
deterministic by the contract. This checker matches the declaration with
the code, both ways:

- every `compare_*.py` module has a section, every section a module;
- the files table equals the comparators' file constants;
- the fields table equals the comparators' exclusion constants
  (`compare_runs.LATENCY_PATHS`, `compare_p1b1.allowed_fields`,
  `compare_p1c.REGRESS_ALLOWED`, `compare_p1c1.ALLOWED`,
  `compare_p1d.P1D_LATENCY`, `compare_p1d1.{P1D1_TIMING, P1D1_INFO,
  EXCLUDED}` and, from P1-E, `compare_p1e.{P1E_TIMING, P1E_INFO,
  EXCLUDED}`), each row's mode and class consistent: a field a `pair`
  comparison skips must be nondeterministic by contract (`duvar-saati`,
  `bilgi`, `ortam`), a decision exclusion (`karar`) is for `regress` only.

With `--runs A B [C …]` it also confirms the declaration on real runs:
every compared (non-excluded) leaf of every pair file is equal in all the
runs, and every declared nondeterministic field exists in them.

    python -m eval.check_compare_fields [--runs RUN_DIR …] [--out path.json]

Exit 0 when nothing is unmatched (and the runs agree), 3 otherwise, 1 on a
missing or malformed declaration.
"""

from __future__ import annotations

import argparse
import importlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path

from eval import compare_runs as C
from eval.datasets.common import dump_json

HERE = Path(__file__).resolve().parent
# another declaration file (deliberate tests: scripts/test_tamper_p1e.sh)
DOC = Path(os.environ.get("ROOTMARK_DETERMINISTIC_FIELDS", HERE / "DETERMINISTIC_FIELDS.md"))
CLASSES = {"duvar-saati", "bilgi", "ortam", "karar"}
PAIR_SAFE = {"duvar-saati", "bilgi", "ortam"}
MODES = {"pair", "regress", "hepsi"}
# the C2-07 test's index in the synthetic p1b_checks.json below; the
# declaration writes it `<C2-07>` (compare_p1b1 finds it by id)
C207 = 1


class DeclError(Exception):
    pass


def _table(text: str, tag: str) -> list[list[str]]:
    m = re.search(rf"<!-- {tag} -->\n(.*?)\n<!-- /{tag} -->", text, re.DOTALL)
    if not m:
        raise DeclError(f"{DOC.name}: no <!-- {tag} --> table")
    rows = []
    for line in m.group(1).splitlines()[2:]:
        if not line.startswith("|"):
            raise DeclError(f"{DOC.name}: {tag}: not a table row: {line!r}")
        rows.append([c.strip() for c in line.strip().strip("|").split("|")])
    return rows


def _tick(cell: str) -> str:
    m = re.fullmatch(r"`([^`]+)`", cell)
    if not m:
        raise DeclError(f"{DOC.name}: expected one `code` cell, got {cell!r}")
    return m.group(1)


def declared(text: str) -> dict:
    sections = sorted(set(re.findall(r"^### `(compare_[a-z0-9_]+)`", text, re.MULTILINE)))
    fields = set()
    classes = {}
    for r in _table(text, "fields-table"):
        if len(r) != 6:
            raise DeclError(f"{DOC.name}: fields-table row needs 6 cells: {r}")
        tool, mode, file, pat, cls = (_tick(r[0]), r[1], _tick(r[2]), _tick(r[3]), r[4])
        if mode not in MODES or cls not in CLASSES or not r[5]:
            raise DeclError(f"{DOC.name}: bad mode/class/reason in {r}")
        key = (tool, mode, file, pat.replace("<C2-07>", str(C207)))
        if key in fields:
            raise DeclError(f"{DOC.name}: duplicate row {key}")
        fields.add(key)
        classes[key] = cls
    files = {}
    for r in _table(text, "files-table"):
        if len(r) != 2:
            raise DeclError(f"{DOC.name}: files-table row needs 2 cells: {r}")
        files[_tick(r[0])] = re.findall(r"`([^`]+)`", r[1])
    return {"sections": sections, "fields": fields, "classes": classes, "files": files}


def _pat(path) -> str:
    return "/".join(map(str, path))


def _keyed(tool: str, mode: str, keys) -> set:
    """`<file>:<a›b›c>` keys (compare_runs form) as rows."""
    out = set()
    for k in keys:
        f, _, p = k.partition(":")
        out.add((tool, mode, f, p.replace("›", "/")))
    return out


def in_code() -> dict:
    mods = {
        p.stem: importlib.import_module(f"eval.{p.stem}") for p in sorted(HERE.glob("compare_*.py"))
    }
    rows: set = set()

    def latency(tool: str, table: dict, mode: str = "hepsi") -> None:
        for f, pats in table.items():
            rows.update((tool, mode, f, _pat(p)) for p in pats)

    latency("compare_runs", mods["compare_runs"].LATENCY_PATHS)
    with tempfile.TemporaryDirectory() as d:
        tests = [{"id": "C1-01"}] * C207 + [{"id": "C2-07"}]
        (Path(d) / "p1b_checks.json").write_text(json.dumps({"B-1": {"tests": tests}}))
        rows |= _keyed("compare_p1b1", "regress", mods["compare_p1b1"].allowed_fields(Path(d)))
    rows |= _keyed("compare_p1c", "regress", mods["compare_p1c"].REGRESS_ALLOWED)
    rows.update(("compare_p1c1", "regress", f, _pat(p)) for f, p in mods["compare_p1c1"].ALLOWED)
    latency("compare_p1d", mods["compare_p1d"].P1D_LATENCY)
    for name, prefix in (
        ("compare_p1d1", "P1D1"),
        ("compare_p1e", "P1E"),
        ("compare_p1e1", "P1E1"),
        ("compare_p2_0", "P2_0"),
        ("compare_p2_0_1", "P2_0_1"),
        ("compare_p2_0_2", "P2_0_2"),
        ("compare_p2a", "P2A"),
    ):
        if name in mods:
            m = mods[name]
            latency(name, getattr(m, f"{prefix}_TIMING"))
            latency(name, getattr(m, f"{prefix}_INFO"))
            rows.update((name, "regress", e[0], _pat(e[1])) for e in m.EXCLUDED)
    files = {}
    consts = {
        "compare_runs": ["RESULT_FILES", "RESULT_FILES_V3"],
        "compare_p1a": ["RESULT_FILES_P1A"],
        "compare_p1b": ["RESULT_FILES_P1B", "BYTE_FILES", "BYTE_DIRS"],
        "compare_p1c": ["PAIR_FILES", "REGRESS_FILES"],
        "compare_p1c1": ["PAIR_FILES"],
        "compare_p1d": ["P1D_FILES", "P1D_BYTES", "CONFORMANCE"],
        "compare_p1d1": ["P1D1_FILES", "P1D1_BYTES"],
        "compare_p1e": ["P1E_FILES", "P1E_BYTES"],
        "compare_p1e1": ["P1E1_FILES", "P1E1_BYTES"],
        "compare_p2_0": ["P2_0_FILES", "P2_0_BYTES"],
        "compare_p2_0_1": ["P2_0_1_FILES", "P2_0_1_BYTES"],
        "compare_p2_0_2": ["P2_0_2_FILES", "P2_0_2_BYTES"],
        "compare_p2a": ["P2A_FILES"],
    }
    for mod, names in consts.items():
        if mod not in mods:
            continue
        for n in names:
            v = getattr(mods[mod], n)
            v = [v] if isinstance(v, str) else v
            files[f"{mod}.{n}"] = [str(x) for x in v]
    return {"modules": sorted(mods), "fields": rows, "files": files}


def audit() -> dict:
    if not DOC.is_file():
        raise DeclError(f"{DOC} missing")
    d = declared(DOC.read_text(encoding="utf-8"))
    c = in_code()
    fmt = lambda s: sorted(" | ".join(r) for r in s)
    bad_class = sorted(
        " | ".join(k) + f" ({cls})"
        for k, cls in d["classes"].items()
        if (k[1] == "regress") != (cls == "karar")
    )
    files_diff = sorted(
        k for k in set(d["files"]) | set(c["files"]) if d["files"].get(k) != c["files"].get(k)
    )
    res = {
        "declaration": DOC.name,
        "modules": c["modules"],
        "sections_without_module": sorted(set(d["sections"]) - set(c["modules"])),
        "modules_without_section": sorted(set(c["modules"]) - set(d["sections"])),
        "fields_declared": len(d["fields"]),
        "fields_in_code": len(c["fields"]),
        "fields_only_declared": fmt(d["fields"] - c["fields"]),
        "fields_only_in_code": fmt(c["fields"] - d["fields"]),
        "mode_class_mismatch": bad_class,
        "file_constants": len(c["files"]),
        "file_constants_differing": files_diff,
    }
    res["unmatched"] = sum(
        len(res[k])
        for k in (
            "sections_without_module",
            "modules_without_section",
            "fields_only_declared",
            "fields_only_in_code",
            "mode_class_mismatch",
            "file_constants_differing",
        )
    )
    res["_decl"] = d
    return res


def _match(pats: list[str], path: tuple) -> bool:
    for pat in pats:
        p = pat.split("/")
        if len(path) >= len(p) and all(
            x == "*" or x == str(y) for x, y in zip(p, path, strict=False)
        ):
            return True
    return False


def check_runs(decl: dict, runs: list[Path]) -> dict:
    """The newest pair set (every documented pair file) in all `runs`:
    compared leaves equal, declared nondeterministic fields present."""
    files = sorted(
        {
            f
            for k, v in decl["files"].items()
            if k.split(".")[1]
            not in (
                "BYTE_FILES",
                "BYTE_DIRS",
                "P1D_BYTES",
                "P1D1_BYTES",
                "P1E_BYTES",
                "P1E1_BYTES",
                "P2_0_BYTES",
                "P2_0_1_BYTES",
                "P2_0_2_BYTES",
            )
            and k != "compare_p1c.REGRESS_FILES"
            # regress only, compared by test id (`compare_p1d.regress`)
            and k != "compare_p1d.CONFORMANCE"
            and k != "compare_runs.RESULT_FILES"
            for f in v
        }
    )
    skip: dict[str, list[str]] = {}
    for tool, mode, f, pat in decl["fields"]:
        if mode != "regress":
            skip.setdefault(f, []).append(pat)
    out = {"runs": [str(r) for r in runs], "files": [], "differing": {}, "declared_absent": []}
    n = 0
    for f in files:
        present = [r for r in runs if (r / f).is_file()]
        if len(present) < len(runs):
            continue  # a file of a later round than these runs
        out["files"].append(f)
        lv = [C.leaves(json.loads((r / f).read_text(encoding="utf-8"))) for r in runs]
        for pat in skip.get(f, []):
            if not any(_match([pat], p) for p in lv[0]):
                out["declared_absent"].append(f"{f}:{pat}")
        for p, v in lv[0].items():
            if _match(skip.get(f, []), p):
                continue
            n += 1
            vals = [x.get(p, "absent") for x in lv]
            if any(x != v for x in vals[1:]):
                out["differing"][f"{f}:{C._key(p)}"] = vals
    out["compared_leaves"] = n
    out["equal"] = not out["differing"]
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", type=Path, nargs="*", default=[])
    ap.add_argument("--out", type=Path)
    x = ap.parse_args(argv)
    try:
        res = audit()
    except DeclError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    decl = res.pop("_decl")
    ok = res["unmatched"] == 0
    if x.runs:
        if len(x.runs) < 2:
            print("error: --runs needs at least two run directories", file=sys.stderr)
            sys.exit(1)
        res["runs_check"] = check_runs(decl, x.runs)
        ok = ok and res["runs_check"]["equal"]
    res["pass"] = ok
    text = dump_json(res)
    if x.out:
        x.out.write_text(text)
    print(text, end="")
    if not ok:
        print(f"check_compare_fields: FAILED (unmatched {res['unmatched']})", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)
    print(
        f"check_compare_fields: OK ({len(res['modules'])} comparators, "
        f"{res['fields_declared']} declared fields, {res['file_constants']} file constants)",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
