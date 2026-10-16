# P1-D.1 — `/goal` İstemi

Aktif tur P1-D.1. Yönergeler: docs/rounds/P1-D.1/GOAL.md (kararlar, kriterler, görevler aynı dosyada); VERIFY.md yalnız doğrulayıcı içindir.

Başlangıç koşulu: reports/rounds/P1-D/VERIFICATION.md hükmü "Güvenilir" ya da "Şu koşullarla güvenilir" ve `p1d-final` etiketi (D1.0 atar).

Önce CLAUDE.md, NORTH_STAR.md (vaat ve ilkeler), spec/SPEC.md §4, §7–§9, docs/ARCHITECTURE.md §5, docs/DEV_PROCESS.md, docs/SPEC_EDITS_CRITERION.md, docs/DECISIONS.md (K12, K16, K-D4-T), reports/rounds/P1-D/VERIFICATION.md (Ş1, Ş2, Ö1–Ö3), reports/rounds/P1-C.1/VERIFICATION.md (Ö1–Ö3), core/CLAUDE.md, index/CLAUDE.md, server/CLAUDE.md oku; sonra GOAL.md.

D1.0'dan D1.7'ye sırayla; DEV_PROCESS §2 disiplini; commit öneki P1-D.1/D1.<n>; D1.0'da ROUND_START ve DECISIONS ekleri.

Kırmızı çizgiler: `AppendOnly` varsayılan kalır ve canonical bloğa dokunan hiçbir ajan yazımı kabul edilmez; her silme/değiştirme olayı aktör taşır; MCP'den terfi daima `denied`; yazan çağrılar vault başına tek FIFO kuyruğundan geçer; spec §1–3, §5–6 değişmez, diğer bölümler yalnız CRITERION ölçütü ve `SPEC_EDITS.md` kaydıyla, sürüm numarasını CRITERION belirler; mevcut vault'lar açılmaya devam eder ve `events_hash`'leri değişmez; GOAL eşikleri değişmez; çalışma zamanı yalnız Rust; sessiz başarısızlık yok; istemci kodu yazılmaz.

Kriter tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.

D1.7 bitince docs/rounds/P1-D.1/progress/READY.md, dur. Doğrulamayı sen yapma.
