# Belirlenimli alanlar (K-E0.2, P1-E/E0)

Her çift koşturma karşılaştırıcısı (`research/eval/compare_*.py`) yalnız **sözleşmeye göre belirlenimli** alanları karşılaştırır. Bu belge her karşılaştırıcı için karşılaştırılan sonuç dosyalarını (dosya tablosu) ve karşılaştırılmayan **her** alanı, sınıfı ve gerekçesiyle (alan tablosu) tutar. Bir karşılaştırma dosyasının burada adı geçmeyen her yaprağı karşılaştırılır, yani belirlenimli ilan edilmiştir.

`scripts/check_compare_fields.sh` (`research/eval/check_compare_fields.py`, `make check-compare-fields`, `make lint`'in parçası) bu tabloları karşılaştırıcıların koduyla iki yönde eşler: her `compare_*.py` için bir bölüm; dosya tablosu = kodun dosya sabitleri; alan tablosu = kodun hariç tutma sabitleri. Eşleşmeyen satır sayısı 0 değilse kapı kırmızıdır. `--runs A B C` ile ayrıca gerçek koşturmalarda her karşılaştırılan yaprağın eşit, her bildirilen belirlenimsiz alanın var olduğu teyit edilir (P1-E VERIFY V0).

## Sınıflar ve kipler
- **duvar-saati:** süre, gecikme, oran ya da bunlardan türeyen bir değer. Makineye ve yüke bağlıdır.
- **bilgi:** sözleşmenin belirlenimsiz bıraktığı bir gözlem (ör. `docs/CLIENT_CONTRACT.md` §5: salt okunur çağrılar paraleldir). Kaydedilir, karşılaştırılmaz. Aynı ölçümün belirlenimli kısmı ayrı bir alanla karşılaştırılır.
- **ortam:** zincirin çalıştığı dizin gibi yerel bir değer.
- **karar:** iki **farklı** sürümün koşturması karşılaştırılırken (`regress`) bir turun kararından doğan, ölçümden önce yazılmış fark. Belirlenimsizlik değildir; aynı commit'in iki koşturması (`pair`) bunu asla hariç tutmaz.

Kip: `pair` (aynı commit, iki temiz klon), `regress` (önceki turun resmi RUN'ı ↔ bu RUN), `hepsi` (karşılaştırıcının iki kipinde de). Denetim kuralı: `pair`/`hepsi` satırı `karar` olamaz, `regress` satırı yalnız `karar` olabilir.

## Karşılaştırıcılar
Kalıtım: `compare_runs.compare`'i ya da onun `leaves`/`_excluded`'ını kullanan her karşılaştırıcı `compare_runs` satırlarını da uygular; `compare_p1d1`, `compare_p1d`'nin `P1D_LATENCY` satırlarını uygular. Bir satır yalnız tanımlandığı karşılaştırıcının bölümünde yazılır.

### `compare_runs`
v2/v3 sonuç dosyaları (`RESULT_FILES`, `RESULT_FILES_V3`); hariç tutulanlar yalnız duvar saati gecikmeleri (`LATENCY_PATHS`).

### `compare_p1a`
P1-A kümesi (`RESULT_FILES_P1A` = v3 + holdout + split_grid); kendi hariç tutması yok (`compare_runs` satırları).

### `compare_p1b`
P1-B kümesi (`RESULT_FILES_P1B`) alan alan + yönetişim vault'unun `events.jsonl`, `traces.jsonl`, `checkpoints/` baytları; kendi hariç tutması yok.

### `compare_p1b1`
Yalnız `regress` (P1-B resmi RUN'ı ↔ P1-B.1): `allowed_fields`, K-B1.2'nin sonuçları. Ayrıca yönetişim vault'unun `traces.jsonl`'i bayt karşılaştırmasında kaydedilir ama hariç tutulur (`excluded: true`, K-B1.2 `trace_id` formülü; kodda satır içi, `main`). `<C2-07>`, `p1b_checks.json`'daki C2-07 testinin sırası (kod kimlikle bulur).

### `compare_p1c`
`pair`: P1-C kümesi (`PAIR_FILES`) + yönetişim baytları, hariç tutma yok. `regress`: `REGRESS_FILES`, `REGRESS_ALLOWED`.

### `compare_p1c1`
`pair`: `PAIR_FILES`, hariç tutma yok. `regress`: K-C1.6 `ALLOWED`.

### `compare_p1d`
`pair`: P1-C.1 kümesi + `P1D_FILES` alan alan + `P1D_BYTES` bayt bayt; `regress` aynı alanlar, uyumluluk raporu `CONFORMANCE` test kimlikleriyle. Hariç: D-10 ve SIGTERM duvar saati alanları (`P1D_LATENCY`).

### `compare_p1d1`
`pair`: `compare_p1d.pair` + `P1D1_FILES` alan alan + `P1D1_BYTES`; hariç: duvar saati ve ortam (`P1D1_TIMING`), sözleşmenin belirlenimsiz bıraktığı iki D1-5 alanı (`P1D1_INFO`, P1-E/E0 F2; yazan ve iz yazan çağrıların yanıtları `write_responses_a_vs_b_equal` ve `write_responses_pipelined_vs_lockstep_equal` ile karşılaştırılır, F1). `regress`: P1-D resmi RUN'ı, `EXCLUDED` (D1-12'nin 29 karar satırı).

### `compare_p1e`
`pair`: `compare_p1d1.pair` + `P1E_FILES` alan alan + `P1E_BYTES` (sahnenin ve eklenti e2e'sinin vault günlükleri, eklentinin HTTP erişim günlüğü, iki sonuç kaydı) bayt bayt; P1-E dosyalarında duvar saati ya da sözleşmece belirlenimsiz alan yok (`P1E_TIMING`, `P1E_INFO` boş). `regress`: P1-D.1 resmi RUN'ı, `EXCLUDED` (P1-E kararlarının satırları, resmi RUN'dan önce yazıldı) ve uyumluluk raporu test kimliğiyle.

## Alan tablosu
<!-- fields-table -->
| Araç | Kip | Dosya | Yol kalıbı | Sınıf | Gerekçe |
|---|---|---|---|---|---|
| `compare_p1b1` | regress | `governance/governance.json` | `traces_sha256` | karar | K-B1.2: `trace_id` formülü değişti |
| `compare_p1b1` | regress | `p1b_checks.json` | `B-1/tests/<C2-07>/actual_hash` | karar | K-B1.2: C2-07 beklenen değeri yeniden üretildi |
| `compare_p1b1` | regress | `p1b_checks.json` | `B-1/tests/<C2-07>/expected_hash` | karar | K-B1.2: aynı |
| `compare_p1b1` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-B1.2: `MANIFEST.json` C2-07 satırı |
| `compare_p1b1` | regress | `p1b_checks.json` | `governance/traces_sha256` | karar | K-B1.2: `trace_id` formülü |
| `compare_p1c` | regress | `p1b1_checks.json` | `tests_seen` | karar | P1-C: uyumluluk raporu C2-09 kazandı (K-C0.2) |
| `compare_p1c` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-C0.2: C2-09 fixture'ı |
| `compare_p1c1` | regress | `p1b1_checks.json` | `criteria/B1-3/pass` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1b1_checks.json` | `criteria/B1-3/spec_version` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_checks.json` | `C-1/spec_version` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_checks.json` | `C-11/differing_outside_exclusions` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_checks.json` | `C-11/pass` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_checks.json` | `C-7/proposal_hits_with_include_proposal` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_index.json` | `index_files_build/manifest.json` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_index.json` | `index_files_compact/manifest.json` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1c1` | regress | `p1c_index.json` | `index_files_final/manifest.json` | karar | K-C1.6 (C1-8 hariç tutma listesi; ölçümden önce yazıldı) |
| `compare_p1d` | hepsi | `p1d/d10.json` | `latencies_ms` | duvar-saati | D-10 izleyici gecikmeleri |
| `compare_p1d` | hepsi | `p1d/d10.json` | `max_ms` | duvar-saati | D-10 en büyük gecikme |
| `compare_p1d` | hepsi | `p1d/d10.json` | `p95_ms` | duvar-saati | D-10 p95 |
| `compare_p1d` | hepsi | `p1d/sigterm.json` | `timer_compactions` | duvar-saati | SIGTERM sırasında zamanlayıcının kaç sıkıştırma yaptığı saate bağlı |
| `compare_p1d` | hepsi | `p1d/sigterm.json` | `trials/*/timer_compactions` | duvar-saati | aynı, deneme başına |
| `compare_p1d` | hepsi | `p1d_checks.json` | `D-10/measured` | duvar-saati | D-10'un ölçülen değeri (ms) |
| `compare_p1d1` | hepsi | `conformance_p1d1.json` | `fixtures_root` | ortam | zincirin klonunun mutlak fixture yolu |
| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `elapsed_ms_lockstep` | duvar-saati | süre (ms) |
| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `elapsed_ms_pipelined` | duvar-saati | süre (ms) |
| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `responses_a_vs_b_equal` | bilgi | sözleşme §5: boru hattındaki salt okunur `blame` paraleldir, önceki yazmayı görmeyebilir; bütün yanıtların eşitliği şans gözlemidir (P1-D.1 doğrulaması Ş0). Yazan ve iz yazan çağrıların yanıtları `write_responses_*_equal` ile karşılaştırılır |
| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `responses_pipelined_vs_lockstep_equal` | bilgi | aynı (§5); kilit adımlı koşu her yanıtı beklediği için `blame` önceki yazmayı her zaman görür, boru hatlısı görmeyebilir |
| `compare_p1d1` | hepsi | `p1d1/d1_6.json` | `elapsed_ms_pipelined` | duvar-saati | süre (ms) |
| `compare_p1d1` | hepsi | `p1d1/d1_6.json` | `elapsed_ms_serial` | duvar-saati | süre (ms) |
| `compare_p1d1` | hepsi | `p1d1/d1_6.json` | `ratio` | duvar-saati | iki sürenin oranı |
| `compare_p1d1` | hepsi | `p1d1/d1_6.json` | `ratio_below_0_6` | duvar-saati | oranın eşikle karşılaştırması (bilgi kriteri) |
| `compare_p1d1` | hepsi | `p1d1_checks.json` | `D1-6/measured` | duvar-saati | D1-6'nın ölçülen oranı |
| `compare_p1d1` | regress | `p1b1_checks.json` | `criteria/B1-3/spec_version` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): the frozen B1-3 record names the spec version |
| `compare_p1d1` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | F2, F6 (C2-10, C3-06, C3-07): MANIFEST.json gains the new fixtures' rows |
| `compare_p1d1` | regress | `p1c1_checks.json` | `C1-6/lexical/dl` | karar | K-D1.7: the lexical record gains dl |
| `compare_p1d1` | regress | `p1c1_checks.json` | `C1-6/rejected_cases` | karar | K-D1.7 (F1): two rejection cases for dl |
| `compare_p1d1` | regress | `p1c1_checks.json` | `C1-7` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): C1-7 checks the conformance report's spec version (1.0.2) |
| `compare_p1d1` | regress | `p1c_checks.json` | `C-1/spec_version` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): the frozen C-1 record names the spec version |
| `compare_p1d1` | regress | `p1c_index.json` | `index_files_build/manifest.json` | karar | K-D1.7: manifest lexical.dl, format mdkernel-index/3 |
| `compare_p1d1` | regress | `p1c_index.json` | `index_files_compact/manifest.json` | karar | K-D1.7: manifest lexical.dl, format mdkernel-index/3 |
| `compare_p1d1` | regress | `p1c_index.json` | `index_files_final/manifest.json` | karar | K-D1.7: manifest lexical.dl, format mdkernel-index/3 |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `cases/11` | karar | K-D1.5: W-3 expects blame of an unknown document as 422; now 404 not_found |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `cases/5` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): D7-3 checks status spec_version 1.0.2 |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `cases/8` | karar | K-D1.5: H-1 expects every tool error as 422; codes now set the status |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `d7_passed` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): count of D7-3 above |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `pass` | karar | K-D1.5, spec 1.0.3: conjunction of the cases above |
| `compare_p1d1` | regress | `p1d/http/http_e2e.json` | `passed` | karar | K-D1.5, spec 1.0.3: count of the cases above |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/3` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): status: versions (spec 1.0.2) |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/41` | karar | K-D1.1: heading-less append to the human espresso.md is denied (append_only) |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/49` | karar | K-D1.1: replace_section of the human Recipe section is denied (append_only) |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/50` | karar | K-D1.1: follows from the denied replace_section |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/52` | karar | K-D1.1: follows from the denied append and replace_section |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/53` | karar | K-D1.6: checkpoint {name} after a search names the state: one CheckpointNamed event |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `pass` | karar | K-D1.1, K-D1.6, spec 1.0.3: conjunction of the checks above |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `per_tool/propose` | karar | K-D1.1: count of the propose checks above |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `per_tool/status` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): count of the status check above |
| `compare_p1d1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `tools_passed` | karar | K-D1.1, spec 1.0.3: count of the tools above |
| `compare_p1d1` | regress | `p1d/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): the scene shows status spec_version |
| `compare_p1d1` | regress | `p1d_checks.json` | `D-2` | karar | K-D1.1, spec 1.0.3: P1-D's D-2 record of the e2e counts above |
| `compare_p1d1` | regress | `p1d_checks.json` | `D-7` | karar | K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION): P1-D's D-7 record of D7-3 and the HTTP counts above |
| `compare_p1d1` | regress | `p1d_checks.json` | `D-9` | karar | K-D1.7, spec 1.0.3, F2/F6: P1-D's D-9 (against P1-C.1) records the spec version, manifest and lexical.dl differences above |
| `compare_runs` | hepsi | `h2.json` | `H2-3` | duvar-saati | H2-3 bir gecikme kriteridir (ms) |
| `compare_runs` | hepsi | `h2.json` | `latency_ms` | duvar-saati | sorgu gecikmesi (ms) |
| `compare_runs` | hepsi | `h3.json` | `H3-1` | duvar-saati | H3-1 yazma→aranabilir gecikme kriteri |
| `compare_runs` | hepsi | `h3.json` | `latency/latencies_ms` | duvar-saati | ölçülen gecikmeler |
| `compare_runs` | hepsi | `h3.json` | `latency/rows/*/ms` | duvar-saati | satır başına gecikme |
| `compare_runs` | hepsi | `h3.json` | `summary/p50` | duvar-saati | gecikme yüzdeliği |
| `compare_runs` | hepsi | `h3.json` | `summary/p95` | duvar-saati | gecikme yüzdeliği |
| `compare_runs` | hepsi | `h3_v3.json` | `H3-1` | duvar-saati | H3-1 yazma→aranabilir gecikme kriteri (v3 H3) |
| `compare_runs` | hepsi | `h3_v3.json` | `latency/latencies_ms` | duvar-saati | ölçülen gecikmeler (v3 H3) |
| `compare_runs` | hepsi | `h3_v3.json` | `latency/rows/*/ms` | duvar-saati | satır başına gecikme (v3 H3) |
| `compare_runs` | hepsi | `h3_v3.json` | `summary/p50` | duvar-saati | gecikme yüzdeliği (v3 H3) |
| `compare_runs` | hepsi | `h3_v3.json` | `summary/p95` | duvar-saati | gecikme yüzdeliği (v3 H3) |
| `compare_p1e` | regress | `p1d1/d1_13.json` | `codes` | karar | K-E0.3: contract §3 and error.rs gain index_stale: 11 codes, the sorted lists shift |
| `compare_p1e` | regress | `p1d1/d1_13.json` | `http` | karar | F3 (ADDENDUM Ş4): contract_check.sh's new HTTP step: endpoints, codes, transport rules |
| `compare_p1e` | regress | `p1d1/d1_3.json` | `outcomes/append:denied` | karar | K-E0.4: a refused heading-less append is policy_violation, no longer denied |
| `compare_p1e` | regress | `p1d1/d1_3.json` | `outcomes/append:policy_violation` | karar | K-E0.4: the same outcomes, counted under the new code |
| `compare_p1e` | regress | `p1d1/d1_5.json` | `propose_denied` | karar | K-E0.4: the 8 heading-less appends of D1-5 answer policy_violation (8 → 0 denied) |
| `compare_p1e` | regress | `p1d1/d1_5.json` | `write_responses` | karar | K-E0.2 (F1): new field: count of compared responses |
| `compare_p1e` | regress | `p1d1/d1_5.json` | `write_responses_a_vs_b_equal` | karar | K-E0.2 (F1): new field: writing and trace-writing responses, pipelined A vs B |
| `compare_p1e` | regress | `p1d1/d1_5.json` | `write_responses_pipelined_vs_lockstep_equal` | karar | K-E0.2 (F1): new field: writing and trace-writing responses, pipelined vs lock-step |
| `compare_p1e` | regress | `p1d1_checks.json` | `D1-13/codes` | karar | K-E0.3: the D1-13 record counts 11 codes |
| `compare_p1e` | regress | `p1d1_checks.json` | `D1-3/outcomes/append:denied` | karar | K-E0.4: the D1-3 record of the outcomes above |
| `compare_p1e` | regress | `p1d1_checks.json` | `D1-3/outcomes/append:policy_violation` | karar | K-E0.4: the D1-3 record of the outcomes above |
<!-- /fields-table -->

## Dosya tablosu
<!-- files-table -->
| Sabit | Dosyalar |
|---|---|
| `compare_runs.RESULT_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h2_tuning.json`, `h2.json`, `h3.json`, `h4.json` |
| `compare_runs.RESULT_FILES_V3` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json` |
| `compare_p1a.RESULT_FILES_P1A` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json` |
| `compare_p1b.RESULT_FILES_P1B` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json` |
| `compare_p1b.BYTE_FILES` | `governance/vault/.mdkernel/events.jsonl`, `governance/vault/.mdkernel/traces.jsonl` |
| `compare_p1b.BYTE_DIRS` | `governance/vault/.mdkernel/checkpoints` |
| `compare_p1c.PAIR_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json`, `c3_quality.json`, `p1c_checks.json`, `p1c_index.json` |
| `compare_p1c.REGRESS_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json` |
| `compare_p1c1.PAIR_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json`, `c3_quality.json`, `p1c_checks.json`, `p1c_index.json`, `c1_variant.json`, `p1c1_checks.json` |
| `compare_p1d.P1D_FILES` | `p1d_checks.json`, `p1d/inspector/inspector.json`, `p1d/mcp_e2e/mcp_e2e.json`, `p1d/http/http_e2e.json`, `p1d/scene/scene.json`, `p1d/tamper_p1d.json`, `p1d/d10.json`, `p1d/sigterm.json` |
| `compare_p1d.P1D_BYTES` | `p1d/mcp_e2e/run1/transcript.jsonl`, `p1d/mcp_e2e/run1/transcript_b.jsonl`, `p1d/mcp_e2e/run1/cli_promote.jsonl`, `p1d/mcp_e2e/run1/vault/.mdkernel/events.jsonl`, `p1d/mcp_e2e/run1/vault/.mdkernel/traces.jsonl`, `p1d/mcp_e2e/run1/vault/.mdkernel/proposals.jsonl`, `p1d/mcp_e2e/run1/vault/espresso.md`, `p1d/mcp_e2e/run1/vault/agent/plan.md`, `p1d/scene/scene.json`, `p1d/inspector/tools_list.json` |
| `compare_p1d.CONFORMANCE` | `conformance_p1c1.json` |
| `compare_p1d1.P1D1_FILES` | `p1d1_checks.json`, `conformance_p1d1.json`, `p1d1/p1d1.json`, `p1d1/d1_1.json`, `p1d1/d1_2.json`, `p1d1/d1_3.json`, `p1d1/d1_4.json`, `p1d1/d1_5.json`, `p1d1/d1_6.json`, `p1d1/d1_7.json`, `p1d1/d1_8.json`, `p1d1/d1_9.json`, `p1d1/d1_11.json`, `p1d1/d1_13.json`, `p1d1/v3_compat.json`, `p1d1/v3_compat_mixed.json`, `p1d1/tamper_p1d1.json` |
| `compare_p1d1.P1D1_BYTES` | `p1d1/d1_3.json`, `p1d1/d1_8.json`, `p1d1/d1_13.json` |
| `compare_p1e.P1E_FILES` | `p1e_checks.json`, `p1e/e0/e14.json`, `p1e/e0/e15.json`, `p1e/scene/scene.json`, `p1e/plugin_e2e/plugin_e2e.json`, `p1e/plugin_e2e/access_log.json`, `p1e/client.json`, `p1e/release/release_local.json`, `p1e/release/conformance_mdk.json`, `p1e/release/conformance_equiv.json`, `p1e/tamper_p1e.json`, `p1e/check_compare_fields.json` |
| `compare_p1e.P1E_BYTES` | `p1e/scene/scene.json`, `p1e/scene/events.jsonl`, `p1e/scene/traces.jsonl`, `p1e/scene/proposals.jsonl`, `p1e/plugin_e2e/plugin_e2e.json`, `p1e/plugin_e2e/access_log.json`, `p1e/plugin_e2e/events.jsonl`, `p1e/plugin_e2e/traces.jsonl`, `p1e/plugin_e2e/proposals.jsonl` |
<!-- /files-table -->
