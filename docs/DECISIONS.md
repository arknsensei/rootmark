# Karar Kayıtları

Biçim: `K<n> · tarih · başlık` → karar, gerekçe, etkilenen belgeler, "sonuç görüldükten sonra" işareti varsa.

- **K1 · 2026-10-13 · ContextNest ilişkisi.** Kavram seti benimsenir, implementasyon değil; blok düzeyi genişleme. Etki: spec §0, ARCHITECTURE §2.
- **K2 · 2026-10-13 · Sözcüksel indeks birinci sınıf.** v1/v2'de `hybrid` her hücrede en iyi. Etki: spec §9, PACKAGES `index`.
- **K3 · 2026-10-13 · Graf sıralama çekirdeğe girmez.** v2 H2 desteklenmedi. Etki: spec §9, ROADMAP R-1.
- **K4 · 2026-10-13 · Baskın parça soy sürekliliği.** Bölme/birleştirmede baskın parça soyu korur. **Sonuç görüldükten sonra alınmış karar** (v2 S2 sonrası); v3'te ölçüldü ve v2 davranışı bayrakla korundu. Etki: spec §5.
- **K5 · 2026-10-13 · Gizlilik sınırı.** Yerel vault + isteğe bağlı sunucu pipeline; zero-knowledge sunucu vaat edilmez. Etki: ARCHITECTURE §4.5.
- **K6 · 2026-10-13 · Rust çekirdek kalıcı.** Prototip crate atılmaz. Etki: PACKAGES.
- **K7 · 2026-10-13 · LLM çıkarımı yalnız önbellekli.** Determinizm için. Etki: spec §10, ROADMAP R-1.
- **K8 · 2026-10-13 · Yankı kontrolü katman modeliyle.** Ajan yazımı varsayılan `proposal`. Etki: spec §7.
- **K9 · 2026-10-13 · Çalışma zamanı tamamen Rust.** MCP sunucusu dahil. Önceki "önce Python" önerisi geri çekildi. Etki: PACKAGES, ROADMAP P1-D.
- **K10 · 2026-10-13 · Tek monorepo, yedi paket.** Etki: PACKAGES.
- **K11 · 2026-10-13 · Spesifikasyon yayımlanır.** Apache-2.0, İngilizce. Etki: spec/.
- **K12 · 2026-10-13 · Faz 1 varsayılan terfi politikası yalnız insan onayı.** Örtük onay (düzenleme, referans) bir politika plugin'i olarak faz 2'de. Etki: spec §7, ROADMAP P1-D.
- **K13 · 2026-10-13 · Bu sohbet geçmişi arşivdir.** Tek gerçek kaynak bu belge setidir; yeni oturumlar yalnız buradan beslenir.
- **K14 · 2026-10-14 · P1-A sonucu.** A-1..A-6 geçti, bağımsız doğrulandı; spec §1–6 "frozen" teyit. Etki: spec, ROADMAP.
- **K15 · 2026-10-14 · P1-A doğrulama önerileri.** Ö1 (FROZEN.md kendi listesinde, git-geçmişi denetimi) ve Ö2 (fixture kökü çalışma anında) P1-B'de kapatılır; Ö3 (bölme-ağırlıklı holdout) ek dilim olarak eklenir, mevcut holdout regresyon olarak kalır. Etki: P1-B GOAL.
- **K16 · 2026-10-14 · Faz 1 terfi politikaları.** Çekirdekte yalnız `Deny` ve `HumanOnly`; örtük onay faz 2 politika plugin'i. Etki: spec §7, P1-B.

## Karar talepleri (açık)
- **K14-T · 2026-10-13 · Eşleyici parametrelerinin olay `meta`'sına yazılması (P1-A/A2'de tespit).** spec §5: "Implementations MAY use other values but MUST record `matcher_version` and parameters in `meta` of every event". v3-final çekirdeği (ve dolayısıyla C1 fixture'ları, mini_vault/D3 `events_hash`'leri) yalnız `meta.matcher_version` yazar. Parametre eklemek `events.jsonl` baytlarını ve A-5 hash'lerini değiştirir (P1-A kırmızı çizgisi). Seçenekler: (a) spec metnini "parameters MUST be recorded when they differ from the validated defaults of `matcher_version`" olarak netleştir (bayt uyumlu; varsayılan dışı parametre için yeni `meta.params` alanı); (b) spec sürüm artışı + yeni hash'ler. P1-A davranışı değiştirmez; karar P1-B öncesi. Etki: spec §5, core `lineage.rs`, CONFORMANCE C1-15.
- **K15-T · 2026-10-13 · Tür adlarının yazımı (P1-A/A2 gözden geçirmesinde tespit).** spec §2.2 blok türlerini (`heading`, `list_item`, …), §4.3 `author_kind`'ı (`human`, `agent`, …) küçük harfle yazar; v3-final çekirdeği `"Heading"`, `"ListItem"`, `"Human"` yazar ve `version_id = H(block_type + "\n" + normalized_text)` bu PascalCase dizgiyi hash'ler. Yazımı değiştirmek tüm `version_id`/`lineage_id`/`events_hash` değerlerini değiştirir (P1-A kırmızı çizgisi). Öneri: spec §3'e "block_type is hashed in its canonical serialized form: `Heading`, `Paragraph`, `ListItem`, `Code`, `Table`, `Blockquote`, `Other`" ve §4.3'e serileştirme biçimi notu (spec sürüm notu, davranış değişmez). Yeni tipler (`Layer`) spec §7 ile uyumlu küçük harf kullanır; P1-B'de tutarlılık kararı. Ayrıca spec §8 `returned` demet gösterimi ↔ `TraceEntry` nesnesi: P1-B'de karar. Etki: spec §2–4, §7–8; C1 fixture'ları v3-final yazımını kayıt altına alır.
