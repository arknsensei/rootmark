# rootmark — Mimari Modeli

Sürüm 1.2.1 · 2026-10-17 · Durum: **§4.1 ve spec §1–6 DONDURULDU (v3 geçti, bağımsız doğrulandı); §4.1 yeni primitifler kanıtlı (P1-B, P1-D.1; spec §7, §8, §11 Validated); §4.2 sözcüksel/vektör/hibrit/sıcak katman kanıtlı (P1-C, P1-C.1; spec §9 Validated), graf/varlık/öğrenilen durum araştırma; §4.3 MCP/HTTP/CLI kanıtlı (P1-D, P1-D.1); §4.4 istemci eklentisi P1-E'de ölçüldü (bağımsız doğrulama "şu koşullarla güvenilir", koşul P1-E.1'de); §4.5 faz 2 taslak.** 1.2: yalnız kanıt durumu (K-E6.1); 1.2.1: P1-E doğrulaması ve e2e tanımı (K-E1.1, K-E1.3).

Bu belge tartışmaz, karar verir. Gerekçeler kısa tutulmuştur; her kararın kanıt durumu §9'daki tabloda, karar listesi §10'dadır.

---

## 1. Tez

Markdown dosyaları, insanların ve ajanların birlikte yazdığı canlı bilginin tek gerçek kaynağıdır. Bu dosyaların üstünde **blok düzeyinde soy, köken ve deterministik türev indeks** sağlayan bir çekirdek, bugün hiçbir üründe olmayan ve pazarın açıkça ihtiyaç duyduğu katmandır. Vektör arama, graf, sync ve plugin'ler bu çekirdeğin üstüne kurulur; çekirdeğin özgün kısmı onlar değil, onların bağlandığı veri modelidir.

## 2. Konumlandırma

- **Değiliz:** Vektör veritabanı (Qdrant, pgvector), ajan hafıza API'si (Mem0, Zep, Letta), not uygulaması (Obsidian, Logseq), ekip wiki'si (Notion, Outline).
- **Yakın kuzen:** ContextNest (2026) — "getirme yönetişim değildir" tezini ve doküman düzeyinde hash-zincirli sürüm, yayın kapısı, deterministik seçim, tüketim izi mekanizmalarını tanımlar. Blok düzeyinde atıf, çok kullanıcılı düzenleme ve yerleşik anlamsal getirme bizzat "gelecek iş" olarak bırakılmıştır. rootmark tam bu üç boşluğu hedefler.
- **ContextNest ile ilişki (karar K1):** Kavram seti ve terimler benimsenir (tipli doküman, taslak/yayın, kontrol noktası, tüketim izi). Spesifikasyonun blok düzeyine indirgenmiş bir genişlemesi olarak konumlanılır; implementasyon paylaşılmaz. Uyumlu dışa aktarım (ContextNest vault üretimi) ileride bir plugin'dir.
- **Hedef kullanıcı (öncelik sırası):** (1) ajan hafızasını md'de tutan geliştiriciler, (2) yönetişim kaygılı ekipler, (3) Obsidian güç kullanıcıları.
- **İlk ürün sahnesi:** Ajan bir vault'a yazar; kullanıcı her bloğun kimin tarafından ne zaman değiştirildiğini görür; onaylanmamış ajan blokları aramaya girmez; dünkü hale blok düzeyinde dönülebilir; hepsi sıradan md dosyaları üstünde, Obsidian'da açılır halde.

## 3. İlkeler

0. **İstemci bağımsızlığı.** Çekirdek hiçbir istemciyi bilmez; tek uyumluluk sözü düz markdown + wikilink. Obsidian, VS Code, CLI, MCP istemcileri eşit statüdedir. "Obsidian'da açılır" bir ürün sahnesidir, mimari hedef değil.
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
- **Yeni primitifler (çekirdek fazı; kanıtlı: P1-B C2, bağımsız doğrulandı; P1-D.1 D1-1..D1-13, "şu koşullarla güvenilir", koşulları P1-E/E0'da kapatıldı, P1-E doğrulamasında teyit edildi (V0)):**
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
- **CLI (`rootmark`):** aynı yüzey; doğrulama ve betikleme için.
- **Sürekli sorgu:** akış üzerinde durum tutan, niyet değişiminde yeniden getiren primitif. Sıcak katman kanıtlı; niyet/patika kısmı araştırma kolu.

### 4.4 L4 — Plugin sınıfları
| Sınıf | Nerede çalışır | Örnek | Sözleşme |
|---|---|---|---|
| İstemci eklentisi | Editör/uygulama | Obsidian paneli, canlı görüşme yardımcısı | Obsidian modeli |
| Pipeline zenginleştirici | Sunucu/yerel iş | chunker, embedder, varlık çıkarıcı, reranker | Girdi/çıktı Arrow/JSON; içerik-adresli önbellek zorunlu |
| Karar/politika | Hook noktaları | terfi kararı, budama, ağırlıklandırma, sürüm tetikleyici | Çekirdek sınırları içinde parametrik tarif; sınır dışı → ret |

Hook noktaları: `on_block_event`, `on_version_trigger`, `on_promotion_request`, `on_search_rank`, `on_compaction`, `on_prune`. Faz 2'nin ilk bildirimsel terfi politikası (P2-P, K-Q.1 (2)): `policy.toml`'un `[[promotion.rules]]` listesi (aktör, kaynak, yol öneki tam eşleşmesi; joker/regex/zaman yok); eşleyici `server`'da yalnız `propose`'ta koşar ve `Promoted`'ı çekirdeğin `on_promotion_request` kancasından (`PromotionPolicy`) yazar — çekirdek karar vermez; genişleme WASM politikasıyla (P2-D). `on_prune` gerçekleşti (P2-R, K-Q.1 (5)): `policy.toml`'un `[[prune.rules]]` listesi (katman `proposal`/`unverified`, son olaydan sonra en az `idle_checkpoints` checkpoint, isteğe bağlı yol önekleri; `canonical` asla) eşleyicisi `server`'da yalnız açık budama çağrısında (`rootmark prune`, `POST /v1/prune`) koşar ve `Demoted` (`meta.prune`) olayını çekirdeğin `PromotionPolicy` kancasından (`Vault::prune_inactive`) yazar; `[retention] keep_checkpoints` iz ve checkpoint dosyalarını bayt bayt `.rootmark` içindeki arşive taşır (`rootmarkcore::retention`), yalnız türetilmiş konum kayıtları silinir.

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

## 7a. Dil kararı (K9)
Çalışma zamanı tamamen Rust: çekirdek, indeks (tantivy, vektör, ONNX embedder), MCP/HTTP sunucu, CLI. Python yalnız `research/`; TypeScript yalnız `clients/`. Fazlı dil geçişi yoktur. Ayrıntı `docs/PACKAGES.md`.

## 8. Güvenlik ve sınırlar
- Yazma yetkisi çekirdekte değil, servis yüzeyinde; çekirdek yalnız olayın kökenini kaydeder.
- Plugin'ler kaynak-sınırlı; sunucu tarafı tenant kodu WASM ya da eşdeğer izolasyonda (faz 2).
- Ölçek sınırı dürüstçe: on binlerce–yüz binlerce doküman, anlam-yoğun korpus. Milyarlarca satır bu sistemin işi değil.

## 9. Kanıt durumu

| Bileşen | Durum | Kanıt |
|---|---|---|
| Diff ile soy takibi (kimlik yazmadan) | **Kanıtlı** | v2 H1-S1 1.00, S4 1.00, R1 0.9998, R2 0.979 |
| Bölme tespiti | **Kanıtlı** | v2 H1-S3 0.96 |
| Birleştirme tespiti / yanlış eşleme | **Kanıtlı** | v3 H1-S2 0.000, H1-H2 0.000, olay F1 0.997 (K4 semantiği) |
| Zor sentetik düzenlemeler | **Kanıtlı (sınırlı)** | v3 H1-H1 0.988; aşırı yeniden yazımda 0.91; bölme eşiği veriyle sınanmadı (P1-A regresyonu) |
| Blok/chunk ayrımı, artımlı indeks | **Kanıtlı** | v2 H3 |
| Determinizm, çift koşturma | **Kanıtlı** | v2 H4 (22.359 alan), v3 H4 (1.577 alan, fark 0), v2 davranışı bayrakla birebir |
| Hibrit arama tabanı | **Kanıtlı** | v1, v2 tüm hücreler |
| Graf sıralama sinyali | **Desteklenmedi** | v2 H2-1/4/5 kaldı |
| Varlık grafı | **Araştırma** | — |
| Akış durumu / patika | **Araştırma** | — |
| Katman, terfi, kontrol noktası, tüketim izi | **Kanıtlı** | P1-B B-1..B-10 (C1+C2 25/25, bağımsız doğrulandı; C2 bugün 10/10); P1-D.1 D1-1..D1-13 (adlandırma, aktörlü olaylar; doğrulama "şu koşullarla güvenilir", koşullar P1-E/E0'da kapatıldı, P1-E doğrulamasında teyit edildi (V0)); spec §7/§8/§11 Validated |
| Ham günlük / sürüm tetikleyici sınırları | **Kanıtlı** | P1-B B-6 (politika sınırları), C2 |
| Sözcüksel (Rust BM25) + kesin vektör + hibrit, kontrol noktasında sorgu | **Kanıtlı** | P1-C/P1-C.1: C3 7/7, Rust kalite referansı 618 değer Δ 0, prototip hattı varyant değişimiyle Δ 0; spec §9 Validated |
| Sıcak katman (yazma→aranabilir) | **Kanıtlı** | P1-C C-8 p95 32.2 ms (< 100 ms) |
| Servis yüzeyi (MCP, HTTP, CLI), ajan yazımı yönetişimi | **Kanıtlı** | P1-D D-1..D-11 ("şu koşullarla güvenilir", koşullar P1-D.1'de kapandı); P1-D.1 (`append_only`, kanal → aktör, tek yazar kuyruğu, kararlı hata kodları; "şu koşullarla güvenilir", koşullar P1-E/E0'da kapatıldı, P1-E doğrulamasında teyit edildi (V0)) |
| İstemci eklentisi (Obsidian) ve ürün sahnesi uçtan uca | **Kanıtlı, koşullu** | P1-E E-1/E-2 11/11 (resmi RUN `20261017T024341Z-632f29e`; E-2 eklentinin komut katmanıyla ölçüldü); bağımsız doğrulama "şu koşullarla güvenilir": sahne ve eklenti gerçek Obsidian arayüzünde, ayrı bir örnekte (`--user-data-dir`, CDP) 11/11 koşturuldu — Obsidian'ın başsız kipi yok, ama arayüz betikle sürülebilir; gerçek arayüz e2e'si P1-E.1'den itibaren resmi kapı (K-E1.3); koşul (`status` sözleşme ihlali) P1-E.1'de |
| Yayım ikilisi (0.1.0: macOS arm64) | **Kısmen** | P1-E E-7 2/4: macOS arm64 ve Linux x86_64 34/34; Intel macOS derlenemedi (sabitlenmiş `ort`'ta hazır onnxruntime yok); Windows ölçülmedi (CI tetiklenmedi); karar kuralı gereği yayım ertelendi (K-E5-T). K-E1.1: 0.1.0 hedefi yalnız `aarch64-apple-darwin`, Linux/Windows CI bilgi, Intel macOS hedef değil |
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
- **K9 Çalışma zamanı tamamen Rust:** MCP sunucusu dahil; önceki "önce Python" önerisi geri çekildi. Python yalnız araştırma, TypeScript yalnız istemci.
- **K10 Tek monorepo, yedi paket:** `docs/PACKAGES.md`.
- **K11 Spesifikasyon yayımlanır:** `spec/` Apache-2.0, İngilizce; ContextNest kavram setiyle uyumlu, blok düzeyi.

## 11. Yol haritası

1. ~~v3~~ **Tamamlandı:** tüm eşikler geçti, bağımsız doğrulandı; spec §1–6 donduruldu.
2. **Çekirdek fazı 1:** katman/terfi/kontrol noktası/tüketim izi primitifleri; BM25 Rust'ta; MCP yüzeyi; sürüm tetikleyici politikası; ilk ürün sahnesi uçtan uca.
3. **Araştırma kolları (paralel, bloke etmez):** varlık grafı (önbellekli Haiku), akış durumu/patika; her biri önceden kayıtlı eşikle.
4. **Çekirdek fazı 2:** sync, tenant, plugin çalışma zamanı izolasyonu.

## 12. Açık sorular
- Tekrar bloklarda bağlam çapası yetmediğinde (aynı içerik, aynı komşular) ne olur; kabul edilen kayıp mı, konum tabanlı çapa mı?
- Sürüm tetikleyici varsayılanları (sessizlik süresi, anlamsal fark) hangi veriyle kalibre edilir?
- Tüketim izi hacmi; budama politikası.
- Varlık grafının "kılık değiştirmiş sözcüksel indeks" olmadığını hangi soru alt kümesi gösterir?
