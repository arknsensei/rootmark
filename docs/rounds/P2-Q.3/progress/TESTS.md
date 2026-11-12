# P2-Q.3 — Ön-kayıtlı testler (R0)

Her test gerçek `rootmark` ikilisiyle (`target/release/rootmark`; `cargo test -p rootmark-server --release`) kendi geçici vault'unda, sentetik içerikle koşar; sunucu alt süreçtir (`rootmark serve --vault <v> --http 127.0.0.1:<port> --watch`), sinyaller gerçek süreç sinyalidir (`kill -<SIG> <pid>`; SIGKILL için `kill -KILL`). Bekleme koşulları zaman aşımlıdır; zaman aşımı başarısızlıktır. "Koştu" ile "geçti" ayrı sayılır: her durum kendi tekrar sayısını ve geçen tekrar sayısını yazar (`ROOTMARK_RESTART_OUT` verilirse JSON'a).

**Tekrar:** her durum **20** tekrar (test kodunun varsayılanı; `ROOTMARK_RESTART_REPS` ile değiştirilebilir, değer sayı değilse hata). macOS (yerel, arm64) ve Linux'ta 20 tekrar. CI'da `make test` varsayılanı koşar; dosyanın CI duvar süresi bir işte 15 dakikayı aşarsa CI'daki tekrar sayısı azaltılır — bu, değişiklikten **önce** bu dosyaya bir ekle (gerekçe ve yeni sayı) yazılır ve Linux'ta 20 tekrar Docker'da (`rust` imajı, yerel) ayrıca koşar. CI yapılandırması değişmez (DESIGN §8.3); sayı ortam değişkeniyle değil test kodunun CI algısıyla (`CI` ortam değişkeni) seçilirse o da ekte yazılır.

**Ortak vault (V0):** 6 sentetik not (`n0.md`…`n4.md`, `sub/k.md`), her biri 2–3 paragraf ve notun kendine özgü bir sözcüğü (`zqr<i>`); tarif `base_recipe()` + `[identity] human_id = "tester"` + `[attestation] unattested = <politika>`, `window_s = 1.0` (bekleme 1,5 s); önce `rootmark ingest-dir V V` (sunucu kapalıyken), sonra sunucu. "Uzlaştırma bitti" = stderr'deki `watch: start: reconciled …` satırı (DESIGN §2.1). Arama: `POST /v1/search` (`mode: lexical`, `include_proposal: true`), blame: `POST /v1/blame`.

## `server/tests/watch_restart.rs` (R1; DESIGN §2–§3)

| Kimlik | Durum | Adımlar | Beklenen (her tekrar) |
|---|---|---|---|
| A1 | (a) SIGKILL, kaldırma beklemesi içinde: **silme** | sunucu hazır (uzlaştırma bitti) → `n0.md` silinir → 0,3 s sonra (bekleme 1,5 s'den önce) SIGKILL → kill anında `n0.md`'nin `BlockDeleted`'ı yok (hata yeniden üretildi: aksi tekrar geçersiz sayılır ve yazılır) → aynı vault'ta yeniden başlat → uzlaştırma biter + bekleme | `n0.md`'nin her canlı soyu tam bir kez `BlockDeleted`; `blame n0.md` boş; `zqr0` aramada yok; özet satırında `1 removals held` |
| A2 | (a) SIGKILL, kaldırma beklemesi içinde: **taşıma** | `n0.md` → `sub/moved.md` (`rename`) → 0,3 s sonra SIGKILL → yeniden başlat | `n0.md`'nin soyları kapalı (tek `BlockDeleted`), `blame n0.md` boş; `sub/moved.md` canlı (blame dolu); `zqr0` aramada yalnız `sub/moved.md` |
| B1 | (b) SIGTERM ~0,5 s sonra | `n0.md` silinir → 0,5 s sonra SIGTERM → çıkış 0 → yeniden başlat | kaldırma SIGTERM'de alındı (yeniden başlatmadan **önce** `BlockDeleted` var); yeniden başlatma yeni olay yazmaz (günlük bayt eşit) |
| C1 | (c) gerçek istemci dizisi | `n0.md` silinir → 0,1 s sonra SIGINT → hemen SIGTERM → 0,5 s sonra süreç hâlâ yaşıyorsa SIGKILL → yeniden başlat | yeniden başlatmadan sonra `n0.md` kapalı, tam bir kez `BlockDeleted`; aramada yok; günlük açılır (yırtık satır yok); hangi yolun aldığı (durdurma ya da başlangıç) kayda yazılır |
| D1 | (d) kapalıyken **düzenle** | SIGTERM → `n1.md`'nin bir paragrafı değiştirilir → ≥ 1 s aralık (D0F-6) → başlat | uzlaştırma `n1.md`'yi aldı (özet `1 changed`); `blame n1.md` diskteki paragraflarla aynı; yeni sözcük aramada `n1.md`'yi döndürür, silinen sözcük döndürmez |
| D2 | (d) kapalıyken **sil** | SIGTERM → `n2.md` silinir → aralık → başlat → bekleme | `n2.md` kapalı (tam bir kez `BlockDeleted`), aramada yok |
| D3 | (d) kapalıyken **ekle** | SIGTERM → `new.md` yazılır → aralık → başlat | `new.md` canlı, blame paragraflarla aynı; sözcüğü aramada |
| D4 | (d) **birlikte** | SIGTERM → `n1.md` düzenlenir, `n2.md` silinir, `n3.md` → `sub/n3.md` taşınır, `new.md` eklenir → aralık → başlat → bekleme | günlüğün canlı dokümanları = diskteki `*.md` kümesi; her dokümanın blame'i dosyasıyla aynı (blok sayısı ve metin); arama diskle tutarlı; olay sırası: önce değişiklikler yolun bayt sırasıyla, sonra kaldırmalar bayt sırasıyla |
| E1 | (e) değişiklik yokken yeniden başlatma | SIGTERM → `events.jsonl` bayt kopyası → başlat → uzlaştırma biter + 2 × bekleme | özet `0 changed, 0 removals held, 0 errors`; `events.jsonl` bayt eşit; yeni olay 0 |
| E2 | (e) üst üste iki boş açılış | E1'den sonra bir kez daha SIGTERM + başlat | yine bayt eşit |
| F1 | (f) politika `unverified` | D4'ün değişiklikleri | düzenlenen/eklenen/taşınan yolların katman veren olayları `meta.layer = "unverified"`, değişiklik ve silme olaylarında `meta.actor = {kind: unattested_fs, id: fs}`, `meta.source = unattested_fs`; değişen soy `LayerChanged` ile `unverified`; blame katmanı `unverified`; `BlockDeleted` `unattested_fs` |
| F2 | (f) politika `canonical` | D4'ün değişiklikleri | katman `canonical`; değişiklik ve silme olaylarının aktörü `{kind: human, id: tester}`, kaynağı `rootmark-serve-watch`; `meta.layer` yok |
| G1 | determinizm | D4 iki taze vault'ta aynı adımlarla | iki `events.jsonl` bayt eşit |
| G2 | yarış (D0F-6) | SIGTERM → `n1.md` düzenlenir ve **aralıksız** hemen başlatılır | `n1.md` tam bir kez alınır: o değişikliğin olayları bir kez (yinelenen `SnapshotCommitted` yok); sonuç D1 ile aynı |

A ve C durumları her iki politikada koşar (A1/A2/C1 `canonical` ve `unverified`; `unverified`'de `BlockDeleted` aktörü `unattested_fs`). B1, D1–D4, E1–E2, G2 `canonical`'da; F1/F2 ayrı. Her durum 20 tekrar (G1: 20 çift).

**Hatanın yeniden üretimi (kanıt):** A1/A2/D1–D3 `ROUND_START` ağacında (uzlaştırmasız) kalır: R1'in ilk test commit'i ürün değişikliğinden önce yerelde koşulur ve kalışı `reports/rounds/P2-Q.3/r1/` altına kaydedilir.

## `server/tests/ingest_dir_remove.rs` (R2; DESIGN §4)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| I1 | V0 `ingest-dir V V` → `n0.md` silinir → `ingest-dir V V` | `n0.md`'nin her canlı soyu tam bir kez `BlockDeleted`; `replay n0.md`'nin bloğu yok; stderr'de kaldırma satırı; JSON `{ingested, events_hash}` biçiminde |
| I2 | politika `canonical` (tarif `[identity] human_id = "tester"`) ile I1 | `BlockDeleted` aktörü `human:tester`, kaynak `rootmark-ingest-dir` |
| I3 | politika `unverified` ile I1 | aktör `unattested_fs`/`fs`, kaynak `unattested_fs` |
| I4 | alt dizin kökü: V0'da `sub/k.md` ve `n1.md` var; `n1.md` silinir; `ingest-dir V V/sub` | `n1.md` (kökün dışı) **aynen** canlı (olayı yok); `sub/k.md`'nin dosyası varken kapatılmaz |
| I5 | alt dizin kökü: `sub/k.md` silinir; `ingest-dir V V/sub` | `sub/k.md` kapatılır; kök dışındaki dokümanların olayı yok |
| I6 | değişiklik yokken `ingest-dir V V` | yeni olay 0 (`events.jsonl` bayt eşit) |
| I7 | vault dışında bir dizin: `ingest-dir V <dış>` | kaldırma yok (V'nin dokümanları aynen); bugünkü davranış |
| I8 | gizli yol: `.hidden/x.md` (günlükte yok) ve sembolik bağ | gizli yollar kaldırılmaz; sembolik bağla gelen doküman kaldırılmaz |
| I9 | determinizm | I1 iki taze vault'ta | `events_hash` eşit |

Yeniden üretim: I1 `ROUND_START` ağacında kalır (kaldırma yok).

## Değişmeden geçmesi gerekenler (Q3-2, Q3-4)
`watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs`, `contract.rs`, `mcp_schemas.rs`, `mcp_compat.rs` aynen geçer; `make test`, `make lint`, `make conformance` (39/39), `make verify FAST=1` temiz. MCP `tools/list` 0.3.2 ile bayt eşit (`mcp_compat.rs`'in sabit dosyası `server/tests/fixtures/p2q2/` ile).

## Performans (DESIGN §7)
| Kimlik | Ölçüm | Eşik |
|---|---|---|
| P1 | D-NOTES kopyası (400 not), `ingest-dir` + `index build`, `serve --watch` başlangıcı: özet satırının süresi, değişiklik yokken | Mac arm64 ≤ 10 s; Docker bilgi |
| P2 | aynı vault'ta kapalıyken bir not düzenlenip yeniden başlatma | bilgi (süre ve `1 changed`) |

## Yayımlanmış 0.3.3 ile (R4 (f))
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| R1 | D-0'ın sondaları yayımlanmış 0.3.3 ikilisiyle: `research/d0/pq2_1_probe.py` (SIGKILL/SIGTERM × sil/taşı), macOS ve Docker (Linux), her durum ≥ 5 tekrar | SIGKILL durumları **alındı** (yeniden başlatmadan sonra `BlockDeleted` var, aramada yok); SIGTERM kontrolü alındı |
| R2 | `research/d0/offline_probe.py` (kapalıyken düzenle/sil/ekle; `ingest-dir`), iki platform, ≥ 3 tekrar | yeniden başlatmada düzenle/sil/ekle alındı 3/3; `ingest-dir` silme alındı 3/3 |
| R3 | gerçek istemci dizisi (C1) yayımlanmış ikiliyle, iki platform, ≥ 5 tekrar | yeniden başlatmadan sonra kapalı |
| R4 | gerçek Claude Code başsız oturumu (`claude -p --mcp-config … --strict-mcp-config`, D-NOTES kopyası): oturum `rootmark serve --mcp stdio --watch`'ı başlatır; oturumun son saniyesinde (yanıt verilmeden hemen önce, oturum kapanırken) bir not silinir; sonraki oturumda `search` o notun sözcüğünü döndürmez; Claude Code sürümü kayıtlı | sonraki oturumda yok; `BlockDeleted` var |
| R5 | D-NOTES senaryosu (`research/p2q/driver.py`) Mac, Docker ve CI (`pilot.yml`, bir koşu) | `events_hash`, vektör dosyaları, indeks manifest'i Mac ↔ Docker ↔ CI ve 0.3.2 kayıtlarıyla bayt eşit (yalnız DESIGN §8.1'deki sürüm dizgileri) |
