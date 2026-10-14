# P1-B.1 — Sonuç raporu

- RUN: `20261014T212243Z-226b511` · ölçülen commit `226b511bda34c1e09a9f8e43890df8ac85b9dfec`
- Üretim: `python -m eval.report_p1b1 --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).

## Çıkış kriterleri (GOAL P1-B.1)

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| B1-1 | v3-final kütüphane API'siyle 9 senaryo mevcut çekirdekle açılır (replay, events_hash, blame) | check_v3_compat 9/9 senaryo; compat_v3final 9/9 test | 9/9 | geçti | `v3_compat_mixed.json, p1b1_tests.log` |
| B1-2 | yeni biçimli günlükte eksik `LayerChanged` | deleted_layer_changed_line_is_an_error ok, missing_layer_changed_in_a_layered_log_is_an_error ok, missing_second_layer_changed_is_an_error ok | `Err` | geçti | `p1b1_tests.log` |
| B1-3 | C1 17/17, C2 8/8 (C2-07 yeni beklenen), `level_claimed 2` | C1 17/17, C2 8/8, level 2, spec 1.0.1 | geçer | geçti | `conformance.json` |
| B1-4 | `trace_id` doğrulaması: 31 / 33 / hex olmayan `query_hash`; `consumer` içinde `\n` | invalid_inputs_are_refused ok, invalid_trace_lines_are_refused_on_open ok | 4/4 `Err` | geçti | `p1b1_tests.log` |
| B1-5 | kayma: (`"ab"`, qh) ve (`"a"`, `"b"+qh`) | formula_has_separators ok, shifted_fields_get_different_ids ok | farklı (ikincisi `Err`) | geçti | `p1b1_tests.log` |
| B1-6 | P1-B resmi RUN'ı ile kalite alanları (C2-07, `traces.jsonl` hariç) | 2195 alan; hariç tutulan farklar 5, dışında 0; bayt: checkpoints/ =, events.jsonl =, traces.jsonl ≠ (hariç) | fark 0 | geçti | `regress_vs_p1b.json` |
| B1-7 | çift koşturma (`make verify` pair, compare_p1b) | 2195 alan, max fark 0.0, bayt eşit True | fark 0, bayt eşit | geçti | `pair_compare.json` |

## B1-1 senaryoları (`v3_compat_mixed.json`)

| Senaryo | Doküman | blame eşit | Sonuç |
|---|---|---|---|
| agent_only | 1 | 1 | geçti |
| transcript_only | 1 | 1 | geçti |
| synthetic_only | 1 | 1 | geçti |
| human_synthetic | 1 | 1 | geçti |
| agent_human | 1 | 1 | geçti |
| human_agent | 1 | 1 | geçti |
| human_transcript | 1 | 1 | geçti |
| synthetic_agent | 1 | 1 | geçti |
| mixed_vault | 4 | 4 | geçti |

## B1-6 hariç tutulan alanlar (`regress_vs_p1b.json`)

- `governance/governance.json:traces_sha256`
- `p1b_checks.json:B-1›tests›6›actual_hash`
- `p1b_checks.json:B-1›tests›6›expected_hash`
- `p1b_checks.json:fixtures_manifest_hash`
- `p1b_checks.json:governance›traces_sha256`

C2-07 yeni beklenen hash: `03f9907946bb4e238ae6937604c33e58`.

## Karar kuralı (GOAL P1-B.1 V8)

B1-1..B1-7 geçti → P1-B karar kuralının B-5 dalı kapanır, **P1-C başlar**.
