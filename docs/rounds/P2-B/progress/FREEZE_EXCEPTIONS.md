# P2-B — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, B5'te (K-P2.4).

## F1 · B2 · `research/eval/p2b_align.py` — gizli dizindeki notların ölçüm vault'undaki yolu
- **Ne:** ölçücü her notu ölçüm vault'una `vault_path(rel)` ile yazar: `.` ile başlayan bir yol bileşeni (`.github/`) `_` ile başlar; baytlar aynen; iki not aynı yola düşerse ölçücü durur. Başka satır değişmedi (`B2_f1.diff`: yalnız ekleme).
- **Neden:** ön-kayıtlı ölçücünün (`7a3b295`) ilk koşusu hiçbir değer yazmadan durdu: `rootmark ingest-dir` gizli dizinleri almaz (vault kuralı), seçici ise `microsoft/terminal`'in `.github/actions/spelling/*.md` notlarını seçmişti; `replay` "no snapshot" verdi. Korpus, tohum, eşikler ve ölçüt değişmedi; yalnız ölçüm vault'unda notun yolu.
- **Önce/sonra:** önce: iki koşu da (`20261025T165950Z-7a3b295`, `20261025T170032Z-7a3b295`) ilk gizli yollu notta `SystemExit` ile durdu, `p2b/` boş kaldı (hiçbir ölçüm değeri görülmeden; dizinler silindi). Sonra: aynı korpusla iki koşu, `align.json` bayt eşit (B2.md). Konum ve blok hesabı yoldan bağımsızdır (dilim, satır, `content_hash` dosya baytlarından).
- **Gözden geçirici onayı:** B5.review.md.
