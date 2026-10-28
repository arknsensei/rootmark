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

## F4 · V4 · `scripts/verify_mac.sh` — kümelerden biri kalınca da RUN satırları ve çalışma ağacı denetimi
- **Ne:** `eval.verify_mac summary`'nin çıkış 3'ü (bir küme kaldı: kayıtlı sonuç) `set -e` ile betiği durduruyordu; artık yakalanır, betik çalışma ağacı denetimini yapar, `RUN=`/`RUN_DIR=` satırlarını yazar ve aynı kodla (3) çıkar; başka bir çıkış hata olarak kalır (`V4_f4.diff`). Kümeler, sıraları, komutlar, değerlendirme aynen.
- **Neden:** V4'ün Mac kümesi (`20261028T013053Z-43055ee`) bütün tekrarları kaydetti ve `mac_checks.json`'u yazdı, ama çıkış 3'te `RUN_DIR` satırı ve kirli ağaç denetimi koşmadı; ölçüm etkilenmedi, kaydın yeri betiğin çıktısında yoktu. Aynı klonda denetim elle yapıldı: `git status --porcelain --untracked-files=no` boş.
- **Önce/sonra:** `V4_f4.diff` (yalnız çıkış yolu); `bash -n` temiz; ölçülen RUN önceki sürümle, sonuçlar aynı kalır (özet adımından sonrası).
- **Gözden geçirici onayı:** V6.review.md.

## F5, F6, F7 · V4 · `research/eval/tier2.py`, `research/eval/portable.py`, `.github/workflows/verify.yml` — karşılaştırma bir kez hesaplanır
- **Ne:** F5 `tier2 pair --frozen <compare_p2r pair --out sonucu>`: dondurulmuş karşılaştırıcının bu iki RUN'ın (çözülmüş yolları birebir aynı olmalı, yoksa hata) sonucunu okur, yeniden karşılaştırmaz; adlandırma kuralları, gömülü kural, hüküm aynen. F6 `portable --files-from <aynı sonuç>`: dosya listesini (dosyalar ve bayt artefaktları) ondan okur; aranan değerler, desenler, hüküm aynen. F7 `verify.yml`'ın karşılaştırma işi: `compare_p2r pair` bir kez, `tier2 pair` ve `portable` onun sonucuyla; zaman aşımı 30 → 120 dk; `chains_run` girdisi: verilen koşunun iki zincirinin yapıtlarını yeniden karşılaştırır (zincir koşmaz; kaydı `chains_run`, `chains_wall_s`, `compare_s`); bir zincirin `exit`'i 0 değilse kırmızı. Zincir işleri aynen (`V4_f5.diff`, `V4_f6.diff`, `V4_f7.diff`).
- **Neden:** V4'ün üç tetiklemesinde (40613392932, 40613401174, 40613409396) altı zincirin hepsi başarıyla bitti ve yapıtlarını yükledi, ama karşılaştırma işi 30 dk zaman aşımında iptal oldu: işin `compare_p2r.pair`'i dört kez koşturuyordu (dondurulmuş hüküm, `tier2 pair`, iki `portable`'ın dosya listesi), her biri ~790 000 alanlık iki RUN üzerinde (yerelde de 10 dk'dan uzun). Ölçülen şey değişmez: aynı fonksiyonun aynı girdideki sonucu bir kez hesaplanıp üç yerde okunur.
- **Önce/sonra:** `test_p2v_tools` (yeni: `test_a_frozen_result_is_read_only_for_the_runs_it_compared` — sonucun okunması, başka RUN'ların sonucunun reddi); `chains_run` ile yeniden karşılaştırmanın sonucu, aynı yapıtlar üzerinde yerelde `tier2 pair` (dondurulmuş karşılaştırmayı yeniden hesaplayarak) ile aynı (V4.md).
- **Gözden geçirici onayı:** V6.review.md.

## Dondurmaya ilk giren dosyalar (istisna değil; yeni satırlar, `P2-V/V3:`)
`research/eval/tier2.py`, `research/eval/portable.py`, `research/eval/verify_mac.py`, `scripts/verify_mac.sh`, `scripts/check_records_portable.sh`, `.github/workflows/verify.yml` — kademe 2'nin karşılaştırıcısı ve iş akışı, host denetleyicisi, Mac kümesi; V4'ün ölçümlerinden önce.
