"""P2-V tools (GOAL P2-V V2, V3): the tier 2 comparison's naming rules, the
tier 2 FAIL count, the host-value checker and the Mac set's formulas."""

import json
import re
import tempfile
from pathlib import Path

import pytest

from eval import compare_p2r, historical_p2r, portable, tier2, verify_mac

REPO = Path(__file__).resolve().parents[2]
P2R_RUN = REPO / "reports/rounds/P2-R/runs/20261026T222501Z-5b235b0"
INV = {
    "pair_exclusions": [
        {"id": "Z1", "file": "p1d/d10.json", "field": None, "class": "Z", "why": "w"},
        {"id": "Z3", "file": "p1d_checks.json", "field": "D-10", "class": "Z", "why": "w"},
        {"id": "Z7", "file": "p2_0_2_checks.json", "field": "H-P2-0·2a", "class": "Z", "why": "w"},
    ],
    "byte_exclusions": [{"id": "Z28", "path": "p2_0_2/d_a/events.jsonl", "class": "Z", "why": "w"}],
    "host_fields": [],
    "commit_bound": [],
    "tier2_set_aside_criteria": [
        {"file": "p1d_checks.json", "criterion": "D-10", "class": "Z", "mac_set": "Z-D10"}
    ],
}


def test_a_named_field_sets_its_subtree_aside_and_nothing_else():
    ex = INV["pair_exclusions"]
    assert tier2.set_aside(ex, "p1d/d10.json", "p95_ms")["id"] == "Z1"
    assert tier2.set_aside(ex, "p1d_checks.json", "D-10›pass")["id"] == "Z3"
    assert tier2.set_aside(ex, "p1d_checks.json", "D-10")["id"] == "Z3"
    # a prefix of a name is not the name
    assert tier2.set_aside(ex, "p1d_checks.json", "D-100›pass") is None
    assert tier2.set_aside(ex, "p1d_checks.json", "D-1›pass") is None
    assert tier2.set_aside(ex, "p2_0_2_checks.json", "H-P2-0·2a′›pass") is None


def test_an_embedded_difference_follows_the_field_it_names_recursively():
    ex = INV["pair_exclusions"]
    k = "regress›differing_fields›p1d1_checks·json:D1-12›regress_vs_p1d›differing_fields›p1d/d10·json:pass›a"
    e = tier2.set_aside(ex, "p2a_checks.json", k)
    assert e is not None and e["id"] == "Z1" and "via" in e
    k2 = "regress›differing_fields›p1d1_checks·json:D1-12›regress_vs_p1d›differing_fields›p1d_checks·json:D-9›pass"
    assert tier2.set_aside(ex, "p2a_checks.json", k2) is None
    assert tier2.set_aside(ex, "p2a_checks.json", "regress›n_fields") is None


def test_a_pattern_in_the_inventory_is_an_error(tmp_path):
    bad = json.loads(json.dumps(INV))
    bad["pair_exclusions"].append({"id": "Z9", "file": "p2_0_2/*.json", "field": None, "why": "w"})
    p = tmp_path / "inv.json"
    p.write_text(json.dumps(bad))
    with pytest.raises(tier2.InventoryError):
        tier2.load_inventory(p)
    nowhy = json.loads(json.dumps(INV))
    nowhy["pair_exclusions"][0]["why"] = ""
    p.write_text(json.dumps(nowhy))
    with pytest.raises(tier2.InventoryError):
        tier2.load_inventory(p)


def test_the_pair_verdict_counts_every_difference_the_inventory_does_not_name(monkeypatch):
    def fake(a, b):
        return {
            "files": ["p1d/d10.json", "p1c_checks.json"],
            "n_fields": 10,
            "differing_fields": {"p1d/d10.json:p95_ms": 1.0, "p1d_checks.json:D-10›pass": 1.0},
            "bytes": {
                "p2_0_2/d_a/events.jsonl": {"equal": False},
                "governance/vault/x": {"equal": True},
            },
        }

    monkeypatch.setattr(compare_p2r, "pair", fake)
    res = tier2.pair(Path("a"), Path("b"), INV)
    assert res["equal"] and res["exclusions_used"] == ["Z1", "Z28", "Z3"]
    assert len(res["not_compared"]) == 4

    def fake2(a, b):
        r = fake(a, b)
        r["differing_fields"]["p1c_checks.json:C-3›pass"] = 1.0
        r["bytes"]["governance/vault/x"] = {"equal": False}
        return r

    monkeypatch.setattr(compare_p2r, "pair", fake2)
    res = tier2.pair(Path("a"), Path("b"), INV)
    assert not res["equal"]
    assert list(res["differing_fields"]) == ["p1c_checks.json:C-3›pass"]
    assert res["differing_bytes"] == ["governance/vault/x"]


def test_a_regress_fail_is_set_aside_only_when_every_difference_is_a_z_field():
    rows = [
        {"file": "p1d_checks.json", "criterion": "D-10", "label": "fail"},
        {
            "file": "p1d1_checks.json",
            "criterion": "D1-12",
            "label": "fail",
            "differing": [
                "regress_vs_p1d/differing_fields/p1d/d10.json:pass",
                "regress_vs_p1d/differing_fields/p1d_checks.json:D-10›pass",
            ],
        },
        {
            "file": "p1e_checks.json",
            "criterion": "E-9",
            "label": "fail",
            "differing": [
                "regress_vs_p1d1/differing_fields/p1d/d10.json:pass",
                "regress_vs_p1d1/differing_fields/p1c_checks.json:C-3›pass",
            ],
        },
        {"file": "p1c_checks.json", "criterion": "C-3", "label": "historical"},
    ]
    s = {
        "fails": rows,
        "counted": ["p1d_checks.json:D-10", "p1d1_checks.json:D1-12", "p1e_checks.json:E-9"],
    }
    res = tier2.fails(Path("run"), s, INV)
    assert [c["criterion"] for c in res["counted"]] == ["p1e_checks.json:E-9"]
    assert {a["criterion"]: a["class"] for a in res["set_aside"]} == {
        "p1d_checks.json:D-10": "Z",
        "p1d1_checks.json:D1-12": "türev",
    }
    s["counted"] = ["p1d_checks.json:D-10"]
    with pytest.raises(ValueError):
        tier2.fails(Path("run"), s, INV)


def test_the_committed_inventory_names_every_exclusion_with_a_reason_and_a_source():
    inv = tier2.load_inventory()
    ids = [
        e["id"]
        for g in ("pair_exclusions", "byte_exclusions", "host_fields", "commit_bound")
        for e in inv[g]
    ]
    assert len(ids) == len(set(ids))
    for s in inv["mac_sets"]:
        for c in s["criteria"]:
            f, _, line = c["source"].partition(":")
            if line:
                text = (REPO / f).read_text(encoding="utf-8").splitlines()
                assert c["criterion"] in text[int(line) - 1], c
            if c["threshold"] is None:
                assert c["why_no_threshold"], c
    keys = {f"{s['file']}:{s['criterion']}" for s in inv["tier2_set_aside_criteria"]}
    crit = {x["id"] for x in inv["criteria"]}
    # every set-aside criterion the chain records is one of the chain's ids
    assert keys - crit <= {"p2_0_2_checks.json:H-P2-0.2i", "p2_0_3_checks.json:H-P2-0.3i"}


def test_the_host_checker_finds_an_injected_work_path_and_not_a_word():
    with tempfile.TemporaryDirectory() as t:
        d = Path(t)
        work = "/Users/someone/work/_temp/v/A"
        (d / "clean.json").write_text(
            json.dumps({"url": "https://github.com/someonesensei/x", "p": "spec/fixtures"})
        )
        (d / "dirty.json").write_text(
            json.dumps({"detail": f"no archive in {work}/repo/target/dist/bin"})
        )
        pats = portable.needles(
            [
                {
                    "workdir": work,
                    "home": "/Users/someone",
                    "user": "someone",
                    "host": "runner-mac-17.local",
                }
            ]
        )
        hits = portable.scan(d, ["clean.json", "dirty.json"], pats)
        assert {h["file"] for h in hits} == {"dirty.json"}
        assert {h["kind"] for h in hits} >= {"path"}
        (d / "host.json").write_text(json.dumps({"seen_on": "runner-mac-17"}))
        assert [h["kind"] for h in portable.scan(d, ["host.json"], pats)] == ["host"]
        with pytest.raises(FileNotFoundError):
            portable.scan(d, ["absent.json"], pats)
        missing = []
        assert portable.scan(d, ["absent.json"], pats, missing) == [] and missing == ["absent.json"]


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_the_mac_set_applies_the_frozen_recorders_formulas():
    # the P2-R 3rd attempt's chain A: every Mac criterion equals its recorder's
    with tempfile.TemporaryDirectory() as t:
        v = Path(t)
        for f in P2R_RUN.iterdir():
            (v / f.name).symlink_to(f)
        out = verify_mac.rep(v)
    for k, rec in out.items():
        f, cid = k.split(":", 1)
        doc = json.loads((P2R_RUN / f).read_text(encoding="utf-8"))
        ref = doc.get(cid)
        if ref is None:
            assert re.fullmatch(r"H-P2-0\.[23]i", cid), k
            continue
        if cid == "tamper":
            # the Mac set measures the real-Obsidian half (tamper_p1e1.json is tier 2's)
            e2e = json.loads((P2R_RUN / "p1e1/tamper_p1e1_e2e.json").read_text(encoding="utf-8"))
            assert rec["pass"] == e2e["pass"], k
            continue
        if cid == "H-P2-0.2d":
            # the Mac set measures its D-C half (D-C′ is H-P2-0.3d)
            assert rec["pass"] == ref["d_c"]["pass"], k
            continue
        assert rec["pass"] == ref["pass"], k


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_tier2_counts_no_fail_of_the_p2r_attempt_outside_the_z_and_o_sets():
    # P2-V's inventory, the one this test was written for (P2-V.1/F2 made
    # P2-V.1's the default; its B3 criterion is test_p2v1_tools')
    p2v = tier2.load_inventory(REPO / "reports/rounds/P2-V/inventory.json")
    res = tier2.fails(P2R_RUN, historical_p2r.classify(P2R_RUN), p2v)
    assert res["classifier_counted"] == 5 and res["counted_fails"] == 0


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_a_stopped_real_obsidian_harness_fails_its_sets_and_keeps_the_others(tmp_path):
    log = tmp_path / "p2_0_3.log"
    log.write_text(
        "Error: in Obsidian: roasting.md was not written by Obsidian within 2 s of the save\n"
    )
    with tempfile.TemporaryDirectory() as t:
        v = Path(t)
        for f in P2R_RUN.iterdir():
            (v / f.name).symlink_to(f)
        out = verify_mac.rep(v, {"p2_0_3": str(log)})
    for k in ("b", "b-yük", "c", "d", "e"):
        rec = out[f"p2_0_3_checks.json:H-P2-0.3{k}"]
        assert rec["pass"] is False and "roasting.md" in rec["log_tail"][-1]
    # the P2-0.2 records' criteria are measured as before
    assert (
        out["p2_0_3_checks.json:H-P2-0.3a"]["pass"] == out["p2_0_2_checks.json:H-P2-0.2a"]["pass"]
    )
    assert out["p2_0_3_checks.json:H-P2-0.3f"]["pass"] is True
    assert out["p1d_checks.json:D-10"]["pass"] is False


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_the_mac_summary_needs_every_thresholded_criterion_in_three_of_three(tmp_path):
    with tempfile.TemporaryDirectory() as t:
        v = Path(t)
        for f in P2R_RUN.iterdir():
            (v / f.name).symlink_to(f)
        crit = verify_mac.rep(v)
    for i in (1, 2, 3):
        (tmp_path / "mac" / f"rep{i}").mkdir(parents=True)
        rec = {"run": f"r{i}", "rep": i, "elapsed_s": 10, "errors": [], "criteria": crit}
        (tmp_path / "mac" / f"rep{i}" / "rep.json").write_text(json.dumps(rec))
    res = verify_mac.summary(tmp_path, 3, tier2.load_inventory())
    by = {s["id"]: s for s in res["sets"]}
    # the P2-R attempt: D-10 and D-A failed there, D-B test-4 too (0.3b, K-P2-03.1)
    assert (
        by["Z-D10"]["pass"] is False
        and by["Z-DA"]["pass"] is False
        and by["O-DB4"]["pass"] is False
    )
    assert by["Z-DB2"]["pass"] is True and by["O-C18"]["pass"] is True
    # E-7 has been 2/4 since K-E5-T: O-REL holds every criterion but E-7, E1-4
    rel = {c["criterion"]: c["passed"] for c in by["O-REL"]["criteria"]}
    assert rel["p1e1_checks.json:E1-3"] == "3/3" and rel["p1e_checks.json:E-7"] == "0/3"
    assert by["O-REL"]["pass"] is False
    assert "p2_0_2_checks.json:H-P2-0.2i" in res["no_threshold"] and not res["pass"]
    nt = next(c for c in by["Z-DA"]["criteria"] if c["criterion"].endswith("H-P2-0.2i"))
    assert nt["pass"] is None and nt["equal_across_reps"] is True


def test_a_frozen_result_is_read_only_for_the_runs_it_compared(tmp_path):
    a, b = tmp_path / "A" / "run", tmp_path / "B" / "run"
    a.mkdir(parents=True)
    b.mkdir(parents=True)
    res = {
        "run_a": str(a),
        "run_b": str(b),
        "files": ["p1d/d10.json"],
        "n_fields": 3,
        "differing_fields": {"p1d/d10.json:p95_ms": 1.0},
        "bytes": {},
    }
    f = tmp_path / "frozen.json"
    f.write_text(json.dumps(res))
    out = tier2.pair(a, b, INV, tier2.frozen_result(f, a, b))
    assert out["equal"] and out["exclusions_used"] == ["Z1"]
    with pytest.raises(ValueError):
        tier2.frozen_result(f, b, a)
    (a / "p1d").mkdir()
    (a / "p1d/d10.json").write_text("{}")
    assert portable.record_files(a, False, files_from=f) == ["p1d/d10.json"]
    with pytest.raises(FileNotFoundError):
        portable.record_files(tmp_path, False, files_from=f)
