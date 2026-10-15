# P1-C — Sonuç raporu

- RUN: `20261015T014417Z-9aa326b` · ölçülen commit `9aa326b01df9c85451f7580f4f0c0e46a033945a`
- Üretim: `python -m eval.report_p1c --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).
- Embedder: `minilm-l6-v2/e73168f9` (model manifest OK: True).

## Çıkış kriterleri (GOAL P1-C + ADDENDUM)

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| C-1 | C3-01..C3-04, `level_claimed 3` | C3 4/4, level 3, spec 1.0.1 | 4/4, level 3 | geçti | `conformance_full.json` |
| C-2 | C1 17/17, C2 9/9 (ADDENDUM) | C1 17/17, C2 9/9 | geçer | geçti | `conformance_full.json, c13_tests.log` |
| C-3 | D3 + D4 test, `hybrid/structural` nDCG@10 Δ vs v2-final (single, multi) | single 0.6769 (v2 0.6740, Δ +0.0029); multi 0.4800 (v2 0.4936, Δ -0.0136) | \|Δ\| ≤ 0.01 | **KALDI** | `c3_quality.json` |
| C-4 | aynı sorgu, checkpoint, indeks: 100 tekrar (3 mod, D3) | 10 sorgu × 3 mod × 100 | birebir | geçti | `p1c_checks.json` |
| C-5 | indeks silinip aynı checkpoint'ten yeniden kurulunca sıralama (200 sorgu × 3 mod) | sıralama eşit True, indeks dosyaları eşit True | birebir | geçti | `p1c_checks.json` |
| C-6 | çift koşturma: `index/` bayt (dosya hash'leri), 200 sorgunun sıralaması, tüm kalite alanları | 99053 alan, en büyük fark 0.0, farklı 0, yönetişim baytları eşit True | eşit | geçti | `pair_compare.json` |
| C-7 | `proposal` varsayılanda yok; `include_proposal` ile etiketli ve sonda; `derived` asla | 1000 sorgu, 297 proposal hit (chunk katmanları {'canonical': 32, 'derived': 1, 'proposal': 7}) | property 1000 sorgu | geçti | `c7_property.json` |
| C-8 | yazma→sıcakta aranabilir (200 doküman, ONNX CPU, model yüklü) | p50 28.5 ms, p95 32.2 ms (200 düzenleme; makineye bağlı) | p95 < 100 ms | geçti | `c8_latency.json` |
| C-9 | `compact()` öncesi/sonrası 200 sorgu sıralaması (iki checkpoint, 3 mod) | 20 sıcak düzenleme, taşınan 20, eşit True | birebir | geçti | `p1c_checks.json` |
| C-10 | 50 sabit chunk: ONNX ↔ prototip sentence-transformers kosinüs | en düşük 0.9999999999992 (n 50) | ≥ 0.999 | geçti | `c10_embedding.json` |
| C-11 | P1-B.1 kalite alanları (P1-B.1 resmi RUN ile) | karar gereği hariç dışında farklı 0 | fark 0 | geçti | `regress_vs_p1b1.json` |
| C-12 | C2-09 (ADDENDUM; C-2'nin parçası) | C2-09 pass | geçer | geçti | `conformance_full.json` |
| C-13 | yeniden numaralı silme: iki `Err` + bir türetilmiş önek (ADDENDUM; C-2'nin parçası) | 3/3 | 3/3 | geçti | `c13_tests.log` |

## C-3 hücreleri (`c3_quality.json`; v2-final `20261013T154851Z-f133d0b`)

| Hücre | n | nDCG@10 | v2 | Δ | recall@10 Δ | MRR Δ |
|---|---|---|---|---|---|---|
| flat/structural/multi | 124 | 0.3613 | 0.3613 | +0.0000 | +0.0000 | +0.0000 |
| flat/structural/single | 173 | 0.5261 | 0.5261 | +0.0000 | +0.0000 | +0.0000 |
| hybrid/structural/multi | 124 | 0.4800 | 0.4936 | -0.0136 | -0.0242 | -0.0032 |
| hybrid/structural/single | 173 | 0.6769 | 0.6740 | +0.0029 | -0.0116 | +0.0069 |

`flat` hücrelerinde nDCG@10, recall@10, MRR Δ 0: embedder ve kesin kosinüs prototipi üretir; `hybrid` farkı sözcüksel taraftan (BM25 varyantı; DECISIONS K-C8-T).

## Kasıtlı bozma (`tamper_p1c.log`): 6/6 ret

- model manifest blake3 changed — error: /var/folders/3h/tkg8gmr96tscr2t77stj77v40000gn/T/mdk-tamper-p1c.XXXXXX.dr9hKQiQvf/m1/minilm-l6-v2/tokenizer.json: blake3 82483bb4f0bdb81779f295ecc5a93285
- model file changed — error: /var/folders/3h/tkg8gmr96tscr2t77stj77v40000gn/T/mdk-tamper-p1c.XXXXXX.dr9hKQiQvf/m2/minilm-l6-v2/sentence_bert_config.json: size 54 != manifest 53
- embedder_id mixed in embedders.jsonl — error: vector store mixes embedder ids ["minilm-l6-v2/e73168f9", "other/00000000"]
- index manifest names another embedder — error: vectors carry embedder_id Some("minilm-l6-v2/e73168f9"), manifest "other/00000000"
- C3 fixture byte changed (manifest mismatch) — error: level 1-2 checker failed (manifest mismatch)
- C3 expected ranking changed (manifest updated) — error: level 3 checker failed (C3-01)

## C-11 karar gereği hariç tutulan alanlar (`regress_vs_p1b1.json`)

- `p1b1_checks.json:tests_seen`
- `p1b_checks.json:fixtures_manifest_hash`

## Karar kuralı (GOAL P1-C)

C-3 kaldı → embedder/tokenizer uyumsuzluğu; karar talebi; **P1-D başlar** (MCP indeks kalitesine bağlı değil), **0.1.0 yayımı bekler**.
Spec §9 "Validated" damgası almaz (C-1..C-11 hepsi geçmedi); §9 "Draft" kalır.
