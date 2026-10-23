# P2-0.3 — `/goal` İstemleri

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-0.3/progress/READY.md "doğrulama için hazır" ile yazıldı, reports/rounds/P2-0.3/FINAL.md var, dal p2-0.3/round uzağa itildi ve main'e tek bir PR açık. Aktif tur P2-0.3 (hipotez turu, kod değişikliği yok: P2-A/A0'ın (A) düzeltmesinden sonra P2-0 hipotezinin yeni tohumla ve iki ortamda yeniden ölçümü). Yönergeler docs/rounds/P2-0.3/GOAL.md ve TASKS.md; VERIFY.md yalnız doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2-02.5, K-P2.3, K-P2A.5, reports/rounds/P2-0.2/FINAL.md, reports/rounds/P2-0.2/VERIFICATION.md (F2: değerin ortam bağımlılığı), reports/rounds/P2-A/FINAL.md (bilgi ölçümü), docs/rounds/P2-0.2/progress/ASSUMPTIONS.md (X0-7, X0-15, X0-18), research/eval/run_p2_0_2.py ve datasets/p2_0_2_sets.py oku; sonra GOAL.md, TASKS.md. Dal: main'den p2-0.3/round aç; R0, R1, R2 sırayla; DEV_PROCESS §2 disiplini (gözden geçirici görev başına en fazla bir tur; commit P2-0.3/R<n>; her görev sonunda git push origin p2-0.3/round; R2 sonunda gh pr create --base main --head p2-0.3/round --title "P2-0.3: yeniden ölçüm" --body-file reports/rounds/P2-0.3/FINAL.md). Kırmızı çizgiler: core, index, server, clients, spec, üreteçler, run_p2_0_2.py, eşikler ve W değişmez (git diff main bu yollarda boş kalır); test-3 ve P2-A bilgi ölçümünün tohumları yeniden kullanılmaz; varsayılan politika bu turda değiştirilmez; NORTH_STAR'ı değiştirme, öneri yaz; damga koyma; yayım yok; sessiz başarısızlık yok. Hipotez düşerse optimize etme, raporla, karar kuralını uygula. Durma koşulunda STOP.md ve PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz. make verify kalite kapısıdır, bağımsız doğrulama değildir; bağımsız doğrulama ve PR birleştirme senin işin değildir.
```

## Doğrulayıcı (ayrı oturum, repo klasöründe `claude`; önce `/goal clear`)

```
/goal reports/rounds/P2-0.3/VERIFICATION.md yazıldı ve hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"), hipotez başına geçti/kaldı, en fazla 3 öneri, V10 vaat paragrafı ve damga görüşü içeriyor. Rolün bağımsız doğrulayıcı (docs/DEV_PROCESS.md §5): docs/rounds/P2-0.3/VERIFY.md V0, V1, V3, V7, V9, V10'u uygula. Doğrulanan commit: <PR'ın başı, sha>. Kendi scratchpad'ine temiz klon al, her koşturmayı orada yap; ana dizine yalnız VERIFICATION.md'yi yaz, kod değiştirme. Gerçek Obsidian'ı ayrı örnekte (--user-data-dir, CDP) sür, kullanıcının açık Obsidian'ına dokunma; çift koşturma sırasında Obsidian deneyi yapma.
```
