"""P1-E.1 criteria recorded in the RUN (GOAL P1-E.1 E1-1..E1-6, E1-8, E1-9),
so that the round report is built from one RUN directory only:
`p1e1_checks.json`.

Read from what `make p1e1-checks` (after `make p1e-checks`) wrote into the
RUN:

- E1-1, E1-2: `p1e1/status/e1_1.json`, `e1_2.json`
  (`server/tests/p1e1_status.rs`, the p1d-final `/2` index fixture).
- E1-3, E1-9 (its e2e step): `p1e1/release/plugin_e2e_real.json`
  (`scripts/plugin_e2e_real.sh`, run by `scripts/release.sh`); a record
  that says `skipped` (no Obsidian binary) is reported as skipped and does
  not pass.
- E1-4: `p1e1/release/release_check.json` (`scripts/release_check.sh` on
  the package `scripts/release.sh` built in this chain) against the item
  ids of docs/RELEASE.md's table.
- E1-5: `p1e1/release/release.json` (the required target's record) and the
  R13 line of the check (`.github/workflows/release.yml`); triggering the
  workflow is a human step, so its artefacts are not part of the chain.
- E1-6: `regress_vs_p1e.json` (`eval.compare_p1e1 regress` against the
  P1-E official RUN) and the conformance report (34/34, level 3).
- E1-8: `p1e1/contract.json` (`scripts/contract_check.sh`, status step)
  and the contract-status cases of `p1e1/tamper_p1e1.json`.
- E1-9: the contract check (docs/CLIENT_CONTRACT.md §2/§3 against the
  server) and the e2e connection step.
- tamper: `p1e1/tamper_p1e1.json`, `p1e1/tamper_p1e1_e2e.json`.
- E1-7 is the A ↔ B half (`pair_compare.json`, `eval.compare_p1e1 pair`),
  read by the report, not here.

    python -m eval.run_p1e1_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

REPO = Path(__file__).resolve().parents[2]
RELEASE_MD = REPO / "docs/RELEASE.md"


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing (make p1e1-checks)")
    return json.loads(path.read_text(encoding="utf-8"))


def release_ids() -> list[str]:
    """The item ids of docs/RELEASE.md's checklist table (`| R<n> |`)."""
    ids = re.findall(r"^\| (R\d+a?) \|", RELEASE_MD.read_text(encoding="utf-8"), re.MULTILINE)
    if not ids:
        raise ValueError(f"{RELEASE_MD}: no checklist items")
    return ids


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    e11 = _load(d / "p1e1/status/e1_1.json")
    e12 = _load(d / "p1e1/status/e1_2.json")
    real = _load(d / "p1e1/release/plugin_e2e_real.json")
    rc = _load(d / "p1e1/release/release_check.json")
    rel = _load(d / "p1e1/release/release.json")
    reg = _load(d / "regress_vs_p1e.json")
    conf = _load(d / "conformance_p1d1.json")
    cc = _load(d / "p1e1/contract.json")
    tamper = _load(d / "p1e1/tamper_p1e1.json")
    tamper_e2e = _load(d / "p1e1/tamper_p1e1_e2e.json")
    items = {i["id"]: i for i in rc["items"]}
    ids = release_ids()
    skipped = real.get("skipped") is True
    req = [t for t in rel["targets"] if t.get("required")]
    arm = next((t for t in rel["targets"] if t["target"] == "aarch64-apple-darwin"), {})
    status_cases = [c for c in tamper["cases"] if c["group"] == "contract-status"]
    out = {
        "E1-1": {"passed": e11["passed"], "cases": e11["cases"], "pass": e11["pass"] is True},
        "E1-2": {"passed": e12["passed"], "pass": e12["pass"] is True},
        "E1-3": (
            {"skipped": True, "reason": real.get("reason"), "pass": False}
            if skipped
            else {
                "harness": real["harness"],
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
        ),
        "E1-4": {
            "release_md_items": ids,
            "script_items": sorted(items),
            "every_item_scripted": sorted(ids) == sorted(items),
            "r7": items.get("R7", {}).get("ok"),
            "release_check": f"{rc['passed']}/{rc['total']}",
            "failed": [i for i, v in items.items() if not v["ok"]],
            "pass": sorted(ids) == sorted(items)
            and items.get("R7", {}).get("ok") is True
            and rc["pass"] is True,
        },
        "E1-5": {
            "required_targets": rel["required_targets"],
            "required": {
                k: arm.get(k)
                for k in ("target", "built", "version", "spec_version", "conformance", "ok")
            },
            "sha256_present": bool(re.fullmatch(r"[0-9a-f]{64}", arm.get("sha256") or "")),
            "other_targets": {
                t["target"]: ("ok" if t["ok"] else "skipped" if t.get("skipped") else "failed")
                for t in rel["targets"]
                if not t.get("required")
            },
            "workflow_r13": items.get("R13", {}).get("ok"),
            "workflow_triggered": "human step (docs/RELEASE.md 3), not part of the chain",
            "pass": rel["required_ok"] is True
            and [t["target"] for t in req] == ["aarch64-apple-darwin"]
            and arm.get("ok") is True
            and bool(re.fullmatch(r"[0-9a-f]{64}", arm.get("sha256") or ""))
            and items.get("R4", {}).get("ok") is True
            and items.get("R13", {}).get("ok") is True
            and all(t["ok"] or t.get("skipped") or t.get("built") for t in rel["targets"]),
        },
        "E1-6": {
            "regress_vs_p1e": {
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
        "E1-8": {
            "status_index": [
                {k: s[k] for k in ("state", "status_get", "status_post", "search_status", "ok")}
                for s in cc["http"]["status_index"]
            ],
            "status_index_fields": cc["http"]["status_index_fields"],
            "contract_check_pass": cc["pass"],
            "deliberate": [
                {"case": c["case"], "as_expected": c["as_expected"]} for c in status_cases
            ],
            "pass": cc["pass"] is True
            and len(cc["http"]["status_index"]) == 4
            and all(s["ok"] for s in cc["http"]["status_index"])
            and len(status_cases) >= 3
            and all(c["as_expected"] for c in status_cases),
        },
        "E1-9": {
            "contract_check_pass": cc["pass"],
            "connection_step": None if skipped else real["e1_9_connection"]["shows"],
            "skipped": skipped,
            "pass": cc["pass"] is True and not skipped and real["e1_9_connection"]["pass"] is True,
        },
        "tamper": {
            "rejected": f"{tamper['rejected'] + tamper_e2e['rejected']}/{tamper['rejected_total'] + tamper_e2e['rejected_total']}",
            "controls": f"{tamper['controls_ok'] + tamper_e2e['controls_ok']}/{tamper['controls_total'] + tamper_e2e['controls_total']}",
            "real_cases_run": tamper_e2e["real_cases_run"],
            "pass": tamper["pass"] is True and tamper_e2e["pass"] is True,
        },
    }
    runinfo.write_json(a.run, "p1e1_checks.json", out)
    runinfo.record_phase(a.run, "p1e1_checks", {}, ["p1e1_checks.json"])
    print("; ".join(f"{k} {'pass' if out[k]['pass'] else 'FAIL'}" for k in out))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
