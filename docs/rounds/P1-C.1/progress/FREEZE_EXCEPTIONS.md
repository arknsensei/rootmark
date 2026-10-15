# P1-C.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-C.1/F<n>` anar). Eski satırlar silinmez ve değişmez.

## F1 · C1.0 · `core/tests/conformance.rs` — `spec_version` 1.0.2
- **Ne:** raporun `"spec_version": "1.0.1"` → `"1.0.2"` (tek satır; `git diff p1c-final -- core/tests/conformance.rs` +1/−1).
- **Neden:** spec 1.0.2 (K-C1.3, K-C1.5; sürüm gerekçesi SPEC_EDITS.md, ASSUMPTIONS C1.0-1). Emsal P1-B.1/F3.
- **Önce/sonra bit eşitliği:** `make conformance` → level 3, 30/30; `jq -c .tests reports/conformance.json` sha256 `ec32e78f…`, P1-C resmi RUN'ının (`20261015T014417Z-9aa326b`) `conformance_full.json`'ındaki `.tests` ile aynı (bütün alanlar, 30 girdi). Değişen tek alan `spec_version`.
- **Gözden geçirici onayı:** C1.0.review.md.
