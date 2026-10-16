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

## F12 · D1.7 · `scripts/verify.sh` — P1-D.1 zinciri
- **Ne:** dört değişiklik.
  - Başlık yorumuna P1-D.1 paragrafı.
  - `chain`'de `make p1d-checks`'ten sonra `make p1d1-checks RUN=$run` (2 satır).
  - `pair`'in son satırı `compare_p1d` → `compare_p1d1`. P1-D kümesi bunun içinde, aynen `compare_p1d.pair`.
  - Yeni `compare_p1d1` işlevi ve `compare-p1d1` alt komutu; kullanım iletisine `compare-p1d1`.

  `diff` +26/−2: değişen 2 satır pair karşılaştırıcısı ve kullanım iletisi, gerisi ekleme. Önceki zincir adımları, sırası ve `compare`…`compare-p1d` aynen duruyor.
- **Neden:** TASKS D1.7 ("`verify.sh` genişletme"). D1-12'nin çift koşturması P1-D.1 ölçümlerini iki bağımsız klonda koşturmalı.
- **Önce/sonra bit eşitliği:** önceki adımlar değişmeden koşar. P1-D çift kümesinin bütün alanları `compare_p1d1 pair` içinde `compare_p1d.pair` ile karşılaştırılır. P1-D resmi RUN'ına göre fark yalnız karar gereği hariçlerde kalır (`compare_p1d1 regress`, D1.7 kaydı).
- **Gözden geçirici onayı:** D1.7.review.md.

## F13 · D1.7 · `scripts/governance_scenario.sh` — ajan düzenlemesi `--mode append`
- **Ne:** iki değişiklik.
  - Senaryo vault'unun `policy.toml`'una `[edit] policy = "unrestricted"`.
  - `espresso.md`'nin ajan düzenlemesi: eskiden dosyanın tamamı `mdk propose` ile üzerine yazılıyordu, artık eklenen paragraf `--mode append` ile öneriliyor. Fixture `clean, sweet cups` içermediği için eski `sed` hiçbir şey değiştirmiyordu ve düzenleme her zaman bu paragrafın eklenmesiydi. Fixture değişirse betik artık açıkça hata verir.

  Başka değişiklik yok.
- **Neden:** K-D1.1 ve D1.0-5 (K-D4-T b) gereği CLI `mdk propose` var olan bir belgenin üzerine yazmaz (`exists`). İnsan bölümüne başlıksız ek de `append_only` altında reddedilir. Değişmeden kalsaydı P1-B `p1b-checks` zincirde ölümcül biçimde dururdu (geliştirme zinciri `20261016T101541Z-142d979`: "espresso.md exists; propose never overwrites").
- **Önce/sonra bit eşitliği:** `bash scripts/governance_scenario.sh target/release/mdk <out>` çıktısı P1-D resmi RUN'ının `governance/` dizini ile karşılaştırıldı. `events.jsonl` ve `traces.jsonl` bayt eşit, `governance.json` eşit (`events_hash` `2b0dc8e8…` aynı). `espresso.md` eski üzerine yazmanın üreteceği baytlarla aynı. Senaryo dizinindeki iki fark karar gereğidir: `policy.toml`'daki `[edit]` bloğu (bu istisna) ve D1.2'den beri CLI `propose`'un da yazdığı `proposals.jsonl` (D1.0-5). Karşılaştırılan yapıtlar (`events.jsonl`, `traces.jsonl`, `checkpoints/`, `governance.json`) eşit.
- **Gözden geçirici onayı:** D1.7.review.md.

## F14 · D1.7 · `scripts/test_p1c1.sh` — C1-6 anahtar listesinde `dl`
- **Ne:** C1-6 denetiminin beklediği anahtar listesi `["b","idf","k1","tokenizer","variant"]` → `["b","dl","idf","k1","tokenizer","variant"]`, denetim etiketi ve bir yorum satırı. Vakalar ve diğer denetimler aynı.
- **Neden:** K-D1.7 manifest'e `lexical.dl` ekler. Betik `make p1c1-checks`'te ölümcüldür: geliştirme zinciri `20261016T102619Z` "44/45 cases" ile durdu.
- **Önce/sonra bit eşitliği:** `bash scripts/test_p1c1.sh <out>` çıkış 0, 45/45. `p1c1_cli.json`, P1-D resmi RUN'ınınkiyle `jq -S` karşılaştırmasında fark yok.
- **Gözden geçirici onayı:** D1.7.review.md.

## F15 · D1.7 · `research/eval/compare_p1d.py` — alan kümesi farkı kayıtlı fark olur
- **Ne:** `compare()` iki tarafın alan kümeleri farklı olduğunda artık `MetricError` (çıkış 1) atmaz. Yalnız bir tarafta olan her alanı `{"a", "b"}` (öbür taraf `"absent"`) olarak `differing_fields`'a yazar (çıkış 3, kaydedilir). Ortak alanların karşılaştırması aynı. Başka değişiklik yok (+5/−3).
- **Neden:** K-D1.7 `p1c1_checks.json`'a `C1-6›lexical›dl` alanını ekler. P1-D'nin dondurulmuş D-9 karşılaştırıcısı (`compare_p1d regress`, P1-C.1 resmi RUN'ına göre) alan kümesi farkını ölümcül sayıyordu. Bu yüzden `make p1d-checks` zincirde duruyordu (geliştirme zinciri `20261016T105235Z-7a4718b`: "field sets differ at ['C1-6›lexical›dl']"). Fark kararın sonucudur ve kaydedilmelidir, çünkü P1-D'nin kendi kuralı da "a criterion that fails is recorded, not fatal"dır.
- **Önce/sonra bit eşitliği:** alan kümeleri eşit olduğunda (A↔B `pair`, P1-D resmi RUN'ı) çıktı değişmez. Değişen tek şey, önceden hata veren durumun artık bir fark kaydı vermesidir. Geliştirme RUN'ında regress (P1-C.1'e göre) çıkış 3 verdi; farklar yalnız karar gereği alanlarda: `spec_version` (×3), `fixtures_manifest_hash`, `C1-6›lexical›dl`, `C1-6›rejected_cases`, `C1-7›pass`, üç indeks manifest hash'i.
- **Bağlı test:** `research/tests/test_p1d_tools.py` (dondurulmuş değil) eski davranışı birebir sınıyordu: `test_field_set_difference_is_an_error`. Karar gereği `test_field_set_difference_is_recorded` oldu ve aynı sıkılıkta: fark iki yönde de `differing_fields`'ta `"absent"` ile görünmeli, sessiz geçiş yok (D1.7-pre-fix).
- **Gözden geçirici onayı:** D1.7.review.md.

## F16 · D1.7 · `research/eval/compare_p1d1.py` — `d1_5.json` bayt listesinden çıktı
- **Ne:** `P1D1_BYTES`'ten `p1d1/d1_5.json` çıkarıldı (bir satır silindi, bir yorum eklendi). Dosyanın bütün alanları `P1D1_FILES` üzerinden alan alan karşılaştırılmaya devam eder: `events.jsonl`/`traces.jsonl` hash'leri ve eşitlikleri dahil, duvar saati alanları (`P1D1_TIMING`) hariç.
- **Neden:** dosya iki duvar saati alanı (`elapsed_ms_lockstep`, `elapsed_ms_pipelined`) taşır. Bu alanlar alan karşılaştırmasında zaten hariçti ama bayt karşılaştırmasına da girmişti. Bu bir ölçüm kodu hatasıdır. İlk resmi çift (`20261016T115216Z-97bb027` ↔ `20261016T121823Z-97bb027`) bu yüzden çıkış 3 verdi: 100 875 alan eşit, tek bayt farkı bu iki süre.
- **Önce/sonra bit eşitliği:** aynı iki RUN düzeltilmiş karşılaştırıcıyla yeniden karşılaştırıldı: çıkış 0, 100 875 alan eşit, baytlar eşit. Diğer bütün bayt girdileri aynı. Resmi RUN çifti düzeltmeden sonra temiz klonlardan yeniden koşturuldu (D1.7 kaydı).
- **Gözden geçirici onayı:** D1.7.review.md.
