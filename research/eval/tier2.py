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

P2-V.1/E3 (FREEZE_EXCEPTIONS F2; K-P2V.3 T4): the inventory is P2-V.1's
(`reports/rounds/P2-V.1/inventory.json`: P2-V's entries as they were, plus
the round's). Entries may name a checklist item by its id (`item`, B6),
resolved to its position in each record. `fails` reads the chain's own
classifier (`--classifier`) and recomputes each counted FAIL's **full**
difference list (B1; the classifier keeps the first 20): a FAIL is
derived only when every difference is set aside by name — an embedded
entry of a Z/O field, an E1 field (`e1_fields`) or a set-aside criterion's
named fields (`criterion_fields`, B2; a criterion without such an entry
keeps its whole record, as in P2-V), or a field of the criterion's own
record so named; the inventory's `derived_criteria` (B3) are counted like
criteria. `equiv` names the E1 fields and bytes as their own group and
reads `commit_bound` items (R14). `pair` is unchanged but for the items.

    python -m eval.tier2 pair <run_A> <run_B> [--frozen compare_p2r.json] [--out path.json]
    python -m eval.tier2 fails <run_dir> --summary <classifier.json> [--classifier eval.historical_p2r] [--out path.json]
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

INVENTORY = ROOT.parent / "reports/rounds/P2-V.1/inventory.json"
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
    named = [
        *inv.get("e1_fields", []),
        *inv.get("e1_bytes", []),
        *inv.get("derived_criteria", []),
        *[f for c in inv.get("criterion_fields", {}).values() for f in c.get("fields", [])],
    ]
    for e in named:
        names = [e.get("file"), e.get("field"), e.get("path"), e.get("item")]
        if any(isinstance(n, str) and PATTERN_CHARS & set(n) for n in names):
            raise InventoryError(
                f"{e.get('id', e.get('field'))}: a pattern is not a name ({names})"
            )
        if not e.get("why"):
            raise InventoryError(f"{e.get('id', e.get('field'))}: no reason")
    return inv


def resolve(entries: list[dict], run: Path) -> list[dict]:
    """Entries naming a checklist item by id (`item`, B6) with `field` set to
    the item's position in `run`'s record (`items›<i>[›<field_in_item>]`);
    an item absent from the record is an error, never a silent skip."""
    out, docs = [], {}
    for e in entries:
        if not e.get("item"):
            out.append(e)
            continue
        if e["file"] not in docs:
            docs[e["file"]] = json.loads((run / e["file"]).read_text(encoding="utf-8"))
        items = docs[e["file"]].get("items", [])
        idx = [i for i, it in enumerate(items) if it.get("id") == e["item"]]
        if len(idx) != 1:
            raise InventoryError(f"{e['id']}: item {e['item']} is not once in {run / e['file']}")
        f = f"items›{idx[0]}" + (f"›{e['field_in_item']}" if e.get("field_in_item") else "")
        out.append({**e, "field": f})
    return out


def _resolved_pair(entries: list[dict], a: Path, b: Path) -> list[dict]:
    ra, rb = resolve(entries, a), resolve(entries, b)
    for x, y in zip(ra, rb, strict=True):
        if x.get("field") != y.get("field"):
            raise InventoryError(
                f"{x['id']}: item {x['item']} at {x['field']} in A, {y['field']} in B"
            )
    return ra


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
    excl = _resolved_pair(inv["pair_exclusions"], a, b)
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
            {k: e.get(k) for k in ("id", "class", "set", "file", "field", "path", "item")}
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
    a regress that differs only in them is a derived FAIL. A criterion with
    an entry in `criterion_fields` (B2) names only its listed fields there
    (its other fields stay counted); the others their whole record."""
    narrowed = inv.get("criterion_fields", {})
    out = []
    for c in inv.get("tier2_set_aside_criteria", []):
        cid = f"{c['file']}:{c['criterion']}"
        if cid in narrowed:
            out += [
                {"id": cid, "file": f["file"], "field": f["field"]}
                for f in narrowed[cid].get("fields", [])
            ]
            continue
        out.append({"id": cid, "file": c["file"], "field": c["criterion"].replace(".", "·")})
    return out


def _classifier_rows(run_dir: Path, module: str, row: dict) -> list[str] | None:
    """The full difference list of a classifier FAIL row (B1): the chain's
    own classifier (`historical_p2a`'s rules with `module`'s listings and
    excluded subtrees), without the cut to 20. None when the row has no
    difference list (not a listed FAIL, absent from the reference, …)."""
    import importlib

    from eval import historical_p2a as H

    if "differing" not in row:
        return None
    m = importlib.import_module(module)
    listing, migration, excl = historical.load(m.LIST), historical.load(m.MIGRATION), m.excluded()
    f, cid = row["file"], row["criterion"]
    doc = historical.load(run_dir / f)
    rec = historical.criteria(doc)[cid]
    ref = historical.criteria(historical.load(ROOT.parent / listing["reference_run"] / f)).get(cid)
    dec = H.decided(f, cid, isinstance(doc.get("criteria"), dict), excl)
    moved = {(e["file"], e["criterion"]): e for e in migration["entries"]}.get((f, cid))
    if moved is not None:
        paths = [p["path"] for p in moved["paths"]]
        full = historical.differing(rec, ref, ["pass"] + paths + [p for p, _ in dec])
        if not full:
            changed = [p for p in moved["paths"] if H.differs_under(rec, ref, p["path"])]
            full = [p["path"] for p in changed if not H.holds(rec, p)]
    else:
        e = {(x["file"], x["criterion"]): x for x in listing["entries"]}.get((f, cid))
        allowed = [c["path"] for c in (e or {}).get("changes", [])] + [p for p, _ in dec]
        full = historical.differing(rec, ref, allowed)
    if full[: len(row["differing"])] != row["differing"]:
        raise ValueError(f"{f}:{cid}: the classifier's list is not the head of the recomputed one")
    return full


def _own_field(cid: str, path: str) -> str:
    """A classifier path of criterion `cid`'s record (`/`-joined keys) as a
    comparator key of its file (`cid›…`, dots as `·`)."""
    return "›".join(p.replace(".", "·") for p in [cid, *path.split("/")])


def _derived_path(named: list[dict], file: str, cid: str, path: str) -> bool:
    """Whether one difference of a classifier row is set aside by name: an
    embedded entry (`…differing_fields/<file>:<field>`, the fields `a`/`b`
    of a difference record as subfields) naming a set-aside field, or a
    field of the criterion's own record so named."""
    hit = _row_paths({"differing": [path]})
    if hit:
        n, fld = hit[0]
        if fld[-2:] in ("/a", "/b"):
            fld = fld[:-2] + "›" + fld[-1]
        if set_aside(named, n, fld) is not None:
            return True
    return set_aside(named, file, _own_field(cid, path)) is not None


def fails(run_dir: Path, summary: dict, inv: dict, classifier: str | None = None) -> dict:
    counted, aside = [], []
    named = [
        *resolve(inv["pair_exclusions"], run_dir),
        *_criterion_fields(inv),
        *inv.get("e1_fields", []),
    ]
    for row in summary["fails"]:
        if row.get("label") in ("historical", "migration_required", "not_measured"):
            continue
        key = f"{row['file']}:{row['criterion']}"
        e = _criterion_aside(inv, row["file"], row["criterion"])
        if e is not None:
            aside.append({"criterion": key, "class": e["class"], "mac_set": e.get("mac_set")})
            continue
        full = _classifier_rows(run_dir, classifier, row) if classifier else None
        listed = full if full is not None else row.get("differing")
        if listed and all(_derived_path(named, row["file"], row["criterion"], p) for p in listed):
            aside.append(
                {
                    "criterion": key,
                    "class": "türev",
                    "why": "every difference of its full list is set aside by name (a Z/O field, an E1 field or a set-aside criterion's named field)",
                    "differences": len(listed),
                    "full_list": full is not None,
                }
            )
            continue
        bad = [
            p for p in (listed or []) if not _derived_path(named, row["file"], row["criterion"], p)
        ]
        counted.append(
            {
                "criterion": key,
                "why": row.get("why"),
                "differences": len(listed or []),
                "not_set_aside": bad[:50],
                "n_not_set_aside": len(bad),
            }
        )
    if summary.get("counted") is not None:
        listed_ids = sorted(summary["counted"])
        seen = sorted({c["criterion"] for c in counted} | {s["criterion"] for s in aside})
        if listed_ids != seen:
            raise ValueError(f"the classifier's counted list {listed_ids} is not its rows {seen}")
    # B3: the inventory's derived criteria count as criteria of their own
    derived = []
    for d in inv.get("derived_criteria", []):
        p = run_dir / d["file"]
        doc = json.loads(p.read_text(encoding="utf-8")) if p.is_file() else None
        val = doc.get(d["field"]) if isinstance(doc, dict) else None
        derived.append({"criterion": d["id"], "file": d["file"], "field": d["field"], "value": val})
        if val is not True:
            counted.append({"criterion": d["id"], "why": f"{d['file']} {d['field']} is {val!r}"})
    return {
        "mode": "fails",
        "run": run_dir.name,
        "reference_run": summary.get("reference_run"),
        "classifier": classifier,
        "classifier_counted": len(summary.get("counted") or []),
        "counted": counted,
        "counted_fails": len(counted),
        "set_aside": aside,
        "derived_criteria": derived,
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
    # B3: the inventory's derived criteria whose record is in the RUN
    for d in load_inventory().get("derived_criteria", []):
        if (run_dir / d["file"]).is_file():
            out.append(d["id"])
    return sorted(out)


def equiv(old: Path, new: Path, inv: dict, mac: dict | None) -> dict:
    res = compare_p2r.pair(old, new)
    named = [
        *[{**e, "group": "Z/O"} for e in _resolved_pair(inv["pair_exclusions"], old, new)],
        *[{**e, "group": "H"} for e in _resolved_pair(inv["host_fields"], old, new)],
        *[{**e, "group": "commit"} for e in _resolved_pair(inv.get("commit_bound", []), old, new)],
        *[{**e, "group": "E1"} for e in inv.get("e1_fields", [])],
    ]
    e1_bytes = {e["path"] for e in inv.get("e1_bytes", [])}
    bex = {e["path"] for e in inv["byte_exclusions"]}
    by_group: dict[str, list[str]] = {"Z/O": [], "H": [], "commit": [], "E1": []}
    unnamed = {}
    for k, v in res["differing_fields"].items():
        e = set_aside(named, *_split(k))
        if e is None:
            unnamed[k] = v
        else:
            by_group[e["group"]].append(k)
    bytes_unnamed = sorted(
        p for p, r in res["bytes"].items() if not r.get("equal", True) and p not in bex | e1_bytes
    )
    bytes_e1 = sorted(
        p for p, r in res["bytes"].items() if not r.get("equal", True) and p in e1_bytes
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
        "e1_bytes_differing": bytes_e1,
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
    f.add_argument(
        "--classifier",
        required=True,
        help="the chain's classifier module (its `classifier` file, e.g. eval.historical_p2r): full difference lists (B1)",
    )
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
            res = fails(x.run, summary, inv, x.classifier)
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
