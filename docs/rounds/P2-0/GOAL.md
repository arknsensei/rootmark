# P2-0 — Tasdikli Yazma (Hipotez Turu)

Başlangıç: `v0.1.0` etiketi var. Bu tur faz 2'nin bloke edicisidir: hipotez düşerse ROADMAP §2 yeniden sıralanır ve NORTH_STAR vaadi karar kaydıyla daraltılır.

## Problem
Yönetişimin sınırı dosya sistemidir (P1-E doğrulaması, risk 1). Dosya erişimli bir ajan md'yi doğrudan yazarsa izleyici onu insan yazımı sayar; `server.token`'ı okursa HTTP'den insan olarak terfi ettirir. Hedef kullanıcının en yaygın kurulumu (Claude Code + yerel vault) tam bu durumdur.

## Hipotez
İnsan kanalı istemcisi her kaydı tasdik ederse ve tasdiksiz değişiklikler `canonical` yerine yeni bir `unverified` katmanı alırsa, dosya erişimli ajan canonical'a sızamaz **ve** gerçek insan düzenlemeleri pratikte hiç `unverified`'a düşmez.

## Tasarım (ölçülecek olan)
- **Katman:** `layer`'a `unverified` eklenir: aramada `proposal` gibi (varsayılanda yok, `include_proposal` ile etiketli), blame'de ayrı işaret, terfi insan kanalından tek tıkla (`promote` `from: unverified`). Spec §7 genişler; sürüm CRITERION ile.
- **Tasdik:** `POST /v1/attest {doc_path, content_hash: blake3(dosya baytları), saved_at_seq?}`; istemci her kayıtta gönderir. Sunucu tasdikleri yalnızca-ekleme `.rootmark/attestations.jsonl`'a yazar.
- **Eşleştirme:** izleyici bir değişiklik yakaladığında dosya hash'ini hesaplar; aynı `doc_path` için **W** saniye içinde (öncesi/sonrası) eşleşen tasdik varsa yeni bloklar `canonical`, yoksa `unverified`. Geç gelen tasdik (W içinde) `unverified`'ı `canonical`'a yükseltir (`LayerChanged`, `reason: late_attestation`). W varsayılan 2.0 s; ayar setinde `{0.5, 1, 2, 3, 5}` taranır.
- **Tasdik anahtarı, iki aşama:**
  - **Aşama 1 (bu tur zorunlu):** tasdik, bağlantıda token ile alınan kısa ömürlü oturum anahtarıyla (HMAC) imzalanır; token'ı okuyabilen ajan hâlâ tasdik üretebilir. Bu, **tasarımın doğrulanması** içindir, tehdidin kapanması değil.
  - **Aşama 2 (bu tur alt-hipotez, macOS):** oturum anahtarı OS anahtar zincirinden, yalnız Obsidian uygulamasına ACL'li bir gizle türetilir (`security` çerçevesi / `Security.framework`); terminal süreci (Claude Code) aynı gizi okuyamaz. Ölçülür; olmazsa raporlanır ve "aşama 1" tehdit modeli yazılı kalır.
- **Politika:** `[attestation] unattested = unverified|canonical` (varsayılan `unverified`); `canonical` seçen kullanıcı eski davranışa döner, `status` bunu açıkça gösterir. CLI: `rootmark attest <doc>` (token ile, vim/CLI kullanıcıları için), `rootmark attest --all` (toplu, mevcut vault'u canonical sayma).
- **Göç:** mevcut vault'larda tüm bloklar `canonical` kalır (geriye uyumluluk); yalnız yeni değişiklikler kurala tabi. `events_hash` değişmez.

## Veri setleri
- **D-A (ajan doğrudan yazma):** `scripts/agent_fs_writer.py`: 200 senaryo, gerçek Claude Code değil ama aynı yolla (dosya sistemi): yeni dosya, mevcut insan dokümanına ekleme, bölüm değiştirme, tam üzerine yazma, yeniden adlandırma, hızlı ardışık 5 yazma, `.rootmark/` dışı her yerde. Her senaryoda beklenen: değişen her blok `unverified`, canonical blok sayısı ve içeriği değişmez.
- **D-B (gerçek insan düzenleme):** gerçek Obsidian (CDP harness, P1-E.1 K-E1.3) ile 300 düzenleme oturumu: tek karakter, paragraf, çoklu dosya, hızlı art arda kayıt (otomatik kayıt 2 s), büyük yapıştırma, dosya yeniden adlandırma, uygulama odak kaybı sırasında kayıt. Ayar 100 / test 200 (W taraması ayarda).
- **D-C (karışık):** D-A ve D-B senaryoları aynı vault'ta eşzamanlı (ajan yazarken insan düzenliyor): her bloğun katmanı doğru kaynağa göre.
- **D-D (anahtar zinciri, macOS):** 20 deneme: Obsidian tasdik üretir; terminal süreci aynı gizi okumaya çalışır.
- **D-E (eski vault):** v0.1.0 ile üretilmiş 3 vault (sahne, e2e, karışık) yeni ikili ile açılır.

## Önceden kayıtlı eşikler
| Kod | Ölçüm | Küme | Eşik |
|---|---|---|---|
| H-P2-0a | Canonical sızıntısı: ajan yazımından sonra `canonical` olan yeni/değişen blok sayısı | D-A test | = 0 |
| H-P2-0a′ | Canonical içerik korunumu: ajan "üzerine yazma"dan sonra eski canonical sürümler `history`/checkpoint ile erişilebilir ve `blame` silen aktörü (`unattested_fs`) gösterir | D-A | 200/200 |
| H-P2-0b | Yanlış `unverified`: gerçek insan düzenlemesinde `unverified` kalan blok oranı (geç tasdik dahil, W sonunda) | D-B test | ≤ 0.01 |
| H-P2-0b′ | Geç tasdikle düzelen oran (bilgi) | D-B test | raporlanır |
| H-P2-0c | Tasdik gecikmesi: kayıt → `canonical` p95 | D-B test | < 500 ms |
| H-P2-0d | Karışık: katman doğruluğu | D-C | ≥ 0.99 |
| H-P2-0e | Anahtar zinciri: terminal sürecinin gizi okuyamadığı deneme oranı | D-D | = 20/20 (alt-hipotez; düşerse rapor, karar kuralını etkilemez) |
| H-P2-0f | Eski vault'lar: açılır, replay eşit, `events_hash` değişmez, tüm bloklar canonical | D-E | 3/3 |
| H-P2-0g | Determinizm: çift koşturma, `attestations.jsonl` ve `events.jsonl` bayt eşit (D-A, D-C) | — | eşit |
| H-P2-0h | Uyumluluk: C2-11 (unverified varsayılanı), C2-12 (tasdik eşleşmesi), C3-08 (aramada unverified) + mevcut 34 | — | 37/37 |

## Karar kuralı
- 0a ∧ 0a′ ∧ 0b ∧ 0c ∧ 0d ∧ 0f ∧ 0g ∧ 0h geçti → hipotez kanıtlandı; spec §7 "Validated"; `unverified` varsayılan olur; tehdit modeli güncellenir (0e sonucuna göre "aşama 1" ya da "aşama 2"); P2-A başlar.
- 0a veya 0a′ kaldı → tasarım kusuru; tur kaldı, karar talebi.
- 0b kaldı, 0a geçti → W ayarı ya da istemci tasdik akışı; bir yama turu; `unverified` varsayılan **olmaz**, politika `canonical` ile yayımlanır, NORTH_STAR vaadi "tasdikli istemci ile" ibaresiyle daraltılır.
- 0e kaldı → tehdit modeli "aşama 1" olarak kalır (token okuyan ajan tasdik üretebilir); vaat buna göre yazılır; P2-A başlar.
- Tanımlanmamış kalış → tur kaldı, karar talebi.

## Yapılmayacaklar
Kum havuzu; OS kullanıcı ayrımı; Windows/Linux anahtar zinciri (P2-A Setup'ta); sync; WASM; terfi politikası değişikliği; ajanın `attest` çağırabilmesi (MCP'de `attest` aracı **yoktur**).
