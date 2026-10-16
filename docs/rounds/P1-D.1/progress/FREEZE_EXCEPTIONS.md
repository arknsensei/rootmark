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

## F6 · D1.4 · `spec/fixtures/**` — C2-10 eklendi
- **Ne:** `spec/fixtures/C2-10/` (`case.c2.json` + `expected.json`). Girdi metinleri C1-08, C1-10, C1-11, C1-12 ve C1-14'ün **var olan** dosyalarıdır (`../C1-xx/…`), kopyalanmadı. `MANIFEST.json` yalnız iki C2-10 satırı kazanır.
- **Neden:** GOAL D1-10 / D1.4 (C2-10 olay aktörü, K-D1.2).
- **Önce/sonra bit eşitliği:** `scripts/gen_fixtures_c2.sh` on C2 `expected.json`'ı ve `MANIFEST.json`'u yeniden üretti. `git diff spec/fixtures`: C1-*, C2-01..09 ve C3-* dosyaları değişmedi, `MANIFEST.json` farkı yalnız `+ C2-10/…` satırları.
- **Gözden geçirici onayı:** D1.4.review.md.

## F7 · D1.4 · `core/tests/conformance/eval_c2.rs` — `ingest … actor`, `actor_tamper`
- **Ne:** iki ekleme, var olan işlemlerin gözlemleri değişmedi.
  - `ingest`'e isteğe bağlı `actor` alanı: atfedilen ingest (`Vault::ingest_by`), gözlemine `attributed` eklenir.
  - Yeni `actor_tamper {kind, change, expect_open_err?}` işlemi.
- **Önce/sonra bit eşitliği:** C2-01..09'un gözlemleri `expected.json`'larıyla bayt eşit (`make conformance` 34/34; bu vakaların `actual_hash`'leri değişmedi).
- **Gözden geçirici onayı:** D1.4.review.md.

## F8 · D1.4 · `core/tests/conformance_c2_gen.rs` — 10 C2 dizini
- **Ne:** `assert_eq!(dirs.len(), 10, "C2-01..C2-10")`.
- **Gözden geçirici onayı:** D1.4.review.md.

## F9 · D1.4 · `core/tests/conformance.rs` — Seviye 2 = C2-01..C2-10, `spec_version` 1.0.3
- **Ne:** `LEVEL2` 10 vaka, rapor `spec_version` `"1.0.3"` (SPEC_EDITS: CRITERION yama kuralı), modül yorumu.
- **Önce/sonra bit eşitliği:** C1 ve C2-01..09 girdileri aynı. Rapor C2-10 girdisini kazanır ve `spec_version` değişir (K-D1.2/K-D1.6/K-D1.8 gereği; emsal P1-B.1/F3, P1-C.1/F1).
- **Gözden geçirici onayı:** D1.4.review.md.

## F10 · D1.4 · `index/tests/conformance_c3.rs` — alt düzey toplamı 27
- **Ne:** `LOWER_TOTAL` 26 → 27 (C2-10), toplam 34, yorum. F4'ün (D1.3) devamı.
- **Gözden geçirici onayı:** D1.4.review.md.

## F11 · D1.4 · `scripts/test_tamper_p1c.sh` — Seviye 3 toplamı 34
- **Ne:** bozulmamış fixture kopyasının denetiminde `.total == 31` → `.total == 34` (tek satır). Bozma vakaları ve beklenen retler aynı.
- **Neden:** betik zincirde ölümcül bir adımdır (`verify.sh`). Uyumluluk raporu artık 34 girdi tutuyor (emsal P1-C.1/F7: 30 → 31).
- **Önce/sonra bit eşitliği:** betik çıkış 0. Günlük satırları P1-D resmi RUN'ının `tamper_p1c.log`'uyla aynı, fark yalnız `mktemp` sonekleri (D1.4 kaydı).
- **Gözden geçirici onayı:** D1.4.review.md.
