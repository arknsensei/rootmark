# P2-P — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, P5'te (K-P2.4).

## F1 · P0 (a3) · `scripts/release_check.sh` — R16'nın geçici dosyaları
- **Ne:** R16'nın iki geçici dosyası (`${TMPDIR:-/tmp}/rootmark-r16.$$`, `…/rootmark-r16k.$$`) betiğin başta `mktemp -d -t rootmark-relcheck.XXXXXX` ile kurduğu ve `trap` ile sildiği `$tmp` dizinine taşındı (`$tmp/r16`, `$tmp/r16k`). Başka satır değişmedi.
- **Neden:** GOAL P2-P 1 (a3); P2-L gözden geçirici bulgusu (c) (`L5.review.md`): öngörülebilir ad (`$$`) paylaşılan geçici dizinde önceden kurulmuş bir dosya ya da bağla yarışa açıktı.
- **Önce/sonra:** `git diff main -- scripts/release_check.sh`: yalnız R16 bloğunun beş satırı, yalnız yol dizgileri değişti (`grep`/`cat`/`rm` komutları ve okunan içerik aynı); `bash -n` temiz. R16'nın kararı dosyanın adından bağımsızdır (yalnız içeriği okunur).
- **Gözden geçirici onayı:** P5.review.md.
