# P2-B — Bağımsız Doğrulama (Blok 2 sonunda, P2-R ile birlikte; K-P2.4)

Bu turun bağımsız doğrulaması tek başına yapılmaz. Blok 2 (P2-B + P2-R) bitince tek bir dar doğrulama koşulur ve `reports/rounds/P2-R/VERIFICATION.md`'ye yazılır; P2-B bölümü ayrı başlıkla. P2-B için doğrulayıcının bakacağı maddeler:

- **VB1 — Tam çift.** `make verify`, bir kez: `loc`, `events`, zaman filtresi ve hizalama değerleri fark 0.
- **VB2 — Hizalama ölçümü:**
  - D-NOTES'un kaynakları, revizyonları, lisansları ve sha256'ları.
  - Seçim tohumu ve ölçüm betiği ölçümden önce commit'li.
  - Hizalama hatasını kendi betiğinle yeniden hesapla: dilim → yeniden ayrıştır → `normalized_text`, örtü.
  - B-3, B-4 eşik dalları.
- **VB3 — Kendi kenar vakaların:**
  - İç içe callout.
  - HTML içinde boş satır.
  - Dipnot tanımı belge sonunda.
  - CRLF + BOM.
  - Ön-bilgi sonrası boş belge.
  - Türkçe ve emoji içeren başlık.
  - Tek satırda iki gömme.

  `loc` dilimi bloğu veriyor mu, `line_*` ile `byte_*` tutarlı mı?
- **VB4 — Bayat dosya.** Dosyayı alınmadan değiştir; `loc_stale: true`. Sunucu eski ofseti yeni dosyaya uygulayan bir yanıt vermiyor. İstemci panelleri satır tahmin etmiyor.
- **VB5 — `events`:**
  - Yanıt `events.jsonl` satırlarıyla bayt eşit.
  - Filtre yalnız süzüyor.
  - Vault dışı `path_prefix` reddediliyor.
  - Çağrı sonrası `events_hash` ve olay dosyası aynen.
  - Büyük `limit` üst sınıra kırpılıyor ya da reddediliyor; sessiz değil.
- **VB6 — Zaman filtresi.** Yol ve ön-bilgi kaynakları çalışıyor. Dosya zamanını (mtime) değiştirmek sonucu değiştirmiyor. Filtresiz `query_hash` önceki RUN ile eşit. `trace` filtreyi taşıyor.
- **VB7 — İki istemci ve sözleşme:**
  - Sözleşme denetleyicisini kendi eklediğin bir sözleşme dışı çağrıyla dene; yakalıyor mu?
  - İki istemci aynı `contract.gen.ts`'i kullanıyor mu?
  - VS Code iskeleti token'ı nerede tutuyor (THREAT_MODEL)?
  - Kaydetmelerin tasdik edilmediği belgeli mi?
- **VB8 — Süreç:**
  - `check_frozen`, `check_deps`, `check_dep_age`.
  - `ROUND_START` birleşmiş `main` commit'i.
  - Karar kuralını GOAL'den türet, FINAL ile karşılaştır.
  - Dondurma istisnaları önce/sonra kanıtlı.
  - Blok 1 doğrulamasının önerileri 1 ve 3 uygulanmış.
- **VB9 — Vaat sorusu.** Konum ve olay akışı NORTH_STAR'ın "her bloğun kim tarafından… yazıldığı bilinir" ve "dosyalar sıradan markdown olarak kalır" vaatlerini etkiliyor mu? md'ye bir şey yazılıyor mu? Bir istemcinin konumla yaptığı düzenleme yönetişim yolunun dışına çıkıyor mu?
