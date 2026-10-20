"""P2-A criteria recorded in the RUN (GOAL P2-A "Önceden kayıtlı çıkış
kriterleri"), so that the round report is built from one RUN directory and
the round's committed records: `p2a_checks.json`. Every criterion names its
source.

- A-1: `p2a/fixture.json` (the `/2` index fixture through the CLI:
  `rename_store`, `index_rebuild` forward and back, byte-equal) and the
  migration tests of `p2a/tests.json` (`p2a_migrate`: every registered
  migration forward and back, `--dry-run` writes nothing, idempotent, the
  `events_hash` guard, refused rollbacks);
- A-2: `p2a/migrate_old.json` (6/6 old vaults; `rootmark conformance`);
- A-3: the status tests of `p2a/tests.json` (`p2a_status`, `contract`) and
  `p1e1/contract.json` (`scripts/contract_check.sh`, run by `make
  p1e1-checks` in this chain);
- A-4: `reports/rounds/P2-A/release_dispatch.json` (the `release.yml`
  dispatch, recorded once: a CI run, not part of a chain);
- A-5: `reports/rounds/P2-A/intel_macos.json` (a report);
- A-6: `propose_kernel::other_spellings_of_existing_names_are_refused` in
  `p2a/tests.json` (this machine) and `reports/rounds/P2-A/ci_linux.json`
  (the Linux CI job);
- A-7: the A0 tests of `p2a/tests.json` (`p2_a_a0`, the two (A) tests of
  `p2_0_2_carry`) and the information measurement
  `reports/rounds/P2-A/info_d_b_test3.json` (present; no threshold);
- A-8: the conformance report `conformance_p1d1.json` (39/39, level 3),
  `scripts/check_frozen.sh` and `scripts/check_deps.sh` exit 0 here; the
  pair's difference is the A ↔ B half (`pair_compare.json`), read by the
  report.
- beside them: the FAIL summary (`p2a/historical.json`, the round's
  classifier = the chain's summary), regress (`regress_vs_p2_0_2.json`)
  and the deliberate cases (`p2a/tamper.json`).

    python -m eval.run_p2a_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from eval import runinfo

REPO = Path(__file__).resolve().parents[2]
ROUND = REPO / "reports/rounds/P2-A"
A1_TESTS = [
    "dry_run_changes_nothing_and_names_the_plan",
    "rename_store_and_index_rebuild_forward_and_back",
    "validations_record_and_roll_back_without_a_byte_changed",
    "a_rollback_refuses_a_damaged_record_or_a_changed_vault",
    "a_migration_that_would_change_events_hash_stops",
    "a_record_naming_paths_outside_its_scope_is_refused",
    "a_record_cannot_widen_its_migration_s_scope",
    "a_rollback_never_follows_a_link_out_of_the_store",
    "the_legacy_flag_still_renames_only",
]
A3_TESTS = [
    "an_older_index_is_migration_required_until_migrated",
    "an_unknown_format_stays_index_stale",
    "http_statuses_of_the_codes",
    "every_code_from_a_real_call",
]
A6_TESTS = ["other_spellings_of_existing_names_are_refused"]
A7_TESTS = [
    "serve_writes_the_canonical_policy_explicitly",
    "unverified_is_the_opt_in_and_behaves_as_in_p2_0_2",
    "a_policy_without_the_table_keeps_the_kernel_default",
    "the_attest_log_tells_refused_matched_and_unmatched_apart",
    "the_attest_log_must_lie_outside_the_vault",
    "a_log_name_linking_into_the_vault_is_refused",
    "a_self_linking_note_renamed_then_rewritten_before_the_watcher_saw_it",
    "a_late_self_link_update_never_vouches_for_new_bytes",
]


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing")
    return json.loads(path.read_text(encoding="utf-8"))


def _tests(tests: dict, names: list[str]) -> dict:
    got = {n: tests["tests"].get(n) for n in names}
    return {
        "tests": got,
        "passed": sum(v is True for v in got.values()),
        "total": len(names),
        "pass": all(v is True for v in got.values()),
    }


def _script(name: str) -> dict:
    r = subprocess.run(
        ["bash", f"scripts/{name}"], cwd=REPO, capture_output=True, text=True, check=False
    )
    return {"exit": r.returncode, "last_line": (r.stdout.strip().splitlines() or [""])[-1][-200:]}


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run)
    tests = _load(d / "p2a/tests.json")
    fixture = _load(d / "p2a/fixture.json")
    old = _load(d / "p2a/migrate_old.json")
    tamper = _load(d / "p2a/tamper.json")
    contract = _load(d / "p1e1/contract.json")
    conf = _load(d / "conformance_p1d1.json")
    hist = _load(d / "p2a/historical.json")
    reg = _load(d / "regress_vs_p2_0_2.json")
    rel = _load(ROUND / "release_dispatch.json")
    intel = _load(ROUND / "intel_macos.json")
    linux = _load(ROUND / "ci_linux.json")
    info = _load(ROUND / "info_d_b_test3.json")
    out = {}
    t1 = _tests(tests, A1_TESTS)
    out["A-1"] = {
        "fixture": {"applied": fixture["applied"], "checks": fixture["checks"]},
        "migration_tests": t1,
        "source": "p2a/fixture.json, p2a/tests.json (p2a_migrate)",
        "pass": fixture["pass"] is True and t1["pass"],
    }
    out["A-2"] = {
        "vaults": [
            {"vault": v["vault"], "applied": v["applied"], "checks": v["checks"], "pass": v["pass"]}
            for v in old["vaults"]
        ],
        "passed": f"{old['passed']}/{len(old['vaults'])}",
        "conformance": old["conformance"],
        "threshold": "6/6",
        "source": "p2a/migrate_old.json",
        "pass": old["pass"] is True,
    }
    t3 = _tests(tests, A3_TESTS)
    out["A-3"] = {
        "status_tests": t3,
        "contract_check": {"pass": contract.get("pass"), "missing": contract.get("missing")},
        "source": "p2a/tests.json (p2a_status, contract), p1e1/contract.json",
        "pass": t3["pass"] and contract.get("pass") is True,
    }
    out["A-4"] = {
        "run": rel["run"],
        "commit": rel["commit"],
        "jobs": {
            t: {"ok": j["ok"], "conformance": j["conformance"]} for t, j in rel["jobs"].items()
        },
        "install": rel["install"],
        "attestations_verified": len(rel["attestations"]),
        "threshold": "2/2 (x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc)",
        "source": "reports/rounds/P2-A/release_dispatch.json",
        "pass": rel["pass"] is True,
    }
    out["A-5"] = {
        "result": intel["result"],
        "decision_request": intel["decision_request"],
        "threshold": "rapor",
        "source": "reports/rounds/P2-A/intel_macos.json",
        "pass": bool(intel.get("result")),
    }
    t6 = _tests(tests, A6_TESTS)
    out["A-6"] = {
        "this_machine": t6,
        "linux_ci": {
            k: linux[k] for k in ("run", "commit", "job", "conclusion", "test") if k in linux
        },
        "source": "p2a/tests.json (propose_kernel), reports/rounds/P2-A/ci_linux.json",
        "pass": t6["pass"] and linux.get("pass") is True,
    }
    t7 = _tests(tests, A7_TESTS)
    out["A-7"] = {
        "tests": t7,
        "information": {
            "value_0_2b": info["value_0_2b"],
            "new_blocks": info["new_blocks"],
            "unverified_after_window": info["unverified_after_window"],
            "commit": info["commit"],
            "threshold": None,
        },
        "source": "p2a/tests.json (p2_a_a0, p2_0_2_carry), reports/rounds/P2-A/info_d_b_test3.json",
        "pass": t7["pass"] and info.get("value_0_2b") is not None,
    }
    frozen, deps = _script("check_frozen.sh"), _script("check_deps.sh")
    out["A-8"] = {
        "conformance": {
            "level_claimed": conf["level_claimed"],
            "passed": conf["passed"],
            "total": conf["total"],
            "spec_version": conf["spec_version"],
        },
        "check_frozen": frozen,
        "check_deps": deps,
        "pair": "A ↔ B: pair_compare.json (eval.compare_p2a pair), read by the report",
        "source": "conformance_p1d1.json, scripts/check_frozen.sh, scripts/check_deps.sh",
        "pass": conf["level_claimed"] == 3
        and conf["passed"] == conf["total"] == 39
        and frozen["exit"] == 0
        and deps["exit"] == 0,
    }
    out["fail_summary"] = {
        "counted_fails": hist["counted_fails"],
        "historical": hist["historical"],
        "migration_required": hist["migration_required"],
        "counted": hist["counted"],
        "reference_run": hist["reference_run"],
        "source": "p2a/historical.json (eval.historical_p2a = the chain's summary)",
        "pass": hist["counted_fails"] == 0,
    }
    out["regress"] = {
        "reference_run": reg["reference_run"],
        "n_fields": reg["n_fields"],
        "differing_fields": reg["differing_fields"],
        "excluded_differences": reg["excluded_differences"],
        "exclusions_without_difference": reg["exclusions_without_difference"],
        "conformance": reg["conformance"],
        "p2_0_2_bytes": reg["p2_0_2_bytes"],
        "source": "regress_vs_p2_0_2.json",
        "pass": reg["equal"] is True,
    }
    out["tamper"] = {
        "cases": {k: c["pass"] for k, c in tamper["cases"].items()},
        "source": "p2a/tamper.json",
        "pass": tamper["pass"] is True,
    }
    runinfo.write_json(a.run, "p2a_checks.json", out)
    runinfo.record_phase(a.run, "p2a_checks", {}, ["p2a_checks.json"])
    print("; ".join(f"{k} {'pass' if v['pass'] else 'FAIL'}" for k, v in out.items()))


if __name__ == "__main__":
    main()
