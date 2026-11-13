# P2-Q.4 — Ön-kayıtlı testler (R0)

Her test gerçek `rootmark` ikilisiyle (`cargo test -p rootmark-server --release`) kendi geçici vault'unda, sentetik içerikle koşar; sunucu alt süreçtir (`rootmark serve --vault <v> --http 127.0.0.1:0 --watch`, port stderr'den), sinyaller gerçek süreç sinyalidir. Bekleme koşulları zaman aşımlıdır; zaman aşımı başarısızlıktır. "Koştu" ile "geçti" ayrı sayılır (`repeat`, `ROOTMARK_RESTART_OUT`). Ortak vault V0 ve tarif P2-Q.3 TESTS'teki gibidir (`restartkit::base_vault`; `window_s = 1.0`, bekleme 1,5 s) — aksi yazılmadıkça.

**Tekrar:** test kodunun varsayılanı **20** (`ROOTMARK_RESTART_REPS`); CI `make test` ile 20. Yerelde **yük altında** (eşzamanlı `cargo test --workspace --release` döngüsü; makine ve yük `progress/` kaydında): `watch_restart.rs`'in A ve C kümeleri (A1/A2/C1, iki politika) ve `watch_restart_forced.rs` macOS ≥ 200, Linux (Docker `rust:1.92.0-trixie`, kök olmayan kullanıcı) ≥ 200; yeni testler (H, I, N) iki platformda ≥ 20.

## R1 — `server/tests/hot_tear_restart.rs` (PQ3-9'un nedeni; DESIGN §1, §2.4)
Vault V0, ama `window_s = 4.0` (bekleme 4,5 s: SIGKILL'den önce `sub/moved.md`'nin eki bitmiş olsun). Adımlar: sunucu hazır → sıcak katman dosyalarının boyutları kaydedilir (`hot/` yoksa "yok") → `n0.md` → `sub/moved.md` → `hot/snapshots.jsonl` `sub/moved.md`'nin satırını taşıyana dek beklenir → `n0.md`'nin `BlockDeleted`'ı yok (doğrulanır) → SIGKILL → sıcak katman, `index_head`'in ek sırasıyla (`records.jsonl`, `embedders.jsonl`, `vectors.f32`, `snapshots.jsonl`) noktanın durumuna kesilir → yeniden başlatma.

| Kimlik | Nokta (SIGKILL'in düştüğü yer) | Disk durumu |
|---|---|---|
| H0 | günlük yazıldı, hiç ek yok | dört dosya eklemeden önceki boyutunda (`hot/` yoksa silinir) |
| H1 | 1. ekten sonra | `records` yeni, öteki üçü önceki boyutunda |
| H2 | 2. ekten sonra | `records`, `embedders` yeni; `vectors`, `snapshots` önceki |
| H3 | 3. ekten sonra | yalnız `snapshots` önceki |
| H4 | 1. ekin ortası | `records.jsonl` eklenen satırın yarısına kesik; öteki üçü önceki |

Beklenen (her tekrar, H0–H4): yeniden başlangıçta stderr'de `index incomplete (` … `): rebuilt` satırı; uzlaştırma özeti `1 removals held`; `n0.md`'nin her canlı soyu tam bir kez `BlockDeleted` (aktör ve kaynak politikanınki); `blame n0.md` boş; `search zqr0` 200 ve yalnız `sub/moved.md`; `blame sub/moved.md` 3 blok; SIGTERM ile çıkış 0; çıkıştan sonra `.rootmark/index/` (sıcak katman yok) aynı günlükten temiz `rootmark index build` (vault kopyasında) ile bayt eşit; `index.old` yok. H0–H4 `canonical`; H2 ayrıca `unverified`.
Yeniden üretim (kanıt): H0–H4 R1'in ürün değişikliğinden **önce** (`ROUND_START` ağacı) koşulur ve kalır — H1–H4'te "timed out: n0.md's removal ingested" (ya da `search` 500), H0'da `search` 500; kayıt `reports/rounds/P2-Q.4/r1/`.

**Yük altında:** `watch_restart.rs` A1/A2/C1 (iki politika) macOS ≥ 200, Linux ≥ 200, 0 kalış; ayrıca `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs`, `watch_restart.rs`'in tamamı aynen geçer.

## R2 — `server/tests/index_install_kill.rs` (PQ3-7; DESIGN §3)
Enjeksiyon `ROOTMARK_TEST_ABORT_AT` ∈ {`install:before_swap`, `install:swapped`, `install:installed`}. Referans: aynı günlüğün vault kopyasında temiz `rootmark index build`'in `.rootmark/index/` baytları ve aynı sorguların sonuçları (`search` lexical ve hybrid: `doc_path`, `chunk_id`, skor).

| Kimlik | Kesilen kurulum | Adımlar | Beklenen (her tekrar) |
|---|---|---|---|
| I1 | `rootmark index build` | V0 (dizin kurulu) → V0'a bir not eklenir ve `ingest-dir` (dizin günlüğün gerisinde) → `ROOTMARK_TEST_ABORT_AT=<nokta> rootmark index build` (abort ile biter, çıkış sinyalli) → `serve` (değişkensiz) | başlangıç çıkışı 0; nokta `swapped`'da `restored` satırı, `installed`'da `removed the previous index` satırı; dizin günlüğün gerisindeyse `index incomplete … rebuilt` satırı (sessiz değil); `search` 200 ve sonuçlar referansla aynı; SIGTERM'den sonra dizin baytları referansla eşit; `index.old` ve `index.build` yok |
| I2 | kapanış sıkıştırması | V0 → `serve` (`ROOTMARK_TEST_ABORT_AT=<nokta>`) → yeni not yazılır, izleyici alır (sıcak katman dolu) → SIGTERM → sıkıştırmanın kurulumu abort → `serve` (değişkensiz) | aynı (eski dizin = soğuk + sıcak; yeni = sıkıştırılmış; ikisi de tam); `search` 200, sonuçlar referansla aynı; SIGTERM'den sonra baytlar referansla eşit |
| I3 | artık `index.old` | V0 → `.rootmark/index.old/` içine bir dosya yazılır → `serve` | satır `removed the previous index …`; `index.old` yok; dizin baytları değişmedi |
| I4 | eski sürümün yarım dizini | V0 → `.rootmark/index/lexical/` içinden bir dosya silinir (manifest kalır; PQ3-7'nin 0.3.3 belirtisi) → `serve` | `index incomplete (` … `): rebuilt`; `search` 200; baytlar referansla eşit |
| I5 | `auto_rebuild = false` | I4, tarifte `[index] auto_rebuild = false` | satır `index incomplete (…); [index] auto_rebuild = false: rebuild it with \`rootmark index build\``; dizin dokunulmadan kalır |
| I6 | enjeksiyon değişkeninin denetimi | `ROOTMARK_TEST_ABORT_AT=bogus rootmark index build` | çıkış hata (sıfır değil), ileti değişkeni anar; dizin değişmez. Değişken yokken `index build` ve kapanış bugünkü gibi (bütün öteki testler) |

I1 ve I2 her nokta için ≥ 20 tekrar (varsayılan 20), iki platform. Yeniden üretim: I4 `ROUND_START` ağacında kalır (`search` 500); I1/I2 enjeksiyon olmadan üretilemez — 0.3.3'te kanıt P2-Q.3'ün `r5` kaydı (`make_test_run1_failures.txt`) ve kod okuması (DESIGN §3.1).

## R3 — `server/tests/watch_noaccess.rs` (PQ3-6; DESIGN §4)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| N1 | sunucu kapalı → `chmod 000 V/sub` → başlat → bekleme × 2 | `sub/k.md`'nin blokları canlı (BlockDeleted yok); stderr'de `sub/k.md` için `not read` notu; özet `errors` ≥ 1, `0 removals held` |
| N2 | sunucu çalışırken `chmod 000 V/sub` → bekleme × 2 → `chmod 755` → bekleme × 2 | olay yok; `sub/k.md` canlı |
| N3 | N1'den sonra `chmod 755` → SIGTERM → başlat → bekleme × 2 | yeni olay 0 (`events.jsonl` bayt eşit, aramanın checkpoint'i hariç) |
| N4 | sunucu kapalı → `chmod 000 V/sub` → `rootmark ingest-dir V V` | `sub/k.md` kapatılmaz (BlockDeleted yok); stderr'de not ya da çekirdeğin okuma hatası (sessiz değil) |
Kök olarak koşulursa (izin yok sayılır) test "not a reproduction" hatasıyla kalır. Yeniden üretim: N1 ve N4 `ROUND_START` ağacında kalır (bloklar kapanır).

## R3 — `server/tests/watch_restart_forced.rs` (PQ3-4; DESIGN §5)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| C2 | C1'in adımları; ilk sunucu `ROOTMARK_WATCH_TEST_STOP_DELAY_MS=2000` ile | 0,5 s'de süreç her tekrarda yaşıyor → SIGKILL her tekrarda (`sigkill_sent` = tekrar sayısı, iki platform); yeniden başlatmadan sonra `n0.md` kapalı, tam bir kez `BlockDeleted`; aramada yok; günlük açılır |
İki politika. Ayar sayı değilse sunucu başlamaz (test: `ROOTMARK_WATCH_TEST_STOP_DELAY_MS=x` → çıkış hata).

**PQ3-8:** `restartkit` ve `watchkit` portu stderr'den okur; P2-Q.3'ün bütün testleri aynen (iddia değişmeden) geçer.

## Değişmeden geçmesi gerekenler (Q4-6)
`contract.rs`, `mcp_schemas.rs`, `mcp_compat.rs`, P2-Q.3'ün testleri (`watch_restart.rs`, `ingest_dir_remove.rs`) ve `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs`, `p1e_e0.rs` aynen geçer; MCP `tools/list` 0.3.3 ile bayt eşit; `make conformance` 39/39.

## Yayımlanmış 0.3.4 ile (R4 (f))
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| P1 | R1'in durumu (taşıma + SIGKILL + yeniden başlatma; doğal SIGKILL ve H1'in disk durumu) yayımlanmış ikiliyle, macOS ve Docker, ≥ 5 tekrar | kaldırma alındı, `search` 200 |
| P2 | R2'nin durumu (I4: eski sürümün yarım dizini; I1 enjeksiyonla) yayımlanmış ikiliyle, iki platform, ≥ 5 tekrar | `search` 200, açık ileti |
| P3 | gerçek Claude Code başsız oturumu ≥ 3 (`claude -p --mcp-config … --strict-mcp-config`, D-NOTES kopyası): oturum `rootmark serve --mcp stdio --watch`'ı başlatır; oturum sonunda bir not silinir; sonraki oturumda `search` 200 ve o not yok; Claude Code sürümü kayıtlı | hepsi |
| P4 | D-NOTES senaryosu (`research/p2q/driver.py`) Mac, Docker ve CI (`pilot.yml`) | `events_hash`, vektör dosyaları, dizin manifest'i Mac ↔ Docker ↔ CI ↔ 0.3.3 bayt eşit (yalnız DESIGN §7.1'deki sürüm dizgileri) |
