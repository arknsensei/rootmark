# P2-L — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, L5'te (K-P2.4); F1 ve F2 o turda onaylanır.

## F1 · L0 (e) · `scripts/release_check.sh`, `scripts/release.sh` — 0.2.0 ve Linux resmi hedef
- **Ne:** `release_check.sh`: R1 sürümü betikte yazılı `0.1.0` yerine `CHANGELOG.md`'nin en yeni `## <sürüm>` başlığıyla karşılaştırır (bütün sürüm dizgileri eşit kalır); R4 zorunlu hedefleri `["aarch64-apple-darwin", "x86_64-unknown-linux-gnu"]` (K-P2A.6) ve her zorunlu kaydın sürüm, spec sürümü, 39/39, sha256'sı; R13 `release.yml`'de Linux `required: true`; R16 `## Hüküm` başlıklı rapor biçimini okur ve "Şu koşullarla güvenilir" hükmünü, insan adımı 1 raporu koşulu kapatan kararla andığında ve `DECISIONS.md`'deki o karar raporu andığında kabul eder (adım 1'in andığı her rapor geçmeli). R2, R3, R4a, R5–R12, R14, R15, R17–R20'nin komutları bayt bayt aynı. `release.sh`: Linux kaydı `required` `true` (üç çağrı) ve başlık yorumu.
- **Neden:** GOAL P2-L 1 (e), K-P2.4 (4) (0.2.0 yayımı, insan onayı), K-P2A.6 (Linux resmi hedef: "Etki: `release.yml`, RELEASE.md R4/R13"); 0.2.0'ın dayandığı doğrulamalar (P2-A, P2-0.3) "şu koşullarla güvenilir" dedi ve koşullarını K-P2A.5 ve K-P2-03.1 kapattı — eski R16 yalnız "Güvenilir" satırını okurdu (P2-0'dan sonraki raporlar `## Hüküm` başlığı kullanır).
- **Önce/sonra:** `git diff main -- scripts/release_check.sh`: yalnız R1, R4, R13, R16 blokları; 0.1.0 ağacında (sürüm `0.1.0`, `## 0.1.0` başlığı) yeni R1 eskisiyle aynı sonucu verir. Zincirde (`make p1e1-checks`, `ROOTMARK_RELEASE_LINUX=0`) Linux kaydı artık `required: true, skipped` olur: `release.sh` çıkışı 1 ("recorded") ve R4 FAIL — bu, K-P2A.6'nın bildirilen etkisidir ve P2-L'nin sınıflandırıcısında bu kararla etiketlenir (L5). Yayım için `release.sh` Linux'u Docker'da kurar ve sınar (`release_check` temiz; `reports/rounds/P2-L/release_0_2_0.json`).
- **Gözden geçirici onayı:** L5.review.md.

## F2 · L0 (e) · `clients/obsidian/e2e/real/run.ts` — durum çubuğu sürümü ikiliden, E1-9 `migration_required`
- **Ne:** gerçek Obsidian e2e'si (K-E1.3, R15) durum çubuğunda beklenen metni `"rootmark 0.1.0 · human:editor"` diye yazıyordu; artık sınanan ikilinin `rootmark --version` çıktısından kurar (`${VERSION} · human:editor`, iki yerde daha); çıktı `rootmark x.y.z` biçiminde değilse hata. E1-9 adımı, eski biçimli (`mdkernel-index/2`) indeksle yeniden kurmadan önceki aramanın hatasını `index_stale: ` yerine `migration_required: ` ile başlar bekler (K-P2A.2: eski bir rootmark'ın indeksi P2-A'dan beri `migration_required` döner; P2-A zincirinde bu adım `migration_required_fails.json`'da bu kararla etiketli bilinen FAIL'di). Başka satır değişmedi.
- **Neden:** sürüm 0.2.0 olunca adım 1 ve E1-9 adımı yanlış nedenle kalırdı; E1-9 ayrıca K-P2A.2'den beri bilinen nedenle kalıyordu; R15 (yayım) ikisinin de geçmesini ister.
- **Önce/sonra:** 0.1.0 ikilisiyle beklenen metinler bayt bayt aynı (`rootmark 0.1.0 · human:editor`, `… · index stale`); `git diff main -- clients/obsidian/e2e/` yalnız bu dosyanın beş yeri; E1-9'un `index_stale` beklentisi K-P2A.2'den sonra hiçbir ikiliyle geçmezdi. Kanıt: `release.sh` içindeki gerçek Obsidian e2e'si 0.2.0 ikilisiyle 11/11 (`dist/plugin_e2e_real/plugin_e2e_real.json`, `release_0_2_0.json`).
- **Gözden geçirici onayı:** L5.review.md.

## F3 · L0 (d) · `scripts/verify.sh` — hızlı doğrulama kipi (`make verify FAST=1`)
- **Ne:** iki yeni işlev (`chain_fast`, `pair_fast`), iki yeni alt komut (`chain-fast`, `pair-fast`), başlık yorumuna bir paragraf; varsayılan dal `FAST=1` iken `pair_fast`'ı, değilse `pair`'i (eskisi gibi) çağırır; kullanım satırı. Hızlı zincir: temiz klon, `make setup`, `make models`, `make d3`, `check_frozen`, `check_deps`, `make conformance` + `rootmark conformance` + `conformance_equiv`, C-3 kalitesi, `make fast-regress` (İngilizce kalite referansının 618 değeri ve uyumluluk raporu test kimliğiyle son resmi RUN'a, P2-0.3'ün `20261024T000230Z-5632b3b`'sine karşı; `eval.fast_regress`), aktif turun kontrolleri (`make round-checks` → `ROUND_CHECKS`, bu turda `p2l-checks`); iki hızlı zincir yan yana koşar, `eval.compare_p2l fast` ile karşılaştırılır, geçen süre `fast_pair_time.json`'a yazılır.
- **Neden:** GOAL P2-L 1 (d), K-P2.4 (3) ("dondurulmuş `scripts/verify.sh`'e kip eklenmesi istisna prosedürüyle (P2-L/L0)"); tam zincir değişmez ve haftalık CI'da kalır.
- **Önce/sonra:** `L0_f3_equal.sh` → `L0_f3_equal.out`: `chain_fast`/`pair_fast` dışındaki bütün işlevlerin metni `main`'dekiyle bayt bayt aynı (sha256 eşit); silinen iki satır varsayılan dalın `pair` çağrısı (artık `else` kolunda, aynı metin) ve kullanım satırı. `FAST` boşken davranış aynıdır.
- **Gözden geçirici onayı:** L5.review.md.

## F4 · L0 (d) · `research/eval/check_compare_fields.py`, `research/eval/DETERMINISTIC_FIELDS.md` — `compare_p2l`
- **Ne:** alan denetimi yeni karşılaştırıcıyı tanır (`("compare_p2l", "P2L")`, dosya sabitleri `P2L_FAST_FILES`, `P2L_FILES`); `DETERMINISTIC_FIELDS.md`'ye `compare_p2l` bölümü, alan satırı (`fast/conformance.json` `fixtures_root`, ortam) ve iki dosya satırı. Bu commit'te var olan yeni P2-L ölçüm dosyaları (`compare_p2l.py`, `fast_regress.py`, `run_p2l_checks.py`) `research/eval/**` satırıyla donar; turun test listesi Makefile'dadır (`P2L_TESTS`), sonraki görevler dosyayı değiştirmeden ekler.
- **Neden:** emsal P2-A/F6; hızlı kipin karşılaştırıcısı belirlenimli alan sözleşmesine girer (K-E0.2).
- **Önce/sonra:** `check_compare_fields.py`'den silinen satır 0, eklenen iki satır (`L0_f3_equal.out`); `python -m eval.check_compare_fields` geçer (`unmatched: 0`). Zincirde `p1e/check_compare_fields.json` ve `p1e1/…`'nin `modules` ve `file_constants`'ı yeni modülü görür — bu kararın bildirilen etkisidir (P2-L sınıflandırıcısı, L5).
- **Gözden geçirici onayı:** L5.review.md.

## F5 · L0 (a) · `core/tests/conformance.rs` — `spec_version` 2.0.3
- **Ne:** raporun `spec_version` dizgisi `"2.0.2"` → `"2.0.3"`; başka satır değişmedi (seviye, test listesi, `LEVEL2` aynı).
- **Neden:** spec 2.0.3 (SPEC_EDITS E1, E2; CRITERION "yama"); R2 `SPEC_VERSION` = `core/tests/conformance.rs` `spec_version` = `spec/SPEC.md` başlığı ister. Emsal P2-0/F7, P2-0.1/F1, P2-0.2/F2.
- **Önce/sonra:** `git diff main -- core/tests/conformance.rs` tek satır; `make conformance` 39/39 seviye 3, her testin sonucu ve hash'leri son resmi RUN'ınkiyle eşit (`fast/regress.json` `conformance.differing` boş; spec sürümü test kimliğiyle karşılaştırmaya girmez).
- **Gözden geçirici onayı:** L5.review.md.

## F6 · L5 · `scripts/verify.sh` — P2-L adımı, sınıflandırıcı, çift karşılaştırıcısı
- **Ne:** tam zincire `make p2-0-3-checks`'ten sonra iki adım (`make p2l-checks`, `make p2l-quality`); FAIL özeti `eval.historical_p2a` yerine `eval.historical_p2l` (P2-0.3'ün resmi RUN'ına karşı); `pair` `compare_p2a` yerine yeni `compare_p2l` işlevi (`compare_p2l pair`: P2-A çift kümesi + P2-L dosyaları); `compare-p2l` alt komutu; başlık yorumu. Hızlı kipin işlevleri değişmedi.
- **Neden:** TASKS L5 ("`verify.sh`: P2-L adımı (L-8 üç vault, L-9 tam çift bir kez, L-10), `compare_p2l` belirlenimli alanlar"); emsal P2-A/F5, P2-0.3/F1.
- **Önce/sonra:** `L5_f6_removed.out`: L0 sürümünden silinen satırlar yalnız sınıflandırıcının iki satırı, `pair`'in karşılaştırıcı çağrısı ve kullanım satırı; öteki adımlar bayt bayt aynı (yalnız ekleme). `compare_p2a` işlevi ve `compare-p2a` alt komutu korunur.
- **Gözden geçirici onayı:** L5.review.md.

## F7 · L5 · `research/eval/compare_p2l.py`, `research/eval/DETERMINISTIC_FIELDS.md` — P2-L ölçüm dosyaları
- **Ne:** `P2L_FAST_FILES` ve `P2L_FILES`'a `p2l_quality.json`, `p2l/d_tr.json`, `p2l/english.json`, `p2l/migrate.json`; `DETERMINISTIC_FIELDS.md`'nin `compare_p2l` bölümü ve dosya satırları. Hariç alan eklenmedi (ölçümler belirlenimlidir). `Makefile` `ROUND_CHECKS` `p2l-checks p2l-quality` (hızlı kip turun ölçümünü de koşar; TASKS L4 "`make verify FAST=1` adımı").
- **Neden:** L-9 (tam çift: iki varyant ve iki embedder ile 12 hücre fark 0) ve hızlı kipin aynı dosyaları karşılaştırması.
- **Önce/sonra:** `compare_p2l.py`'de yalnız iki liste uzadı; `python -m eval.check_compare_fields` geçer (15 karşılaştırıcı, `unmatched: 0`).
- **Gözden geçirici onayı:** L5.review.md.
