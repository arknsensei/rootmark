# P2-Q.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna **eklenen** bir yeniden dondurma satırıyla eşleşir; satırın notu `P2-Q.1/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · R4 · `scripts/release_check.sh` — R16 `### Hüküm` başlığını da okur
- **Ne:** `verdict_of`'un ikinci biçiminde `sed -n '/^## Hüküm/,/^## /p'` → `sed -n '/^###* Hüküm/,/^## /p'` (BRE: `##` ve ardından sıfır ya da daha çok `#`; yani `## Hüküm` ve `### Hüküm`), ve yorumda bir satır. Başka satır değişmedi (`git diff ROUND_START -- scripts/release_check.sh`: 1 kod satırı + 1 yorum satırı).
- **Neden:** 0.3.1'in yayım önkoşulu (RELEASE.md insan adımı 1, R16) P2-Q'nün bağımsız doğrulamasıdır (`reports/rounds/P2-Q/VERIFICATION.md`, koşullarını K-P2Q.5 ve P2-Q.1/R3 kapattı). O rapor hükmünü `### Hüküm` başlığının altına ilk kalın satır olarak yazmış (`**Şu koşullarla güvenilir.**`); betik yalnız `**Hüküm: …**` satırını ve `## Hüküm` başlığını okuyor → R16 "no verdict" ile kalır, Q1-8 (R1–R20 21/21) sağlanamaz. Doğrulama raporu başka bir turun kaydıdır, değiştirilmez; ölçen betik biçimi tanır hale getirilir.
- **Önce/sonra:** `reports/rounds/*/VERIFICATION.md`'nin 20 raporunun hepsinde `verdict_of`'un eski ve yeni ifadesi yan yana koşturuldu: daha önce okunan 14 raporun hükmü **aynı** (bayt eşit dizge); daha önce okunamayan 3'ü (P1-B.1 `Güvenilir`, P1-C `Güvenilir`, P2-Q `Şu koşullarla güvenilir`) artık okunuyor; 3'ü (P1-A, P1-B, P2-V.1) iki ifadede de boş. Okunan bir hüküm değişmedi; yalnız okunamayan biçim kapsandı. Komut ve çıktı: `R4.md` "F1 önce/sonra".
- **Gözden geçirici onayı:** R5 (turun tek gözden geçirici turu; `R5.review.md`).
