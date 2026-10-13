# v3 Varsayımları

Her satırın başındaki etiket, varsayımın etkileyebileceği hipotezi gösterir (`[H1] [H3] [H4] [-]`).

## T20
- [-] `.DS_Store` (macOS Finder dosyası) `.gitignore`'a eklendi; izlenmeyen bu dosya `runinfo.worktree_clean()` sonucunu `false` yapıyordu. Ölçüm koduna dokunulmadı.

## T21
- [H4] Varsayılan eşleyici sürümü, sürüm verilmediğinde `$MDK_MATCHER` ortam değişkeninden okunur (yoksa `v3`); hem Python bağlaması hem `mdk` CLI aynı kuralı izler. Gerekçe: dondurulmuş v2 kodu (`eval/freeze_check.py`, `eval/datasets/*.py`) `Vault(...)`'u sürüm vermeden çağırır; bu dosyalar değiştirilemediği için v2 davranışı ancak süreç düzeyinde `MDK_MATCHER=v2` ile korunabilir. `Makefile`'daki v2 hedefleri bu nedenle `MDK_MATCHER=v2` ile koşar.
- [H1] `THETA_V3`/`THETA_SPLIT_V3` T24'e kadar yer tutucu olarak ızgara eşitlik-kırma noktası (0.60, 0.65) ile başlatıldı; T24 ayar fazı bunları yazar.
- [H1] SPEC_DELTA §5.3 adım 5 (bölme yönü artık kontrolü) "değişmez" diyor; tespit yöntemi aynı bırakıldı, ancak V3'te sonuç bölmesine de K4 uygulanır (§5.1: yeni soy yalnız baskın olmayan parçalara). Adım 6 birleştirmelerinde `meta.residual=true`; V3 birleştirme olayları `meta.residual` taşır (V2 olayları değişmedi).
- [H1] Adım 6'da "komşu" yalnız eşleşmemiş (alınmamış) eski bloklardan oluşan ardışık pencere olarak yorumlandı (o dahil 2–3 blok); blok türü koşulu yok; en yüksek `sim` kazanır, eşitlikte ilk bulunan.
- [H1] "Her parça ≥ 1 kelime": normalize metinde en az bir boşluksuz dizgi.
