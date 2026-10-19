# P2-0.2 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir; satırın notu `P2-0.2/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · X2 · `scripts/verify.sh` — `pair`'de yük altında çift (`VERIFY_LOAD=1`); D-C'nin T3'ü
- **Ne:** `pair()` `VERIFY_LOAD=1` ise iki zincir boyunca mantıksal çekirdek sayısı kadar (`nproc`, macOS'ta `sysctl -n hw.ncpu`) `yes >/dev/null` başlatır, zincirler bitince durdurur (ASSUMPTIONS X0-13); karşılaştırma aynen. Başka satır değişmedi (`git diff`: yalnız `pair()` başındaki yük bloğu, yük durdurma ve yorumu). D-C'nin T3'ü (ASSUMPTIONS X0-9: `apart` ve `same_doc_after` tiklerde ajan, insan alımı günlüğe girince yazar; `collide` aynen) **yeni** dosyada, `research/eval/run_p2_0_2.py` (`run_d_c`, `wait_ingested`); P2-0.1'in dondurulmuş `run_p2_0_1.py`'si ve P2-0'ın üreteci (`p2_0_sets.d_c`) değişmedi; zincire bağlanması X5'te (ayrı istisna).
- **Neden:** TASKS X2; GOAL P2-0.2 T3, H-P2-0.2i; P2-0.1 doğrulaması F2/Ö2 (D-C `apart` tiklerde iki dosyanın gözlem sırası yarışı; karşılaştırıcı gevşetilmez).
- **Önce/sonra bit eşitliği:** (X2.md) T3'lü (ve X2-1'li) D-C'nin `events.jsonl`'ı, P2-0.1 resmi RUN'ının (`20261019T001516Z-21b16ea`) D-C `events.jsonl`'ıyla **`seq` dışında aynı olay çoklu kümesi** (`jq -cS 'del(.seq)' | sort` eşit): T3 yalnız `apart` tiklerdeki iki alımın sırasını sabitler, hiçbir olayı değiştirmez; `collide` tiklerin olayları bit eşit. `attestations.jsonl`: X2-1'den önce P2-0.1'inkiyle bayt eşit; X2-1'le (tasdik yazımdan önce, ASSUMPTIONS X2-1) yalnız 7 `collide` satırına `matched: false` eklenir; `attestation_id`, `at_seq` ve satır sırası P2-0.1'inkiyle aynı (`del(.matched)` sonrası bayt eşit). `VERIFY_LOAD` ayarlı değilken `pair()` öncekiyle aynı yoldan geçer. D-C doğruluğu her koşuda 340/340.
- **Gözden geçirici onayı:** X2.review.md.

## F2 · X3 · uyumluluk değerlendiricileri: `core/tests/conformance/eval_c2.rs`, `core/tests/conformance.rs`, `core/tests/conformance_c2_gen.rs`, `index/tests/conformance_c3.rs` — C2-14 (spec 2.0.2, K-P2-02.3)
- **Ne:** `eval_c2.rs`: `attest` işlemi `renamed_from`, `link_update_of`, `link_update_from` alır ve başvuru sunucusunun denetimini yapar (`link_caps`: `rootmarkcore::attest::check_link_update` eski yolun ya da belgenin başına karşı; başka notta değişen blok `canonical`; reddedilen tasdik hata) ve tavanları tasdik kimliğine bağlar; `external`'ın birebir eşleşmesi tavanlı tasdikte `plan_ingest_carried` + `Caps::within` ile alır (aşan plan hata: tasdiksiz alınacaktır); geç yükseltme tavanlanır (`cap_raises`). Var olan işlemlerin gözlemleri aynen. `conformance.rs`: `LEVEL2` 14 test, raporun `spec_version` `"2.0.2"`, başlık yorumu. `conformance_c2_gen.rs`: dizin sayısı 14. `conformance_c3.rs`: `LOWER_TOTAL` 30 → 31.
- **Neden:** GOAL P2-0.2 H-P2-0.2j ("mevcut 38 + yeni fixture'lar (T1/T2 için en az biri)"); spec 2.0.2 (SPEC_EDITS, K-P2-02.3).
- **Kapsam (X3 gözden geçirmesi R6):** değerlendirici başvuru sunucusunun yalnız C2-14'ün sınadığı yollarını modeller; şunlar kapsam dışıdır ve ileride bir fixture sınarsa yeni bir dondurma istisnası gerekir: (i) `link_update_from` verilmediğinde eski yolun kabul edilmiş bir yeniden adlandırma tasdikinden çıkarılması, (ii) geç bağlantı güncellemesinin değişiklikten önceki başa karşı denetimi, (iii) tavanı aşan planın tasdiksiz alınması (değerlendiricide hata), (iv) tavansız tasdikin bekleyen tavanı devralması (X1 B3).
- **Önce/sonra bit eşitliği:** X3.md (eski 38 testin `id`, `result`, `actual_hash`, `expected_hash`'i P2-0.1 resmi RUN'ının `conformance_p1d1.json`'uyla `jq -S` sonrası eşit; `scripts/gen_fixtures_c2.sh` var olan fixture'ların `expected.json`'unu bayt bayt aynı bıraktı: `git status spec/` yalnız `MANIFEST.json` ve yeni `C2-14/`).
- **Gözden geçirici onayı:** X3.review.md.

## F3 · X3 · `spec/fixtures/**` — C2-14 fixture'ı ve MANIFEST
- **Ne:** yeni dizin `spec/fixtures/C2-14/` (`case.c2.json`, sekiz belge, `expected.json`); `MANIFEST.json`'a yalnız yeni dosyaların satırları eklendi.
- **Neden:** F2 ile aynı.
- **Önce/sonra:** F2 kaydı; `git diff --stat spec/fixtures`: yalnız `MANIFEST.json`'a eklenen satırlar.
- **Gözden geçirici onayı:** X3.review.md.

## F4 · X3 · uyumluluk sayısı 38 → 39: `scripts/test_tamper_p1c.sh`, `scripts/test_tamper_p1d1.sh`, `scripts/release_local.sh`, `scripts/release_target.sh`, `scripts/release_check.sh`
- **Ne:** her betikte uyumluluk toplamının denetimi `38` → `39` ve sürüm notu yorumları ("39/39 since spec 2.0.2; 38/38 for 2.0.1, …"). Başka satır değişmedi.
- **Neden:** C2-14 (F2) toplamı 39 yapar; bu betikler 38'i sabit denetler (P2-0.1/F3'ün aynısı).
- **Önce/sonra:** değişiklik yalnız sabit; `bash scripts/test_tamper_p1c.sh`, `test_tamper_p1d1.sh` X3 kalite kapılarında koşar (X3.md).
- **Gözden geçirici onayı:** X3.review.md.
