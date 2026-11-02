"""P2-V.2 (TASKS Y1, Y2): the round's inventory — P2-V.1's entries as they
were plus only K-P2V1.2's four names — and the tools' readings of them."""

import hashlib
import json
from pathlib import Path

import pytest

from p2v2 import inventory as I

REPO = Path(__file__).resolve().parents[2]
P2V = REPO / "reports/rounds/P2-V/inventory.json"
P2V1 = REPO / "reports/rounds/P2-V.1/inventory.json"
P2V2 = REPO / "reports/rounds/P2-V.2/inventory.json"
# blake2b of the earlier inventories at ROUND_START (89c4b28): they stay as
# they are (V2-1)
P2V_B2 = "a0b14d7758c7b7339af02af68322a4ba5e3eb91f6c84821bc8635a419d2ab86dbd91f7d7f50d20970ec3407413540a6c26001f0cd8be523e2da90a5ddb78137a"
P2V1_B2 = "08c235c8cac5264dbbced111d763cb89026f6064c1cc97d2d77d1b36d5241319ce3449d663c2580c291e9e606a47330f5d6eefd3c1aa45dfcc89a4edb50c0b39"


def _load(p: Path) -> dict:
    return json.loads(p.read_text(encoding="utf-8"))


def test_earlier_inventories_are_unchanged():
    assert hashlib.blake2b(P2V.read_bytes()).hexdigest() == P2V_B2
    assert hashlib.blake2b(P2V1.read_bytes()).hexdigest() == P2V1_B2


def test_p2v2_differs_from_p2v1_only_by_the_four_names():
    d = I.diff(_load(P2V1), _load(P2V2))
    assert d["removed"] == []
    assert d["outside_allowed"] == []
    # changed leaves: metadata and R7's status only
    assert d["changed"] == [
        "('base', 'file')",
        "('base', 'rule')",
        "('observed_outside_e1', 0, 'status')",
        "('round',)",
        "('round_start',)",
        "('task',)",
    ]


def test_the_four_names_are_k_p2v1_2s():
    inv = _load(P2V2)
    # T2: E1-W
    (w,) = inv["e1_families"]["named"]
    assert (w["id"], w["item"], w["field_in_item"]) == ("E1-W", "R7", "detail")
    assert inv["observed_outside_e1"][0]["status"] == "adlandırıldı: E1-W (K-P2V1.2)"
    # T3: tamper·det historical, still a criterion
    (t,) = inv["derived_criteria"]
    assert t["id"] == "p1e1_checks.json:tamper·det" and t["reading"] == "historical"
    assert t["reference"]["pass"] is False and t["until"] == "P2-C"
    assert any("generated with the p1d-final mdk" in c for c in t["reference"]["failing_cases"])
    # T4: H5
    h5 = [h for h in inv["host_fields"] if h["id"] == "H5"]
    assert h5 and h5[0]["file"] == "conformance.json" and h5[0]["field"] == "fixtures_root"
    assert h5[0]["class"] == "measured_host_value"
    # T5: the five real-Obsidian O criteria, with P2-V's thresholds
    ro = inv["mac_reading"]["reported_only"]
    assert [r["criterion"] for r in ro] == list(I.REPORTED_ONLY)
    p2v = {
        f"{c['file']}:{c['criterion']}": c for s in _load(P2V)["mac_sets"] for c in s["criteria"]
    }
    for r in ro:
        assert r["threshold"] == p2v[r["criterion"]]["threshold"] and r["carry"] == "P2-C"
    # no other name: thresholds and criteria as in P2-V.1
    base = _load(P2V1)
    assert inv["mac_sets"] == base["mac_sets"]
    assert inv["tier2_set_aside_criteria"] == base["tier2_set_aside_criteria"]
    assert inv["pair_exclusions"] == base["pair_exclusions"]
    assert inv["byte_exclusions"] == base["byte_exclusions"]
    assert inv["e1_fields"] == base["e1_fields"]


# --- Y2: the tools' readings (FREEZE_EXCEPTIONS F1–F3) -----------------------

FIX = Path(__file__).resolve().parent / "fixtures/p2v2"
P2R_RUN = REPO / "reports/rounds/P2-R/runs/20261026T222501Z-5b235b0"


def _tamper_run(tmp_path: Path, record: bytes) -> Path:
    run = tmp_path / "run"
    (run / "p1e1").mkdir(parents=True)
    (run / "p1e1/tamper_p1e1.json").write_bytes(record)
    # the checklist the inventory's item entries resolve in (B6)
    (run / "p1e1/release").mkdir()
    items = [{"id": f"R{i}", "item": "x", "ok": True, "detail": ""} for i in range(1, 21)]
    items.insert(3, {"id": "R4a", "item": "x", "ok": True, "detail": ""})
    (run / "p1e1/release/release_check.json").write_text(json.dumps({"items": items}))
    return run


def test_f1_tamper_det_equal_to_its_reference_is_not_counted(tmp_path):
    from eval import tier2

    rec = (FIX / "tamper_p1e1.json").read_bytes()
    run = _tamper_run(tmp_path, rec)
    res = tier2.fails(run, {"fails": []}, tier2.load_inventory(P2V2))
    assert res["counted_fails"] == 0 and res["pass"]
    (h,) = res["historical"]
    assert h["record_equal"] and h["class"] == "tarihsel" and h["until"] == "P2-C"
    assert res["derived_criteria"][0]["reading"] == "historical"
    # the criterion is still listed (no criterion lost)
    assert "p1e1_checks.json:tamper·det" in [d["criterion"] for d in res["derived_criteria"]]
    # P2-V.1's inventory: counted, as P2-V.1 did
    old = tier2.fails(run, {"fails": []}, tier2.load_inventory(P2V1))
    assert [c["criterion"] for c in old["counted"]] == ["p1e1_checks.json:tamper·det"]
    assert "historical" not in old


def test_f1_a_deterministic_corruption_keeping_pass_false_is_counted(tmp_path):
    # Ş2 of P2-V.1's verification: a control case broken, `pass` still false
    from eval import tier2

    doc = json.loads((FIX / "tamper_p1e1.json").read_text(encoding="utf-8"))
    doc["cases"][0]["got"], doc["cases"][0]["as_expected"] = "rejected", False
    doc["controls_ok"] = 0
    run = _tamper_run(tmp_path, json.dumps(doc, indent=1).encode())
    res = tier2.fails(run, {"fails": []}, tier2.load_inventory(P2V2))
    assert res["counted_fails"] == 1
    assert set(res["counted"][0]["historical"]["differs"]) == {"blake2b", "failing_cases"}


def test_f1_a_changed_reference_text_is_counted(tmp_path):
    # VERIFY VY3 (a): the reading is a real comparison with the reference
    from eval import tier2

    run = _tamper_run(tmp_path, (FIX / "tamper_p1e1.json").read_bytes())
    inv = tier2.load_inventory(P2V2)
    inv["derived_criteria"][0]["reference"]["failing_cases"][1] = "fixture regenerated elsewhere"
    res = tier2.fails(run, {"fails": []}, inv)
    assert res["counted_fails"] == 1
    assert res["counted"][0]["historical"]["differs"] == ["failing_cases"]


def test_f1_pair_names_the_inventory_read():
    from eval import tier2

    assert tier2._inventory_name(P2V1) == "reports/rounds/P2-V.1/inventory.json"
    assert tier2.INVENTORY == P2V2


HOST = {"workdir": "/home/runner/work/_temp/v/A", "home": "/home/runner", "user": "runner"}


def test_f2_h5_is_skipped_by_name_and_reported(tmp_path):
    from eval import portable

    run = tmp_path / "run"
    run.mkdir()
    root = "/home/runner/work/_temp/v/A/repo/spec/fixtures"
    (run / "conformance.json").write_text(json.dumps({"fixtures_root": root, "passed": 25}))
    pats = portable.needles([HOST])
    named = portable.measured_host_values(_load(P2V2))
    assert [e["id"] for e in named] == ["H5"]
    skipped: list = []
    hits = portable.scan(run, ["conformance.json"], pats, None, named, skipped)
    assert hits == [] and [s["id"] for s in skipped] == ["H5"]
    # P2-V.1's inventory: no measured host value → found, as before
    assert portable.measured_host_values(_load(P2V1)) == []
    assert portable.scan(run, ["conformance.json"], pats)


def test_f2_another_host_value_is_still_found(tmp_path):
    # VERIFY VY3 (b): the skip by name does not leak to another field or record
    from eval import portable

    run = tmp_path / "run"
    run.mkdir()
    root = "/home/runner/work/_temp/v/A/repo/spec/fixtures"
    (run / "conformance.json").write_text(
        json.dumps({"fixtures_root": root, "log": "/home/runner/work/_temp/v/A/x.log"})
    )
    (run / "other.json").write_text(json.dumps({"fixtures_root": root}))
    pats = portable.needles([HOST])
    named = portable.measured_host_values(_load(P2V2))
    hits = portable.scan(run, ["conformance.json", "other.json"], pats, None, named, [])
    assert {h["file"] for h in hits} == {"conformance.json", "other.json"}
    assert all("x.log" in h["context"] for h in hits if h["file"] == "conformance.json")


def test_f3_a_missing_d10_record_is_a_failed_repetition(tmp_path):
    from eval import verify_mac

    d = tmp_path / "rr"
    (d / "p1d").mkdir(parents=True)
    (d / "p1d/serve_watch.log").write_text("edit 0 of espresso.md not searchable after 10 s\n")
    v = verify_mac._d10(d)
    assert v["pass"] is False and v["missing_records"] == ["p1d/d10.json", "p1d/sigterm.json"]
    assert any("not searchable" in x for x in v["log_tail"])
    # a timeout record (serve_watch.rs, P2-V.2/Y2): failed, its measurements kept
    (d / "p1d/d10.json").write_text(
        json.dumps(
            {
                "timeout": True,
                "deadline_ms": 1,
                "stopped_at": {"edit": 0, "doc": "espresso.md"},
                "edits": 0,
                "latencies_ms": [],
                "p95_ms": None,
                "max_ms": 0.0,
                "pass": False,
            }
        )
    )
    (d / "p1d/sigterm.json").write_text(json.dumps({"trials": [], "pass": True}))
    v = verify_mac._d10(d)
    assert v["pass"] is False and v["timeout"]["d10"] is True and v["deadline_ms"] == 1
    assert v["missing_records"] == []


def _mac_rows(tmp_path: Path):
    import tempfile

    from eval import verify_mac

    with tempfile.TemporaryDirectory() as t:
        v = Path(t)
        for f in P2R_RUN.iterdir():
            (v / f.name).symlink_to(f)
        first = verify_mac.rep(v, first=True)
        later = verify_mac.rep(v, first=False)
        # repetition 2: serve_watch wrote no record (Ş1)
        e = tmp_path / "e"
        (e / "p1d").mkdir(parents=True)
        (e / "p1d/serve_watch.log").write_text("not searchable after 10 s\n")
        stopped = {**later, "p1d_checks.json:D-10": verify_mac._d10(e)}
    return [
        {"run": f"r{i}", "rep": i, "elapsed_s": 1, "errors": [], "criteria": c}
        for i, c in ((1, first), (2, stopped), (3, later))
    ]


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_f3_summary_records_the_failed_repetition_and_writes_mac_checks(tmp_path, monkeypatch):
    from eval import verify_mac

    main = tmp_path / "main"
    for i, rec in enumerate(_mac_rows(tmp_path), 1):
        (main / "mac" / f"rep{i}").mkdir(parents=True)
        (main / "mac" / f"rep{i}" / "rep.json").write_text(json.dumps(rec))
    res = verify_mac.summary(main, 3, _load(P2V2))
    by = {s["id"]: s for s in res["sets"]}
    d10 = by["Z-D10"]["criteria"][0]
    assert d10["values"][1]["pass"] is False and d10["values"][1]["missing_records"]
    assert by["Z-D10"]["pass_3of3"] is False  # the set failed (kaldı), recorded
    assert d10["in_verdict"] is False  # Z (K-P2V.3)
    # reported_only: the five, outside the verdict, with thresholds
    ro = {r["criterion"]: r for r in res["reported_only"]}
    assert sorted(ro) == sorted(I.REPORTED_ONLY)
    assert all(r["in_verdict"] is False and r["threshold"] for r in ro.values())
    # the other O criteria are in the verdict with 3/3
    assert "p1e1_checks.json:E1-3" in res["in_verdict"]
    # mac_checks.json is written by the CLI with a failed repetition too (the
    # RUN id resolved to this directory)
    monkeypatch.setattr(verify_mac.runinfo, "run_dir", lambda r, create=False: Path(r))
    rc = verify_mac.main(
        ["summary", "--run", str(main), "--reps", "3", "--elapsed", "3", "--setup", "1"]
    )
    assert rc in (0, 3) and (main / "mac_checks.json").is_file()


@pytest.mark.skipif(not P2R_RUN.is_dir(), reason="the P2-R archive is not in this tree")
def test_f3_reread_keeps_the_recorded_values(tmp_path):
    from eval import verify_mac

    main = tmp_path / "main"
    for i, rec in enumerate(_mac_rows(tmp_path), 1):
        (main / "mac" / f"rep{i}").mkdir(parents=True)
        (main / "mac" / f"rep{i}" / "rep.json").write_text(json.dumps(rec))
    for inv in (P2V1, P2V2):
        rec = verify_mac.summary(main, 3, _load(inv)) | {"elapsed_s": 3, "setup_s": 1, "total_s": 4}
        res = verify_mac.reread(rec, _load(inv))
        assert verify_mac.dump_json(res) == verify_mac.dump_json(rec)  # same inventory: same bytes
        assert verify_mac.values_equal(rec, verify_mac.reread(rec, _load(P2V2)))["pass"]
    # a reported_only value corrupted: reported, the verdict unchanged
    rec = verify_mac.summary(main, 3, _load(P2V2)) | {"elapsed_s": 3, "setup_s": 1, "total_s": 4}
    for s in rec["sets"]:
        for c in s["criteria"]:
            if c["criterion"] == "p2_0_3_checks.json:H-P2-0.3c":
                c["values"][0] = {**c["values"][0], "pass": True}
    after = verify_mac.reread(rec, _load(P2V2))
    assert after["pass"] == rec["pass"]
    # a deterministic O criterion corrupted: the verdict fails
    for s in rec["sets"]:
        for c in s["criteria"]:
            if c["criterion"] == "p1c1_checks.json:C1-8":
                c["values"][0] = {**c["values"][0], "pass": False}
    assert verify_mac.reread(rec, _load(P2V2))["pass"] is False
