# Yol Haritası v2 — Faz 1 Retrospektifi ve Faz 2

Bu belge `docs/ROADMAP.md`'nin yerine geçer (v1 `docs/archive/ROADMAP_v1.md` olarak saklanır). Faz 1 kapanış kararları `DECISIONS.md` K-E1.x.

## 1. Faz 1 bilançosu (2026-10-13 → 2026-10-17)

### Ne üretildi
- `rootmark 0.1.0` (Apple Silicon): Rust çekirdek + indeks + sunucu; spec 1.0.3, uyumluluk Seviye 3 (34 test); MCP (10 araç) ve HTTP (token, insan kanalı); minimal Obsidian istemcisi; sahne uçtan uca ve gerçek Obsidian'da bağımsız doğrulanmış.
- Yayımlanabilir spesifikasyon; blok düzeyinde soy, köken, katman/terfi, kontrol noktası, tüketim izi, deterministik indeks; determinizm iki bağımsız klonda 100 bin+ alanda sıfır fark.
- Üç prototip turu + sekiz geliştirme turu; her tur bağımsız doğrulandı; ihlal sayısı toplam 0, GOAL dışı bulgu her turda en az 1.

### Ne öğrendik (süreç)
1. **Önceden kayıtlı eşik ve karar kuralı** tartışmayı sonuçtan koparıyor; "kaldı" dalları en az "geçti" kadar değerli (H2 düştü, C-3 düştü, E-7 düştü; hepsi doğru karar üretti).
2. **Doğrulayıcı her turda GOAL'ün ölçmediği bir şey buldu** (lineage ULID, F1 karışık vault, canonical silme, boru hattı, `status`). Kriterler davranışı ölçüyor, doğrulayıcı niyeti ölçüyor. Bu yüzden "şu koşullarla güvenilir" hükmü ve ADDENDUM/kısa yama turu mekanizması kalıcı.
3. **Ölçüm kodu ürün kodu kadar hata yapar** (ULID, `report.py` geri düşme, `compare` belirlenimsiz alan, SIGPIPE). Belirlenimli alan listesi ve dondurma denetleyicisi politikası bu yüzden var.
4. **Sonuç görüldükten sonra alınan kararlar meşru, ama işaretli olmalı** (K4, K-C1.2, hariç tutma listeleri). Gizlemek yerine yazmak güveni artırdı.
5. **"Önce Python sonra Rust" tuzağı** bir kez kuruldu (MCP) ve geri çekildi; faz 1 sonunda çalışma zamanında Python yok.

### Ne öğrendik (ürün)
- Hibrit (sözcüksel + vektör) arama taban; wikilink grafı sıralama sinyali olarak katkı vermedi. Graf, varsa, gezinme yapısıdır.
- Yönetişimin sınırı kanal değil **dosya sistemidir** (P1-E doğrulaması, risk 1). Dosya erişimli ajan md'yi doğrudan yazarsa izleyici onu insan sayar; token'ı okursa insan olarak terfi ettirir. Vaat, hedef kullanıcının en yaygın kurulumunda (Claude Code + yerel vault) tam değil. **Faz 2'nin ilk işi bu.**
- Biçim göçleri ve bayat durumlar (indeks `/2→/3`, `status`) sync'ten önce altyapı istiyor.
- İstemci blok hizalaması tahminle çalışıyor; sözleşmede blok konumu yok.

## 2. Faz 2 — sıra ve gerekçe

| Tur | Ad | Tür | İçerik | Çıkış kriteri (özet; GOAL'de önceden kayıtlı) |
|---|---|---|---|---|
| **P2-0** | Tasdikli yazma | **Hipotez turu** (prototip disiplini: eşik, D-set, bağımsız doğrulama) | İnsan kanalı istemcisi her kaydı `(doc_path, content_hash)` ile sunucuya **tasdik eder**; izleyicinin yakaladığı tasdiksiz değişiklik yeni katman `unverified` alır (canonical değil, aramada proposal gibi); insan tek tıkla terfi ettirir; politika ile `unverified→canonical` otomatik (eski davranış) seçilebilir. Token'ın ajan tarafından okunması riskine karşı: tasdik, istemcinin kullanıcı oturumuna bağlı kısa ömürlü anahtarla (OS anahtar zinciri varsa oradan) imzalanır; dosyadaki token yalnız okuma kanalı olur. | H-P2-0a: Claude Code'un doğrudan md yazdığı 200 vakada `canonical` sızıntısı 0; H-P2-0b: Obsidian ile insan düzenlemelerinde yanlış `unverified` ≤ %1 (tasdik gecikmesi/yarış); H-P2-0c: tasdik gecikmesi p95 < 500 ms; eski vault'lar açılır, `events_hash` değişmez; spec §7 "layers" genişlemesi CRITERION ile. |
| **P2-0.1** | Tasdik yolları (yama) | **Yama turu** (P2-0 kaldı: K-P2-0.6) | Yeniden adlandırma/bağlantı güncellemesi tasdiki (aklama korumalı), çakışmada blok düzeyi eşleşme, Obsidian dışı insan (CLI) ve ajan yolları, tehdit modeli (CDP yolu). | P2-0'ın eşikleri aynen (0b ≤ 0.01, 0d ≥ 0.99, sızıntı 0) yeni D-B tohumuyla; Obsidian dışı kanallarda aynı eşikler; GOAL'de önceden kayıtlı. |
| **P2-0.2** | Bağlantı biçimleri ve katman taşıma (yama) | **Yama turu** (P2-0.1 doğrulaması "şu koşullarla": K-P2-01.6) | Markdown bağlantı etiketi ve kendine bağlantı dahil bağlantı denetimi, yeniden adlandırmada blok blok katman taşıma (yükseltme yok), D-C ölçüm determinizmi, "Validated" damgası doğrulamadan sonra, CRITERION bilinmeyen-alan kuralı. | P2-0'ın eşikleri aynen, karışık bağlantı biçimli yeni D-B tohumuyla; çift koşturma yüksüz ve yük altında eşit; GOAL'de önceden kayıtlı. |
| **P2-A** | Göç ve sürüm uyumu | Altyapı | İndeks/olay/checkpoint biçim sürümleri için göç çerçevesi (`rootmark migrate`, kuru koşturma, geri alma), `status` sürüm matrisi, tarihsel FAIL yerine "göç gerekli" durumu; Linux/Windows kurulum paketleri ve imzalama ("Setup"); Intel macOS yeniden değerlendirme. | Her biçim için ileri göç + geri alma testi; 0.1.0 vault'u → güncel sürüm göçü bayt bayt doğrulanabilir; kurulum paketleri 2 platform. |
| **P2-0.3** | Tasdikli yazma: yeniden ölçüm — **kapandı: 0.3b kaldı, varsayılan `canonical` kesin (K-P2-03.1)** | Hipotez turu (kod değişikliği yok) | (A) düzeltmesi sonrası P2-0 hipotezi yeni tohumla, yüksüz ve yüklü çiftte; geçerse `unverified` yeniden varsayılan, spec damgası, NORTH_STAR notu. | P2-0 eşikleri aynen (0b ≤ 0.01 resmi zincir; yük altında ≤ 0.02); GOAL'de önceden kayıtlı. |
| **P2-L** | Dil: Türkçe ve çok dilli indeks; 0.2.0 yayımı — **kapandı: L-2/L-6 eşik altı, `uni_fold_tr` ve multilingual opt-in, 0.2.0 yayımlandı (K-P2L.3)** | **Blok 1** geliştirme (K-P2.4) | Unicode farkında, Türkçe katlamalı sözcüksel varyant (manifest'te bildirilen); çok dilli embedder seçeneği; yol öneki arama filtresi; P2-A göçüyle `migration_required`; K-P2A.5 (3) küçük işler. (K-Q.1 madde 1, 3a) | Türkçe/TR-EN D-set'te nDCG mevcut varyanttan düşük değil, Türkçe sorgularda ≥ +0.10; İngilizce referans Δ 0; çift fark 0; C3 iki varyantla. |
| **P2-P** | Bildirimsel terfi politikası — **kapandı: hepsi geçti; Blok 1 doğrulaması "güvenilir", §7.1 Validated (K-P2P.6)** | **Blok 1** geliştirme; blok sonu doğrulama | `policy.toml` kural listesi: kaynak/aktör + yol önekiyle `proposal → canonical`; olay kural kimliği ve politika sürümü taşır; geri alınabilir; `unattested_fs` asla terfi etmez; başsız insan kanalı (bot) tehdit modeli ve aktör/kanal etiketi. (K-Q.1 madde 2, 7) | Kural testleri; güvenlik vakaları (tasdiksiz yazım terfi etmez, kural dışı yol terfi etmez); sahne: toplayıcı ajan → otomatik canonical → `trace`. |
| **P2-B** | İstemci sözleşmesi v2 — **kapandı (K-P2B.5; `reports/rounds/P2-B/FINAL.md`: B-3 kaldı 0.00551, geri kalanı geçti; `loc` deneysel, K-P2B-T1 (a)+(c))** | **Blok 2** geliştirme | Blok konumu (satır aralığı, kaynak ofset) sözleşmeye; hizalama ölçümü (gerçek kullanıcı notları korpusu: HTML, dipnot, callout, gömme); `include_proposal`/`unverified` görünümü; VS Code istemcisi iskeleti. | Hizalama hatası ≤ %0.5 korpusta; sözleşme dışı çağrı 0; iki istemci aynı sözleşmeyle. |
| **P2-R** | Saklama ve budama — **kaldı, karar kuralı (R-1 (b) lafzen, R-6 lafzen, R-10; `reports/rounds/P2-R/FINAL.md`); kararlar K-P2R.4 ve K-P2R.5: yeniden koşu yok (CI çifti de ölçüm tasarımından kırmızı), R-6 budama kayıtlarının eşitliğiyle karşılandı, R-10 P2-V'de kapanır; Blok 2 doğrulaması dar kapsamla** | **Blok 2** geliştirme; blok sonu doğrulama | Proposal sönümleme/budama (K19-T), `traces.jsonl` budaması (§12), checkpoint sıklığı; ölçek sınırı değişmez. (K-Q.1 madde 5) | Budama determinizmi; budanan içerik checkpoint'ten okunur; `events_hash` korunur; önceden kayıtlı hacim eşikleri. |
| **P2-V** | Doğrulama altyapısı: kademeli zincir (K-P2V.1) — **kaldı, karar kuralı (V-3, V-4, V-6, V-7; `reports/rounds/P2-V/FINAL.md`); doğrulama "şu koşullarla güvenilir"; K-P2V.3: T1 (a)+(c), T2 (c)+(b), T3 (a), T4 → P2-V.1; kademeler P2-V.1 ile yürürlüğe girer** | Altyapı (ürün kodu değişmez); kısa bağımsız doğrulama | Zincir envanteri ve D/Z/O/H sınıflandırması (ön-kayıt); kademe 1 FAST CI'da PR'larda; kademe 2 tam çift yalnız CI'da, A/B paralel; zamanlamaya duyarlı ve Obsidian kümeleri Mac kümesinde kendi eşikleriyle; kayıtlarda host yolu yasak; doğrulayıcı tam çifti CI'dan alır. | Hiçbir kriter kaybolmaz, D alanları eski↔yeni eşit; CI tam çifti iki tetiklemede yeşil ≤ 2 sa 30 dk; Mac kümesi ≤ 30 dk; FAST CI'da ≤ 30 dk. |
| **P2-V.1** | Gömme determinizmi ve kademelerin tamamlanması (K-P2V.3) | **Yama turu** (P2-V kaldı; doğrulamanın iki koşulu) | Gömme vektörlerini işlemciden bağımsız kılmak (iki işlemcide fizibilite, seçim, `index/` değişikliği, indeks göçü); üç koşucuda bayt eşitliği kademe 1'de; P2-V gözden geçiricisinin araç bulguları (B1–B3, B6, R14), CPU modeli köprüsü, karşılaştırıcı hızı; Mac kümesinde tekrarsız deterministik kriterler ve tarihsel etiket. | Vektör baytları x86_64/aarch64 Linux ve macOS arm64'te eşit; kalite kriterleri kendi eşiklerinde; göç taze kurulumla bayt eşit; hiçbir kriter kaybolmaz; kademe 2 iki tetiklemede yeşil ≤ 3 sa 15 dk; FAST CI ≤ 40 dk. |
| **P2-C** | Sync | Altyapı + hipotez | Blok içi CRDT (Yjs/Automerge eşdeğeri Rust), bloklar arası nesne alışverişi, türev indeksler senkronize edilmez; iki replika determinizmi; çakışma çözümünde soy korunumu. | İki replika aynı olay dizisine yakınsar; çakışmalı düzenlemelerde soy koruma ≥ 0.95; `events_hash` replika başına deterministik. |
| **P2-D** | WASM politikalar | Plugin | `EditPolicy`/`PromotionPolicy`/`VersionTrigger` için `wasmtime` çalışma zamanı, kaynak sınırları, örtük onay politikası ilk örnek. | Sandbox kaçışı testi; politika değişimi olaylarda izli; örtük onay politikası sahnede. |
| **P2-E** | Çok vault ve sunucu dağıtımı | Altyapı | Tenant = vault + anahtar + politika; uzak erişim (TLS), dinlenmede şifreleme; istemci başına kimlik doğrulamalı ajan kimliği (kural listesinin varsayılan önerilmesinin önkoşulu, K-P2P.6); tek motor çok replika. | Tenant izolasyonu kasıtlı test; uzak sahne. |

P2-0 bloke edicidir: tasdikli yazma hipotezi düşerse faz 2'nin geri kalanı yeniden sıralanır ve tehdit modeli "yalnız kum havuzlu ajan" olarak daraltılır; bu durumda ürün vaadi de daraltılır ve NORTH_STAR karar kaydıyla değişir.

## 3. Araştırma kolları (paralel, `research/`, önceden kayıtlı eşik)
| Kol | Hipotez | Durum |
|---|---|---|
| R-1 Varlık grafı (D-set adayı: qorbit Türkçe finans korpusu, K-Q.1; P2-L'den sonra) | LLM çıkarımlı varlık/ilişki grafı, hibrit üstüne takma ad / dil geçişi / ilişki sorularında kazanç | başlamadı |
| R-2 Akış durumu | niyet durumu (üstel ortalama + histerezis) kayan pencereye göre daha az yanlış alarm | başlamadı |
| R-3 Soy sınırı | >%30 yeniden yazımda "soy koptu olabilir" skoru | başlamadı |
| R-4 Öğrenilen ağırlıklar | kullanım sinyaliyle sıralama ağırlıkları, keşif payıyla | başlamadı |
| R-5 Graf gezinme | ajanın link üzerinden ikinci sorgu attığı çok-adımlı protokol vs tek sorgu | yeni (H2'nin alternatif hipotezi) |

## 4. Süreç değişiklikleri (DEV_PROCESS'e işlenir)
- Her turun VERIFY'ı "GOAL'ün ölçmediği ama NORTH_STAR vaadini ilgilendiren bir şey gördün mü" sorusunu açıkça içerir (zaten fiilen yapılıyor; yazılı hale gelir).
- Hipotez turları (P2-0, P2-C) prototip şablonunu (D-set, eşik, karar kuralı, çift koşturma) kullanır; geliştirme turları mevcut şablonu.
- Yayım etiketi her zaman insan adımı; `release.yml` yalnız `workflow_dispatch`.
- **Doğrulama zinciri kademelidir (P2-V, K-P2V.1; DEV_PROCESS §3, §5; yürürlüğü P2-V.1'in karar kuralına bağlı, K-P2V.3).** Kademe 1 (`make verify FAST=1`) her PR'da CI'da; kademe 2 (tam çift) yalnız CI'da (`verify.yml`, iki `ubuntu-latest` koşucusu, haftalık ve `workflow_dispatch -f sha=`), hüküm D sınıfında, Z/O alanları ön-kayıtlı envanterle (`reports/rounds/P2-V/inventory.json`) adıyla dışarıda; Mac kümesi (`make verify-mac`: Z/O kümeleri 3 tekrar, kendi eşikleriyle) blok sonunda ve Z/O kümesine dokunan turlarda. Tam çift geliştiricinin makinesinde koşulmaz; doğrulayıcı tam çifti kendi CI tetiklemesiyle alır ve karşılaştırmayı kendisi yeniden yapar. Zincire kayıt ekleyen tur, kaydın sınıfını ve Z/O ise eşiğini GOAL'ünde ön-kayıt olarak yazar.

## 5. Değişiklik politikası
Yol haritası değişebilir; değişiklik `DECISIONS.md`'ye gerekçe ve etkilenen turlarla. Tur başladıktan sonra GOAL değişmez; ADDENDUM yalnız tur başlamadan önce.
