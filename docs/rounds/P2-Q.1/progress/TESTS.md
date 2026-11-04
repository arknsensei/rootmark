# P2-Q.1 — Ön-kayıtlı testler (R0)

Her test kendi vault'unda gerçek `rootmark serve --watch` (ve gerektiğinde `--http 127.0.0.1:0`) süreciyle koşar; ikili `target/release/rootmark` (`cargo test -p rootmark-server --release`). Bekleme koşulları zaman aşımlıdır (gecikme ölçülmez, eşik yok); zaman aşımı başarısızlıktır. CI'da `make test` ile macOS ve `ubuntu-latest`'te.

## `server/tests/watch_newdir.rs` (R1, PQ-22)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| N1 | 20 tekrar: yeni dizin `n1/dNN/` oluştur, **hemen** `dNN/a.md` yaz (tek `write`) | 20 belgenin hepsi günlükte: her biri için `BlockCreated` + `SnapshotCommitted`; `blame` blokları gösterir |
| N2 | 20 tekrar: iç içe `n2/eNN/x/y/` tek seferde (`create_dir_all`), hemen `y/deep.md` | 20/20 alınmış |
| N3 | 20 tekrar: yeni `n3/fNN/` ve hemen 10 dosya (`k0.md`..`k9.md`) | 200/200 alınmış |
| N4 | N1–N3'ten sonra | her belge için tam bir `SnapshotCommitted` (tarama + kendi olayı → ikinci alım no-op); `BlockDeleted` 0 |
| N5 | yeni `.gizli/` ve `n5/.obs/` altına md yaz; ardından görünür bir işaret dosyası | gizli yollar günlükte 0 (PQ-13 davranışı değişmez); işaret alınmış |
| N6 | vault dışında dolu bir dizin (2 not, iç içe) oluşturup vault'a `rename` ile taşı | iki not da alınmış |

## `server/tests/watch_remove.rs` (R2, PQ-18)
| Kimlik | Adımlar | Beklenen |
|---|---|---|
| D1 | `canonical` politika: not yaz, alınsın; `remove_file` | notun her soyu için tam bir `BlockDeleted` (`meta.actor.kind = human`, `meta.source = rootmark-serve-watch`) ve bloksuz `SnapshotCommitted`; HTTP `search` (sıcak katman) notun eşsiz sözcüğü için 0 sonuç; `blame` boş |
| D2 | `unverified` politika: insan notu tasdikli yazılsın (CLI `rootmark attest`); sonra tasdiksiz `remove_file` | `BlockDeleted` `meta.actor = {unattested_fs, fs}`, `meta.source = unattested_fs`; `meta.layer` yok; aramada 0 |
| D3 | tasdiksiz taşıma `a.md → sub/b.md` (`canonical` politika) | eski yol: her soyu `BlockDeleted`, `blame` boş; yeni yol: `BlockCreated`, `canonical`; arama yalnız yeni yolu döndürür |
| D4a | `unverified` politika, tasdikli yeniden adlandırma, tasdik **dosya işleminden önce kaydedilmez, hemen sonra** (zamanında) | `attest` 200; yeni yolun blokları `canonical`; eski yolun her soyu tam bir kez `BlockDeleted`; yeni yol için içerikli `SnapshotCommitted` tam bir kez |
| D4b | aynı, tasdik izleyici yeni yolu tasdiksiz aldıktan **sonra** (geç, `W` içinde) | `attest` 200; yeni yolun blokları geç tasdikle `canonical` (`LayerChanged` `cause: late_attestation`); eski yol tam bir kez `BlockDeleted`, kaldırma tasdikten sonra; çift işlem 0 |
| D5a | not sil ve bekleme dolmadan aynı içerikle yeniden yarat | `BlockDeleted` 0; olay 0 (içerik eşit) |
| D5b | not sil, kaldırmanın alınmasını bekle, aynı içerikle yeniden yarat | önce `BlockDeleted` (her soy), sonra `BlockCreated` |
| D6 | dizin `dd/` (3 not, biri iç içe) `remove_dir_all` | 3 notun her soyu `BlockDeleted`; aramada 0 |
| D7a | dizin `mv/` (2 not) vault dışına `rename` | iki notun her soyu `BlockDeleted` |
| D7b | dizin `mw/` (2 not) vault içinde `mw2/`'ye `rename` | eski yollar `BlockDeleted`, yeni yollar `BlockCreated` |
| D8 | not sil ve hemen SIGTERM | süreç 0 ile çıkar; günlükte notun `BlockDeleted`'ı var (bekleyen kaldırma durdurmada işlendi) |
| D9 | D1 + D3 + D6 dizisi iki taze vault'ta aynı adımlarla | iki `events.jsonl` bayt eşit (determinizm) |

Geriye uyum (Q1-5) mevcut testlerle: `make conformance` 39/39; eski sürümlerin vault'ları (`gh0_migrate`, `p2a_migrate`, `p2v1_migrate`, `p1e1_status`) değişmeden geçer.

## Koşu (R4)
- Pilot: P2-Q'nün sürücüsüyle D-NOTES (H01–H23, H09 dahil) 0.3.1 ile Mac, Docker (`linux/amd64`), CI (`pilot.yml`, iki koşu): `events_hash`, vektör dosyaları, indeks manifest'i **Mac ↔ Docker ↔ CI** bayt eşit.
- Obsidian kopyası, O06/O10/O14: silinen ve taşınan notların eski yollarında canlı blok 0; silinenler aramada 0.
