# P1-A — Nihai Rapor

RUN: 20261013T221822Z-27d8ae8 · Commit: 27d8ae8f8888 (temiz) · Rust rustc 1.92.0 (ded5c06cf 2025-12-08) · Python 3.12.9 · SEED 20261013

> `research/eval/report_p1a.py` tarafından yalnız `20261013T221822Z-27d8ae8` RUN dizininden üretildi. Elle sayı yoktur; her tablonun altında kaynak alan yolları vardır. Eşikler `docs/rounds/P1-A/GOAL.md`'den aynen kodlanmıştır.

## 1. Çıkış kriterleri
| Kod | Ölçüm | Eşik | Sonuç | Durum |
|---|---|---|---|---|
| A-1 | C1-01..C1-17 | hepsi geçer | 17/17 (level 1, fixture manifest ok) | GEÇTİ |
| A-2 | Holdout H1-H1 / H1-H2 / H1-H3 / H1-H4 | ≥ 0.90 / ≤ 0.03 / ≥ 0.70 / ≥ 0.90 | 0.993540 / 0.000000 / 0.979381 / 1.000000 | GEÇTİ |
| A-3 | v3 D1-easy, D1-hard, D2 sayıları v3-final ile | birebir | 1577 alan, maks. göreli fark 0.0 | GEÇTİ |
| A-4 | make verify çift koşturma, kalite alanları maks. göreli fark | 0 | 0.0 (1790 alan, 11 dosya) | GEÇTİ |
| A-5 | mini_vault events_hash v2 / v3 | 8914f9b418ae360d49c85c951050f432 / 808a41f21a43e121d7d688fee9f5bc24 | 8914f9b418ae360d49c85c951050f432 / 808a41f21a43e121d7d688fee9f5bc24 | GEÇTİ |
| A-6 | Bağımlılık yönü (cargo tree, import tarayıcı) | ters bağımlılık 0 | ihlal 0; check_deps OK=True; kasıtlı ters bağımlılık testi OK=True | GEÇTİ |
| A-7 | split_hard: THETA_SPLIT ızgarasında farklı amaç değeri sayısı | bilgi (eşik yok) | 3 | bilgi |

Kaynak: A-1 ← `p1a_checks.json:A-1.{passed,total,level_claimed,fixtures_manifest}` · A-2 ← `holdout.json:{H1-H1,H1-H2,H1-H3,H1-H4}` · A-3 ← `regress_vs_v3final.json:{n_fields,max_relative_difference,H4-2_equal}` · A-4 ← `pair_compare.json:{max_relative_difference,n_fields,files,A-4_equal}` · A-5 ← `p1a_checks.json:A-5.events_hash` · A-6 ← `p1a_checks.json:A-6.{violations,check_deps_ok,deliberate_reverse_dependency_test_ok}` · A-7 ← `split_grid.json:A-7`

## 2. Karar kuralı çıktısı
- **A-1..A-6 geçti → spec §1–6 "frozen" damgası teyit edilir, P1-B başlar.**

## 3. A-3 — v3-final regresyonu (seçili alanlar)
| Alan | Değer |
|---|---|
| H1-S1 | 1.0 |
| H1-S2 | 0.0 |
| H1-S3 | 1.0 |
| H1-S4 | 1.0 |
| H1-H1 | 0.9882451721242653 |
| H1-H2 | 0.0 |
| H1-H3 | 0.9967213114754099 |
| H1-H4 | 1.0 |
| H1-G | 4 |
| H1-R1 | 0.9998433399913838 |
| H1-R2 | 0.9817254568635784 |
| H4-V2 equal | True |

Kaynak: `h1s_v3.json:{H1-S1..S4,H1-H1..H4,H1-G}` · `h1r_v3.json:{H1-R1,H1-R2}` · `h4_v2repro.json:H4-V2.equal`; tüm 9 dosyanın alan alan karşılaştırması `regress_vs_v3final.json` (referans RUN `20261013T182004Z-f82dabd`).

## 4. A-2 — Holdout
100 doküman, 1000 puanlı adım · eşleyici v3 (THETA=0.5, THETA_SPLIT=0.65, derlenmiş) · uygun aday 271

| Op | Hedef oran (D1-hard) | Gerçekleşen oran | Adım | Koruma | Yanlış eşleme | Olay recall |
|---|---|---|---|---|---|---|
| dup_delete_adjacent | 0.10 | 0.134 | 134 | 1.0000 | — | — |
| insert | 0.05 | 0.072 | 72 | — | 0.0000 | — |
| merge_edit | 0.15 | 0.128 | 128 | 0.9922 | — | 0.9453 |
| move_edit | 0.10 | 0.119 | 119 | 1.0000 | — | — |
| rephrase_extreme | 0.10 | 0.097 | 97 | 0.9588 | — | — |
| rephrase_heavy | 0.15 | 0.215 | 215 | 1.0000 | — | — |
| rephrase_light | 0.15 | 0.166 | 166 | 1.0000 | — | — |
| split_edit | 0.15 | 0.049 | 49 | 1.0000 | 0.0000 | 1.0000 |
| split_three | 0.05 | 0.020 | 20 | 1.0000 | 0.0000 | 1.0000 |

Olay F1 ayrıntısı: P 0.9948 · R 0.9645 · tp 190 / tahmin 191 / gerçek 197 · payda H1 774, H2 161, H4 268

Kesişimler (programatik): holdout∩easy 0, holdout∩hard 0, easy∩hard 0, üçü 0.

Not: gerçekleşen oranlar yeniden çekilişlerden sonradır ve D1-hard tablosundan sapar (easy ∪ hard dışında çok cümleli paragraf az; ASSUMPTIONS A5). H1-H3 olay gerçeklerinin kaynağı: bölme adımı 69, birleştirme adımı 128 — holdout olay F1'i ağırlıkla birleştirmeleri ölçer.

Kaynak: `holdout.json:holdout.{per_op,H1-H3_detail,n_retention,n_false_match,n_dup}` · `holdout.json:{disjoint,realised_ops,eligible_docs}` · hedef oranlar `eval/datasets/synthetic_edits_v2.py:OPS_HARD`

## 5. A-7 — Bölme eşiği zorluk dilimi (bilgi)
60 doküman, 360 puanlı adım · THETA=0.5 sabit · derlenmiş THETA_SPLIT=0.65 değiştirilmedi · alt dizi kontrolü 292 parça, 0 alt dizi

| THETA_SPLIT | H1-H1 | H1-H2 | H1-H3 | H1-H4 | Amaç | both_edited olay recall | heavy olay recall | short_piece olay recall |
|---|---|---|---|---|---|---|---|---|
| 0.45 | 1.0000 | 0.0000 | 0.9857 | 1.0000 | 3.985714 | 1.0000 | 0.9420 | 1.0000 |
| 0.55 | 1.0000 | 0.0000 | 0.9857 | 1.0000 | 3.985714 | 1.0000 | 0.9420 | 1.0000 |
| 0.65 | 1.0000 | 0.0000 | 0.9857 | 1.0000 | 3.985714 | 1.0000 | 0.9420 | 1.0000 |
| 0.75 | 1.0000 | 0.0000 | 0.9821 | 1.0000 | 3.982079 | 1.0000 | 0.9275 | 1.0000 |
| 0.85 | 1.0000 | 0.0000 | 0.9784 | 1.0000 | 3.978417 | 0.9917 | 0.9203 | 1.0000 |

Farklı amaç değeri sayısı (6 ondalık): **3**.

Kaynak: `split_grid.json:{grid[*],A-7,substring_check,n_docs,n_steps}`

## 6. Koşturma kaydı
| Faz | Commit | Temiz | Çıktılar |
|---|---|---|---|
| h1r | 27d8ae8f8888 | True | h1r.json |
| h1r_v3 | 27d8ae8f8888 | True | h1r_v3.json |
| h1s_test | 27d8ae8f8888 | True | h1s.json |
| h1s_tune | 27d8ae8f8888 | True | h1s_tuning.json |
| h1s_v3_test | 27d8ae8f8888 | True | h1s_v3.json |
| h1s_v3_tune | 27d8ae8f8888 | True | h1s_v3_tuning.json |
| h3_v3 | 27d8ae8f8888 | True | h3_v3.json |
| h4_v2repro | 27d8ae8f8888 | True | h4_v2repro.json |
| h4_v3 | 27d8ae8f8888 | True | h4_v3.json |
| holdout | 27d8ae8f8888 | True | holdout.json |
| p1a_checks | 27d8ae8f8888 | True | p1a_checks.json |
| split_grid | 27d8ae8f8888 | True | split_grid.json |

Kaynak: `manifest.json:phases`
