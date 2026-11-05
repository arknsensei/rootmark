# P2-Q.1 — İnsan adımı bekleniyor

**Durum (2026-11-05): R0–R3 TAMAMLANDI; R4'ün (a)–(c)'si yapıldı, (d) (pilotun 0.3.1 ile yeniden ölçümü) yayımlanmış 0.3.1'i bekliyor.** Yayım PR'ı **#51** (`release/0.3.1` → `main`, "rootmark 0.3.1") açık; yerel paket `release_check` **21/21** (`f5002de`; R15 gerçek Obsidian 11/11). Ayrıntı: `R4.md`. Kademe 2'nin düzeltilmiş commit'teki koşusu `41944358184` yeşil (D farkı 0, sayılan FAIL 0, portable 0); PR #51'in CI'ında `make test` macOS ve `ubuntu-latest`'te yeşil (yeni izleyici testleri Linux x86_64'te de).

## Şimdi beklenen — yayımın insan adımları (sırasıyla; `docs/RELEASE.md` "İnsan adımları")
1. PR #51'in CI'ı yeşil olunca birleşmesi (ajan PR birleştirmez).
2. `gh workflow run release.yml --ref main`; `release-json` artefaktının `dist/release.json`'ında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok` (Q1-8). Bu koşu `release.yml`'nin yeni `permissions: contents: read`'ini de ilk kez sınar.
3. `git tag -a v0.3.1 -m "rootmark 0.3.1"` birleşmiş `main` commit'inde; `git push origin v0.3.1`.
4. GitHub sürümü `v0.3.1`: `release-files` artefaktının dosyaları (arşivler, `install.sh`, `install.ps1`, `SHA256SUMS`) + yerel paketten wheel, `rootmark-0.3.1.zip`, eklentinin `main.js`, `manifest.json`, `styles.css` ve paketin `SHA256SUMS`'ı `SHA256SUMS.local` adıyla (kılavuz §5.1). Yerel paket ana çalışma dizininin `dist/`'indedir (git-ignored; `f5002de`'den kuruldu; ajan ona dokunmaz).

Ajan sürümü `gh release view v0.3.1` ile bekler; görünce R4 (d)'yi koşar: D-NOTES Mac ve Docker'da yayımlanmış 0.3.1 ile, CI'da `gh workflow run pilot.yml --ref main` iki kez; Obsidian kopyasında sahne (O06/O10/O14). Sonra R5 (rapor, gözden geçirici, READY, turun PR'ı).

## Ayrıca (bloke etmez)
- **CodeQL #2, #3, #4 (`js/bad-code-sanitization`, dondurulmuş gerçek Obsidian harness'i):** gerekçeli "used in tests" kapatması (Q1-7) — GitHub'da uyarı kapatma ajanın oturumunda otomatik modda reddedildi. Komut ve gerekçe `R3_security.md`. (#1 `release.yml` birleşmeden sonraki CodeQL taramasında kapanır.)
- **K-P2Q1-T1 (karar talebi):** Claude Code 2.1.318 rootmark'ın MCP araçlarını listeleyemiyor (PQ1-1; `rmcp 3.5.0` istek meta doğrulaması). Seçenekler `docs/DECISIONS.md`'de; bu turun ürün sınırı dışında.
- **Tur dalı ↔ `main`:** PR #51 tur dalının R0–R4 commit'lerini de taşır (yayım dalı tur dalının üstünde). `main`'e rebase ile girince tur dalının aynı commit'leri yeniden yazılmış olur; turun PR'ı (R5) için `p2-q.1/round` birleşmiş `main`'in üstüne yeniden temellendirilir ve **zorla itilmesi** gerekir (ajanın zorla itmesi bu oturumda reddedildi — P2-Q'deki gibi): beklenen `git push --force-with-lease=p2-q.1/round:<o anki uzak sha> origin p2-q.1/round` (kullanıcı) ya da bu itmeye izin. Ajan yeniden temellendirmeyi yerelde yapar ve buraya sha'yı yazar.
