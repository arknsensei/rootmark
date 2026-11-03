# P2-Q — İnsan adımı bekleniyor

**Durum (2026-11-03): K-P2Q-T1 kararlaştırıldı — (a), kullanıcı bu oturumda; kayıt K-P2Q.2. Ek karar K-P2Q.3 (Türkçe sorguların ek ön-kaydı). `fseventsd` kullanıcı tarafından yeniden başlatıldı (e2e yeniden denenir).** Sıradaki insan adımları yayımındır (aşağıda); ajan yayım dalını hazırlayıp PR'ı taslaktan çıkarınca burası güncellenir.

**İkinci istek — makinenin dosya olay servisi (ortam):** `fseventsd` 27 gündür ~%99 CPU'da (Finder ~%92; `uptime` 27 gün). Bu durumda gerçek Obsidian dışarıdan yazılan bir notu görmüyor: `scripts/plugin_e2e_real.sh` hem 0.3.0 adayında hem `main`'in 0.2.0 eklentisiyle aynı adımda kalıyor (3/11, adım 4 "timed out waiting for the agent's note in the vault"); yayım maddesi **R15** (gerçek Obsidian e2e 11/11) ve Q2 (c) (Obsidian sahnesi) buna bağlı. P2-V.2'de de aynı belirti not edilmişti (`fseventsd` %99,7). **Beklenen:** Mac'in yeniden başlatılması (ya da `sudo killall fseventsd`); ardından bana haber. Ajan sistem süreçlerine dokunmaz.

**Beklerken sürenler (ölçüm yok):** `docs/GUIDE.md`, README, `release/0.3.0` dalının hazırlığı (sürüm dizgileri, CHANGELOG, RELEASE.md; R1–R20'nin yerel denemesi; PR taslak), pilot sürücüsü, `observe.py`, Docker imajı ve `pilot.yml`.

**Karardan sonraki insan adımları (sırasıyla; Q1 (e)):**
1. `release/0.3.0` PR'ının (`rootmark 0.3.0`) birleşmesi.
2. `gh workflow run release.yml --ref main`; `release-json` artefaktında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`.
3. `git tag -a v0.3.0 -m "rootmark 0.3.0"` birleşmiş `main` commit'inde ve `git push origin v0.3.0`.
4. GitHub sürümü `v0.3.0`: `release-files` artefaktının dosyaları + eklentinin `main.js`, `manifest.json`, `styles.css` ayrıca (BRAT).
Ajan sürümü `gh release view v0.3.0` ile bekler, sonra Q2'ye geçer.
