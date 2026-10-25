# P2-B — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, B5'te (K-P2.4).

## F1 · B2 · `research/eval/p2b_align.py` — gizli dizindeki notların ölçüm vault'undaki yolu
- **Ne:** ölçücü her notu ölçüm vault'una `vault_path(rel)` ile yazar: `.` ile başlayan bir yol bileşeni (`.github/`) `_` ile başlar; baytlar aynen; iki not aynı yola düşerse ölçücü durur. Başka satır değişmedi (`B2_f1.diff`: yalnız ekleme).
- **Neden:** ön-kayıtlı ölçücünün (`7a3b295`) ilk koşusu hiçbir değer yazmadan durdu: `rootmark ingest-dir` gizli dizinleri almaz (vault kuralı), seçici ise `microsoft/terminal`'in `.github/actions/spelling/*.md` notlarını seçmişti; `replay` "no snapshot" verdi. Korpus, tohum, eşikler ve ölçüt değişmedi; yalnız ölçüm vault'unda notun yolu.
- **Önce/sonra:** önce: iki koşu da (`20261025T165950Z-7a3b295`, `20261025T170032Z-7a3b295`) ilk gizli yollu notta `SystemExit` ile durdu, `p2b/` boş kaldı (hiçbir ölçüm değeri görülmeden; dizinler silindi). Sonra: aynı korpusla iki koşu, `align.json` bayt eşit (B2.md). Konum ve blok hesabı yoldan bağımsızdır (dilim, satır, `content_hash` dosya baytlarından).
- **Gözden geçirici onayı:** B5.review.md.

## F2 · B5 · `scripts/verify.sh` — P2-B adımı, sınıflandırıcı, çift karşılaştırıcısı
- **Ne:** tam zincire `make p2p-checks`'ten sonra bir adım (`make p2b-checks` → `p2b_checks.json`: B-1, B-2 (D1-3'ün dondurulmuş kaydedicisi dahil), B-5/B-6 (P2-L'nin L-7 testleri dahil); `p2b/align.json` (B-3/B-4; kalan ölçüt kaydedilir, çıkış 3 zinciri durdurmaz, K-P2B-T1); `p2b/det.json` (B-9); `p2b/client_checks.json`, `p2b/obsidian_e2e.json`, `p2b/vscode_e2e.json` (B-7, B-8)); FAIL özeti `eval.historical_p2p` yerine `eval.historical_p2b` (P2-P'nin resmi RUN'ına karşı); `pair` ve hızlı çift `compare_p2p` yerine yeni `compare_p2b` (`compare_p2p` + P2-B dosyaları), `compare-p2b` alt komutu; `compare-p2p` korunur. Hızlı zincirin adımları değişmedi (aktif turun kontrolleri Makefile'ın `ROUND_CHECKS`'i: `p2b-checks` eklendi; Makefile dondurulmuş değil).
- **Neden:** TASKS B5 ("`verify.sh`: P2-B adımı (B-2 `events_hash`/C3-06, B-3/B-4 hizalama, B-5/B-6, B-9 determinizm). Dondurma istisnası kaydıyla, önce/sonra"); emsal P2-P/F2, P2-L/F6.
- **Önce/sonra:** `B5_f2_removed.out`: P2-P sürümünden silinen satırlar yalnız sınıflandırıcının iki satırı, hızlı çiftin karşılaştırıcı çağrısının üç satırı (günlük, çağrı, hata iletisi), `pair`'in karşılaştırıcı çağrısı ve kullanım satırı; öteki adımlar bayt bayt aynı (yalnız ekleme: başlık yorumu, `make p2b-checks` adımı, `compare_p2b` işlevi, `compare-p2b` alt komutu). `bash -n` temiz.
- **Gözden geçirici onayı:** B5.review.md.

## F3 · B5 · `research/eval/check_compare_fields.py`, `research/eval/DETERMINISTIC_FIELDS.md` — `compare_p2b`; yeni P2-B ölçüm dosyaları
- **Ne:** alan denetimi yeni karşılaştırıcıyı tanır (`("compare_p2b", "P2B")`, dosya sabitleri `P2B_FAST_FILES`, `P2B_FILES`); `DETERMINISTIC_FIELDS.md`'ye `compare_p2b` bölümü, alan satırı (`fast/conformance.json` `fixtures_root`, ortam) ve iki dosya satırı. Bu commit'te var olan yeni P2-B ölçüm dosyaları (`compare_p2b.py`, `historical_p2b.py` ve iki listesi — P2-P'ninkilerin aynısı, referansı P2-P'nin resmi RUN'ı —, `run_p2b_checks.py` (P2-P'ninkinin kopyası, çıktı adı), `run_p2b_det.py`, `report_p2b_align.py`) `research/eval/**` satırıyla yeniden dondurulur.
- **Neden:** emsal P2-P/F3, P2-L/F4, F7; çiftin karşılaştırıcısı belirlenimli alan sözleşmesine girer (K-E0.2).
- **Önce/sonra:** `check_compare_fields.py`'den silinen satır 0, eklenen iki satır; `DETERMINISTIC_FIELDS.md`'den silinen satır 0; `python -m eval.check_compare_fields` geçer (`unmatched: 0`).
- **Gözden geçirici onayı:** B5.review.md.
