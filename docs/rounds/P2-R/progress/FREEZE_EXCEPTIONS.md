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

## F3 · R0 · `research/eval/historical_fails_p2b.json`, `research/eval/migration_required_fails_p2b.json` — K-P2B.4 girdileri çıkar (GOAL P2-R R0 (b))
- **Ne:** uyarlanan kaydedicilerin K-P2B.4 girdileri ve yolları listeden çıktı: dört girdi bütünüyle (`p1d1` D1-13, `p1d1` tamper, `p1e` E-16, `p1e` tamper), dört girdinin K-P2B.4 yolları (`p1d1` D1-12'nin 14 yolu, `p1d` D-2'nin 2, `p1e1` E1-8'in 2, `p1e1` tamper'ın 1), göç gerekli listesinde E1-9'un `contract_check_pass` yolu ve `K-P2B.4` kararı. Geliştirme RUN'ında hâlâ farklı olan yollar adlarıyla geri girdi: D1-12'nin Inspector ve D-1 yolları (10) **K-P2B.4 altında** (P1-D'nin 10 araçlı resmi RUN'ına karşı 11 araç: aracın kendisi, kaydedici değil; ASSUMPTIONS R0-7), uyarlanan `mcp_e2e`'nin yeni yolları **K-P2R.3** altında (D1-12 16, D-2 3). Sınıflandırıcının kodu (`historical_p2b.py`, `historical_p2a.py`, `historical.py`) değişmedi. `R0_f3.diff`.
- **Neden:** GOAL P2-R R0 (b) ve R-1 (b): "K-P2B.4'ün … girdileri listeden çıkar; sınıflandırıcı kodu değişmez; sayılan FAIL 0".
- **Önce/sonra:** aynı geliştirme RUN'ına (`20261026T082210Z-2f85d43`, `reports/rounds/P2-R/devchain_2f85d43/`) karşı `eval.historical_p2b`: önce (P2-B'nin listeleri) sayılan FAIL 2 (D1-12, D-2), sonra 0 (30 tarihsel, 2 göç gerekli). `git diff`: hiçbir başka girdi ya da yol değişmedi; değişen `why`/`about` metinleri yalnız ekleme.
- **Gözden geçirici onayı:** R5.review.md.

## F4 · R2 · `scripts/contract_check.sh`, `server/tests/contract.rs` — yeni hata kodu `archived`
- **Ne:** `contract_check.sh`'in `code_req`'ine `archived` kolu: HTTP vault'unda iki yeni durum (`propose` + `checkpoint`), tarife geçici `[retention] keep_checkpoints = 1`, `POST /v1/prune`, tarif geri, ilk checkpoint'e `at` ile `search` → 410 `archived`. `contract.rs`: `http_statuses_of_the_codes`'a `(Code::Archived, 410)`, yeni `archived_from_a_real_call` (CLI ile iki checkpoint, `prune`, `expect_code(… Code::Archived)`). Silinen satır yok (`R2_f4.diff`: yalnız ekleme).
- **Neden:** GOAL P2-R 3 ve 5: arşivlenmiş checkpoint'e sorgu `archived` koduyla reddedilir, CLIENT_CONTRACT §3'e yeni kod. Dondurulmuş D1-13 denetleyicisi sözleşmenin her kodunu bir testle ve gerçek bir HTTP çağrısıyla eşler; yeni kod olmadan çıkış 1 olurdu (emsal P2-A/F2'nin `migration_required`'ı).
- **Önce/sonra:** önce (F1 sürümü, `archived` sözleşmede ve sunucuda): `cargo test --test contract` `http_statuses_of_the_codes` düşer (12 ≠ 13), `contract_check` `http-code:archived` ve `code:archived` eksik der; sonra `contract` 5/5, `contract_check` çıkış 0, "11 tools, 13 codes" ve `archived` 410 (`reports/rounds/P2-R/r0b/contract_check_r2.json`).
- **Gözden geçirici onayı:** R5.review.md.

## F5 · R3 · `research/eval/check_p2b_align_regress.py` — biçim
- **Ne:** `ruff format` iki liste ifadesini satırlara böldü; davranış aynen (`R3_f5.diff`: yalnız satır kırma).
- **Neden:** R0'da (`2f85d43`) eklenen ve F3'te (`research/eval/**` satırı) donan dosya `ruff format --check`'ten geçmiyordu: `make lint` düşerdi (R-10 "kalite kapıları temiz").
- **Önce/sonra:** önce `ruff format --check .` "1 file would be reformatted" (bu dosya); sonra 251/251 biçimli; `tests/test_p2r_tools.py` 4/4 aynen.
- **Gözden geçirici onayı:** R5.review.md.
