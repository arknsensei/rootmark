# v3 — Doğrulama Farkı (`docs/04_VERIFY.md` üzerine)

v2 `04_VERIFY.md` V1–V10 aynen uygulanır; şu eklerle. Ana depoya erişim yasağı aynen geçerli.

- **V1:** `scripts/verify.sh pair` v2 + v3 fazlarını birlikte koşturur. Çıkış kodu 0 olmalı; H4-2 ≠ 0 ise betik sıfır olmayan kodla çıkmalı (kasıtlı test: A'nın bir kalite JSON'unda bir sayıyı değiştirip `compare_runs` ile karşılaştır → sıfır olmayan kod bekle).
- **V2 ek — H4-V2:** `--matcher v2` ile D3 `events_hash` `e6945bbc2da9c47376e017e815d7e51b` mi; v2 D1 üzerinde v2 metrikleriyle H1-S1..S4 git'teki `20261013T154851Z-f133d0b/h1s.json` ile eşit mi. Değilse `[İHLAL]`.
- **V3 ek:** `T23: freeze eval v3` sonrası `eval/metrics_v3.py`, `eval/datasets/synthetic_edits_v2.py`, `eval/run_h1s_v3.py`, `eval/run_h1r_v3.py`, `eval/run_h4_v2repro.py`, `freeze_fixture_v3/**` değişmemiş; v2 dondurması da bozulmamış (`git diff a480432 HEAD -- <v2 donmuş dosyalar>` boş).
- **V4 ek:** `docs/v3/00_GOAL.md` ilk commit sonrası değişmemiş; karar kuralını kendin türet; H1-G geçerlilik şartının doğru uygulandığını kontrol et.
- **V6 ek:** `data/d1k4` metinleri `data/d1` ile bayt bayt eşit mi (`cmp -r`). Easy/hard doküman kesişimi boş mu.
- **V8 ek:** `metrics_v3.py` K4 S2 tanımı: elle örnek — merge sonucu paydada değil (S2=0 olmalı), split baskın olmayan parça eski soya bağlanmış (S2>0 olmalı). Olay F1'de `dominant` eşitsizliği doğru sayılmamalı.
- **V9 ek:** `mdk --matcher v2` ve `--matcher v3` mini_vault hash'leri: v2 = `8914f9b418ae360d49c85c951050f432`, v3 farklı; Python bağlaması aynı değerleri veriyor mu.
- **Sonuç:** ek olarak "K4 tanım değişikliği raporda açıkça işaretlenmiş mi" sorusuna evet/hayır.
