# P1-C — Görevler

Sıra: C0 → C1 → C2 → C3 → C4 → C5 → C6 → C7 → C8 → C9. Progress `docs/rounds/P1-C/progress/`. Commit öneki `P1-C/C<n>:`.

## C0 — Tur açılışı
**Yapar:** `git tag p1b1-final`; `ROUND_START`; `index/CLAUDE.md`; `index/Cargo.toml` bağımlılıklar (`tantivy`, `ort`, `tokenizers`, `ndarray`; sürümler sabit, ≥ 14 gün önce yayımlanmış); `scripts/fetch_model.sh` + `models/MANIFEST.json` (ONNX dosyası, tokenizer dosyaları, blake3); `make models` (manifest doğrular); CI model önbelleği.
**Kabul:** `cargo build -p mdkernel-index`; `make models` manifest OK; `check_deps.sh` temiz.

## C1 — Embedder
**Yapar:** `Embedder` trait; `OnnxMiniLm` (mean pooling, L2 normalize, tek iş parçacığı, sabit `intra_threads=1`); `embedder_id = "minilm-l6-v2/" + model_blake3[:8]`; `research/data/embed_ref.json` (50 chunk, prototip `sentence-transformers` ile; **bu dosya `research/` içinde üretilir ve commit'lenir**).
**Kabul:** C-10; aynı metin 100 kez aynı f32 vektör (bit eşit); batch ve tek tek embed sonucu bit eşit.

## C2 — Sözcüksel indeks
**Yapar:** tantivy şeması; deterministik yazım ve tek segment; `lexical(query, k, at)`; tokenizer eşdeğerlik testi (prototip `[a-z0-9]+` ile 200 chunk'ta aynı token listesi, `research` betiğiyle üretilmiş referans `research/data/tok_ref.json`).
**Kabul:** indeks dizini iki kurulumda bayt eşit; sıralama 100 tekrar eşit; tokenizer referansla eşit.

## C3 — Vektör deposu ve hibrit
**Yapar:** vektör deposu; `flat`, `hybrid` (RRF); `Hit`; eşitlik kırma; `embedder_id` uyuşmazlığı `Err`.
**Kabul:** C-4 (üç mod); RRF birim testi (elle örnek); `embedder_id` karışımı `Err`.

## C4 — Kontrol noktası, katman, iz
**Yapar:** chunk kaydında `snapshot_id`, `layer`; `at` filtresi; `include_proposal`; `derived` asla; `record_trace` çağrısı; `search` iz yazar.
**Kabul:** C-7 property testi; checkpoint sonrası ingest → `at` eski checkpoint sonuçları değişmez; her `search` bir iz satırı (`trace_id` K-B1.2 formülü, bağımsız hesapla test).

## C5 — Sıcak katman ve izleyici
**Yapar:** sıcak depo; `watch(doc)`; birleştirme; `compact()`; `index status`.
**Kabul:** C-8 (200 doküman, tohumlu düzenleme, p50/p95 raporda); C-9; yalnız etkilenen chunk'lar embed ediliyor (sayaç).

## C6 — CLI
**Yapar:** `index build/status/compact/watch`, `search`; JSON çıktı; `--at`, `--include-proposal`, `--mode`.
**Kabul:** `core/tests/cli_index.rs` uçtan uca; `search --at` eski checkpoint; `--include-proposal` etiketli.

## C7 — Uyumluluk C3 ve spec
**Yapar:** `spec/fixtures/C3-01..04/` (mini_vault + proposal'lı blok + 10 sorgu; beklenen sıralamalar); `gen_fixtures_c3.sh`; spec §9 açıklık düzeltmeleri `SPEC_EDITS.md` (K-B1.3 ölçütüyle).
**Kabul:** C-1, C-2; C3 beklenen değerleri yeniden üretilebilir (`git status` temiz).

## C8 — Kalite regresyonu (`research/`)
**Yapar:** `research/eval/run_c3_quality.py` (yeni dosya): D3 ingest (bağlama ile), `index build`, D4 single/multi test bölümü, `hybrid/structural` nDCG@10, recall@10, MRR; v2 referans `research/reports/runs/20261013T115311Z…` (v2-final) `hybrid/structural` hücreleriyle Δ; yalnız `hybrid` ve `flat` modları (graf yok).
**Kabul:** C-3; sorgu sayısı hücre başına beklenen (fail-fast); `FROZEN.md` yeni satırlar.

## C9 — Doğrulama zinciri, rapor, hazır
**Yapar:** `verify.sh`: models → conformance L3 → regress (P1-B.1 seti + C-3) → determinism (C-4/5/6/9) → hot (C-8) → property (C-7) → tamper (model manifest bozma, `embedder_id` karışımı, fixture bozma) → pair compare (`index/` bayt, 200 sorgu sıralaması); `report_p1c.py`; `FINAL.md`; `READY.md`.
**Kabul:** C-6, C-11; izlenebilirlik; karar kuralı.
