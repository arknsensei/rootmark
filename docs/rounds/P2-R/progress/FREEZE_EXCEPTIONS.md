# P2-R — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, R5'te (K-P2.4).

## F1 · R0 · `scripts/mcp_e2e.sh`, `scripts/contract_check.sh`, `server/tests/contract.rs` — kaydedicilerin 11 araca uyarlanması (K-P2B.4'ün önerisi)
- **Ne:** `mcp_e2e.sh`: `TOOLS`'a `events` (tek satır değişti), oturumun sonuna üç `events` çağrısı (248 geçerli sayfa, 249 `kinds` süzgeci, 250 bilinmeyen tür) ve onların üç kontrolü; önceki 247 çağrının kimlikleri ve kontrolleri aynen. `contract_check.sh`: `tool_args`'a `events) echo '{"since_seq":0,"limit":5}'` (ve bir yorum). `contract.rs`: `every_tool_answers`'a `call_tool(&k, "events", …)` ve iki iddia. Silinen tek satır `TOOLS`'un eski tanımı (`R0_f1.diff`).
- **Neden:** GOAL P2-R R0 (b) (K-P2B.4'ün önerisi): P2-B'nin 11. aracı `events` bu kaydedicilerde FAIL kaydettiriyordu (D-2 el sıkışması, D1-13 `tool:events`/`POST /v1/events` eksik ve onlara dayanan kontroller); sınıflandırıcının listesine K-P2B.4 ile girmişlerdi. Ölçülen şey gevşemez: her araç (artık 11) `tools/list`'te ve sözleşmede, her araç gerçek bir test çağrısı ve gerçek bir HTTP çağrısıyla eşleşir; kurcalamaların reddi aynen (kurcalama betiklerine dokunulmadı).
- **Önce/sonra:** `contract_check.sh` önce çıkış 1, `missing: ["tool:events", "http-endpoint:POST /v1/events (no request in this script)"]` (`reports/rounds/P2-R/r0b/contract_check_before.json`); sonra çıkış 0, "11 tools, 12 codes", `missing: []` (`contract_check_after.json`). `mcp_e2e.sh` sonra: el sıkışma `true` ("tools/list has every tool of this round" 11 araçla geçer), `events` 3/3; araçlar 9/11 — kalan iki aracın (`status`, `propose`) FAIL'leri önceki turlardan tarihsel ve sınıflandırıcıda (`mcp_e2e_after.json`). `cargo test --test contract`: 4/4. Sınıflandırıcıya etkisi geliştirme zincirinde ölçülür (R0.md).
- **Gözden geçirici onayı:** R5.review.md.

## F2 · R0 · `scripts/mcp_e2e.sh` — `events` kontrolleri kontrol listesinin sonunda
- **Ne:** F1'in üç `events` kontrolü (çağrı 248–250) kontrol listesinin ortasından (`propose`'un olay denetiminden önce) en sonuna, `per_tool` hesabından hemen önceye taşındı; kontrollerin kendisi, çağrılar ve öteki satırlar aynen (`R0_f2.diff`: yalnız taşıma ve bir yorum).
- **Neden:** geliştirme zinciri `20261026T082210Z-2f85d43`'te D1-12'nin P1-D'ye karşı gerilemesi, ortaya eklenen üç kontrolün sonrakilerin konumunu kaydırması yüzünden 36 `mcp_e2e.json:checks›<n>›…` yolu gösterdi (D-2 3 yol). Eski turların kayıtlarıyla karşılaştırmalarda yalnız yeni kontroller farklı olsun diye yeni kontroller sona alındı; ölçülen şey değişmez.
- **Önce/sonra:** önce (F1): D1-12 farklı alanları 46 (10 Inspector/D-1, 36 `mcp_e2e`); sonra: R0.md (geliştirme zincirinde `p1d-checks` ve `p1d1-checks` aynı RUN'da yeniden koşar). `mcp_e2e` sonucu aynen: el sıkışma `true`, `events` 3/3.
- **Gözden geçirici onayı:** R5.review.md.
