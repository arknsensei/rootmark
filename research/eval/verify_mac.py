"""P2-V Mac set (`make verify-mac`, `scripts/verify_mac.sh`; GOAL P2-V V2,
V-6; K-P2V.1): the inventory's Z and O sets measured on a Mac, three
repetitions, each against its own round's threshold.

The sets and their criteria are the inventory's `mac_sets`
(`reports/rounds/P2-V/inventory.json`, pre-registered in P2-V/V1). Every
criterion is computed with the frozen recorder's own code, not a copy:

- P2-0.2's `H-P2-0.2a`, `a′`, `f` and the D-C half of `d`:
  `eval.run_p2_0_2_checks.checks` on the repetition's RUN (D-C′, the real
  Obsidian half, is P2-0.3's `H-P2-0.3d`);
- P2-0.3's `H-P2-0.3a`…`f`, `b-yük`, `logs_vs_p2a`: `eval.run_p2_0_3.checks`
  on a view of the RUN in which `p2a_checks.json` is `{}` (the chain's
  P2-A recorders are not a Mac set; `checks` only copies them);
- P1-D's `D-10`: `run_p1d_checks`' rule, `d10.pass and sigterm.pass` (the
  1000 ms threshold is the `serve_watch` test's own);
- P1-E.1's `E1-3`, the e2e step of `E1-9`, `E1-4` and the real-Obsidian
  half of `tamper`, P1-E's `E-7`: `run_p1e1_checks`' and `run_p1e_checks`'
  rules on the records of `release.sh`, `release_check.sh`,
  `test_tamper_p1e1_e2e.sh` and `release_local.sh` (E1-9's contract half
  and `tamper_p1e1.json` are deterministic and stay in tier 2);
- P1-C.1's `C1-8`: the frozen recorder's own value — the repetition runs
  the chain's steps from `holdout` to `p1c1-checks` (its reference RUN's
  vectors and scores are bound to the release Mac's hardware).

Criteria whose round set no threshold but byte equality between two chains
(`H-P2-0.2i`, `H-P2-0.3i`: the D-A, D-A2, D-C logs) or "reported"
(`H-P2-0.3b′`) are recorded with their values and marked `eşik yok`: the
logs' blake3 per repetition, equal or not across the three.

    python -m eval.verify_mac rep --run <rep RUN> --main <RUN> --rep <i> --elapsed <s>
    python -m eval.verify_mac summary --run <RUN> --reps 3 --elapsed <s> --setup <s>

`summary` writes `<RUN>/mac_checks.json`: per set and criterion the passes
out of the repetitions, `pass` when every thresholded criterion passed in
3/3; exit 0 when every set passed, 3 otherwise (recorded), 1 on a missing
record.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path

import blake3

from eval import run_p1e1_checks as r1
from eval import run_p2_0_2_checks as c2
from eval import run_p2_0_3 as c3
from eval import runinfo
from eval.datasets.common import ROOT, dump_json

INVENTORY = ROOT.parent / "reports/rounds/P2-V/inventory.json"
LOGS = [f"{s}/{f}" for s in ("d_a", "d_a2", "d_c") for f in ("events.jsonl", "attestations.jsonl")]


def _load(p: Path):
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _p2_0_3(d: Path) -> dict:
    with tempfile.TemporaryDirectory(prefix="rootmark-verify-mac-view-") as t:
        v = Path(t)
        for f in d.iterdir():
            if f.name != "p2a_checks.json":  # replaced below, never written through
                (v / f.name).symlink_to(f.resolve())
        # the chain's P2-A recorders: `checks` only copies them (not a Mac set)
        (v / "p2a_checks.json").write_text("{}\n", encoding="utf-8")
        return c3.checks(v, c3.REF_RUN.resolve())


def _d10(d: Path) -> dict:
    d10, sig = _load(d / "p1d/d10.json"), _load(d / "p1d/sigterm.json")
    # run_p1d_checks' D-10 (the 1000 ms threshold is serve_watch's own)
    return {
        "measured": {"edits": d10["edits"], "p95_ms": d10["p95_ms"], "max_ms": d10["max_ms"]},
        "threshold_p95_ms": 1000,
        "layers_canonical_human": d10["layers_canonical_human"],
        "sigterm_compacts": d10["sigterm_compacted"] and d10["sigterm_exit_0"],
        "sigterm_never_lost": sig["pass"],
        "pass": d10["pass"] and sig["pass"],
    }


def _e1(d: Path) -> tuple[dict, dict]:
    real = _load(d / "p1e1/release/plugin_e2e_real.json")
    if real.get("skipped"):
        sk = {"skipped": True, "reason": real.get("reason"), "pass": False}
        return sk, {**sk, "connection_step": None}
    # run_p1e1_checks' E1-3, and E1-9's e2e step
    e13 = {
        "steps": f"{real['passed']}/{real['total']}",
        "failed": [s["step"] for s in real["steps"] if not s["pass"]],
        "e3_md_changes_only_at_propose": real["e3_md_changes_only_at_propose"],
        "e4_outside_contract": real["e4_access_log"]["outside_contract"],
        "e4_requests": real["e4_access_log"]["requests"],
        "pass": real["passed"] == 11
        and real["total"] == 11
        and len(real["steps"]) == 11
        and real["e3_md_changes_only_at_propose"]
        and real["e4_access_log"]["outside_contract"] == 0,
    }
    e19 = {
        "connection_step": real["e1_9_connection"]["shows"],
        "pass": real["e1_9_connection"]["pass"] is True,
    }
    return e13, e19


P2_0_2_CRITERIA = ("H-P2-0.2a", "H-P2-0.2a′", "H-P2-0.2d", "H-P2-0.2f")
P2_0_3_SETS = ("a", "a′", "b", "b-yük", "b′", "c", "d", "e", "f")
# P2-0.3's criteria computed from the P2-0.2 records alone (`run_p2_0_3.checks`
# reads them through `run_p2_0_2_checks.checks` with only the real-Obsidian
# sets replaced): the same values when the real-Obsidian harness stopped
FROM_P2_0_2 = {"a": "H-P2-0.2a", "a′": "H-P2-0.2a′", "f": "H-P2-0.2f"}


def _err(log: str) -> dict:
    lines = (
        Path(log).read_text(encoding="utf-8", errors="replace").splitlines()
        if Path(log).is_file()
        else []
    )
    return {
        "error": "the set's harness stopped (a failed repetition)",
        "log_tail": lines[-6:],
        "pass": False,
    }


def _e7(d: Path) -> dict:
    rel = _load(d / "p1e/release/release_local.json")
    four = _load(ROOT.parent / "docs/rounds/P1-E/progress/E5_release.json")
    # run_p1e_checks' E-7
    return {
        "chain_local_target": {
            k: rel[k] for k in ("target", "version", "spec_version", "conformance", "pass")
        },
        "four_targets": f"{four['ok']}/4 (docs/rounds/P1-E/progress/E5_release.json)",
        "pass": rel["pass"] and four["ok"] == 4,
    }


def _e14(d: Path) -> dict:
    rc = _load(d / "p1e1/release/release_check.json")
    ids = r1.release_ids()
    items = {i["id"]: i for i in rc["items"]}
    # run_p1e1_checks' E1-4
    return {
        "release_check": f"{rc['passed']}/{rc['total']}",
        "failed": [i for i, v in items.items() if not v["ok"]],
        "every_item_scripted": sorted(ids) == sorted(items),
        "pass": sorted(ids) == sorted(items)
        and items.get("R7", {}).get("ok") is True
        and rc["pass"] is True,
    }


def _tamper_e2e(d: Path) -> dict:
    t = _load(d / "p1e1/tamper_p1e1_e2e.json")
    # run_p1e1_checks' tamper, its real-Obsidian half (tamper_p1e1.json is tier 2's)
    return {
        "rejected": f"{t['rejected']}/{t['rejected_total']}",
        "controls": f"{t['controls_ok']}/{t['controls_total']}",
        "real_cases_run": t["real_cases_run"],
        "half": "tamper_p1e1_e2e (the real Obsidian); tamper_p1e1.json is deterministic, tier 2",
        "pass": t["pass"] is True,
    }


def _c18(d: Path) -> dict:
    doc = _load(d / "p1c1_checks.json")
    c = doc.get("criteria", doc)["C1-8"]
    reg = c["regress"]
    # the frozen recorder's own C1-8 (make p1c1-checks in the repetition)
    return {
        "reference_run": reg["reference_run"],
        "n_fields": reg["n_fields"],
        "differing_fields": sorted(reg["differing_fields"])[:50],
        "n_differing": len(reg["differing_fields"]),
        "pass": c["pass"],
    }


def rep(d: Path, errors: dict[str, str] | None = None) -> dict:
    """Every Mac criterion of one repetition, keyed `<checks file>:<id>`; a
    harness that stopped (`errors`: `p2_0_2` / `p2_0_3` → its log) makes its
    sets' criteria failed, with the error."""
    errors = errors or {}
    if "p2_0_2" in errors:
        e = _err(errors["p2_0_2"])
        a = {k: e for k in P2_0_2_CRITERIA} | {"H-P2-0.2d": {"d_c": e, **e}}
        b = {f"H-P2-0.3{k}": e for k in P2_0_3_SETS} | {"logs_vs_p2a": e}
    else:
        a = c2.checks(d)
        if "p2_0_3" in errors:
            e = _err(errors["p2_0_3"])
            b = {
                f"H-P2-0.3{k}": (
                    {**a[FROM_P2_0_2[k]], "from": FROM_P2_0_2[k]} if k in FROM_P2_0_2 else e
                )
                for k in P2_0_3_SETS
            }
            b["logs_vs_p2a"] = c3._logs(d, c3.REF_RUN.resolve())
        else:
            b = _p2_0_3(d)
    dc = a["H-P2-0.2d"]["d_c"]
    e13, e19 = _e1(d)
    out = {
        "p1c1_checks.json:C1-8": _err(errors["p1c"]) if "p1c" in errors else _c18(d),
        "p1d_checks.json:D-10": _d10(d),
        "p1e_checks.json:E-7": _e7(d),
        "p1e1_checks.json:E1-3": e13,
        "p1e1_checks.json:E1-4": _e14(d),
        "p1e1_checks.json:tamper": _tamper_e2e(d),
        "p1e1_checks.json:E1-9": {**e19, "half": "e2e step (the contract check is tier 2's)"},
        "p2_0_2_checks.json:H-P2-0.2a": a["H-P2-0.2a"],
        "p2_0_2_checks.json:H-P2-0.2a′": a["H-P2-0.2a′"],
        "p2_0_2_checks.json:H-P2-0.2d": {
            "d_c": dc,
            "half": "D-C (≥ 0.99); D-C′ (real Obsidian) is H-P2-0.3d",
            "pass": dc["pass"],
        },
        "p2_0_2_checks.json:H-P2-0.2f": a["H-P2-0.2f"],
    }
    for k in P2_0_3_SETS:
        out[f"p2_0_3_checks.json:H-P2-0.3{k}"] = b[f"H-P2-0.3{k}"]
    out["p2_0_3_checks.json:logs_vs_p2a"] = b["logs_vs_p2a"]
    logs = {
        g: blake3.blake3((d / "p2_0_2" / g).read_bytes()).hexdigest()[:32]
        if (d / "p2_0_2" / g).is_file()
        else None
        for g in LOGS
    }
    out["p2_0_2_checks.json:H-P2-0.2i"] = {"logs": logs, "pass": None}
    out["p2_0_3_checks.json:H-P2-0.3i"] = {"logs": logs, "pass": None}
    return out


def summary(main: Path, reps: int, inv: dict) -> dict:
    rows = []
    for i in range(1, reps + 1):
        rows.append(_load(main / "mac" / f"rep{i}" / "rep.json"))
    sets = []
    for s in inv["mac_sets"]:
        crit = []
        for c in s["criteria"]:
            key = f"{c['file']}:{c['criterion']}"
            vals = [r["criteria"][key] for r in rows]
            if c.get("threshold") is None:
                same = None
                if all("logs" in v for v in vals):
                    same = all(v["logs"] == vals[0]["logs"] for v in vals)
                crit.append(
                    {
                        "criterion": key,
                        "threshold": None,
                        "eşik yok": c.get("why_no_threshold"),
                        "equal_across_reps": same,
                        "values": vals,
                        "pass": None,
                    }
                )
                continue
            passes = [v.get("pass") is True for v in vals]
            crit.append(
                {
                    "criterion": key,
                    "threshold": c["threshold"],
                    "source": c["source"],
                    "passed": f"{sum(passes)}/{reps}",
                    "values": vals,
                    "pass": all(passes) and reps == 3,
                }
            )
        held = [x["pass"] for x in crit if x["threshold"] is not None]
        sets.append({"id": s["id"], "class": s["class"], "criteria": crit, "pass": all(held)})
    return {
        "reps": reps,
        "rep_runs": [r["run"] for r in rows],
        "rep_elapsed_s": [r["elapsed_s"] for r in rows],
        "sets": sets,
        "criteria": sorted({c["criterion"] for s in sets for c in s["criteria"]}),
        "no_threshold": sorted(
            c["criterion"] for s in sets for c in s["criteria"] if c["threshold"] is None
        ),
        "pass": all(s["pass"] for s in sets) and reps == 3,
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("rep")
    r.add_argument("--run", required=True)
    r.add_argument("--main", required=True)
    r.add_argument("--rep", type=int, required=True)
    r.add_argument("--elapsed", type=int, required=True)
    r.add_argument(
        "--error", action="append", default=[], help="<harness>=<log>: a stopped harness"
    )
    s = sub.add_parser("summary")
    s.add_argument("--run", required=True)
    s.add_argument("--reps", type=int, required=True)
    s.add_argument("--elapsed", type=int, required=True)
    s.add_argument("--setup", type=int, required=True)
    a = ap.parse_args(argv)
    main_dir = runinfo.run_dir(a.run if a.cmd == "summary" else a.main, create=False)
    if a.cmd == "rep":
        d = runinfo.run_dir(a.run, create=False)
        errors = dict(x.split("=", 1) for x in a.error)
        out = {
            "run": a.run,
            "rep": a.rep,
            "elapsed_s": a.elapsed,
            "errors": sorted(errors),
            "criteria": rep(d, errors),
        }
        o = main_dir / "mac" / f"rep{a.rep}"
        o.mkdir(parents=True, exist_ok=True)
        (o / "rep.json").write_text(dump_json(out), encoding="utf-8")
        for k, v in out["criteria"].items():
            p = v.get("pass")
            print(f"verify-mac rep {a.rep}: {k} {'-' if p is None else 'pass' if p else 'FAIL'}")
        return 0
    inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
    res = summary(main_dir, a.reps, inv)
    res |= {"elapsed_s": a.elapsed, "setup_s": a.setup, "total_s": a.elapsed + a.setup}
    (main_dir / "mac_checks.json").write_text(dump_json(res), encoding="utf-8")
    for x in res["sets"]:
        print(f"verify-mac: {x['id']} ({x['class']}) {'pass' if x['pass'] else 'FAIL'}")
        for c in x["criteria"]:
            tag = "eşik yok" if c["threshold"] is None else c["passed"]
            print(f"verify-mac:   {c['criterion']} {tag}")
    print(
        f"verify-mac: {'pass' if res['pass'] else 'FAIL'}; reps {res['rep_elapsed_s']} s, total {res['total_s']} s"
    )
    return 0 if res["pass"] else 3


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (FileNotFoundError, KeyError, json.JSONDecodeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
