"""P2-V.2/Y1 (TASKS Y1; GOAL P2-V.2 İlkeler, Kapsam 2): the round's inventory,
`reports/rounds/P2-V.2/inventory.json`, written once before the round's tool
changes and never changed after (pre-registration).

The P2-V.1 inventory (`reports/rounds/P2-V.1/inventory.json`) is carried
over entry for entry; this round adds only K-P2V1.2's four names, by name
and without a pattern:

- **T2 (a) — E1-W** (`e1_families.named`): the release checklist's R7
  `detail` (the Python wheel's file name, `linux_x86_64` →
  `manylinux_2_34_x86_64`) is a result of E1 (`ort` removed, so maturin can
  tag the extension manylinux 2.34); `observed_outside_e1`'s entry moves to
  "adlandırıldı: E1-W (K-P2V1.2)". The release scripts find the wheel by the
  pattern `*.whl`, not by name (`scripts/release_check.sh:71`).
- **T3 (c) — `tamper·det` historical** (`derived_criteria`): the criterion
  stays (its id is listed, `tier2 criteria`), read by record equality
  against P2-V.1's record (`pass` false, the failing cases' text — "fixture
  regenerated with the p1d-final mdk", "p1e-final server …" — and the
  record's blake2b); equal → not counted, different → counted. Its cause is
  removed in P2-C (`until`).
- **T4 (a) — H5** (`host_fields`): `conformance.json` `fixtures_root` (the
  chain RUN's P1-B view) is "the host value the criterion measures" (B1-6
  measures that it is absolute; the frozen P1-B recorder); `eval.portable`
  skips that field by name and reports the skip.
- **T5 (a) — `reported_only`** (`mac_reading`): the non-historical
  real-Obsidian O criteria (H-P2-0.3b-yük, H-P2-0.3c, H-P2-0.3d, H-P2-0.3e,
  P1-E.1 tamper e2e) are reported with their values and P2-V thresholds, as
  Z is, outside the set's verdict; carried to P2-C with their ids and
  thresholds (condition 1).

    python -m p2v2.inventory --tamper-record <P2-V.1 RUN>/p1e1/tamper_p1e1.json --out ../reports/rounds/P2-V.2/inventory.json

(from `research/`, `PYTHONPATH=.`). The tamper record is P2-V.1's (tier-2
run 41239746833, chain A; downloaded, its blake2b in
`reports/rounds/P2-V.2/ci/41239746833/manifest.json`): read for the
reference of the historical reading, nothing else.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "reports/rounds/P2-V.1/inventory.json"
TAMPER_REF = "reports/rounds/P2-V.2/ci/41239746833/tier2-A/run/p1e1/tamper_p1e1.json"
REPORTED_ONLY = (
    "p2_0_3_checks.json:H-P2-0.3b-yük",
    "p2_0_3_checks.json:H-P2-0.3c",
    "p2_0_3_checks.json:H-P2-0.3d",
    "p2_0_3_checks.json:H-P2-0.3e",
    "p1e1_checks.json:tamper",
)


def build(base: dict, tamper: bytes) -> dict:
    inv = json.loads(json.dumps(base))  # every P2-V.1 entry, as it is
    inv["round"] = "P2-V.2"
    inv["task"] = "P2-V.2/Y1"
    inv["round_start"] = "89c4b28"
    inv["base"] = {
        "file": "reports/rounds/P2-V.1/inventory.json",
        "rule": "P2-V.1'in envanteri aynen taşınır (her girdi, kimliği ve gerekçesiyle); bu dosya yalnız K-P2V1.2'nin dört adını ekler (`p2v2_changes`)",
        "p2v1_base": base["base"],
    }

    # T2 (a): E1-W
    inv["e1_families"]["named"] = [
        {
            "id": "E1-W",
            "family": "E1-W",
            "file": "p1e1/release/release_check.json",
            "field": None,
            "item": "R7",
            "field_in_item": "detail",
            "decision": "K-P2V1.2 (K-P2V1-T2 (a))",
            "why": "Python tekerleğinin dosya adı (`linux_x86_64` → `manylinux_2_34_x86_64`): E1 `ort`'u (ONNX Runtime, C++ ve libstdc++ bağlantısı) çıkardı, maturin uzantıyı artık manylinux 2.34 politikasıyla etiketleyebiliyor — ürün değişikliğinin bilinen yan etkisi; R7'nin kendisi iki zincirde de geçer",
            "observed": "kademe 2 41239746833 A (`58377de`) ↔ P2-V V0m 40613392932 (`352f790`): `rootmark_bindings-0.2.0-cp311-abi3-linux_x86_64.whl` → `…-manylinux_2_34_x86_64.whl` (`reports/rounds/P2-V.1/ci/41239746833/equiv_v0m.json`)",
            "note": "yayım betikleri tekerleği adla değil `*.whl` kalıbıyla bulur (`scripts/release_check.sh:71`, `scripts/release.sh:106`; P2-V.1 doğrulamasında denetlendi)",
            "use": "yalnız ONNX Runtime sayısalının kayıtlarına karşı karşılaştırmalarda adlı (`tier2 equiv`); iki zincir arasında D",
        }
    ]
    inv["e1_families"]["counts"]["E1-W"] = 1
    for e in inv["observed_outside_e1"]:
        if e.get("item") == "R7" and e.get("field_in_item") == "detail":
            e["status"] = "adlandırıldı: E1-W (K-P2V1.2)"
            e["status_p2v1"] = base["observed_outside_e1"][0]["status"]

    # T3 (c): tamper·det, historical reading by record equality
    doc = json.loads(tamper)
    failing = [c["case"] for c in doc["cases"] if c.get("as_expected") is not True]
    if doc.get("pass") is not False or not any(
        "generated with the p1d-final mdk" in c for c in failing
    ):
        raise SystemExit(f"{TAMPER_REF}: not P2-V.1's record (pass false, p1d-final fixture case)")
    for d in inv["derived_criteria"]:
        if d["id"] == "p1e1_checks.json:tamper·det":
            d["reading"] = "historical"
            d["reference"] = {
                "record": TAMPER_REF,
                "run": "41239746833",
                "chain": "A",
                "commit": "58377de",
                "pass": False,
                "failing_cases": failing,
                "reason": "generated with the p1d-final mdk",
                "blake2b": hashlib.blake2b(tamper).hexdigest(),
                "bytes": len(tamper),
            }
            d["decision"] = "K-P2V1.2 (K-P2V1-T3 (c): (a) şimdi, (b) P2-C)"
            d["until"] = "P2-C"
            d["historical_why"] = (
                "`tamper_p1e1.json` `pass` P2-B'den beri false: Linux'ta \"p1d-final mdk ile yeniden üretilen fixture\" kontrolü tutmaz (dondurulmuş eski ikilinin ORT vektörleri ↔ yayım Mac'inde üretilmiş fixture; K-P2B.4). Kayıt eşitliğiyle okunur: kayıt (blake2b), `pass` ve beklenmedik vakaların metni referansla aynıysa FAIL sayılmaz, biri farklıysa sayılır (Ş2'nin kör noktası: `pass`'i false bırakan deterministik bir bozma kaydı değiştirir, sayılır). Nedeninin giderilmesi P2-C'nin ön-kayıtlı kriteri."
            )

    # T4 (a): H5
    inv["host_fields"].append(
        {
            "id": "H5",
            "file": "conformance.json",
            "field": "fixtures_root",
            "class": "measured_host_value",
            "where": "Makefile p1b-checks (RUN'ın P1-B görünümü; dondurulmuş `run_p1b_checks.py:42` ondan `fixtures_root_absolute`'u yazar)",
            "decision": "K-P2V1.2 (K-P2V1-T4 (a))",
            "why": "kriterin ölçtüğü host değeri: P1-B.1'in B1-6'sı (`p1b_checks` `fixtures_root_absolute`) kökün mutlak olmasını ölçer, kayıt tanım gereği host'a bağlıdır (göreli yapılınca kademe 2 41235960263/41235968688 B1-6'da durdu); `eval.portable` bu alanı adıyla atlar ve `skipped_by_name`'de raporlar; çift karşılaştırmasında dondurulmuş karşılaştırıcının ortam alanıdır (D farkına girmez)",
        }
    )

    # T5 (a): reported_only real-Obsidian O criteria
    thresholds = {
        f"{c['file']}:{c['criterion']}": (s["id"], c)
        for s in inv["mac_sets"]
        for c in s["criteria"]
    }
    ro = []
    for cid in REPORTED_ONLY:
        sid, c = thresholds[cid]
        ro.append(
            {
                "criterion": cid,
                "mac_set": sid,
                "threshold": c["threshold"],
                "source": c["source"],
                "carry": "P2-C",
                "decision": "K-P2V1.2 (K-P2V1-T5 (a), şart 1)",
                "why": "tarihsel olmayan gerçek Obsidian O kriteri: Z gibi 3 tekrar değerleriyle ve P2-V eşiğiyle raporlanır, Mac kümesinin hükmüne girmez (K-P2V.3 T2'nin genişletilmesi); kimliği ve eşiğiyle P2-C'ye ürün bulgusu (harness `attest` 403 durması, izleyici gecikmesi)",
            }
        )
    inv["mac_reading"]["reported_only"] = ro

    inv["rules"] = [
        *inv["rules"],
        'P2-V.2 (K-P2V1.2 T3): tarihsel okumalı türev kriter (`reading: "historical"`) kayıt eşitliğiyle sayılır: kayıt referansla aynıysa (blake2b, `pass`, beklenmedik vakaların metni) FAIL sayılmaz, farklıysa sayılır; kimliği kriter listesinde kalır.',
        "P2-V.2 (K-P2V1.2 T5): `mac_reading.reported_only` kriterleri değerleriyle (3 tekrar) ve eşikleriyle raporlanır, Mac kümesinin hükmüne girmez (`in_verdict: false`); tarihsel olmayan, tek koşu olmayan öteki O kriterleri 3/3 ile hükme girer, Z kümeleri hükme girmez (K-P2V.3).",
    ]
    inv["p2v2_changes"] = [
        "T2 (a): `e1_families.named` E1-W (R7 `detail`), `e1_families.counts.E1-W`; `observed_outside_e1` R7 durumu `adlandırıldı: E1-W (K-P2V1.2)` (eski metni `status_p2v1`)",
        "T3 (c): `derived_criteria` `p1e1_checks.json:tamper·det` + `reading`, `reference`, `decision`, `until`, `historical_why`",
        "T4 (a): `host_fields` H5 (`conformance.json` `fixtures_root`, `class: measured_host_value`)",
        "T5 (a): `mac_reading.reported_only` (beş kriter)",
        "kurallar: `rules`'a iki satır (T3, T5)",
        "üst bilgi: `round`, `task`, `round_start`, `base` (tabanın kendisi `base.p2v1_base`)",
    ]
    return inv


def _leaves(x, p=()):
    if isinstance(x, dict):
        if not x:
            yield p, {}
        for k, v in x.items():
            yield from _leaves(v, (*p, k))
    elif isinstance(x, list):
        if not x:
            yield p, []
        for i, v in enumerate(x):
            yield from _leaves(v, (*p, i))
    else:
        yield p, x


# the paths (as prefixes) where P2-V.2 may differ from P2-V.1: the four
# names, the two rules, the metadata (V2-1: "programla fark yalnız ekler")
ALLOWED = (
    ("round",),
    ("task",),
    ("round_start",),
    ("base",),
    ("p2v2_changes",),
    ("e1_families", "named"),
    ("e1_families", "counts", "E1-W"),
    ("observed_outside_e1", 0, "status"),
    ("observed_outside_e1", 0, "status_p2v1"),
    ("derived_criteria", 0, "reading"),
    ("derived_criteria", 0, "reference"),
    ("derived_criteria", 0, "decision"),
    ("derived_criteria", 0, "until"),
    ("derived_criteria", 0, "historical_why"),
    ("mac_reading", "reported_only"),
)


def diff(base: dict, new: dict) -> dict:
    """The programmatic difference P2-V.1 → P2-V.2: every leaf removed,
    changed or added, and those outside the allowed additions."""
    a, b = dict(_leaves(base)), dict(_leaves(new))
    removed = sorted(map(str, set(a) - set(b)))
    changed = sorted(str(k) for k in set(a) & set(b) if a[k] != b[k])
    added = sorted(set(b) - set(a), key=str)
    n_host, n_rules = len(base["host_fields"]), len(base["rules"])
    ok = (
        *ALLOWED,
        ("host_fields", n_host),
        ("rules", n_rules),
        ("rules", n_rules + 1),
    )

    def allowed(k: tuple) -> bool:
        return any(k[: len(p)] == p for p in ok)

    changed_keys = [k for k in set(a) & set(b) if a[k] != b[k]]
    return {
        "removed": removed,
        "changed": changed,
        "added": [str(k) for k in added],
        "outside_allowed": sorted(str(k) for k in [*added, *changed_keys] if not allowed(k))
        + removed,
        "added_top": sorted({str(k[:2]) for k in added if allowed(k)}),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tamper-record", type=Path, default=ROOT / TAMPER_REF)
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args()
    base = json.loads(BASE.read_text(encoding="utf-8"))
    inv = build(base, a.tamper_record.read_bytes())
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(inv, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"inventory: {a.out} ({len(inv['p2v2_changes'])} changes)")


if __name__ == "__main__":
    main()
