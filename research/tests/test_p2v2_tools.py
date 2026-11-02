"""P2-V.2 (TASKS Y1, Y2): the round's inventory — P2-V.1's entries as they
were plus only K-P2V1.2's four names — and the tools' readings of them."""

import hashlib
import json
from pathlib import Path

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
