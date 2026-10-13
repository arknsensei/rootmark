# 01 — Veri Modeli ve Çekirdek API (v2)

Aynen uygulanır. §1–7 Rust çekirdeğinde (`core/`), §8–10 Python'da (`mdkernel/`). Python tarafı §1–7'yi yalnız `mdkernel_core` bağlaması üzerinden kullanır.

Hash fonksiyonu her yerde `blake3`, çıktı hex'in ilk 32 karakteri. Zaman damgası hiçbir kimliğe girmez.

## 1. Blok (Block) — değişmez, içerik-adresli

```rust
pub struct Block {
    pub version_id: String,      // blake3(block_type + "\n" + normalized_text)
    pub lineage_id: String,      // bkz. §5.1, deterministik
    pub block_type: BlockType,   // Heading | Paragraph | ListItem | Code | Table | Blockquote | Other
    pub text: String,            // ham markdown
    pub normalized_text: String, // §1.1
    pub heading_path: Vec<String>,
    pub heading_level: Option<u8>,
    pub parents: Vec<String>,    // önceki version_id'ler
    pub provenance: Provenance,  // { author_kind: Human|Agent|Transcript|Synthetic, source: String }
}
```

### 1.1 Normalizasyon
`text` → Unicode NFC → `\r\n`/`\r` → `\n` → baş/son boşluk kırp → ardışık boşluk karakterleri (`\s+`) tek boşluk. **Kod bloğunda** yalnız NFC ve satır sonu normalizasyonu. Vurgu işaretleri korunur.

### 1.2 Blok sınırları
Parser: `pulldown-cmark` (sabit sürüm), `Options::ENABLE_TABLES | ENABLE_FOOTNOTES | ENABLE_STRIKETHROUGH`. Üst düzey olaylardan:
- `Heading` → `Heading`, `heading_level` 1-6
- Liste dışı `Paragraph` → `Paragraph`
- Her üst düzey `Item` → `ListItem`; iç içe listeler ebeveyn öğenin metnine dahil, ayrı blok değil
- `CodeBlock` → `Code`
- `Table` → tek `Table`
- `BlockQuote` → tek `Blockquote`
- `Rule`, `Html`, `FootnoteDefinition`, diğer → `Other`
- Baştaki `---` ile sınırlı YAML frontmatter → `Snapshot.frontmatter` (ham string), blok üretmez
- Boş paragraf blok üretmez

`heading_path`: bloğun üstündeki başlık metinleri kökten yaprağa; `Heading` bloğunun kendi yolu kendisini içermez.

### 1.3 Wikilink
`[[hedef]]`, `[[hedef|etiket]]`, `[[hedef#başlık]]`, `[[hedef#başlık|etiket]]`. `hedef` dosya adı (uzantısız, yol bağımsız, baş/son boşluk kırpılmış). `extract_wikilinks(text) -> Vec<WikiLink { target, heading: Option<String> }>`.

## 2. Snapshot

```rust
pub struct Snapshot {
    pub snapshot_id: String,     // blake3(version_id'ler "\n" ile)
    pub doc_path: String,        // vault köküne göre, "/" ayraçlı
    pub blocks: Vec<(String, String)>, // (lineage_id, version_id), sıralı
    pub frontmatter: Option<String>,
    pub parent_snapshot: Option<String>,
}
```

## 3. Olay günlüğü — yalnızca-ekleme JSONL

`<vault>/.mdkernel/events.jsonl`. Satır:
```rust
pub struct Event {
    pub seq: u64,                // 0'dan artan, vault genelinde
    pub doc_path: String,
    pub kind: EventKind,         // BlockCreated | BlockUpdated | BlockMoved | BlockSplit | BlockMerged | BlockDeleted | SnapshotCommitted
    pub lineage_ids: Vec<String>,
    pub old_version_ids: Vec<String>,
    pub new_version_ids: Vec<String>,
    pub snapshot_id: Option<String>,
    pub meta: serde_json::Value, // matcher_version, score, anchor_kind vb.
}
```
`seq` zaman damgası değildir; deterministiktir. `replay(doc_path) -> Option<Snapshot>` olayları yeniden oynatarak son snapshot'ı kurar ve `snapshot_id` doğrulanır. Günlük **hash'i** (`blake3` tüm dosya) H4-1'in ölçtüğü şeydir; bu yüzden olaylarda makineye bağlı hiçbir alan olamaz.

## 4. Nesne deposu
`<vault>/.mdkernel/objects/<vid[0..2]>/<vid>.json`, `<vault>/.mdkernel/snapshots/<sid>.json`. Var olan nesne yeniden yazılmaz (byte eşitliği kontrol edilir; farklıysa bu bir hatadır ve panik yerine `Err` döner).

## 5. Soy eşleyici v2 — `match(prev: Option<&Snapshot>, drafts: &[BlockDraft]) -> (Vec<Block>, Vec<Event>)`

### 5.1 Deterministik soy kimliği
Yeni soy oluşturulurken:
```
lineage_id = blake3("lineage\n" + doc_path + "\n" + version_id + "\n" + k)
```
`k` = bu snapshot'ta aynı `version_id`'ye sahip, **yeni soy alan** bloklar arasında 0'dan başlayan sıra (belge sırasıyla). Böylece aynı vault aynı sırayla iki kez ingest edilirse kimlikler aynıdır.

### 5.2 Bağlam çapası (tekrar bloklar için)
Her blok için `ctx = blake3(prev_version_id|"" + "\n" + version_id + "\n" + next_version_id|"")`. Aynı `version_id`'ye sahip birden çok blok varsa eşleme önce `ctx` eşitliğiyle, sonra kalan kopyalar için sırayla yapılır (k'ıncı → k'ıncı). Olay `meta.anchor_kind ∈ {"exact","ctx","ordinal"}`.

### 5.3 Adımlar (sırayla; her adım kalanlar üzerinde)
1. **Kesin eşleşme** (`version_id` eşit; tekrar varsa §5.2). Konum değiştiyse `BlockMoved`; konum tanımı: `prev` içindeki sıra ile yeni sıra arasındaki en uzun artan altdizi (LIS) dışında kalanlar taşınmış sayılır.
2. **Bölme adayları.** Eşleşmemiş her eski blok `o` için, yeni eşleşmemiş **ardışık** `n_i..n_j` (2 ≤ j−i+1 ≤ 4) pencereleri; `sim(o, concat(n_i..n_j)) ≥ THETA_SPLIT` ve her `n` için `len(n) ≥ 0.15·len(o)` ise aday; en yüksek `sim` kazanır → `BlockSplit`; yeniler §5.1 ile yeni soy, `parents=[o.version_id]`.
3. **Birleştirme adayları.** Simetrik: eski ardışık `o_i..o_j` ↔ yeni tek `n` → `BlockMerged`; `n` yeni soy, `parents = o_i..o_j`.
4. **Benzerlik ataması.** Kalan eski×yeni, aynı `block_type`:
   ```
   s = 0.70·sim(old,new) + 0.15·[heading_path eşit] + 0.15·max(0, 1 − |pos_old − pos_new| / max(n_old, n_new))
   ```
   `sim` = normalize edilmiş Levenshtein benzerliği (`strsim::normalized_levenshtein`), 0–1. Atama: Hungarian (`pathfinding::kuhn_munkres` veya `hungarian` crate), `s ≥ THETA` olanlar `BlockUpdated`, `parents=[old.version_id]`, soy devralınır.
5. **Artık kontrolü.** Adım 4'te eşleşen bir çiftin yanında kalan eşleşmemiş yeni ardışık blok(lar), eski blokla `sim(old, concat(new_matched, residual)) > sim(old, new_matched) + 0.10` ise adım 4 eşleşmesi geri alınır ve `BlockSplit` üretilir. (v1'in "büyük parça tek başına eşleşir" hatasının son savunması.)
6. **Kalanlar.** Yeni → `BlockCreated` (§5.1); eski → `BlockDeleted`.

Parametreler: `THETA` varsayılan 0.60, `THETA_SPLIT` varsayılan 0.70; ayar seti üzerinde `THETA ∈ {0.50..0.70 adım 0.05}` × `THETA_SPLIT ∈ {0.60, 0.70, 0.80}` taranır (T6), seçim `docs/progress/T06.md`'ye yazılır ve `core/src/lineage.rs` sabitlerine işlenir. Olaylara `meta.matcher_version = "v2"`.

Ayar seçim kuralı (önceden kayıtlı): `H1-S1 + (1 − H1-S2) + H1-S3 + H1-S4` toplamı maksimum; eşitlikte (0.60, 0.70)'e en yakın.

## 6. Chunk — türetilmiş

```rust
pub struct Chunk {
    pub chunk_id: String,        // blake3(version_id'ler "\n")
    pub doc_path: String,
    pub lineage_ids: Vec<String>,
    pub version_ids: Vec<String>,
    pub heading_path: Vec<String>,
    pub text: String,            // heading_path " > " ile başa, sonra bloklar "\n\n"
    pub wikilinks: Vec<WikiLink>,
    pub strategy: String,        // "fixed" | "structural"
    pub token_count: u32,        // tiktoken-rs cl100k_base
}
```
- `fixed`: tüm doküman metni 512 token pencere, 64 örtüşme, blok sınırı gözetmez; `lineage_ids` pencereye giren bloklar. Taban çizgisi.
- `structural`: aynı `heading_path` altında ardışık bloklar, 512 token'ı aşacak blok eklenmez; tek blok 512'yi aşarsa kendi başına chunk. Başlık değişince chunk kapanır.

Aynı `chunk_id` birden çok dokümanda çıkarsa (tekrar içerik) indekste **hepsi** tutulur, `chunk_id` yerine indeks anahtarı `(doc_path, chunk_id)` olur (v1'de ilk doküman tutuluyordu; bu bazı gerçekleri indeks dışında bırakmıştı).

## 7. Rust API yüzeyi (`core/src/lib.rs`) ve CLI

```rust
pub fn parse_document(text: &str, doc_path: &str) -> Result<(Vec<BlockDraft>, Option<String>)>;
pub fn normalize(text: &str, is_code: bool) -> String;
pub fn extract_wikilinks(text: &str) -> Vec<WikiLink>;
pub struct Vault { root: PathBuf }        // open(root), objects/snapshots/events erişimi
impl Vault {
    pub fn ingest(&mut self, doc_path: &str, text: &str, prov: Provenance) -> Result<IngestResult>; // parse→match→store→log
    pub fn snapshot(&self, doc_path: &str) -> Result<Option<Snapshot>>;
    pub fn block(&self, version_id: &str) -> Result<Block>;
    pub fn replay(&self, doc_path: &str) -> Result<Option<Snapshot>>;
    pub fn events_hash(&self) -> Result<String>;
    pub fn resolve_truth(&self, doc_path: &str, version_id: &str, k: usize) -> Result<String>; // → lineage_id, yoksa Err
}
pub fn chunk(snapshot: &Snapshot, vault: &Vault, strategy: Strategy) -> Result<Vec<Chunk>>;
pub struct IngestResult { snapshot: Snapshot, events: Vec<Event>, changed_lineages: Vec<String> }
```
CLI `mdk`: `mdk ingest <vault> <file...>`, `mdk ingest-dir <vault> <dir>` (dosya adı sırasıyla, deterministik), `mdk replay <vault> <doc>`, `mdk events-hash <vault>`, `mdk chunk <vault> <doc> --strategy`, `mdk resolve <vault> <doc> <version_id> <k>`. Tüm çıktılar JSON, `--pretty` ile girintili.

Python bağlaması (`bindings/`, modül `mdkernel_core`): aynı fonksiyonlar, dönüşler `dict`/`list`. Çekirdek `Result::Err` → Python `RuntimeError`.

## 8. İndeks (Python)
- LanceDB tablo `chunks_<strategy>`: `key (doc_path + "::" + chunk_id), doc_path, chunk_id, lineage_ids, heading_path, text, vector(384)`.
- BM25: aynı chunk'lar, tokenizasyon küçük harf + `[a-z0-9]+`.
- Doküman embedding: chunk vektörlerinin ortalaması, normalize.
- Graf (`networkx.Graph`, düğüm = `key`): `wikilink` (A, hedef D'nin tüm chunk'larına 1.0; `#başlık` varsa yalnız o başlık altı), `hierarchy` (ardışık 0.5; aynı üst başlık 0.3), `knn` (kosinüs top-5 × 0.3). Çakışan kenarlar toplanır, tavan 1.0. Diske `graph_<strategy>.gpickle`.

## 9. Arama stratejileri — `(query, k) -> [(key, score)]`
- `flat`: ANN kosinüs top-k (LanceDB).
- `hybrid`: `flat` top-50 + BM25 top-50 → RRF (`1/(60+rank)`) → top-k.
- `graph_flat`: tohum = `flat` top-5. **Ego alt grafı:** tohumlardan 2 sıçrama (networkx `ego_graph` birleşimi, en fazla 2000 düğüm; aşarsa kenar ağırlığına göre budanır). Bu alt grafta kişiselleştirilmiş PageRank: `scipy.sparse` ile güç iterasyonu, `alpha=0.85`, tohumlara eşit kütle, 50 iterasyon veya L1 fark < 1e-6. `score = ALPHA·cos(q, chunk) + (1−ALPHA)·ppr/ppr_max`. top-k.
- `graph_hybrid`: aynı, tohum = `hybrid` top-5; `score = ALPHA·rrf_norm + (1−ALPHA)·ppr_norm` (rrf_norm = RRF skoru / tohumların maks RRF; alt grafta RRF skoru olmayan düğümler için `cos(q, chunk)`·(maks RRF'nin kosinüs karşılığı) yerine basitçe `rrf_norm = 0.5·cos_norm`).
- `hier`: doküman embedding top-20 doküman → chunk'ları içinde `hybrid` → top-k. (v1 top-5 çok dardı.)

`ALPHA`: ayar setinde `{0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9}` taranır, `graph_hybrid`/`structural` için; seçim test öncesi sabitlenir. **Duyarlılık eğrisi** (tüm ALPHA değerlerinde ayar seti nDCG) rapora zorunlu.

Gecikme ölçümüne sorgu embedding'i dahil, tüm stratejilerde.

## 10. Sıcak katman (Python)
Bellekte `numpy` matris + anahtar listesi. `add(chunks)`, `search(q, k)` kaba kuvvet; `flat`/`hybrid` sıcak+soğuk birleştirir, aynı anahtar varsa sıcak kazanır; `compact()` LanceDB'ye yazar. `watcher.process(path)` → `Vault.ingest` → `changed_lineages` içeren chunk'ları yeniden üret → sıcağa ekle.

## 11. Değişmezler (test edilir)
- Aynı `normalized_text`+`block_type` → aynı `version_id`.
- Aynı vault, aynı ingest sırası → aynı `events.jsonl` baytları (H4-1).
- `replay` son snapshot ile `snapshot_id` eşit.
- Chunk yeniden üretimi hiçbir `lineage_id` değiştirmez; olay üretmez.
- Nesne deposuna aynı kimlikle farklı içerik yazmak `Err`.
- `resolve_truth` çözemezse `Err`, asla boş/None.
