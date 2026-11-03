# P2-Q — İnsan adımı bekleniyor

**Durum (2026-11-03): K-P2Q-T1 kararlaştırıldı — (a), kullanıcı bu oturumda; kayıt K-P2Q.2. Ek karar K-P2Q.3 (Türkçe sorguların ek ön-kaydı). `fseventsd` kullanıcı tarafından yeniden başlatıldı (e2e yeniden denenir).** Sıradaki insan adımları yayımındır (aşağıda); ajan yayım dalını hazırlayıp PR'ı taslaktan çıkarınca burası güncellenir.

**Açık istek — dosya olay servisi (ortam; R15'i bloke ediyor):** `fseventsd` (pid 107) **7 Ekim'den beri** aynı süreç, ~%100–160 CPU — yeniden başlatılmamış (2026-11-03 23:3x denetimi). Gerçek Obsidian e2e (`scripts/plugin_e2e_real.sh`) beş koşuda bir kez 11/11, dört kez 3/11 (hep adım 4: "timed out waiting for the agent's note in the vault"; 0.2.0 eklentisiyle de aynı): Obsidian dışarıdan yazılan notu görmüyor. Yayımın son kalan maddesi **R15** budur (`release_check` 20/21; R1–R14, R16–R20 `ok`, R16 K-P2Q.2 ile geçti). Yeşil çıkana kadar yeniden koşmak seçim olur; yapılmadı. **Beklenen:** Mac'in yeniden başlatılması (en güvenlisi) ya da `sudo killall fseventsd` (launchd yeniden başlatır); sonra bana haber — paketin e2e'si yeniden koşulur, R15 denetlenir, PR #46 taslaktan çıkar.

**Beklerken sürenler (ölçüm yok):** `docs/GUIDE.md`, README, `release/0.3.0` dalının hazırlığı (sürüm dizgileri, CHANGELOG, RELEASE.md; R1–R20'nin yerel denemesi; PR taslak), pilot sürücüsü, `observe.py`, Docker imajı ve `pilot.yml`.

**Karardan sonraki insan adımları (sırasıyla; Q1 (e)):**
1. `release/0.3.0` PR'ının (`rootmark 0.3.0`) birleşmesi.
2. `gh workflow run release.yml --ref main`; `release-json` artefaktında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`.
3. `git tag -a v0.3.0 -m "rootmark 0.3.0"` birleşmiş `main` commit'inde ve `git push origin v0.3.0`.
4. GitHub sürümü `v0.3.0`: `release-files` artefaktının dosyaları + eklentinin `main.js`, `manifest.json`, `styles.css` ayrıca (BRAT).
Ajan sürümü `gh release view v0.3.0` ile bekler, sonra Q2'ye geçer.
