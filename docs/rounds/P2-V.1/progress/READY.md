# P2-V.1 — READY

**doğrulama için hazır**

- **Ölçülen commit:** `58377deadc5b360cc010b7f7049e37f4de555459` (ürün ve araç kodunun son hali). Sonraki commit'ler (`8724e65` E4, `1127d4e` E5, bu READY commit'i) yalnız kayıt ve belge; dalın başı bu dosyanın commit'idir.
- **Dal / PR:** `p2-v.1/round` → `main`, **PR #39** (gövde `reports/rounds/P2-V.1/FINAL.md`). PR'ı ajan birleştirmez.
- **Kademe 2 (`verify.yml`, `58377de`):** **`41239746833`**, **`41239754895`** — zincirler 0/0, D farkı 0, sayılan FAIL 1/1 (`tamper·det`), `portable` 4 bulgu (`conformance.json`), duvar 9 141 / 9 221 s, CPU AMD EPYC 7763 (dört zincir). İlk deneme (`1d97064`): `41235960263`, `41235968688`.
- **E2 (işlemciler arası, kademe 1):** dal **`41219475628`** (`26128c7`: Xeon 6973P-C, Neoverse-N2, Apple M1 — 3/3 bayt eşit); PR #39 **`41252646332`** (`1127d4e`: EPYC 9V45, Neoverse-N2, Apple M1 — 3/3 bayt eşit; bütün işler yeşil, FAST 35 dk 42 s).
- **Mac kümesi RUN:** `20261031T204025Z-58377de` (`reports/rounds/P2-V.1/mac/`; 7 188 s). Yerel FAST RUN: `20261031T224707Z-58377de` (çıkış 0).
- **Öteki koşular:** E0 CI `41204929528`; E1 CI `41215085044`; E1 commit'inde kademe 2 gözlemi `41215091092`; eski zincir referansı P2-V V0m `40613392932`.

## Karar kuralının sonucu
**Tur kaldı** (V1-6, V1-7, V1-8, V1-10); eski zincir yürürlükte; kademeler yürürlükte değil (DEV_PROCESS §3, kök `CLAUDE.md`). Karar talepleri **K-P2V1-T2** (R7 tekerlek adı), **K-P2V1-T3** (`tamper·det`), **K-P2V1-T4** (`conformance.json` host yolu), **K-P2V1-T5** (Mac kümesinde gerçek Obsidian O); bilgi K-P2V1-B1 (hız). Geçenler: V1-1, V1-2, V1-3, V1-4, V1-5, V1-9, V1-11. Ayrıntı `reports/rounds/P2-V.1/FINAL.md`.

## Kapılar
`make lint` temiz; `make conformance` 39/39; `make verify FAST=1` çıkış 0; `check_frozen`/`check_deps`/`check_dep_age` temiz; CI'da (PR #39) test + lint + conformance ubuntu-latest ve macOS yeşil. **Yerel `make test`'te bir kalış:** `server/tests/p2_0_attest.rs` `cli_attest_raises_a_watched_edit` (izleyicinin 2,5 s penceresi; `fseventsd` ~%100 CPU); aynı test `main`'de bu makinede 3'te 2 kalıyor, CI'da iki işletim sisteminde geçiyor; test değiştirilmedi (E5.md, FINAL Notlar). Gözden geçirici E5: 2. tur **KABUL**.
