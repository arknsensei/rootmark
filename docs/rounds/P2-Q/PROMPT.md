# P2-Q — `/goal` İstemleri

Sıra: bu turu açan PR (P2-V.2 kapanışı K-P2V2.2 + P2-Q açılışı K-P2V2.3) birleşir; sonra bu tur başlar.

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-Q/progress/READY.md "doğrulama için hazır", commit sha ve turun PR'ındaki CI koşu kimliğiyle yazıldı; reports/rounds/P2-Q/FINAL.md, reports/rounds/P2-Q/FINDINGS.md, reports/rounds/P2-Q/numbers.json ve docs/rounds/P2-Q/progress/SCENARIO.md var; dal p2-q/round uzağa itildi ve main'e tek bir PR açık — ya da tur bir insan adımında bekliyor ve docs/rounds/P2-Q/progress/HUMAN_STEP.md bekleneni yazıyor. Aktif tur P2-Q (pilot: 0.3.0 yayımı ve yayımlanmış paketle gerçek içerikte kullanım; önce Mac — qorbit md kopyası başsız ajan sahnesiyle, kullanıcının Obsidian vault'unun kopyası gerçek Obsidian'la — sonra kullanıcının erişim verdiği Linux sunucuda qorbit'in yanında, ona dokunmadan; ürün kodu değişmez, bulgular düzeltilmez kaydedilir; hız sayıları eşiksiz ölçülür; kişisel içerik hiçbir kayda girmez). Yönergeler docs/rounds/P2-Q/GOAL.md ve TASKS.md; VERIFY.md turdan sonraki doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/ROADMAP.md §2, docs/DECISIONS.md K-Q.1, K-P2V1.2, K-P2V2.2, K-P2V2.3, docs/RELEASE.md, README.md (Türkçe yapılandırma), docs/THREAT_MODEL.md, reports/rounds/P2-V.2/FINAL.md, packaging/install.sh, scripts/release.sh, scripts/release_check.sh, .github/workflows/release.yml, clients/obsidian/e2e/ (gerçek Obsidian harness'i) oku; sonra GOAL.md, TASKS.md. Dal: main'den git checkout -B p2-q/round; yayım hazırlığı release/0.3.0 dalında; Q0'dan Q4'e sırayla; DEV_PROCESS §2 disiplini; gözden geçirici yalnız Q4'te; görev başına commit P2-Q/Q<n>; her görev sonunda git push origin p2-q/round; Q4 sonunda gh pr create --base main --head p2-q/round --title "P2-Q: pilot ve 0.3.0" --body-file reports/rounds/P2-Q/FINAL.md.
```

Ajanın sınırları (istemin dışında; GOAL'de de yazılı):
- Etiket atmaz, sürüm yayımlamaz, `release.yml`'yi tetiklemez; PR'ları birleştirmez (insan adımı, `HUMAN_STEP.md`; onay şartı için K-P2V2.2).
- Asıl vault'a (`~/Works/Obs`) ve qorbit deposuna yazmaz; yalnız kopyalar, depo dışında. Kişisel içerik kayda, günlüğe, PR'a girmez.
- Sunucuda qorbit'in servislerine, dosyalarına, portlarına dokunmaz; yalnız `127.0.0.1`; erişim yoksa Q3 PARK.
- Ürün hatasını düzeltmez; `FINDINGS.md`'ye yazar.
- Durma koşulunda STOP.md ve PARK; soru sorulmaz, varsayım ASSUMPTIONS.md'ye.

## P2-Q doğrulayıcısı (ayrı oturum; P2-Q PR'ı birleştikten sonra)

```
/goal reports/rounds/P2-Q/VERIFICATION.md p2q/verification dalında commit'lendi ve dal uzağa itildi; dosya VQ1–VQ4 bölümleri, kriter başına geçti/kaldı satırları (Q-1..Q-9), kendi kurulumunun ve kendi senaryo koşusunun sonuçları, tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil") ve en fazla 3 öneri içeriyor; main'e bu dosyayla bir PR açıldı. Rolün P2-Q'nün bağımsız doğrulayıcısı (docs/DEV_PROCESS.md §5): docs/rounds/P2-Q/VERIFY.md VQ1–VQ4. Doğrulanan commit: <P2-Q'nün birleşmiş main commit'i, sha>. Önce CLAUDE.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2V2.3 ve K-P2Q.1, docs/rounds/P2-Q/GOAL.md, reports/rounds/P2-Q/FINAL.md, reports/rounds/P2-Q/FINDINGS.md oku. Kendi scratchpad'ine temiz klon al (make setup ve make models önce); kişisel vault'un içeriğini okuma. Dal: main'den git checkout -B p2q/verification; commit "P2-Q/verification: VERIFICATION.md"; git push -u origin p2q/verification; gh pr create --base main --head p2q/verification --title "P2-Q doğrulaması" --body-file reports/rounds/P2-Q/VERIFICATION.md.
```
