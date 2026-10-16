# P1-D.1 — READY

doğrulama için hazır

- **commit:** `f75d55759829387619c5ec6aa20920ec96517277`. D1.0–D1.7 TAMAMLANDI; her görevin `<görev>.md` ve `<görev>.review.md` kaydı TAMAMLANDI.
- **ölçülen commit:** `15273654438c3eab7490e1e79078cedde151bdb2` (`P1-D.1/D1.7-fix:`). Resmi RUN'ın bütün aşamaları bu commit'te koştu, `worktree_clean true`. Sonraki commit'ler yalnız şunlara dokunur: `reports/rounds/P1-D.1/`, `docs/rounds/P1-D.1/progress/` ve `docs/DECISIONS.md` (K-D4-T kapanış notu).
- **RUN:** `20261016T124520Z-1527365` (`reports/rounds/P1-D.1/runs/20261016T124520Z-1527365/`).
  - `scripts/verify.sh pair` çıkış 0. A bu RUN, B `20261016T131108Z-1527365`.
  - `pair_compare.json` (`eval.compare_p1d1 pair`): 100 875 alan, fark 0, P1-D ve P1-D.1 yapıtları bayt eşit.
  - Kopyaya e2e çalışma vault'ları alınmadı (P1-D emsali).
- **rapor:** `reports/rounds/P1-D.1/FINAL.md`. **D1-1..D1-13 geçti** → karar kuralına göre **K-D4-T kapanır, P1-E başlar**.
- **İlk resmi çift** (`97bb027`) çıkış 3 verdi. Tek fark `d1_5.json`'un iki duvar saati alanıydı: karşılaştırıcı hatası, F16 ile düzeltildi. Temiz çift yukarıdadır (D1.7.md).
- **Önceki turların dondurulmuş kaydedicileri** resmi RUN'da kalan kriterleri karar gereği kaydeder: P1-D D-2/D-7/D-9, P1-C.1 C1-7. Hepsi D1-12'nin ölçümden önce yazılmış hariçlerinde (`compare_p1d1.EXCLUDED`, 29 önek). Ayrıntı D1.7.md'de.
- **Açık karar talebi:** K-D1.4-T. Spec 1.0.3'ün sürüm düzeyi yama mı, majör mü? Tur yamayı seçti (ASSUMPTIONS D1.4-1, SPEC_EDITS "Açık nokta").
- **Dondurma istisnaları:** P1-D.1/F1–F16 (`progress/FREEZE_EXCEPTIONS.md`), gözden geçirici onaylı.
- **Kalite kapıları:**
  - `make test`: cargo hepsi `ok`, pytest 198, vitest istemci paketi yok.
  - `make lint`: çıkış 0 (`97bb027`; F16 yalnız karşılaştırıcıda tek satır, sonrasında pytest/ruff/`check_frozen` temiz).
  - `make conformance`: level 3, 34/34.
  - `make verify` (pair): çıkış 0.
- **Doğrulama ortamı notu:**
  - P1-D'nin D-1'i resmi MCP Inspector'ı `npx` ile çeker. D1-11 `p1d-final` ağacını derler ve ONNX Runtime ikilisini indirebilir. Node.js ve ağ erişimi gerekir.
  - `scripts/test_p1d1.sh` ve `test_tamper_p1d1.sh` `MDK_MODELS`'i (varsayılan `<repo>/models`) kullanır.
