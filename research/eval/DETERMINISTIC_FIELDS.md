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

### `compare_p1e1`
`pair`: `compare_p1e.pair` + `P1E1_FILES` alan alan + `P1E1_BYTES` (gerçek Obsidian e2e kaydı ve erişim günlüğü) bayt bayt; hariç: yayım ikilisinin `sha256`'sı (`P1E1_INFO`, ortam: derleme dizinine bağlı olabilir; RELEASE.md insan adımı 3 makineler arası hash'i yalnız kaydeder). `regress`: P1-E resmi RUN'ı, `EXCLUDED` (P1-E.1 kararlarının satırları, resmi RUN'dan önce yazıldı) ve uyumluluk raporu test kimliğiyle.

### `compare_p2_0`
`pair`: `compare_p1e1.pair` + `P2_0_FILES` alan alan + `P2_0_BYTES` (D-A ve D-C vault'larının `events.jsonl` ve `attestations.jsonl`'i, H-P2-0g) bayt bayt; hariç: gerçek Obsidian D-B ölçümleri (`P2_0_TIMING`, duvar saati: `p2_0_checks.json`'da H-P2-0b, b′, c; kopyalanan ayar kaydının `windows` satırları). `p2_0/d_b.json` bütünüyle duvar saati kaydıdır, dosya listesinde yoktur. `regress`: P1-E.1 resmi RUN'ı, `EXCLUDED` (P2-0 kararlarının satırları, resmi RUN'dan önce yazıldı) ve uyumluluk raporu test kimliğiyle (referansın 34 testi eşit, C2-11, C2-12, C3-08 geçti, spec 2.0.0).

### `compare_p2_0_1`
`pair`: `compare_p1e1.pair` + `P2_0_1_FILES` alan alan + `P2_0_1_BYTES` (D-A, D-A2 ve D-C vault'larının `events.jsonl` ve `attestations.jsonl`'i, H-P2-0.1i) bayt bayt; hariç: gerçek Obsidian kümelerinin ve düzenleyici oturumlarının ölçümleri (`P2_0_1_TIMING`, duvar saati: `p2_0_1_checks.json`'da H-P2-0.1b, b′, c, d′, e, f). `p2_0_1/d_b.json`, `d_a_prime.json`, `d_c_prime.json` ve `d_b2.json` bütünüyle duvar saati kayıtlarıdır, dosya listesinde yoktur. `regress`: P2-0 resmi RUN'ı, `EXCLUDED` (P2-0.1 kararlarının satırları, resmi RUN'dan önce geliştirme RUN'ından yazıldı), uyumluluk raporu test kimliğiyle (referansın 37 testi eşit, C2-13 geçti, spec 2.0.1) ve P2-0'ın yeniden koşan kümeleri (D-A her alan ve `events.jsonl`, D-E'nin `v0.1.0` satırları; hariç tutmasız).

### `compare_p2_0_2`
`pair`: `compare_p1e1.pair` + `P2_0_2_FILES` alan alan + `P2_0_2_BYTES` (D-A, D-A2 ve D-C vault'larının `events.jsonl` ve `attestations.jsonl`'i, H-P2-0.2i) bayt bayt; hariç: gerçek Obsidian kümelerinin ve düzenleyici oturumlarının ölçümleri (`P2_0_2_TIMING`, duvar saati: `p2_0_2_checks.json`'da H-P2-0.2b, b′, c, d'nin `d_c_prime` alt ağacı ve `pass`'i, e, f). `p2_0_2/d_b.json`, `d_a_prime.json`, `d_c_prime.json` ve `d_b2.json` bütünüyle duvar saati kayıtlarıdır, dosya listesinde yoktur. `regress`: P2-0.1 resmi RUN'ı, `EXCLUDED` (P2-0.2 kararlarının satırları, resmi RUN'dan önce geliştirme RUN'ından yazıldı), uyumluluk raporu test kimliğiyle (referansın 38 testi eşit, C2-14 geçti, spec 2.0.2) ve P2-0.1'in yeniden koşan kümeleri (D-A ve D-A2 her alan ve `events.jsonl`, D-E'nin beş satırı; D-C: T3 sırayı sabitlediği için `seq` dışında aynı olay çoklu kümesi ve `collide` satırlarındaki `matched: false` dışında aynı tasdik satırları — X2; hariç tutmasız).

### `compare_p2a`
`pair`: `compare_p2_0_2.pair` (P1-E.1 çift kümesi, P2-0.2 dosyaları ve günlükleri; P2-A zincirinde P2-0.2'nin gerçek Obsidian kümeleri `skipped` kaydedilir, K-P2.3) + `P2A_FILES` alan alan; P2-A kümeleri belirlenimlidir (vault'ların geçici yolu `<vault>` yazılır), hariç tutma yok (`P2A_TIMING`, `P2A_INFO` boş). `regress`: P2-0.2 resmi RUN'ı; dosyalar P1-E.1 çift kümesi ve `P2_0_2_FILES`, `EXCLUDED` (P2-A kararlarının satırları, resmi RUN'dan önce geliştirme RUN'ından yazıldı), uyumluluk raporu test kimliğiyle (39 test eşit, spec 2.0.2) ve D-A, D-A2, D-C günlükleri bayt bayt (`compare_p2_0_2.P2_0_2_BYTES`).

### `compare_p2l`
`fast` (P2-L/L0 d, `make verify FAST=1`, K-P2.4 (3)): hızlı zincirin dosyaları (`P2L_FAST_FILES`: `fast/` altındaki uyumluluk raporları, `conformance_equiv` çıkışı ve `regress.json`, `c3_quality.json`, `p2l_checks.json`; L5'ten beri P2-L'nin ölçüm dosyaları `p2l_quality.json`, `p2l/{d_tr,english,migrate}.json` de) alan alan; hariç: `fast/conformance.json`'ın `fixtures_root`'u (`P2L_INFO`, ortam: zincirin klonunun mutlak yolu). `pair`: `compare_p2a.pair` + `P2L_FILES` (`p2l_checks.json`, `p2l_quality.json`, `p2l/{d_tr,english,migrate}.json`) alan alan, aynı hariçle; P2-L'nin ölçümleri belirlenimlidir (iz kimlikleri içerikten, vault yolları geçici dizinde ama kayda yazılmaz). `regress` kipi yok (`EXCLUDED` boş): hızlı zincirin son resmi RUN'a karşı regresyonu `eval.fast_regress`'tir.

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
| `compare_p1e1` | hepsi | `p1e1/release/release.json` | `targets/*/sha256` | ortam | yayım ikilisinin sha256'sı: aynı commit, başka klon dizini (P1-E.1/F6) |
| `compare_p1e1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `checks/4/pass` | karar | K-E1.2: P1-D's MCP e2e status check expects the old `index == "present"` string; status.index is an object |
| `compare_p1e1` | regress | `p1d/mcp_e2e/mcp_e2e.json` | `per_tool/status/passed` | karar | K-E1.2: the same status check, counted (3 → 2 of 4) |
| `compare_p1e1` | regress | `p1d_checks.json` | `D-2/per_tool/status/passed` | karar | K-E1.2: D-2's record of the status check above (D-2 already failed, historical) |
| `compare_p1e1` | regress | `p1d1/d1_13.json` | `http/status_index` | karar | F2 (E1-8): contract_check.sh's new status step: status in the four index states |
| `compare_p1e1` | regress | `p1d1/d1_13.json` | `http/status_index_fields` | karar | F2 (E1-8): the index fields the status step reads from the contract's §2 row |
| `compare_p1e1` | regress | `p1d1_checks.json` | `D1-12/pass` | karar | K-E1.2: D1-12 regresses against P1-D and sees the status check above |
| `compare_p1e1` | regress | `p1d1_checks.json` | `D1-12/regress_vs_p1d/equal` | karar | K-E1.2: the same, its comparison result |
| `compare_p1e1` | regress | `p1d1_checks.json` | `D1-12/regress_vs_p1d/differing_fields/p1d/mcp_e2e/mcp_e2e.json:checks›4›pass` | karar | K-E1.2: the one leaf D1-12 sees: the status check above |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/pass` | karar | K-E1.2: E-9 regresses against P1-D.1 and sees the status check above |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/equal` | karar | K-E1.2: the same, its comparison result |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d/mcp_e2e/mcp_e2e.json:checks›4›pass` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d/mcp_e2e/mcp_e2e.json:per_tool›status›passed` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d1_checks.json:D1-12›pass` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d1_checks.json:D1-12›regress_vs_p1d›differing_fields›p1d/mcp_e2e/mcp_e2e·json:checks›4›pass` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d1_checks.json:D1-12›regress_vs_p1d›equal` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/differing_fields/p1d_checks.json:D-2›per_tool›status›passed` | karar | K-E1.2: a leaf E-9 sees: the status check above or its records |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1/excluded_differences/p1d1/d1_13.json:http/leaves` | karar | F2 (E1-8): E-9's count of excluded d1_13 http leaves grows by the status step |
| `compare_p1e1` | regress | `p1e/check_compare_fields.json` | `fields_declared` | karar | F5, F6: the field audit sees the new comparator compare_p1e1 (its module, rows and file constants) |
| `compare_p1e1` | regress | `p1e/check_compare_fields.json` | `fields_in_code` | karar | F5, F6: the field audit sees the new comparator compare_p1e1 (its module, rows and file constants) |
| `compare_p1e1` | regress | `p1e/check_compare_fields.json` | `file_constants` | karar | F5, F6: the field audit sees the new comparator compare_p1e1 (its module, rows and file constants) |
| `compare_p1e1` | regress | `p1e/check_compare_fields.json` | `modules` | karar | F5, F6: the field audit sees the new comparator compare_p1e1 (its module, rows and file constants) |
| `compare_p1e1` | regress | `p1e/client.json` | `bundle_bytes` | karar | K-E1.2 (E1.2): main.js carries the index notice of the connection status |
| `compare_p1e1` | regress | `p1e/client.json` | `vitest/files` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p1e1` | regress | `p1e/client.json` | `vitest/tests` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p1e1` | regress | `p1e/client.json` | `vitest/passed` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-10/bundle_bytes` | karar | K-E1.2 (E1.2): main.js carries the index notice of the connection status |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-10/vitest/files` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-10/vitest/tests` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p1e1` | regress | `p1e_checks.json` | `E-10/vitest/passed` | karar | K-E1.2 (E1.2): the client's index-notice unit tests (97 → 100) |
| `compare_p2_0` | hepsi | `p2_0_checks.json` | `H-P2-0b` | duvar-saati | D-B test: gerçek Obsidian kayıtlarının yanlış-`unverified` oranı (kaydın ve izleyicinin zamanlamasına bağlı) |
| `compare_p2_0` | hepsi | `p2_0_checks.json` | `H-P2-0b′` | duvar-saati | D-B test: geç tasdikle düzelen kayıtlar (zamanlamaya bağlı) |
| `compare_p2_0` | hepsi | `p2_0_checks.json` | `H-P2-0c` | duvar-saati | D-B test: kayıt → `canonical` gecikmesi (ms) |
| `compare_p2_0` | hepsi | `p2_0/tuning.json` | `windows` | duvar-saati | D-B ayar koşusunun beş `W` satırı (oran, gecikme) |
| `compare_p2_0` | regress | `conformance_p1d1.json` | `fixtures_manifest_hash` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `conformance_p1d1.json` | `passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `conformance_p1d1.json` | `spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `conformance_p1d1.json` | `total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `conformance_p1d1.json` | `tests` | karar | K-P2-0.4: the report holds three new tests (by-id comparison: the 34 of the reference equal, the three new pass) |
| `compare_p2_0` | regress | `p1e/release/conformance_rootmark.json` | `fixtures_manifest_hash` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_rootmark.json` | `passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_rootmark.json` | `spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_rootmark.json` | `total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_rootmark.json` | `tests` | karar | K-P2-0.4: the report holds three new tests (by-id comparison: the 34 of the reference equal, the three new pass) |
| `compare_p2_0` | regress | `p1b1_checks.json` | `criteria/B1-3/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-P2-0.4: fixture manifest: C2-11, C2-12, C3-08 |
| `compare_p2_0` | regress | `p1c1_checks.json` | `C1-7/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1c_checks.json` | `C-1/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1d/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1d1_checks.json` | `D1-10/pass` | karar | K-P2-0.4: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0` | regress | `p1d1_checks.json` | `D1-10/passed` | karar | K-P2-0.4: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0` | regress | `p1d1_checks.json` | `D1-10/spec_version` | karar | K-P2-0.4: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0` | regress | `p1d1_checks.json` | `D1-12/regress_vs_p1d/conformance/spec_version/1` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1d_checks.json` | `D-9/conformance/spec_version/1` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/plugin_e2e/plugin_e2e.json` | `steps/0/shows/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_equiv.json` | `passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_equiv.json` | `spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/conformance_equiv.json` | `total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/release_local.json` | `conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/release_local.json` | `conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/release/release_local.json` | `spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/scene/scene.json` | `steps/0/shows/mcp_spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1/release/release.json` | `spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1/release/release.json` | `targets/0/conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1/release/release.json` | `targets/0/conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1/release/release.json` | `targets/0/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1/release/release.json` | `commit` | karar | K-P2-0.1: the release package of this round's commit (the reference is P1-E.1's) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/3/item` | karar | K-P2-0.1: R4a's item names the commit of the package (the reference is P1-E.1's) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/1/item` | karar | K-P2-0.4: the checklist item names spec 2.0.0 / 37 (F9) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/4/item` | karar | K-P2-0.4: the checklist item names spec 2.0.0 / 37 (F9) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/9/item` | karar | K-P2-0.4: the checklist item names spec 2.0.0 / 37 (F9) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-5/required/conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-5/required/conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-5/required/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-6/conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-6/conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-6/pass` | karar | K-P2-0.4: E1-6 expects 34/34 and regresses against P1-E: it sees the P2-0 differences |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-6/regress_vs_p1e` | karar | K-P2-0.4: E1-6's regress against P1-E sees every P2-0 difference (this regress lists them by decision) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-7/chain_local_target/spec_version` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-9/conformance/passed` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-9/conformance/total` | karar | K-P2-0.4: spec 2.0.0 and conformance 37 (C2-11, C2-12, C3-08) |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1` | karar | K-P2-0.4: E-9's regress against P1-D.1 sees every P2-0 difference |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/#len` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/14/detail` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/14/item` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/16` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `total` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-4/release_check` | karar | K-P2-0.3: R14 rewritten, R16 added (17 items) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-4/release_md_items/#len` | karar | K-P2-0.3: R16 in docs/RELEASE.md |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-4/release_md_items/16` | karar | K-P2-0.3: R16 in docs/RELEASE.md |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-4/script_items` | karar | K-P2-0.3: R16 scripted |
| `compare_p2_0` | regress | `p1d1/d1_13.json` | `http/status_fields` | karar | K-P2-0.3: contract_check.sh's status step reads every field of the §2 status row |
| `compare_p2_0` | regress | `p1d1/d1_13.json` | `http/http_only` | karar | K-P2-0.5: contract_check.sh's §2.6 step: attest_session, attest over HTTP, absent from MCP |
| `compare_p2_0` | regress | `p1e1/contract.json` | `http/status_fields` | karar | K-P2-0.3: contract_check.sh's status step reads every field of the §2 status row |
| `compare_p2_0` | regress | `p1e1/contract.json` | `http/http_only` | karar | K-P2-0.5: contract_check.sh's §2.6 step: attest_session, attest over HTTP, absent from MCP |
| `compare_p2_0` | regress | `p1e/client.json` | `bundle_bytes` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e/client.json` | `vitest/files` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e/client.json` | `vitest/passed` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e/client.json` | `vitest/tests` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-10/bundle_bytes` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-10/vitest/files` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-10/vitest/passed` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e_checks.json` | `E-10/vitest/tests` | karar | K-P2-0.5: the plugin attests its own saves (Z3): bundle and unit tests |
| `compare_p2_0` | regress | `p1e1/release/release_check.json` | `items/8/detail` | karar | K-P2-0.5: R8 records the plugin bundle size (Z3) |
| `compare_p2_0` | regress | `p1e1/tamper_p1e1.json` | `cases/1/as_expected` | karar | K-P2-0.5: the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14) |
| `compare_p2_0` | regress | `p1e1/tamper_p1e1.json` | `cases/1/detail/as_expected` | karar | K-P2-0.5: the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14) |
| `compare_p2_0` | regress | `p1e1/tamper_p1e1.json` | `pass` | karar | K-P2-0.5: the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14) |
| `compare_p2_0` | regress | `p1e1/tamper_p1e1.json` | `rejected` | karar | K-P2-0.5: the p1e-final server case expects only status gaps; it now also lacks §2.6 and status.attestation (F14) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-8/deliberate/1/as_expected` | karar | K-P2-0.5: the same p1e-final case (F14) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `E1-8/pass` | karar | K-P2-0.5: the same p1e-final case (F14) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `tamper/pass` | karar | K-P2-0.5: the same p1e-final case (F14) |
| `compare_p2_0` | regress | `p1e1_checks.json` | `tamper/rejected` | karar | K-P2-0.5: the same p1e-final case (F14) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `fields_declared` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `fields_in_code` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `file_constants` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `modules/#len` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `modules/9` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e/check_compare_fields.json` | `modules/10` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `fields_declared` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `fields_in_code` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `file_constants` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `modules/#len` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `modules/9` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/check_compare_fields.json` | `modules/10` | karar | F13: the field audit sees the new comparator compare_p2_0 (its module, rows and file constants) |
| `compare_p2_0` | regress | `p1e1/historical.json` | `counted_fails` | karar | K-P2-0.4: eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them |
| `compare_p2_0` | regress | `p1e1/historical.json` | `counted` | karar | K-P2-0.4: eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them |
| `compare_p2_0` | regress | `p1e1/historical.json` | `fails` | karar | K-P2-0.4: eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them |
| `compare_p2_0` | regress | `p1e1/historical.json` | `historical` | karar | K-P2-0.4: eval.historical (reference P1-E) sees the FAILs P2-0 decisions cause; eval.historical_p2_0 classifies them |
| `compare_p2_0` | regress | `p1e1/status/e1_1.json` | `fixture` | karar | K-GH.1: the fixture label after the project rename (GH-0, after the P1-E.1 official RUN) |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1b` | duvar-saati | D-B test-2: gerçek Obsidian kayıtlarının yanlış-`unverified` oranı (zamanlamaya bağlı) |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1b′` | duvar-saati | D-B test-2: yeniden adlandırma oturumlarının yanlış-`unverified` sayısı |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1c` | duvar-saati | D-B test-2: kayıt → `canonical` gecikmesi (ms) |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1d′` | duvar-saati | D-C′: gerçek Obsidian çakışma tikleri (gecikme ve izleyicinin gözlemine bağlı) |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1e` | duvar-saati | D-A′: gerçek Obsidian yeniden adlandırma senaryoları (gözlem zamanlaması) |
| `compare_p2_0_1` | hepsi | `p2_0_1_checks.json` | `H-P2-0.1f` | duvar-saati | D-B2: CLI tasdikinin izleyiciyle yarışı, gecikme (ms) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2b` | duvar-saati | D-B test-3: gerçek Obsidian kayıtlarının yanlış-`unverified` oranı (zamanlamaya bağlı) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2b′` | duvar-saati | D-B test-3: yeniden adlandırma oturumları ve biçim başına rapor (zamanlamaya bağlı) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2c` | duvar-saati | D-B test-3: kayıt → `canonical` gecikmesi (ms) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2d/d_c_prime` | duvar-saati | D-C′: gerçek Obsidian çakışma tikleri (gecikme ve izleyicinin gözlemine bağlı) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2d/pass` | duvar-saati | D-C ve D-C′'nün birleşik sonucu (D-C′ zamanlamaya bağlı) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2e` | duvar-saati | D-A′: gerçek Obsidian yeniden adlandırma ve taşıma senaryoları (gözlem zamanlaması) |
| `compare_p2_0_2` | hepsi | `p2_0_2_checks.json` | `H-P2-0.2f` | duvar-saati | D-B2: CLI tasdikinin izleyiciyle yarışı, gecikme (ms) |
| `compare_p2_0_2` | regress | `conformance_p1d1.json` | `fixtures_manifest_hash` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `conformance_p1d1.json` | `passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `conformance_p1d1.json` | `spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `conformance_p1d1.json` | `tests` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `conformance_p1d1.json` | `total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1b1_checks.json` | `criteria/B1-3/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-P2-02.3: fixture manifest: C2-14 |
| `compare_p2_0_2` | regress | `p1c1_checks.json` | `C1-7/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1c_checks.json` | `C-1/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1d/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1d1_checks.json` | `D1-10/passed` | karar | K-P2-02.3: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0_2` | regress | `p1d1_checks.json` | `D1-10/spec_version` | karar | K-P2-02.3: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0_2` | regress | `p1d1_checks.json` | `D1-12/regress_vs_p1d/conformance/spec_version/1` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1d_checks.json` | `D-9/conformance/spec_version/1` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `fields_declared` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `fields_in_code` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `file_constants` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `modules/#len` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `modules/11` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/check_compare_fields.json` | `modules/12` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e/client.json` | `bundle_bytes` | karar | K-P2-02.2: the plugin attests every rename (K-P2-02.2): its bundle size |
| `compare_p2_0_2` | regress | `p1e/plugin_e2e/plugin_e2e.json` | `steps/0/shows/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_equiv.json` | `passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_equiv.json` | `spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_equiv.json` | `total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_rootmark.json` | `fixtures_manifest_hash` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_rootmark.json` | `passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_rootmark.json` | `spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_rootmark.json` | `tests` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/conformance_rootmark.json` | `total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/release_local.json` | `conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/release_local.json` | `conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/release/release_local.json` | `spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/scene/scene.json` | `steps/0/shows/mcp_spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `fields_declared` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `fields_in_code` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `file_constants` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `modules/#len` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `modules/11` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/check_compare_fields.json` | `modules/12` | karar | P2-0.2/F6: the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants) |
| `compare_p2_0_2` | regress | `p1e1/historical.json` | `fails` | karar | K-P2-02.3: eval.historical (reference P1-E) sees the FAILs P2-0, P2-0.1 and P2-0.2 decisions cause; eval.historical_p2_0_2 classifies them |
| `compare_p2_0_2` | regress | `p1e1/release/release.json` | `commit` | karar | K-P2-02.1: the release package of this round's commit (the reference is P2-0.1's) |
| `compare_p2_0_2` | regress | `p1e1/release/release.json` | `spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1/release/release.json` | `targets/0/conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1/release/release.json` | `targets/0/conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1/release/release.json` | `targets/0/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/1/item` | karar | K-P2-02.3: the checklist item names 39/39 (P2-0.2/F4) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/14/detail` | karar | K-P2-02.1: R14's detail names the newest ROUND_START (this round's) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/3/item` | karar | K-P2-02.1: R4a's item names the commit of the package (the reference is P2-0.1's) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/4/item` | karar | K-P2-02.3: the checklist item names 39/39 (P2-0.2/F4) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/8/detail` | karar | K-P2-02.2: R8 records the plugin bundle size (every rename attested, K-P2-02.2) |
| `compare_p2_0_2` | regress | `p1e1/release/release_check.json` | `items/9/item` | karar | K-P2-02.3: the checklist item names 39/39 (P2-0.2/F4) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-5/required/conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-5/required/conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-5/required/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-6/conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-6/conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e1_checks.json` | `E1-6/regress_vs_p1e` | karar | K-P2-02.3: E1-6's regress against P1-E sees every P2-0, P2-0.1 and P2-0.2 difference |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-10/bundle_bytes` | karar | K-P2-02.2: the plugin attests every rename (K-P2-02.2): its bundle size |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-7/chain_local_target/spec_version` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-9/conformance/passed` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-9/conformance/total` | karar | K-P2-02.3: spec 2.0.2 and conformance 39 (C2-14) |
| `compare_p2_0_2` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1` | karar | K-P2-02.3: E-9's regress against P1-D.1 sees every P2-0, P2-0.1 and P2-0.2 difference |
| `compare_p2_0_1` | regress | `conformance_p1d1.json` | `fixtures_manifest_hash` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `conformance_p1d1.json` | `passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `conformance_p1d1.json` | `spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `conformance_p1d1.json` | `total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `conformance_p1d1.json` | `tests` | karar | K-P2-01.3: the report holds one new test (by-id comparison: the 37 of the reference equal, C2-13 passes) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_rootmark.json` | `fixtures_manifest_hash` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_rootmark.json` | `passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_rootmark.json` | `spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_rootmark.json` | `total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_rootmark.json` | `tests` | karar | K-P2-01.3: the report holds one new test (by-id comparison: the 37 of the reference equal, C2-13 passes) |
| `compare_p2_0_1` | regress | `p1b1_checks.json` | `criteria/B1-3/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1b_checks.json` | `fixtures_manifest_hash` | karar | K-P2-01.3: fixture manifest: C2-13 |
| `compare_p2_0_1` | regress | `p1c1_checks.json` | `C1-7/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1c_checks.json` | `C-1/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1d/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1d1_checks.json` | `D1-10/passed` | karar | K-P2-01.3: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0_1` | regress | `p1d1_checks.json` | `D1-10/spec_version` | karar | K-P2-01.3: D1-10 expects 34 tests and spec 1.0.3 |
| `compare_p2_0_1` | regress | `p1d1_checks.json` | `D1-12/regress_vs_p1d/conformance/spec_version/1` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1d_checks.json` | `D-9/conformance/spec_version/1` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/plugin_e2e/plugin_e2e.json` | `steps/0/shows/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_equiv.json` | `passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_equiv.json` | `spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/conformance_equiv.json` | `total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/release_local.json` | `conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/release_local.json` | `conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/release/release_local.json` | `spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/scene/scene.json` | `steps/0/shows/mcp_spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e/scene/scene.json` | `steps/0/shows/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1/release/release.json` | `spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1/release/release.json` | `targets/0/conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1/release/release.json` | `targets/0/conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1/release/release.json` | `targets/0/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1/release/release.json` | `commit` | karar | K-P2-01.1: the release package of this round's commit (the reference is P2-0's) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/3/item` | karar | K-P2-01.1: R4a's item names the commit of the package (the reference is P2-0's) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/3/detail` | karar | K-P2-01.1: R4a's detail names the commit of the package (the reference is P2-0's) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/14/detail` | karar | K-P2-01.1: R14's detail names the newest ROUND_START (this round's) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/1/item` | karar | K-P2-01.3: the checklist item names 38/38 (P2-0.1/F3) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/4/item` | karar | K-P2-01.3: the checklist item names 38/38 (P2-0.1/F3) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/9/item` | karar | K-P2-01.3: the checklist item names 38/38 (P2-0.1/F3) |
| `compare_p2_0_1` | regress | `p1e1/release/release_check.json` | `items/8/detail` | karar | K-P2-01.2: R8 records the plugin bundle size (process wrapper, content, link updates) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-5/required/conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-5/required/conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-5/required/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-6/conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-6/conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e1_checks.json` | `E1-6/regress_vs_p1e` | karar | K-P2-01.3: E1-6's regress against P1-E sees every P2-0 and P2-0.1 difference |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-7/chain_local_target/conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-7/chain_local_target/spec_version` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-9/conformance/passed` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-9/conformance/total` | karar | K-P2-01.3: spec 2.0.1 and conformance 38 (C2-13) |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-9/regress_vs_p1d1` | karar | K-P2-01.3: E-9's regress against P1-D.1 sees every P2-0 and P2-0.1 difference |
| `compare_p2_0_1` | regress | `p1d1/d1_13.json` | `http/status_fields` | karar | K-P2-01.4: contract_check.sh's status step reads the new status.attestation fields link_updates (K-P2-01.2) and unmatched (K-P2-01.4) |
| `compare_p2_0_1` | regress | `p1e1/contract.json` | `http/status_fields` | karar | K-P2-01.4: contract_check.sh's status step reads the new status.attestation fields link_updates (K-P2-01.2) and unmatched (K-P2-01.4) |
| `compare_p2_0_1` | regress | `p1e/client.json` | `bundle_bytes` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-10/bundle_bytes` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e/client.json` | `vitest/passed` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-10/vitest/passed` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e/client.json` | `vitest/tests` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e_checks.json` | `E-10/vitest/tests` | karar | K-P2-01.2: the plugin attests link updates and sends content (Y1, Y2): bundle and unit tests |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `fields_declared` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `fields_in_code` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `file_constants` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `modules/#len` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `modules/10` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e/check_compare_fields.json` | `modules/11` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `fields_declared` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `fields_in_code` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `file_constants` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `modules/#len` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `modules/10` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/check_compare_fields.json` | `modules/11` | karar | F6: the field audit sees the new comparator compare_p2_0_1 (its module, rows and file constants) |
| `compare_p2_0_1` | regress | `p1e1/historical.json` | `counted_fails` | karar | K-P2-01.3: eval.historical (reference P1-E) sees the FAILs P2-0 and P2-0.1 decisions cause; eval.historical_p2_0_1 classifies them |
| `compare_p2_0_1` | regress | `p1e1/historical.json` | `counted` | karar | K-P2-01.3: eval.historical (reference P1-E) sees the FAILs P2-0 and P2-0.1 decisions cause; eval.historical_p2_0_1 classifies them |
| `compare_p2_0_1` | regress | `p1e1/historical.json` | `fails` | karar | K-P2-01.3: eval.historical (reference P1-E) sees the FAILs P2-0 and P2-0.1 decisions cause; eval.historical_p2_0_1 classifies them |
| `compare_p2a` | regress | `p1d1/d1_13.json` | `codes` | karar | K-P2A.2: the contract's codes: migration_required (12) |
| `compare_p2a` | regress | `p1d1/d1_13.json` | `http` | karar | K-P2A.2: contract_check over HTTP: migration_required triggered, stale state's search code, status.migrations fields |
| `compare_p2a` | regress | `p1d1_checks.json` | `D1-13/codes` | karar | K-P2A.2: D1-13's code count: 12 |
| `compare_p2a` | regress | `p1e/check_compare_fields.json` | `file_constants` | karar | P2-A/F6: the field audit sees the new comparator compare_p2a |
| `compare_p2a` | regress | `p1e/check_compare_fields.json` | `modules` | karar | P2-A/F6: the field audit sees the new comparator compare_p2a |
| `compare_p2a` | regress | `p1e/client.json` | `bundle_bytes` | karar | K-P2A.2: the plugin's index notice names `rootmark migrate` (view.ts): its bundle size |
| `compare_p2a` | regress | `p1e/client.json` | `vitest` | karar | K-P2A.2: the client's tests: one more (the generated contract's code) |
| `compare_p2a` | regress | `p1e/e0/e14.json` | `auto_rebuild_false` | karar | K-P2A.2: E-14 on the /2 index: search 503 migration_required |
| `compare_p2a` | regress | `p1e/e0/e14.json` | `pass` | karar | K-P2A.2: E-14 on the /2 index: search 503 migration_required |
| `compare_p2a` | regress | `p1e/e0/e14.json` | `passed` | karar | K-P2A.2: E-14 on the /2 index: search 503 migration_required |
| `compare_p2a` | regress | `p1e1/check_compare_fields.json` | `file_constants` | karar | P2-A/F6: the field audit sees the new comparator compare_p2a |
| `compare_p2a` | regress | `p1e1/check_compare_fields.json` | `modules` | karar | P2-A/F6: the field audit sees the new comparator compare_p2a |
| `compare_p2a` | regress | `p1e1/contract.json` | `codes` | karar | K-P2A.2: contract_check: migration_required (12 codes) |
| `compare_p2a` | regress | `p1e1/contract.json` | `http` | karar | K-P2A.2: contract_check over HTTP: migration_required, stale state's search code, status.migrations fields |
| `compare_p2a` | regress | `p1e1/historical.json` | `counted` | karar | K-P2A.2: P1-E.1's classifier sees the migration-required FAILs (E1-1, E1-4, E1-9, E-14) |
| `compare_p2a` | regress | `p1e1/historical.json` | `counted_fails` | karar | K-P2A.2: P1-E.1's classifier sees the migration-required FAILs (E1-1, E1-4, E1-9, E-14) |
| `compare_p2a` | regress | `p1e1/historical.json` | `fails` | karar | K-P2A.2: P1-E.1's classifier sees the migration-required FAILs (E1-1, E1-4, E1-9, E-14) |
| `compare_p2a` | regress | `p1e1/release/plugin_e2e_real.json` | `e1_9_connection` | karar | K-P2A.2: the real-Obsidian e2e on the /2 index: migration_required before the rebuild, the notice names `rootmark migrate` |
| `compare_p2a` | regress | `p1e1/release/plugin_e2e_real.json` | `pass` | karar | K-P2A.2: the real-Obsidian e2e on the /2 index (E1-9) |
| `compare_p2a` | regress | `p1e1/release/release.json` | `commit` | karar | K-P2A.3: the release package is built from this round's commit |
| `compare_p2a` | regress | `p1e1/release/release_check.json` | `items` | karar | K-P2A.3: release_check R17–R20 (P2-A/F3); R15 follows E1-9 (K-P2A.2) |
| `compare_p2a` | regress | `p1e1/release/release_check.json` | `pass` | karar | K-P2A.3: release_check R17–R20; R15 follows E1-9 |
| `compare_p2a` | regress | `p1e1/release/release_check.json` | `passed` | karar | K-P2A.3: release_check R17–R20; R15 follows E1-9 |
| `compare_p2a` | regress | `p1e1/release/release_check.json` | `total` | karar | K-P2A.3: release_check R17–R20 (21 items) |
| `compare_p2a` | regress | `p1e1/status/e1_1.json` | `auto_rebuild_false` | karar | K-P2A.2: E1-1 on the /2 index: search 503 migration_required |
| `compare_p2a` | regress | `p1e1/status/e1_1.json` | `cases` | karar | K-P2A.2: E1-1 on the /2 index: search 503 migration_required |
| `compare_p2a` | regress | `p1e1/status/e1_1.json` | `pass` | karar | K-P2A.2: E1-1 on the /2 index |
| `compare_p2a` | regress | `p1e1/status/e1_1.json` | `passed` | karar | K-P2A.2: E1-1 on the /2 index |
| `compare_p2a` | regress | `p1e1_checks.json` | `E1-1` | karar | K-P2A.2: E1-1: migration required (eval.historical_p2a) |
| `compare_p2a` | regress | `p1e1_checks.json` | `E1-4` | karar | K-P2A.3: E1-4: release_check R17–R20 (K-P2A.3) and R15 (E1-9, K-P2A.2) |
| `compare_p2a` | regress | `p1e1_checks.json` | `E1-6` | karar | K-P2A.2: E1-6's regress against P1-E sees the new code |
| `compare_p2a` | regress | `p1e1_checks.json` | `E1-9` | karar | K-P2A.2: E1-9: migration required (eval.historical_p2a) |
| `compare_p2a` | regress | `p1e_checks.json` | `E-10/bundle_bytes` | karar | K-P2A.2: the plugin bundle (view.ts notice) |
| `compare_p2a` | regress | `p1e_checks.json` | `E-10/vitest` | karar | K-P2A.2: the client's tests: one more |
| `compare_p2a` | regress | `p1e_checks.json` | `E-14` | karar | K-P2A.2: E-14: migration required (eval.historical_p2a) |
| `compare_p2a` | regress | `p1e_checks.json` | `E-9` | karar | K-P2A.2: E-9's regress against P1-D.1 sees the new code |
| `compare_p2a` | regress | `p2_0_2_checks.json` | `fail_summary` | karar | K-P2A.2: P2-0.2's classifier sees the migration-required FAILs; P2-A's classifier is eval.historical_p2a |
| `compare_p2a` | regress | `p2_0_2_checks.json` | `regress` | karar | K-P2A.2: P2-0.2's regress against P2-0.1; P2-A's regress is this comparison |
| `compare_p2l` | hepsi | `fast/conformance.json` | `fixtures_root` | ortam | zincirin klonunun mutlak fixture yolu |
<!-- /fields-table -->

## Dosya tablosu
<!-- files-table -->
| Sabit | Dosyalar |
|---|---|
| `compare_runs.RESULT_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h2_tuning.json`, `h2.json`, `h3.json`, `h4.json` |
| `compare_runs.RESULT_FILES_V3` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json` |
| `compare_p1a.RESULT_FILES_P1A` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json` |
| `compare_p1b.RESULT_FILES_P1B` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json` |
| `compare_p1b.BYTE_FILES` | `governance/vault/.rootmark/events.jsonl`, `governance/vault/.rootmark/traces.jsonl` |
| `compare_p1b.BYTE_DIRS` | `governance/vault/.rootmark/checkpoints` |
| `compare_p1c.PAIR_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json`, `c3_quality.json`, `p1c_checks.json`, `p1c_index.json` |
| `compare_p1c.REGRESS_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json` |
| `compare_p1c1.PAIR_FILES` | `h1s_tuning.json`, `h1s.json`, `h1r.json`, `h1s_v3_tuning.json`, `h1s_v3.json`, `h1r_v3.json`, `h3_v3.json`, `h4_v3.json`, `h4_v2repro.json`, `holdout.json`, `split_grid.json`, `holdout_split.json`, `p1b_checks.json`, `governance/governance.json`, `p1b1_checks.json`, `c3_quality.json`, `p1c_checks.json`, `p1c_index.json`, `c1_variant.json`, `p1c1_checks.json` |
| `compare_p1d.P1D_FILES` | `p1d_checks.json`, `p1d/inspector/inspector.json`, `p1d/mcp_e2e/mcp_e2e.json`, `p1d/http/http_e2e.json`, `p1d/scene/scene.json`, `p1d/tamper_p1d.json`, `p1d/d10.json`, `p1d/sigterm.json` |
| `compare_p1d.P1D_BYTES` | `p1d/mcp_e2e/run1/transcript.jsonl`, `p1d/mcp_e2e/run1/transcript_b.jsonl`, `p1d/mcp_e2e/run1/cli_promote.jsonl`, `p1d/mcp_e2e/run1/vault/.rootmark/events.jsonl`, `p1d/mcp_e2e/run1/vault/.rootmark/traces.jsonl`, `p1d/mcp_e2e/run1/vault/.rootmark/proposals.jsonl`, `p1d/mcp_e2e/run1/vault/espresso.md`, `p1d/mcp_e2e/run1/vault/agent/plan.md`, `p1d/scene/scene.json`, `p1d/inspector/tools_list.json` |
| `compare_p1d.CONFORMANCE` | `conformance_p1c1.json` |
| `compare_p1d1.P1D1_FILES` | `p1d1_checks.json`, `conformance_p1d1.json`, `p1d1/p1d1.json`, `p1d1/d1_1.json`, `p1d1/d1_2.json`, `p1d1/d1_3.json`, `p1d1/d1_4.json`, `p1d1/d1_5.json`, `p1d1/d1_6.json`, `p1d1/d1_7.json`, `p1d1/d1_8.json`, `p1d1/d1_9.json`, `p1d1/d1_11.json`, `p1d1/d1_13.json`, `p1d1/v3_compat.json`, `p1d1/v3_compat_mixed.json`, `p1d1/tamper_p1d1.json` |
| `compare_p1d1.P1D1_BYTES` | `p1d1/d1_3.json`, `p1d1/d1_8.json`, `p1d1/d1_13.json` |
| `compare_p1e.P1E_FILES` | `p1e_checks.json`, `p1e/e0/e14.json`, `p1e/e0/e15.json`, `p1e/scene/scene.json`, `p1e/plugin_e2e/plugin_e2e.json`, `p1e/plugin_e2e/access_log.json`, `p1e/client.json`, `p1e/release/release_local.json`, `p1e/release/conformance_rootmark.json`, `p1e/release/conformance_equiv.json`, `p1e/tamper_p1e.json`, `p1e/check_compare_fields.json` |
| `compare_p1e.P1E_BYTES` | `p1e/scene/scene.json`, `p1e/scene/events.jsonl`, `p1e/scene/traces.jsonl`, `p1e/scene/proposals.jsonl`, `p1e/plugin_e2e/plugin_e2e.json`, `p1e/plugin_e2e/access_log.json`, `p1e/plugin_e2e/events.jsonl`, `p1e/plugin_e2e/traces.jsonl`, `p1e/plugin_e2e/proposals.jsonl` |
| `compare_p1e1.P1E1_FILES` | `p1e1_checks.json`, `p1e1/status/e1_1.json`, `p1e1/status/e1_2.json`, `p1e1/contract.json`, `p1e1/tamper_p1e1.json`, `p1e1/tamper_p1e1_e2e.json`, `p1e1/release/release.json`, `p1e1/release/release_check.json`, `p1e1/release/plugin_e2e_real.json`, `p1e1/release/access_log_real.json`, `p1e1/historical.json`, `p1e1/check_compare_fields.json` |
| `compare_p1e1.P1E1_BYTES` | `p1e1/release/plugin_e2e_real.json`, `p1e1/release/access_log_real.json` |
| `compare_p2_0.P2_0_FILES` | `p2_0_checks.json`, `p2_0/tuning.json`, `p2_0/d_e.json`, `p2_0/d_a.json`, `p2_0/d_c.json`, `p2_0/tamper_p2_0.json` |
| `compare_p2_0.P2_0_BYTES` | `p2_0/d_a/events.jsonl`, `p2_0/d_a/attestations.jsonl`, `p2_0/d_c/events.jsonl`, `p2_0/d_c/attestations.jsonl` |
| `compare_p2_0_1.P2_0_1_FILES` | `p2_0_1_checks.json`, `p2_0_1/d_e.json`, `p2_0_1/d_a.json`, `p2_0_1/d_a2.json`, `p2_0_1/d_c.json`, `p2_0_1/tamper_p2_0_1.json` |
| `compare_p2_0_1.P2_0_1_BYTES` | `p2_0_1/d_a/events.jsonl`, `p2_0_1/d_a/attestations.jsonl`, `p2_0_1/d_a2/events.jsonl`, `p2_0_1/d_a2/attestations.jsonl`, `p2_0_1/d_c/events.jsonl`, `p2_0_1/d_c/attestations.jsonl` |
| `compare_p2_0_2.P2_0_2_FILES` | `p2_0_2_checks.json`, `p2_0_2/rules.json`, `p2_0_2/d_e.json`, `p2_0_2/d_a.json`, `p2_0_2/d_a2.json`, `p2_0_2/d_c.json`, `p2_0_2/tamper_p2_0_2.json` |
| `compare_p2_0_2.P2_0_2_BYTES` | `p2_0_2/d_a/events.jsonl`, `p2_0_2/d_a/attestations.jsonl`, `p2_0_2/d_a2/events.jsonl`, `p2_0_2/d_a2/attestations.jsonl`, `p2_0_2/d_c/events.jsonl`, `p2_0_2/d_c/attestations.jsonl` |
| `compare_p2a.P2A_FILES` | `p2a_checks.json`, `p2a/migrate_old.json`, `p2a/fixture.json`, `p2a/tamper.json`, `p2a/tests.json` |
| `compare_p2l.P2L_FAST_FILES` | `fast/conformance.json`, `fast/conformance_rootmark.json`, `fast/conformance_equiv.json`, `fast/regress.json`, `c3_quality.json`, `p2l_checks.json`, `p2l_quality.json`, `p2l/d_tr.json`, `p2l/english.json`, `p2l/migrate.json` |
| `compare_p2l.P2L_FILES` | `p2l_checks.json`, `p2l_quality.json`, `p2l/d_tr.json`, `p2l/english.json`, `p2l/migrate.json` |
<!-- /files-table -->
