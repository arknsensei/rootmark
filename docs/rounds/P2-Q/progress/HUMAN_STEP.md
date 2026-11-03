# P2-Q — İnsan adımı bekleniyor

**Durum (2026-11-03 21:12Z): yayım insan adımları tamamlandı — `v0.3.0` yayımlı (PR #46 `0528ee8`, `release.yml` `41717748927`). Açık kalan tek istek: yeniden temellendirilmiş `p2-q/round`'un uzağa zorla itilmesi (aşağıda). Önceki durum: K-P2Q-T1 kararlaştırıldı — (a), kullanıcı bu oturumda; kayıt K-P2Q.2. Ek karar K-P2Q.3 (Türkçe sorguların ek ön-kaydı). `fseventsd` kullanıcı tarafından yeniden başlatıldı (e2e yeniden denenir).** Sıradaki insan adımları yayımındır (aşağıda); ajan yayım dalını hazırlayıp PR'ı taslaktan çıkarınca burası güncellenir.

**Dosya olay servisi — giderildi:** `fseventsd` yeniden başladı (yeni süreç, 2026-11-03 23:22; CPU ~%0). Paketin gerçek Obsidian e2e'si yeniden koşuldu: **11/11** (E1-9, E-3 doğru; sözleşme dışı istek 0/57); `release_check.sh` **21/21**.

**Şimdi beklenen — yayımın insan adımları (sırasıyla):**
0. PR #46'nın taslaktan çıkarılması (`gh pr ready 46`; ajanın bu oturumdaki yetkisi dışında kaldı).
1. PR #46 (`rootmark 0.3.0`, `release/0.3.0`) CI yeşil olunca birleşmesi.
2. `gh workflow run release.yml --ref main`; `release-json` artefaktında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`.
3. `git tag -a v0.3.0 -m "rootmark 0.3.0"` birleşmiş `main` commit'inde, `git push origin v0.3.0`.
4. GitHub sürümü `v0.3.0`: `release-files` artefaktının dosyaları (arşivler, `install.sh`, `install.ps1`, `SHA256SUMS`) + yerel paketten wheel, `rootmark-0.3.0.zip`, eklentinin `main.js`, `manifest.json`, `styles.css` ve paketin `SHA256SUMS`'ı `SHA256SUMS.local` adıyla (kılavuz §5.1 eklenti zip'ini onunla denetler). Yerel paketin yeri ajanın çalışma dizinindedir (sohbette verildi; kayda host yolu yazılmaz).
Ajan sürümü `gh release view v0.3.0` ile bekler, görünce Q2'ye geçer.

**Beklerken sürenler (ölçüm yok):** `docs/GUIDE.md`, README, `release/0.3.0` dalının hazırlığı (sürüm dizgileri, CHANGELOG, RELEASE.md; R1–R20'nin yerel denemesi; PR taslak), pilot sürücüsü, `observe.py`, Docker imajı ve `pilot.yml`.

**Karardan sonraki insan adımları (sırasıyla; Q1 (e)):**
1. `release/0.3.0` PR'ının (`rootmark 0.3.0`) birleşmesi.
2. `gh workflow run release.yml --ref main`; `release-json` artefaktında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`.
3. `git tag -a v0.3.0 -m "rootmark 0.3.0"` birleşmiş `main` commit'inde ve `git push origin v0.3.0`.
4. GitHub sürümü `v0.3.0`: `release-files` artefaktının dosyaları + eklentinin `main.js`, `manifest.json`, `styles.css` ayrıca (BRAT).
Ajan sürümü `gh release view v0.3.0` ile bekler, sonra Q2'ye geçer.

**Açık istek (2026-11-04): `p2-q/round`'un zorla itilmesi.** Yayım `main`'e birleşince tur dalı yeni `main`'in (`0528ee8`) üstüne yeniden temellendirildi (yerel; turun kendi dosyaları öncekiyle aynı). Ajanın zorla itmesi bu oturumda reddedildi. **Beklenen:** `git push --force-with-lease=p2-q/round:f43dd05 origin p2-q/round` (kullanıcı) ya da bu itmeye izin. Q2'nin Mac aşaması yerelde sürer; CI işi (`pilot.yml --ref p2-q/round`) ve turun PR'ı uzak dalın güncel olmasını bekler.
