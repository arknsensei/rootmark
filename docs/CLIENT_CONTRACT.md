# mdk istemci sözleşmesi (P1-D.1, K-D1.5; P1-E/E0 K-E0.3, K-E0.4; P1-E.1 K-E1.2)

Durum: P1-E.1/E1.1. Bu belge P1-E istemcisinin (ve her MCP/HTTP istemcisinin) `mdk serve`'e bağımlılığının **tamamıdır**. Burada yazmayan bir davranışa istemci güvenmez. Sözleşmedeki her araç ve her hata kodu en az bir testle, her §1 ucu ve §3 durum satırı gerçek bir HTTP çağrısıyla eşleşir (`scripts/contract_check.sh`). Spec kavramları için `spec/SPEC.md`, güvenlik modeli için `docs/SECURITY.md`.

## 1. Taşımalar ve kanal → aktör (K-D1.3)
| Taşıma | Başlatma | Kimlik | Aktör |
|---|---|---|---|
| MCP stdio | `mdk serve --vault <v> --mcp stdio` | `initialize.params.clientInfo` → tüketici `<name>/<version>` (boşsa iz yazan araçlar, `propose` ve `promote` `invalid_arguments` ile reddeder) | daima `agent:<tüketici>` |
| HTTP | `mdk serve --vault <v> --http 127.0.0.1:<port>` (`:0` serbest port; gerçek port stderr'de `mdk serve: http listening on 127.0.0.1:<port>`) | `Authorization: Bearer <.mdkernel/server.token>` (zorunlu), `X-Consumer: <etiket>` (zorunlu, boş olamaz) | daima `human:<policy.toml [identity] human_id>` |

- `X-Consumer` yalnız iz etiketidir (`traces.jsonl` `consumer`) ve `propose`'un köken `source`'udur. Aktör türünü **değiştiremez**.
- HTTP'de **yazma uçları** (`propose`, `promote`, `checkpoint`) `[identity] human_id` ister. Yoksa 403 `denied` döner. Okuma ve iz yazan okuma (`search`, `read_block`) `human_id` olmadan çalışır.
- HTTP yalnız `127.0.0.1`'dedir. Uçlar: `POST /v1/<araç>` (gövde = araç argümanları, JSON nesnesi; boş gövde `{}`) ve `GET /v1/status`. Gövde sınırı `[server] max_propose_bytes + 1 MiB`'dır (üstü 413 `too_large`).
- **Tek süreç:** MCP ve HTTP aynı vault için tek bir `mdk serve` sürecinde birlikte açılmalıdır: `mdk serve --vault <v> --mcp stdio --http 127.0.0.1:<port>`. §5'teki sıra garantisi yalnız bir süreç içindir. Aynı vault'a ikinci bir `mdk serve` ya da CLI yazımı sıralanmaz ve kilitlenmez (`docs/SECURITY.md`).

## 2. Araçlar
Argüman şemaları MCP `tools/list`'tedir (JSON Schema 2020-12, `additionalProperties: false`). Bilinmeyen alan `invalid_arguments` döner. Yazan araçlar ve iz yazan araçlar vault başına tek FIFO kuyruğundan geçer (§5).

| Araç | Yazar mı | Argümanlar | Başarılı yanıt (ana alanlar) |
|---|---|---|---|
| `status` | hayır | — | `vault, spec_version, mdk_version, level_claimed, embedder, index {state, format, expected_format, auto_rebuild}, last_checkpoint, policy {promotion, edit, …}, server_policy {server, index, identity}` |
| `search` | iz | `query` (boş/yalnız boşluk değil), `k` 1–1000 (10), `mode` `hybrid`\|`lexical`\|`flat` (`hybrid`), `at` (checkpoint kimliği ya da adı; yoksa güncel durumun checkpoint'i), `include_proposal` (false) | `checkpoint_id, mode, include_proposal, query_hash, trace_id, hits[{doc_path, chunk_id, lineage_ids, layer, score}]` |
| `read_block` | iz | `doc_path`, `lineage_id`, `at` | `checkpoint_id, block {doc_path, block {version_id, lineage_id, block_type, text, heading_path, parents, provenance, layer}, inactive}, trace` |
| `history` | hayır | `lineage_id` | `lineage_id, layer, events[]` (silme/değiştirme olaylarında `meta.actor {kind, id}`, `meta.source`; 1.0.3) |
| `blame` | hayır | `doc_path`, `at` | `doc_path, at, blocks[{lineage_id, version_id, layer, layer_source, author_kind, source, seq_created, seq_last, inactive}]` |
| `propose` | evet | `doc_path`, `content`, `reason` (boş değil), `mode` `create`\|`append`\|`replace_section` (`create`), `heading` (yalnız `replace_section`) | `doc_path, mode, bytes, synced_external_events, edit_policy, actor, snapshot_id, events, chunks_embedded, blocks, events_hash` |
| `promote` | evet | `lineage_id`, `reason` | `decision` `promoted` (+ `event`, `layer`) ya da `denied` (+ `reason`, `actor`, `layer`); her ikisinde `policy`, `policy_version`, `lineage_id` |
| `checkpoint` | evet | `name` (isteğe bağlı) | `checkpoint_id, name, event_seq, docs` |
| `checkpoints` | hayır | — | `checkpoints[]` (oluşturma sırası; ad, adlandırma olayından sonra görünür) |
| `trace` | hayır | `since_seq` (0), `consumer` | `traces[{trace_id, checkpoint_id, consumer, query_hash, returned[{doc_path, lineage_id, version_id}], at_seq}]` |

### 2.1 `propose` ve düzenleme politikası (K-D1.1)
- Ajan metni dosyaya kimlik, işaret ya da meta veri eklenmeden yazılır. Satır sonları belgenin satır sonuna uydurulur (CRLF belgede CRLF), sonda tek bir satır sonu kalır, `append`'de önüne bir boş satır konur. Bloklar `author_kind: Agent`, `source` = tüketici olarak alınır ve `proposal` katmanındadır. Terfiye kadar varsayılan aramaya girmezler.
- `create` var olan bir dosyanın üzerine yazmaz (`exists`). `append` var olan belgenin sonuna bir boş satırdan sonra ekler. `replace_section`, metni (baştaki `#`'ler ve kapanış `#`'leri, çevre boşluklar atılmış hali) birebir `heading` olan **tek** ATX başlığının gövdesini, aynı ya da daha üst düzeydeki bir sonraki ATX başlığına kadar değiştirir (yoksa ya da birden çoksa `heading_not_found`). Setext başlıklar (`===`/`---` alt çizgili) bölüm sınırı değildir.
- Varsayılan politika `[edit] policy = "append_only"`. Kanal ne olursa olsun, şu yazımlar `denied` (403) ile reddedilir. Ajanın metni hiçbir yere yazılmaz: dosya, `proposals.jsonl` ve isteğin kendi olayları yazılmaz (istisna için sondaki dış düzenleme maddesine bakın):
  - canonical (ya da derived) bir bloğu değiştiren, silen, taşıyan, bölen, birleştiren ya da katmanını düşüren yazım;
  - yalnız canonical bloklardan oluşan bir arama birimini (chunk) değiştiren yazım;
  - var olan bir belgenin ön bilgisini (front matter) değiştiren yazım;
  - proposal olmayan bir blok içeren bölüme `replace_section`.
- **İstemci kuralı:** insan yazımı bir bölümün sonuna eklenen metin o bölümün chunk'ına katılır ve reddedilir. Eklenecek metin kendi başlığıyla başlamalıdır (`## Öneri: …`; §4.1 ajan yönergesi). Başlıksız böyle bir `append` `policy_violation` (422) ve `hint` ile döner (K-E0.4); başlıkla başlayan ama yine canonical içeriğe dokunan yazım `denied`'dır. Yalnız proposal bloklardan oluşan bölüm (ajanın kendi önerisi) `replace_section` ile düzeltilebilir.
- `[edit] policy = "unrestricted"` yalnız açıkça yazılırsa her yazıma izin verir. O zaman da silme ve değiştirme olayları aktörü taşır.
- Sunucunun dışında yapılmış, henüz alınmamış bir düzenleme varsa, politika kararından önce insan yazımı olarak alınır (`synced_external_events`). Ret durumunda da insanın yazdığı olarak `events.jsonl`'de kalır. Bu olaylar ajanın değil, insanındır.

### 2.2 `search` ve `include_proposal` (spec §9, K-C1.3, K-D1.8)
- Varsayılan yanıtta yalnız `canonical` sonuçlar vardır, `derived` hiçbir zaman dönmez.
- `include_proposal: true` iken:
  - `canonical` sonuçlar bayrak kapalıyken dönenle **bayt eşittir** (aynı chunk'lar, sıra, skor);
  - ardından `proposal` sonuçları gelir: ayrı bir geçişte, yalnız canonical istatistiklerle sıralanmış, `layer: "proposal"` etiketli, skoru ×0.5.
- **Yanıt uzunluğu (normatif):** katman başına en çok `k` hit, toplam en çok `2k`.
- Her `search` ve `read_block` tam bir tüketim izi yazar (`trace_id` yanıtta). Boş sorgu `empty_query` döner ve iz yazmaz.
- Aynı checkpoint + sorgu + kip + `k` → aynı sıralı sonuç (spec §10).

### 2.3 Checkpoint akışı (spec §8, K-D1.6)
- Bir durumun tek checkpoint'i vardır. Kimlik içerikten ve olay ofsetinden türetilir. Adsız `checkpoint {}` durumun var olan checkpoint'ini döndürür. O checkpoint adlıysa → `exists`.
- `at` verilmeden yapılan `search`/`read_block`, güncel durumun **adsız** checkpoint'ini oluşturur (yoksa).
- `checkpoint {name}` şöyle çalışır:
  - (a) durumun checkpoint'i yoksa adlı olarak oluşturur;
  - (b) durumun adsız checkpoint'i varsa ona ad verir (aynı kimlik);
  - (c) aynı ad yeniden verilirse aynı checkpoint'i döndürür.
- Başka bir checkpoint'in adı ya da adlı bir durum için başka bir ad → `exists`. Ad boş olamaz; başında ya da sonunda boşluk, denetim karakteri taşıyamaz; 32 hex biçiminde olamaz (`invalid_arguments`). İç boşluklara izin verilir.
- Önerilen akış: önce `search`/`read_block`, sonra `checkpoint {name}` (sıra fark etmez). Dünkü hale dönmek için `search {at: <ad>}` kullanılır.

### 2.4 `promote` (spec §7, K12/K16, K-D1.3)
- Çekirdek karar vermez, `policy.toml [promotion]` karar verir (varsayılan `human_only`).
- Bilinmeyen ya da canlı olmayan soy → `not_found` (politikadan önce, iki kanalda da).
- Politikanın reddi her kanalda bir **yanıttır, hata değildir**: `decision: "denied"` (HTTP 200), olay yazılmaz. MCP'den gelen istek (aktör `agent`) `human_only` altında daima böyle döner. HTTP'den gelen istek `[promotion] deny` altında böyle döner.
- HTTP'den gelen istek (`human_id` ayarlı), `human_only` altında → `decision: "promoted"` ve bir `Promoted` olayı (`meta.actor {kind: human, id}`).
- Politikanın izin verdiği bir istek `proposal` olmayan ya da budanmış bir soy içinse → `invalid_arguments` (HTTP 400).
- Bir chunk, bloklarının en düşük katmanını alır. Bir belgeyi aramada görünür kılmak için chunk'ın bütün soyları terfi ettirilir.

### 2.5 Yanıtların istemcinin dayandığı iç alanları (P1-E/E1)
§2 tablosunun ana alanlarının iç yapısı. Bunlar P1-D.1'in davranışıdır (yeni bir şey değil); istemci yalnız buradakilere dayanır ve her yanıtta denetler (eksik ya da yanlış tipte alan → istemcide `bad_response`, sessiz kabul yok). Burada yazmayan alanlar gelebilir; istemci onlara dayanmaz.
- Blok özeti (`blame.blocks[]`, `propose.blocks[]`): `lineage_id`, `version_id` (dize), `layer` (`canonical`\|`proposal`\|`derived`), `layer_source` (dize), `author_kind` (dize: `Human`, `Agent`, `Transcript`, `Synthetic`), `source` (dize, insan yazımında boş olabilir), `seq_created`, `seq_last` (tamsayı), `inactive` (boole). Belgedeki sırasıyla (snapshot sırası).
- Olay (`history.events[]`, `promote.event`): `seq` (tamsayı), `kind` (dize), `doc_path` (dize), `lineage_ids` (dize dizisi), `meta` (nesne; §4.2'deki `actor {kind, id}`, `source`; `Promoted`'da `reason`, `from`, `to`).
- Okunan blok (`read_block.block.block`): `version_id`, `lineage_id`, `block_type`, `text` (dize), `heading_path` (dize dizisi), `layer`, `provenance {author_kind, source}`.
- Hit (`search.hits[]`): `doc_path`, `chunk_id` (dize), `lineage_ids` (dize dizisi), `layer`, `score` (sayı).
- İz (`trace.traces[]`, `read_block.trace`): `trace_id`, `checkpoint_id`, `consumer`, `query_hash` (dize), `at_seq` (tamsayı), `returned[{doc_path, lineage_id, version_id}]`.
- Checkpoint (`checkpoint`, `checkpoints[]`, `status.last_checkpoint`): `checkpoint_id` (dize), `name` (dize ya da `null`), `event_seq` (tamsayı), `docs` (nesne: `doc_path` → snapshot kimliği). `status.last_checkpoint` checkpoint yoksa `null`.
- `status`: `spec_version`, `mdk_version` (dize), `level_claimed` (tamsayı), `embedder` (dize ya da `null`: diskteki indeks manifest'inin `embedder_id`'si; manifest yok ya da okunamıyorsa `null`), `index` (nesne, K-E1.2), `policy.promotion`, `policy.edit` (dize), `server_policy.identity.human_id` (dize ya da `null`).
- `status.index` (K-E1.2; P1-E/E0'daki `"absent"`/`"present"` dizesinin yerine): `state` (dize: `ok` — manifest bu uygulamanın biçiminde ve sözcüksel kaydında; `stale` — manifest okunur ama başka biçimde ya da sözcüksel kayıtta, eski bir `mdk`'nin indeksi; `missing` — indeks dizini ya da manifest yok; `damaged` — manifest okunamıyor ya da çözülemiyor), `format` (dize ya da `null`: diskteki manifest'in `format`'ı), `expected_format` (dize: bu `mdk`'nin biçimi), `auto_rebuild` (boole: vault politikasının `[index] auto_rebuild`'i). `status` yalnız manifest'i okur: dosyaları manifest'le uyuşmayan bir indeks `ok` görünür ve indeks isteyen çağrılarda `internal` döner (§3). `stale`, `missing` ve `damaged`'da indeks isteyen çağrılar `index_stale` döner; yeniden kurma yolu `mdk index build <vault>` (ya da `stale`/`missing`'de `auto_rebuild = true` ile `mdk serve`'i yeniden başlatmak; `damaged` manifest'in üstüne kendiliğinden kurulmaz).

## 3. Hata kodları (K-D1.5)
Başarısız bir araç çağrısı şu biçimlerde döner:
- **MCP:** `result.isError: true`, `content[0].text` = `"error: <mesaj>"` ve `structuredContent = {"error": {"code": "<kod>", "message": "<mesaj>"}}`.
- **HTTP:** durum kodu aşağıdaki tablodan, gövde `{"error": {"code", "message"}}`.
- Bazı hatalar ayrıca `"hint"` taşır (`{"error": {"code", "message", "hint"}}`): çağıranın ne yapabileceği, kararlı bir dizge. Şimdilik tek örnek: başlıksız `append` → `"hint": "start the appended text with a heading"` (K-E0.4). İstemci hint'i kullanıcıya gösterir; `hint` yoksa alan yoktur.

İstemci yalnız `code`'a göre karar verir. `message` insanlar içindir ve değişebilir.

| Kod | HTTP | Anlam |
|---|---|---|
| `invalid_arguments` | 400 | Şemaya ya da kurala uymayan argüman (tip, bilinmeyen alan, `k` aralığı, kip, `.md` olmayan ad, boş `reason`, `heading` kullanımı, tüketici kimliği yok). HTTP'de `X-Consumer` eksik/boş, gövde JSON nesnesi değil ve yanlış yöntem (405) de bu koddur. |
| `empty_query` | 400 | Boş ya da yalnız boşluk sorgusu. |
| `path_outside_vault` | 400 | Vault ya da belgeleri dışına çıkan yol: `..`, mutlak, gizli bileşen (`.mdkernel/`), sembolik bağ. |
| `denied` | 403 | Politika ya da kanal reddi: `append_only` düzenleme politikası, `human_id`'siz HTTP yazımı. 401 (token eksik/yanlış) da `denied` kodunu taşır. |
| `not_found` | 404 | Bilinmeyen soy, belge, checkpoint, araç ya da HTTP yolu. `append`/`replace_section` için var olmayan belge. |
| `exists` | 409 | Yaratılacak şey var: `create` ile var olan belge; `doc_path`'i yalnız harf ya da Unicode biçimiyle var olan bir belgeden ayrılan yol; alınmış checkpoint adı; adlı durum için başka bir ad ya da adsız istek. |
| `too_large` | 413 | `content` `[server] max_propose_bytes`'ı aşıyor ya da HTTP gövdesi sınırı aşıyor. |
| `heading_not_found` | 422 | `replace_section` başlığı belgede yok ya da birden fazla. |
| `policy_violation` | 422 | Vault'un `policy.toml`'u geçersiz; ya da ajanın insan bloklarına `append`'i kendi başlığıyla başlamadığı için `append_only` altında reddedildi (K-E0.4; `hint` taşır). |
| `index_stale` | 503 | Vault'un indeksi eksik, başka bir biçimde (eski `mdk`'nin yazdığı indeks) ya da manifest'i okunamıyor (K-E1.2): yeniden kurulmalı (`mdk index build`, ya da eksik/eski biçimde `[index] auto_rebuild = true` ile `mdk serve`'i yeniden başlatmak; K-E0.3). Kurtarılabilir bir durumdur, `internal` değildir. Yalnız indeks isteyen çağrılar döner; `status` hiçbir indeks durumunda dönmez. |
| `internal` | 500 | Sunucu ya da vault hatası (ör. manifest'le uyuşmayan indeks dosyası). Raporlanması gereken bir hatadır, sessiz başarı değildir. |

İstisnalar:
- MCP'de bilinmeyen araç bir JSON-RPC hatasıdır (`-32602`). HTTP'de 404 `not_found`'dur.
- `mdk serve` başlarken indeks yoksa kurar; başka biçimdeyse `[index] auto_rebuild = true` (varsayılan) ile yeniden kurar. `auto_rebuild = false` ise indeksi olduğu gibi bırakır, stderr'e yazar ve indeks isteyen çağrılar (`search`, indeks taşıyan vault'ta `propose`) `index_stale` döner. Okunamayan manifest'i (`damaged`) hiçbir ayarda kendiliğinden yeniden kurmaz; indeks isteyen çağrılar yine `index_stale` döner. Salt okunur çağrılar (`blame`, `history`, `checkpoints`, `trace`, …) indekssiz de çalışır (K-E0.3); `status` her indeks durumunda 200 döner ve durumu `index.state`'te bildirir (K-E1.2).
- Politikanın `promote` reddi her kanalda yanıttır, hata değildir (§2.4).

## 4. Olaylar, köken ve ajan yönergesi

### 4.1 Ajan yönergesi (K-E0.4, K-D1.1)
- Ajan bir insan belgesine (insan yazımı, `canonical` bloklar içeren belgeye) `append` ile yazarken metni **yeni bir başlıkla** (`## …`) başlatır. Böylece metin kendi bölümünü ve kendi chunk'ını açar, hiçbir canonical bloğa ya da canonical arama birimine dokunmaz ve `append_only` altında kabul edilir.
- Başlıksız bir `append` insan bölümünün chunk'ına katılırsa reddedilir: `policy_violation` + `hint: "start the appended text with a heading"`. Hiçbir şey yazılmaz.
- Ajan kendi önerisini (yalnız proposal bloklardan oluşan bölüm) `replace_section` ile düzeltir. İnsan bölümünü değiştirmek ya da silmek `denied`'dır; bunun yolu insanın kendisidir.
- MCP `propose` aracının açıklaması bu yönergeyi söyler (`tools/list`).

### 4.2 Olaylar ve köken (spec §4.3, K-D1.2)
- `history` çıktısındaki silme ve değiştirme olayları (`BlockDeleted`, `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged`) sunucu üzerinden yazıldıysa `meta.actor {kind, id}` ve `meta.source` taşır. Aktör kanaldan gelir: `agent:<istemci>`, `human:<human_id>`; izleyicinin aldığı dış düzenlemeler için `human:<human_id>` ya da `human:unidentified`.
- 1.0.3 öncesi yazılmış olaylar ve `mdk ingest`/`ingest-dir` (düşük düzey çekirdek yolu) aktör taşımaz. İstemci bunu "aktör kaydedilmemiş" olarak gösterir, aktör uydurmaz.

## 5. Sıra ve determinizm (K-D1.4, spec §10)
- Yazan ve iz yazan çağrılar (`search`, `read_block`, `propose`, `promote`, `checkpoint`; ayrıca izleyici ve sıkıştırma) vault başına tek FIFO kuyruğunda, **geliş sırasıyla**, tek tek işlenir. Kuyruk tek `mdk serve` sürecinindir (§1). MCP'de geliş sırası isteğin stdio'dan okunduğu sıradır. HTTP'de isteğin kabul edildiği andır. MCP ve HTTP aynı kuyruğu paylaşır.
- Boru hatlı bir istemci (yanıt beklemeden gönderen), aynı istek dizisiyle kilit adımlı bir istemciyle aynı `events.jsonl`, `traces.jsonl` ve md dosyalarını üretir. Yanıtlar sırasız gelebilir; istemci onları `id` ile eşler.
- İptal edilen bir çağrı (`notifications/cancelled`) yine geliş yerinde koşar. İptal bir yazmayı ne geri alır ne de taşır.
- Salt okunur çağrılar (`status`, `history`, `blame`, `checkpoints`, `trace`) birbirleriyle paralel çalışır, bir yazar çalışırken bekler. Bir okuma, kendinden önce gönderilmiş ama henüz işlenmemiş bir yazmayı görmeyebilir. Bir yazmanın sonucunu görmek isteyen istemci onun yanıtını bekler.

## 6. Sürüm
`status.spec_version` (spec sürümü, `docs/SPEC_EDITS_CRITERION.md`) ve `status.mdk_version`. Bu sözleşme P1-D.1'in davranışı, P1-E/E0'ın iki eki (K-E0.3 `index_stale`, K-E0.4 `hint` ve ajan yönergesi) ve P1-E.1'in `status` eki (K-E1.2: `status.index` nesnesi, `status` indeksten bağımsız, okunamayan manifest `index_stale`) birliktedir. Değişiklikleri bir karar kaydıyla (`docs/DECISIONS.md`) ve bu belgenin güncellenmesiyle olur.
