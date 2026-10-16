"""P1-D.1 round report (TASKS D1.7): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
D1-1..D1-13 table, the deliberate corruptions, and the decision rule of
docs/rounds/P1-D.1/GOAL.md, derived from the rows:

- D1-1..D1-13 all pass → K-D4-T closes, P1-E starts.
- D1-1, D1-3 or D1-4 fails → governance; P1-E does not start.
- D1-5 fails → determinism; P1-E does not start.
- any other failure → undefined failure (K-D1.9): the round fails, a
  decision request.

D1-12 = the e2e pipelined vaults inside each chain, `regress_vs_p1d.json`
(P1-D quality fields, decision exclusions) and `pair_compare.json`
(`eval.compare_p1d1 pair`, A ↔ B). Writes `<RUN_DIR>/FINAL.md` and
`reports/rounds/P1-D.1/FINAL.md`.

    python -m eval.report_p1d1 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-D.1" / "FINAL.md"
GOVERNANCE = ("D1-1", "D1-3", "D1-4")
DETERMINISM = ("D1-5",)
ALL = tuple(f"D1-{n}" for n in range(1, 14))


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool) -> str:
    return "geçti" if b else "**KALDI**"


def rows(d: Path) -> list[dict]:
    c = _load(d, "p1d1_checks.json")
    pair = _load(d, "pair_compare.json")
    d6 = c["D1-6"]["measured"]
    reg = c["D1-12"]["regress_vs_p1d"]
    return [
        {
            "id": "D1-1",
            "what": "`append_only` altında canonical bloğa dokunan 6 `replace_section` (tam, kısmi, başlık, karışık, iç içe, insan dokümanının tamamı)",
            "value": f"{c['D1-1']['denied']} `denied`; dosya ve `events.jsonl` değişmedi: {c['D1-1']['files_and_events_unchanged']}",
            "threshold": "6/6 `denied`, dosya ve `events.jsonl` değişmez",
            "pass": c["D1-1"]["pass"],
            "source": "p1d1/d1_1.json",
        },
        {
            "id": "D1-2",
            "what": "yalnız proposal bloklardan oluşan bölüme `replace_section`",
            "value": f"kabul {c['D1-2']['accepted']}, bloklar {', '.join(c['D1-2']['section_layers'])}",
            "threshold": "kabul, bloklar `proposal`",
            "pass": c["D1-2"]["pass"],
            "source": "p1d1/d1_2.json",
        },
        {
            "id": "D1-3",
            "what": "500 ajan yazımı (create/append/replace) sonrası varsayılan `search` canonical sonuçları (200 sorgu × 3 kip)",
            "value": f"{c['D1-3']['accepted']}/{c['D1-3']['writes']} yazım kabul ({', '.join(f'{k} {v}' for k, v in c['D1-3']['outcomes'].items())}); {c['D1-3']['comparisons']} karşılaştırmada fark {c['D1-3']['differing']}",
            "threshold": "bayt eşit",
            "pass": c["D1-3"]["pass"],
            "source": "p1d1/d1_3.json",
        },
        {
            "id": "D1-4",
            "what": "`unrestricted` ile silme → `BlockDeleted.meta.actor`; `history` silen aktörü gösterir",
            "value": f"olay {c['D1-4']['event_actor']}, history {c['D1-4']['history_actor']}",
            "threshold": "1/1",
            "pass": c["D1-4"]["pass"],
            "source": "p1d1/d1_4.json",
        },
        {
            "id": "D1-5",
            "what": "boru hatlı 100 çağrı (40 `propose`) iki özdeş vault'a",
            "value": f"A↔B {c['D1-5']['pipelined_a_vs_b']}; kilit adımlı ile {c['D1-5']['pipelined_vs_lockstep']}",
            "threshold": "`events.jsonl`, `traces.jsonl`, md dosyaları bayt eşit",
            "pass": c["D1-5"]["pass"],
            "source": "p1d1/d1_5.json",
        },
        {
            "id": "D1-6",
            "what": "boru hatlı 100 salt okunur çağrı",
            "value": f"{c['D1-6']['correct']} doğru; süre oranı {d6['ratio']:.2f} (< 0.6: {d6['ratio_below_0_6']}, bilgi)",
            "threshold": "100/100; süre < seri %60 (bilgi)",
            "pass": c["D1-6"]["pass"],
            "source": "p1d1/d1_6.json",
        },
        {
            "id": "D1-7",
            "what": "HTTP `promote` (`human_id`) → `human:<id>`; MCP `promote` → `denied`; `human_id`'siz HTTP yazma → 403",
            "value": f"{c['D1-7']['passed']} (MCP {c['D1-7']['mcp_promote']}, HTTP'siz kimlik {c['D1-7']['http_without_human_id']}, aktör {c['D1-7']['http_promote_actor']})",
            "threshold": "3/3",
            "pass": c["D1-7"]["pass"],
            "source": "p1d1/d1_7.json",
        },
        {
            "id": "D1-8",
            "what": "12 kötü girdi (P1-D V2/V3) × MCP ve HTTP → beklenen kod",
            "value": c["D1-8"]["passed"],
            "threshold": "24/24",
            "pass": c["D1-8"]["pass"],
            "source": "p1d1/d1_8.json",
        },
        {
            "id": "D1-9",
            "what": "adsız checkpoint'i adlandırma; ad çakışması → `exists`",
            "value": c["D1-9"]["passed"],
            "threshold": "2/2",
            "pass": c["D1-9"]["pass"],
            "source": "p1d1/d1_9.json",
        },
        {
            "id": "D1-10",
            "what": "uyumluluk 31/31 + C2-10, C3-06, C3-07",
            "value": f"{c['D1-10']['passed']}, level {c['D1-10']['level_claimed']}, spec {c['D1-10']['spec_version']} (CRITERION: {c['D1-10']['spec_version_by_criterion']}); yeni: {c['D1-10']['new_tests']}",
            "threshold": "34/34, spec sürümü CRITERION kuralıyla",
            "pass": c["D1-10"]["pass"],
            "source": "conformance_p1d1.json",
        },
        {
            "id": "D1-11",
            "what": "eski vault'lar (p1d-final sahne ve e2e vault'ları, v3-final karışık) açılır, replay eşit, `events_hash` değişmez",
            "value": "; ".join(
                f"{k}: {v['replay_equal']}/{v['docs']} replay, hash eşit {v['events_hash_equal']}"
                for k, v in c["D1-11"]["vaults"].items()
            )
            + f"; v3-final {c['D1-11']['v3_final_compat']}",
            "threshold": "eşit",
            "pass": c["D1-11"]["pass"],
            "source": "p1d1/d1_11.json",
        },
        {
            "id": "D1-12",
            "what": "çift koşturma (boru hatlı vakalar dahil); P1-D kalite alanları (karar gereği hariçler)",
            "value": f"pair {pair['n_fields']} alan, fark {len(pair['differing_fields'])}, baytlar eşit {pair['bytes_equal']}; P1-D'ye göre {reg['n_fields']} alan, hariç dışı fark {len(reg['differing_fields'])}, hariç tutulan fark {len(reg['excluded_differences'])} alt ağaç",
            "threshold": "fark 0",
            "pass": c["D1-12"]["pass"] and pair["equal"],
            "source": "pair_compare.json, regress_vs_p1d.json",
        },
        {
            "id": "D1-13",
            "what": "`docs/CLIENT_CONTRACT.md`, `docs/SECURITY.md`; her araç/kod bir testle eşleşir (`scripts/contract_check.sh`)",
            "value": f"{c['D1-13']['tools']} araç, {c['D1-13']['codes']} kod, eksik {len(c['D1-13']['missing'])}",
            "threshold": "eksik 0",
            "pass": c["D1-13"]["pass"] and (ROOT.parent / "docs" / "SECURITY.md").is_file(),
            "source": "p1d1/d1_13.json",
        },
    ]


def decide(r: list[dict]) -> tuple[str, list[str]]:
    failed = [x["id"] for x in r if not x["pass"]]
    if not failed:
        return "D1-1..D1-13 geçti → **K-D4-T kapanır, P1-E başlar.**", failed
    if any(f in GOVERNANCE for f in failed):
        return "D1-1/D1-3/D1-4 kaldı → **yönetişim; P1-E başlamaz.**", failed
    if any(f in DETERMINISM for f in failed):
        return "D1-5 kaldı → **determinizm; P1-E başlamaz.**", failed
    return "Tanımlanmamış kalış → **tur kaldı, karar talebi** (K-D1.9).", failed


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p1d1_checks.json")
    r = rows(d)
    assert tuple(x["id"] for x in r) == ALL
    rule, failed = decide(r)
    lines = [
        "# P1-D.1 — FINAL",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).",
        "",
        "| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |",
        "|---|---|---|---|---|---|",
    ]
    for x in r:
        lines.append(
            f"| {x['id']} | {x['what']} | {x['value'].replace('|', '\\|')} | {x['threshold']} | {_ok(x['pass'])} | `{x['source']}` |"
        )
    t = c["tamper"]
    lines += [
        "",
        f"Kasıtlı bozmalar (`p1d1/tamper_p1d1.json`): {t['rejected']} reddedildi, kontroller {t['controls']} → {_ok(t['pass'])}.",
        "",
        "## Karar kuralı",
        "",
        f"Kalan kriterler: {', '.join(failed) if failed else 'yok'}.",
        "",
        rule,
        "",
    ]
    text = "\n".join(lines)
    (d / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(rule)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
