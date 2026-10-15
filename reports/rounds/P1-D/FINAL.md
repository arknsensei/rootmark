# P1-D — Sonuç raporu

- RUN: `20261015T210419Z-a6b52fe` · ölçülen commit `a6b52fe74ddfecb85457ed9edde4f97528fc7e6f`
- Üretim: `python -m eval.report_p1d --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).

## Çıkış kriterleri (GOAL P1-D)

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| D-1 | Resmi MCP Inspector `tools/list`, şema doğrulaması (JSON Schema 2020-12), araç başına Inspector çağrısı | @modelcontextprotocol/inspector@2.9.0: şema 10/10, çağrı 10/10 | 9/9 (tablo 10 araç; ASSUMPTIONS D5-1) | geçti | `p1d/inspector/inspector.json` |
| D-2 | araç başına betikli uçtan uca test (`scripts/mcp_e2e.sh`, stdio JSON-RPC) | 10/10 araç; el sıkışması evet | 9/9 (10 araç) | geçti | `p1d/mcp_e2e/mcp_e2e.json` |
| D-3 | `propose` sonrası: düz md (regex `lineage\|version_id\|mdkernel` 0), bloklar `proposal`, varsayılan `search` bulmuyor, `include_proposal` etiketli buluyor | 4/4 | 4/4 | geçti | `p1d/mcp_e2e/mcp_e2e.json` |
| D-4 | MCP `promote` → `denied/human_only`; CLI `promote --actor human:<id>` → `Promoted`; ardından `search` buluyor | 3/3 | 3/3 | geçti | `p1d/mcp_e2e/mcp_e2e.json` |
| D-5 | her `search`/`read_block` için tam bir iz, `consumer` = MCP istemci adı/sürümü | 2/2 denetim (200 çağrı → 200 iz) | 200 → 200 | geçti | `p1d/mcp_e2e/mcp_e2e.json` |
| D-6 | `checkpoint` → `propose` + CLI terfi → `search --at <cp>` eski, `search` yeni | 2/2 | 2/2 | geçti | `p1d/mcp_e2e/mcp_e2e.json` |
| D-7 | HTTP: token'sız 401, yanlış token 401, doğru 200, `X-Consumer` yok 400, dış arayüze bağlanma reddi | 5/5 (diğer HTTP/servis vakaları 10/10) | 5/5 | geçti | `p1d/http/http_e2e.json` |
| D-8 | çift koşturma: MCP e2e çıktıları, vault `events.jsonl`/`traces.jsonl` (zaman damgası/token yok) | zincir içi iki koşturma eşit evet; A↔B 99985 alan, farklı 0; P1-D baytları 10/10 eşit, bütün baytlar eşit evet | eşit | geçti | `p1d/mcp_e2e/mcp_e2e.json, pair_compare.json` |
| D-9 | C1/C2/C3 uyumluluk değişmedi; P1-C.1 resmi RUN'ı ile P1-C.1 çift kümesi (kalite regresyonu dahil) | 99341 alan, farklı 0; uyumluluk C1 17/17, C2 9/9, C3 5/5, farklı test 0 | fark 0 | geçti | `regress_vs_p1c1.json` |
| D-10 | `serve --watch`: dışarıdan insan düzenlemesi → `search` (sıcak) buluyor; `canonical` | 20 düzenleme, p95 131.0 ms, en çok 622.8 ms; canonical/Human evet; SIGTERM compact evet, compact sırasında kaybolmuyor evet | p95 < 1000 ms, katman doğru | geçti | `p1d/d10.json, p1d/sigterm.json` |
| D-11 | kasıtlı: `propose` `../`, vault dışı yol, 10 MB üstü, olmayan `replace_section` başlığı | MCP e2e 4/4; tamper 4/4 | 4/4 hata | geçti | `p1d/mcp_e2e/mcp_e2e.json, p1d/tamper_p1d.json` |

## Ürün sahnesi, ajan tarafı (`p1d/scene/scene.json`, TASKS D6)

9/9 adım (scene-agent/1.0).

| Adım | Araç | Gösterdiği | Sonuç |
|---|---|---|---|
| 1 | status | `{"spec_version": "1.0.2", "level_claimed": 3, "policy": "human_only", "embedder": "minilm-l6-v2/e73168f9"}` | geçti |
| 2 | propose | `{"doc_path": "agent/scene.md", "plain_markdown": true, "blocks": [{"layer": "proposal", "author_kind": "Agent", "source": "scene-agent/1.0"}, {"layer": "proposal", "author_kind": "Agent", "source": "scene-agent/1.0"}]}` | geçti |
| 3 | search | `{"hits_on_doc": 0}` | geçti |
| 4 | search | `{"hits_on_doc": [{"layer": "proposal"}]}` | geçti |
| 5 | promote | `{"decision": "denied", "policy": "human_only", "events_written": 0}` | geçti |
| 6 | cli promote --actor human:editor | `[{"kind": "Promoted", "layer": "canonical", "actor": {"id": "editor", "kind": "human"}}, {"kind": "Promoted", "layer": "canonical", "actor": {"id": "editor", "kind": "human"}}]` | geçti |
| 7 | search | `{"hits_on_doc": [{"layer": "canonical"}]}` | geçti |
| 8 | trace | `{"traces": 3, "returned_from_doc": [0, 2, 2]}` | geçti |
| 9 | checkpoint | `{"checkpoint_id": "e3a3ac4954afb5591fcdf9f69803c84d", "event_seq": 115, "same_as_search_step_7": true}` | geçti |

## Kasıtlı bozmalar (`p1d/tamper_p1d.json`)

18/18 bozma reddedildi, 3/3 kontrol geçti.

- [schema] untouched Inspector list (control): beklendiği gibi
- [schema] inputSchema type misspelt: beklendiği gibi
- [schema] additionalProperties true: beklendiği gibi
- [schema] a tool removed: beklendiği gibi
- [schema] required names no property: beklendiği gibi
- [schema] property type not a JSON Schema type: beklendiği gibi
- [schema] schema differs from the server's: beklendiği gibi
- [token] token file mode 644 → start refused: beklendiği gibi
- [token] token file not 64 hex → start refused: beklendiği gibi
- [token] right token (control): beklendiği gibi
- [token] wrong token → 401: beklendiği gibi
- [token] another vault's token → 401: beklendiği gibi
- [token] token without Bearer → 401: beklendiği gibi
- [x-consumer] missing → 400: beklendiği gibi
- [x-consumer] empty → 400: beklendiği gibi
- [x-consumer] whitespace → 400: beklendiği gibi
- [d-11] propose ../escape.md → error, nothing written: beklendiği gibi
- [d-11] propose outside the vault → error, nothing written: beklendiği gibi
- [d-11] propose over 10 MB → error, nothing written: beklendiği gibi
- [d-11] replace_section with a missing heading → error, file unchanged: beklendiği gibi
- [d-11] valid propose (control): beklendiği gibi

## D-9 farkları (`regress_vs_p1c1.json`)

- (fark yok)

## Notlar

- D-1/D-2: GOAL "9 araç" der; GOAL tablosu 10 araç listeler (`status, search, read_block, history, blame, propose, promote, checkpoint, checkpoints, trace`). Hepsi ölçüldü (ASSUMPTIONS D5-1).
- D-4/D-6: insan terfisi CLI'dan (`mdk promote --actor human:<id>`); bir chunk bloklarının en düşük katmanını alır (spec §9), bu yüzden dokümanın bütün soyları terfi ettirildi (ASSUMPTIONS D5-2).
- Açık karar talepleri: K-D4-T (`replace_section` insan bloklarını terfisiz siler; CLI `mdk propose`; TOCTOU sınırı) — `docs/DECISIONS.md`.

## Karar kuralı (GOAL P1-D)

D-1..D-11 geçti → **P1-E başlar**.
