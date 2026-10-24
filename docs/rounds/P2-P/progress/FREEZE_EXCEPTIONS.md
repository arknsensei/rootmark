# P2-P — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, P5'te (K-P2.4).

## F1 · P0 (a3) · `scripts/release_check.sh` — R16'nın geçici dosyaları
- **Ne:** R16'nın iki geçici dosyası (`${TMPDIR:-/tmp}/rootmark-r16.$$`, `…/rootmark-r16k.$$`) betiğin başta `mktemp -d -t rootmark-relcheck.XXXXXX` ile kurduğu ve `trap` ile sildiği `$tmp` dizinine taşındı (`$tmp/r16`, `$tmp/r16k`). Başka satır değişmedi.
- **Neden:** GOAL P2-P 1 (a3); P2-L gözden geçirici bulgusu (c) (`L5.review.md`): öngörülebilir ad (`$$`) paylaşılan geçici dizinde önceden kurulmuş bir dosya ya da bağla yarışa açıktı.
- **Önce/sonra:** `git diff main -- scripts/release_check.sh`: yalnız R16 bloğunun beş satırı, yalnız yol dizgileri değişti (`grep`/`cat`/`rm` komutları ve okunan içerik aynı); `bash -n` temiz. R16'nın kararı dosyanın adından bağımsızdır (yalnız içeriği okunur).
- **Gözden geçirici onayı:** P5.review.md.

## F2 · P5 · `scripts/verify.sh` — P2-P adımı, sınıflandırıcı, çift karşılaştırıcısı
- **Ne:** tam zincire `make p2l-quality`'den sonra bir adım (`make p2p-checks` → `p2p_checks.json`: P-1'in üç vault'u, P-2..P-5 ve S1–S7, P-7, toplayıcı sahnesi P-3/P-6); FAIL özeti `eval.historical_p2l` yerine `eval.historical_p2p` (P2-L'nin resmi RUN'ına karşı); `pair` `compare_p2l` yerine yeni `compare_p2p` işlevi (`compare_p2p pair`: `compare_p2l pair` + `p2p_checks.json`); hızlı çiftin karşılaştırıcısı `compare_p2l fast` yerine `compare_p2p fast` (aynı dosyalar + `p2p_checks.json`); `compare-p2p` alt komutu; başlık yorumu. `compare_p2l` işlevi ve `compare-p2l` alt komutu korunur.
- **Neden:** TASKS P5 ("`verify.sh`: P2-P adımı (P-1 üç vault, P-5 S6, P-6 sahne; dondurma istisnası kaydı önce/sonra)"); emsal P2-L/F6, P2-A/F5.
- **Önce/sonra:** `P5_f2_removed.out`: P2-L sürümünden silinen satırlar yalnız sınıflandırıcının iki satırı, hızlı çiftin karşılaştırıcı çağrısının üç satırı (günlük, çağrı, hata iletisi), `pair`'in karşılaştırıcı çağrısı ve kullanım satırı; öteki adımlar bayt bayt aynı (yalnız ekleme). Hızlı zincirin adımları değişmedi (aktif turun kontrolleri Makefile'ın `ROUND_CHECKS`'i: `p2l-checks p2l-quality p2p-checks`).
- **Gözden geçirici onayı:** P5.review.md.

## F3 · P5 · `research/eval/check_compare_fields.py`, `research/eval/DETERMINISTIC_FIELDS.md` — `compare_p2p`
- **Ne:** alan denetimi yeni karşılaştırıcıyı tanır (`("compare_p2p", "P2P")`, dosya sabitleri `P2P_FAST_FILES`, `P2P_FILES`); `DETERMINISTIC_FIELDS.md`'ye `compare_p2p` bölümü, alan satırı (`fast/conformance.json` `fixtures_root`, ortam) ve iki dosya satırı. Bu commit'te var olan yeni P2-P ölçüm dosyaları (`run_p2p_checks.py`, `compare_p2p.py`, `historical_p2p.py` ve iki listesi — P2-L'ninkilerin kopyası, referans P2-L'nin resmi RUN'ı) `research/eval/**` satırıyla donar.
- **Neden:** emsal P2-L/F4, F7; çiftin karşılaştırıcısı belirlenimli alan sözleşmesine girer (K-E0.2).
- **Önce/sonra:** `check_compare_fields.py`'den silinen satır 0, eklenen iki satır; `DETERMINISTIC_FIELDS.md`'den silinen satır 0; `python -m eval.check_compare_fields` geçer (`unmatched: 0`).
- **Gözden geçirici onayı:** P5.review.md.
