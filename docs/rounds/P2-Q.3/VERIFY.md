# P2-Q.3 — Bağımsız Doğrulama (kısa; yama turu)

**Çalışma koşulları:**
- Ayrı oturum, temiz klon (kendi scratchpad'in; önce `make setup`, `make models`). Ana çalışma dizinine erişim yok; kod değiştirilmez.
- Hedef ≤ 2 saat.
- Kademe 2'yi bir kez kendin tetikle; Mac kümesini koşma, turun kaydını oku.
- Kişisel içerik okunmaz; vault olarak sentetik kasa ya da D-NOTES kopyası kullanılır.

- **VR1 — Tasarım ve sınır.**
  - `DESIGN.md` ve `TESTS.md`'nin R1/R2'nin kodundan önce commit'li olduğunu `git log` ile göster.
  - Uzlaştırmanın kuralını spec §7.1 ile karşılaştır: başlangıçta tasdik penceresi yok; `unverified` ve `canonical` politikalarında katman ve aktör izleyicinin tasdiksiz dış değişikliğiyle aynı mı? Spec'te olmayan bir olay türü ya da alan eklendi mi?
  - `git diff ROUND_START..<sha>`: ürün değişikliği yalnız `serve.rs`, `tools.rs`, gerekirse `main.rs` ve yeni testler mi? `docs/CLIENT_CONTRACT.md` değişmedi mi?
- **VR2 — Hatayı kendin üret ve kapandığını gör.**
  - `v0.3.2` ve `v0.3.3` sürümlerini kur.
  - Kendi betiğinle, kendi Mac'inde ve Docker'da (Linux):
    - SIGKILL, kaldırma beklemesi içinde (silme ve taşıma): 0.3.2'de kalıcı, 0.3.3'te yeniden başlatmadan sonra kapalı;
    - Claude Code'un dizisi SIGINT → SIGTERM → ~0,5 s → SIGKILL;
    - sunucu kapalıyken düzenle, sil, ekle: 0.3.2'de alınmıyor, 0.3.3'te alınıyor;
    - değişiklik yokken yeniden başlatma: yeni olay 0, günlük bayt eşit;
    - `ingest-dir`: silme alınıyor; alt dizin kökünde dışarıdaki dokümanlar aynen.
    Her durum en az 5 tekrar.
  - `tools/list` (MCP) 0.3.2 ile bayt eşit mi?
- **VR3 — Karşılaştırma ve kademe 2.**
  - D-NOTES senaryosunu P2-Q'nün sürücüsüyle 0.3.3 ile kendi Mac'inde ve Docker'ında koş: turun Mac, Docker ve CI kayıtlarıyla ve 0.3.2 kayıtlarıyla bayt eşit mi (ya da yalnız R0'da adlı farklar)?
  - Kademe 2'yi kendin tetikle (`gh workflow run verify.yml --ref main -f sha=<doğrulanan commit>`): D farkı 0, sayılan FAIL 0 olmalı. Değişen kayıtlar R0'ın önceden adlandırdıklarıyla aynı mı? E1-4/R14'ün etikete bağlılığını hesaba kat.
- **VR4 — `check_deps`, belgeler, karar kuralı.**
  - R3'ün kök neden açıklamasını (ya da hipotez listesini) kendi macOS'unda sına; `make lint`'i temiz klonda iki kez koş.
  - Kılavuzun yeni metnini izle: sunucu kapalıyken bir notu sil, sunucuyu başlat, kılavuzun söylediği oluyor mu?
  - Açık Dependabot ve CodeQL uyarıları 0 mı, değilse gerekçeli mi? 0.3.3 sürümünün `SHA256SUMS`'ını ve köken tasdiklerini denetle.
  - Karar kuralını GOAL'den türet, FINAL ile karşılaştır.
  - Vaat sorusu (NORTH_STAR): "dosyalar gerçek kaynaktır" sözü artık sunucu kapalıyken ve öldürülünce yapılan değişikliklerde de tutuyor mu?
- **Sonuç:** ihlal/fark/şüphe; Q3-1..Q3-10 için geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
