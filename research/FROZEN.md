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
| `scripts/check_v3_compat.sh` | `P1-B.1/B1.3:` | yeniden dondurma P1-B.1/F4: v3-final kütüphane API'siyle karışık yazarlı 9 senaryo (eski senaryo çıktısı bayt eşit) |
| `scripts/gen_v3final_mixed.sh` | `P1-B.1/B1.3:` | v3-final karışık yazarlı vault üreteci betiği (B1-1) |
| `scripts/v3final-mixed-gen/**` | `P1-B.1/B1.3:` | üreteç kaynağı + `scenarios.json` (`expect_blame` dahil) |
| `core/tests/fixtures/v3_final_mixed/**` | `P1-B.1/B1.3:` | v3-final'den üretilmiş 9 senaryo fixture'ı |
| `research/eval/compare_p1b1.py` | `P1-B.1/B1.4-pre:` | B1-6 karşılaştırıcısı (P1-B resmi RUN'ı; yalnız C2-07 ve `traces.jsonl` hariç) |
| `research/eval/run_p1b1_checks.py` | `P1-B.1/B1.4-pre:` | B1-1..B1-6 RUN kaydı |
| `research/eval/report_p1b1.py` | `P1-B.1/B1.4-pre:` | P1-B.1 rapor üreteci (resmi RUN'dan önce) |
| `spec/fixtures/**` | `P1-C/C0:` | yeniden dondurma P1-C/F1: C2-09 negatif iz girdisi (K-C0.2); `MANIFEST.json` yalnız C2-09 satırları |
| `core/tests/conformance.rs` | `P1-C/C0:` | yeniden dondurma P1-C/F2: Seviye 2 = C2-01..C2-09 |
| `core/tests/conformance/eval_c2.rs` | `P1-C/C0:` | yeniden dondurma P1-C/F3: `trace_tamper` işlemi |
| `core/tests/conformance_c2_gen.rs` | `P1-C/C0:` | yeniden dondurma P1-C/F4: 9 C2 dizini |
| `research/eval/gen_embed_ref.py` | `P1-C/C1:` | C-10 referans üreteci (prototip `sentence-transformers`) |
| `research/data/embed_ref.json` | `P1-C/C1:` | C-10 referansı: 50 D3 structural chunk + prototip vektörleri |
| `research/eval/gen_tok_ref.py` | `P1-C/C2:` | tokenizer referans üreteci (prototip `bm25_tokens`) |
| `research/data/tok_ref.json` | `P1-C/C2:` | tokenizer referansı: 200 D3 structural chunk + 8 kenar dizgisi |
| `spec/fixtures/**` | `P1-C/C7:` | yeniden dondurma P1-C/F5: C3-01..C3-04 (Seviye 3); `MANIFEST.json` yalnız C3 satırları |
| `index/tests/conformance_c3.rs` | `P1-C/C7:` | Seviye 3 denetleyicisi (raporu uzatır, `level_claimed 3`) |
| `index/tests/conformance/eval_c3.rs` | `P1-C/C7:` | C3 değerlendiricisi (üreteçle ortak) |
| `index/tests/conformance_c3_gen.rs` | `P1-C/C7:` | C3 fixture üreteci (mevcut indeksten) |
| `scripts/gen_fixtures_c3.sh` | `P1-C/C7:` | C3 fixture üreteci betiği |
| `spec/fixtures/**` | `P1-C/C7-fix:` | yeniden dondurma P1-C/F6: C3-03 sonraki içerikle güçlendirildi (C7 gözden geçirme 1) |
| `index/tests/conformance_c3.rs` | `P1-C/C7-fix:` | yeniden dondurma P1-C/F7: rapor `fixtures_manifest_hash` denetimi |
| `research/eval/run_c3_quality.py` | `P1-C/C8:` | C-3 koşturucusu (eşik kodda; sonuç görüldükten sonra değiştirilmedi) |
| `scripts/verify.sh` | `P1-C/C9-pre:` | yeniden dondurma P1-C/F8: P1-C zinciri (models, Seviye 3, p1b1-checks, p1c-checks, tamper P1-C, `compare_p1c pair`) |
| `research/eval/run_p1c_checks.py` | `P1-C/C9-pre:` | C-1..C-13 RUN kaydı (C-4/5/6/9 D3'te) |
| `research/eval/compare_p1c.py` | `P1-C/C9-pre:` | C-6 (pair) ve C-11 (regress, karar gereği hariçler) karşılaştırıcısı |
| `research/eval/report_p1c.py` | `P1-C/C9-pre:` | P1-C rapor üreteci (resmi RUN'dan önce) |
| `scripts/test_tamper_p1c.sh` | `P1-C/C9-pre:` | P1-C kasıtlı bozma vakaları |
| `index/tests/governance.rs` | `P1-C/C9-pre:` | C-7 property testi (`MDK_C7_OUT`) |
| `index/tests/embed.rs` | `P1-C/C9-pre:` | C-10 testi (`MDK_C10_OUT`) |
| `index/tests/hot.rs` | `P1-C/C9-pre:` | C-8 ve C-9 testleri (`MDK_C8_OUT`) |
| `core/tests/compat_v3final.rs` | `P1-C/C9-pre:` | C-13 testleri (`kc01_*`) |
| `research/eval/report_p1c.py` | `P1-C/C9-fix:` | yeniden dondurma P1-C/F9: C-3 eşik hücresinde `\|` kaçışı (Markdown tablo) |
| `core/tests/conformance.rs` | `P1-C.1/C1.0:` | yeniden dondurma P1-C.1/F1: `spec_version` 1.0.2 (K-C1.3, K-C1.5) |
| `research/eval/bm25_lucene.py` | `P1-C.1/C1.1:` | C1-1: Lucene/tantivy BM25 (K-C1.1) Python'da, yalnız formülden |
| `research/eval/run_c1_variant.py` | `P1-C.1/C1.1:` | C1-1 koşturucusu (eşik kodda; ilk geliştirme koşturmasından sonra değiştirilmedi) |
| `spec/fixtures/**` | `P1-C.1/C1.3:` | yeniden dondurma P1-C.1/F2: C3-05 eklendi; C3-01..04 beklenen değerleri K-C1.3 gereği (yalnız `include_proposal: true` gözlemleri); `MANIFEST.json` yalnız C3 satırları |
| `index/tests/conformance/eval_c3.rs` | `P1-C.1/C1.3:` | yeniden dondurma P1-C.1/F3: `canonical_prefix_of` gözlemi |
| `index/tests/conformance_c3.rs` | `P1-C.1/C1.3:` | yeniden dondurma P1-C.1/F4: Seviye 3 = C3-01..C3-05, toplam 31 |
| `index/tests/conformance_c3_gen.rs` | `P1-C.1/C1.3:` | yeniden dondurma P1-C.1/F5: 5 C3 dizini |
| `scripts/verify.sh` | `P1-C.1/C1.4-pre:` | yeniden dondurma P1-C.1/F6: P1-C.1 zinciri (`p1c1-checks`, `compare_p1c1 pair`) |
| `scripts/test_tamper_p1c.sh` | `P1-C.1/C1.4-pre:` | yeniden dondurma P1-C.1/F7: Seviye 3 toplamı 31 |
| `scripts/test_p1c1.sh` | `P1-C.1/C1.4-pre:` | C1-4..C1-6 CLI kasıtlı testleri (kilit dosyası, boş sorgu, manifest alanları) |
| `index/tests/p1c1.rs` | `P1-C.1/C1.4-pre:` | C1-2..C1-6 testleri (`MDK_P1C1_OUT`) |
| `research/eval/compare_p1c1.py` | `P1-C.1/C1.4-pre:` | C1-8 karşılaştırıcısı (pair hariçsiz; regress K-C1.6 hariçleri) |
| `research/eval/run_p1c1_checks.py` | `P1-C.1/C1.4-pre:` | C1-1..C1-9 RUN kaydı |
| `research/eval/gen_quality_ref.py` | `P1-C.1/C1.4-pre:` | C1-9 referans üreteci (yalnız geçen C1-1 kanıtıyla) |
| `research/data/quality_ref_rust.json` | `P1-C.1/C1.4-pre:` | Rust kalite regresyon referansı (K-C1.2; P1-C resmi RUN `c3_quality.json`'ından) |
| `research/eval/report_p1c1.py` | `P1-C.1/C1.4-pre:` | P1-C.1 rapor üreteci (resmi RUN'dan önce) |
| `research/eval/compare_p1c1.py` | `P1-C.1/C1.4-fix:` | yeniden dondurma P1-C.1/F8: ortak yaprakların anahtar dizgisiyle sıralanması (geliştirme zincirinde `TypeError`) |
| `scripts/check_v3_compat.sh` | `P1-D/D0:` | yeniden dondurma P1-D/F1: güncel `mdk` `-p mdkernel-server` ile derlenir (ikili `server`'a taşındı; v3-final tarafı değişmedi) |
| `scripts/test_p1c1.sh` | `P1-D/D0:` | yeniden dondurma P1-D/F2: `mdk` `-p mdkernel-server` ile derlenir |
| `scripts/test_tamper_p1c.sh` | `P1-D/D0:` | yeniden dondurma P1-D/F3: `mdk` `-p mdkernel-server` ile derlenir |
| `scripts/verify.sh` | `P1-D/D7-pre:` | yeniden dondurma P1-D/F4: P1-D zinciri (`p1d-checks`), `pair` → `compare_p1d pair`, `compare-p1d` alt komutu |
| `research/eval/compare_p1d.py` | `P1-D/D7-pre:` | D-8 (pair: P1-C.1 kümesi + P1-D dosyaları + baytlar) ve D-9 (regress: P1-C.1 resmi RUN, hariçsiz) karşılaştırıcısı |
| `research/eval/run_p1d_checks.py` | `P1-D/D7-pre:` | D-1..D-11 RUN kaydı (`p1d_checks.json`) |
| `research/eval/report_p1d.py` | `P1-D/D7-pre:` | P1-D rapor üreteci ve karar kuralı (resmi RUN'dan önce) |
| `scripts/mcp_e2e.sh` | `P1-D/D7-pre:` | D-2..D-6, D-8 (zincir içi), D-11 MCP uçtan uca betiği |
| `scripts/mcp_inspector.sh` | `P1-D/D7-pre:` | D-1 resmi MCP Inspector betiği (`@modelcontextprotocol/inspector@2.9.0`) |
| `scripts/http_e2e.sh` | `P1-D/D7-pre:` | D-7 ve servis (izleyici, SIGTERM, zamanlayıcı, token) betiği |
| `scripts/scene_agent.sh` | `P1-D/D7-pre:` | TASKS D6 ajan sahnesi |
| `scripts/test_tamper_p1d.sh` | `P1-D/D7-pre:` | P1-D kasıtlı bozmaları (şema, token, X-Consumer, D-11) |
| `server/tests/serve_watch.rs` | `P1-D/D7-pre:` | D-10 gecikme (`MDK_D10_OUT`) ve SIGTERM testi (`MDK_SIGTERM_OUT`) |
| `server/tests/mcp_schemas.rs` | `P1-D/D7-pre:` | D-1 şema denetleyicisi (`MDK_TOOLS_LIST`) |
| `server/examples/mcp_client.rs` | `P1-D/D7-pre:` | e2e/sahne stdio JSON-RPC sürücüsü |
| `index/tests/p1c1.rs` | `P1-D.1/D1.3:` | yeniden dondurma P1-D.1/F1: C1-6 `lexical.dl`, biçim `mdkernel-index/3` (K-D1.7) |
| `spec/fixtures/**` | `P1-D.1/D1.3:` | yeniden dondurma P1-D.1/F2: C3-06, C3-07 eklendi; `MANIFEST.json` yalnız bu satırları kazandı |
| `index/tests/conformance/eval_c3.rs` | `P1-D.1/D1.3:` | yeniden dondurma P1-D.1/F3: `agent_write` işlemi, `lengths` gözlemi |
| `index/tests/conformance_c3.rs` | `P1-D.1/D1.3:` | yeniden dondurma P1-D.1/F4: Seviye 3 = C3-01..C3-07 |
| `index/tests/conformance_c3_gen.rs` | `P1-D.1/D1.3:` | yeniden dondurma P1-D.1/F5: 7 C3 dizini |
| `spec/fixtures/**` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F6: C2-10 eklendi; `MANIFEST.json` yalnız C2-10 satırları |
| `core/tests/conformance/eval_c2.rs` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F7: `ingest … actor`, `actor_tamper` |
| `core/tests/conformance_c2_gen.rs` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F8: 10 C2 dizini |
| `core/tests/conformance.rs` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F9: Seviye 2 = C2-01..C2-10, `spec_version` 1.0.3 |
| `index/tests/conformance_c3.rs` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F10: alt düzey toplamı 27 (toplam 34) |
| `scripts/test_tamper_p1c.sh` | `P1-D.1/D1.4:` | yeniden dondurma P1-D.1/F11: Seviye 3 toplamı 34 |
| `scripts/verify.sh` | `P1-D.1/D1.7-pre:` | yeniden dondurma P1-D.1/F12: P1-D.1 zinciri (`p1d1-checks`), `pair` → `compare_p1d1 pair`, `compare-p1d1` alt komutu |
| `research/eval/compare_p1d1.py` | `P1-D.1/D1.7-pre:` | D1-12 karşılaştırıcısı (pair: P1-D kümesi + P1-D.1 dosyaları; regress: P1-D resmi RUN, karar gereği hariçler ölçümden önce) |
| `research/eval/run_p1d1_checks.py` | `P1-D.1/D1.7-pre:` | D1-1..D1-13 RUN kaydı (`p1d1_checks.json`) |
| `research/eval/report_p1d1.py` | `P1-D.1/D1.7-pre:` | P1-D.1 rapor üreteci ve karar kuralı (resmi RUN'dan önce) |
| `scripts/test_p1d1.sh` | `P1-D.1/D1.7-pre:` | D1-1..D1-9, D1-11, D1-13 betiği |
| `scripts/test_tamper_p1d1.sh` | `P1-D.1/D1.7-pre:` | P1-D.1 kasıtlı bozmaları (sözleşme, olay aktörü, fixture, politika) |
| `scripts/contract_check.sh` | `P1-D.1/D1.7-pre:` | D1-13 sözleşme denetleyicisi |
| `server/tests/p1d1_criteria.rs` | `P1-D.1/D1.7-pre:` | D1-1..D1-9 ölçümleri (`MDK_P1D1_OUT`) |
| `server/tests/contract.rs` | `P1-D.1/D1.7-pre:` | sözleşme vakaları (D1-13 eşleşmesi) |
| `server/examples/mcp_pipeline.rs` | `P1-D.1/D1.7-pre:` | boru hatlı stdio JSON-RPC sürücüsü |
| `scripts/governance_scenario.sh` | `P1-D.1/D1.7-pre:` | yeniden dondurma P1-D.1/F13: ajan düzenlemesi `--mode append`, senaryo vault'u `[edit] unrestricted` (K-D1.1, D1.0-5) |
| `scripts/test_p1c1.sh` | `P1-D.1/D1.7-pre:` | yeniden dondurma P1-D.1/F14: C1-6 anahtar listesinde `lexical.dl` (K-D1.7) |
| `research/eval/compare_p1d.py` | `P1-D.1/D1.7-pre:` | yeniden dondurma P1-D.1/F15: alan kümesi farkı kayıtlı fark (çıkış 3), hata değil |
<!-- /frozen-table -->

## Notlar
- **Yeniden dondurma (P1-B, Ö1):** tur görevi dondurulmuş bir dosyayı değiştirmek zorundaysa tabloya aynı kalıpla **yeni bir satır eklenir** (eski satır silinmez, değişmez); yeni satırın commit'i eskisinin torunu olmalı ve notu `docs/rounds/<tur>/progress/FREEZE_EXCEPTIONS.md` içindeki bir girdiyi (`<tur>/F<n>`) anmalıdır. Denetleyici o kalıbı yeni commit'ten itibaren denetler. Tablo her `docs/rounds/*/progress/ROUND_START` sha'sından beri yalnız satır kazanabilir (`git log -p` + uç karşılaştırması).
- `research/eval/**` altında A5'te eklenen dosyalar (`datasets/synthetic_edits_v3.py`, `run_holdout.py`, `run_split_grid.py`) ve yeni manifest'ler A5 commit'inden itibaren kendi satırlarıyla dondurulmuştur.
- Bir satır yalnız dondurulduğu commit'te **var olan** dosyaları kapsar; sonradan eklenen dosyalar kendi satırlarıyla dondurulur. Satırın commit'i henüz yoksa (görev commit'lenmemiş) satır "henüz dondurulmadı" diye raporlanır.
- `research/docs/progress/**` ve `research/reports/{commentary.md,v3/commentary.md}` prototipin tarihsel kayıtlarıdır; yalnız dondurulmuş rapor üreteçleri (`eval/report*.py`) ve onların testleri için `v3-final`'den aynen geri yüklendi.
