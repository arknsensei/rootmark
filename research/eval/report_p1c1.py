"""P1-C.1 round report (GOAL C1.4): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
C1-1..C1-9 table and the decision rule of docs/rounds/P1-C.1/GOAL.md,
derived from the rows:

- C1-1 fails → the variant explanation is wrong, there is a bug: P1-D D3
  does not start; decision request.
- C1-1 passes and every other criterion passes → K-C8-T closes, spec §9
  "Validated", the 0.1.0 release block is lifted, P1-D starts in full.
- C1-1 passes and another criterion fails → the rule names no branch:
  reported as such, no branch is assumed.

C1-7's spec version is the one docs/SPEC_EDITS_CRITERION.md's rule gives
(1.0.2; GOAL names 1.1.0 — SPEC_EDITS.md, ASSUMPTIONS C1.0-1); the report
says so. C1-8 = `regress_vs_p1c.json` (K-C1.6) and `pair_compare.json`
(A ↔ B). Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P1-C.1/FINAL.md`.

    python -m eval.report_p1c1 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-C.1" / "FINAL.md"


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool) -> str:
    return "geçti" if b else "**KALDI**"


def evaluate(d: Path) -> list[dict]:
    c = _load(d, "p1c1_checks.json")
    pair = _load(d, "pair_compare.json")
    c11 = c["C1-1"]
    worst = max(abs(v) for cell in c11["cells"].values() for v in cell["delta"].values())
    pq = c11["per_query_equal_to_rust"]
    c18 = c["C1-8"]["regress"]
    return [
        {
            "id": "C1-1",
            "what": "Python tantivy-varyant BM25 + RRF (prototip hattı) ↔ Rust `hybrid`/`flat`, 4 hücre, nDCG@10 / recall@10 / MRR",
            "value": f"en büyük \\|Δ\\| {worst:.6f}; sorgu düzeyinde eşit hybrid {pq['hybrid']['ndcg@10']}/{pq['hybrid']['n']}, flat {pq['flat']['ndcg@10']}/{pq['flat']['n']}",
            "threshold": f"\\|Δ\\| ≤ {c11['threshold']}",
            "pass": c11["pass"],
            "source": "c1_variant.json",
        },
        {
            "id": "C1-2",
            "what": "`include_proposal=true` canonical öneki ↔ `false` sonucu, 1000 sorgu × 3 mod, proposal'lı vault",
            "value": f"{c['C1-2']['canonical_prefix_equal']}/{c['C1-2']['searches']} bayt eşit",
            "threshold": "1000/1000 (× 3 mod)",
            "pass": c["C1-2"]["pass"] and c["C1-2"]["searches"] >= 3000,
            "source": "p1c1/c1_2_3.json",
        },
        {
            "id": "C1-3",
            "what": "proposal hit'ler etiketli, ×0.5, hepsi canonical'lardan sonra (bağımsız yeniden hesapla)",
            "value": f"{c['C1-3']['searches_matching_independent_ranking']}/{c['C1-3']['searches']} arama, "
            f"{c['C1-3']['proposal_hits']} proposal hit; eski karışık istatistiğe duyarlı lexical sorgu "
            f"{c['C1-3']['lexical_queries_sensitive_to_mixed_statistics']}",
            "threshold": "property 1000",
            "pass": c["C1-3"]["pass"],
            "source": "p1c1/c1_2_3.json",
        },
        {
            "id": "C1-4",
            "what": "`search` sonrası `index/` `diff -r` (`.lock` dahil)",
            "value": f"kütüphane {c['C1-4']['library']['searches']} arama, kilit {c['C1-4']['library']['lock_files']}; "
            f"CLI {c['C1-4']['cli']['as_expected']}/{c['C1-4']['cli']['cases']} vaka",
            "threshold": "eşit",
            "pass": c["C1-4"]["pass"],
            "source": "p1c1/c1_4.json, p1c1_cli.json",
        },
        {
            "id": "C1-5",
            "what": "boş / yalnız boşluk sorgu",
            "value": f"{c['C1-5']['library']['rejected']}/{c['C1-5']['library']['tried']} `Err`; "
            f"iz/olay/checkpoint değişmedi {c['C1-5']['library']['traces_unchanged'] and c['C1-5']['library']['events_unchanged'] and c['C1-5']['library']['checkpoints_unchanged']}",
            "threshold": "`Err`, iz satırı yok",
            "pass": c["C1-5"]["pass"],
            "source": "p1c1/c1_5.json, p1c1_cli.json",
        },
        {
            "id": "C1-6",
            "what": "manifest `lexical.{variant,idf,k1,b,tokenizer}`; eksik/farklı → `index status` hata",
            "value": f"`{json.dumps(c['C1-6']['lexical'], ensure_ascii=False)}`; {c['C1-6']['rejected_cases']} bozma reddedildi",
            "threshold": "var; eksikse hata",
            "pass": c["C1-6"]["pass"],
            "source": "p1c1/c1_6.json, p1c1_cli.json",
        },
        {
            "id": "C1-7",
            "what": "uyumluluk C1/C2/C3, spec sürümü, `level_claimed`",
            "value": f"C1 {c['C1-7']['C1']}, C2 {c['C1-7']['C2']}, C3 {c['C1-7']['C3']}, "
            f"spec {c['C1-7']['spec_version']}, level {c['C1-7']['level_claimed']}",
            "threshold": f"17/17, 9/9, 5/5, spec {c['C1-7']['spec_version_expected']} (GOAL: 1.1.0 → CRITERION kuralı), level 3",
            "pass": c["C1-7"]["pass"],
            "source": "conformance_p1c1.json",
        },
        {
            "id": "C1-8",
            "what": "çift koşturma (A ↔ B, hariçsiz) ve P1-C resmi RUN'ı ile kalite alanları (K-C1.6 hariçleri)",
            "value": f"A↔B {pair['n_fields']} alan, farklı {len(pair['differing_fields'])}, baytlar eşit {pair['bytes_equal']}; "
            f"P1-C ile {c18['n_fields']} alan, hariç dışında farklı {len(c18['differing_fields'])}",
            "threshold": "fark 0",
            "pass": pair["equal"] and c["C1-8"]["pass"],
            "source": "pair_compare.json, regress_vs_p1c.json",
        },
        {
            "id": "C1-9",
            "what": "`research/data/quality_ref_rust.json` ↔ bu RUN'ın `c3_quality.json`'ı (hücreler + sorgu düzeyi)",
            "value": f"{c['C1-9']['compared']} değer, farklı {len(c['C1-9']['differing'])}",
            "threshold": "0",
            "pass": c["C1-9"]["pass"],
            "source": "c3_quality.json, quality_ref_rust.json",
        },
    ]


def decision(rows: list[dict]) -> list[str]:
    ok = {r["id"]: r["pass"] for r in rows}
    if not ok["C1-1"]:
        return [
            (
                "C1-1 kaldı → varyant açıklaması yanlış, hata var; **P1-D D3 başlamaz**; karar talebi. "
                "Referans değişmez (K-C1.2)."
            )
        ]
    if all(ok.values()):
        return [
            (
                'C1-1 geçti ∧ diğerleri geçti → **K-C8-T kapanır**, spec §9 **"Validated"**, '
                "**0.1.0 yayım engeli kalkar**, **P1-D tam başlar** (D3 dahil)."
            )
        ]
    bad = [k for k, v in ok.items() if not v]
    return [
        (
            f"C1-1 geçti; {', '.join(bad)} kaldı → karar kuralı bu durumu adlandırmaz; dal varsayılmaz, "
            'karar talebi gerekir. Spec §9 "Draft" kalır.'
        )
    ]


def render(d: Path) -> str:
    man = _load(d, "manifest.json")
    rows = evaluate(d)
    c = _load(d, "p1c1_checks.json")
    lines = [
        "# P1-C.1 — Sonuç raporu",
        "",
        f"- RUN: `{d.name}` · ölçülen commit `{commit_of(man)}`",
        "- Üretim: `python -m eval.report_p1c1 --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).",
        "",
        "## Çıkış kriterleri (GOAL P1-C.1)",
        "",
        "| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |",
        "|---|---|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f"| {r['id']} | {r['what']} | {r['value']} | {r['threshold']} | {_ok(r['pass'])} | `{r['source']}` |"
        )
    lines += [
        "",
        "## C1-1 hücreleri (`c1_variant.json`)",
        "",
        "| Hücre | Δ nDCG@10 | Δ recall@10 | Δ MRR | Okapi ↔ v2-final Δ nDCG@10 |",
        "|---|---|---|---|---|",
    ]
    for key, cell in sorted(c["C1-1"]["cells"].items()):
        dl = cell["delta"]
        ok = c["C1-1"]["okapi_delta_vs_v2"][key]["ndcg@10"]
        lines.append(
            f"| {key} | {dl['ndcg@10']:+.6f} | {dl['recall@10']:+.6f} | {dl['mrr']:+.6f} | {ok:+.6f} |"
        )
    lines += [
        "",
        (
            "Aynı prototip hattı (D3 chunk'ları, `sentence-transformers` vektörleri, prototip tokenları, "
            "RRF 60/50) yalnız BM25 varyantı değiştirilerek koşturulur: Lucene/tantivy varyantı Rust "
            "sayılarını, prototipin `BM25Okapi`'si v2-final sayılarını üretir. `eval/bm25_lucene.py` "
            "formülden yazıldı; işlem sırası Rust'ınkine bilerek eşitlendi (C1.1 gözden geçirme notu 1). "
            "Kanıt, Rust indeksinin **bildirdiği** varyantla (tam token uzunluğu, fieldnorm'suz) eşleşmedir."
        ),
        "",
        "## C1-8 karar gereği hariç tutulan alanlar (`regress_vs_p1c.json`, K-C1.6)",
        "",
        *([f"- `{k}`" for k in c["C1-8"]["regress"]["differing_excluded"]] or ["- (fark yok)"]),
        "",
        "## Notlar",
        "",
        (
            "- C1-7: spec sürümü `docs/SPEC_EDITS_CRITERION.md` sürüm kuralıyla **1.0.2** (GOAL `1.1.0` "
            "öngörür; PROMPT gereği CRITERION kazanır; gerekçe `docs/rounds/P1-C.1/progress/SPEC_EDITS.md`)."
        ),
        (
            "- C3-01..04 beklenen değerleri yalnız `include_proposal: true` aramalarında değişti; liste uzunluğu "
            "farkı K-C1.3'ün katman başına `k` kuralından (ASSUMPTIONS C1.2-1), sıra farkı canonical-yalnız "
            'istatistikten (SPEC_EDITS "Fixture değişiklikleri").'
        ),
        "",
        "## Karar kuralı (GOAL P1-C.1)",
        "",
        *decision(rows),
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    try:
        text = render(a.run_dir)
    except (FileNotFoundError, RuntimeError, KeyError) as e:
        die(str(e))
    (a.run_dir / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {a.run_dir / 'FINAL.md'} and {OUT}")


if __name__ == "__main__":
    main()
