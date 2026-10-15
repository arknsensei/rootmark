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

## F6 · C1.4-pre · `scripts/verify.sh` — P1-C.1 zinciri
- **Ne:** `chain`'e P1-C kasıtlı bozmalarından sonra `make p1c1-checks RUN=…` (+ `p1c1_cli.log`'un son satırı); `pair`'in karşılaştırması `compare_p1c` → yeni `compare_p1c1` işlevi (`eval.compare_p1c1 pair`); yeni `compare-p1c1` modu; başlık yorumu; kullanım metni. `compare_p1c`, `compare_p1b`, `compare` işlevleri ve modları aynen duruyor; P1-A/P1-B/P1-C zincir adımlarının hepsi aynı sırada ve aynı komutlarla (`git diff p1c-final -- scripts/verify.sh`: silinen 2 satır = `pair`'deki çağrı ve kullanım metni).
- **Neden:** GOAL C1.4 "verify.sh genişletme (C1-2..C1-6 kasıtlı testler dahil)"; C1-8 çift koşturma.
- **Önce/sonra:** `compare_p1c1 pair`, `compare_p1c pair`'in her dosyasını (`compare_p1c.PAIR_FILES`) aynı dondurulmuş yaprak çıkarımı ve alan karşılaştırmasıyla, hariçsiz karşılaştırır ve `c1_variant.json`, `p1c1_checks.json`'u ekler (üst küme); bayt yapıtları `compare_p1b.bytes_equal` ile aynı. Geliştirme zinciri: C1.4.md.
- **Gözden geçirici onayı:** C1.4.review.md.

## F7 · C1.4-pre · `scripts/test_tamper_p1c.sh` — Seviye 3 toplamı 31
- **Ne:** `conf()` içindeki `.total == 30` → `.total == 31` (tek satır).
- **Neden:** C3-05 (K-C1.3, F2/F4): bozulmamış fixture kopyası artık 31/31; eski sayı "untouched fixture copy fails conformance" ile betiği durdururdu.
- **Önce/sonra:** `bash scripts/test_tamper_p1c.sh` → 6/6 bozma reddedildi (P1-C'deki altı vaka, aynı ret nedenleri).
- **Gözden geçirici onayı:** C1.4.review.md.

## F8 · C1.4-fix · `research/eval/compare_p1c1.py` — ortak yaprakların sıralanması
- **Ne:** `compare()` içinde `sorted(set(la) & set(lb))` → `sorted(…, key=C._key)` (tek satır).
- **Neden:** geliştirme zinciri `regress`'te `TypeError: '<' not supported between instances of 'str' and 'int'` ile durdu: yaprak yolları dizgi ve liste indeksi karıştırır (`compare_runs.leaves`). Ölçüm henüz hiç sonuç üretmemişti (çökme).
- **Önce/sonra:** önceki sürüm hiçbir RUN'da sonuç üretmedi; sıralama yalnız karşılaştırma sırasını belirler, hangi alanların karşılaştırıldığını ya da sonucunu değil. `research/tests/test_p1c1_tools.py::test_leaves_with_lists_and_dicts` (liste + sözlük yaprakları) eklendi; 5/5.
- **Gözden geçirici onayı:** C1.4.review.md.
