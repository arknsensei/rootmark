# P1-C — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-C/F<n>` anar). Eski satırlar silinmez ve değişmez.

## F1 · C0 · `spec/fixtures/**` — C2-09 (K-C0.2)
- **Ne:** yeni dizin `spec/fixtures/C2-09/` (`case.c2.json`, `doc.md` = `C2-07/doc.md` kopyası, `expected.json`) ve `MANIFEST.json`'a yalnız bu üç dosyanın satırı (`git diff p1b1-final -- spec/fixtures`: `MANIFEST.json` +3/−0, başka değişen dosya yok). Üretim: `bash scripts/gen_fixtures_c2.sh` (C2 mevcut çekirdekten, C1 + manifest dondurulmuş v3-final üretecinden).
- **Neden:** ADDENDUM K-C0.2, P1-B.1 doğrulamasının 2. önerisi.
- **Önce/sonra bit eşitliği:** C1 ve C2-01..08 dosyaları bayt eşit (`git diff` silinen satır 0); `make conformance` raporunda C2-09 dışındaki 25 girdinin `{id,result,actual_hash,expected_hash}` JSON'u P1-B.1 resmi RUN'ının (`20261014T212243Z-226b511`) `conformance.json`'ı ile bayt eşit (sha256 `f5196a0d…` iki tarafta).
- **Gözden geçirici onayı:** C0.review.md.

## F2 · C0 · `core/tests/conformance.rs` — Seviye 2 = C2-01..C2-09
- **Ne:** `LEVEL2` dizisine `"C2-09"` (8 → 9) ve başlık yorumu. Başka satır yok.
- **Neden:** K-C0.2 — `level_claimed 2` için C2 9/9.
- **Önce/sonra bit eşitliği:** F1'deki 25 girdi eşitliği; `level_claimed 2`, 26/26.
- **Gözden geçirici onayı:** C0.review.md.

## F3 · C0 · `core/tests/conformance/eval_c2.rs` — `trace_tamper` işlemi
- **Ne:** yeni işlem `trace_tamper {field, value, expect_open_err}`: son `traces.jsonl` satırının `query_hash` ya da `consumer` alanı verilen değere çekilir, `trace_id` §8 formülüyle yeniden hesaplanır (yalnız girdi kuralı reddedebilsin), vault yeniden açılır, beklenen hata alt dizgisi denetlenir, dosya geri yazılır. Başlık yorumuna işlem satırı. Mevcut işlemlerin kodu değişmedi (yalnız eklenen satırlar).
- **Neden:** C2-09 senaryosu bir dosya bozma işlemi gerektirir; mevcut `tamper` yalnız `events.jsonl` meta anahtarı siler.
- **Önce/sonra bit eşitliği:** C2-01..08 beklenen değerleri yeni değerlendiriciyle bayt eşit üretildi (`gen_fixtures_c2.sh` sonrası `git diff` yalnız C2-09 + MANIFEST); F1'deki rapor eşitliği.
- **Gözden geçirici onayı:** C0.review.md.

## F4 · C0 · `core/tests/conformance_c2_gen.rs` — 9 C2 dizini
- **Ne:** `assert_eq!(dirs.len(), 8, "C2-01..C2-08")` → `9, "C2-01..C2-09"`.
- **Neden:** üreteç C2-09'u da yazar; eski sayı denetimi üretimi durdururdu.
- **Önce/sonra bit eşitliği:** F3 ile aynı kayıt.
- **Gözden geçirici onayı:** C0.review.md.

## F5 · C7 · `spec/fixtures/**` — C3-01..C3-04 (Seviye 3)
- **Ne:** yeni dizinler `spec/fixtures/C3-01..04/` (`case.c3.json`, `agent.md`, `expected.json`; insan belgeleri `../mini_vault`'tan okunur) ve `MANIFEST.json`'a yalnız bu on iki dosyanın satırı. Üretim: `bash scripts/gen_fixtures_c3.sh` (C3 mevcut indeksten, C1 + manifest dondurulmuş v3-final üretecinden).
- **Neden:** GOAL P1-C C-1, TASKS C7.
- **Önce/sonra bit eşitliği:** `git diff p1b1-final -- spec/fixtures` silinen satır 0 (C1, C2-01..08 ve C0'daki C2-09 dosyaları bayt eşit); `make conformance` raporunda Seviye 1–2'nin C2-09 dışındaki 25 girdisi P1-B.1 resmi RUN'ı ile bayt eşit (sha256 `f5196a0d…`), C2-09 C0'daki değerle aynı; çekirdek denetleyici (`core/tests/conformance.rs`) değişmedi.
- **Gözden geçirici onayı:** C7.review.md.
