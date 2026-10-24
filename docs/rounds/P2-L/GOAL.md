# P2-L — Dil: Türkçe ve Çok Dilli İndeks; 0.2.0 Yayımı (Geliştirme Turu, Blok 1)

Başlangıç: P2-0 dizisi kapandı (K-P2-03.1), P2-A tamamlandı. Bu tur K-P2.4 dozuyla bir **geliştirme turu**dur: birim/entegrasyon testleri, kalite kapıları, hızlı doğrulama zinciri; tam çift koşturma ve bağımsız doğrulama **Blok 1'in sonunda** (P2-P bitince). Hedef süre 1 gün.

## Kapsam
1. **L0 — Açılış, kapanış kalıntıları, 0.2.0 yayımı.** (a) K-P2-03.1 maddeleri: NORTH_STAR "ölçülen koşul" notuna P2-0.3 ölçümü (yarışa bağlı; varsayılan `canonical` kesin); THREAT_MODEL sınır metni; spec §7.1 varsayılan cümlesi CRITERION ile `canonical`, C2-11'in başlığı "`unverified` seçildiğinde" (beklenenler değişmez), eskimiş "P2-0.2 doğrulaması" atfı. (b) P2-0.3 doğrulamasının Ö3'ü: yeniden adlandırma sonrası `canonical` → `unverified` düşüşü `--attest-log` ile yeniden üretilir; hata ise düzeltilir ve birim testi yazılır (spec §7.1 "yeniden adlandırma katman düşürmez"); değilse neden kaydedilir. (c) P2-A doğrulamasının F1 (`migrate --dry-run` planı zincirlenir) ve F2 (`install.sh` `jq` önkoşulu). (d) **Hızlı doğrulama kipi:** `make verify FAST=1` — temiz klon + yalnız aktif turun kontrolleri + `rootmark conformance` + son resmi RUN'a karşı regresyon + tek çift (iki dizin); tam zincir `make verify` değişmez ve haftalık CI'da kalır (dondurma istisnası, önce/sonra kanıt). (e) **0.2.0 yayımı:** sürümler `0.2.0` (Cargo workspace, Python wheel, Obsidian `manifest.json`), CHANGELOG 0.2.0 (P2-0 dizisi: tasdikli yazma opt-in, spec 2.0.2; P2-A: göç çerçevesi, `status.migrations`, Linux x86_64 resmi hedef, kurulum betikleri, provenance), RELEASE.md R-maddeleri; **ayrı küçük PR** (`release/0.2.0`), orkestratör birleştirir; birleşince ajan `main`'de `v0.2.0` etiketini atar, `release.yml`'i tetikler, GitHub release'i varlıklarla oluşturur (K-GH.4 emsali, insan onayı bu GOAL'dedir); imzalama yok.
2. **L1 — Sözcüksel varyant `uni_fold_tr`.** Unicode farkında tokenizer (harf/rakam dizileri, `Alphabetic`/`Numeric`), Türkçe büyük/küçük harf katlaması (İ→i, I→ı, ardından genel küçültme), Unicode NFC; manifest `lexical.tokenizer` kimliği; tantivy tokenizer kaydı; `mdk_lower_alnum` aynen kalır ve İngilizce kalite referansı (`quality_ref_rust.json`) **değişmez**. Varsayılan tokenizer kararı L4'ün ölçümüne bağlı (aşağıda).
3. **L2 — Çok dilli embedder seçeneği.** `[index] embedder = "minilm-l6-v2" | "multilingual-minilm-l12-v2"` (paraphrase-multilingual-MiniLM-L12-v2, ONNX, sabitlenmiş sha256+blake3 manifest, `scripts/fetch_model.sh` genişlemesi); `embedder_id` manifest'te; kesin kosinüs ve içerik-adresli önbellek aynen; varsayılan `minilm-l6-v2` kalır.
4. **L3 — Yol öneki filtresi.** `search.path_prefix` (vault-göreli, normalize; birden çok önek dizi olarak), sıralamadan önce uygulanır, belirlenimli; `query_hash`'e girer, `trace` kaydına yazılır; MCP ve HTTP; CLIENT_CONTRACT §2 (CRITERION ile).
5. **L4 — Türkçe ölçüm.** `research/data/d_tr`: sabitlenmiş, lisansı uygun açık Türkçe korpus (ör. Türkçe Vikipedi'nin belirli bir dökümünden seçilmiş ≥ 300 madde; kaynak, revizyon ve sha256 kayıtlı) + ≥ 100 TR/EN karışık belge (aynı kaynaktan iki dilli maddeler); sorgular deterministik üretilir (D3 yöntemi: başlık/cümle örnekleme, sabit tohum), doğruluk kaynak belge; metrikler nDCG@10, recall@10, MRR; `lexical`/`flat`/`hybrid` × {`mdk_lower_alnum`, `uni_fold_tr`} × {`minilm-l6-v2`, `multilingual-minilm-l12-v2`}. **LLM yok.**
6. **L5 — Zincir, rapor, hazır.** `make verify FAST=1` temiz; tam `make verify` bu turda **bir kez** (yeni varyantların determinizmi için); FINAL; READY; tek PR.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| L-1 | `uni_fold_tr`: "değerlendirme", "Şengönül", "İSTANBUL/istanbul", "ılık/ILIK" tek token ve katlama birim testleri; `mdk_lower_alnum` çıktısı 200 chunk'ta değişmedi (C2 referansı) | hepsi |
| L-2 | D-TR, `lexical`: `uni_fold_tr` nDCG@10 − `mdk_lower_alnum` nDCG@10 | ≥ +0.10 |
| L-3 | İngilizce referans (`quality_ref_rust.json`, 618 değer) `mdk_lower_alnum` ile Δ | 0 |
| L-4 | İngilizce referans `uni_fold_tr` ile Δ nDCG@10 (varsayılan kararı için) | raporlanır; |Δ| ≤ 0.001 ise `uni_fold_tr` varsayılan olur, değilse opt-in |
| L-5 | D-TR, `flat`: multilingual − minilm nDCG@10 | ≥ +0.05 |
| L-6 | D-TR, `hybrid` en iyi hücre (yeni varyant + multilingual) − eski en iyi | ≥ +0.10 |
| L-7 | Yol öneki filtresi: birim/sözleşme testleri; filtreli sorgu `trace`'te; eski `query_hash`'ler (filtresiz) değişmez | hepsi |
| L-8 | Eski indeks (`mdkernel-index/3`, eski tokenizer) yeni ikilide `status.migrations` → `migration_required`; `rootmark migrate` yeniden kurar; `events_hash` değişmez | 3 vault |
| L-9 | Determinizm: tam çift bir kez, iki varyant ve iki embedder ile indeks baytları ve kalite alanları fark 0 | eşit |
| L-10 | Uyumluluk: C3 fixture'ları iki tokenizer varyantıyla (yeni fixture'lar); mevcut 39 değişmez | hepsi |
| L-11 | L0: Ö3 düşüşü açıklandı (hata ise düzeltildi + test); FAST kipi ≤ 20 dk; 0.2.0 yayımı: etiket, release varlıkları 3 hedef, `rootmark --version` = `0.2.0`, provenance 5/5, `release_check.sh` temiz | yazılı ve testli |

## Karar kuralı
- **Hepsi geçti** → P2-P başlar (Blok 1 devam); varsayılan tokenizer L-4'e göre; multilingual opt-in.
- **L-2, L-5 veya L-6 kaldı** (diğerleri geçti) → yeni varyantlar opt-in olarak kalır, Türkçe kazanç "ölçüldü, eşik altı" diye raporlanır; karar talebi (model/tokenizer seçeneği); P2-P yine başlar.
- **L-3, L-8, L-9 veya L-10 kaldı** → regresyon; tur kaldı, karar talebi; P2-P başlamaz.
- **L-11'in yayım kısmı kaldı** → yayım ertelenir, tur yine ilerler; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
Spec §1–6; eski tokenizer ve İngilizce referansın değişmesi; LLM ile sorgu/etiket üretimi; ANN; graf; sync; kod imzalama; varsayılan embedder değişikliği; `unverified` varsayılanı (kapandı).
