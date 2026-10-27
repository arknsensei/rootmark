# P2-V — `/goal` İstemleri

Sıra: önce Blok 2 doğrulaması (`docs/rounds/P2-R/PROMPT.md`'deki doğrulayıcı istemi) biter ve PR'ı birleşir; sonra bu tur başlar.

## Ana ajan (repo klasöründe `claude`, `main` güncel; önce `/goal clear`)

```
/goal docs/rounds/P2-V/progress/READY.md "doğrulama için hazır", commit sha ve kademe 2 CI koşu kimlikleriyle yazıldı; reports/rounds/P2-V/FINAL.md ve reports/rounds/P2-V/inventory.json var; dal p2-v/round uzağa itildi ve main'e tek bir PR açık. Aktif tur P2-V (altyapı turu: doğrulama zincirini kademelere ayırma — envanter ve D/Z/O/H sınıflandırması ön-kayıt olarak, kademe 1 FAST CI'da PR'larda, kademe 2 tam çift yalnız CI'da A ve B paralel, Mac kümesi make verify-mac, host yolu sızıntısı denetimi, eşdeğerlik kanıtı). Yönergeler docs/rounds/P2-V/GOAL.md ve TASKS.md; VERIFY.md turdan sonraki doğrulayıcı içindir. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/ROADMAP.md §2, docs/DECISIONS.md K-P2.4, K-P2R.4, K-P2R.5, K-P2V.1, reports/rounds/P2-R/FINAL.md, reports/rounds/P2-R/VERIFICATION.md, scripts/verify.sh, Makefile (verify, fast-regress, ROUND_CHECKS), .github/workflows/verify.yml ve ci.yml, research/FROZEN.md, scripts/check_frozen.sh, scripts/release_check.sh oku; sonra GOAL.md, TASKS.md. Dal: main'den git checkout -B p2-v/round; V0'dan V6'ya sırayla; DEV_PROCESS §2 disiplini; gözden geçirici yalnız V6'da tek tur; görev başına commit P2-V/V<n>; her görev sonunda git push origin p2-v/round; V6 sonunda gh pr create --base main --head p2-v/round --title "P2-V: kademeli doğrulama zinciri" --body-file reports/rounds/P2-V/FINAL.md.
```

Ajanın sınırları (istemin dışında; GOAL'de de yazılı):
- Ürün kodu (`core`, `index`, `server`, `bindings`, `clients`), spec, uyumluluk fixture'ları, NORTH_STAR ve eski turların GOAL/eşikleri değişmez.
- Tam `make verify` yerelde koşulmaz; tam çift yalnız CI'da.
- V1 envanteri V2'den önce commit'lenir ve sonra değişmez.
- Dondurulmuş ölçüm kodu yalnız istisna prosedürüyle (DEV_PROCESS §4).
- Kriter kalırsa optimize edilmez; raporlanır ve karar kuralı uygulanır.
- Durma koşulunda STOP.md ve PARK; soru sorulmaz, varsayım ASSUMPTIONS.md'ye.
- PR'ı orkestratör birleştirir; bağımsız doğrulama ayrı oturumda.

## P2-V doğrulayıcısı (ayrı oturum, repo klasöründe `claude`, `main` güncel; önce `/goal clear`; P2-V PR'ı birleştikten sonra)

```
/goal reports/rounds/P2-V/VERIFICATION.md p2v/verification dalında commit'lendi, dal uzağa itildi ve main'e bu dosyayı içeren tek bir PR açık; VERIFICATION.md VV1–VV5 bölümleri, kriter başına geçti/kaldı satırları (V-1..V-8), kendi tetiklediğin kademe 2 CI koşusunun kimliği, tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil") ve en fazla 3 öneri içeriyor. Rolün P2-V'nin bağımsız doğrulayıcısı (docs/DEV_PROCESS.md §5, K-P2V.1): docs/rounds/P2-V/VERIFY.md VV1–VV5. Doğrulanan commit: <P2-V'nin birleşmiş main commit'i, sha>. Önce CLAUDE.md, docs/DEV_PROCESS.md, docs/DECISIONS.md K-P2R.5, K-P2V.1, K-P2V.2, docs/rounds/P2-V/GOAL.md, reports/rounds/P2-V/FINAL.md, reports/rounds/P2-V/inventory.md oku. Kendi scratchpad'ine temiz klon al ve yerel koşturmaları orada yap. Dal: main'den git checkout -B p2v/verification; commit "P2-V/verification: VERIFICATION.md"; git push -u origin p2v/verification; gh pr create --base main --head p2v/verification --title "P2-V doğrulaması" --body-file reports/rounds/P2-V/VERIFICATION.md.
```
