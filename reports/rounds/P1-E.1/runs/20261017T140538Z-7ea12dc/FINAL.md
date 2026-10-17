# P1-E.1 — FINAL

- RUN: `20261017T140538Z-7ea12dc`; commit `7ea12dc6543b5d0cc1fc6257aec4095735ff1543`
- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).

## Çıkış kriterleri

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| E1-1 | Gerçek `/2` indeks + `auto_rebuild=false`: `status` GET/POST/MCP 200 `stale`; `search` 503; `blame/checkpoints/trace` 200; `auto_rebuild=true` → hepsi 200 | 8/8 | 8/8 | geçti | `p1e1/status/e1_1.json` |
| E1-2 | Okunamayan manifest: `status` 200 `damaged`; `search` 503 | 2/2 | 2/2 | geçti | `p1e1/status/e1_2.json` |
| E1-3 | Gerçek Obsidian e2e harness 11 adım (yerel) | 11/11; E-3 True; sözleşme dışı 0/57 | 11/11 | geçti | `p1e1/release/plugin_e2e_real.json` |
| E1-4 | `release_check.sh` R7 SIGPIPE düzeltmesi; kontrol listesinin bütün maddeleri betikle | R7 True; 16 madde, betikte 16; release_check 16/16 | geçer | geçti | `p1e1/release/release_check.json` |
| E1-5 | `release.yml` insan tetiklemeli; `aarch64-apple-darwin` kaydı (hash, `--version`, `--spec-version`, 34/34); diğerleri kayıt ya da atlandı | aarch64-apple-darwin mdk 0.1.0 spec 1.0.3 conformance 34/34, sha256 True; diğer: {'x86_64-pc-windows-msvc': 'skipped', 'x86_64-unknown-linux-gnu': 'skipped'}; R13 True | 1/1 zorunlu | geçti | `p1e1/release/release.json` |
| E1-6 | Uyumluluk 34/34; P1-E kalite alanları regresyon (P1-E resmi RUN) | 101929 alan, hariç dışı fark 0, karar gereği 29 alt ağaç, kullanılmayan hariç 0; uyumluluk 34/34 | fark 0 | geçti | `regress_vs_p1e.json` |
| E1-7 | Çift koşturma (A ↔ B) | 103189 alan, fark 0; yapıtlar bayt eşit True | fark 0 | geçti | `pair_compare.json` |
| E1-8 | `contract_check.sh` HTTP adımı `status`'un yeni alanını ve 503 istisnasının `status`'u kapsamadığını denetler | durumlar ['ok', 'stale', 'damaged', 'missing']; kasıtlı 3/3 | geçer | geçti | `p1e1/contract.json` |
| E1-9 | Sözleşme §3 güncel; eklenti bağlantı ekranı `stale/damaged` durumunu ve yeniden kurma yolunu gösteriyor (e2e adımı) | durum çubuğu `mdk 0.1.0 · human:editor · index stale`; yeniden kurma sonrası `mdk 0.1.0 · human:editor` | 1/1 | geçti | `p1e1/release/plugin_e2e_real.json, p1e1/contract.json` |

Kasıtlı vakalar (`p1e1/tamper_p1e1.json`, `p1e1/tamper_p1e1_e2e.json`): 4/4 reddedildi, kontroller 3/3, gerçek Obsidian vakaları koştu: True → geçti.

## Önceki turların kaydedicileri (K-E1.4)

`p1e1/historical.json` (referans: `reports/rounds/P1-E/runs/20261017T024341Z-632f29e`): 10 tarihsel FAIL sayılmadı; sayılan FAIL 0.

- B1-3 (p1b1_checks.json): FAIL — tarihsel, karar K-C1.3, K-C1.5, K-D1.2, K-D1.4-T, K-E0.1 ile kapandı
- C1-7 (p1c1_checks.json): FAIL — tarihsel, karar K-D1.2, K-D1.4-T, K-E0.1 ile kapandı
- C-11 (p1c_checks.json): FAIL — tarihsel, karar K-D1.2, K-D1.4-T, K-E0.1 ile kapandı
- C-3 (p1c_checks.json): FAIL — tarihsel, karar K-C8-T, K-C1.1, K-C1.2 ile kapandı
- D1-12 (p1d1_checks.json): FAIL — tarihsel, karar K-E1.2 ile kapandı
- D-2 (p1d_checks.json): FAIL — tarihsel, karar K-D1.1, K-D1.5, K-E0.4, K-D1.4-T, K-E1.2 ile kapandı
- D-7 (p1d_checks.json): FAIL — tarihsel, karar K-D1.4-T, K-D1.5 ile kapandı
- D-9 (p1d_checks.json): FAIL — tarihsel, karar K-D1.7, K-D1.4-T ile kapandı
- E-7 (p1e_checks.json): FAIL — tarihsel, karar K-E5-T, K-E1.1 ile kapandı
- E-9 (p1e_checks.json): FAIL — tarihsel, karar K-E1.2, F2 (E1-8) ile kapandı

## Karar kuralı

Kalan kriterler: yok; bağımsız doğrulama açık.

**E1-1..E1-9 geçti → bağımsız doğrulama "Güvenilir" derse insan adımı:** `git tag v0.1.0`, `release.yml` tetikleme (workflow_dispatch), `docs/RELEASE.md` kontrol listesi; faz 1 kapanır. Doğrulama bu RUN'ın parçası değildir; etiket ve tetikleme bu turda yapılmadı.
