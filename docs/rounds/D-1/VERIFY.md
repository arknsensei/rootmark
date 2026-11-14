# D-1 — Bağımsız Doğrulama

**Çalışma koşulları:**
- Ayrı oturum, temiz klon (kendi scratchpad'in; önce `make setup`, `make models`). Ana çalışma dizinine erişim yok; kod değiştirilmez.
- Hedef ≤ 3 saat.
- Ürün deposuna, tap deposuna, kayıt defterlerine yazma yok; yalnız okuma ve yayımlanmış varlıkları indirme.
- Kişisel içerik okunmaz; vault olarak sentetik kasa ya da D-NOTES kopyası.

- **VD1 — Aktarımı kendin üret.** Laboratuvarı `v0.4.0`'da klonla, `research/transfer/`'i kendin koş: ağaç kimliği ürün `main`'inin ağacına ve FINAL'deki üç kimliğe eşit mi? `TRANSFER.json`'dan rastgele 30 `lab:` satırı seç ve blob'larını etiketle karşılaştır. Kendi yazdığın bir sızıntı taraması (ürün ağacında `research/`, `reports/`, `docs/rounds/` öneki; `.rootmark` bileşeni; ev dizini, kullanıcı adı, e-posta deseni; token desenli dizgi; Türkçe süreç belgesi adları) — aracın kendi denetiminden bağımsız olarak.
- **VD2 — Kanalları yabancı gibi kur.** Yalnız ürün README'si ve siteyi okuyarak, temiz bir Linux konteynerinde ve kendi macOS'unda: `install.sh`, `brew install rootmark-ai/tap/rootmark`, `docker run` hızlı başlangıcı (iki kayıt defterinden de), Obsidian BRAT varlıkları. Her yolda `--version`, `rootmark conformance`; `sha256sum -c SHA256SUMS`; `gh attestation verify -R rootmark-ai/rootmark` (arşivler, imaj). İki kayıt defterindeki imaj özeti eşit mi? Konteynerde `--watch` bağlanan birimdeki düzenlemeyi alıyor mu? README'nin komutları yazıldığı gibi koşuyor mu?
- **VD3 — Belgeler ve iddialar.** `docs.rootmark.ai`'nin bütün sayfaları ve bağlantıları; "git ile fark" sayfasının her iddiasını atıf yaptığı laboratuvar kaydında bul ve iddianın kayıtla aynı koşulda olduğunu göster (ölçülen koşul yazılı mı; abartı var mı?). Uyumluluk rozeti ölçülmüş `conformance.json`'la tutarlı mı?
- **VD4 — Ürün davranışı, güvenlik, karar kuralı.** `git diff ROUND_START..<sha> -- core index server bindings spec clients/*/src` yalnız sürüm dizgileri mi; `tools/list` 0.3.4 ile bayt eşit mi? Laboratuvar `ci.yml`'inde dondurma denetimleri `deps`'te mi ve yalnız dondurulmuş bir `.md`'yi değiştiren dalda kalıyor mu (kendi dalınla, itmeden `act` ya da betik düzeyinde)? R21 FREEZE_EXCEPTIONS'ta mı ve eski kayıtla FAIL veriyor mu? İki depoda açık Dependabot ve CodeQL uyarıları; ürün deposunda ve iş akışlarında gizli bilgi, yönetici posta adresi ve kişisel içerik 0 mı? Karar kuralını GOAL'den türet, FINAL ile karşılaştır.
- **Vaat sorusu (NORTH_STAR):** ürünün ilk halka açık yüzü (README, site, "git ile fark") ölçülen koşulu doğru anlatıyor mu — vaadi o koşulun ötesine genişletiyor mu?
- **Sonuç:** ihlal/fark/şüphe; D1-1..D1-12 için geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
