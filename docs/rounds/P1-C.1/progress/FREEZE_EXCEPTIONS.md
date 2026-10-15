# P1-C.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-C.1/F<n>` anar). Eski satırlar silinmez ve değişmez.

## F1 · C1.0 · `core/tests/conformance.rs` — `spec_version` 1.0.2
- **Ne:** raporun `"spec_version": "1.0.1"` → `"1.0.2"` (tek satır; `git diff p1c-final -- core/tests/conformance.rs` +1/−1).
- **Neden:** spec 1.0.2 (K-C1.3, K-C1.5; sürüm gerekçesi SPEC_EDITS.md, ASSUMPTIONS C1.0-1). Emsal P1-B.1/F3.
- **Önce/sonra bit eşitliği:** `make conformance` → level 3, 30/30; `jq -c .tests reports/conformance.json` sha256 `ec32e78f…`, P1-C resmi RUN'ının (`20261015T014417Z-9aa326b`) `conformance_full.json`'ındaki `.tests` ile aynı (bütün alanlar, 30 girdi). Değişen tek alan `spec_version`.
- **Gözden geçirici onayı:** C1.0.review.md.

## F2 · C1.3 · `spec/fixtures/**` — C3-05 ve K-C1.3 gereği C3-01..04 beklenen değerleri
- **Ne:** yeni `spec/fixtures/C3-05/` (`case.c3.json`, `agent.md`, `expected.json`); `C3-01..04/expected.json` yeniden üretildi; `MANIFEST.json`'da yalnız bu dosyaların satırları (4 değişen, 3 eklenen; `git diff` başka satır yok). Üretim: `bash scripts/gen_fixtures_c3.sh` (değişmedi).
- **Neden:** K-C1.3 (CONFORMANCE C3-05; C3-01..04'ün `include_proposal: true` aramaları). Değişen her fixture'ın gerekçesi SPEC_EDITS.md "Fixture değişiklikleri".
- **Önce/sonra bit eşitliği:** C1-*, C2-*, `mini_vault`, C3 `case.c3.json`/girdi dosyaları bayt eşit (`git diff p1c-final --stat -- spec/fixtures` yalnız C3 `expected.json`'ları, C3-05 ve MANIFEST); `make conformance` → level 3, 31/31; Seviye 1–2'nin 26 girdisi P1-C resmi RUN'ı ile bayt eşit (`jq -c '[.tests[]|select(.level<=2)]'` sha256 `9d26e401…` iki tarafta). C3-01..04'te **bayrak kapalı her arama gözlemi bayt eşit**; değişen 17 işlemin hepsi `include_proposal: true`.
- **Gözden geçirici onayı:** C1.3.review.md.

## F3 · C1.3 · `index/tests/conformance/eval_c3.rs` — `canonical_prefix_of` gözlemi
- **Ne:** `search` işlemine isteğe bağlı `canonical_prefix_of: <label>`: her sıralamanın `canonical` öneki etiketin sıralamasıyla aynı mı (`{label, equal}`); başlık yorumuna bir cümle. Mevcut işlemlerin kodu değişmedi (+27/−1; silinen satır başlık yorumundaki cümle sonu).
- **Neden:** C3-05 özellik vakası (K-C1.3).
- **Önce/sonra bit eşitliği:** alanı taşımayan vakaların gözlemi değişmez (C3-01..04'ün bayrak kapalı gözlemleri bayt eşit; F2).
- **Gözden geçirici onayı:** C1.3.review.md.

## F4 · C1.3 · `index/tests/conformance_c3.rs` — Seviye 3 = C3-01..C3-05
- **Ne:** `LEVEL3` 4 → 5 (`"C3-05"`), başlık yorumu (beş girdi, toplam 31). Başka satır yok.
- **Neden:** K-C1.3; GOAL C1-7 "C3 5/5".
- **Önce/sonra bit eşitliği:** F2'deki 26 girdi eşitliği; `level_claimed 3`, 31/31.
- **Gözden geçirici onayı:** C1.3.review.md.

## F5 · C1.3 · `index/tests/conformance_c3_gen.rs` — 5 C3 dizini
- **Ne:** `assert_eq!(dirs.len(), 4, "C3-01..C3-04")` → `5, "C3-01..C3-05"`.
- **Neden:** üreteç C3-05'i de yazar.
- **Önce/sonra bit eşitliği:** F2 ile aynı kayıt; üreteç iki kez koşturulunca ağaç değişmiyor.
- **Gözden geçirici onayı:** C1.3.review.md.
