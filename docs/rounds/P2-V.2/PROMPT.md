# P2-V.2 — `/goal` İstemleri

Sıra: P2-V.1'i kapatan PR (#41) ve bu turu açan PR birleşir; sonra bu tur başlar.

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-V.2/progress/READY.md "doğrulama için hazır", commit sha ve turun PR'ındaki CI koşu kimliğiyle yazıldı; reports/rounds/P2-V.2/FINAL.md, reports/rounds/P2-V.2/inventory.json, reports/rounds/P2-V.2/reread/summary.md ve reports/rounds/P2-V.2/y4/mac_checks.json var; dal p2-v.2/round uzağa itildi ve main'e tek bir PR açık. Aktif tur P2-V.2 (kapanış mini turu, yeni ölçüm yok: K-P2V1.2'nin dört kararını envantere ve araçlara işlemek — yeni envanter önce commit'lenir; P2-V.1'in mevcut kademe 2 koşuları 41239746833, 41239754895, 41292205176 ve Mac kaydı yeni envanterle yeniden okunur; Mac kümesinin D-10 zaman aşımında kayıt kaybı (Ş1) giderilir ve tek tekrarlı enjekte koşuyla test edilir; DEV_PROCESS §3 ve CLAUDE.md'de kademeler yürürlüğe girer). Yönergeler docs/rounds/P2-V.2/GOAL.md ve TASKS.md; VERIFY.md turdan sonraki doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/ROADMAP.md §2, docs/DECISIONS.md K-P2V.1, K-P2V.3, K-P2V1.1, K-P2V1.2 ve K-P2V1-T2..T5, reports/rounds/P2-V.1/FINAL.md, reports/rounds/P2-V.1/VERIFICATION.md, reports/rounds/P2-V.1/inventory.md, docs/rounds/P2-V.1/progress/E5.review.md, docs/rounds/P2-V.1/progress/FREEZE_EXCEPTIONS.md, research/eval/tier2.py, research/eval/verify_mac.py, research/eval/portable.py, research/p2v1/inventory.py, scripts/verify_mac.sh, server/tests/serve_watch.rs, .github/workflows/verify.yml oku; sonra GOAL.md, TASKS.md. Dal: main'den git checkout -B p2-v.2/round; Y0'dan Y5'e sırayla; DEV_PROCESS §2 disiplini; gözden geçirici yalnız Y5'te; görev başına commit P2-V.2/Y<n>; her görev sonunda git push origin p2-v.2/round; Y5 sonunda gh pr create --base main --head p2-v.2/round --title "P2-V.2: kademelerin yürürlüğe girmesi" --body-file reports/rounds/P2-V.2/FINAL.md.
```

Ajanın sınırları (istemin dışında; GOAL'de de yazılı):
- Yeni ölçüm yok: `verify.yml` tetiklenmez, Mac kümesi 3 tekrar koşulmaz, tam `make verify` yerelde koşulmaz; tek yeni koşu Y4'ün tek tekrarlı enjekte koşusudur.
- Envanterin ekleri yalnız K-P2V1.2'nin dört adı; yeniden okuma hâlâ FAIL ya da adsız fark veriyorsa yeni ad eklenmez, raporlanır, karar kuralı uygulanır.
- Y1'in envanteri Y2'den önce commit'lenir ve sonra değişmez; P2-V ve P2-V.1 envanterleri aynen kalır.
- Dondurulmuş ölçüm kodu yalnız istisna prosedürüyle (DEV_PROCESS §4), her değişiklik için önce/sonra bayt eşitliği kanıtı.
- Ürün kodu değişmez (`core`, `index`, `server/src`, `bindings`, `clients`, spec); `server/tests/serve_watch.rs` yalnız zaman aşımı kaydı ve test-içi eşik düğmesi.
- Durma koşulunda STOP.md ve PARK; soru sorulmaz, varsayım ASSUMPTIONS.md'ye.
- PR'ı orkestratör birleştirir; bağımsız doğrulama ayrı oturumda.

## P2-V.2 doğrulayıcısı (ayrı oturum, repo klasöründe `claude`, `main` güncel; önce `/goal clear`; P2-V.2 PR'ı birleştikten sonra)

```
/goal reports/rounds/P2-V.2/VERIFICATION.md p2v2/verification dalında commit'lendi ve dal uzağa itildi; dosya VY1–VY4 bölümleri, kriter başına geçti/kaldı satırları (V2-1..V2-8), kendi indirdiğin yapıtların blake2b'leri, kendi koşturduğun yeniden okumanın ve Y4 (b) tekrarının sonuçları, tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil") ve en fazla 3 öneri içeriyor; main'e bu dosyayla bir PR açıldı. Rolün P2-V.2'nin bağımsız doğrulayıcısı (docs/DEV_PROCESS.md §5): docs/rounds/P2-V.2/VERIFY.md VY1–VY4. Doğrulanan commit: <P2-V.2'nin birleşmiş main commit'i, sha>. Önce CLAUDE.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2V1.2 ve K-P2V2.1, docs/rounds/P2-V.2/GOAL.md, reports/rounds/P2-V.2/FINAL.md, reports/rounds/P2-V.2/inventory.md, reports/rounds/P2-V.2/reread/summary.md oku. Kendi scratchpad'ine temiz klon al ve yerel koşturmaları orada yap; kademe 2 tetikleme, Mac kümesini 3 tekrar koşturma. Dal: main'den git checkout -B p2v2/verification; commit "P2-V.2/verification: VERIFICATION.md"; git push -u origin p2v2/verification; gh pr create --base main --head p2v2/verification --title "P2-V.2 doğrulaması" --body-file reports/rounds/P2-V.2/VERIFICATION.md.
```
