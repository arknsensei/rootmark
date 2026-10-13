# 02 — Görev Grafı (v2)

Sıra: T0 → T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12 → T13 → T14 → T15 → T16 → T17 → T18.
Park kuralı `CLAUDE.md`'de. T12 park edilirse T13–T16 devam eder, T17 bekler.

---

## T0 — İskelet
**Yapar:** Cargo workspace (`core/`, `bindings/`), `rust-toolchain.toml`, `pyproject.toml` (maturin backend), `Makefile` (tüm hedefler; eval hedefleri `RUN` yoksa `exit 1`), `.gitignore` (`data/` hariç `data/MANIFEST.*.json`, `data/d4/`, `data/splits/`), `docs/progress/`, `ASSUMPTIONS.md`, `FREEZE_EXCEPTIONS.md` (boş), `tests/fixtures/mini_vault/` (12 doküman: wikilink'li, tekrar bloklu, iç içe listeli, kodlu, tablolu; elle yazılır), `tests/fixtures/freeze_fixture/` (T9'da doldurulur).
**Kabul:** `cargo build`, `cargo test` (boş), `maturin develop --release` başarılı, `python -c "import mdkernel_core"`. `make test` her ikisini koşturur. İlk commit.

## T1 — Parser ve normalizasyon (Rust)
**Yapar:** `core/src/{model,parse,normalize,wikilink}.rs`. SPEC §1.1–1.3. Testler: her blok türü, frontmatter, iç içe liste, 4 wikilink biçimi, NFC, kod bloğu boşluk korunumu, boş paragraf.
**Kabul:** Aynı metin → aynı `version_id`. `mini_vault` tamamı ayrıştırılır, blok sayıları `docs/progress/T01.md`'de.

## T2 — Nesne deposu ve olay günlüğü (Rust)
**Yapar:** `core/src/{store,events}.rs`, `Vault::open`, `replay`, `events_hash`. Testler: aynı nesne iki kez → bayt değişmez; aynı kimlik farklı içerik → `Err`; replay `snapshot_id` eşit; günlük yalnızca-ekleme (önek testi).
**Kabul:** Hepsi geçer; `events.jsonl` satırlarında zaman damgası veya rastgele alan yok (regex testi).

## T3 — Soy eşleyici v2 (Rust)
**Yapar:** `core/src/lineage.rs` SPEC §5 adım adım, `Vault::ingest`, `resolve_truth`. Her adım için elle vakalar: kesin+taşıma (LIS), tekrar blok ctx çapası, tekrar blok ordinal, split 2 ve 3 parça, merge, benzerlik güncelleme, **artık kontrolü** (büyük parça + küçük artık → split), create/delete. Aynı metinle ikinci ingest sıfır olay.
**Kabul:** Her SPEC §5 adımına en az 2 test. `resolve_truth` yanlış referansta `Err`. `lineage_id` §5.1 formülüne uyuyor (test formülü yeniden hesaplar).

## T4 — Chunker ve CLI (Rust)
**Yapar:** `core/src/chunk.rs` (fixed, structural, tiktoken-rs), `core/src/bin/mdk.rs` tüm komutlar.
**Kabul:** structural başlık sınırını geçmiyor; 512+ tek blok bölünmüyor; `chunk_id` version_id'lerden; `mdk ingest-dir` + `mdk events-hash` mini_vault'ta iki koşturmada aynı hash.

## T5 — Python bağlaması ve sentetik üreteç
**Yapar:** `bindings/src/lib.rs` SPEC §7 fonksiyonları; `tests/test_bindings.py` (Rust testlerinin Python'dan tekrarı: 5 vaka). `eval/datasets/synthetic_edits.py` (EVAL §D1, `dup_delete` dahil), `eval/truth.py`.
**Kabul:** Her op için tutarlılık testi; oranlar 1000 adımda ±%3; `truth.json` yalnız `(version_id, k)` referansı içeriyor (regex: 26 karakterli ULID benzeri hiçbir alan yok). Tokenizer eşitliği testi: 20 rastgele chunk'ta `tiktoken` ve `tiktoken-rs` aynı sayı.

## T6 — D3, D1 ve H1-S
**Yapar:** `eval/datasets/vault.py` (D3 klon, kabul, ingest, `MANIFEST.d3.json`), D1 üretimi + manifest. `eval/metrics.py` H1 kısmı. `eval/run_h1s.py`: `--phase tune` (THETA×THETA_SPLIT ızgarası, ayar seti) → `tuning.json`, seçimi SPEC §5.3 kuralıyla yap, `core/src/lineage.rs` sabitlerini güncelle, yeniden derle; `--phase test` sabitlerin `tuning.json` ile eşit olduğunu doğrular, test setinde ölçer.
**Kabul:** JSON'da S1–S4; ızgara tam; ayar/test kesişimi boş (kod içi kontrol); ızgara boyunca sonuçlar **değişiyor** (v1'de değişmiyordu; değişmiyorsa eşleyicide THETA kullanılmıyor demektir → görev kalmış sayılır).

## T7 — D2 ve H1-R
**Yapar:** `eval/datasets/git_history.py` + manifest, `eval/run_h1r.py`, kayıp kategorizasyonu.
**Kabul:** R1, R2, `touched_retention`, kategori dağılımı JSON'da; çift ≥ 300; "bitişik satır" tanımı kodda görünür.

## T8 — İndeks, taban çizgileri, sıcak katman
**Yapar:** `mdkernel/{embed,index,retrieval,hot,watcher}.py`: SPEC §8, §9 `flat`/`hybrid`, §10. `tests/test_index.py` (mini_vault üzerinde). `eval/metrics.py` H2/H3/H4 fonksiyonları birim testli (elle hesaplı örnekler, EVAL'deki tanımlar; nDCG >1 çıkarsa hata testi).
**Kabul:** `hybrid("x", 10)` 10 geçerli anahtar; metrik testleri geçer; sıcak gölgeleme testi.

## T9 — DONDURMA
**Yapar:** `tests/fixtures/freeze_fixture/`: mini_vault + 20 elle yazılmış soru (gerçekler içerik-referanslı) + 3 adımlık sentetik düzenleme; `make freeze-check` bu fixture üzerinde tüm metrikleri koşturup `tests/fixtures/freeze_fixture/expected.json` ile karşılaştırır. `expected.json` bu görevde yazılır.
**Kabul:** `make freeze-check` geçer. Commit mesajı tam olarak `T9: freeze eval`. Bu commit'ten sonra `eval/metrics.py`, `eval/datasets/*.py`, `eval/truth.py` dondurulmuş.

## T10 — Graf
**Yapar:** `mdkernel/graph.py` SPEC §8 graf; `(doc_path, chunk_id)` anahtar; `[[D#h]]` testi; ağırlık tavanı.
**Kabul:** Kenar dağılımı `T10.md`'de; testler.

## T11 — Graf arama ve hier
**Yapar:** `graph_flat`, `graph_hybrid`, `hier` SPEC §9; ego alt graf + sparse PPR.
**Kabul:** Deterministik; ego alt graf düğüm sınırı testi; PPR yalnız tohumlara kütle; `hier` 20 doküman.

## T12 — Soru seti (API gerekir; yoksa PARK)
**Yapar:** `eval/datasets/questions.py` EVAL §D4; ayrı süreç; `data/d4/`, `data/splits/d4.json` **git'e eklenir**.
**Kabul:** single ≥ 180, multi ≥ 130; sızıntı + dosya adı filtreleri uygulanmış, `generation_log.jsonl`; gerçekler içerik-referanslı; `eval/truth.py` ile **tamamı çözülüyor** (koşturma testi).

## T13 — H3
**Yapar:** `eval/run_h3.py`.
**Kabul:** H3-1 p50/p95, H3-2 kayıp/kazanç 0, olay hash eşit.

## T14 — H4-1
**Yapar:** `eval/run_h4.py`: D3'ü sil, yeniden kur, `events_hash` vs manifest.
**Kabul:** Eşit; JSON'da iki hash.

## T15 — Rapor üreteci
**Yapar:** `eval/report.py` (yalnız `RUN`), `eval/compare_runs.py` (H4-2 için alan listesi sabit), şablon doldurma, PNG'ler (THETA ızgarası ısı haritası, ALPHA eğrisi, nDCG çubuk).
**Kabul:** `RUN` eksik dosya → hata (test). Elle sayı yok. Karar kuralı uygulanıyor.

## T16 — `make verify`
**Yapar:** `scripts/verify.sh`: temiz dizine klon → kurulum → `make d2 d3 d1` (manifest doğrulama) → `make freeze-check` → tüm `eval-*` (T12 park ise H2 atlanır ve bu açıkça yazılır) → `make report` → `reports/runs/<RUN>/` çıktısı. Aynı betik iki ayrı dizinde koşturulup `compare_runs.py` ile H4-2 üretilir.
**Kabul:** Betik, ana çalışma dizininden bağımsız; hiçbir mutlak yol yok; `set -euo pipefail`.

## T17 — H2 (T12 bitmiş olmalı)
**Yapar:** `eval/run_h2.py --phase tune RUN=` (ALPHA ızgarası, yalnız `graph_hybrid/structural`, ayar seti; `tuning.json` + duyarlılık eğrisi), `ALPHA_DEFAULT` sabitle; `--phase test RUN=` (sabit = tuning doğrulaması; 20 hücre; bootstrap; full_cover).
**Kabul:** H2-1..H2-5 JSON'da; `--phase test` sabit uyuşmazlığında hata (test); hücre başına sorgu sayısı beklenen (test).

## T18 — Nihai rapor ve devir
**Yapar:** `make report RUN=<son>` → `reports/FINAL.md`; `docs/progress/T18.md` sonunda "T19 (bağımsız doğrulama) için hazır" + kullanılan `RUN` kimliği. Ana ajan durur.

## T19 — Bağımsız doğrulama
Ayrı oturum, temiz klon, `docs/04_VERIFY.md`.
