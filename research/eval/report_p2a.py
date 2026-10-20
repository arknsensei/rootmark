"""P2-A round report (TASKS A6): `reports/rounds/P2-A/FINAL.md` from one RUN
directory (`p2a_checks.json`, `pair_compare.json` of the pair) and the
round's committed records — the criteria table A-1…A-8 against GOAL P2-A's
pre-registered thresholds, the decision rule's branch, the information
measurement under its own heading (no threshold, no decision), the FAIL
summary, regress and the deliberate cases.

The decision rule (GOAL P2-A, unchanged):
- every criterion passed → P2-B starts; the Linux CI job required (human
  ruleset step); the installation packages "publishable" in RELEASE.md (the
  tag is a human step);
- A-4 or A-5 failed (the others passed) → a platform-scope decision request;
  P2-B starts, the installation item stays open;
- A-1, A-2, A-3, A-7 or A-8 failed → the round failed, decision request; P2-B
  does not start;
- A-6 failed → K-GH.6 reopens (option (b)), P2-B starts;
- an undefined failure → the round failed, decision request.

    python -m eval.report_p2a --run-dir <RUN_DIR> --pair <pair_compare.json> --out <FINAL.md>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

CRITERIA = ["A-1", "A-2", "A-3", "A-4", "A-5", "A-6", "A-7", "A-8"]
THRESHOLDS = {
    "A-1": "hepsi geçer",
    "A-2": "6/6",
    "A-3": "sözleşme testi + `contract_check` yeşil",
    "A-4": "2/2",
    "A-5": "rapor",
    "A-6": "geçer",
    "A-7": "yazılı ve testli; bilgi",
    "A-8": "eşit; hepsi",
}


def branch(passed: dict[str, bool]) -> tuple[str, str]:
    failed = {c for c, p in passed.items() if not p}
    if not failed:
        return (
            "hepsi geçti",
            'P2-B başlar; Linux CI işi zorunlu (insan ruleset adımı); kurulum paketleri RELEASE.md\'de "yayımlanabilir" (etiket insan adımı).',
        )
    if failed <= {"A-4", "A-5"}:
        return (
            "A-4 veya A-5 kaldı",
            "platform kapsamı karar talebi; P2-B başlar, kurulum maddesi açık kalır.",
        )
    if failed & {"A-1", "A-2", "A-3", "A-7", "A-8"}:
        return ("A-1, A-2, A-3, A-7 veya A-8 kaldı", "tur kaldı, karar talebi; P2-B başlamaz.")
    if failed == {"A-6"}:
        return (
            "A-6 kaldı",
            "K-GH.6 yeniden açılır (seçenek (b): platforma bağlı davranış), P2-B başlar.",
        )
    return ("tanımlanmamış kalış", "tur kaldı, karar talebi.")


def summary(c: str, r: dict, pair: dict) -> str:
    if c == "A-1":
        t = r["migration_tests"]
        return f"fikstür ileri/geri bayt eşit ({', '.join(r['fixture']['applied'])}); göç testleri {t['passed']}/{t['total']}"
    if c == "A-2":
        return f"{r['passed']} eski vault; `rootmark conformance` {r['conformance']['passed']}/{r['conformance']['total']}"
    if c == "A-3":
        t = r["status_tests"]
        return f"durum/sözleşme testleri {t['passed']}/{t['total']}; `contract_check` {'yeşil' if r['contract_check']['pass'] else 'kırmızı'}"
    if c == "A-4":
        j = r["jobs"]
        s = ", ".join(
            f"{t} {'ok' if v['ok'] else 'FAIL'} {v['conformance']['passed']}/{v['conformance']['total']}"
            for t, v in sorted(j.items())
        )
        return f"{s}; köken tasdiki {r['attestations_verified']} arşivde doğrulandı ({r['run']})"
    if c == "A-5":
        return f"{r['result'].split(':')[0]}; karar talebi {r['decision_request']}"
    if c == "A-6":
        return f"bu makinede {r['this_machine']['passed']}/{r['this_machine']['total']}; Linux CI {r['linux_ci'].get('conclusion')} ({r['linux_ci'].get('run')})"
    if c == "A-7":
        t = r["tests"]
        return f"A0 testleri {t['passed']}/{t['total']}; bilgi ölçümü kayıtlı"
    if c == "A-8":
        p = f"çift: {pair['n_fields']} alan, fark {len(pair['differing_fields'])}, baytlar {'eşit' if pair['bytes_equal'] else 'FARKLI'}"
        return f"{p}; uyumluluk {r['conformance']['passed']}/{r['conformance']['total']} seviye {r['conformance']['level_claimed']}; `check_frozen` çıkış {r['check_frozen']['exit']}, `check_deps` çıkış {r['check_deps']['exit']}"
    return ""


def render(run_dir: Path, pair: dict) -> str:
    chk = json.loads((run_dir / "p2a_checks.json").read_text(encoding="utf-8"))
    passed = {c: chk[c]["pass"] is True for c in CRITERIA}
    passed["A-8"] = passed["A-8"] and pair["equal"] is True
    name, then = branch(passed)
    info = chk["A-7"]["information"]
    lines = [
        "# P2-A — FINAL (göç ve sürüm uyumu, kurulum; geliştirme turu)",
        "",
        f"RUN `{run_dir.name}`; çift karşılaştırması `pair_compare.json` (A ↔ B). Üreteç: `research/eval/report_p2a.py`; elle eklenen notlar en sonda, kaynaklarıyla.",
        "",
        "## Kriterler (GOAL P2-A, önceden kayıtlı)",
        "| Kod | Eşik | Ölçülen | Sonuç |",
        "|---|---|---|---|",
    ]
    for c in CRITERIA:
        lines.append(
            f"| {c} | {THRESHOLDS[c]} | {summary(c, chk[c], pair)} | {'geçti' if passed[c] else '**kaldı**'} |"
        )
    fs, rg, tp = chk["fail_summary"], chk["regress"], chk["tamper"]
    lines += [
        "",
        "## Karar kuralı",
        f"Dal: **{name}** → {then}",
        "",
        "## Bilgi ölçümü (karar kriteri değil; K-P2-02.5 (2))",
        f"D-B test-3 (P2-0.2'nin ön-kayıtlı kümesi, gerçek Obsidian, tek zincir, (A) düzeltmesinden sonra; commit `{info['commit'][:7]}`): yeni blokların W sonunda `unverified` kalanı **{info['unverified_after_window']}/{info['new_blocks']} = {info['value_0_2b']:.4f}** (P2-0.2 resmi: 116/2743 = 0.0423). Eşik yok, karar yok; P2-0.3 hipotez turunun ön verisidir (`reports/rounds/P2-A/info_d_b_test3.json`).",
        "",
        "## Sınıflandırıcı, regress, kasıtlı vakalar",
        f"- FAIL özeti (`eval.historical_p2a`, {fs['reference_run']}): sayılan FAIL **{fs['counted_fails']}**, tarihsel {fs['historical']}, göç gerekli {fs['migration_required']}{'' if not fs['counted'] else ' — sayılanlar: ' + ', '.join(fs['counted'])}.",
        f"- Regress (`eval.compare_p2a regress`, {rg['reference_run']}): {rg['n_fields']} alan, dışlama dışı fark {len(rg['differing_fields'])}, dışlanan farklar {len(rg['excluded_differences'])}, farkı olmayan dışlama {len(rg['exclusions_without_difference'])}; uyumluluk testleri eşit {rg['conformance']['equal']}; D-A, D-A2, D-C günlükleri bayt eşit {rg['p2_0_2_bytes']['equal']}.",
        f"- Kasıtlı vakalar (`p2a/tamper.json`): {', '.join(f'{k} {chr(10003) if v else chr(10007)}' for k, v in tp['cases'].items())}.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--pair", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args(argv)
    pair = json.loads(a.pair.read_text(encoding="utf-8"))
    a.out.write_text(render(a.run_dir, pair), encoding="utf-8")
    print(f"report_p2a: {a.out}")


if __name__ == "__main__":
    main()
