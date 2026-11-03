# P2-Q — Bağımsız Doğrulama (kısa; pilot turu)

Ayrı oturum, temiz klon (kendi scratchpad'in), ana çalışma dizinine erişim yok; kod değiştirilmez. Hedef ≤ 2 saat. Kademe 2 tetiklenmez; Mac kümesi koşulmaz. Kişisel vault'un içeriği okunmaz (yalnız kendi kopyanın sayıları).

- **VQ1 — Yayım.** `v0.3.0` sürümünün arşivlerini indir, `SHA256SUMS`'ı ve köken tasdiklerini (`gh attestation verify`) denetle; `install.sh` ile kendi önekine kur; `--version`, `rootmark conformance` 39/39. `Q1_verifications.md`'deki her koşulun kapatan kararını DECISIONS'ta bul (R16).
- **VQ2 — Senaryoyu kendin koş.** Ön-kaydın (`SCENARIO.md`, `scenario.json`) Q2'den önce commit'li olduğunu `git log` ile; qorbit md'sinin **kendi kopyanla** (aynı kaynak commit'i, `progress/Q0.md`'deki blake2b'lerle karşılaştır) Q2 (b)'nin senaryosunu kurulu 0.3.0 ile koş: adım sonuçları, sızıntı 0, `events_hash` ve indeks dosyaları turun Mac kaydıyla bayt eşit mi.
- **VQ3 — Kasıtlı bozmalar.** (a) Ajanın doğrudan dosya yazımını `canonical` gösteren bir kaydı sürücüye ver → Q-5 kalmalı; (b) kayıtlara kopyadan bir cümle ekle → içerik sızıntısı denetimi bulmalı; (c) bir sorgunun beklenen kümesini değiştir → değerlendirici farkı yazmalı.
- **VQ4 — Sınır, bulgular, karar kuralı.** `git diff ROUND_START..<sha>` ürün dizinlerinde yalnız sürüm dizgileri; `FINDINGS.md`'nin her bulgusu sınıflı ve hedefli; `numbers.json`'ın sayıları kayıtlardan; Q3 koştuysa servis/port blake2b'leri öncesi = sonrası. Karar kuralını GOAL'den türet, FINAL ile karşılaştır; vaat sorusu: gerçek içerikte NORTH_STAR'ın vaadini ilgilendiren, GOAL'ün ölçmediği bir şey gördün mü.
- **Sonuç:** ihlal/fark/şüphe; Q-1..Q-9 için geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
