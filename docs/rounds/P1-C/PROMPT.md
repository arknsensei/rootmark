# P1-C — `/goal` İstemi

Aktif tur P1-C. Yönergeler: docs/rounds/P1-C/GOAL.md, docs/rounds/P1-C/ADDENDUM.md (GOAL'in parçasıdır; C0 görevi ADDENDUM'daki işleri içerir), docs/rounds/P1-C/TASKS.md, docs/rounds/P1-C/VERIFY.md (yalnız doğrulayıcı içindir, sen uygulamazsın).

Başlangıç koşulu: `p1b1-final` etiketi var ve reports/rounds/P1-B.1/VERIFICATION.md hükmü "Güvenilir". Değilse dur ve docs/rounds/P1-C/progress/STOP.md yaz.

Önce CLAUDE.md, NORTH_STAR.md, spec/SPEC.md (§6, §8, §9, §10, §12), spec/CONFORMANCE.md (Level 2–3), docs/ARCHITECTURE.md §4.2, docs/PACKAGES.md (index), docs/DEV_PROCESS.md, docs/SPEC_EDITS_CRITERION.md, core/CLAUDE.md oku; sonra GOAL.md, ADDENDUM.md, TASKS.md.

Görevleri C0'dan C9'a sırayla yürüt; park kuralı dışında atlama yok. Her görevde DEV_PROCESS §2 disiplini: progress kaydı (docs/rounds/P1-C/progress/), kabul ölçütleri tek tek, gözden geçirici alt-ajan, görev başına commit (P1-C/C<n>: ...). C0'da ROUND_START dosyasına başlangıç commit sha'sını ve DECISIONS.md'ye K-C0.1..K-C0.3'ü ekle.

Kırmızı çizgiler: index yalnız core'a bağımlı; ANN yok, kesin arama; tek embedder, tek iş parçacığı, model hash manifest'te sabit; proposal varsayılanda dönmez, derived asla dönmez; her search tüketim izi yazar; events.jsonl ve spec §1–8 değişmez; spec §9 yalnız docs/SPEC_EDITS_CRITERION.md ölçütüyle ve SPEC_EDITS.md kaydıyla; GOAL ve ADDENDUM eşikleri değişmez; graf stratejileri koşturulmaz; çalışma zamanı yalnız Rust; sessiz başarısızlık yok.

Kriter tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md yaz, görevi PARK et, bağımsız görevlere devam et; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.

C9 bitince docs/rounds/P1-C/progress/READY.md'ye "doğrulama için hazır", commit sha ve RUN kimliğini yaz; dur. Doğrulamayı sen yapma.
