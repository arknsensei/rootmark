# P2-Q.1 — `/goal` İstemleri

Sıra: bu turu açan PR (P2-Q kapanışı K-P2Q.5, isim kararı K-D0.1) birleşir; sonra bu tur başlar.

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-Q.1/progress/READY.md "doğrulama için hazır", commit sha ve turun PR'ındaki CI koşu kimliğiyle yazıldı; reports/rounds/P2-Q.1/FINAL.md ve docs/rounds/P2-Q.1/progress/DESIGN.md var; dal p2-q.1/round uzağa itildi ve main'e tek bir PR açık — ya da tur bir insan adımında bekliyor ve docs/rounds/P2-Q.1/progress/HUMAN_STEP.md bekleneni yazıyor. Aktif tur P2-Q.1 (yama turu, K-P2Q.4: P2-Q'nün iki ürün hatası — PQ-22 Linux'ta serve --watch yeni dizinin ilk dosyasını kaçırıyor, PQ-18 izleyici silme ve taşımayı almıyor — yalnız izleyicide, spec'in mevcut olay biçimiyle düzeltilir; tasarım notu koddan önce; yeni entegrasyon testleri macOS ve Linux'ta; kademe 2 bir tetikleme ve Mac kümesi bir kez; P2-Q doğrulamasının koşulu VF-1 ve önerileri kılavuzda (Obsidian güven penceresi, masaüstü ajan ayrımı — sınanarak); CodeQL ve Dependabot uyarıları; 0.3.1 yayımı; pilotun D-NOTES senaryosu ve Obsidian'ın silme/taşıma adımları 0.3.1 ile yeniden). Yönergeler docs/rounds/P2-Q.1/GOAL.md ve TASKS.md; VERIFY.md turdan sonraki doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/ROADMAP.md §2, docs/DECISIONS.md K-P2Q.4, K-P2Q.5, K-GH1.1, K-D0.1, spec/SPEC.md §5, §7, §7.1, §8, docs/CLIENT_CONTRACT.md, docs/THREAT_MODEL.md, reports/rounds/P2-Q/FINAL.md, reports/rounds/P2-Q/FINDINGS.md (PQ-18, PQ-22), reports/rounds/P2-Q/VERIFICATION.md, docs/GUIDE.md, server/src/serve.rs, server/src/tools.rs (izleme alımı), server/CLAUDE.md, research/p2q/ (sürücü, pilot.yml) oku; sonra GOAL.md, TASKS.md. Dal: main'den git checkout -B p2-q.1/round; yayım hazırlığı release/0.3.1 dalında; R0'dan R5'e sırayla; DEV_PROCESS §2 disiplini; gözden geçirici yalnız R5'te; görev başına commit P2-Q.1/R<n>; her görev sonunda git push origin p2-q.1/round; R5 sonunda gh pr create --base main --head p2-q.1/round --title "P2-Q.1: izleyici yaması ve 0.3.1" --body-file reports/rounds/P2-Q.1/FINAL.md.
```

Ajanın sınırları (istemin dışında; GOAL'de de yazılı):
- Ürün değişikliği yalnız izleyicide (`server/src`); `core`, `index`, `bindings`, istemcilerin kaynakları, spec §1–6, fixture'lar değişmez. Spec'in tanımlamadığı durum uydurulmaz: PARK + karar talebi.
- Etiket atmaz, sürüm yayımlamaz, PR birleştirmez (insan adımı, `HUMAN_STEP.md`). Alt-ajan (gözden geçirici) çağrısı auto modda engellenirse kullanıcıdan auto moddan çıkıp onaylamasını ister.
- Kişisel içerik makineden çıkmaz (P2-Q'nün kuralları).
- Durma koşulunda STOP.md ve PARK; soru sorulmaz, varsayım ASSUMPTIONS.md'ye.

## P2-Q.1 doğrulayıcısı (ayrı oturum; P2-Q.1 PR'ı birleştikten sonra)

```
/goal reports/rounds/P2-Q.1/VERIFICATION.md p2q1/verification dalında commit'lendi ve dal uzağa itildi; dosya VR1–VR4 bölümleri, kriter başına geçti/kaldı satırları (Q1-1..Q1-9), PQ-22'yi 0.3.0'da yeniden üretip 0.3.1'de kapandığını gösteren kendi koşun, kendi kademe 2 tetiklemenin kimliği, tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil") ve en fazla 3 öneri içeriyor; main'e bu dosyayla bir PR açıldı. Rolün P2-Q.1'in bağımsız doğrulayıcısı (docs/DEV_PROCESS.md §5): docs/rounds/P2-Q.1/VERIFY.md VR1–VR4. Doğrulanan commit: <P2-Q.1'in birleşmiş main commit'i, sha>. Önce CLAUDE.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2Q.4, K-P2Q.5 ve K-P2Q1.x, docs/rounds/P2-Q.1/GOAL.md, docs/rounds/P2-Q.1/progress/DESIGN.md, reports/rounds/P2-Q.1/FINAL.md, spec/SPEC.md §5, §7, §7.1 oku. Kendi scratchpad'ine temiz klon al (make setup ve make models önce); kişisel içerik okuma. Dal: main'den git checkout -B p2q1/verification; commit "P2-Q.1/verification: VERIFICATION.md"; git push -u origin p2q1/verification; gh pr create --base main --head p2q1/verification --title "P2-Q.1 doğrulaması" --body-file reports/rounds/P2-Q.1/VERIFICATION.md.
```
