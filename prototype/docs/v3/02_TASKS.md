# v3 — Görev Grafı

Sıra: T20 → T21 → T22 → T23 → T24 → T25 → T26 → T27 → T28. Park kuralı `CLAUDE.md`. Her görev kabul ölçütleri + gözden geçirici alt-ajan + commit ile biter.

## T20 — Tur açılışı
**Yapar:** `git tag v2-final` (HEAD'de v2 raporu commit'li olmalı). `CLAUDE.md` sonuna `docs/v3/CLAUDE_APPEND.md` içeriği eklenir. `docs/progress/v3/` dizini. `docs/ARCHITECTURE.md` deponun `docs/` kökünde (yoksa paketten kopyalanır). `docs/v3/05_REPORT_TEMPLATE.md` v2 şablonundan türetilir (H2 bölümü çıkar; H1-easy, H1-hard, H1-real, H4-V2 bölümleri girer).
**Kabul:** etiket var; `CLAUDE.md` "aktif tur: docs/v3" satırını içeriyor; `make test` ve `make lint` temiz.

## T21 — Eşleyici v3 (Rust)
**Yapar:** `MatcherVersion::{V2,V3}`; `01_SPEC_DELTA.md` §5.3 adımları; vault başına sürüm kilidi; CLI/bağlama parametresi. Rust testleri: K4 baskın devralma (split 2/3 parça, merge 2/3 blok), asgari parça oranının kaldırılması, birleştirme yönü artık kontrolü (en az 3 vaka), karışık sürüm `Err`, **V2 bayt-eşitliği** (mini_vault ile v2-final etiketindeki `events_hash` `8914f9b418ae360d49c85c951050f432` birebir).
**Kabul:** tüm testler; `mdk ingest-dir --matcher v2 tests/fixtures/mini_vault` hash'i v2-final ile eşit; `--matcher v3` farklı ve deterministik (iki koşturma aynı).

## T22 — D1-easy(K4) ve D1-hard üreteçleri
**Yapar:** `eval/datasets/synthetic_edits_v2.py` (yeni dosya; v2 üretecini içe aktarır, kopyalamaz). `data/d1k4` metinleri `data/d1` ile bayt bayt eşit (test); D1-hard op'ları `03_EVAL_DELTA.md` tablosuna göre; manifest'ler.
**Kabul:** oranlar 1000 adımda ±%3; `split_edit`/`merge_edit` parçalarının hiçbiri eski metnin tam alt dizisi değil (test: `sim < 1.0`); easy/hard doküman kesişimi boş; truth yalnız `(version_id, k)`.

## T23 — Metrikler v3 ve dondurma
**Yapar:** `eval/metrics_v3.py` (v2 fonksiyonlarını içe aktarır), birim testler (K4 S2 tanımı: merge sonucu paydada değil; dominant eşitliği olay F1'de; H1-G sayımı), `tests/fixtures/freeze_fixture_v3/` (mini_vault + 3 adım K4 gerçekli düzenleme + `expected.json`), `make freeze-check-v3`.
**Kabul:** `freeze-check` (v2) **hâlâ geçiyor**; `freeze-check-v3` geçiyor; commit mesajı tam olarak `T23: freeze eval v3`.

## T24 — H1-S v3
**Yapar:** `eval/run_h1s_v3.py --phase tune` (D1-hard ayar; ızgara; H1-G; seçim; V3 sabitlerini yaz; yeniden derle) → `--phase test` (sabit doğrulama; D1-easy test + D1-hard test; S1–S4, H1–H4; bilgi amaçlı v2-tanımlı S2).
**Kabul:** JSON'da H1-S1..S4, H1-H1..H4, H1-G; ızgara tam; ayar/test kesişimi boş; `--phase test` sabit uyuşmazlığında hata (test).

## T25 — H1-R v3
**Yapar:** `eval/run_h1r_v3.py`: D2 aynı çiftler, eşleyici v3; R1, R2, kayıp kategorileri.
**Kabul:** çift sayısı v2 ile aynı (836); JSON'da R1, R2, kategoriler.

## T26 — H3/H4 regresyon ve v2 yeniden üretimi
**Yapar:** `make eval-h3 RUN=` (eşleyici v3); `make eval-h4 RUN=` (D3 yeniden kurulum, `MANIFEST.d3.json`'a `events_hash_v3` alanı eklenir — v2 alanı korunur); `eval/run_h4_v2repro.py` (EVAL_DELTA §H4-V2).
**Kabul:** H3 JSON; H4-1 (v3) eşit; H4-V2 iki koşulu da JSON'da ve eşit; `MANIFEST.d3.json` v2 `events_hash` değişmemiş (git diff ile test).

## T27 — Rapor ve verify genişletme
**Yapar:** `eval/report_v3.py`, `scripts/verify.sh` v3 fazlarını ekler ve H4-2 ≠ 0 ise sıfır olmayan kodla çıkar (v2 doğrulayıcının 1. önerisi). `make report-v3 RUN=` → `reports/v3/FINAL.md`; ham JSON'lar `git add -f`. `docs/progress/v3/T27.md` sonunda "T28 için hazır" + RUN.
**Kabul:** FINAL.md'deki her sayı JSON'a izlenebilir; karar kuralı çıktısı; `verify.sh` çıkış kodu testi.

## T28 — Bağımsız doğrulama
Ayrı oturum, temiz klon, `docs/v3/04_VERIFY_DELTA.md` + v2 `docs/04_VERIFY.md`.
