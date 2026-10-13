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
