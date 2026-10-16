# P1-E — `/goal` İstemi

Aktif tur P1-E. Yönergeler: docs/rounds/P1-E/GOAL.md, TASKS.md; VERIFY.md yalnız doğrulayıcı içindir.

Başlangıç koşulu: reports/rounds/P1-D.1/VERIFICATION.md 'Şu koşullarla güvenilir'; koşullar docs/rounds/P1-E/ADDENDUM.md ile E0'da kapatılır. E0 tamamlanıp E-12..E-16 geçmeden E1'e geçme.

docs/rounds/P1-E/ADDENDUM.md'yi de oku; E0 görevi ADDENDUM'daki işleri içerir.

Önce CLAUDE.md, NORTH_STAR.md (ürün sahnesi), docs/CLIENT_CONTRACT.md (istemcinin tek bağımlılığı), docs/SECURITY.md, docs/ARCHITECTURE.md §2–§3 (istemci bağımsızlığı) ve §4.4, docs/PACKAGES.md (clients), docs/DEV_PROCESS.md, docs/DECISIONS.md (K-D1.1, K-D1.3, K-D1.5), server/CLAUDE.md oku; sonra GOAL.md, TASKS.md.

E0'dan E8'e sırayla; DEV_PROCESS §2 disiplini; commit öneki P1-E/E<n>; E0'da ROUND_START.

Kırmızı çizgiler: istemci çekirdeğe yalnız CLIENT_CONTRACT'taki HTTP uçlarıyla bağlanır, `core/index/server` kaynağına dokunmaz; istemci md dosyasına asla yazmaz; terfi yalnız gerekçeli ve insan kanalından; `mdk serve` istemciden başlatılmaz; çalışma zamanı bağımlılığı Obsidian API dışında sıfır; spec değişmez; çekirdek/indeks/sunucu kodu bu turda yalnız `mdk conformance` alt komutu ve yayım betikleri için değişir, davranış değişmez (regresyon fark 0); GOAL eşikleri değişmez; **0.1.0 etiketini sen atmazsın**; sessiz başarısızlık yok.

Kriter tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz. Obsidian başsız koşturulamıyorsa harness sınırlamasını ASSUMPTIONS'a yaz ve komut katmanı harness'iyle devam et.

E8 bitince docs/rounds/P1-E/progress/READY.md, dur. Doğrulamayı sen yapma.
