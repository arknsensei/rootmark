# P1-A — Bağımsız Doğrulama

Ayrı oturum, temiz klon, ana çalışma dizinine erişim yok. Kod değiştirme. Çıktı: `reports/rounds/P1-A/VERIFICATION.md`. Etiketler `[OK] [FARK] [ŞÜPHE] [İHLAL]`.

- **V1 Çift koşturma:** `scripts/verify.sh pair <remote> A B` çıkış 0; A↔B kalite alanları fark 0; A↔ana RUN fark 0.
- **V2 Uyumluluk:** A'da `reports/conformance.json` 17/17; fixture beklenen değerleri `v3-final` çekirdeğiyle kendin yeniden üret (prototip sha `SOURCE.md`'de), eşit mi.
- **V3 Hash'ler:** mini_vault v2/v3 hash'leri GOAL A-5 ile eşit; `events.jsonl` v3-final fixture ile `cmp` eşit.
- **V4 Regresyon:** `make regress` çıktısı `v3-final` JSON'larıyla alan alan eşit.
- **V5 Holdout:** holdout ∩ easy ∩ hard = ∅; H1-H* A-2 eşikleri; holdout üreteci eşleyici kodundan sonra commit'lenmiş mi (git log sırası).
- **V6 Dondurma:** `research/FROZEN.md` listesindeki dosyalar tur içinde değişmemiş; denetleyiciler sabit-bağımsız (kasıtlı test).
- **V7 Bağımlılık yönü:** `scripts/check_deps.sh` A'da temiz; kasıtlı ters bağımlılık eklenince hata.
- **V8 Kasıtlı bozma:** fixture beklenen değeri, manifest hash'i, holdout truth, eksik rapor dosyası → her biri hata.
- **V9 Karar kuralı:** GOAL'den kendin türet; FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe sayısı; hüküm; en fazla 3 öneri.
