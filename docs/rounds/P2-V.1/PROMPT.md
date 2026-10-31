# P2-V.1 — `/goal` İstemleri

Sıra: P2-V'yi kapatan ve bu turu açan PR birleşir; sonra bu tur başlar.

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-V.1/progress/READY.md "doğrulama için hazır", commit sha, kademe 2 ve E2 CI koşu kimlikleriyle yazıldı; reports/rounds/P2-V.1/FINAL.md, reports/rounds/P2-V.1/e0/feasibility.md ve reports/rounds/P2-V.1/inventory.json var; dal p2-v.1/round uzağa itildi ve main'e tek bir PR açık. Aktif tur P2-V.1 (yama turu: gömme vektörlerini işlemciden bağımsız kılmak — önce iki işlemcide fizibilite ölçümü ve seçim, sonra ürün değişikliği, indeks göçü ve üç koşucuda bayt eşitliği; P2-V gözden geçiricisinin araç bulguları B1, B2, B3, B6, R14; CPU modeli köprüsü; karşılaştırıcı hızı; Mac kümesinde tekrarsız deterministik kriterler ve tarihsel etiket). Yönergeler docs/rounds/P2-V.1/GOAL.md ve TASKS.md; VERIFY.md turdan sonraki doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/ROADMAP.md §2, docs/DECISIONS.md K-P2.4, K-P2V.1, K-P2V.2, K-P2V.3 ve K-P2V-T1..T4, reports/rounds/P2-V/FINAL.md, reports/rounds/P2-V/VERIFICATION.md, reports/rounds/P2-V/inventory.md, docs/rounds/P2-V/progress/V6.review.md, docs/rounds/P2-V/progress/FREEZE_EXCEPTIONS.md, index/src/embed.rs, index/src/vectors.rs, research/eval/tier2.py, research/eval/historical.py, research/eval/verify_mac.py, research/eval/portable.py, .github/workflows/verify.yml ve ci.yml oku; sonra GOAL.md, TASKS.md. Dal: main'den git checkout -B p2-v.1/round; E0'dan E5'e sırayla; DEV_PROCESS §2 disiplini; gözden geçirici E1 sonunda ve E5'te; görev başına commit P2-V.1/E<n>; her görev sonunda git push origin p2-v.1/round; E5 sonunda gh pr create --base main --head p2-v.1/round --title "P2-V.1: gömme determinizmi ve kademelerin tamamlanması" --body-file reports/rounds/P2-V.1/FINAL.md.
```

Ajanın sınırları (istemin dışında; GOAL'de de yazılı):
- Ürün değişikliği yalnız gömmenin sayısal çıktısı (`index/`) ve göç kaydı (`server/`); embedder modeli, tokenizer, sözcüksel indeks, spec §1–6, uyumluluk fixture'ları, NORTH_STAR ve eski turların GOAL/eşikleri değişmez.
- E0'ın seçimi E1'den önce, E3'ün envanteri araç değişikliklerinden önce commit'lenir ve sonra değişmez.
- Tam `make verify` yerelde koşulmaz; tam çift yalnız CI'da. Mac kümesi sessiz makine gerektirmez (K-P2V.3).
- Dondurulmuş ölçüm kodu yalnız istisna prosedürüyle (DEV_PROCESS §4).
- Kriter kalırsa optimize edilmez; raporlanır ve karar kuralı uygulanır.
- Durma koşulunda STOP.md ve PARK; soru sorulmaz, varsayım ASSUMPTIONS.md'ye.
- PR'ı orkestratör birleştirir; bağımsız doğrulama ayrı oturumda.

## P2-V.1 doğrulayıcısı (ayrı oturum, repo klasöründe `claude`, `main` güncel; önce `/goal clear`; P2-V.1 PR'ı birleştikten sonra)

```
/goal reports/rounds/P2-V.1/VERIFICATION.md p2v1/verification dalında commit'lendi ve dal uzağa itildi; dosya VW1–VW5 bölümleri, kriter başına geçti/kaldı satırları (V1-1..V1-11), kendi tetiklediğin kademe 2 ve E2 CI koşularının kimlikleri, tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil") ve en fazla 3 öneri içeriyor; main'e bu dosyayla bir PR açıldı. Rolün P2-V.1'in bağımsız doğrulayıcısı (docs/DEV_PROCESS.md §5): docs/rounds/P2-V.1/VERIFY.md VW1–VW5. Doğrulanan commit: <P2-V.1'in birleşmiş main commit'i, sha>. Önce CLAUDE.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2V.3 ve K-P2V1.1, docs/rounds/P2-V.1/GOAL.md, reports/rounds/P2-V.1/FINAL.md, reports/rounds/P2-V.1/e0/feasibility.md, reports/rounds/P2-V.1/inventory.md oku. Kendi scratchpad'ine temiz klon al ve yerel koşturmaları orada yap. Dal: main'den git checkout -B p2v1/verification; commit "P2-V.1/verification: VERIFICATION.md"; git push -u origin p2v1/verification; gh pr create --base main --head p2v1/verification --title "P2-V.1 doğrulaması" --body-file reports/rounds/P2-V.1/VERIFICATION.md.
```
