# P1-D — `/goal` İstemi

Aktif tur P1-D. Yönergeler: docs/rounds/P1-D/GOAL.md, TASKS.md; VERIFY.md yalnız doğrulayıcı içindir.

Başlangıç koşulu: reports/rounds/P1-C/VERIFICATION.md hükmü "Güvenilir" ve `p1c-final` etiketi (D0 atar). D3 ve sonrası için ek koşul: `p1c1-final` etiketi ve reports/rounds/P1-C.1/VERIFICATION.md "Güvenilir". D2 bitince bu yoksa D3'ü PARK et, progress/WAITING.md yaz ve dur; etiket gelince aynı istemle devam et ve WAITING.md'yi kapat.

Önce CLAUDE.md, NORTH_STAR.md (ürün sahnesi), spec/SPEC.md §7–§10, docs/ARCHITECTURE.md §4.3 ve §5, docs/PACKAGES.md (server), docs/DEV_PROCESS.md, docs/DECISIONS.md (K12, K16, K-C1.3), core/CLAUDE.md, index/CLAUDE.md oku; sonra GOAL.md, TASKS.md.

D0'dan D8'e sırayla; DEV_PROCESS §2 disiplini; commit öneki P1-D/D<n>; D0'da ROUND_START.

Kırmızı çizgiler: çalışma zamanı yalnız Rust; `server` yalnız `core` ve `index`'e bağımlı; MCP için resmi Rust SDK; `propose` dışında markdown yazan hiçbir yol yok ve `propose` dosyaya kimlik/meta veri yazmaz; `promote` MCP'den varsayılan politikayla Deny döner ve bu değiştirilmez; her `search`/`read_block` iz yazar; HTTP yalnız 127.0.0.1 ve token'lı; spec değişmez (yeni bölüm yok); GOAL eşikleri değişmez; sessiz başarısızlık yok; istemci kodu yazılmaz.

Kriter tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.

D8 bitince docs/rounds/P1-D/progress/READY.md, dur. Doğrulamayı sen yapma.
