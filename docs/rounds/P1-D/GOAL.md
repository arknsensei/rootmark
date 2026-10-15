# P1-D — Sunucu ve MCP

Başlangıç: P1-C doğrulandı (`p1c-final`). D0–D2 hemen başlar; **D3 (search aracı) ve sonrası P1-C.1 doğrulamasını bekler** (`p1c1-final`).

## Amaç
`server/` crate'i: `mdk` ikilisi buraya taşınır; MCP sunucusu (resmi Rust SDK `rmcp`, stdio taşıma; HTTP taşıma isteğe bağlı), HTTP API (`axum`, yerel, token'lı), dosya izleyici servisi, politika yükleme, tüketici kimliği. Spec'e yeni bölüm eklenmez; var olan primitifler dışarı açılır. İlk ürün sahnesinin ajan tarafı bu turda çalışır hale gelir.

## Kapsam
### MCP araçları (`server/src/mcp/`)
| Araç | Girdi | Davranış | Yazma |
|---|---|---|---|
| `status` | — | vault yolu, spec/`mdk` sürümü, `level_claimed`, embedder, son checkpoint, aktif politika | hayır |
| `search` | `query, k?, mode?, at?, include_proposal?` | `index` API; her çağrı iz yazar, `consumer` = MCP istemci adı + sürüm (handshake'ten) | iz |
| `read_block` | `doc_path, lineage_id, at?` | blok sürümü + katman + köken; iz yazar | iz |
| `history` | `lineage_id` | sürüm zinciri, olaylar | hayır |
| `blame` | `doc_path, at?` | blok sırasıyla köken/katman | hayır |
| `propose` | `doc_path, content, reason` | **çekirdeğin markdown yazdığı tek yol:** dosya vault'a düz md olarak yazılır (varsa üzerine değil; `propose` yeni doküman ya da mevcut dokümana **ek** olarak, `--mode append|replace_section <heading>`), ardından `author_kind=agent` ile ingest → bloklar `proposal`. Dosyaya kimlik/meta veri yazılmaz. | md + olay |
| `promote` | `lineage_id, reason` | `PromotionPolicy` hook'u ile; varsayılan `HumanOnly` → MCP tüketicisi `agent` olduğundan **Deny** döner ve bu bir hata değil, yanıt (`decision: denied, policy: human_only`) | hayır (deny) |
| `checkpoint` | `name?` | oluşturur, kimliği döner | checkpoint |
| `checkpoints` | — | liste | hayır |
| `trace` | `since_seq?, consumer?` | iz listesi | hayır |
Araç şemaları JSON Schema ile; her araç için `description` spec terimleriyle.

### HTTP API
Aynı araçlar `POST /v1/<tool>`; `GET /v1/status`; yalnız `127.0.0.1`; `Authorization: Bearer <token>` (`.mdkernel/server.token`, ilk başlatmada üretilir, dosya izni 0600); token'sız → 401. İstemciler (P1-E) bunu kullanır.

### Servis
`mdk serve --vault <path> [--mcp stdio] [--http 127.0.0.1:PORT] [--watch]`: izleyici `index.watch` ile sıcak katmanı günceller; `compact` zamanlayıcısı (politika: `index.compact_every_s`); kapanışta compact. Tek süreç, tek vault (çok vault faz 2).

### Politika ve kimlik
`policy.toml` genişler: `[promotion] policy`, `[server] http_port, token_rotate_days`, `[index] compact_every_s`. Tüketici kimliği her izde; HTTP'de `X-Consumer` zorunlu, boşsa 400.

### CLI taşıma
`core/src/bin/mdk.rs` → `server/src/bin/mdk.rs`; alt komutlar değişmez; `core` artık ikili üretmez.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| D-1 | MCP Inspector (resmi) ile `tools/list` ve 9 aracın şema doğrulaması | 9/9 geçerli |
| D-2 | Her araç için betikli uçtan uca test (`scripts/mcp_e2e.sh`, stdio üzerinden JSON-RPC) | 9/9 |
| D-3 | `propose` sonrası: md dosyası düz markdown (kimlik/meta yok: regex `lineage|version_id|mdkernel` dosyada 0), bloklar `proposal`, `search` varsayılanda bulmuyor, `include_proposal` ile etiketli buluyor | 4/4 |
| D-4 | `promote` MCP'den → `denied/human_only`; aynı soy CLI `promote --actor human:<id>` ile → `Promoted`; ardından `search` buluyor | 3/3 |
| D-5 | Her `search`/`read_block` çağrısı için tam bir iz; `consumer` MCP istemci adı | 200 çağrı → 200 iz |
| D-6 | `checkpoint` → sonra `propose` + CLI promote → `search --at <cp>` eskiyi, `search` yeniyi veriyor | 2/2 |
| D-7 | HTTP: token'sız 401, yanlış token 401, doğru 200; `X-Consumer` yoksa 400; dış arayüze bağlanma reddi | 5/5 |
| D-8 | Çift koşturma: MCP e2e çıktılarında (zaman damgası ve token hariç) fark 0; vault `events.jsonl`/`traces.jsonl` bayt eşit | eşit |
| D-9 | C1/C2/C3 uyumluluk değişmedi; P1-C.1 kalite regresyonu | fark 0 |
| D-10 | `serve --watch`: dışarıdan md düzenlemesi → 1 s içinde `search` (sıcak) buluyor; `author_kind=human` → `canonical` | p95 < 1000 ms, katman doğru |
| D-11 | Kasıtlı: `propose` ile `../` yolu, vault dışı yol, 10 MB üstü içerik, `replace_section` olmayan başlık | 4/4 hata |

## Karar kuralı
D-1..D-11 geçti → P1-E başlar. D-3/D-4/D-5 kaldı → yönetişim ihlali; P1-E başlamaz. D-8 kaldı → determinizm; P1-E başlamaz. D-1/D-7 kaldı → P1-E başlar, 0.1.0 bekler.

## Yapılmayacaklar
İstemci kodu; çok vault; uzak erişim/TLS; örtük onay; WASM; yeni spec bölümü; `propose` dışında md yazan yol.
