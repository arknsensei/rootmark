# mdkernel-v3 — Nihai Rapor

RUN: 20261013T182004Z-f82dabd · Commit: f82dabdba102 (temiz) · Rust rustc 1.92.0 (ded5c06cf 2025-12-08) · Python 3.12.9 · SEED 20261013
Veri: D2 1500248d8f23 (MANIFEST c8b9283a028efd5bddbea5f29780f2e2) · D3 fbc07769235b (v2 e6945bbc2da9c47376e017e815d7e51b / v3 7335baebefba377c84574928dd4cc05c) · D1 (MANIFEST 7516fda12ac6c0a6d910af9025b31f63) · D1-easy(K4) (MANIFEST 1710f3774c2276cc8545af4460557b80) · D1-hard (MANIFEST 657bb8d3f8df3757f0e8bcc45441fc7e)

> `eval/report_v3.py` tarafından yalnız `reports/runs/20261013T182004Z-f82dabd/` içinden üretildi. Elle sayı yoktur. Her tablo altında kaynak alan yolları. Eşikler `docs/v3/00_GOAL.md`'den aynen kodlanmıştır.

> **K4 tanım değişikliği (sonuç görüldükten sonra alınmış tasarım kararı):** v3'te bölme/birleştirmede baskın parça soyu korur (`docs/ARCHITECTURE.md` §10 K4). Bu karar v2 sonuçları görüldükten sonra alınmıştır ve gerçek tanımını değiştirir; H1-S2/H1-H2 yanlış eşleme paydası buna göre yeniden yazılmıştır (merge sonucu paydada değildir). v2 tanımlı S2 bilgi amaçlı ayrıca raporlanır; v2 davranışının bayrakla birebir korunduğu H4-V2 ile gösterilir.

## 1. Özet
| Kod | Ölçüm | Küme | Eşik | Sonuç | Durum |
|---|---|---|---|---|---|
| H1-S1 | Soy koruma (rephrase_*, move) | D1-easy (K4) | ≥ 0.95 | 1.000000 | GEÇTİ |
| H1-S2 | Yanlış eşleme (K4 tanımı) | D1-easy | ≤ 0.02 | 0.000000 | GEÇTİ |
| H1-S3 | Bölme/birleştirme olay F1 | D1-easy | ≥ 0.80 | 1.000000 | GEÇTİ |
| H1-S4 | Tekrar-blok koruma (dup_delete) | D1-easy | ≥ 0.95 | 1.000000 | GEÇTİ |
| H1-H1 | Soy koruma (rephrase_*, move, split_edit, merge_edit baskın) | D1-hard | ≥ 0.90 | 0.988245 | GEÇTİ |
| H1-H2 | Yanlış eşleme | D1-hard | ≤ 0.03 | 0.000000 | GEÇTİ |
| H1-H3 | Bölme/birleştirme olay F1 | D1-hard | ≥ 0.70 | 0.996721 | GEÇTİ |
| H1-H4 | Tekrar-blok koruma (dup_delete_adjacent) | D1-hard | ≥ 0.90 | 1.000000 | GEÇTİ |
| H1-R1 | Gerçek tarihçe, dokunulmamış blok koruma | D2 | ≥ 0.999 | 0.999843 | GEÇTİ |
| H1-R2 | Gerçek tarihçe, genel koruma | D2 | ≥ 0.85 | 0.981725 | GEÇTİ |
| H1-G | Izgarada farklı amaç değeri sayısı (geçerlilik) | D1-hard ayar | ≥ 3 | 4 | GEÇTİ |
| H3-1 | Yazma→aranabilir p95 (ms) | D3 | < 1000 | 21.421935 | GEÇTİ |
| H3-2 | Yeniden-chunk soy kaybı (kayıp + kazanç) | D3 | = 0 | 0 | GEÇTİ |
| H4-1 | Olay günlüğü hash, D3 yeniden kurulum, eşleyici v3 | D3 | = manifest (v3) | eşit | GEÇTİ |
| H4-V2 | Eşleyici v2 bayrağıyla D3 hash ve v2 h1s.json kalite alanları | D3, D1-v2 | = eşit | eşit | GEÇTİ |
| H4-2 | İki temiz koşturmada kalite metrikleri maks. göreli fark | tümü | = 0 | — | doğrulamada |

Kaynak: H1-S1 ← `h1s_v3.json:H1-S1` · H1-S2 ← `h1s_v3.json:H1-S2` · H1-S3 ← `h1s_v3.json:H1-S3` · H1-S4 ← `h1s_v3.json:H1-S4` · H1-H1 ← `h1s_v3.json:H1-H1` · H1-H2 ← `h1s_v3.json:H1-H2` · H1-H3 ← `h1s_v3.json:H1-H3` · H1-H4 ← `h1s_v3.json:H1-H4` · H1-R1 ← `h1r_v3.json:H1-R1` · H1-R2 ← `h1r_v3.json:H1-R2` · H1-G ← `h1s_v3_tuning.json:H1-G` · H3-1 ← `h3_v3.json:H3-1.p95_ms` · H3-2 ← `h3_v3.json:H3-2.lost+gained` · H4-1 ← `h4_v3.json:H4-1.equal` · H4-V2 ← `h4_v2repro.json:H4-V2.equal` · H4-2 ← `scripts/verify.sh pair (compare_runs.py)`

## 2. Karar kuralı çıktısı
- H1-easy: geçti · H1-hard: geçti (kanıt) · H1-real: geçti · H3: geçti · H4-1: GEÇTİ · H4-V2: GEÇTİ · H4-2 doğrulamada ölçülür.
- Kural H4 koşulu: aşağıdaki çıkarımlar H4-2'nin doğrulamada geçmesine bağlıdır; H4-2 geçmezse determinizm bozulmuştur ve sonuçlar geçersizdir.
- **H1-S* ∧ H1-R* ∧ H1-H* ∧ H3 ∧ H4 geçti → `01_SPEC.md` §1–7 (K4 ve v3 eşleyiciyle) dondurulur; çekirdek fazı 1 başlar.**

## 3. H1-easy — D1-easy (K4 gerçek)
Test bölünmesi: 150 doküman, 1500 puanlı adım · eşleyici v3 (THETA=0.5, THETA_SPLIT=0.65)

| Kod | Değer |
|---|---|
| H1-S1 | 1.0000 |
| H1-S2 | 0.0000 |
| H1-S3 | 1.0000 |
| H1-S4 | 1.0000 |

Olay F1 ayrıntısı: P 1.0000 · R 1.0000 · tp 307 / tahmin 307 / gerçek 307 · payda S1 799, S2 299, S4 300

v2 tanımlı S2 (bilgi): 0.506601 (payda 606)

| Op | Adım | Koruma | Yanlış eşleme | Olay recall |
|---|---|---|---|---|
| delete | 73 | — | — | — |
| dup_delete | 150 | 1.0000 | — | — |
| insert | 171 | — | 0.0000 | — |
| merge | 179 | 1.0000 | — | 1.0000 |
| move | 230 | 1.0000 | — | — |
| rephrase_heavy | 222 | 1.0000 | — | — |
| rephrase_light | 347 | 1.0000 | — | — |
| split | 128 | 1.0000 | 0.0000 | 1.0000 |

Kaynak `h1s_v3.json:easy`

## 4. H1-hard — D1-hard
### 4.1 THETA × THETA_SPLIT ızgarası (ayar, D1-hard ayar bölünmesi)
Ayar: 50 doküman · kural: max S1 + (1-S2) + S3 + S4 (H1-H*); ties -> closest to (0.60, 0.65)

| THETA | THETA_SPLIT | H1 | H2 | H3 | H4 | Amaç |
|---|---|---|---|---|---|---|
| 0.5000 | 0.5500 | 0.9928 | 0.0000 | 0.9932 | 1.0000 | 3.9860 |
| 0.5000 | 0.6500 | 0.9928 | 0.0000 | 0.9932 | 1.0000 | 3.9860 |
| 0.5000 | 0.7500 | 0.9928 | 0.0000 | 0.9932 | 1.0000 | 3.9860 |
| 0.5500 | 0.5500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.5500 | 0.6500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.5500 | 0.7500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.6000 | 0.5500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.6000 | 0.6500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.6000 | 0.7500 | 0.9879 | 0.0000 | 0.9932 | 1.0000 | 3.9811 |
| 0.6500 | 0.5500 | 0.9855 | 0.0000 | 0.9932 | 1.0000 | 3.9787 |
| 0.6500 | 0.6500 | 0.9855 | 0.0000 | 0.9932 | 1.0000 | 3.9787 |
| 0.6500 | 0.7500 | 0.9855 | 0.0000 | 0.9932 | 1.0000 | 3.9787 |
| 0.7000 | 0.5500 | 0.9565 | 0.0000 | 0.9932 | 1.0000 | 3.9497 |
| 0.7000 | 0.6500 | 0.9565 | 0.0000 | 0.9932 | 1.0000 | 3.9497 |
| 0.7000 | 0.7500 | 0.9565 | 0.0000 | 0.9932 | 1.0000 | 3.9497 |

Seçilen: THETA=0.5, THETA_SPLIT=0.65 · H1-G (farklı amaç değeri) 4 · Kaynak `h1s_v3_tuning.json`
### 4.2 Test
Test bölünmesi: 150 doküman, 1500 puanlı adım

| Kod | Değer |
|---|---|
| H1-H1 | 0.9882 |
| H1-H2 | 0.0000 |
| H1-H3 | 0.9967 |
| H1-H4 | 1.0000 |

Olay F1 ayrıntısı: P 1.0000 · R 0.9935 · tp 456 / tahmin 456 / gerçek 459 · payda H1 1191, H2 403, H4 330

v2 tanımlı S2 (bilgi): 0.532483 (payda 862)

| Op | Adım | Koruma | Yanlış eşleme | Olay recall |
|---|---|---|---|---|
| dup_delete_adjacent | 165 | 1.0000 | — | — |
| insert | 91 | — | 0.0000 | — |
| merge_edit | 200 | 1.0000 | — | 0.9850 |
| move_edit | 170 | 1.0000 | — | — |
| rephrase_extreme | 162 | 0.9136 | — | — |
| rephrase_heavy | 224 | 1.0000 | — | — |
| rephrase_light | 229 | 1.0000 | — | — |
| split_edit | 206 | 1.0000 | 0.0000 | 1.0000 |
| split_three | 53 | 1.0000 | 0.0000 | 1.0000 |

Kaynak `h1s_v3.json:hard`

## 5. H1-real — D2 gerçek tarihçe (eşleyici v3)
Depo https://github.com/rust-lang/book@1500248d8f23 · commit 300 · çift 836 · blok 56472 · dokunulmamış 51066

R1 0.999843 · R2 0.981725 · hunk içi (bilgi) 0.810581

Kayıp kategorileri: dup_content 8, hunk_adjacent 0, other 0, type_change 0 · `other` sayısı 0 · Kaynak `h1r_v3.json`

## 6. H3 (eşleyici v3)
p50 16.49 ms · p95 21.42 ms (N=200) · yeniden-chunk kayıp 0/0 · olay hash eşit eşit · hit@1 0.810, hit@10 0.960 (bilgi) · Kaynak `h3_v3.json`

## 7. H4
H4-1 (v3): manifest `events_hash_v3` 7335baebefba377c84574928dd4cc05c vs yeniden kurulum 7335baebefba377c84574928dd4cc05c → eşit · v2 günlüğü (aynı yeniden kurulum) e6945bbc2da9c47376e017e815d7e51b → eşit · Kaynak `h4_v3.json`
### 7.1 H4-V2 — v2 davranışının yeniden üretimi
D3 `--matcher v2` `events_hash`: e6945bbc2da9c47376e017e815d7e51b (beklenen e6945bbc2da9c47376e017e815d7e51b) → eşit

v2 D1, v2 metrikleri, `v2-final:reports/runs/20261013T154851Z-f133d0b/h1s.json` ile alan alan: 35 alan, farklı 0 → eşit · Kaynak `h4_v2repro.json`

H4-2: doğrulama aşamasında ölçülür (`scripts/verify.sh pair`).

## 8. Varsayımlar, park, dondurma istisnaları
**[H1]** (14 madde)
- `THETA_V3`/`THETA_SPLIT_V3` T24'e kadar yer tutucu olarak ızgara eşitlik-kırma noktası (0.60, 0.65) ile başlatıldı; T24 ayar fazı bunları yazar.
- SPEC_DELTA §5.3 adım 5 (bölme yönü artık kontrolü) "değişmez" diyor; tespit yöntemi aynı bırakıldı, ancak V3'te sonuç bölmesine de K4 uygulanır (§5.1: yeni soy yalnız baskın olmayan parçalara). Adım 6 birleştirmelerinde `meta.residual=true`; V3 birleştirme olayları `meta.residual` taşır (V2 olayları değişmedi).
- Adım 6'da "komşu" yalnız eşleşmemiş (alınmamış) eski bloklardan oluşan ardışık pencere olarak yorumlandı (o dahil 2–3 blok); blok türü koşulu yok; en yüksek `sim` kazanır, eşitlikte ilk bulunan.
- "Her parça ≥ 1 kelime": normalize metinde en az bir boşluksuz dizgi.
- D1-hard uygunluk kuralı v2 kuralından gevşek: ≥4 blok, ≥2 paragraf, ≥2 düzyazı bloğu, ≥1 çok cümleli paragraf, render geri dönüşü. Gerekçe: v2 kuralıyla D3'te 285 uygun doküman var, 200'ü D1-easy'de; kalan 85, istenen 200 farklı doküman için yetmiyor. Gevşek kuralla easy dışında 205 aday var. Durumda yapılamayan op'lar v2'deki gibi yeniden çekilir (`redraws`).
- `split_three` cümle **veya** yan cümle sınırından (`,` `;` sonrası) bölünür; EVAL_DELTA cümle sınırını yalnız `split_edit` için şart koşuyor. Yalnız cümle sınırıyla op %1.65'te kalıyordu (±%3 kabulünü ihlal).
- "Parçaların hiçbiri eski metnin tam alt dizisi değil" şartı, `split_edit` tanımı gereği (yalnız **bir** parça düzenlenir) parça başına sağlanamaz; "her bölme/birleştirme örneği tam birleşim değildir" olarak yorumlandı: `sim(eski, concat(parçalar)) < 1.0` (bölme) ve `sim(concat(eskiler), yeni) < 1.0` (birleştirme). `split_three` da bir parçada `rephrase_light` içerir.
- `rephrase_extreme`: %30–45 kelime değişimi; paragraf ≥2 cümleyse 0.5 olasılıkla cümle sırası karıştırılır. `move_edit`: düzyazı blok `rephrase_light` + taşıma. Baskın parça: normalize metnin karakter sayısı (eşleyicinin ölçtüğü), eşitlikte ilk.
- Yeniden çekilişler gerçekleşen oranları kaydırır (split/merge ~%13.5, split_three %3.45); tümü ±%3 içinde.
- (Gözden geçirici notu) Tanım gereği her `split_edit` örneğinde bir parça, `split_three`'te iki parça eski metnin aynen alt dizisidir; `merge_edit` örneklerinin ~%53'ünde (142/269) eski bloklardan biri birleşik metinde aynen durur (`rephrase_light` tek yarıya düşmüş). Ayrıca `split_three` parçalarının 18/207'si ve `split_edit` parçalarının 11/538'i < 3 kelime. T24 sonuçları bu bilgiyle yorumlanmalı.
- H1-H1 op kümesi `00_GOAL.md` tablosundaki listeye birebir uyar (`rephrase_*`, `move_edit`, `split_edit`, `merge_edit` baskın parçalar); `split_three` H1-H1'e girmez ama baskın olmayan parçaları H1-H2 paydasına, olayları H1-H3'e girer.
- Bölme olayının gerçeği K4 ile: sonuç soy kümesi `{eski soy} ∪ {baskın olmayan parçaların (tahmin edilen) soyları}`, dominant = eski soy. Böylece baskın parça yeni soy alırsa veya `meta.dominant` başka bir soyu gösterirse olay yanlış sayılır. v2 olayları (`dominant` yok) hiçbir zaman doğru sayılmaz.
- Ayar ızgarasında THETA_SPLIT (0.55/0.65/0.75) hiçbir metriği değiştirmedi; D1-hard bölme/birleştirme örnekleri bu aralığın hepsinde aynı biçimde yakalanıyor. H1-G yalnız THETA'dan dolayı 4. Seçilen THETA_SPLIT_V3 = 0.65 eşitlik-kırma kuralının sonucudur, veriden öğrenilmiş bir değer değildir.
- `freeze-check-v3` denetleyicisi T24 sabitlerinden bağımsız hale getirildi (FREEZE_EXCEPTIONS v3/T24).
**[H3]** (1 madde)
- H3 v3, v2 gömme indeksini (`data/d3_index`, D3 v2 günlüğüne göre anahtarlı) yeniden kullanır; bu, v3 ikizinin soy/sürüm listelerinin doküman doküman `data/d3` ile aynı olduğu doğrulandıktan sonra yapılır.
**[H4]** (2 madde)
- Varsayılan eşleyici sürümü, sürüm verilmediğinde `$MDK_MATCHER` ortam değişkeninden okunur (yoksa `v3`); hem Python bağlaması hem `mdk` CLI aynı kuralı izler. Gerekçe: dondurulmuş v2 kodu (`eval/freeze_check.py`, `eval/datasets/*.py`) `Vault(...)`'u sürüm vermeden çağırır; bu dosyalar değiştirilemediği için v2 davranışı ancak süreç düzeyinde `MDK_MATCHER=v2` ile korunabilir. `Makefile`'daki v2 hedefleri bu nedenle `MDK_MATCHER=v2` ile koşar.
- D3 (`data/d3`) v2 eşleyiciyle kurulu kalır (v2 manifest'i ve dondurulmuş kurucu); v3 günlüğü aynı dokümanların ayrı bir vault'a eşleyici v3 ile taze ingest'idir. İlk ingest'te önceki snapshot olmadığından iki eşleyici aynı soyları atar; fark yalnız `meta.matcher_version` etiketindedir — H4-1 (v3) bu nedenle esasen kurulumun determinizmini sınar, eşleyici farkını değil (eşleyici farkı H1 ve H3-1 düzenlemelerinde devrededir).
**[-]** (4 madde)
- `.DS_Store` (macOS Finder dosyası) `.gitignore`'a eklendi; izlenmeyen bu dosya `runinfo.worktree_clean()` sonucunu `false` yapıyordu. Ölçüm koduna dokunulmadı.
- `T23: freeze eval v3` commit mesajı, v2'deki `T9: freeze eval` gibi ek satırsız yazıldı ("tam olarak").
- `make eval-h1s-v3` EVAL_DELTA'daki `--phase` argümanını `PHASE=tune|test` değişkeniyle alır (`make` komut satırı bayrak geçiremez); ayrıca `eval-h1s-v3-tune` kısayolu.
- `make eval-h3`/`eval-h4` artık v3 koşturucularıdır (T26); v2 koşturucular `eval-h3-v2`/`eval-h4-v2` olarak `MDK_MATCHER=v2` altında korunur. `make d3` dondurulmuş `eval.datasets.vault`'u doğrudan çağırmaz (o modülün `main()`'i eklenen `events_hash_v3` alanı yüzünden manifest'i tümüyle eşit bulamaz); `eval/d3_v3.py` aynı `build()`'i çağırıp v2 alanlarını ayrı karşılaştırır.

**Park:** Park edilen görev yok.

**Dondurma istisnaları:**

# Dondurma İstisnaları

(Boş. Girdi şartları CLAUDE.md §Ölçüm dürüstlüğü 2.)

Not (T12): `eval/datasets/questions.py` SDK 1.x'teki `temperature` argüman değişikliğine dosya dondurulmadan **önce** uyarlandı (`extra_body`); bu bir dondurma istisnası değildir (ASSUMPTIONS T12).

## v3 / T24 — `eval/freeze_check_v3.py` parametreleri sabitlendi

**Ne:** `freeze-check-v3` denetleyicisi, dondurma dışı ingest'lerde THETA/THETA_SPLIT'i derlenmiş V3 sabitlerinden (`matcher_defaults("v3")`) alıyordu. T24 ayar fazı tasarım gereği `THETA_V3`'ü 0.60 → 0.50 yaptı (commit "T24: V3 constants from D1-hard tuning"); bunun üzerine `make freeze-check-v3` çıktısı değişti (`9e8389668db180218e2e0ce7a254c6d7` ≠ `expected.json`). Düzeltme: denetleyici dondurma anındaki sabitleri (0.60, 0.65) açıkça kullanır.

**Neden / sınıf:** Bu bir bağımlılık API değişikliği **değildir**; T23'te denetleyicinin T24'te değişecek derlenmiş sabitlere bağlı yazılmasından doğan bir tasarım kusurudur. `eval/freeze_check_v3.py` CLAUDE.md v3 dondurma listesinde yer almıyor (T23.md'de gönüllü olarak donmuş ilan edildi); dondurulmuş `eval/metrics_v3.py`, `eval/datasets/synthetic_edits_v2.py`, `tests/fixtures/freeze_fixture_v3/**` (`expected.json` dahil) **değişmedi**. Şeffaflık için istisna prosedürü tam uygulanır.

**Diff:**
```diff
diff --git a/eval/freeze_check_v3.py b/eval/freeze_check_v3.py
index 8870476..a1f61f1 100644
--- a/eval/freeze_check_v3.py
+++ b/eval/freeze_check_v3.py
@@ -36,6 +36,10 @@ EDIT_SOURCES = ("roasting.md", "maintenance.md", "espresso.md", "cupping.md", "s
 EDIT_STEPS = 3
 MAX_EDIT_RUNS = 80
 GRID = ((0.55, 0.55), (0.60, 0.65), (0.70, 0.75))
+# THETA / THETA_SPLIT of every non-grid ingest: the compiled V3 constants at
+# freeze time (T23), pinned so that tuning (T24 writes THETA_V3/THETA_SPLIT_V3)
+# cannot change this output (FREEZE_EXCEPTIONS.md, T24)
+PARAMS = (0.60, 0.65)
 
 
 def _rec_lists(rec: dict) -> dict:
@@ -63,7 +67,7 @@ def _dominant_index(step: dict) -> int | None:
     return i
 
 
-def _records(truth: dict, out_dir: Path, vault_dir: Path, params=(None, None)) -> list[dict]:
+def _records(truth: dict, out_dir: Path, vault_dir: Path, params=PARAMS) -> list[dict]:
     v = mdkernel_core.Vault(str(vault_dir), params[0], params[1], matcher=MATCHER)
     doc = truth["doc_path"]
     v.ingest(doc, (out_dir / truth["base_file"]).read_text(encoding="utf-8"), "Synthetic", "d1")
@@ -165,7 +169,7 @@ def check_h1r(tmp: Path) -> dict:
     old = (FIX / pair["old"]).read_bytes().decode("utf-8")
     new = (FIX / pair["new"]).read_bytes().decode("utf-8")
     drafts, _ = mdkernel_core.parse_document(old, pair["path"])
-    v = mdkernel_core.Vault(str(tmp / "h1r"), matcher=MATCHER)
+    v = mdkernel_core.Vault(str(tmp / "h1r"), PARAMS[0], PARAMS[1], matcher=MATCHER)
     v.ingest(pair["path"], old)
     old_blocks = v.snapshot_blocks(pair["path"])
     for b, d in zip(old_blocks, drafts, strict=True):
@@ -263,7 +267,9 @@ def run() -> dict:
         tmp = Path(t)
         return {
             "matcher": MATCHER,
-            "matcher_defaults": list(mdkernel_core.matcher_defaults(MATCHER)),
+            # key kept for bit-identity with expected.json: the pinned PARAMS
+            # (equal to the compiled V3 defaults when the fixture was frozen)
+            "matcher_defaults": list(PARAMS),
             "easy": check_easy(tmp),
             "hard": check_hard(tmp),
             "h1r": check_h1r(tmp),
```

**Bit düzeyinde eşitlik kaydı (`tests/fixtures/freeze_fixture_v3/`):**
- Önce (dondurma anı, commit aa3a2e2, V3 sabitleri 0.60/0.65): `freeze-check-v3 OK (d138cddcde1d0978a020a6fe8df2f610)` (T23.md, T23.review.md).
- Eski denetleyici + T24 sabitleri: `freeze-check-v3 FAILED: output 9e8389668db180218e2e0ce7a254c6d7`.
- Sonra (yeni denetleyici + T24 sabitleri): `freeze-check-v3 OK (d138cddcde1d0978a020a6fe8df2f610)` — `expected.json` ile bayt bayt aynı; `MDK_MATCHER=v2` altında da aynı.
- `make freeze-check` (v2): `d489a0ed44b15d3aeb272e16cdd4a5af` (değişmedi).

**Gözden geçirici onayı:** ONAYLANDI — `docs/progress/v3/T24.review.md` ("FREEZE_EXCEPTIONS v3/T24: ONAYLANDI"; eski denetleyici `9e838966…` FAILED, yeni denetleyici `d138cddc…` OK bağımsız olarak yeniden üretildi).

## 9. Serbest yorum (ajan; sayı içermez)
Bu turun amacı soy eşleyicisindeki tek açık kalemi kapatmaktı: önceki turda yanlış eşleme ölçütü birleştirme işlemlerinde eşiği aşıyordu. mimari belgedeki baskın parça kararıyla (bölme ve birleştirmede baskın parça soyu korur) birlikte eşleyici bölmede en uzun yeni parçaya, birleştirmede en uzun eski bloğa soyu devrediyor; asgari parça oranı kaldırıldı ve birleştirme yönünde artık kontrolü eklendi.

Dikkat edilmesi gereken nokta, yanlış eşleme ölçütünün tanımının bu kararla değişmiş olmasıdır. Birleştirme sonucu artık paydada değildir; bu nedenle önceki turla doğrudan karşılaştırma yalnız bilgi amaçlıdır. Eski tanımla hesaplanan değer raporda ayrıca verilmiştir ve beklendiği gibi yüksektir, çünkü yeni semantikte baskın parçanın var olan bir soya bağlanması doğru davranıştır.

Zor sentetik set, ayar ızgarasının bilgi taşıması için tasarlandı. Izgara bu kez düz değil, ancak farklılık yalnız benzerlik eşiğinden geliyor; bölme eşiği denenen aralığın tamamında sonucu değiştirmedi, dolayısıyla seçilen bölme eşiği veriden değil eşitlik kırma kuralından geliyor. Zor sette en zayıf işlem, kelimelerin büyük kısmının değiştiği ve cümle sırasının karıştırıldığı aşırı yeniden yazımdır; bu, benzerlik tabanlı kimlik çıkarımının doğal sınırına işaret ediyor.

Zor set için uygun doküman havuzu dar kaldı: önceki turun uygunluk kuralıyla kolay sete ayrılan dokümanların dışında yeterli aday yoktu, bu yüzden daha gevşek bir kural kullanıldı ve havuzun neredeyse tamamı seçildi. Üç parçaya bölme işlemi de cümle sınırlarına ek olarak yan cümle sınırlarından bölünebiliyor. Bu varsayımlar kayıtlıdır ve sonuçlar bu çerçevede okunmalıdır.

Önceki turun davranışı bayrakla korunuyor: eski eşleyici yolu, değerlendirme vault'unun olay günlüğünü ve önceki turun sentetik test sonuçlarını alan alan birebir yeniden üretiyor. Ayar fazından sonra dondurma denetleyicisinin derlenmiş sabitlere bağlı olduğu ortaya çıktı; denetleyici dondurma anındaki parametrelere sabitlendi ve bu, istisna kaydında gözden geçirici onayıyla belgelendi. Dondurulmuş metrik kodu, üreteçler ve beklenen çıktı değişmedi.

İki temiz koşturma arasındaki determinizm ölçümü bağımsız doğrulamaya bırakılmıştır; karar kuralının dondurma çıkarımı ona koşulludur.
