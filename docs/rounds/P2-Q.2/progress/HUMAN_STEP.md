# P2-Q.2 — İnsan adımı bekleniyor

**Durum (2026-11-08, 22:30Z): R0–R2 TAMAMLANDI; R3 KISMİ (yalnız minimatch, 2026-11-10T22:26Z'den sonra); R4'ün (a)–(c)'si yapıldı, (d) (yayımlanmış 0.3.2 ile gerçek Claude Code ve D-NOTES) yayımlanmış 0.3.2'yi bekliyor.** Yayım PR'ı **#55** (`release/0.3.2` → `main`, "rootmark 0.3.2") açık. Yerel paket `release_check` **21/21** verdi (`8525371`; R15 gerçek Obsidian 11/11). Aynı ürün kodunda (`d6eb311`) kademe 2 `42515401135` temiz, Mac kümesi `pass`. Ayrıntı: `R4.md`.

## Şimdi beklenen — yayımın insan adımları (sırasıyla; `docs/RELEASE.md` "İnsan adımları")
1. PR #55'in CI'ı yeşil olunca birleşmesi (ajan PR birleştirmez).
2. `gh workflow run release.yml --ref main`. `release-json` artefaktının `dist/release.json`'ında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok` olmalı (39/39).
3. `git tag -a v0.3.2 -m "rootmark 0.3.2"` birleşmiş `main` commit'inde, ardından `git push origin v0.3.2`.
4. GitHub sürümü `v0.3.2`. Eklenecekler:
   - `release-files` artefaktının dosyaları (arşivler, `install.sh`, `install.ps1`, `SHA256SUMS`);
   - yerel paketten wheel, `obsidian/rootmark-0.3.2.zip`, eklentinin `main.js`, `manifest.json` ve `styles.css`'i ayrıca, paketin `SHA256SUMS`'ı da `SHA256SUMS.local` adıyla (kılavuz §5.1).

   Yerel paket şurada: `/private/tmp/claude-501/-Users-arkns-Works-mdkernel/45ea385a-f188-48a6-b3f5-639ccf321ebe/scratchpad/rel032/dist/` (`release/0.3.2` çalışma ağacı, `8525371`'den kuruldu; sha'lar `reports/rounds/P2-Q.2/release/SHA256SUMS.local`). İsterseniz paket ana dizinde `bash scripts/release.sh` ile yeniden kurulur (≈ 15 dk, Docker ve Obsidian gerekir).

Ajan sürümü `gh release view v0.3.2` ile bekler. Görünce R4 (d)'yi koşar:
- yüklü 0.3.2 ile gerçek Claude Code (D-NOTES kopyası);
- D-NOTES senaryosu Mac'te, Docker'da ve CI'da (`gh workflow run pilot.yml --ref main`, bir koşu); 0.3.1 kayıtlarıyla bayt karşılaştırması.

Sonra R3 (e) (minimatch, 2026-11-10T22:26Z'den sonra) ve R5 gelir (rapor, gözden geçirici, READY, turun PR'ı).

## Ayrıca
- **Tur dalı ↔ `main`:** PR #55 tur dalının R0–R3 commit'lerini de taşıyor (yayım dalı `d6eb311`'in üstünde). `main`'e rebase ile girerse turun PR'ı (R5) için `p2-q.2/round` birleşmiş `main`'in üstüne yeniden temellendirilir. Bu **zorla itme** gerektirir; ajanın zorla itmesi P2-Q ve P2-Q.1'de reddedilmişti. Beklenen: `git push --force-with-lease=p2-q.2/round:<o anki uzak sha> origin p2-q.2/round` (kullanıcı) ya da bu itmeye izin. Ajan yeniden temellendirmeyi yerelde yapar ve sha'yı buraya yazar. PR #55 merge commit ya da hızlı ileri sarmayla girerse buna gerek yok.
- **Claude Code oturumu:** R4 (d)'nin gerçek Claude Code sınaması `claude -p` ile oturum açılmış bir Claude Code ister. Şu an `claude auth status` → `loggedIn: true` (2.1.319). Oturum düşerse `! claude auth login` gerekir.
