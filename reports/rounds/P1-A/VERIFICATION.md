# P1-A — Bağımsız Doğrulama Raporu

- **Doğrulanan commit:** `35293d3560f4d43585fe888dbe24425397292f9d` (READY.md commit'i; klon HEAD'i bununla eşit, çalışma ağacı temiz)
- **Ölçülen kod commit'i (resmi RUN):** `27d8ae8` · resmi RUN `20261013T221822Z-27d8ae8`
- **Tarih:** 2026-10-14 (UTC ~07:00–07:20)
- **Ortam:** macOS 27.0.1 arm64 · rustc 1.92.0 (ded5c06cf 2025-12-08) · cargo 1.92.0 · Python 3.13.5 (sistem), 3.12.9 (`research/.venv`, uv 0.11.8) · node v24.3.0
- **Yöntem:** Ana çalışma dizininden yalnız `git clone`. Temiz klon `…/scratchpad/verify-P1-A/repo`; zincirler `A/`, `B/`; kasıtlı bozmalar `v6/`, `v7/`, `v8/` altında ayrı kopyalarda yapıldı ve her testten sonra kopya silindi ya da geri alındı. Asıl klon temiz (`git status --porcelain` boş). Alan alan karşılaştırmalar uygulayıcının `compare_p1a`/`compare_runs` araçlarına ek olarak doğrulayıcının kendi betiğiyle (`scratchpad/verify-P1-A/cmpjson.py`) de yapıldı: JSON yaprakları tam eşitlikle karşılaştırıldı, yalnız gecikme yaprakları (`ms`, `latencies_ms`, `p50/p95`) hariç tutuldu.

Etiketler: `[OK] [FARK] [ŞÜPHE] [İHLAL]`

---

## V1 — Çift koşturma `[OK]`
**Komut:** `bash repo/scripts/verify.sh pair <scratch>/repo <scratch>/A <scratch>/B` (remote = temiz klon @35293d3)
- Çıkış kodu **0**. A zinciri: RUN `20261014T070229Z-35293d3` (518 s). B zinciri: RUN `20261014T071110Z-35293d3` (533 s).
- `pair_compare.json`: `max_relative_difference = 0.0`, `n_fields = 1790`, 11 dosya, `A-4_equal = true`.
- Kendi karşılaştırmam, A↔B, 12 dosya (9 regresyon dosyası + holdout, split_grid, p1a_checks): 1871 kalite yaprağı, **0 fark**. Yalnız gecikme yapraklarında fark var (409 yaprak, beklenen).
- A↔resmi RUN (`reports/rounds/P1-A/runs/20261013T221822Z-27d8ae8`): `compare_p1a` çıkış 0, 1790 alan, maks. göreli fark 0.0. Kendi karşılaştırmam: 11 dosya, 1781 kalite yaprağı, **0 fark**.
- Her iki RUN'da tüm 12 faz `worktree_clean = true`.
- A RUN'ından `make report` ile üretilen FINAL.md, resmi FINAL.md'den yalnız RUN kimliği, commit ve A-4 satırında ayrılıyor; A-4 satırı farklı çünkü pair_compare tek zincirde yok.

## V2 — Uyumluluk `[OK]`
**Komutlar:** A zincirinde `make conformance`; ayrı klonda (`v2gen/`) `spec/fixtures/C1-*/expected.json` ve `MANIFEST.json` silindi, ardından `bash scripts/gen_fixtures.sh` çalıştırılıp `git status` alındı.
- `A/repo/reports/conformance.json`: `level_claimed 1`, **17/17**, her testte `result = "pass"`, `fixtures_manifest = "ok"`. B'de de 17/17.
- Test kimlikleri C1-01..C1-17, `spec/CONFORMANCE.md` tablosuyla birebir.
- Yeniden üretim v3-final çekirdeğinden yapıldı. Klondaki `v3-final` etiketi = `71c244e551258601bfd5550b61d1aa7335a938fa` = SOURCE.md'deki sha = subtree birleştirme commit'inin (`e561688^2`) sha'sı. Prototip deposunda (`/Users/arkns/Works/BSv2`, yalnız `rev-parse`) `v3-final` ve `v2-final` sha'ları aynı. Üretimden sonra `git status --porcelain` **0 satır**: 17 `expected.json` ve `MANIFEST.json` bayt bayt aynı yeniden üretildi.
- `spec/fixtures/MANIFEST.json` blake3[:32] = `ea3d8539e7a44d08a78c7812b8403146` (SOURCE.md ile eşit). 84 dosyalık listede uyumsuz hash 0. `core.sha` = v3-final.
- Sınır: üreteç, uygulayıcının `core/tests/conformance/eval.rs` değerlendiricisini kullanıyor. Doğrulama bu dosyanın v3-final çekirdeğiyle aynı çıktıyı verdiğini gösteriyor; değerlendiricinin spec'i doğru yorumladığını ayrıca kanıtlamıyor.

## V3 — Hash'ler `[OK]`
**Komutlar:** `git archive v3-final Cargo.toml Cargo.lock rust-toolchain.toml .cargo core` ile ayrı dizinde `cargo build --offline --release --bin mdk` (v3-final `mdk`). Mevcut çekirdekten de ayrı target dizininde `mdk` derlendi. Her iki ikili ve her matcher için `mdk --matcher <m> ingest-dir V spec/fixtures/mini_vault`, ardından `… ingest-dir V core/tests/fixtures/v3_final/edit` ve `cmp`.
- v3-final ikilisi: ilk geçiş `events_hash` v2 = `8914f9b418ae360d49c85c951050f432`, v3 = `808a41f21a43e121d7d688fee9f5bc24` (GOAL A-5 ile eşit).
- Mevcut çekirdek ikilisi: aynı iki hash.
- `cmp`: v3-final'in ürettiği `events.jsonl` ile depodaki `core/tests/fixtures/v3_final/mini_vault_edit.{v2,v3}.events.jsonl` **eşit**. Mevcut çekirdeğin `events.jsonl` dosyası ile v3-final'inki **eşit** (v2 ve v3).
- Bağlamalar üzerinden (A RUN `p1a_checks.json` `A-5.events_hash`) aynı iki hash.
- `spec/fixtures/mini_vault` ile `v3-final:tests/fixtures/mini_vault` arasında `diff -r` farksız.

## V4 — Regresyon `[OK]`
**Komut:** zincir içinde `make regress RUN=20261014T070229Z-35293d3` (çıkış 0, `compare_runs --set v3` → `regress_vs_v3final.json`: 1577 alan, maks. göreli fark 0.0, `H4-2_equal true`). Ek olarak kendi betiğim: `cmpjson.py research/reports/runs/20261013T182004Z-f82dabd <RUN_A>` üzerinde 9 dosya.
- 9 dosyada (h1s, h1s_tuning, h1r, h1s_v3, h1s_v3_tuning, h1r_v3, h3_v3, h4_v3, h4_v2repro) **1570 kalite yaprağı, 0 fark**, tek tarafta kalan anahtar 0. Gecikme dışındaki h3 alanları da (hit@1/hit@10, reembedded_chunks …) eşit.
- Referans RUN dizinleri (`20261013T154851Z-f133d0b`, `20261013T182004Z-f82dabd`) `v3-final:reports/runs/…` ile `diff -rq` farksız.

## V5 — Holdout `[OK]` (bir şüphe notuyla, bkz. Ş1)
**Komutlar:** A'daki yeniden üretilmiş `data/{d1,d1h,d1ho,d1sh}/docs.json` üzerinde kendi Python kesişim betiğim (kaynak yolu ve `v00.md` içeriğinin sha256'sı); `git log` sırası.
- Kaynak kümeleri: easy 200, hard 200, holdout 100. holdout∩easy **0**, holdout∩hard **0**, easy∩hard **0**, üçü **0**. Aynı sonuç `v00.md` metin hash'leriyle de çıktı. Bilgi: split_hard∩holdout = 14; üreteçte belgelenmiş ve izinli ("may share base texts").
- Holdout `MANIFEST.d1ho.json` A ve B'de doğrulandı (`make datasets`).
- A-2 eşikleri (A RUN `holdout.json`, B'de ve resmi RUN'da aynı): H1-H1 **0.993540** ≥ 0.90 · H1-H2 **0.000000** ≤ 0.03 · H1-H3 **0.979381** ≥ 0.70 · H1-H4 **1.000000** ≥ 0.90 → **geçer**. Eşleyici, derlenmiş v3 değerlerinde çalıştı (`matcher_defaults("v3")`).
- Git sırası: eşleyici kodu (`core/src/lineage.rs`) son kez A2'de `e622d93` commit'inde değişti (yalnız `layer` alanı; v3-final'e göre fark 2+/1−, davranış-nötr). Sabitler en son prototipte `b017755` (T24) commit'inde değişti. Holdout üreteci `synthetic_edits_v3.py` ve `MANIFEST.d1ho.json` ilk kez A5'te `3cbc1e6` commit'inde girdi; `e622d93` bu commit'in atasıdır. Sıra doğru: önce eşleyici, sonra holdout.
- Gerçekleşen op dağılımını truth dosyalarından kendim saydım (1000 puanlı adım). Sonuç FINAL ile birebir: split_edit 0.049 (hedef 0.15), split_three 0.020 (hedef 0.05), dup_delete 0.134 vb. → bkz. **Ş1**.

## V6 — Dondurma `[OK]` (bir şüphe notuyla, bkz. Ş2)
**Tur başlangıcı:** `ae9799f` (tur yönergeleri) → `56135fd` (P1-A/A0 başlangıç kaydı). GOAL.md, TASKS.md, VERIFY.md, spec/SPEC.md, spec/CONFORMANCE.md ve NORTH_STAR.md yalnız `ae9799f` commit'inde var; tur içinde değişmemiş.
- `bash scripts/check_frozen.sh` → çıkış 0, 25 satır OK.
- Kendi denetimim: dondurma commit'lerinden sonra dondurulmuş yolları değiştiren commit'leri `git show --name-status` ile listeledim. Hepsi yalnız `A` (yeni dosya); dondurulmuş bir dosyada `M`/`D` yok. `scripts/verify.sh` ve `test_tamper.sh`, A6 dondurmasından önce `bede274` (A6-pre) ve `d1a9ba7` (A6-pre2) commit'lerinde değişmiş. Bu, FROZEN.md'deki dondurma noktasıyla (A6) tutarlı.
- `research/eval/**` v3-final `eval/` ile karşılaştırıldı: yalnız `freeze_check.py` ve `freeze_check_v3.py` farklı. Fark, FREEZE_EXCEPTIONS F1'de belgelenmiş ve gözden geçirici onaylı: parametreler `params.json` dosyasına taşınmış, değerler aynı. Geri kalanlar yeni dosyalar. Veri manifest'leri (d1, d1h, d1k4, d2, d3), d4 ve splits v3-final ile aynı.
- FROZEN.md tablosunun geçmişinde yalnız satır eklenmiş; satır silinmemiş, gevşetilmemiş.
- **Sabit-bağımsızlık (kasıtlı, `v6/` kopyası):** `core/src/lineage.rs` sabitleri THETA 0.97, THETA_SPLIT 0.98, THETA_V3 0.96, THETA_SPLIT_V3 0.99 yapıldı; `make setup` ile yeniden derlendi (`matcher_defaults` değişti). Sonuçlar:
  - `make freeze-check` → çıkış 0, `d489a0ed44b15d3aeb272e16cdd4a5af` (değişmedi)
  - `make freeze-check-v3` → çıkış 0, `d138cddcde1d0978a020a6fe8df2f610` (değişmedi)
  - `make conformance` → 17/17
  - Kontrol: v3-final `freeze_check.py` aynı kopyada **çıkış 1** verdi ("output differs"). Değişikliğin etkili olduğu ve testin ayırt edici gücü olduğu görüldü. Not: ilk denemede kullanılan 0.33/0.44/0.77/0.88 değerleri fixture çıktısını değiştirmedi; bu değerlerde test güçsüz kalıyor.
- **Hash/parametre değişince denetleyici hata vermeli (her biri ayrı, sonra geri alındı):** v2 `params.json` theta 0.6→0.95 → çıkış 2 · v3 `params.json` params[0] 0.6→0.95 → çıkış 2 · v2 `expected.json` 1 bayt → çıkış 2 · v3 `expected.json` 1 bayt → çıkış 2 · v3 fixture vault'ta bir .md dosyası değiştirildi → çıkış 2.
- **check_frozen kasıtlı:** dondurulmuş `run_holdout.py` çalışma ağacında değiştirildi → çıkış 1. Aynı değişiklik commit'lendi → çıkış 1. `research/data/splits/d4.json` silindi → çıkış 1. Ancak FROZEN.md'den ilgili satır silinip dosya değiştirildiğinde → **çıkış 0**. Bkz. Ş2.

## V7 — Bağımlılık yönü `[OK]`
**Komutlar:** temiz klonda `bash scripts/check_deps.sh` → çıkış 0, "check_deps: OK". A ve B RUN `p1a_checks.json` A-6: ihlal 0, `check_deps_ok true`, kasıtlı test true. Kasıtlı ters bağımlılıklar `v7/` altında, her biri ayrı klonda denendi:
| Durum | Çıkış | Yakalayan |
|---|---|---|
| core → `mdkernel-index` ([dependencies]) | 1 | statik tarama + cargo tree |
| index → `mdkernel-server` ([dev-dependencies]) | 1 | statik tarama + cargo tree |
| core → `srv = { package = "mdkernel-server" }` (yeniden adlandırılmış) | 1 | statik tarama + cargo tree (döngü hatası) |
| research `from index import x` | 1 | import tarayıcı |
| research `import mdkernel_index` | 1 | import tarayıcı |
| clients `import … from '../../core/src/lib'` | 1 | import tarayıcı |

## V8 — Kasıtlı bozma `[OK]` (bir şüphe notuyla, bkz. Ş3)
Her durum ayrı kopyada yapıldı, kopyalar sonra silindi. Uyumluluk durumları için kopya taze `git clone` olarak alındı ve kendi `target/` dizini kullanıldı (gerekçe: Ş3).
| Durum | Komut | Çıkış | Kanıt |
|---|---|---|---|
| Fixture beklenen değeri (C1-08 `expected.json` 1 karakter) | `make conformance` | 2 | `spec/fixtures changed (frozen): C1-08/expected.json: blake3 a6aac97a… != manifest "1c093c53…"` |
| Aynı + `MANIFEST.json` uyumlu güncellendi | `make conformance` | 2 | `C1-08: actual output differs from expected.json` |
| Fixture manifest hash'i (`spec/fixtures/MANIFEST.json` ilk hash) | `make conformance` | 2 | `C1-01/case.json: blake3 … != manifest "f9fe…"` |
| Holdout manifest hash'i (`MANIFEST.d1ho.json`, `docs/017/truth.json`) | `python -m eval.run_holdout` | 1 | `data/d1ho does not match MANIFEST.d1ho.json` |
| Holdout truth (op etiketi değişti, manifest aynı) | `run_holdout` | 1 | manifest uyuşmazlığı |
| Holdout truth (eşleme kimlikleri bozuldu + manifest güncellendi) | `run_holdout` | 1 | `TruthError: malformed truth reference` |
| Eksik rapor dosyası `holdout.json` / `split_grid.json` / `h3_v3.json` | `compare_p1a A A'` | 1 | `… missing` |
| Eksik rapor dosyası `holdout.json` / `p1a_checks.json` / `split_grid.json` / `regress_vs_v3final.json` | `make report RUN_DIR=…` | 2 | `FileNotFoundError` (bozulmamış kopyada kontrol: çıkış 0) |
- Bilgi: `compare_p1a`, `p1a_checks.json` ve `manifest.json` eksikken çıkış 0 veriyor. Tasarım gereği yalnız kalite dosyalarını karşılaştırıyor. Bu iki dosyanın eksikliğini `make report` yakalıyor. Kendi karşılaştırmamda A↔B `p1a_checks.json` eşit (90 yaprak).

## V9 — Karar kuralı `[OK]`
Kendi ölçtüğüm sayılarla (A ve B zincirleri ile V2–V7) kural uygulandı:
- A-1 17/17 ✔ · A-2 0.9935 / 0.0000 / 0.9794 / 1.0000 ✔ · A-3 0 fark (1570/1577 alan) ✔ · A-4 0 (1790 alan, kendi sayımımla 1871 yaprak) ✔ · A-5 iki hash eşit ✔ · A-6 0 ihlal ✔ · A-7 farklı amaç değeri **3** (0.45/0.55/0.65 → 3.985714; 0.75 → 3.982079; 0.85 → 3.978417), bilgi.
- Türetilen karar: **A-1..A-6 geçti → spec §1–6 "frozen" damgası teyit edilir, P1-B başlar.** FINAL.md §2 ile **aynı**.

---

## Şüpheler (ayrıntı)
- **Ş1 (V5): holdout op dağılımı D1-hard tablosundan sapıyor.** GOAL kapsamı "D1-hard op dağılımı" diyor. Gerçekleşen dağılımda split_edit 4.9% (hedef 15%), split_three 2.0% (hedef 5%). H1-H3 olay gerçeğinde 69 bölme adımına karşılık 128 birleştirme adımı var; holdout F1'i ağırlıkla birleştirmeyi ölçüyor. Sapma ASSUMPTIONS A5'te ve FINAL §4'te açıkça yazılmış; eşiklerin hiçbiri sınırda değil. Yine de kapsam ifadesine harfiyen uymuyor; bölme davranışı holdout'ta zayıf örnekleniyor.
- **Ş2 (V6): `research/FROZEN.md` kendi dondurma listesinde değil.** Bir satır silinip dondurulmuş dosya değiştirildiğinde `check_frozen.sh` OK veriyor (kopyada denendi, çıkış 0). Geçmişte böyle bir şey yapılmamış; satırlar yalnız eklenmiş. Ama koruma tek bir belge düzenlemesiyle kapatılabiliyor.
- **Ş3 (V8): uyumluluk testinin fixture yolu derleme anında sabitleniyor (`CARGO_MANIFEST_DIR`).** Önceden derlenmiş `target/` ile birlikte kopyalanan bir ağaçta fixture bozuldu; cargo yeniden derlemedi ve test binary'si **asıl ağacın** fixture'larını okudu. Sonuç `make conformance` çıkış 0, 17/17 (yanlış geçti). Taze klonda ve yerinde bozmada hata doğru yakalanıyor. Sorun yalnız taşınmış veya kopyalanmış derleme önbelleğinde ortaya çıkıyor. Sessiz yanlış hedefleme riski taşıdığı için kaydedildi.

## Sonuç
- **İhlal: 0 · Fark: 0 · Şüphe: 3** (Ş1, Ş2, Ş3)
- **Hüküm:** P1-A çıkış kriterleri A-1..A-6 bağımsız olarak yeniden üretildi ve geçti. A-7 = 3 (bilgi). Çift koşturma, resmi RUN ve v3-final referansıyla kalite alanlarında fark yok. Dondurma, bağımlılık yönü ve bozma denetimleri hatayı yakalıyor. Karar FINAL ile aynı: **spec §1–6 "frozen" teyit edilir, P1-B başlayabilir.** Şüpheler hükmü değiştirmiyor.
- **Öneriler (en fazla 3):**
  1. `research/FROZEN.md` dosyasını kendi listesine ekleyin. Ya da `check_frozen.sh`, tablo satırlarının tur içinde silinmediğini veya gevşetilmediğini git geçmişinden denetlesin.
  2. Uyumluluk testi fixture kökünü çalışma anında çözsün (ör. `MDK_FIXTURES` ya da `make conformance` içinden `$(CURDIR)/spec/fixtures` verilsin). Test, `MANIFEST.json` mutlak yolunu ve hash'ini `reports/conformance.json` içine yazsın.
  3. Sonraki eşleyici turunda holdout'u bölme açısından yeterli örnekle genişletin (hedef D1-hard oranına yakın split_edit/split_three) ya da GOAL'de "gerçekleşen dağılım" tanımını açıkça yazın.
