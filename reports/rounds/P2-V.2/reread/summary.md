# P2-V.2 — Yeniden okuma (Y3; GOAL V2-2, V2-3)

Girdi: P2-V.1'in kademe 2 koşularının indirilmiş yapıtları (`reports/rounds/P2-V.2/ci/<koşu>/`, blake2b manifestli; K-P2V2.1) ve Mac kümesi kaydı `reports/rounds/P2-V.1/mac/20261031T204025Z-58377de/mac_checks.json`. Yeni ölçüm yok. Komutlar `verify.yml`'nin karşılaştırma işininkiler (`research/p2v2/reread.py`): yerelde dondurulmuş `compare_p2r pair` (CI'ınkiyle `run_a`/`run_b` yankısı dışında aynı: `frozen_compare_vs_ci.json`, 3/3), `tier2 pair --frozen`, `tier2 fails` (A, B; zincirin sınıflandırıcısıyla), `eval.portable --all` (A, B), `tier2 criteria` (A, B); `41239746833` için `tier2 equiv` V0m `40613392932` A ↔ A, `--mac` P2-V.1 Mac kaydı. "Sonra" = P2-V.2 envanteri (`reread/<koşu>/`), "önce" = aynı araç, P2-V.1 envanteri (`reread/<koşu>/before_p2v1/`).

## Kademe 2 (V2-2)
| Koşu | Commit; işlemciler | `tier2 pair` D farkı (alan / bayt; alan sayısı) | `tier2 fails` sayılan FAIL A / B (önce) | `portable --all` bulgu A / B (önce); atlanan | Kriterler A / B |
|---|---|---|---|---|---|
| 41239746833 | `58377de`; EPYC 7763 / EPYC 7763 | **0 / 0** (788 890) | **0 / 0** (1 / 1: `tamper·det`) — `tamper·det` tarihsel, kayıt eşit | **0 / 0** (4 / 4) — H5 ×1 her zincirde | 135 / 135 |
| 41239754895 | `58377de`; EPYC 7763 / EPYC 7763 | **0 / 0** (788 907) | **0 / 0** (1 / 1) — tarihsel, kayıt eşit | **0 / 0** (4 / 4) — H5 | 135 / 135 |
| 41292205176 | `695f952`; EPYC 9V45 / EPYC 7763 | **0 / 0** (788 913) | **0 / 0** (1 / 1) — tarihsel, kayıt eşit | **0 / 0** (4 / 4) — H5 | 135 / 135 |

- **`tier2 equiv`** (V0m `352f790` ↔ `41239746833` A, `equiv_v0m.json`): adsız D farkı **0** (önce 1: R7 `detail`), D bayt farkı 0; adlı farklar E1 **216 976** (önce 216 975; +1 = E1-W), Z/O 133, H 2, commit 3; kriterler eski 135 ⊆ yeni 135 (+ Mac 23), **kaybolan 0**; `pass` true (önce false).
- **Hiçbir kriter kaybolmadı:** her koşuda her zincirin kriter kimlikleri (`criteria_{A,B}.json`, `tamper·det` dahil, 135) P2-V.1 envanteriyle alınan listeyle ve P2-V.1'in commit'li `criteria_*.json`'uyla aynı küme (bayt eşit).
- **"Önce" çıktıları P2-V.1'in kayıtlarıyla aynı:** `before_p2v1_vs_p2v1_records.json` — 22 dosyadan 19 bayt eşit, 3 (`tier2_pair.json`) yalnız `frozen_result_read` girdi yolu yankısında farklı, 0 farklı (FREEZE_EXCEPTIONS).

## Önce ↔ sonra farkı yalnız dört adda (`research/p2v2/name_diff.py`, `reread/<koşu>/before_after_names.json`; açıklanamayan fark **0** her koşuda)
| Dosya | Farklı yaprak (her koşuda) | Ad |
|---|---|---|
| `tier2_pair.json` | 2: `inventory` (okunan envanterin adı), `frozen_result_read` (çıktı dizininin yankısı) | — (hüküm alanları aynı: D farkı 0, aynı `exclusions_used`) |
| `tier2_fails_{A,B}.json` | 18: `counted` / `counted_fails` / `pass` (sayılan `tamper·det` düştü; başka sayılan yok, yeni sayılan yok), `historical` (tarihsel okuma satırı, `record_equal: true`), `derived_criteria›0›{reading, record_equal}` | **T3** `tamper·det` tarihsel |
| `portable_{A,B}.json` | 23: `findings` (düşenlerin hepsi `conformance.json` `fixtures_root`; yeni bulgu yok), `pass`, `skipped_by_name` (H5) | **T4** H5 |
| `criteria_{A,B}.json` | 0 | — |
| `equiv_v0m.json` | 4: `d_fields_differing` (R7 `detail` düştü; yalnız o), `pass`, `named_counts›E1` (+1) | **T2** E1-W |

## Mac kümesi (V2-3; `verify_mac reread`)
`mac_reread.json` (P2-V.2 envanteri), `mac_reread_before_p2v1.json` (P2-V.1 envanteri — P2-V.1'in `mac_checks.json`'uyla **bayt eşit**).
- **Değerler kayıtla bayt eşit:** 23/23 kriter (`mac_values.json`; `reread` her kriterin `values`'ını kaydınkiyle `dump_json` baytlarıyla karşılaştırır).
- **`reported_only` beşi**, kimlik, değer ve P2-V eşikleriyle, `in_verdict: false`, `carry: P2-C`: H-P2-0.3b-yük **0/3** (≤ 0.02), H-P2-0.3c **0/3** (p95 < 500 ms), H-P2-0.3d **0/3** (D-C ≥ 0.99; D-C′ ≥ 0.95 ve ajan canonical = 0), H-P2-0.3e **0/3** (ajan canonical = 0; sahte 10/10), tamper e2e **2/3** (gerçek Obsidian kasıtlı vakaları reddedilir, kontroller geçer).
- **Deterministik O** C1-8 **1/1 geçti** (`e1_fields` okuması).
- **Tarihsel üçü kayıt eşit:** E-7, E1-4 (tek koşu; hükümde kayıt eşitliğiyle), H-P2-0.3b (hükmün dışında).
- **Z** kümeleri 3 tekrar değerleriyle, hükmün dışında: D-10 0/3, Z-DA 0/3, Z-DC 0/3, Z-DB2 0/3 (değerler P2-V.1 kaydındaki gibi).
- Öteki O kriterleri 3/3 ile hükümde: E1-3 **3/3**, E1-9 **3/3**.
- **Hüküm: geçti** (`pass: true`; hükümdeki kriterler C1-8, E-7, E1-4, E1-3, E1-9).
- Önce ↔ sonra (`mac_before_after_names.json`, 71 yaprak, açıklanamayan 0): `reading`, `in_verdict`, `reported_only` etiketleri (T5) ve E1-3, E1-9'un satır `pass`'i (`null` → `true`: T5 muafiyeti yalnız beşe verdi, öteki O kriterleri 3/3 ile hükümde). Hiçbir `values` yaprağı farklı değil.

## Sonuç
V2-2: D farkı 0 (3/3 koşu), sayılan FAIL 0 (A ve B, 3/3), bulgu 0 (A ve B, 3/3), equiv adsız D farkı 0, kriter kümesi aynı — **geçti**. V2-3: değerler bayt eşit, `reported_only` 5 (`in_verdict: false`), deterministik O 1/1, tarihsel üçü kayıt eşit, Z 3 tekrar değerleriyle, hüküm geçti — **geçti**. Yeni ad eklenmedi.
