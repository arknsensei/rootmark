# P1-D.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir. Satırın notu `P1-D.1/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · D1.3 · `index/tests/p1c1.rs` — C1-6 manifest kaydı `lexical.dl` (K-D1.7)
- **Ne:** `c16_manifest_lexical_record` şunları bekler:
  - `format` `mdkernel-index/3`;
  - `lexical` nesnesinde altıncı alan `dl = "exact token count, f64, no fieldnorm quantization"`;
  - kaldırma ve değiştirme vakalarına `dl` (6 + 1 + 6 + 1 = 14 ret vakası, önceden 12);
  - modül yorumunda `dl`.

  Başka değişiklik yok.
- **Neden:** K-D1.7 manifest'e `lexical.dl` ekler ve indeks biçimini `/3`'e çıkarır. Eski test 5 alanlı nesneyi ve `/2`'yi birebir bekliyordu. Değişmeden kalsaydı `make p1c1-checks` (zincir) ölümcül biçimde başarısız olurdu.
- **Önce/sonra bit eşitliği:** C1-2..C1-5 testlerinin çıktıları (`c1_2_3.json`, `c1_4.json`, `c1_5.json`, `MDK_P1C1_OUT`), P1-D resmi RUN'ınınkilerle bayt eşit. `c1_6.json` farklıdır:
  - `lexical` nesnesi `dl` alanını kazanır;
  - iki yeni ret vakası eklenir (`remove lexical.dl`, `change lexical.dl`);
  - var olan altı vakanın hata **metni** değişir, ret kararları aynı kalır: beş `change lexical.*` iletisi artık kaydın tamamını `dl` ile basıyor, `unknown lexical.extra` iletisinin beklenen alan listesinde `dl` var.

  Bütün vakalar reddedilmeye devam eder (`pass: true`). D1.3 gözden geçirmesi düzeltmesi.
- **Gözden geçirici onayı:** D1.3.review.md.

## F2 · D1.3 · `spec/fixtures/**` — C3-06, C3-07 eklendi
- **Ne:** `spec/fixtures/C3-06/` (case + 6 ajan metni + expected) ve `C3-07/` (case + 3 ajan belgesi + expected). `MANIFEST.json` yalnız bu dosyaların satırlarını kazanır. Diğer bütün fixture dosyaları ve satırları bayt eşit kalır.
- **Neden:** GOAL D1-10 / D1.3 (C3-06 canonical değişmezliği, K-D1.1; C3-07 yanıt uzunluğu, K-D1.8).
- **Önce/sonra bit eşitliği:** `scripts/gen_fixtures_c3.sh` yedi C3 `expected.json`'ı ve `MANIFEST.json`'u yeniden üretir. `git diff spec/fixtures`: C1-*, C2-*, C3-01..05 dosyaları değişmedi, `MANIFEST.json` farkı yalnız `+` satırları.
- **Gözden geçirici onayı:** D1.3.review.md.

## F3 · D1.3 · `index/tests/conformance/eval_c3.rs` — `agent_write` işlemi, `lengths` gözlemi
- **Ne:** iki ekleme, var olan işlemlerin gözlemleri değişmedi.
  - `agent_write {doc, file, source, mode, heading?}`: ajanın tam metni, varsayılan `append_only`'nin etkiye göre kararı; izinliyse atfedilen ingest + indeks.
  - `search … lengths: true`: katman başına ve toplam en büyük yanıt uzunluğu ile sınır denetimi.
- **Neden:** C3-06 ve C3-07 bu işlemlerle ifade edilir.
- **Önce/sonra bit eşitliği:** C3-01..05'in gözlemleri, `expected.json`'ları ile bayt eşit (`make conformance` 33/33; o vakaların `actual_hash`'leri değişmedi).
- **Gözden geçirici onayı:** D1.3.review.md.

## F4 · D1.3 · `index/tests/conformance_c3.rs` — Seviye 3 = C3-01..C3-07
- **Ne:** `LEVEL3` 7 vaka. Toplam `LOWER_TOTAL + 7`. Yorum.
- **Önce/sonra bit eşitliği:** C3-01..05 girdileri aynı. Rapor iki girdi kazanır.
- **Gözden geçirici onayı:** D1.3.review.md.

## F5 · D1.3 · `index/tests/conformance_c3_gen.rs` — 7 C3 dizini
- **Ne:** `assert_eq!(dirs.len(), 7, "C3-01..C3-07")`.
- **Gözden geçirici onayı:** D1.3.review.md.
