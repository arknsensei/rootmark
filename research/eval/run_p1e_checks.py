"""P1-E criteria recorded in the RUN (GOAL E-1..E-10; ADDENDUM E-12..E-16),
so that the round report is built from one RUN directory only:
`p1e_checks.json`.

Read from what `make p1e-checks` (after `make p1d1-checks`) wrote into the
RUN:

- E-1, E-3, E-5, E-6: `p1e/scene/scene.json` (`scripts/scene.sh`).
- E-2, E-3, E-4, E-5, E-6: `p1e/plugin_e2e/plugin_e2e.json`
  (`scripts/plugin_e2e.sh`, the plugin's command layer against a real
  `rootmark serve`, recording proxy).
- E-7: the chain's local target `p1e/release/release_local.json`
  (`scripts/release_local.sh`) and the four-target release record of the
  commit (`docs/rounds/P1-E/progress/E5_release.json`, `scripts/release.sh`;
  four targets need Docker and other hosts, outside the chain).
- E-9: `regress_vs_p1d1.json` (`eval.compare_p1e regress` against the P1-D.1
  official RUN) and the conformance report (34/34, level 3).
- E-10: `p1e/client.json` (`scripts/client_checks.sh`).
- E-12..E-16 (E0): the P1-D.1 pair re-run record (`reports/rounds/P1-D.1/
  runs/20261016T230508Z-06cd370/pair_recheck.json`), `p1e/check_compare_fields.
  json`, `p1e/e0/e14.json`, `p1e/e0/e15.json`, `p1d1/d1_13.json` (HTTP step)
  and `p1e/tamper_p1e.json`.
- E-8 is the A ↔ B half (`pair_compare.json`, `eval.compare_p1e pair`) and
  E-11 the independent verifier's: both read by the report, not here.

    python -m eval.run_p1e_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

REPO = Path(__file__).resolve().parents[2]
RELEASE_RECORD = REPO / "docs/rounds/P1-E/progress/E5_release.json"
PAIR_RECHECK = REPO / "reports/rounds/P1-D.1/runs/20261016T230508Z-06cd370/pair_recheck.json"


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing (make p1e-checks)")
    return json.loads(path.read_text(encoding="utf-8"))


def _step(rec: dict, n: int) -> dict:
    return next(s for s in rec["steps"] if s["step"] == n)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    scene = _load(d / "p1e/scene/scene.json")
    e2e = _load(d / "p1e/plugin_e2e/plugin_e2e.json")
    rel = _load(d / "p1e/release/release_local.json")
    four = _load(RELEASE_RECORD)
    reg = _load(d / "regress_vs_p1d1.json")
    conf = _load(d / "conformance_p1d1.json")
    client = _load(d / "p1e/client.json")
    cf = _load(d / "p1e/check_compare_fields.json")
    e14 = _load(d / "p1e/e0/e14.json")
    e15 = _load(d / "p1e/e0/e15.json")
    d113 = _load(d / "p1d1/d1_13.json")
    tamper = _load(d / "p1e/tamper_p1e.json")
    recheck = _load(PAIR_RECHECK)
    t_cases = {c["case"]: c for c in tamper["cases"]}
    out = {
        "E-1": {
            "steps": f"{scene['passed']}/{scene['total']}",
            "failed": [s["step"] for s in scene["steps"] if not s["pass"]],
            "pass": scene["pass"] and scene["passed"] == 11 and scene["total"] == 11,
        },
        "E-2": {
            "harness": e2e["harness"],
            "steps": f"{e2e['passed']}/{e2e['total']}",
            "failed": [s["step"] for s in e2e["steps"] if not s["pass"]],
            "pass": e2e["passed"] == 11 and e2e["total"] == 11 and len(e2e["steps"]) == 11,
        },
        "E-3": {
            "scene_md_changes_only_at_propose": scene["e3_md_changes_only_at_propose"],
            "plugin_e2e_md_changes_only_at_propose": e2e["e3_md_changes_only_at_propose"],
            "pass": scene["e3_md_changes_only_at_propose"] and e2e["e3_md_changes_only_at_propose"],
        },
        "E-4": {
            "requests": e2e["e4_access_log"]["requests"],
            "endpoints_used": e2e["e4_access_log"]["endpoints_used"],
            "outside_contract": e2e["e4_access_log"]["outside_contract"],
            "pass": e2e["e4_access_log"]["outside_contract"] == 0
            and e2e["e4_access_log"]["requests"] > 0,
        },
        "E-5": {
            "plugin_promote": _step(e2e, 6)["shows"],
            "scene_promote": _step(scene, 6)["shows"],
            "pass": _step(e2e, 6)["pass"] and _step(scene, 6)["pass"],
        },
        "E-6": {
            "plugin": {"attempt": _step(e2e, 10)["shows"], "unchanged": _step(e2e, 11)["shows"]},
            "scene": {"attempt": _step(scene, 10)["shows"], "unchanged": _step(scene, 11)["shows"]},
            "pass": all(_step(r, n)["pass"] for r in (e2e, scene) for n in (10, 11)),
        },
        "E-7": {
            "chain_local_target": {
                "target": rel["target"],
                "version": rel["version"],
                "spec_version": rel["spec_version"],
                "conformance": rel["conformance"],
                "equal_to_make_conformance": rel["equal_to_make_conformance"],
                "pass": rel["pass"],
            },
            "four_targets": {
                "record": "docs/rounds/P1-E/progress/E5_release.json",
                "commit": four["commit"],
                "targets": {
                    t["target"]: (
                        "ok"
                        if t["ok"]
                        else ("built, check failed" if t["built"] else f"not built: {t['reason']}")
                    )
                    for t in four["targets"]
                },
                "ok": f"{four['ok']}/4",
            },
            "pass": rel["pass"] and four["ok"] == 4,
        },
        "E-9": {
            "regress_vs_p1d1": {
                "reference_run": reg["reference_run"],
                "n_fields": reg["n_fields"],
                "differing_fields": reg["differing_fields"],
                "excluded_differences": reg["excluded_differences"],
                "exclusions_without_difference": reg["exclusions_without_difference"],
                "conformance": reg["conformance"],
                "equal": reg["equal"],
            },
            "conformance": {
                "level_claimed": conf["level_claimed"],
                "passed": conf["passed"],
                "total": conf["total"],
            },
            "pass": reg["equal"]
            and conf["level_claimed"] == 3
            and conf["passed"] == 34
            and conf["total"] == 34,
        },
        "E-10": {
            "tsc_strict": client["tsc_strict"],
            "tsc_eslint_vitest_exit": [
                client["tsc_exit"],
                client["eslint_exit"],
                client["vitest_exit"],
            ],
            "vitest": client["vitest"],
            "bundle_bytes": client["bundle_bytes"],
            "runtime_dependencies": client["runtime_dependencies"],
            "bundle_requires": client["bundle_requires"],
            "pass": client["pass"],
        },
        "E-12": {
            "record": str(PAIR_RECHECK.relative_to(REPO)),
            "exit_code": recheck["exit_code"],
            "n_fields": recheck["pair_compare"]["n_fields"],
            "differing_fields": len(recheck["pair_compare"]["differing_fields"]),
            "pass": recheck["exit_code"] == 0 and recheck["pair_compare"]["equal"],
        },
        "E-13": {"unmatched": cf["unmatched"], "pass": cf["pass"] and cf["unmatched"] == 0},
        "E-14": {"passed": f"{e14['passed']}/{e14['cases']}", "pass": e14["pass"]},
        "E-15": {"passed": f"{e15['passed']}/{e15['cases']}", "pass": e15["pass"]},
        "E-16": {
            "http_step_pass": d113["http"]["pass"],
            "endpoint_removed_rejected": t_cases.get(
                "HTTP endpoint POST /v1/checkpoints removed (MCP still lists the tool)", {}
            ).get("as_expected"),
            "pass": d113["pass"]
            and d113["http"]["pass"]
            and t_cases.get(
                "HTTP endpoint POST /v1/checkpoints removed (MCP still lists the tool)", {}
            ).get("as_expected")
            is True,
        },
        "tamper": {
            "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
            "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
            "pass": tamper["pass"],
        },
    }
    runinfo.write_json(a.run, "p1e_checks.json", out)
    runinfo.record_phase(a.run, "p1e_checks", {}, ["p1e_checks.json"])
    print("; ".join(f"{k} {'pass' if out[k]['pass'] else 'FAIL'}" for k in out))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
