# P1-C — İndeks

Başlangıç koşulu: P1-B.1 doğrulandı (`p1b1-final` etiketi).

## Amaç
`index/` crate'ini yazmak: sözcüksel indeks, vektör deposu, embedder, hibrit birleşim, kontrol noktasına göre deterministik sorgu, katman filtresi, sıcak katman. Spec §9'u C3-01..C3-04 ile doğrulamak. Arama kalitesinin prototip v2 `hybrid` sonuçlarıyla tutarlı olduğunu `research/` ile göstermek. MCP/HTTP yok (P1-D).

## Kapsam (`index/`, yalnız `core`'a bağımlı)
- **Chunk kaynağı:** `core::chunk(snapshot, Structural)`; indeks anahtarı `(doc_path, chunk_id)`; her chunk kaydı `snapshot_id`, `lineage_ids`, `layer` (kapsadığı blokların en düşük katmanı: biri `proposal` ise `proposal`), `heading_path`, `text`.
- **Sözcüksel:** `tantivy`; tokenizasyon küçük harf + `[a-z0-9]+` eşdeğeri (prototiple aynı; `research` regresyonu bunu ölçer); BM25 varsayılan parametreler; indeks dizini `.mdkernel/index/lexical/`; **deterministik segment düzeni:** tek yazar, belge sırası `(doc_path, chunk_id)` sözlük sırası, `commit` sonrası `merge` ile tek segment; sorgu sonucu eşitlikte `(doc_path, chunk_id)` ile kırılır.
- **Vektör:** `.mdkernel/index/vectors/` f32 normalize, kesin kosinüs (ANN yok); sıralama eşitliği aynı anahtarla kırılır. Her kayıt `embedder_id` (model hash + sürüm) taşır; farklı `embedder_id` karışımı → `Err`.
- **Embedder:** `trait Embedder { fn id(&self) -> &str; fn embed(&self, texts) -> Vec<Vec<f32>> }`; dahili `OnnxMiniLm` (`ort` crate, `all-MiniLM-L6-v2` ONNX, mean pooling, normalize; model dosyası `scripts/fetch_model.sh` ile HuggingFace'ten indirilir, blake3 ile sabitlenir `models/MANIFEST.json`; CPU, tek iş parçacığı sabit, deterministik). Tokenizer: `tokenizers` crate, aynı model dosyasından.
- **Hibrit:** `flat`, `lexical`, `hybrid` (RRF `1/(60+rank)`, her taraftan top-50), hepsi `(query, k, at: Checkpoint, include_proposal: bool) -> Vec<Hit>`; `Hit { doc_path, chunk_id, lineage_ids, layer, score }`; `include_proposal=false` varsayılan; `true` ise `proposal` hit'ler etiketli ve skorları ×0.5.
- **Kontrol noktasına göre sorgu:** indeks tüm görülen chunk sürümlerini tutar; sorgu `at` checkpoint'in `docs` haritasındaki `snapshot_id`'lere ait chunk'larla sınırlanır. `derived` katman hiçbir zaman döndürülmez.
- **Tüketim izi:** her sorgu `core::record_trace(at, consumer, H(query), returned)`; P1-B API'si.
- **Sıcak katman:** `.mdkernel/index/hot/` bellekte + diske yazılan ek; `watch(doc)` → `core::ingest` → değişen soyları içeren chunk'lar → embed → sıcak; sorgu sıcak+soğuk birleştirir, aynı anahtarda sıcak kazanır; `compact()` sıcağı soğuğa taşır, sonuç değişmez.
- **CLI (`core/src/bin/mdk.rs`, P1-D'de taşınır):** `index build [--embedder]`, `index status`, `search <q> [--k] [--mode flat|lexical|hybrid] [--at <checkpoint>] [--include-proposal]`, `index compact`, `index watch <doc>`.

## Spec ve uyumluluk
- `spec/fixtures/C3-01..04/`; `make conformance` → `level_claimed 3`.
- Spec §9 yalnız açıklık (K-B1.3 ölçütü) ya da karar kaydıyla değişir; tur sonunda "Validated".

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| C-1 | C3-01..C3-04 | 4/4, `level_claimed 3` |
| C-2 | C1 17/17, C2 8/8 | geçer |
| C-3 | D3 vault + D4 soru seti (git'te), `hybrid/structural`, prototip v2 `hybrid` nDCG@10 ile fark (single, multi) | |Δ| ≤ 0.01 |
| C-4 | Aynı sorgu, aynı checkpoint, aynı indeks: 100 tekrar sıralama | birebir |
| C-5 | İndeks silinip aynı checkpoint'ten yeniden kurulunca sıralama | birebir |
| C-6 | Çift koşturma: `index/` dizinleri bayt eşit, 200 sorgunun sıralaması eşit | eşit |
| C-7 | `proposal` chunk'lar varsayılanda hiç dönmez; `include_proposal` ile etiketli ve sonda | property testi 1000 sorgu |
| C-8 | Yazma→sıcakta aranabilir p95 (ONNX CPU, model yüklü) | < 100 ms |
| C-9 | `compact()` öncesi/sonrası 200 sorgu sıralaması | birebir |
| C-10 | Embedding regresyonu: 50 sabit chunk için ONNX embedding ile prototip `sentence-transformers` embedding'i (git'te `research/data/embed_ref.json`) kosinüs | ≥ 0.999 |
| C-11 | Regresyon: P1-B.1 kalite alanları | fark 0 |

## Karar kuralı
C-1..C-11 geçti → spec §9 "Validated", `level_claimed 3`; P1-D başlar. C-3 veya C-10 kaldı → embedder/tokenizer uyumsuzluğu; karar talebi; P1-D **başlar** (MCP indeks kalitesine bağlı değil) ama 0.1.0 yayımı bekler. C-4/C-5/C-6 kaldı → determinizm; P1-D başlamaz. C-7 kaldı → yönetişim ihlali; P1-D başlamaz.

## Yapılmayacaklar
ANN (HNSW vb.); graf, varlık, reranker; MCP/HTTP; istemci; birden fazla embedder; GPU; H2 ölçümü (C-3 yalnız `hybrid` regresyonudur, graf stratejileri koşturulmaz).
