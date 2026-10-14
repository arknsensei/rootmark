# P1-B.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-B.1/F<n>` anar). Eski satırlar silinmez ve değişmez.

## F1 · B1.0 · `scripts/check_frozen.sh` — noktalı tur kimliği
- **Ne:** yeniden dondurma satırının notundaki istisna atfı kalıbı `[A-Z0-9]+-[A-Z0-9]+/F[0-9]+` → `[A-Z0-9]+-[A-Z0-9]+(\.[0-9]+)?/F[0-9]+`. Başka değişiklik yok (`git diff p1b-final -- scripts/check_frozen.sh`: 1 satır).
- **Neden:** bu turun kimliği `P1-B.1`; eski kalıp `P1-B.1/F<n>` atfını hiç eşleştirmez (`P1-B` sonrası `.`), dolayısıyla bu turun her yeniden dondurma satırı (B1.2 `spec/fixtures/**`, `conformance.rs`; B1.3 `check_v3_compat.sh`) "cites no freeze exception" ihlali verirdi.
- **Önce/sonra bit eşitliği:** `p1b-final` tablosuyla (değişiklikten önce, aynı ağaç) eski ve yeni denetleyicinin çıktısı satır satır aynı, tek fark denetleyicinin kendi satırı (değişmiş dosya → bu girdinin yeniden dondurma satırıyla kapanır). Eski atıfların hepsi (`P1-B/F1`…`P1-B/F8`) yeni kalıpla aynı dizgiye eşleşir (`(\.[0-9]+)?` boş). Kayıt: B1.0.md.
- **Gözden geçirici onayı:** B1.0.review.md.
