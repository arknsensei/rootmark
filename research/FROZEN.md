# Dondurma listesi (P1-A)

docs/DEV_PROCESS.md §4 ve kök `CLAUDE.md`: aşağıdaki dosyalar, "dondurulduğu commit" sütunundaki commit'ten (konu satırı öneki) sonra tur içinde **değişmez**. İstisna yalnız `docs/rounds/<tur>/progress/FREEZE_EXCEPTIONS.md` girdisi + önce/sonra bit eşitliği (`make freeze-check`, `make freeze-check-v3`, `make conformance`) + gözden geçirici onayı ile. Denetleyicilerin kendisi listededir ve derlenmiş, ayarlanabilir sabitlere bağlı olamaz: parametreleri fixture taşır (`tests/fixtures/freeze_fixture*/params.json`, `spec/fixtures/C1-*/case.json`); bu `make test-freeze-indep` ile sınanır.

Satır biçimi `yol-kalıbı | dondurulduğu commit | not` — `scripts/check_frozen.sh` bu tabloyu okur.

<!-- frozen-table -->
| Yol kalıbı | Dondurulduğu commit | Not |
|---|---|---|
| `research/eval/**` | `P1-A/A4:` | prototip ölçüm kodu (v2 + v3 dondurmaları aynen geçerli); A5'te eklenen dosyalar kendi satırlarıyla |
| `research/tests/fixtures/freeze_fixture/**` | `P1-A/A4:` | v2 dondurma fixture'ı + `params.json` (F1) |
| `research/tests/fixtures/freeze_fixture_v3/**` | `P1-A/A4:` | v3 dondurma fixture'ı + `params.json` (F1) |
| `research/data/MANIFEST.*.json` | `P1-A/A4:` | veri manifest'leri (v3-final ile bayt eşit) |
| `research/data/d4/**` | `P1-A/A4:` | API ile üretilmiş soru seti (H2; bu turda koşturulmaz) |
| `research/data/splits/**` | `P1-A/A4:` | bölünmeler |
| `research/reports/runs/20261013T154851Z-f133d0b/**` | `P1-A/A4:` | v2 nihai RUN (H4-V2 referansı) |
| `research/reports/runs/20261013T182004Z-f82dabd/**` | `P1-A/A4:` | v3 nihai RUN (A-3 referansı) |
| `spec/fixtures/**` | `P1-A/A3:` | C1 uyumluluk fixture'ları (v3-final'den üretildi) |
| `core/tests/conformance.rs` | `P1-A/A3:` | uyumluluk denetleyicisi |
| `core/tests/conformance/eval.rs` | `P1-A/A3:` | uyumluluk değerlendiricisi (üreteçle ortak) |
| `scripts/gen_fixtures.sh` | `P1-A/A3:` | fixture üreteci |
| `scripts/fixture-gen/**` | `P1-A/A3:` | fixture üreteci |
| `scripts/test_freeze_indep.sh` | `P1-A/A4:` | denetleyici sabit-bağımsızlık testi |
| `research/eval/datasets/synthetic_edits_v3.py` | `P1-A/A5:` | holdout + split_hard üreteci |
| `research/eval/run_holdout.py` | `P1-A/A5:` | A-2 koşturucusu |
| `research/eval/run_split_grid.py` | `P1-A/A5:` | A-7 koşturucusu (bilgi) |
| `research/data/MANIFEST.d1ho.json` | `P1-A/A5:` | holdout manifest'i |
| `research/data/MANIFEST.d1sh.json` | `P1-A/A5:` | split_hard manifest'i |
| `research/eval/compare_p1a.py` | `P1-A/A6:` | A-4 karşılaştırıcısı (dondurulmuş `compare_runs.compare` ile) |
| `research/eval/run_p1a_checks.py` | `P1-A/A6:` | A-1 / A-5 / A-6 RUN kaydı |
| `scripts/test_tamper.sh` | `P1-A/A6:` | kasıtlı bozma testleri |
| `research/eval/report_p1a.py` | `P1-A/A7-pre:` | P1-A rapor üreteci (resmi RUN'dan önce dondurulur) |
| `scripts/verify.sh` | `P1-A/A6:` | doğrulama zinciri (A6'da tamamlanır) |
| `scripts/check_frozen.sh` | `P1-A/A6:` | dondurma denetleyicisi |
| `research/FROZEN.md` | `P1-B/B0:` | dondurma listesinin kendisi (Ö1) [append-only]: tablo satırları yalnız eklenir; her `ROUND_START`'tan beri ayrıca denetlenir |
| `scripts/check_frozen.sh` | `P1-B/B0:` | yeniden dondurma P1-B/F1: Ö1 (tablo yalnız-ekleme denetimi, yeniden dondurma kuralı) |
| `core/tests/conformance.rs` | `P1-B/B0:` | yeniden dondurma P1-B/F2: Ö2 (fixture kökü çalışma anında, `fixtures_root`, `fixtures_manifest_hash`) |
| `scripts/test_tamper.sh` | `P1-B/B0:` | yeniden dondurma P1-B/F3: B-7 ve B-8 kasıtlı vakaları |
| `scripts/check_frozen.sh` | `P1-B/B0-fix:` | yeniden dondurma P1-B/F4: tablo işaretçisi ekleme/çoğaltma açığı (B0 gözden geçirmesi) |
| `core/tests/conformance.rs` | `P1-B/B7:` | yeniden dondurma P1-B/F5: Seviye 2 (C2-01..08), `level_claimed` 2 |
| `spec/fixtures/**` | `P1-B/B7:` | yeniden dondurma P1-B/F6: C2 fixture'ları eklendi, `MANIFEST.json` yalnız C2 satırları kazandı (C1 dosyaları bayt eşit) |
| `core/tests/conformance/eval_c2.rs` | `P1-B/B7:` | C2 değerlendiricisi (üreteçle ortak) |
| `core/tests/conformance_c2_gen.rs` | `P1-B/B7:` | C2 fixture üreteci (mevcut çekirdekten) |
| `scripts/gen_fixtures_c2.sh` | `P1-B/B7:` | C2 fixture üreteci betiği |
| `research/eval/datasets/synthetic_edits_v4.py` | `P1-B/B8-pre:` | d1hs üreteci (Ö3), H1 sonucu görülmeden |
| `research/data/MANIFEST.d1hs.json` | `P1-B/B8-pre:` | d1hs manifest'i |
| `research/eval/run_holdout_split.py` | `P1-B/B8-pre:` | B-9 koşturucusu (eşikler kodda) |
| `scripts/verify.sh` | `P1-B/B9-pre:` | yeniden dondurma P1-B/F7: P1-B doğrulama zinciri (Seviye 2, regress + d1hs, p1b-checks, tamper, compare_p1b) |
| `research/eval/run_p1b_checks.py` | `P1-B/B9-pre:` | B-1/B-2/B-3/B-5/B-6/B-10 RUN kaydı |
| `research/eval/compare_p1b.py` | `P1-B/B9-pre:` | B-4 karşılaştırıcısı (alanlar + bayt) |
| `research/eval/report_p1b.py` | `P1-B/B9-pre:` | P1-B rapor üreteci (resmi RUN'dan önce) |
| `scripts/governance_scenario.sh` | `P1-B/B9-pre:` | B-4 yönetişim senaryosu |
| `scripts/test_policy_bounds.sh` | `P1-B/B9-pre:` | B-6 betiği |
| `scripts/check_v3_compat.sh` | `P1-B/B9-pre:` | B-5 betiği (v3-final ikilisi) |
| `scripts/verify.sh` | `P1-B/B9-pre2:` | yeniden dondurma P1-B/F8: P1-A zincir adımları ve `compare` anlamı korunur, `pair` `compare-p1b` kullanır |
| `scripts/check_frozen.sh` | `P1-B.1/B1.0:` | yeniden dondurma P1-B.1/F1: noktalı tur kimliği (`P1-B.1/F<n>`) istisna atfında |
| `spec/fixtures/**` | `P1-B.1/B1.2:` | yeniden dondurma P1-B.1/F2: C2-07 beklenen değeri ve `MANIFEST.json` satırı (K-B1.2, spec 1.0.1) |
| `core/tests/conformance.rs` | `P1-B.1/B1.2:` | yeniden dondurma P1-B.1/F3: `spec_version` 1.0.1 |
<!-- /frozen-table -->

## Notlar
- **Yeniden dondurma (P1-B, Ö1):** tur görevi dondurulmuş bir dosyayı değiştirmek zorundaysa tabloya aynı kalıpla **yeni bir satır eklenir** (eski satır silinmez, değişmez); yeni satırın commit'i eskisinin torunu olmalı ve notu `docs/rounds/<tur>/progress/FREEZE_EXCEPTIONS.md` içindeki bir girdiyi (`<tur>/F<n>`) anmalıdır. Denetleyici o kalıbı yeni commit'ten itibaren denetler. Tablo her `docs/rounds/*/progress/ROUND_START` sha'sından beri yalnız satır kazanabilir (`git log -p` + uç karşılaştırması).
- `research/eval/**` altında A5'te eklenen dosyalar (`datasets/synthetic_edits_v3.py`, `run_holdout.py`, `run_split_grid.py`) ve yeni manifest'ler A5 commit'inden itibaren kendi satırlarıyla dondurulmuştur.
- Bir satır yalnız dondurulduğu commit'te **var olan** dosyaları kapsar; sonradan eklenen dosyalar kendi satırlarıyla dondurulur. Satırın commit'i henüz yoksa (görev commit'lenmemiş) satır "henüz dondurulmadı" diye raporlanır.
- `research/docs/progress/**` ve `research/reports/{commentary.md,v3/commentary.md}` prototipin tarihsel kayıtlarıdır; yalnız dondurulmuş rapor üreteçleri (`eval/report*.py`) ve onların testleri için `v3-final`'den aynen geri yüklendi.
