# Geliştirme zinciri `20261026T114425Z-95139c8` (P2-R/R5)

- `scripts/verify.sh chain` `95139c8`'te (F6, F7 sonrası), `verify.sh`'nin bir kopyasından koştu; kesintisiz bitti (4806 s, Obsidian adımları dahil). Geliştirme kaydıdır, resmi değil.
- `historical_p2r_before.json`: F7'deki listelerle sayılan FAIL 4 (D1-12, D-2, E1-8, tamper): P2-B'nin RUN'ı bu alanları başarısız tutar, F1'in uyarlaması onları yeniden geçirir. `historical_p2r_after.json`: F8 listeleriyle sayılan FAIL 0 (30 tarihsel, 2 göç gerekli).
- `p2r_checks.json`: R-1, R-2/R-4, R-5/R-8 `pass: true`. `p2r/volume.json` R3'ün kaydıyla (`reports/rounds/P2-R/r3/volume.json`, `88c50eb`) bayt bayt aynı; `p2r/det.json` çiftin karşılaştırdığı belirlenim kaydı.
- Uyarlanan kaydedicilerin kayıtları: `p1d_checks.json` D-2 el sıkışması `true` (2/2); `p1e1_checks.json` E1-8 `contract_check_pass: true`, kurcalama kontrolleri 3/3; `p1d1_checks.json`.
