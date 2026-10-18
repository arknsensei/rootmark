# P2-0.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir; satırın notu `P2-0.1/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · Y2 · uyumluluk değerlendiricileri: `core/tests/conformance/eval_c2.rs`, `core/tests/conformance.rs`, `core/tests/conformance_c2_gen.rs`, `index/tests/conformance_c3.rs` — C2-13 (spec 2.0.1, K-P2-01.3)
- **Ne:** `eval_c2.rs`: `attest` işlemi `content: true` / `content_file` alır (içerik dosyanın baytları değilse hata), kaydı başvuru sunucusunun sırasıyla yapar (pencere, geç kısmi eşleşme, sonra günlük satırı; `partial: true`); `external` işlemi birebir eşleşme yoksa `Window::observe_partial`'ı dener (tasdik edilen baytlar tasdikli alım, `partial` satırı, kalan tasdiksiz alım; gözlem `{"decision": "partial", "attested", "attributed"}`); tasdiksiz alımlar sunucu gibi `pending` listesinde (gözlenen sürümler, adlanan soylar) tutulur; `attestation_tamper` yeni değişiklik `partial_false`. Var olan işlemlerin gözlemleri aynen. `conformance.rs`: `LEVEL2` 13 test, rapordaki `spec_version` `"2.0.1"`, başlık yorumu. `conformance_c2_gen.rs`: dizin sayısı 13. `conformance_c3.rs`: `LOWER_TOTAL` 29 → 30.
- **Neden:** TASKS Y2 "CONFORMANCE C2-13 tanımı + fixture; `make conformance` 38/38"; GOAL H-P2-0.1j; spec 2.0.1 (SPEC_EDITS, K-P2-01.3).
- **Önce/sonra bit eşitliği:** `make conformance`: Seviye 3, **38/38**. Eski 37 testin her birinin `id`, `result`, `actual_hash`, `expected_hash`'i P2-0 resmi RUN'ının `reports/rounds/P2-0/runs/20261018T161850Z-e378be2/conformance_p1d1.json`'uyla `jq -S` sonrası **bayt eşit** (`cmp`); C2-13 geçer. `scripts/gen_fixtures_c2.sh` var olan C2 fixture'larının `expected.json`'unu bayt bayt aynı bıraktı (`git status spec/`: yalnız `MANIFEST.json` ve yeni `C2-13/`). Gömülü `rootmark conformance`: 38/38, spec 2.0.1.
- **Gözden geçirici onayı:** Y2.review.md.

## F2 · Y2 · `spec/fixtures/**` — C2-13 fixture'ı ve MANIFEST
- **Ne:** yeni dizin `spec/fixtures/C2-13/` (vaka `case.c2.json`, on belge, `expected.json`); `MANIFEST.json`'a yalnız yeni dosyaların satırları eklendi (silinen ya da değişen satır yok). Var olan fixture dosyaları aynen.
- **Neden:** F1 ile aynı.
- **Önce/sonra:** F1 kaydı; `git diff --stat spec/fixtures`: yalnız `MANIFEST.json`'a eklenen satırlar.
- **Gözden geçirici onayı:** Y2.review.md.

## F3 · Y2 · uyumluluk sayısı 37 → 38: `scripts/test_tamper_p1c.sh`, `scripts/test_tamper_p1d1.sh`, `scripts/release_local.sh`, `scripts/release_target.sh`, `scripts/release_check.sh`
- **Ne:** her betikte uyumluluk toplamının denetimi `37` → `38` (`.total == 38`, `.passed == 38`, R4 madde metni) ve yorumlarda sürüm notu ("38/38 since spec 2.0.1; 37/37 for 2.0.0, 34/34 before"). Başka satır değişmedi (`git diff -U0`: 11 satır, hepsi bu sayı ya da yorumu).
- **Neden:** C2-13 (F1) toplamı 38 yapar; bu betikler 37'yi sabit denetler ve aksi halde doğru bir derlemeyi kırmızı gösterirdi (P2-0/F9'un aynısı).
- **Önce/sonra:** değişiklik yalnız sabit; davranış 38/38 bir raporla eskisinin 37/37 bir raporla verdiği sonucu verir. `bash scripts/test_tamper_p1c.sh`, `test_tamper_p1d1.sh` Y2 kalite kapılarında koşar (Y2.md).
- **Gözden geçirici onayı:** Y2.review.md.
