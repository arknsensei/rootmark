# P1-C — Bağımsız Doğrulama

Ayrı oturum, temiz klon, ana dizine erişim yok. Kod değiştirme. Çıktı `reports/rounds/P1-C/VERIFICATION.md`.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; A↔B kalite alanları fark 0; `A/.../.mdkernel/index/` ↔ B `diff -r` eşit (lexical, vectors, hot); 200 sorgu sıralaması eşit; A↔resmi RUN fark 0.
- **V2 Model:** `models/MANIFEST.json` hash'lerini HuggingFace'ten **kendin indirip** doğrula; manifest bozma → `make models` hata; `embedder_id` manifest'ten türetilmiş.
- **V3 Uyumluluk:** `conformance.json` `level_claimed 3`, 29/29, `spec_version 1.0.1`; C3 beklenen değerlerini yeniden üret, `git status` temiz.
- **V4 Determinizm (kendi testlerin):** aynı sorgu 100 tekrar; indeks sil → `index build` → aynı sıralama; `compact` öncesi/sonrası; farklı dizinde kurulum → `index/` bayt eşit.
- **V5 Katman:** `proposal` bloklu vault'ta 100 rastgele sorgu: varsayılanda `proposal` hit 0; `--include-proposal` ile etiketli ve skor ×0.5; `derived` hiç; `--at` eski checkpoint'te sonraki ingest görünmüyor.
- **V6 İz:** her `search` bir satır; 5 izin `trace_id`'sini formülle bağımsız hesapla; `query_hash = H(query)` 32 hex.
- **V7 Kalite (C-3, C-10):** `run_c3_quality.py` A çıktısı; v2-final `hybrid/structural` hücreleriyle Δ'yı kendin hesapla; `embed_ref.json` ile 50 kosinüs kendin hesapla.
- **V8 Sıcak katman:** C-8 p95 **durumu** (sayı makineye bağlı); yalnız etkilenen chunk'lar embed (sayaç).
- **V9 Dondurma, bağımlılık yönü, F-istisnaları:** `check_frozen.sh`; `check_deps.sh`; `index` → `server`/`clients` bağımlılığı yok; `SPEC_EDITS.md` K-B1.3 ölçütüyle.
- **V10 Karar kuralı:** GOAL'den türet; FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri.

## İstemler
**Ana (`/goal`):**
```
Aktif tur P1-C (docs/rounds/P1-C/). Başlangıç koşulu: p1b1-final etiketi var ve reports/rounds/P1-B.1/VERIFICATION.md "güvenilir". Yoksa dur.
CLAUDE.md, NORTH_STAR.md, spec/SPEC.md (§6, §8, §9, §10, §12), spec/CONFORMANCE.md (Level 3), docs/ARCHITECTURE.md §4.2, docs/PACKAGES.md (index), docs/DEV_PROCESS.md, docs/SPEC_EDITS_CRITERION.md, core/CLAUDE.md oku; sonra GOAL.md ve TASKS.md.
C0'dan C9'a sırayla; DEV_PROCESS §2 disiplini; commit öneki P1-C/C<n>.
Kırmızı çizgiler: index yalnız core'a bağımlı; ANN yok, kesin arama; tek embedder, tek iş parçacığı, model hash sabit; proposal varsayılanda dönmez, derived asla dönmez; her search iz yazar; events.jsonl ve spec §1–8 değişmez; spec §9 yalnız K-B1.3 ölçütüyle; GOAL eşikleri değişmez; graf stratejileri koşturulmaz; sessiz başarısızlık yok.
Kriter tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.
C9 bitince READY.md, dur. Doğrulamayı sen yapma.
```
**Doğrulama:**
```
Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 ve docs/rounds/P1-C/VERIFY.md V1–V10; model dosyalarını kendin indir; verify.sh'i iki dizinde koştur. Kod değiştirme. Tek çıktı reports/rounds/P1-C/VERIFICATION.md.
```
