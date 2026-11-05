# P2-Q.1 — İnsan adımı

**Durum (2026-11-05, öğleden sonra): yayımın insan adımları tamamlandı.** PR #51 birleşti (`e74e1f8`), `release.yml` `41994456323` başarılı (`release.json`: zorunlu `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`, Windows da `ok`), `v0.3.1` `e74e1f8`'de etiketli, GitHub sürümü 12 dosyayla yayımlı. CodeQL açık uyarısı 0. Ajan R4 (d)'yi koşuyor (CI `pilot.yml` iki koşu, Docker, Mac, Obsidian sahnesi), sonra R5.

## Açık istekler (R5'ten önce)
1. ~~Tur dalının zorla itilmesi~~ — **yapıldı (kullanıcı):** uzak `p2-q.1/round` yeniden temellendirilmiş haliyle (`785b259`, `main` `e74e1f8`'in üstünde); sonraki commit'ler normal itmeyle.
2. ~~Kararın teyidi~~ — **yapıldı:** kullanıcı bu oturumda K-P2Q1-T1 (a)'yı teyit etti (K-P2Q1.2).
3. ~~Obsidian sahnesi için P2-Q'nün kişisel yol anahtarı~~ — **kararla kapandı (K-P2Q1.3: yeni anahtar, yeni ön-kayıt; sahne koşuldu).** Eski metin: (R4 (d), Q1-3).** `research/p2q/obsidian_scene.py` ön-kayıtlı seçimleri (`scenario.json` `obsidian`: silinen/adlandırılan notlar `k2:` özetleriyle) yalnız P2-Q'nün 32 baytlık anahtarıyla çözebilir (`P2Q_PATH_KEY_FILE`; P2-Q'de turun scratchpad'inde, depo dışında). Ajanın anahtar dosyasını araması otomatik modda (haklı olarak) reddedildi. **Beklenen:** anahtar dosyasının **yolu** (değeri değil; ajan dosyayı yalnız ortam değişkeniyle betiğe verir, içeriğini okumaz ya da yazmaz) ya da anahtar yoksa bunun söylenmesi — o durumda O06/O10/O14 ölçülemez ve Q1-3'ün koşu yarısı için karar talebi yazılır. Kopya `~/Works/Obs`'tan salt okunur alınır (P2-Q'nün kuralları: içerik makineden çıkmaz, kayıtta yalnız `k2:` ve sayılar).

