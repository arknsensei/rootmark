"""P2-0 run comparisons (TASKS Z7; H-P2-0g), with the frozen leaf extraction
and field comparison (`compare_runs.leaves`, `metrics.max_relative_difference`)
and the frozen P1-E.1 comparator (`compare_p1e1`).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): the P1-E.1 pair set
(`compare_p1e1.pair`: everything since P1-A, fields and bytes) and the P2-0
result files (`P2_0_FILES`), every field except the wall-clock ones
(`P2_0_TIMING`: what the real-Obsidian D-B sessions measure — the
false-unverified rate, late raises, latency — and the records of them;
declared in research/eval/DETERMINISTIC_FIELDS.md), and the D-A and D-C
vault logs byte for byte (`P2_0_BYTES`: `events.jsonl` and
`attestations.jsonl`, H-P2-0g). `p2_0/d_b.json` is a wall-clock record as a
whole and is not compared. No other exclusion.

`regress <p1e1_run> <run>`: the P1-E.1 pair set of files (P1-C.1 set +
P1-D, P1-D.1, P1-E and P1-E.1 files) of the P1-E.1 official RUN against
this RUN, every field (the earlier comparators' own wall-clock and
informational fields aside), **except** the subtrees of `EXCLUDED` — each a
difference that a P2-0 decision causes, written from a development RUN
before the official RUN (precedent K-C1.6, compare_p1e1.EXCLUDED) — and
the conformance report by test id: the 34 tests of the reference equal,
the three new ones (C2-11, C2-12, C3-08) passed; spec 1.0.3 → 2.0.0 and 34
→ 37 are the P2-0 decision (K-P2-0.1). A difference outside the exclusions
fails; every excluded leaf that differs is listed, and so is every
exclusion that matched no difference.

    python -m eval.compare_p2_0 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2_0 regress <p1e1_run> <run> [--out path.json]

Exit code: 0 when nothing differs (outside the exclusions), 3 when
something does (the comparison is still written), 1 on a missing file or
field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p1d as P
from eval import compare_p1d1 as D1
from eval import compare_p1e as E
from eval import compare_p1e1 as E1
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P2_0 = Path("p2_0")
P2_0_FILES = [
    "p2_0_checks.json",
    "p2_0/tuning.json",
    "p2_0/d_e.json",
    "p2_0/d_a.json",
    "p2_0/d_c.json",
    "p2_0/tamper_p2_0.json",
]
# wall-clock fields of the P2-0 files (file, path pattern; "*" = any key/index):
# the real-Obsidian D-B measurements (H-P2-0b, b′, c) as recorded in
# p2_0_checks.json, and the D-B tuning rows of the committed tuning record
# (copied, but measured on a wall clock)
P2_0_TIMING: dict[str, list[tuple]] = {
    "p2_0_checks.json": [("H-P2-0b",), ("H-P2-0b′",), ("H-P2-0c",)],
    "p2_0/tuning.json": [("windows",)],
}
P2_0_INFO: dict[str, list[tuple]] = {}
P2_0_BYTES = [
    P2_0 / "d_a" / "events.jsonl",
    P2_0 / "d_a" / "attestations.jsonl",
    P2_0 / "d_c" / "events.jsonl",
    P2_0 / "d_c" / "attestations.jsonl",
]
CONFORMANCE = E.CONFORMANCE
NEW_TESTS = ["C2-11", "C2-12", "C3-08"]

# P1-E.1 official RUN → this RUN: subtrees whose difference a P2-0 decision
# causes, `(file, path prefix, decision, why)`; written from the development
# RUN before the official RUN (Z7 record); development RUN
# 20261018T125648Z-78b5119 (Z7.md): every differing leaf under a row below
# (narrowest subtree or leaf, no wildcard)
EXCLUDED: list[tuple[str, tuple, str, str]] = [
    (
        "conformance_p1d1.json",
        ("fixtures_manifest_hash",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "conformance_p1d1.json",
        ("passed",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "conformance_p1d1.json",
        ("spec_version",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "conformance_p1d1.json",
        ("total",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "conformance_p1d1.json",
        ("tests",),
        "K-P2-0.4",
        "the report holds three new tests (by-id comparison: the 34 of the reference equal, the three new pass)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("fixtures_manifest_hash",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("passed",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("spec_version",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("total",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("tests",),
        "K-P2-0.4",
        "the report holds three new tests (by-id comparison: the 34 of the reference equal, the three new pass)",
    ),
    (
        "p1b1_checks.json",
        ("criteria", "B1-3", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1b_checks.json",
        ("fixtures_manifest_hash",),
        "K-P2-0.4",
        "fixture manifest: C2-11, C2-12, C3-08",
    ),
    (
        "p1c1_checks.json",
        ("C1-7", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1c_checks.json",
        ("C-1", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1d/scene/scene.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    ("p1d1_checks.json", ("D1-10", "pass"), "K-P2-0.4", "D1-10 expects 34 tests and spec 1.0.3"),
    ("p1d1_checks.json", ("D1-10", "passed"), "K-P2-0.4", "D1-10 expects 34 tests and spec 1.0.3"),
    (
        "p1d1_checks.json",
        ("D1-10", "spec_version"),
        "K-P2-0.4",
        "D1-10 expects 34 tests and spec 1.0.3",
    ),
    (
        "p1d1_checks.json",
        ("D1-12", "regress_vs_p1d", "conformance", "spec_version", "1"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1d_checks.json",
        ("D-9", "conformance", "spec_version", "1"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/plugin_e2e/plugin_e2e.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("passed",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("spec_version",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("total",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/release_local.json",
        ("conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/release_local.json",
        ("conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/release/release_local.json",
        ("spec_version",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/scene/scene.json",
        ("steps", "0", "shows", "mcp_spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e/scene/scene.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1/release/release.json",
        ("spec_version",),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1/release/release.json",
        ("commit",),
        "K-P2-0.1",
        "the release package of this round's commit (the reference is P1-E.1's)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "3", "item"),
        "K-P2-0.1",
        "R4a's item names the commit of the package (the reference is P1-E.1's)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "1", "item"),
        "K-P2-0.4",
        "the checklist item names spec 2.0.0 / 37 (F9)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "4", "item"),
        "K-P2-0.4",
        "the checklist item names spec 2.0.0 / 37 (F9)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "9", "item"),
        "K-P2-0.4",
        "the checklist item names spec 2.0.0 / 37 (F9)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "pass"),
        "K-P2-0.4",
        "E1-6 expects 34/34 and regresses against P1-E: it sees the P2-0 differences",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "regress_vs_p1e"),
        "K-P2-0.4",
        "E1-6's regress against P1-E sees every P2-0 difference (this regress lists them by decision)",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "spec_version"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "conformance", "passed"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "conformance", "total"),
        "K-P2-0.4",
        "spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "regress_vs_p1d1"),
        "K-P2-0.4",
        "E-9's regress against P1-D.1 sees every P2-0 difference",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "#len"),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "14", "detail"),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "14", "item"),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "16"),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1/release/release_check.json",
        ("total",),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1_checks.json",
        ("E1-4", "release_check"),
        "K-P2-0.3",
        "R14 rewritten, R16 added (17 items)",
    ),
    (
        "p1e1_checks.json",
        ("E1-4", "release_md_items", "#len"),
        "K-P2-0.3",
        "R16 in docs/RELEASE.md",
    ),
    ("p1e1_checks.json", ("E1-4", "release_md_items", "16"), "K-P2-0.3", "R16 in docs/RELEASE.md"),
    ("p1e1_checks.json", ("E1-4", "script_items"), "K-P2-0.3", "R16 scripted"),
    (
        "p1d1/d1_13.json",
        ("http", "status_fields"),
        "K-P2-0.3",
        "contract_check.sh's status step reads every field of the §2 status row",
    ),
    (
        "p1d1/d1_13.json",
        ("http", "http_only"),
        "K-P2-0.5",
        "contract_check.sh's §2.6 step: attest_session, attest over HTTP, absent from MCP",
    ),
    (
        "p1e1/contract.json",
        ("http", "status_fields"),
        "K-P2-0.3",
        "contract_check.sh's status step reads every field of the §2 status row",
    ),
    (
        "p1e1/contract.json",
        ("http", "http_only"),
        "K-P2-0.5",
        "contract_check.sh's §2.6 step: attest_session, attest over HTTP, absent from MCP",
    ),
    (
        "p1e/client.json",
        ("bundle_bytes",),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e/client.json",
        ("vitest", "files"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e/client.json",
        ("vitest", "passed"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e/client.json",
        ("vitest", "tests"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e_checks.json",
        ("E-10", "bundle_bytes"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e_checks.json",
        ("E-10", "vitest", "files"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e_checks.json",
        ("E-10", "vitest", "passed"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e_checks.json",
        ("E-10", "vitest", "tests"),
        "K-P2-0.5",
        "the plugin attests its own saves (Z3): bundle and unit tests",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "8", "detail"),
        "K-P2-0.5",
        "R8 records the plugin bundle size (Z3)",
    ),
    (
        "p1e1/tamper_p1e1.json",
        ("cases", "1", "as_expected"),
        "K-P2-0.5",
        "the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14)",
    ),
    (
        "p1e1/tamper_p1e1.json",
        ("cases", "1", "detail", "as_expected"),
        "K-P2-0.5",
        "the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14)",
    ),
    (
        "p1e1/tamper_p1e1.json",
        ("pass",),
        "K-P2-0.5",
        "the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14)",
    ),
    (
        "p1e1/tamper_p1e1.json",
        ("rejected",),
        "K-P2-0.5",
        "the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-8", "deliberate", "1", "as_expected"),
        "K-P2-0.5",
        "the same p1e-final case (F14)",
    ),
    ("p1e1_checks.json", ("E1-8", "pass"), "K-P2-0.5", "the same p1e-final case (F14)"),
    ("p1e1_checks.json", ("tamper", "pass"), "K-P2-0.5", "the same p1e-final case (F14)"),
    ("p1e1_checks.json", ("tamper", "rejected"), "K-P2-0.5", "the same p1e-final case (F14)"),
    (
        "p1e/check_compare_fields.json",
        ("fields_declared",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("fields_in_code",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("file_constants",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "#len"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "9"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "10"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("fields_declared",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("fields_in_code",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("file_constants",),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "#len"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "9"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "10"),
        "F13",
        "the field audit sees the new comparator compare_p2_0 (its module, rows and file constants)",
    ),
    (
        "p1e1/historical.json",
        ("counted_fails",),
        "K-P2-0.4",
        "eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them",
    ),
    (
        "p1e1/historical.json",
        ("counted",),
        "K-P2-0.4",
        "eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them",
    ),
    (
        "p1e1/historical.json",
        ("fails",),
        "K-P2-0.4",
        "eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them",
    ),
    (
        "p1e1/historical.json",
        ("historical",),
        "K-P2-0.4",
        "eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them",
    ),
    (
        "p1e1/status/e1_1.json",
        ("fixture",),
        "K-GH.1",
        "the fixture label after the project rename (GH-0, after the P1-E.1 official RUN)",
    ),
]


def _match(pats: list[tuple], path: tuple) -> bool:
    return any(
        len(path) >= len(pat)
        and all(p == "*" or p == str(x) for p, x in zip(pat, path, strict=False))
        for pat in pats
    )


def _skip(name: str, path: tuple) -> bool:
    return (
        E1._skip(name, path)
        or _match(P2_0_TIMING.get(name, []), path)
        or _match(P2_0_INFO.get(name, []), path)
    )


def _excl_match(name: str, path: tuple) -> tuple | None:
    for e in EXCLUDED:
        f, pre = e[0], e[1]
        if (
            f == name
            and len(path) >= len(pre)
            and all(p == "*" or p == str(x) for p, x in zip(pre, path, strict=False))
        ):
            return e
    return None


def compare_files(a: Path, b: Path, files: list[str], exclude) -> dict:
    """`compare_p1e1.compare_files` with the P2-0 skips."""
    n = 0
    differing: dict = {}
    excluded: dict = {}
    for name in files:
        la = {p: v for p, v in C.leaves(P._load(a, name)).items() if not _skip(name, p)}
        lb = {p: v for p, v in C.leaves(P._load(b, name)).items() if not _skip(name, p)}
        for p in sorted(set(la) | set(lb), key=C._key):
            e = exclude(name, p)
            va, vb = la.get(p, "absent"), lb.get(p, "absent")
            if e is not None:
                if va != vb:
                    k = f"{e[0]}:{'/'.join(map(str, e[1]))}"
                    excluded.setdefault(k, {"decision": e[2], "leaves": 0})
                    excluded[k]["leaves"] += 1
                la.pop(p, None)
                lb.pop(p, None)
                continue
            if p not in la or p not in lb:
                differing[f"{name}:{C._key(p)}"] = {"a": va, "b": vb}
                la.pop(p, None)
                lb.pop(p, None)
        keys = sorted(la, key=C._key)
        fa = {C._key(p): la[p] for p in keys}
        fb = {C._key(p): lb[p] for p in keys}
        r = M.max_relative_difference(fa, fb, sorted(fa))
        n += r["n_fields"]
        for k, v in r["differing_fields"].items():
            differing[f"{name}:{k}"] = v
    return {"n_fields": n, "differing_fields": differing, "excluded_differences": excluded}


def p2_0_bytes(a: Path, b: Path) -> dict:
    out = {}
    for f in P2_0_BYTES:
        pa, pb = a / f, b / f
        for p in (pa, pb):
            if not p.is_file():
                raise FileNotFoundError(f"{p} missing")
        out[f.as_posix()] = {
            "blake3": file_blake3(pa),
            "bytes": pa.stat().st_size,
            "equal": pa.read_bytes() == pb.read_bytes(),
        }
    return out


def pair(a: Path, b: Path) -> dict:
    res = E1.pair(a, b)
    q = compare_files(a, b, P2_0_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P2_0_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    pb = p2_0_bytes(a, b)
    res["bytes"].update(pb)
    res["h_p2_0g"] = {"files": pb, "equal": all(v["equal"] for v in pb.values())}
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    files = [*P1C1_FILES, *P.P1D_FILES, *D1.P1D1_FILES, *E.P1E_FILES, *E1.P1E1_FILES]
    res = compare_files(ref, run, files, _excl_match)
    ca, cb = P._load(ref, CONFORMANCE), P._load(run, CONFORMANCE)
    by_a = {t["id"]: t for t in P._tests(ca)}
    by_b = {t["id"]: t for t in P._tests(cb)}
    new = {i: by_b.get(i) for i in NEW_TESTS}
    conf = {
        "tests": len(by_b),
        "reference_tests": len(by_a),
        "differing_tests": sorted(i for i in by_a if by_a[i] != by_b.get(i)),
        "unexpected_new_tests": sorted(set(by_b) - set(by_a) - set(NEW_TESTS)),
        "new_tests_passed": {i: (t or {}).get("result") == "pass" for i, t in new.items()},
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "passed": [ca["passed"], cb["passed"]],
    }
    conf["equal"] = (
        not conf["differing_tests"]
        and not conf["unexpected_new_tests"]
        and all(v is True for v in conf["new_tests_passed"].values())
        and ca["level_claimed"] == cb["level_claimed"] == 3
        and cb["spec_version"] == "2.0.0"
        and cb["passed"] == ca["passed"] + len(NEW_TESTS)
    )
    res["conformance"] = conf
    res["reference_run"] = ref.name
    res["files"] = files
    keyed = [f"{e[0]}:{'/'.join(map(str, e[1]))}" for e in EXCLUDED]
    res["exclusions"] = [
        {"file": e[0], "path": "/".join(map(str, e[1])), "decision": e[2], "why": e[3]}
        for e in EXCLUDED
    ]
    res["exclusions_without_difference"] = [
        k for k in keyed if k not in res["excluded_differences"]
    ]
    res["equal"] = not res["differing_fields"] and conf["equal"]
    return res


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["pair", "regress"])
    ap.add_argument("a", type=Path)
    ap.add_argument("b", type=Path)
    ap.add_argument("--out", type=Path)
    x = ap.parse_args(argv)
    try:
        res = pair(x.a, x.b) if x.mode == "pair" else regress(x.a, x.b)
    except (FileNotFoundError, M.MetricError, json.JSONDecodeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(C._finite(res))
    if x.out:
        x.out.write_text(text)
    print(text, end="")
    if not res["equal"]:
        print(f"{x.mode} FAILED: differing {sorted(res['differing_fields'])}", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
