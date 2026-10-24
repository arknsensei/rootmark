# P2-L — Bağımsız Doğrulama (Blok 1 sonunda, P2-P ile birlikte; K-P2.4)

Bu turun bağımsız doğrulaması tek başına yapılmaz; Blok 1 (P2-L + P2-P) bitince tek bir dar doğrulama koşulur ve `reports/rounds/P2-P/VERIFICATION.md`'ye yazılır (P2-L bölümü ayrı başlıkla). P2-L için doğrulayıcının bakacağı maddeler:

- **VL1** Tam çift (`make verify`, bir kez): iki tokenizer ve iki embedder ile indeks baytları ve 12 hücre fark 0.
- **VL2** Türkçe ölçüm: D-TR'nin kaynağı, revizyonu ve sha256'sı; sorgu üretecinin tohumu ölçümden önce commit'li; 12 hücreyi kendin yeniden üret; L-2, L-5, L-6 eşik dalları.
- **VL3** İngilizce referans Δ 0 (`mdk_lower_alnum`) ve L-4 kararının doğru uygulanmış olması (varsayılan tokenizer).
- **VL4** Tokenizer birim vakaları kendi örneklerinle (Türkçe büyük İ/ı, Almanca ß, Rusça, emoji, rakam-harf karışık).
- **VL5** Yol öneki filtresi: vault dışına çıkma denemeleri (`..`, mutlak yol, NFD varyantı) reddediliyor; filtresiz `query_hash` değişmemiş; `trace` filtreyi taşıyor.
- **VL6** Eski indeksli vault → `migration_required`, göç, `events_hash` eşit.
- **VL7** 0.2.0 yayımı: `v0.2.0` etiketi `main` commit'inde; release varlıkları `SHA256SUMS` ve provenance 5/5; Linux arşivi Docker'da `rootmark --version` `0.2.0`, `conformance` 39/39.
- **VL8** Süreç: `check_frozen`, `check_deps`, `check_dep_age`; FAST kipinin tam zinciri değiştirmediği (dondurma istisnası kaydı); karar kuralı GOAL'den; NORTH_STAR notu K-P2-03.1 ile uyumlu.
