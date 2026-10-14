"""P1-B.1 checks recorded in the RUN (GOAL B1-1..B1-6), so that the round
report is built from one RUN directory only. Inputs are files `make
p1b1-checks` and `make p1b-checks` wrote into the RUN:

- `v3_compat_mixed.json` (B1-1, scripts/check_v3_compat.sh second output),
- `p1b1_tests.log` (cargo test output of core/tests/compat_v3final.rs and
  core/tests/trace_id_v101.rs: B1-1, B1-2, B1-4, B1-5),
- `conformance.json` (B1-3),
- `regress_vs_p1b.json` (B1-6, eval.compare_p1b1 against the P1-B official RUN).

    python -m eval.run_p1b1_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from eval import runinfo

# test name → criterion (core/tests/compat_v3final.rs, core/tests/trace_id_v101.rs)
TESTS = {
    "B1-1": [
        "agent_only",
        "transcript_only",
        "synthetic_only",
        "human_then_synthetic",
        "agent_then_human_keeps_proposal",
        "human_then_agent_derives_the_drop",
        "human_then_transcript_derives_the_drop",
        "synthetic_then_agent_derives_the_drop",
        "mixed_four_document_vault",
    ],
    "B1-2": [
        "deleted_layer_changed_line_is_an_error",
        "missing_layer_changed_in_a_layered_log_is_an_error",
        "missing_second_layer_changed_is_an_error",
    ],
    "B1-4": ["invalid_inputs_are_refused", "invalid_trace_lines_are_refused_on_open"],
    "B1-5": ["shifted_fields_get_different_ids", "formula_has_separators"],
}


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1b1-checks / make p1b-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def tests(d: Path) -> dict[str, str]:
    p = d / "p1b1_tests.log"
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1b1-checks)")
    out = {}
    for ln in p.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^test (\S+) \.\.\. (ok|FAILED|ignored)$", ln)
        if m:
            out[m.group(1)] = m.group(2)
    return out


def run(run_id: str) -> dict:
    d = runinfo.run_dir(run_id, create=False)
    t = tests(d)
    mixed = _load(d, "v3_compat_mixed.json")
    conf = _load(d, "conformance.json")
    reg = _load(d, "regress_vs_p1b.json")
    by_level = {1: [], 2: []}
    for e in conf["tests"]:
        by_level[e["level"]].append(e)
    crit = {}
    for k, names in TESTS.items():
        res = {n: t.get(n, "missing") for n in names}
        crit[k] = {"tests": res, "pass": all(v == "ok" for v in res.values())}
    crit["B1-1"]["v3_compat_mixed"] = {
        "passed": mixed["passed"],
        "total": mixed["total"],
        "scenarios": [
            {k: s[k] for k in ("scenario", "docs", "blame_equal", "pass")}
            for s in mixed["scenarios"]
        ],
    }
    crit["B1-1"]["pass"] = (
        crit["B1-1"]["pass"] and mixed["pass"] and mixed["passed"] == mixed["total"] == 9
    )
    c207 = next(e for e in conf["tests"] if e["id"] == "C2-07")
    crit["B1-3"] = {
        "spec_version": conf["spec_version"],
        "level_claimed": conf["level_claimed"],
        "C1": f"{sum(e['result'] == 'pass' for e in by_level[1])}/{len(by_level[1])}",
        "C2": f"{sum(e['result'] == 'pass' for e in by_level[2])}/{len(by_level[2])}",
        "C2-07_expected_hash": c207["expected_hash"],
        "fixtures_manifest": conf["fixtures_manifest"],
        "pass": conf["level_claimed"] == 2
        and conf["passed"] == conf["total"] == 25
        and len(by_level[1]) == 17
        and len(by_level[2]) == 8
        and conf["spec_version"] == "1.0.1",
    }
    crit["B1-6"] = {
        "n_fields": reg["n_fields"],
        "differing_excluded": reg["differing_excluded"],
        "differing_outside_exclusions": reg["differing_outside_exclusions"],
        "bytes": {k: v["equal"] for k, v in reg["bytes"].items()},
        "pass": reg["B1-6_equal"],
    }
    out = {"criteria": crit, "tests_seen": len(t)}
    runinfo.write_json(run_id, "p1b1_checks.json", out)
    runinfo.record_phase(run_id, "p1b1_checks", {}, ["p1b1_checks.json"])
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    out = run(a.run)
    print(json.dumps({k: v["pass"] for k, v in out["criteria"].items()}))


if __name__ == "__main__":
    main()
