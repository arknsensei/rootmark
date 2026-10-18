# P2-0 — `/goal` İstemi

Aktif tur P2-0 (hipotez turu). Yönergeler: docs/rounds/P2-0/GOAL.md, TASKS.md; VERIFY.md yalnız doğrulayıcı içindir.

Başlangıç koşulu: `v0.1.0` etiketi ve reports/rounds/P1-E.1/VERIFICATION.md "Güvenilir". Değilse dur, STOP.md yaz.

Önce CLAUDE.md, NORTH_STAR.md, docs/ROADMAP.md §1–§2 (P2-0 gerekçesi), spec/SPEC.md §7–§8, docs/CLIENT_CONTRACT.md, docs/SECURITY.md ve docs/THREAT_MODEL.md, docs/DEV_PROCESS.md, docs/SPEC_EDITS_CRITERION.md, reports/rounds/P1-E/VERIFICATION.md (faz 2 risk 1), reports/rounds/P1-E.1/VERIFICATION.md (öneriler), core/ index/ server/ clients/obsidian CLAUDE.md'leri oku; sonra GOAL.md, TASKS.md.

Z0'dan Z8'e sırayla; DEV_PROCESS §2 disiplini; commit öneki P2-0/Z<n>; Z0'da ROUND_START ve DECISIONS ekleri.

Dal ve PR (K-P2.1): `main` korumalıdır, doğrudan push edilemez. Çalışma dalı `p2-0/round` (açılış commit'iyle hazır, `main` HEAD'den); görev başına commit (`P2-0/Z<n>: ...`) bu dalda; her görev sonunda `git push origin p2-0/round`. Görev başına PR açma. Z8 bitince READY.md'yi commit'le, dalı it ve tek bir PR aç (`gh pr create --base main --head p2-0/round --title "P2-0: tasdikli yazma" --body-file reports/rounds/P2-0/FINAL.md`); PR'ı birleştirme, dur. ROUND_START: GOAL gereği `v0.1.0` etiketi (regresyon referansı); çalışma tabanı `main` HEAD. GH-0'dan devralınan not: Linux CI işi bilgi amaçlıdır (K-GH.6), zorunlu iş macOS.

Kırmızı çizgiler: GOAL eşikleri ve karar kuralı değişmez; W yalnız D-B ayar setinde seçilir, test seti sabitlenmeden okunmaz; MCP'de attest aracı yoktur; eski vault'larda tüm bloklar canonical kalır ve events_hash değişmez; istemci md yazmaz; spec §1–6 değişmez, §7–§8 yalnız GOAL'deki genişleme ve CRITERION ile; çalışma zamanı yalnız Rust (plugin TS); sessiz başarısızlık yok; NORTH_STAR'ı değiştirme, öneri yaz.

Hipotez düşerse optimize etme, raporla, karar kuralını uygula. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz. Gerçek Obsidian yoksa D-B'yi PARK et ve 0b/0c "ölçülmedi" raporla.

Z8 bitince docs/rounds/P2-0/progress/READY.md, dur. Doğrulamayı sen yapma.
