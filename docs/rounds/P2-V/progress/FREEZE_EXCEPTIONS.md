# P2-V — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, V6'da (K-P2.4; başlık "ölçülen şey gevşedi mi").

## F1 · V3 · `scripts/release_check.sh` — R17'nin `detail`'i host yolu yazmaz (envanter H1; K-P2R.5 (2))
- **Ne:** R17'nin iki `detail` metni. Host arşivi yokken `no archive of this host's target in $dist/bin` → `no archive of this host's target (<hedef>) in the package's bin/`; `install.sh` başarısız olursa günlüğün son satırında geçici dizin, paket dizini ve depo kökü `<tmp>`, `<dist>`, `<repo>` ile değiştirilir. Maddenin kendisi (`ok`, denetim, kimlik, etiket) ve öteki 20 madde aynen.
- **Neden:** GOAL P2-V V3 ve K-P2R.5 (2): R17 Mac arşivi olmayan bir hostta klon dizinini (`…/verify/A/repo/target/p1e1-dist/bin`) yazıyordu; A ↔ B bayt farkı (CI 40265181721, V0 40582139980). Kayıtta host değeri yok ilkesi (GOAL İlkeler).
- **Önce/sonra:** `docs/rounds/P2-V/progress/V3_f1.txt` — boş bir paket dizininde (host arşivi yok) iki sürüm: `.items[] | [id, ok, detail]` farkı yalnız R17'nin `detail`'i; `passed/total` 11/21 ve 11/21. Kademe 2'de iki zincirin `release_check.json`'u eşit ve `eval.portable` temiz (V4).
- **Gözden geçirici onayı:** V6.review.md.

## F2 · V3 · `scripts/verify.sh` — hızlı zincirin `fast/conformance.json`'u fixture kökünü depoya göreli yazar (envanter H4)
- **Ne:** `chain_fast`'ta `cp reports/conformance.json "$fd/conformance.json"` yerine aynı dosyanın `fixtures_root`'u klonun köküne göreli yazılmış kopyası (`jq`, tek satır + yorum). Zincirin adımları, sırası, karşılaştırmalar (`compare_p2r fast`), tam zincir (`chain`, `pair`) ve öteki her satır aynen.
- **Neden:** GOAL P2-V V3 (V1'in bütün H alanları; Blok 2 doğrulaması önerisi 2 (a)): kademe 1'in geliştirme koşusunda `eval.portable --fast` bu alanda koşucunun iş dizinini buldu (`/home/runner/work/_temp/f/A/repo/spec/fixtures`, koşu 40590894752). Dondurulmuş karşılaştırıcı bu alanı zaten `ortam` sayar (`compare_p2l.P2L_INFO`), `fast_regress` okumaz: hiçbir ölçüm değişmez.
- **Önce/sonra:** `docs/rounds/P2-V/progress/V3_f2.txt` — aynı rapordan iki kayıt: 281 yaprağın yalnız `fixtures_root`'u farklı (mutlak → `spec/fixtures`). Hızlı çiftin sonucu: V4/V6'nın FAST koşuları (alan sayısı ve eşitlik).
- **Gözden geçirici onayı:** V6.review.md.

## F3 · V3 · `research/eval/tier2.py` — biçim (`ruff format`)
- **Ne:** bir sözlük ifadesi satırlara bölündü (`V3_f3.diff`); başka değişiklik yok.
- **Neden:** `make lint` (`ruff format --check`) CI'da kırmızıydı (koşu 40613417655); dondurma `P2-V/V3:`'te biçim denetiminden önce yapılmıştı (emsal P2-R/F5).
- **Önce/sonra:** iki sürümün Python AST'si eşit (`ast.dump` aynı); `test_p2v_tools` 11/11 iki sürümde; kademe 2'nin `43055ee`'de başlatılan koşuları önceki sürümle aynı şeyi hesaplar.
- **Gözden geçirici onayı:** V6.review.md.

## Dondurmaya ilk giren dosyalar (istisna değil; yeni satırlar, `P2-V/V3:`)
`research/eval/tier2.py`, `research/eval/portable.py`, `research/eval/verify_mac.py`, `scripts/verify_mac.sh`, `scripts/check_records_portable.sh`, `.github/workflows/verify.yml` — kademe 2'nin karşılaştırıcısı ve iş akışı, host denetleyicisi, Mac kümesi; V4'ün ölçümlerinden önce.
