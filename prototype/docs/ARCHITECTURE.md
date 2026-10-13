# mdkernel — Mimari Modeli

Sürüm 1.0 · 2026-10-13 · Durum: **v3 ölçümüne kadar geçici; v3 geçerse §4 dondurulur**

Bu belge tartışmaz, karar verir. Gerekçeler kısa tutulmuştur; her kararın kanıt durumu §9'daki tabloda, karar listesi §10'dadır.

---

## 1. Tez

Markdown dosyaları, insanların ve ajanların birlikte yazdığı canlı bilginin tek gerçek kaynağıdır. Bu dosyaların üstünde **blok düzeyinde soy, köken ve deterministik türev indeks** sağlayan bir çekirdek, bugün hiçbir üründe olmayan ve pazarın açıkça ihtiyaç duyduğu katmandır. Vektör arama, graf, sync ve plugin'ler bu çekirdeğin üstüne kurulur; çekirdeğin özgün kısmı onlar değil, onların bağlandığı veri modelidir.

## 2. Konumlandırma

- **Değiliz:** Vektör veritabanı (Qdrant, pgvector), ajan hafıza API'si (Mem0, Zep, Letta), not uygulaması (Obsidian, Logseq), ekip wiki'si (Notion, Outline).
- **Yakın kuzen:** ContextNest (2026) — "getirme yönetişim değildir" tezini ve doküman düzeyinde hash-zincirli sürüm, yayın kapısı, deterministik seçim, tüketim izi mekanizmalarını tanımlar. Blok düzeyinde atıf, çok kullanıcılı düzenleme ve yerleşik anlamsal getirme bizzat "gelecek iş" olarak bırakılmıştır. mdkernel tam bu üç boşluğu hedefler.
- **ContextNest ile ilişki (karar K1):** Kavram seti ve terimler benimsenir (tipli doküman, taslak/yayın, kontrol noktası, tüketim izi). Spesifikasyonun blok düzeyine indirgenmiş bir genişlemesi olarak konumlanılır; implementasyon paylaşılmaz. Uyumlu dışa aktarım (ContextNest vault üretimi) ileride bir plugin'dir.
- **Hedef kullanıcı (öncelik sırası):** (1) ajan hafızasını md'de tutan geliştiriciler, (2) yönetişim kaygılı ekipler, (3) Obsidian güç kullanıcıları.
- **İlk ürün sahnesi:** Ajan bir vault'a yazar; kullanıcı her bloğun kimin tarafından ne zaman değiştirildiğini görür; onaylanmamış ajan blokları aramaya girmez; dünkü hale blok düzeyinde dönülebilir; hepsi sıradan md dosyaları üstünde, Obsidian'da açılır halde.

## 3. İlkeler

1. **Dosyalar gerçek, indeksler türev.** Her türev katman silinip yeniden üretilebilir; md dosyasına kimlik ya da meta veri yazılmaz.
2. **Determinizm.** Aynı girdi ve aynı komut, farklı makinede bit düzeyinde aynı soy kimliklerini, olay günlüğünü ve kalite metriklerini üretir. Rastgelelik taşıyan her bileşen (ANN indeksi, LLM çıkarımı) ya kesin alternatifle değiştirilir ya da içerik-adresli önbellekle sabitlenir.
3. **Köken birinci sınıf.** Her blok sürümü kimin (insan/ajan/transkript), hangi kaynaktan ve hangi olayla oluştuğunu taşır; sorgulanabilir, değiştirilemez.
4. **Çekirdek karar vermez, karar verilebilir kılar.** Terfi, budama, ağırlıklandırma gibi kararlar hook'larla plugin'e bırakılır; çekirdek yalnız standart ve kendini koruma kurallarını koyar.
5. **Öğrenme türev katmanda yaşar.** Link ağırlıkları, adaptörler, patika istatistikleri geri alınabilir sürümlerdir; gerçek kaynağa sessizce dokunulmaz.
6. **Altyapı dürüst olmayan sonucu üretemez.** Sessiz başarısızlık yerine hata; manifest doğrulaması; bağımsız çift koşturmayla doğrulama.

## 4. Katmanlar

```
L5  Sync ve tenant          (faz 2; CRDT blok içi, nesne alışverişi bloklar arası)
L4  Plugin'ler              (istemci eklentileri · pipeline zenginleştiriciler · karar/politika plugin'leri)
L3  Servis yüzeyi           (MCP sunucusu · CLI · sürekli sorgu)
L2  Türev indeksler         (sözcüksel · vektör · graf · varlık önbelleği · sıcak katman · öğrenilen ağırlıklar)
L1  Çekirdek (Rust)         (parser · blok/snapshot · soy · olay günlüğü · nesne deposu · chunker · köken · katmanlar · kontrol noktası · tüketim izi)
L0  Markdown dosyaları      (tek gerçek kaynak; Obsidian/git uyumlu)
```

### 4.1 L1 — Çekirdek (kanıtlanmış kısım `01_SPEC.md` §1–7)
- **Blok:** md'nin yapısal atomu; değişmez; `version_id` içerik-adresli; `lineage_id` deterministik türetilmiş. Kanıt: H1-S1/S4/R1/R2, H4.
- **Snapshot:** sıralı `(lineage_id, version_id)`; içerik-adresli.
- **Olay günlüğü:** yalnızca-ekleme, zaman damgasız sıra numarası; replay ile her snapshot yeniden kurulur. Kanıt: H4-1.
- **Nesne deposu:** içerik-adresli; aynı kimlik farklı içerik → hata.
- **Soy eşleyici:** diff ile kimlik çıkarımı; kesin eşleşme → bağlam çapası → bölme/birleştirme → benzerlik → artık kontrolü. v3'te birleştirme simetrisi eklenir (K4).
- **Chunk:** türev; bloklara referans verir, bloklar chunk'ı bilmez; strateji değişimi soy kaybetmez. Kanıt: H3-2.
- **Yeni primitifler (v3/çekirdek fazı, kanıt bekliyor):**
  - *Köken* zaten blokta; **katman** alanı eklenir: `layer ∈ {canonical, proposal, derived}`; ajan yazımı varsayılan `proposal`.
  - *Terfi* olayı: `proposal → canonical`, tetikleyen ilkeyi (insan, politika plugin'i) ve gerekçeyi taşır; ContextNest'in yayın kapısının blok düzeyi karşılığı.
  - *Kontrol noktası:* vault genelinde `(doc_path → snapshot_id)` haritası + olay günlüğü ofseti; adlandırılabilir; geçmiş-noktası sorgu bu kimlikle yapılır.
  - *Tüketim izi:* bir ajan sorgusuna dönen blok sürümleri ve kontrol noktası, yalnızca-ekleme ayrı günlüğe yazılır ("bu cevap hangi bloklara dayandı").
  - *Ham günlük / sürüm ayrımı:* her kayıt olayı ham günlüğe düşer (budanabilir); sürüm yalnız politika tetikleyicisiyle doğar (sessizlik, anlamsal fark eşiği); plugin parametre verir, çekirdek sınırları koyar.

### 4.2 L2 — Türev indeksler
- **Sözcüksel (BM25) birinci sınıf (K2).** Kanıt: v1 ve v2'de `hybrid` her hücrede en iyi. Çekirdek fazında Rust'a taşınır (tantivy).
- **Vektör:** kesin kosinüs (ANN yok) ölçülen boyutlarda yeterli ve deterministik; ANN ancak ölçek gerektirdiğinde ve determinizm garantisiyle (sabit insertion order, exhaustive doğrulama) eklenir.
- **Hibrit:** RRF ile birleşim; varsayılan arama yolu.
- **Graf:** wikilink + hiyerarşi + knn. Kanıt: `graph_flat` vektör üstüne küçük anlamlı kazanç; `graph_hybrid` sözcüksel üstüne **kazanç yok** (v2). **Çekirdeğe bağlanmaz; araştırma kolu (K3).**
- **Varlık önbelleği:** LLM ile blok başına çıkarım, `(version_id, model, istem hash)` anahtarlı, git'e giren önbellek. Pipeline zenginleştirici plugin'inin ilk örneği. Araştırma kolu.
- **Sıcak katman:** bellekte kaba kuvvet; yazma→aranabilir 30 ms altı. Kanıt: H3-1. Sıkıştırma soğuğa taşır.
- **Öğrenilen durum:** link ağırlıkları, reranker, embedding adaptörü — hepsi sürümlü, geri alınabilir. Henüz ölçülmedi.

### 4.3 L3 — Servis yüzeyi
- **MCP sunucusu:** `search`, `read_block`, `history`, `blame`, `propose`, `promote`, `checkpoint`, `trace` araçları. Mutasyon araçları terfi politikasına tabi.
- **CLI (`mdk`):** aynı yüzey; doğrulama ve betikleme için.
- **Sürekli sorgu:** akış üzerinde durum tutan, niyet değişiminde yeniden getiren primitif. Sıcak katman kanıtlı; niyet/patika kısmı araştırma kolu.

### 4.4 L4 — Plugin sınıfları
| Sınıf | Nerede çalışır | Örnek | Sözleşme |
|---|---|---|---|
| İstemci eklentisi | Editör/uygulama | Obsidian paneli, canlı görüşme yardımcısı | Obsidian modeli |
| Pipeline zenginleştirici | Sunucu/yerel iş | chunker, embedder, varlık çıkarıcı, reranker | Girdi/çıktı Arrow/JSON; içerik-adresli önbellek zorunlu |
| Karar/politika | Hook noktaları | terfi kararı, budama, ağırlıklandırma, sürüm tetikleyici | Çekirdek sınırları içinde parametrik tarif; sınır dışı → ret |

Hook noktaları: `on_block_event`, `on_version_trigger`, `on_promotion_request`, `on_search_rank`, `on_compaction`, `on_prune`.

### 4.5 L5 — Sync ve tenant (faz 2)
- Blok içi canlı düzenleme CRDT; blok sürümleri arası git-benzeri nesne alışverişi; türev indeksler asla senkronize edilmez, her replika yeniden üretir.
- Tenant = vault + anahtar + indeks + plugin seti. "Tek motor, çok replika": istemci ve sunucu aynı çekirdek, farklı kapasite ve politika.
- Gizlilik sınırı (K5): yerel vault varsayılan; sunucu pipeline isteğe bağlı; dinlenmede şifreli, işlem anında tenant anahtarıyla çözme. Zero-knowledge sunucu vaat edilmez.

## 5. Köken, katmanlar ve yankı kontrolü

Üç katman: **canonical** (insan yazımlı ya da terfi etmiş), **proposal** (ajan/transkript yazımı, onaysız), **derived** (indeks, çıkarım, özet). Arama varsayılan olarak canonical + proposal(düşük ağırlık); derived yalnız indeks olarak. Terfi olayı tek yoldur; terfi kararı plugin'e aittir, çekirdek yalnız olayın şeklini ve geri alınabilirliğini garanti eder. Onaylanmayan proposal'lar politikaya göre sönümlenir ve budanır. Öğrenme sinyalleri popülerlik yanlılığına karşı keşif payı ve zamanla sönüm taşır.

## 6. Arama ve canlı akış

- Varsayılan: `hybrid` (BM25 + vektör, RRF), structural chunk.
- Çok-adımlı sorular: graf sıralama sinyali olarak **kanıtlanamadı**; alternatif hipotez "graf = gezinme yapısı" (ajan ikinci sorguyu link üzerinden atar; akış üzerinde yayılan aktivasyon) araştırma kolunda.
- Canlı akış: segmentleyici → niyet durumu (üstel ortalama + açı değişimi, histerezis) → tohumlu getirme; patika günlüğü üretir; öğrenilmiş SSM ancak günlük biriktikten sonra.

## 7. Determinizm sözleşmesi
- Kimlikler blake3; zaman damgası kimliğe girmez; soy kimliği `(doc_path, ilk version_id, sıra)` türevi.
- Değerlendirme gerçekleri içerik-referanslı; çözülemeyen → hata.
- LLM çıktıları yalnız önbellek üzerinden; önbellek anahtarı model+istem+içerik.
- Her koşturma manifest'li; rapor tek koşturmadan; çift temiz koşturma doğrulaması.

## 8. Güvenlik ve sınırlar
- Yazma yetkisi çekirdekte değil, servis yüzeyinde; çekirdek yalnız olayın kökenini kaydeder.
- Plugin'ler kaynak-sınırlı; sunucu tarafı tenant kodu WASM ya da eşdeğer izolasyonda (faz 2).
- Ölçek sınırı dürüstçe: on binlerce–yüz binlerce doküman, anlam-yoğun korpus. Milyarlarca satır bu sistemin işi değil.

## 9. Kanıt durumu

| Bileşen | Durum | Kanıt |
|---|---|---|
| Diff ile soy takibi (kimlik yazmadan) | **Kanıtlı** | v2 H1-S1 1.00, S4 1.00, R1 0.9998, R2 0.979 |
| Bölme tespiti | **Kanıtlı** | v2 H1-S3 0.96 |
| Birleştirme tespiti / yanlış eşleme | **Açık** | v2 H1-S2 0.038 > 0.02; merge kaynaklı; K4 ile v3 |
| Zor sentetik düzenlemeler | **Ölçülmedi** | v2 ızgarası düz; v3 D1-hard |
| Blok/chunk ayrımı, artımlı indeks | **Kanıtlı** | v2 H3 |
| Determinizm, çift koşturma | **Kanıtlı** | v2 H4-1, H4-2 (22.359 alan, fark 0) |
| Hibrit arama tabanı | **Kanıtlı** | v1, v2 tüm hücreler |
| Graf sıralama sinyali | **Desteklenmedi** | v2 H2-1/4/5 kaldı |
| Varlık grafı | **Araştırma** | — |
| Akış durumu / patika | **Araştırma** | — |
| Katman, terfi, kontrol noktası, tüketim izi | **Tasarlandı** | çekirdek fazı |
| Öğrenilen ağırlıklar / adaptör | **Tasarlandı** | — |
| Sync, tenant | **Faz 2** | — |

## 10. Kararlar

- **K1 ContextNest:** kavram ve terimler benimsenir, implementasyon değil; blok düzeyi genişleme olarak konumlanır.
- **K2 Sözcüksel indeks birinci sınıf:** BM25 çekirdek bileşeni; Rust'a taşınır.
- **K3 Graf arama çekirdeğe bağlanmaz:** araştırma kolu; yeni hipotezler (varlık grafı, gezinme) ayrı ölçülür.
- **K4 Birleşmede soy sürekliliği (v3 öncesi alınan karar):** Bölme ve birleştirme **baskın parçanın soyunu korur**; bölmede en uzun yeni parça eski soyu devralır, diğerleri yeni soy + ebeveyn bağı; birleştirmede sonuç en uzun eski parçanın soyunu devralır, olay tüm ebeveynleri kaydeder. Gerekçe: soya bağlı sinyal ve linklerin sürekliliği. Bu v2 spesifikasyonundaki "split/merge sonuçları yeni soy alır" kuralını değiştirir ve v3 gerçek tanımına yansır; S2 tanımı buna göre yeniden yazılır ve bu değişiklik v3 kaydında açıkça "sonuç görüldükten sonra alınmış tasarım kararı" olarak işaretlenir.
- **K5 Gizlilik sınırı:** yerel vault + isteğe bağlı sunucu pipeline; zero-knowledge sunucu yok.
- **K6 Rust çekirdek kalıcı:** v2 crate atılmaz; çekirdek fazı üzerine kurulur.
- **K7 LLM çıkarımı yalnız önbellekli:** Haiku sınıfı model; önbellek git'te; doğrulama API'siz.
- **K8 Yankı kontrolü katman modeliyle:** ajan yazımı varsayılan `proposal`; terfi tek yol.

## 11. Yol haritası

1. **v3 (kısa):** K4 ile eşleyici; D1-hard; H1 tamamı yeniden; H3/H4 regresyon. Geçerse §4.1 ve `01_SPEC.md` §1–7 dondurulur.
2. **Çekirdek fazı 1:** katman/terfi/kontrol noktası/tüketim izi primitifleri; BM25 Rust'ta; MCP yüzeyi; sürüm tetikleyici politikası; ilk ürün sahnesi uçtan uca.
3. **Araştırma kolları (paralel, bloke etmez):** varlık grafı (önbellekli Haiku), akış durumu/patika; her biri önceden kayıtlı eşikle.
4. **Çekirdek fazı 2:** sync, tenant, plugin çalışma zamanı izolasyonu.

## 12. Açık sorular
- Tekrar bloklarda bağlam çapası yetmediğinde (aynı içerik, aynı komşular) ne olur; kabul edilen kayıp mı, konum tabanlı çapa mı?
- Sürüm tetikleyici varsayılanları (sessizlik süresi, anlamsal fark) hangi veriyle kalibre edilir?
- Tüketim izi hacmi; budama politikası.
- Varlık grafının "kılık değiştirmiş sözcüksel indeks" olmadığını hangi soru alt kümesi gösterir?
