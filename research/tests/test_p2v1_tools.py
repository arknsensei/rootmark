"""P2-V.1/E3 (FREEZE_EXCEPTIONS F2): tier 2 with P2-V.1's inventory — items
by id (B6), the derived criterion (B3), a Z/O criterion's narrowed names
(B2), the full-list derived-FAIL rule (B1), the E1 names (equiv only)."""

import json
from pathlib import Path

import pytest

from eval import historical_p2r, tier2

REPO = Path(__file__).resolve().parents[2]
P2R_RUN = REPO / "reports/rounds/P2-R/runs/20261026T222501Z-5b235b0"
INV = tier2.load_inventory()


def test_p2v1_inventory_carries_every_p2v_entry():
    p2v = json.loads((REPO / "reports/rounds/P2-V/inventory.json").read_text())
    for g in (
        "pair_exclusions",
        "byte_exclusions",
        "host_fields",
        "commit_bound",
        "tier2_set_aside_criteria",
    ):
        ids = {e.get("id", (e.get("file"), e.get("criterion"))) for e in INV[g]}
        for e in p2v[g]:
            assert e.get("id", (e.get("file"), e.get("criterion"))) in ids, (g, e)
    assert {e["id"] for e in INV["commit_bound"]} >= {"C1", "C2", "C3"}


def test_an_item_is_resolved_by_its_id_in_each_record(tmp_path):
    run = tmp_path / "run"
    (run / "p1e1/release").mkdir(parents=True)
    items = [{"id": f"R{i}", "item": "x", "ok": True, "detail": ""} for i in range(1, 21)]
    items.insert(3, {"id": "R4a", "item": "inputs of abc = HEAD", "ok": True, "detail": ""})
    (run / "p1e1/release/release_check.json").write_text(json.dumps({"items": items}))
    got = {e["id"]: e["field"] for e in tier2.resolve(INV["commit_bound"], run) if e.get("item")}
    assert got["C2"] == "items›3›item"
    assert got["C3"] == "items›14›detail"  # R14 after R4a: position 14
    (run / "p1e1/release/release_check.json").write_text(json.dumps({"items": items[:3]}))
    with pytest.raises(tier2.InventoryError):
        tier2.resolve(INV["commit_bound"], run)


def test_c1_8_sets_aside_only_its_named_fields():
    named = tier2._criterion_fields(INV)
    c18 = [e for e in named if e["id"] == "p1c1_checks.json:C1-8"]
    assert c18 and all(e["field"] != "C1-8" for e in c18)
    # a ranking's id inside C1-8's regress is not set aside; its flat score is
    names = [*named, *INV["e1_fields"]]
    k = "C1-8›regress›differing_fields›p1c_index·json:rankings_200›flat›0›0›3"
    assert tier2.set_aside(names, "p1c1_checks.json", k) is not None
    k_id = "C1-8›regress›differing_fields›p1c_index·json:rankings_200›flat›0›0›1"
    assert tier2.set_aside(names, "p1c1_checks.json", k_id) is None
    # other set-aside criteria keep their whole record (P2-V)
    assert any(e["field"] == "D-10" for e in named)


def test_a_classifier_path_with_a_difference_record_leaf_is_read_as_its_field():
    names = [*tier2._criterion_fields(INV), *INV["e1_fields"]]
    p = "differing_fields/p1c1_checks.json:C1-8›regress›differing_fields›c3_quality·json:index›embedder_id/a"
    assert tier2._derived_path(names, "p1d_checks.json", "D-9", p)
    own = "regress_vs_p1d/excluded_differences/p1d_checks.json:D-9/leaves"
    assert tier2._derived_path(names, "p1d1_checks.json", "D1-12", own)
    assert not tier2._derived_path(
        names, "p1d_checks.json", "D-9", "differing_fields/p1d/d2.json:x"
    )


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_the_derived_tamper_criterion_is_counted_as_registered():
    # B3 as pre-registered (inventory `derived_criteria`): tamper_p1e1.json's
    # `pass`; in this Mac RUN it is false (historical since P2-B, K-P2B.4)
    res = tier2.fails(P2R_RUN, historical_p2r.classify(P2R_RUN), INV, "eval.historical_p2r")
    assert [c["criterion"] for c in res["counted"]] == ["p1e1_checks.json:tamper·det"]
    assert res["derived_criteria"][0]["value"] is False
    assert "p1e1_checks.json:tamper·det" in tier2.criteria(P2R_RUN)


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_the_mac_reading_runs_deterministic_o_once_and_reads_decisions_by_record(tmp_path):
    import tempfile

    from eval import verify_mac

    with tempfile.TemporaryDirectory() as t:
        v = Path(t)
        for f in P2R_RUN.iterdir():
            (v / f.name).symlink_to(f)
        first, later = verify_mac.rep(v, first=True), verify_mac.rep(v, first=False)
    assert "p1c1_checks.json:C1-8" in first and "p1c1_checks.json:C1-8" not in later
    for i, c in ((1, first), (2, later), (3, later)):
        (tmp_path / "mac" / f"rep{i}").mkdir(parents=True)
        rec = {"run": f"r{i}", "rep": i, "elapsed_s": 1, "errors": [], "criteria": c}
        (tmp_path / "mac" / f"rep{i}" / "rep.json").write_text(json.dumps(rec))
    res = verify_mac.summary(tmp_path, 3, INV)
    by = {s["id"]: s for s in res["sets"]}
    c18 = by["O-C18"]["criteria"][0]
    assert c18["passed"] == "1/1" and c18["pass"] is True and c18["values"][0]["n_differing"] == 0
    rel = {c["criterion"]: c for c in by["O-REL"]["criteria"]}
    assert (
        rel["p1e_checks.json:E-7"]["passed"] == "0/1" and rel["p1e_checks.json:E-7"]["record_equal"]
    )
    assert (
        rel["p1e1_checks.json:E1-3"]["passed"] == "3/3"
        and rel["p1e1_checks.json:E1-3"]["pass"] is None
    )
    # Z sets: values and 3/3 reported, outside the verdict
    assert by["Z-D10"]["pass"] is None and by["Z-D10"]["pass_3of3"] is False
    assert res["historical"]["p2_0_3_checks.json:H-P2-0.3b"] is True and res["pass"] is True
