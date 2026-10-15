# P1-C.1 — Sonuç raporu

- RUN: `20261015T094412Z-b9f7622` · ölçülen commit `b9f7622aafcb397c1bce9bd8ea50ae41f415d3e2`
- Üretim: `python -m eval.report_p1c1 --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).

## Çıkış kriterleri (GOAL P1-C.1)

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| C1-1 | Python tantivy-varyant BM25 + RRF (prototip hattı) ↔ Rust `hybrid`/`flat`, 4 hücre, nDCG@10 / recall@10 / MRR | en büyük \|Δ\| 0.000000; sorgu düzeyinde eşit hybrid 297/297, flat 297/297 | \|Δ\| ≤ 0.001 | geçti | `c1_variant.json` |
| C1-2 | `include_proposal=true` canonical öneki ↔ `false` sonucu, 1000 sorgu × 3 mod, proposal'lı vault | 3000/3000 bayt eşit | 1000/1000 (× 3 mod) | geçti | `p1c1/c1_2_3.json` |
| C1-3 | proposal hit'ler etiketli, ×0.5, hepsi canonical'lardan sonra (bağımsız yeniden hesapla) | 3000/3000 arama, 14529 proposal hit; eski karışık istatistiğe duyarlı lexical sorgu 980 | property 1000 | geçti | `p1c1/c1_2_3.json` |
| C1-4 | `search` sonrası `index/` `diff -r` (`.lock` dahil) | kütüphane 24 arama, kilit 0; CLI 45/45 vaka | eşit | geçti | `p1c1/c1_4.json, p1c1_cli.json` |
| C1-5 | boş / yalnız boşluk sorgu | 96/96 `Err`; iz/olay/checkpoint değişmedi True | `Err`, iz satırı yok | geçti | `p1c1/c1_5.json, p1c1_cli.json` |
| C1-6 | manifest `lexical.{variant,idf,k1,b,tokenizer}`; eksik/farklı → `index status` hata | `{"b": 0.75, "idf": "ln(1 + (N - n + 0.5) / (n + 0.5))", "k1": 1.2, "tokenizer": "mdk_lower_alnum", "variant": "lucene-bm25"}`; 12 bozma reddedildi | var; eksikse hata | geçti | `p1c1/c1_6.json, p1c1_cli.json` |
| C1-7 | uyumluluk C1/C2/C3, spec sürümü, `level_claimed` | C1 17/17, C2 9/9, C3 5/5, spec 1.0.2, level 3 | 17/17, 9/9, 5/5, spec 1.0.2 (GOAL: 1.1.0 → CRITERION kuralı), level 3 | geçti | `conformance_p1c1.json` |
| C1-8 | çift koşturma (A ↔ B, hariçsiz) ve P1-C resmi RUN'ı ile kalite alanları (K-C1.6 hariçleri) | A↔B 99341 alan, farklı 0, baytlar eşit True; P1-C ile 99053 alan, hariç dışında farklı 0 | fark 0 | geçti | `pair_compare.json, regress_vs_p1c.json` |
| C1-9 | `research/data/quality_ref_rust.json` ↔ bu RUN'ın `c3_quality.json`'ı (hücreler + sorgu düzeyi) | 618 değer, farklı 0 | 0 | geçti | `c3_quality.json, quality_ref_rust.json` |

## C1-1 hücreleri (`c1_variant.json`)

| Hücre | Δ nDCG@10 | Δ recall@10 | Δ MRR | Okapi ↔ v2-final Δ nDCG@10 |
|---|---|---|---|---|
| flat/structural/multi | +0.000000 | +0.000000 | +0.000000 | +0.000000 |
| flat/structural/single | +0.000000 | +0.000000 | +0.000000 | +0.000000 |
| hybrid/structural/multi | +0.000000 | +0.000000 | +0.000000 | +0.000000 |
| hybrid/structural/single | +0.000000 | +0.000000 | +0.000000 | +0.000000 |

Aynı prototip hattı (D3 chunk'ları, `sentence-transformers` vektörleri, prototip tokenları, RRF 60/50) yalnız BM25 varyantı değiştirilerek koşturulur: Lucene/tantivy varyantı Rust sayılarını, prototipin `BM25Okapi`'si v2-final sayılarını üretir. `eval/bm25_lucene.py` formülden yazıldı; işlem sırası Rust'ınkine bilerek eşitlendi (C1.1 gözden geçirme notu 1). Kanıt, Rust indeksinin **bildirdiği** varyantla (tam token uzunluğu, fieldnorm'suz) eşleşmedir.

## C1-8 karar gereği hariç tutulan alanlar (`regress_vs_p1c.json`, K-C1.6)

- `p1b1_checks.json:criteria›B1-3›pass`
- `p1b1_checks.json:criteria›B1-3›spec_version`
- `p1b_checks.json:fixtures_manifest_hash`
- `p1c_checks.json:C-11›differing_outside_exclusions`
- `p1c_checks.json:C-11›pass`
- `p1c_checks.json:C-1›spec_version`
- `p1c_checks.json:C-7›proposal_hits_with_include_proposal`
- `p1c_index.json:index_files_build›manifest·json`
- `p1c_index.json:index_files_compact›manifest·json`
- `p1c_index.json:index_files_final›manifest·json`

## Notlar

- C1-7: spec sürümü `docs/SPEC_EDITS_CRITERION.md` sürüm kuralıyla **1.0.2** (GOAL `1.1.0` öngörür; PROMPT gereği CRITERION kazanır; gerekçe `docs/rounds/P1-C.1/progress/SPEC_EDITS.md`).
- C3-01..04 beklenen değerleri yalnız `include_proposal: true` aramalarında değişti; liste uzunluğu farkı K-C1.3'ün katman başına `k` kuralından (ASSUMPTIONS C1.2-1), sıra farkı canonical-yalnız istatistikten (SPEC_EDITS "Fixture değişiklikleri").

## Karar kuralı (GOAL P1-C.1)

C1-1 geçti ∧ diğerleri geçti → **K-C8-T kapanır**, spec §9 **"Validated"**, **0.1.0 yayım engeli kalkar**, **P1-D tam başlar** (D3 dahil).
