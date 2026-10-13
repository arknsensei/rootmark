# Yol Haritası

Her tur bir `/goal` koşturmasıdır: kendi yönerge klasörü (`docs/rounds/<id>/`), kabul ölçütleri, bağımsız doğrulama ve çıkış kriteri ile. Turlar sıralıdır; araştırma turları paraleldir ve hiçbir faz turunu bloke etmez.

## Faz 1 — Çekirdek ve ilk ürün sahnesi

| Tur | Ad | İçerik | Çıkış kriteri |
|---|---|---|---|
| **P1-A** | Yeniden yapılandırma ve dondurma | Monorepo iskeleti; prototip `core/` ve `research/` taşınır; `spec/` §1–6 fixture'ları ve C1 uyumluluk testleri; `make conformance`; `make verify` uyarlanır; doğrulayıcının üç önerisi (holdout sentetik set, bölme eşiği zorluk bölgesi, dondurma denetleyicisi politikası) kalıcı regresyon olarak | C1-01..C1-17 geçer; holdout sette H1 eşikleri geçer; çift koşturma determinizmi |
| **P1-B** | Yönetişim primitifleri | `core`: katman, terfi/demote hook'u, kontrol noktası, tüketim izi, ham günlük/sürüm tetikleyici sınırları, `policy.toml` yükleyici; C2 testleri | C2-01..C2-08 geçer; replay katman geçişlerini kurar |
| **P1-C** | İndeks | `index`: tantivy, vektör deposu + kesin kosinüs, embedder trait + ONNX MiniLM, RRF, kontrol noktasına göre sorgu, sıcak katman + sıkıştırma; C3 testleri; `research` ile kalite regresyonu (v2 hybrid sayıları ±0) | C3-01..C3-04; `hybrid` nDCG v2 ile eşit; yazma→aranabilir p95 < 100 ms |
| **P1-D** | Sunucu | `server`: `mdk` CLI, MCP araçları (`search`, `read_block`, `history`, `blame`, `propose`, `promote`, `checkpoint`, `trace`), HTTP eşdeğeri, dosya izleyici, varsayılan politika (yalnız insan onayı) | MCP uyumluluk testi (resmi inspector); her araç için betikli uçtan uca test |
| **P1-E** | İstemci ve ürün sahnesi | `clients/obsidian` minimal (blame paneli, proposal işaretleme, promote, checkpoint seçici); `scripts/scene.sh`: Claude Code → propose → blame → arama dışı → promote → trace → checkpoint geri dönüş; bağımsız doğrulayıcı sahneyi koşturur | Sahne betiği temiz ortamda geçer; doğrulama raporu "güvenilir"; `mdk` 0.1.0 yayımı |

Faz 1 çıkışı: `NORTH_STAR.md` ürün sahnesi uçtan uca; `spec` §7–11 "donmuş"a terfi eder; uyumluluk Seviye 3.

## Faz 2 — Ekip ve genişleme
| Tur | İçerik |
|---|---|
| P2-A | Blok içi CRDT, bloklar arası nesne alışverişi (sync), tenant = vault + anahtar + politika |
| P2-B | WASM plugin çalışma zamanı (politika ve zenginleştirici), kaynak sınırları |
| P2-C | Node bağlaması, VS Code istemcisi, sürekli sorgu primitifi (akış) |
| P2-D | Sunucu dağıtımı, dinlenmede şifreleme, çok tenant |

## Araştırma kolları (paralel, `research/`, önceden kayıtlı eşiklerle)
| Kol | Hipotez | Veri |
|---|---|---|
| R-1 Varlık grafı | LLM ile çıkarılmış varlık/ilişki grafı, hibrit arama üstüne takma ad / dil geçişi / ilişki sorularında kazanç sağlar | önbellekli Haiku çıkarımı (git'te), içeriksiz pasajları eleyen soru üreteci, ikinci vault |
| R-2 Akış durumu | Üstel ortalama + histerezisli niyet durumu, kayan pencere tabanına göre niyet değişimini daha az yanlış alarmla yakalar | vault'tan sentetik gezinen transkriptler |
| R-3 Soy sınırı | Aşırı yeniden yazımda (>%30) "soy koptu olabilir" sinyali için güvenilir bir skor | D1-hard genişletilmiş |
| R-4 Öğrenilen ağırlıklar | Kullanım sinyaliyle güncellenen sıralama ağırlıkları, keşif payıyla popülerlik yanlılığına girmeden kazanç sağlar | sentetik oturum günlükleri |

Araştırma sonucu ancak önceden kayıtlı eşiği geçer ve bağımsız doğrulanırsa `spec`'e ve çekirdeğe önerilir; öneri bir karar kaydıyla girer.

## Değişiklik politikası
Yol haritası değişebilir; değişiklik `docs/DECISIONS.md`'ye tarih, gerekçe ve etkilenen turlarla yazılır. Tur içeriği, tur başladıktan sonra değişmez; yeni ihtiyaç bir sonraki tura gider.
