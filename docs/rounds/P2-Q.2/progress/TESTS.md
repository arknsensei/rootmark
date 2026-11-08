# P2-Q.2 — Ön-kayıtlı testler (R0)

Her test gerçek `rootmark` ikilisiyle (`target/release/rootmark`, `cargo test -p rootmark-server --release`) kendi geçici vault'unda koşar. Bekleme koşulları zaman aşımlıdır; zaman aşımı başarısızlıktır. CI'da `make test` ile macOS ve `ubuntu-latest`'te (`ci.yml`, değişmeden). "Koştu" ile "geçti" ayrı sayılır: her test dosyası kendi vaka sayısını yazar.

## `server/tests/mcp_compat.rs` (R1, PQ1-1; DESIGN §3)
Vault: `spec/fixtures/mini_vault`'un `rootmark ingest-dir` kopyası; sunucu `rootmark serve --vault <v> --mcp stdio` (JSON-RPC, satır başına bir ileti). Her "çalışır" hücresinde şu adımlar:
1. el sıkışma (hücreye göre);
2. `tools/list` → 11 araç; sonucun sıralı anahtarlarla yeniden yazılmış JSON'u `server/tests/fixtures/p2q2/tools_list_0.3.1.json` ile **bayt eşit** (0.3.1'in yayımlanmış ikilisinden, `research/p2q2/mcp_list.py`, R1'in ilk commit'i);
3. `tools/call search` (`{"query": "espresso", "k": 3}`) → `isError` yok, `hits` dizisi;
4. `tools/call propose` (`mode: create`, yeni bir yol, `reason`) → `isError` yok; günlükte o yolun `BlockCreated` olayları `meta.provenance.author_kind = "Agent"`, `meta.provenance.source = "<clientInfo.name>/<clientInfo.version>"`;
5. akış kapanır, sunucu 0 ile çıkar.

| Kimlik | Hücre | Beklenen |
|---|---|---|
| M1 | G1: `initialize` 2025-06-18, `_meta` yok | çalışır |
| M2 | G2: `initialize` 2025-11-25, `_meta` yok | çalışır |
| M3 | G3: `initialize` 2026-07-28 istenir | müzakere edilen `2025-11-25`; çalışır |
| M4 | G4: `initialize` 2025-11-25; her isteğin `_meta`'sı tam, `protocolVersion = 2025-11-25` | çalışır |
| M5 | G5: `initialize` 2025-06-18; isteklerde yalnız `_meta.progressToken` | çalışır |
| M6 | G6: `initialize` 2025-11-25; `tools/list` `_meta.protocolVersion = 2026-07-28` (tam) | hata `-32022`; ardından meta'sız `tools/list` çalışır (oturum kilitlenmez) |
| M7 | G7: önce `tools/list` tam 2026-07-28 meta'sıyla (el sıkışmasız), sonra `initialize` 2025-11-25 | ilk istek `-32022`; sonra çalışır |
| M8 | C1: `r0_requests.json`'daki Claude Code 2.1.319 dizisinin birebir biçimi (`server/discover` dize kimlikli, tam 2026-07-28 `_meta`'sı: `protocolVersion`, `clientInfo {claude-code, 2.1.319}`, `clientCapabilities {elicitation, extensions, roots}`; `initialize` 2025-11-25 `capabilities {elicitation, roots}`; `notifications/initialized`) | keşif `-32022`; çalışır; aktör kaynağı `claude-code/2.1.319` |
| M9 | C2: `server/discover` yalnız `_meta.protocolVersion` ile, sonra `initialize` 2025-11-25 | keşif `-32602`; çalışır |
| M10 | determinizm: M8'in dizisi iki taze vault'ta | iki `events.jsonl` bayt eşit (`events_hash` eşit) |
| M11 | M1–M9 sonunda | `mcp_schemas.rs`'in denetimiyle aynı araç adları ve sırası (`tools::specs`) |

Yeniden üretim (kanıt): M8 `ROUND_START` ağacında (rmcp 3.5.0) **kalır** (`tools/list` `-32602`), R1 ağacında geçer; iki koşunun günlüğü `reports/rounds/P2-Q.2/r1/`.

## Gerçek Claude Code, başsız (R1 yerel ikili, R4 yayımlanmış 0.3.2; `research/p2q2/claude_probe.py`)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| CC1 | D-NOTES'un depo dışı kopyası (index kurulmuş), `claude -p … --mcp-config --strict-mcp-config`, sunucu kaydedicinin arkasında | araç listesi 11/11 (adlar 0.3.1 ile aynı) |
| CC2 | oturum `search` çağırır | başarılı (`isError` yok) |
| CC3 | oturum `propose` (`mode: create`, `agent-probe/claude-code.md`) çağırır | başarılı; günlükte `author_kind = Agent`, `source = claude-code/<sürüm>`; `blame` yolun bloklarını `proposal` gösterir (insan onayı bekliyor); varsayılan aramada yok |
| CC0 | aynı adımlar yayımlanmış `v0.3.1` ile (R0) | araç listesi alınamıyor (PQ1-1 yeniden üretimi) — **R0'da koştu: kaldı, beklendiği gibi** (`claude_code/r0_v0.3.1.json`) |

Kayıt: Claude Code sürümü, komut (yollar yer tutucuyla), iletilerin biçimi, hata ayıklama günlüğünün rootmark satırları (yollar maskeli), yeni olayların türü/yolu/aktörü. Token, içerik, host değeri yazılmaz.

## `server/tests/watch_hold.rs` (R2, PQ1-4; DESIGN §4)
Sunucu `rootmark serve --vault <v> --http 127.0.0.1:<port> --watch`, `[attestation] unattested = "unverified"`, `window_s = 1.0` (hold 1.5 s). Gecikme `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` ile (yalnız bu test dosyasının süreçleri; ürün varsayılanı değişmez).

| Kimlik | Adımlar | Beklenen |
|---|---|---|
| H1 | ≥ 20 tekrar, gecikme 3000 ms (> hold): insan notu `hNN.md` tasdikli yazılır, `canonical`; `hNN.md → hNN-r.md` yeniden adlandırılır; yeni yol tasdiksiz alınınca (`blame` 3 blok) `rootmark attest hNN-r.md --renamed-from hNN.md` (geç tasdik) | her tekrarda `attest` başarılı; yeni yolun blokları `canonical` (`LayerChanged`, `cause: late_attestation`); eski yolun her soyu **tam bir kez** `BlockDeleted` (`unattested_fs`) ve kaldırma geç tasdikten **sonra**; yeni yol için içerikli `SnapshotCommitted` bir kez |
| H2 | gecikme 3000 ms, zamanında tasdik (yeniden adlandırmadan hemen sonra `attest`) | `attest` başarılı; yeni yol `canonical`; eski yol bir kez `BlockDeleted`, yeni yolun alımından sonra |
| H3 | gecikme 3000 ms, tasdiksiz silme | eski yol bir kez `BlockDeleted` (`unattested_fs`); kaldırma alınır (bekleme sonsuz değil) |
| H4 | gecikmesiz; bekleyen bir silme varken başka bir nota 10 s boyunca 300 ms'de bir yazılır | kaldırma, silmeden sonra en geç 2 × hold + 1.5 s içinde alınır (akış bekletmez; DESIGN §4.3 (2)) |
| H5 | gecikme 3000 ms, aynı yeniden adlandırma dizisi (H1'in bir tekrarı) iki taze vault'ta | iki `events.jsonl` bayt eşit |

Yeniden üretim (kanıt): R2'nin ilk commit'i yalnız test yapılandırmasını (gecikme) ve testi ekler — o ağaçta H1 **kalır** (geç tasdik reddedilir, yeni yol `unverified`); düzeltme commit'inde geçer. İki koşunun günlüğü `reports/rounds/P2-Q.2/r2/`. macOS'ta yerel, Linux'ta Docker (`rust:1.92.0-bookworm`) ve CI (`make test`).

## Değişmeden geçmesi gerekenler
`mcp_schemas.rs`, `contract.rs`, `p1d1_server.rs`, `p1d1_criteria.rs` (MCP), `watch_remove.rs` (D1–D10), `watch_newdir.rs` (N1–N6), `serve_watch.rs`; `make conformance` 39/39; `scripts/mcp_inspector.sh` (Inspector 2.9.0: araç listesi ve 11 çağrı).
