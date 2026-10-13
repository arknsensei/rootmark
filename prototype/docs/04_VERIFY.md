# 04 — Bağımsız Doğrulama (T19)

İkinci, bağımsız ajan içindir. Ana oturumun bağlamı yok. Kod değiştirme. Çıktı: `reports/VERIFICATION.md`. Etiketler: `[OK]`, `[FARK]`, `[ŞÜPHE]`, `[İHLAL]`.

v1 doğrulamasından ders: sayıları **kendi** koşturmandan al; ana ajanın yerel verisine, önbelleğine, `data/` dizinine dokunma. Ana depoya erişimin olsa bile **okuma dahil** kullanma; doğrulama yalnız klonlanan git içeriği ve ağdan indirilenlerle yapılır. HuggingFace model önbelleği kullanılabilir (ağırlıklar sabit).

## V1 — Çift temiz koşturma
`scripts/verify.sh` betiğini **iki ayrı dizinde** koştur (A ve B). Her biri sıfırdan klonlar, kurar, veri üretir, ölçer, rapor üretir. Süreleri kaydet. Başarısızlık → `[İHLAL]` ve neden.

## V2 — H4 (determinizm)
- `data/MANIFEST.d3.json` `events_hash` ile A ve B'nin `make eval-h4` çıktısı: üçü eşit mi? Değilse `[İHLAL]` (H4-1).
- `eval/compare_runs.py A B` → kalite alanlarında maksimum göreli fark. 0 değilse `[İHLAL]` (H4-2), hangi alanlar.
- `git`'teki `FINAL.md` (ana ajan, `RUN` kimliği `docs/progress/T18.md`'de) ile A'nın `FINAL.md`'si: kalite sayıları birebir mi? Değilse `[İHLAL]`. Gecikmeler: yalnız eşik **durumu** karşılaştırılır.

## V3 — Dondurma
`git log --oneline -- eval/metrics.py eval/truth.py eval/datasets/` → `T9: freeze eval` sonrası değişiklik var mı? Varsa `FREEZE_EXCEPTIONS.md`'de girdi, önce/sonra `freeze-check` eşitliği ve gözden geçirici onayı üçü de var mı? Üçü yoksa `[İHLAL]`; varsa `[OK]` ve özet. `make freeze-check` A'da geçiyor mu?

## V4 — Eşikler ve karar
`git log -p -- docs/00_GOAL.md` ilk commit sonrası değişiklik → `[İHLAL]`. `FINAL.md` karar kuralı çıktısını kendin `00_GOAL.md` ile yeniden türet; farklıysa `[İHLAL]`.

## V5 — Sızıntı ve ayar/test
- D4'ten 20 soru (`random.Random(20261013)`), gerçek metinleri **A'nın** `data/d3/.mdkernel` nesnelerinden çöz (`mdk resolve` + `objects/`); 8+ kelime ortak dizi → `[İHLAL]`. Tüm sorularda programatik tarama.
- Dosya adı filtresi: soru metni hedef doküman adını içeriyor mu, 20 örnekte elle.
- `run_h2.py --phase test` çalışmadan önce test setini okuyan **herhangi** bir kod yolu var mı (`grep` + okuma) → varsa `[İHLAL]`.
- Bölünme kesişimleri programatik.

## V6 — Fail-fast kontrolleri (kasıtlı bozma)
A'nın kopyasında, kod değiştirmeden veri bozarak:
- `data/d4/single.jsonl` içinde bir `version_id`'yi değiştir → `make eval-h2 RUN=x` **hata vermeli**; sessizce çalışırsa `[İHLAL]`.
- `data/MANIFEST.d3.json` içinde bir hash'i değiştir → `make d3` hata vermeli.
- `reports/runs/<RUN>/h2.json` sil → `make report RUN=<RUN>` hata vermeli.
Sonra kopyayı at.

## V7 — İzlenebilirlik
`FINAL.md`'den 10 sayı → `reports/runs/<RUN>/*.json`'da bul. Kategori dağılımı (H1-R1 kayıpları) JSON ile eşleşiyor mu.

## V8 — Metrik uyumu
`eval/metrics.py` ↔ `03_EVAL.md` satır satır; elle örnekler: nDCG (min_cover=2, 3 chunk), full_cover (yalnız A kapsanmış → 0), recall multi (0.5), H1-S4 (dup_delete: 2 kalan kopya korunmuş → 1.0; biri kaymış → 0.5). nDCG>1 üreten bir giriş bul; fonksiyon hata veriyor mu.

## V9 — Rust/Python tutarlılığı
`mdk ingest-dir` ile `tests/fixtures/mini_vault` ingest et, `mdk events-hash`; aynı şeyi Python `mdkernel_core` ile yap; hash eşit mi. 5 blok için `mdk chunk` ve Python chunk `token_count` eşit mi.

## V10 — Varsayımlar ve park
`ASSUMPTIONS.md` (etiketli), `STOP.md`, park edilmiş görevler, `FREEZE_EXCEPTIONS.md`, `EVAL_FREEZE_ISSUES.md` özeti; sonucu etkileyebilecekler.

## Sonuç
İhlal/fark/şüphe sayısı; "güvenilir / şu koşullarla / değil"; karar kuralına katılıp katılmadığın; ana ajana önerilen en fazla 3 madde.
