# P1-B — FINAL

RUN `20261014T135400Z-7735a07` · commit `7735a07775d7c201604ac49cd560891377035aa8` · üretildi: `python -m eval.report_p1b` (yalnız bu RUN dizini).

## Çıkış kriterleri (GOAL, önceden kayıtlı)

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak (RUN içinde) |
|---|---|---|---|---|---|
| B-1 | C2-01..C2-08 | 8/8, level_claimed 2 | 8/8; level_claimed 2 | geçti | `p1b_checks.json B-1, level_claimed` |
| B-2 | C1-01..C1-17 | 17/17 | 17/17 | geçti | `p1b_checks.json B-2` |
| B-3 | regression: v3-final 9 files + P1-A holdout/split_grid | max rel. diff 0.0 (1577 fields) / 0.0 (1790 fields) | 0 / 0 | geçti | `p1b_checks.json B-3 (regress_vs_v3final.json, regress_vs_p1a.json)` |
| B-4 | make verify pair: quality fields + traces.jsonl, checkpoints/ | max rel. diff 0.0 (2195 fields); governance/vault/.mdkernel/checkpoints/ =, governance/vault/.mdkernel/events.jsonl =, governance/vault/.mdkernel/traces.jsonl = | 0, byte-equal | geçti | `pair_compare.json` |
| B-5 | v3-final vault opens: replay snapshot_id, events_hash | 12 docs, replay equal True, events_hash f3cb01242e8cb7123e417a72a196250f → f3cb01242e8cb7123e417a72a196250f | equal | geçti | `p1b_checks.json B-5 (v3_compat.json)` |
| B-6 | policy bound violations (6 cases) | 6 cases, all Err True, no fallback True | 6 Err, none silent | geçti | `p1b_checks.json B-6 (policy_bounds.json)` |
| B-7 | Ö2: moved target/ + broken fixture | 1 case(s) detected; fixtures_root of the copy True | make conformance error | geçti | `tamper.log` |
| B-8 | Ö1: FROZEN.md row deleted / relaxed | 3 case(s) detected; clean clone exit 0 True | check_frozen.sh error | geçti | `tamper.log` |
| B-9 | d1hs H1-H1 / H1-H2 / H1-H3 | 0.992874 / 0.000000 / 0.997222 | ≥ 0.90 / ≤ 0.03 / ≥ 0.70 | geçti | `holdout_split.json b9` |
| B-10 | layer transitions = replay (seeded fuzz) | seed 20261014: 1000 steps, 80 lineages, replay equal True, seed 7: 1000 steps, 68 lineages, replay equal True | equal | geçti | `p1b_checks.json B-10 (fuzz_<seed>.json)` |

## d1hs (Ö3)

- Kaynak: `https://github.com/rust-lang/book` @ `1500248d8f23` `src/*.md` (102 uygun, 60 doküman, 600 puanlanan adım) — ASSUMPTIONS B8-1, DECISIONS K20-T.
- Gerçekleşen oranlar: split_edit 0.295 (≥ 0.25), split_three 0.205 (≥ 0.10). `holdout_split.json realised_ops`
- Kesişimler: {"d1ho_and_easy": 0, "d1ho_and_hard": 0, "d1hs_and_d1ho": 0, "d1hs_and_easy": 0, "d1hs_and_hard": 0, "d1hs_texts_equal_to_d3": 0, "easy_and_hard": 0} `holdout_split.json disjoint`
- H1 kodları: H1-H1 0.992874, H1-H2 0.000000, H1-H3 0.997222, H1-H4 1.000000
- Yan yana (aynı RUN, aynı ölçüt): d1ho H1-H1 0.993540, H1-H2 0.000000, H1-H3 0.979381, H1-H4 1.000000 `holdout.json`. Çekince: d1hs tabanları D2 korpusu (v3-final H1-R'de değerlendirme için kullanıldı); bölümler dilimi kolaylaştırıyor olabilir (K20-T).

## Karar kuralı (GOAL'den türetildi)

- B-1..B-8 ∧ B-10 geçti → spec §7, §8, §11 "Validated"; P1-C başlar.

## Yönetişim senaryosu (B-4 bayt karşılaştırmasının konusu)

- `events_hash` 2b0dc8e88f8d56ee4c6be6356d4c0fee (127 olay), `traces.jsonl` 4 satır (sha256 dd6e4400ad8f8840…), 4 checkpoint (baseline, proposed, reviewed, final). `governance/governance.json`
- Kasıtlı bozmalar: 12 yakalandı. `tamper.log`
