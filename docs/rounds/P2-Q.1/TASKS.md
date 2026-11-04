# P2-Q.1 — Görevler

Genel kurallar:
- Sıra: R0 → R1 → R2 → R3 → R4 → R5 (R1 ve R2 bağımsızdır, sırayla yapılır; R3'ün belge işleri beklerken sürebilir). R0'ın `DESIGN.md` + `TESTS.md`'si commit'lenmeden R1/R2'nin kodu yazılmaz.
- Progress `docs/rounds/P2-Q.1/progress/`. Commit öneki `P2-Q.1/R<n>:`. Dal `p2-q.1/round` (yalnız birleşmiş `main`'den, K-P2.2); yayım hazırlığı ayrı dal `release/0.3.1`.
- Yama turu, araç dozu (K-P2.4): gözden geçirici yalnız R5'te (tek tur).
- Yeni testler yeni dosyalarda (`server/tests/watch_newdir.rs`, `server/tests/watch_remove.rs` gibi); dondurulmuş test ve kaydedicilere dokunulmaz (değişmesi gerekirse DEV_PROCESS §4).
- İnsan adımı beklemek durma değildir (`HUMAN_STEP.md`); soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye. Spec'in tanımlamadığı bir durum: PARK + karar talebi (uydurma yok).
- Kişisel içerik: Obsidian kopyası yalnız silme/taşıma adımları için yeniden oluşturulur, depo dışında; P2-Q'nün gizlilik kuralları (anahtarlı özetler, sızıntı denetimi) aynen.

## R0 — Açılış ve tasarım
**Yapar:** `ROUND_START`; DECISIONS K-P2Q1.1 (açılış). `progress/DESIGN.md`: PQ-22 ve PQ-18'in yöntemi; olay biçimi spec atıflarıyla (§5 eşleştirmenin kalanı, §7 olayların aktör/kaynağı, §7.1 tasdiksiz değişiklik ve tasdikli yeniden adlandırma); izleyicinin olay türlerinin (create/modify/remove/rename; macOS FSEvents ve Linux inotify farkları, `notify` kitaplığının birleştirdiği olaylar) ürün davranışına eşlemesi; sınır durumlar; politika etkisi (`append_only`, `unattested`); eski vault'ların göçsüz açılması. `progress/TESTS.md`: ön-kayıtlı testler ve beklenen olaylar. Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi (beklenmeyen değişiklik = adsız fark).
**Kabul:** Q1-1'in ilk yarısı.

## R1 — PQ-22: yeni dizin
**Yapar:** izleyici, yeni dizin olayında dizini tarar ve içindeki md dosyalarını alır (iç içe dizinler dahil; `.rootmark/` ve gizli dizinler bugünkü kurallarla dışarıda); ikinci alım içerik eşitse no-op. Yeni entegrasyon testi (Linux ve macOS; ≥ 20 tekrar; yeni dizin + hemen yazım, iç içe, çok dosya). CI'da `ubuntu-latest` ve macOS iş akışlarında koşar.
**Kabul:** Q1-2'nin test yarısı.

## R2 — PQ-18: silme ve taşıma
**Yapar:** kaldırılan md dosyasının blokları `BlockDeleted` (spec'in biçimi, aktör/kaynak); tasdiksiz taşıma = eski yolun silinmesi + yeni yolun alınması; tasdikli yeniden adlandırma çift işlenmez; dizin silme/taşıma. Yeni entegrasyon testleri (macOS ve Linux): silme → arama ve `blame` boş, günlükte `BlockDeleted`; tasdiksiz taşıma; tasdikli yeniden adlandırma; silip yeniden yaratma; dizin silme.
**Kabul:** Q1-3'ün test yarısı; Q1-5.

## R3 — Belgeler, CodeQL, bağımlılık
**Yapar:** (a) kılavuz §5.1 VF-1 (güven penceresi, "Exit Restricted mode"); `FINDINGS.md` PQ-24 (VF-1), PQ-25 (G2), PQ-26 (`migrate` alan adları); (b) kılavuz §4.1 masaüstü ajan ayrımı + Claude Code izin kuralı örneği, sınaması (`progress/R3_agent_isolation.md`: kurallı bir oturumun `.rootmark/server.token`'ı okuma denemesi reddedildi — komut ve çıktı, token değeri yazılmadan); (c) `CHANGELOG.md` 0.3.1 taslağı, Known issues; (d) `release.yml` `permissions`; harness uyarıları için seçim (istisnayla düzeltme ya da gerekçeli kapatma) ve kaydı; (e) `minimatch` (2026-11-10'dan sonra; `eslint` ile `make lint-ts` ve `make test-ts` temizse `overrides`, değilse gerekçe). Uyarıların son durumu `progress/R3_security.md` (`gh api` dependabot ve code-scanning listeleri).
**Kabul:** Q1-6, Q1-7.

## R4 — Doğrulama koşuları ve 0.3.1
**Yapar:** (a) kademe 2 bir tetikleme (`gh workflow run verify.yml --ref p2-q.1/round -f sha=<sha>`); yapıtlar, `tier2 pair/fails`, `portable`; değişen kayıtların R0 listesiyle karşılaştırması. (b) `make verify-mac` bir kez (önce `fseventsd` sağlığı `ps` ile; sağlıksızsa `HUMAN_STEP.md`). (c) `release/0.3.1` dalı, R1–R20, PR; `HUMAN_STEP.md` (birleşme, `release.yml`, etiket, sürüm). (d) Sürüm yayımlanınca: P2-Q'nün sürücüsüyle D-NOTES senaryosu Mac, Docker ve CI'da (`pilot.yml`, iki koşu) 0.3.1 ile; Obsidian kopyasında O06/O10/O14. Kayıtlar `reports/rounds/P2-Q.1/`.
**Kabul:** Q1-2, Q1-3 (koşu yarıları), Q1-4, Q1-8.

## R5 — Rapor, gözden geçirici, hazır
**Yapar:** `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`; `FINDINGS.md` (PQ-22, PQ-18 kapandı); gözden geçirici tek tur (GOAL başlığı); `reports/rounds/P2-Q.1/FINAL.md` (Q1-1..Q1-9, karar kuralının dalı, koşu kimlikleri); READY; tek PR (`gh pr create --base main --head p2-q.1/round --title "P2-Q.1: izleyici yaması ve 0.3.1" --body-file reports/rounds/P2-Q.1/FINAL.md`).
**Kabul:** Q1-9; READY "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
