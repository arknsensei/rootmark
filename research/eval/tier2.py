"""P2-V tier 2 and equivalence (GOAL P2-V V2, V3, V4; K-P2V.1): the full
pair's verdict on the deterministic (D) class only, the counted FAILs of a
chain without the timing-sensitive (Z) and real-Obsidian/macOS (O) sets,
the criterion ids of a RUN, and the old ↔ new equivalence.

The classes come from the pre-registered inventory
(`reports/rounds/P2-V/inventory.json`, committed in P2-V/V1 before this
module; never changed after): every compared file and field is D unless the
inventory names it — by file, or by field with its subtree — in
`pair_exclusions` (Z/O, with set and reason), `byte_exclusions` (Z/O
artefacts compared byte for byte) or, for `equiv` only, `host_fields` (H,
fixed in V3) and `commit_bound` (D fields whose input is the commit id
itself). No entry is a pattern: a `*`, `?` or `[` in a name is an error.

The comparison itself is the frozen one (`eval.compare_p2r.pair`, every
file and field of the old full pair): this module only sets aside what the
inventory names and reports it. One structural rule follows the names: a
record that embeds another record's per-field differences (a regress's
`differing_fields` or `excluded_differences`, keyed `<file>:<field>`) has
such an entry set aside iff the field it names is set aside — recursively,
so a regress of a regress follows the same list.

    python -m eval.tier2 pair <run_A> <run_B> [--frozen compare_p2r.json] [--out path.json]
    python -m eval.tier2 fails <run_dir> --summary <classifier.json> [--out path.json]
    python -m eval.tier2 criteria <run_dir> [--out path.json]
    python -m eval.tier2 equiv <old_run> <new_run> [--mac mac_checks.json] [--out path.json]

`pair`: exit 0 when no D field and no D artefact differs, 3 otherwise, 1 on
a missing file or a malformed inventory; it prints the names it set aside
("karşılaştırma dışı: …"). `fails`: the chain's own classifier output
(its FAIL summary, `--summary`), with each counted FAIL either kept or set
aside — a Z/O criterion of the inventory (measured by `make verify-mac`), or
a FAIL whose every reported difference is an embedded entry set aside by
the rule above or naming a Z/O criterion's own record (a regress that
differs only in Z/O fields, e.g. D-9 through C1-8); exit 0 when no
FAIL is counted, 3 otherwise. `equiv`: exit 0 when every difference between
an old-chain RUN and a new one is named (Z/O, H or commit-bound) and every
old criterion id is in the new RUN or the Mac set, 3 otherwise.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2r, historical
from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import ROOT, dump_json

INVENTORY = ROOT.parent / "reports/rounds/P2-V/inventory.json"
EMBEDDING = ("differing_fields", "excluded_differences")
PATTERN_CHARS = set("*?[")


class InventoryError(ValueError):
    pass


def load_inventory(path: Path = INVENTORY) -> dict:
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing")
    inv = json.loads(path.read_text(encoding="utf-8"))
    for group in ("pair_exclusions", "byte_exclusions", "host_fields", "commit_bound"):
        for e in inv.get(group, []):
            names = [e.get("file"), e.get("field"), e.get("path")]
            if any(isinstance(n, str) and PATTERN_CHARS & set(n) for n in names):
                raise InventoryError(f"{group} {e.get('id')}: a pattern is not a name ({names})")
            if not e.get("why") and not e.get("fix"):
                raise InventoryError(f"{group} {e.get('id')}: no reason")
    for e in inv.get("tier2_set_aside_criteria", []):
        if not e.get("mac_set") and not e.get("why"):
            raise InventoryError(f"criterion {e.get('file')}:{e.get('criterion')}: no reason")
    return inv


def _under(key: str, field: str | None) -> bool:
    return field is None or key == field or key.startswith(field + "›")


def _named(entries: list[dict], name: str, key: str) -> dict | None:
    for e in entries:
        if e["file"] == name and _under(key, e.get("field")):
            return e
    return None


def _embedded(key: str) -> tuple[str, str] | None:
    """`(<file>, <field>)` of the first embedded difference entry in a
    comparator key (`…›differing_fields›p2_0_2/d_a·json:rows›0›…`)."""
    parts = key.split("›")
    for i, p in enumerate(parts[:-1]):
        if p in EMBEDDING and ":" in parts[i + 1]:
            name, first = parts[i + 1].split(":", 1)
            # the file name went through `compare_runs._key` (dots as `·`)
            return name.replace("·", "."), "›".join([first, *parts[i + 2 :]])
    return None


def set_aside(entries: list[dict], name: str, key: str, depth: int = 0) -> dict | None:
    """The inventory entry that sets the field `name:key` aside, directly or
    through an embedded difference entry (recursively)."""
    e = _named(entries, name, key)
    if e is not None:
        return e
    emb = _embedded(key)
    if emb is None or depth > 16:
        return None
    inner = set_aside(entries, emb[0], emb[1], depth + 1)
    if inner is None:
        return None
    return {**inner, "via": f"{name}:{key.split('›')[0]}"}


def _split(k: str) -> tuple[str, str]:
    name, key = k.split(":", 1)
    return name, key


def frozen_result(path: Path, a: Path, b: Path) -> dict:
    """The frozen comparator's own output (`compare_p2r pair --out`) for the
    RUNs `a` and `b`, read instead of comparing again (P2-V/F5)."""
    res = json.loads(path.read_text(encoding="utf-8"))
    names = (str(Path(res.get("run_a", "")).resolve()), str(Path(res.get("run_b", "")).resolve()))
    if names != (str(a.resolve()), str(b.resolve())):
        raise ValueError(f"{path} compares {names}, not ({a}, {b})")
    for k in ("files", "n_fields", "differing_fields", "bytes"):
        if k not in res:
            raise ValueError(f"{path}: no `{k}` (not a compare_p2r pair result)")
    return res


def pair(a: Path, b: Path, inv: dict, frozen: dict | None = None) -> dict:
    res = frozen if frozen is not None else compare_p2r.pair(a, b)
    excl = inv["pair_exclusions"]
    bex = {e["path"]: e for e in inv["byte_exclusions"]}
    differing, aside = {}, {}
    for k, v in res["differing_fields"].items():
        e = set_aside(excl, *_split(k))
        if e is None:
            differing[k] = v
        else:
            aside[k] = e["id"] + (f" via {e['via']}" if "via" in e else "")
    bytes_diff, bytes_aside = [], {}
    for p, r in res["bytes"].items():
        if r.get("equal", True):
            continue
        if p in bex:
            bytes_aside[p] = bex[p]["id"]
        else:
            bytes_diff.append(p)
    used = sorted({x.split(" ")[0] for x in aside.values()} | set(bytes_aside.values()))
    return {
        "mode": "pair",
        "frozen_comparison": "eval.compare_p2r.pair (every file and field of the old full pair)",
        "inventory": str(INVENTORY.relative_to(ROOT.parent)),
        "files": res["files"],
        "n_fields": res["n_fields"],
        "differing_fields": differing,
        "differing_bytes": sorted(bytes_diff),
        "set_aside_fields": aside,
        "set_aside_bytes": bytes_aside,
        "not_compared": [
            {k: e.get(k) for k in ("id", "class", "set", "file", "field", "path")}
            for e in [*excl, *inv["byte_exclusions"]]
        ],
        "exclusions_used": used,
        "informational_fields": res.get("informational_fields", {}),
        "equal": not differing and not bytes_diff,
    }


def _criterion_aside(inv: dict, f: str, cid: str) -> dict | None:
    for e in inv.get("tier2_set_aside_criteria", []):
        if e["file"] == f and e["criterion"] == cid:
            return e
    return None


def _row_paths(row: dict) -> list[tuple[str, str]] | None:
    """A classifier row's reported differences as embedded `(file, field)`
    entries; None when one of them is not an embedded entry."""
    # the classifier joins the record's path with `/`
    # (`regress_vs_p1d/differing_fields/p1d/d10.json:pass`); the embedded
    # entry is `<file>:<field>` with the field already in comparator form
    out = []
    for p in row.get("differing") or []:
        hit = None
        for tag in EMBEDDING:
            head = p.split(f"{tag}/", 1)[1] if f"{tag}/" in p else None
            if head is not None and ":" in head:
                hit = tuple(head.split(":", 1))
                break
        if hit is None:
            return None
        out.append(hit)
    return out or None


def _criterion_fields(inv: dict) -> list[dict]:
    """The Z/O criteria's own records as named fields (comparator key form):
    a regress that differs only in them is a derived FAIL."""
    return [
        {
            "id": f"{c['file']}:{c['criterion']}",
            "file": c["file"],
            "field": c["criterion"].replace(".", "·"),
        }
        for c in inv.get("tier2_set_aside_criteria", [])
    ]


def fails(run_dir: Path, summary: dict, inv: dict) -> dict:
    counted, aside = [], []
    named = [*inv["pair_exclusions"], *_criterion_fields(inv)]
    for row in summary["fails"]:
        if row.get("label") in ("historical", "migration_required", "not_measured"):
            continue
        key = f"{row['file']}:{row['criterion']}"
        e = _criterion_aside(inv, row["file"], row["criterion"])
        if e is not None:
            aside.append({"criterion": key, "class": e["class"], "mac_set": e.get("mac_set")})
            continue
        paths = _row_paths(row)
        if paths and all(set_aside(named, n, f) for n, f in paths):
            aside.append(
                {
                    "criterion": key,
                    "class": "türev",
                    "why": "every reported difference is an embedded entry of a Z/O field or criterion",
                    "differing": row["differing"],
                }
            )
            continue
        counted.append({"criterion": key, "why": row.get("why"), "differing": row.get("differing")})
    if summary.get("counted") is not None:
        listed = sorted(summary["counted"])
        seen = sorted({c["criterion"] for c in counted} | {s["criterion"] for s in aside})
        if listed != seen:
            raise ValueError(f"the classifier's counted list {listed} is not its rows {seen}")
    return {
        "mode": "fails",
        "run": run_dir.name,
        "reference_run": summary.get("reference_run"),
        "classifier_counted": len(summary.get("counted") or []),
        "counted": counted,
        "counted_fails": len(counted),
        "set_aside": aside,
        "pass": not counted,
    }


def criteria(run_dir: Path) -> list[str]:
    files = sorted(p.name for p in run_dir.glob("*_checks.json"))
    if not files:
        raise FileNotFoundError(f"no *_checks.json in {run_dir}")
    out = []
    for f in files:
        doc = json.loads((run_dir / f).read_text(encoding="utf-8"))
        out += [f"{f}:{c}" for c in historical.criteria(doc)]
    return sorted(out)


def equiv(old: Path, new: Path, inv: dict, mac: dict | None) -> dict:
    res = compare_p2r.pair(old, new)
    named = [
        *[{**e, "group": "Z/O"} for e in inv["pair_exclusions"]],
        *[{**e, "group": "H"} for e in inv["host_fields"]],
        *[{**e, "group": "commit"} for e in inv.get("commit_bound", [])],
    ]
    bex = {e["path"] for e in inv["byte_exclusions"]}
    by_group: dict[str, list[str]] = {"Z/O": [], "H": [], "commit": []}
    unnamed = {}
    for k, v in res["differing_fields"].items():
        e = set_aside(named, *_split(k))
        if e is None:
            unnamed[k] = v
        else:
            by_group[e["group"]].append(k)
    bytes_unnamed = sorted(
        p for p, r in res["bytes"].items() if not r.get("equal", True) and p not in bex
    )
    c_old, c_new = criteria(old), criteria(new)
    c_mac = sorted(mac.get("criteria", [])) if mac else []
    lost = sorted(set(c_old) - set(c_new) - set(c_mac))
    return {
        "mode": "equiv",
        "old_run": old.name,
        "new_run": new.name,
        "n_fields": res["n_fields"],
        "d_fields_differing": unnamed,
        "d_bytes_differing": bytes_unnamed,
        "named_differences": {g: sorted(v) for g, v in by_group.items()},
        "criteria_old": len(c_old),
        "criteria_new": len(c_new),
        "criteria_mac": len(c_mac),
        "criteria_lost": lost,
        "criteria_new_only": sorted(set(c_new) - set(c_old)),
        "pass": not unnamed and not bytes_unnamed and not lost,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    sub = ap.add_subparsers(dest="mode", required=True)
    p = sub.add_parser("pair")
    p.add_argument("a", type=Path)
    p.add_argument("b", type=Path)
    p.add_argument(
        "--frozen", type=Path, help="the frozen `compare_p2r pair --out` result of these RUNs"
    )
    f = sub.add_parser("fails")
    f.add_argument("run", type=Path)
    f.add_argument("--summary", type=Path, required=True)
    c = sub.add_parser("criteria")
    c.add_argument("run", type=Path)
    e = sub.add_parser("equiv")
    e.add_argument("old", type=Path)
    e.add_argument("new", type=Path)
    e.add_argument("--mac", type=Path)
    for s in (p, f, c, e):
        s.add_argument("--out", type=Path)
        s.add_argument("--inventory", type=Path, default=INVENTORY)
    x = ap.parse_args(argv)
    try:
        inv = load_inventory(x.inventory)
        if x.mode == "pair":
            res = pair(x.a, x.b, inv, frozen_result(x.frozen, x.a, x.b) if x.frozen else None)
            res["frozen_result_read"] = str(x.frozen) if x.frozen else None
            ok = res["equal"]
        elif x.mode == "fails":
            summary = json.loads(x.summary.read_text(encoding="utf-8"))
            res = fails(x.run, summary, inv)
            ok = res["pass"]
        elif x.mode == "criteria":
            ids = criteria(x.run)
            res, ok = {"mode": "criteria", "run": x.run.name, "n": len(ids), "criteria": ids}, True
        else:
            mac = json.loads(x.mac.read_text(encoding="utf-8")) if x.mac else None
            res = equiv(x.old, x.new, inv, mac)
            ok = res["pass"]
    except (
        FileNotFoundError,
        InventoryError,
        M.MetricError,
        json.JSONDecodeError,
        ValueError,
    ) as err:
        print(f"error: {err}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(C._finite(res))
    if x.out:
        x.out.write_text(text, encoding="utf-8")
    print(text, end="")
    if x.mode == "pair":
        print(f"karşılaştırma dışı: {', '.join(res['exclusions_used']) or '-'}", file=sys.stderr)
        print(
            "not compared (inventory): "
            + ", ".join(f"{e['id']}={e.get('file') or e.get('path')}" for e in res["not_compared"]),
            file=sys.stderr,
        )
    if not ok:
        print(f"{x.mode} FAILED", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
