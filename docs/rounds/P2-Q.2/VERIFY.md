# P2-Q.2 — Bağımsız Doğrulama (kısa; yama turu)

**Çalışma koşulları:**
- Ayrı oturum, temiz klon (kendi scratchpad'in; önce `make setup`, `make models`). Ana çalışma dizinine erişim yok; kod değiştirilmez.
- Hedef ≤ 2 saat.
- Kademe 2'yi bir kez kendin tetikle; Mac kümesini koşma, turun kaydını oku.
- Kişisel içerik okunmaz; vault olarak D-NOTES kopyası kullanılır.

- **VR1 — Tasarım ve sınır.**
  - `DESIGN.md` ve `TESTS.md`'nin R1/R2'nin kodundan önce commit'li olduğunu `git log` ile göster.
  - Kök neden açıklamasını rmcp'nin kaynağıyla (kullanılan ve önceki sürüm) karşılaştır. Seçilen yöntem bir fork ya da yamalı kopya mı? `rmcp` sürümü ≥ 14 günlük mü (`check_dep_age`)?
  - `git diff ROUND_START..<sha>`: ürün değişikliği yalnız `mcp.rs`, izleyici, `Cargo.toml`/`Cargo.lock` mu?
- **VR2 — Hatayı kendin üret ve kapandığını gör.**
  - `v0.3.1` ve `v0.3.2` sürümlerini kur.
  - Kendi oturumundaki güncel Claude Code ile başsız (`claude -p --mcp-config … --strict-mcp-config`):
    - 0.3.1'de araç listesi alınamamalı (PQ1-1'i yeniden üret);
    - 0.3.2'de 11 araç listelenmeli, `search` ve `propose` başarılı olmalı, `propose` olay günlüğünde ajan aktörüyle görünmeli.
  - Kendi küçük JSON-RPC betiğinle eski bir istemciyi taklit et (`2025-06-18`, `_meta`'sız): 0.3.2 çalışıyor mu? `tools/list` çıktısı 0.3.1'inkiyle bayt eşit mi?
- **VR3 — PQ1-4, karşılaştırma ve kademe 2.**
  - PQ1-4'ün yeni testini kendi klonunda en az 20 tekrar koş.
  - D-NOTES senaryosunu P2-Q'nün sürücüsüyle 0.3.2 ile kendi Mac'inde ve Docker'ında koş: turun Mac, Docker ve CI kayıtlarıyla ve 0.3.1 kayıtlarıyla bayt eşit mi?
  - Kademe 2'yi kendin tetikle (`gh workflow run verify.yml --ref main -f sha=<doğrulanan commit>`): D farkı 0, sayılan FAIL 0 olmalı. Değişen kayıtlar R0'ın önceden adlandırdıklarıyla aynı mı? Kendi betiğinle karşılaştır; E1-4/R14'ün etikete bağlılığını hesaba kat.
- **VR4 — Belgeler, güvenlik, karar kuralı.**
  - Kılavuz §4.1'i güncel Claude Code ile izle.
  - Açık Dependabot ve CodeQL uyarıları 0 mı, değilse gerekçeli mi?
  - 0.3.2 sürümünün `SHA256SUMS`'ını ve köken tasdiklerini denetle.
  - Karar kuralını GOAL'den türet, FINAL ile karşılaştır.
  - Vaat sorusu: ajan yolu (Claude Code `propose` → insan onayı) şimdi kılavuzu izleyen yeni bir kullanıcı için baştan sona çalışıyor mu?
- **Sonuç:** ihlal/fark/şüphe; Q2-1..Q2-9 için geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
