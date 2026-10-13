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

## T22
- [H1] D1-hard uygunluk kuralı v2 kuralından gevşek: ≥4 blok, ≥2 paragraf, ≥2 düzyazı bloğu, ≥1 çok cümleli paragraf, render geri dönüşü. Gerekçe: v2 kuralıyla D3'te 285 uygun doküman var, 200'ü D1-easy'de; kalan 85, istenen 200 farklı doküman için yetmiyor. Gevşek kuralla easy dışında 205 aday var. Durumda yapılamayan op'lar v2'deki gibi yeniden çekilir (`redraws`).
- [H1] `split_three` cümle **veya** yan cümle sınırından (`,` `;` sonrası) bölünür; EVAL_DELTA cümle sınırını yalnız `split_edit` için şart koşuyor. Yalnız cümle sınırıyla op %1.65'te kalıyordu (±%3 kabulünü ihlal).
- [H1] "Parçaların hiçbiri eski metnin tam alt dizisi değil" şartı, `split_edit` tanımı gereği (yalnız **bir** parça düzenlenir) parça başına sağlanamaz; "her bölme/birleştirme örneği tam birleşim değildir" olarak yorumlandı: `sim(eski, concat(parçalar)) < 1.0` (bölme) ve `sim(concat(eskiler), yeni) < 1.0` (birleştirme). `split_three` da bir parçada `rephrase_light` içerir.
- [H1] `rephrase_extreme`: %30–45 kelime değişimi; paragraf ≥2 cümleyse 0.5 olasılıkla cümle sırası karıştırılır. `move_edit`: düzyazı blok `rephrase_light` + taşıma. Baskın parça: normalize metnin karakter sayısı (eşleyicinin ölçtüğü), eşitlikte ilk.
- [H1] Yeniden çekilişler gerçekleşen oranları kaydırır (split/merge ~%13.5, split_three %3.45); tümü ±%3 içinde.
- [H1] (Gözden geçirici notu) Tanım gereği her `split_edit` örneğinde bir parça, `split_three`'te iki parça eski metnin aynen alt dizisidir; `merge_edit` örneklerinin ~%53'ünde (142/269) eski bloklardan biri birleşik metinde aynen durur (`rephrase_light` tek yarıya düşmüş). Ayrıca `split_three` parçalarının 18/207'si ve `split_edit` parçalarının 11/538'i < 3 kelime. T24 sonuçları bu bilgiyle yorumlanmalı.

## T23
- [H1] H1-H1 op kümesi `00_GOAL.md` tablosundaki listeye birebir uyar (`rephrase_*`, `move_edit`, `split_edit`, `merge_edit` baskın parçalar); `split_three` H1-H1'e girmez ama baskın olmayan parçaları H1-H2 paydasına, olayları H1-H3'e girer.
- [H1] Bölme olayının gerçeği K4 ile: sonuç soy kümesi `{eski soy} ∪ {baskın olmayan parçaların (tahmin edilen) soyları}`, dominant = eski soy. Böylece baskın parça yeni soy alırsa veya `meta.dominant` başka bir soyu gösterirse olay yanlış sayılır. v2 olayları (`dominant` yok) hiçbir zaman doğru sayılmaz.
- [-] `T23: freeze eval v3` commit mesajı, v2'deki `T9: freeze eval` gibi ek satırsız yazıldı ("tam olarak").

## T24
- [H1] Ayar ızgarasında THETA_SPLIT (0.55/0.65/0.75) hiçbir metriği değiştirmedi; D1-hard bölme/birleştirme örnekleri bu aralığın hepsinde aynı biçimde yakalanıyor. H1-G yalnız THETA'dan dolayı 4. Seçilen THETA_SPLIT_V3 = 0.65 eşitlik-kırma kuralının sonucudur, veriden öğrenilmiş bir değer değildir.
- [-] `make eval-h1s-v3` EVAL_DELTA'daki `--phase` argümanını `PHASE=tune|test` değişkeniyle alır (`make` komut satırı bayrak geçiremez); ayrıca `eval-h1s-v3-tune` kısayolu.
- [H1] `freeze-check-v3` denetleyicisi T24 sabitlerinden bağımsız hale getirildi (FREEZE_EXCEPTIONS v3/T24).

## T26
- [H4] D3 (`data/d3`) v2 eşleyiciyle kurulu kalır (v2 manifest'i ve dondurulmuş kurucu); v3 günlüğü aynı dokümanların ayrı bir vault'a eşleyici v3 ile taze ingest'idir. İlk ingest'te önceki snapshot olmadığından iki eşleyici aynı soyları atar; fark yalnız `meta.matcher_version` etiketindedir — H4-1 (v3) bu nedenle esasen kurulumun determinizmini sınar, eşleyici farkını değil (eşleyici farkı H1 ve H3-1 düzenlemelerinde devrededir).
- [H3] H3 v3, v2 gömme indeksini (`data/d3_index`, D3 v2 günlüğüne göre anahtarlı) yeniden kullanır; bu, v3 ikizinin soy/sürüm listelerinin doküman doküman `data/d3` ile aynı olduğu doğrulandıktan sonra yapılır.
- [-] `make eval-h3`/`eval-h4` artık v3 koşturucularıdır (T26); v2 koşturucular `eval-h3-v2`/`eval-h4-v2` olarak `MDK_MATCHER=v2` altında korunur. `make d3` dondurulmuş `eval.datasets.vault`'u doğrudan çağırmaz (o modülün `main()`'i eklenen `events_hash_v3` alanı yüzünden manifest'i tümüyle eşit bulamaz); `eval/d3_v3.py` aynı `build()`'i çağırıp v2 alanlarını ayrı karşılaştırır.
