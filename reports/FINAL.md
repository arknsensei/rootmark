# mdkernel-v2 — Nihai Rapor

RUN: 20261013T154851Z-f133d0b · Commit: f133d0b80d41 (temiz) · Rust rustc 1.92.0 (ded5c06cf 2025-12-08) · Python 3.12.9 · SEED 20261013
Veri: D2 1500248d8f23 (MANIFEST c8b9283a028efd5bddbea5f29780f2e2) · D3 fbc07769235b (events e6945bbc2da9c47376e017e815d7e51b) · D1 (MANIFEST 7516fda12ac6c0a6d910af9025b31f63) · D4 single 173 / multi 124 (test)

> `eval/report.py` tarafından yalnız `reports/runs/20261013T154851Z-f133d0b/` içinden üretildi. Elle sayı yoktur. Her tablo altında kaynak alan yolları.

## 1. Özet
| Kod | Ölçüm | Eşik | Sonuç | Durum |
|---|---|---|---|---|
| H1-S1 | Sentetik soy koruma (rephrase_*, move) | ≥ 0.95 | 1.000000 | GEÇTİ |
| H1-S2 | Sentetik yanlış eşleme | ≤ 0.02 | 0.037954 | KALDI |
| H1-S3 | Bölme/birleştirme F1 | ≥ 0.80 | 0.961083 | GEÇTİ |
| H1-S4 | Tekrar-blok soy koruma (dup_delete) | ≥ 0.95 | 1.000000 | GEÇTİ |
| H1-R1 | Gerçek tarihçe, dokunulmamış blok koruma | ≥ 0.999 | 0.999843 | GEÇTİ |
| H1-R2 | Gerçek tarihçe, genel koruma | ≥ 0.85 | 0.978662 | GEÇTİ |
| H2-1 | graph_hybrid − hybrid, multi ΔnDCG@10 | ≥ +0.03 | 0.008964 | KALDI |
| H2-2 | graph_hybrid − hybrid, single ΔnDCG@10 | ≥ −0.02 | -0.018821 | GEÇTİ |
| H2-3 | graph_hybrid p95 / hybrid p95 gecikme | ≤ 3.0 | 1.775889 | GEÇTİ |
| H2-4 | graph_hybrid − hybrid, multi Δfull_cover@10 | ≥ +0.03 | 0.024194 | KALDI |
| H2-5 | graph_flat − flat, multi ΔnDCG@10 | ≥ +0.05 | 0.028967 | KALDI |
| H3-1 | Yazma→aranabilir p95 (ms) | < 1000 | 21.108710 | GEÇTİ |
| H3-2 | Yeniden-chunk soy kaybı (kayıp + kazanç) | = 0 | 0 | GEÇTİ |
| H4-1 | Olay günlüğü hash eşitliği (D3 yeniden kurulum) | = eşit | eşit | GEÇTİ |
| H4-2 | İki temiz koşturmada kalite metrikleri maks. göreli fark | = 0 | — | doğrulamada |

Kaynak: H1-S1 ← `h1s.json:H1-S1` · H1-S2 ← `h1s.json:H1-S2` · H1-S3 ← `h1s.json:H1-S3` · H1-S4 ← `h1s.json:H1-S4` · H1-R1 ← `h1r.json:H1-R1` · H1-R2 ← `h1r.json:H1-R2` · H2-1 ← `h2.json:H2-1` · H2-2 ← `h2.json:H2-2` · H2-3 ← `h2.json:H2-3` · H2-4 ← `h2.json:H2-4` · H2-5 ← `h2.json:H2-5` · H3-1 ← `h3.json:H3-1.p95_ms` · H3-2 ← `h3.json:H3-2.lost+gained` · H4-1 ← `h4.json:H4-1.equal` · H4-2 ← `VERIFICATION.md (compare_runs.py)`

## 2. Karar kuralı çıktısı
- H1: geçmedi · H2: geçmedi · H3: geçti · H4: H4-1 geçti, H4-2 doğrulamada ölçülür.
- Kural H4 koşulu: aşağıdaki çıkarımlar H4-2'nin doğrulamada (VERIFICATION.md) geçmesine bağlıdır; H4-2 geçmezse başka hiçbir sonuç geçerli sayılmaz.
- **H1 geçmedi → veri modeli/eşleyici yeniden ele alınır; H2/H3 saklanır.**

## 3. H1
### 3.1 THETA × THETA_SPLIT ızgarası (ayar)
| THETA | THETA_SPLIT | S1 | S2 | S3 | S4 | Amaç |
|---|---|---|---|---|---|---|
| 0.5000 | 0.6000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.5000 | 0.7000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.5000 | 0.8000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.5500 | 0.6000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.5500 | 0.7000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.5500 | 0.8000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6000 | 0.6000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6000 | 0.7000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6000 | 0.8000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6500 | 0.6000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6500 | 0.7000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.6500 | 0.8000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.7000 | 0.6000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.7000 | 0.7000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |
| 0.7000 | 0.8000 | 0.9963 | 0.0337 | 0.9668 | 1.0000 | 3.9295 |

Kaynak `h1s_tuning.json:grid`.

Izgara düz (`h1s_tuning.json:grid_flat`). Izgara dışı tanı noktası (seçilemez) THETA=0.95, THETA_SPLIT=0.99: S1 0.4630, S2 0.0144, S3 0.9619, S4 1.0000 (`h1s_tuning.json:sensitivity_probe`).

Seçilen: THETA=0.60, THETA_SPLIT=0.70 · Şekil `figs/h1s_grid.png`
### 3.2 Test, op bazında
| Op | Adım | Koruma | Yanlış eşleme | Olay recall |
|---|---|---|---|---|
| delete | 73 | — | — | — |
| dup_delete | 150 | 1.0000 | — | — |
| insert | 171 | — | 0.0000 | — |
| merge | 179 | — | 0.1229 | 0.8771 |
| move | 230 | 1.0000 | — | — |
| rephrase_heavy | 222 | 1.0000 | — | — |
| rephrase_light | 347 | 1.0000 | — | — |
| split | 128 | — | 0.0039 | 0.9922 |

S3 ayrıntı: P 1.0000, R 0.9251, doğru 284 / tahmin 284 / gerçek 307. n: S1 799, S2 606, S4 300; 150 doküman, 1500 adım. Kaynak `h1s.json:per_op`, `h1s.json:H1-S3_detail`.
### 3.3 Gerçek tarihçe
Depo https://github.com/rust-lang/book@1500248d8f23 · commit 300 · çift 836 · blok 56472 · dokunulmamış 51066

R1 0.999843 · R2 0.978662 · hunk içi (bilgi) 0.778579

Kayıp kategorileri: dup_content 8, hunk_adjacent 0, other 0, type_change 0 · `other` örnekleri: yok (Kaynak `h1r.json`)

## 4. H2
ALPHA seçilen 0.9 (kural: max mean nDCG@10 over all tuning questions; ties -> closest to 0.6, then smaller) · Duyarlılık eğrisi (ayar, 50 soru):

| ALPHA | nDCG tümü | single | multi |
|---|---|---|---|
| 0.3000 | 0.5193 | 0.5614 | 0.4771 |
| 0.4000 | 0.5279 | 0.5905 | 0.4653 |
| 0.5000 | 0.5280 | 0.5943 | 0.4618 |
| 0.6000 | 0.5379 | 0.6090 | 0.4667 |
| 0.7000 | 0.5502 | 0.6438 | 0.4567 |
| 0.8000 | 0.5646 | 0.6738 | 0.4555 |
| 0.9000 | 0.5710 | 0.6814 | 0.4607 |

`figs/h2_alpha.png` · Kaynak `h2_tuning.json:curve`
### 4.1 nDCG@10
| Strateji | structural single | structural multi | fixed single | fixed multi |
|---|---|---|---|---|
| flat | 0.5261 | 0.3613 | 0.2447 | 0.3368 |
| hybrid | 0.6740 | 0.4936 | 0.4203 | 0.4126 |
| graph_flat | 0.5139 | 0.3903 | 0.2555 | 0.3652 |
| graph_hybrid | 0.6551 | 0.5025 | 0.4089 | 0.4308 |
| hier | 0.3446 | 0.4105 | 0.2939 | 0.4341 |

Kaynak `h2.json:cells.<strateji>/<chunk>/<tür>.ndcg@10`; sorgu sayısı single 173, multi 124.
### 4.2 Recall@5/10, MRR, full_cover@10 (structural)
| Strateji | Tür | Recall@5 | Recall@10 | MRR | full_cover@10 |
|---|---|---|---|---|---|
| flat | single | 0.6012 | 0.6705 | 0.4806 | 0.6705 |
| flat | multi | 0.3387 | 0.4597 | 0.4377 | 0.2177 |
| hybrid | single | 0.7225 | 0.8439 | 0.6218 | 0.8439 |
| hybrid | multi | 0.4718 | 0.5645 | 0.6395 | 0.2339 |
| graph_flat | single | 0.6012 | 0.6416 | 0.4727 | 0.6416 |
| graph_flat | multi | 0.3427 | 0.5202 | 0.4461 | 0.3065 |
| graph_hybrid | single | 0.7225 | 0.8208 | 0.6034 | 0.8208 |
| graph_hybrid | multi | 0.4718 | 0.5766 | 0.6481 | 0.2581 |
| hier | single | 0.3988 | 0.4277 | 0.3179 | 0.4277 |
| hier | multi | 0.3831 | 0.4516 | 0.5506 | 0.1935 |

Kaynak `h2.json:cells.<strateji>/structural/<tür>`.
### 4.3 Gecikme ms (p50/p95, sorgu embedding'i dahil)
| Strateji | structural p50 | structural p95 | fixed p50 | fixed p95 |
|---|---|---|---|---|
| flat | 5.4932 | 6.4935 | 4.5127 | 5.2414 |
| hybrid | 11.8197 | 18.8165 | 6.8071 | 9.2456 |
| graph_flat | 9.5624 | 26.8669 | 7.1183 | 9.7586 |
| graph_hybrid | 17.3813 | 33.4159 | 10.1636 | 13.2517 |
| hier | 9.0173 | 15.2704 | 5.1573 | 7.6268 |

Kaynak `h2.json:latency_ms`.
### 4.4 Bootstrap %95 GA (structural, eşleştirilmiş, 10000 örnek)
graph_hybrid−hybrid nDCG: single Δ -0.0188 [-0.0345, -0.0043] (n=173) · multi Δ 0.0090 [-0.0024, 0.0210] (n=124)

graph_hybrid−hybrid full_cover: multi Δ 0.0242 [0.0000, 0.0565] (n=124)

graph_flat−flat nDCG: single Δ -0.0122 [-0.0251, 0.0001] (n=173) · multi Δ 0.0290 [0.0110, 0.0474] (n=124)

`figs/h2_ndcg.png` · Kaynak `h2.json:bootstrap`

## 5. H3
p50 16.48 ms · p95 21.11 ms (N=200) · yeniden-chunk kayıp 0/0 · olay hash eşit eşit · hit@1 0.810, hit@10 0.960 (bilgi) · Kaynak `h3.json`

## 6. H4
H4-1: manifest e6945bbc2da9c47376e017e815d7e51b vs yeniden kurulum e6945bbc2da9c47376e017e815d7e51b → eşit (Kaynak `h4.json`)

H4-2: doğrulama aşamasında ölçülür (`VERIFICATION.md`).

## 7. Varsayımlar, park, dondurma istisnaları
**[H1]** (20 madde)
- T2: Nesne deposu yalnız anahtarının belirlediği alanları saklar. `objects/<vid>.json` = `{version_id, block_type, normalized_text, heading_level}`; `snapshots/<sid>.json` = `{snapshot_id, version_ids}`. Gerekçe: SPEC §4 "aynı kimlik farklı içerik → Err" kuralı, ham metin (yalnız boşluk/satır kırılımı farklı, aynı `version_id`; rust-lang/book'ta paragraf yeniden kırma commit'leri) veya snapshot'a özgü alanlar (doc_path, soy, ebeveyn; aynı içerikli iki doküman veya geri alma commit'i aynı `snapshot_id`) saklansaydı meşru girdilerde hata üretirdi. Oluşum alanları (soy, ebeveynler, köken, frontmatter, ebeveyn snapshot) olay günlüğündedir; `heading_path` blok sırasından yeniden hesaplanır.
- T3: §5.1 `k` sayacı, belgede **daha önce kullanılmış** (canlı veya silinmiş) bir soy kimliğine denk gelirse bir sonraki `k`'ya geçer. Formül yalnız sayaçla uygulansaydı, var olan bir bloğun ikinci kopyası sonradan eklendiğinde yeni kopya canlı soyla aynı kimliği alırdı (çakışma), silinmiş bir bloğun içeriği geri geldiğinde ölü soy diriltilirdi. Taze bir belgede formül birebir aynıdır (test `lineage_ids_follow_formula`).
- T3: §5.3 adım 4'te Hungarian ağırlığı `s ≥ THETA` ise `s`, değilse 0'dır (eşik atamadan önce uygulanır). Eşik altı çiftler toplamı büyütmek için eşik üstü çiftlerin seçimini değiştiremez; ayrıca uzunluk sınırı (`0.70·min/max + hp + pos < THETA`) ile Levenshtein hesabı atlanabilir ve bu kesin bir budamadır.
- T3: Adım 2/3 adayları küresel olarak `sim` azalan sırada (eşitlikte eski konum, pencere başı, pencere sonu) açgözlü seçilir; "en yüksek sim kazanır" böyle yorumlandı. Birleştirilen metinler tek boşlukla bağlanır; uzunluk = karakter sayısı.
- T3: Adım 5 (artık kontrolü) adım 4 eşleşmesini içeren, iki yanındaki eşleşmemiş ardışık yeni bloklardan oluşan tüm pencereleri (toplam ≤ 4 blok; sol ve sağ uzanım birbirinden bağımsız sınırlanır) dener; en yüksek `sim` kazanır. Kurala göre artık parça yalnız yaklaşık olarak eski bloğun %15'inden kısa olduğunda (daha uzunsa adım 2'nin alanındadır) ve birleşik benzerlik `sim(old,new)`'i 0.10'dan fazla aştığında tetiklenir; düzenlenmemiş bir bölmede bu, artığın kabaca %10'dan uzun olmasını gerektirir. Ayrıca SPEC adım sırası gereği adım 2, "güncelleme + bitişik ekleme" çiftini de split olarak görebilir (spec davranışı, değiştirilmedi).
- T3: `BlockMoved` yalnız adım 1 eşleşmelerinde (LIS dışı) üretilir; `anchor_kind` yalnız `BlockMoved` olaylarında görünür. Değişmeden kalan bloklar olay üretmez; aynı metinle yeniden ingest sıfır olay üretir (frontmatter de aynıysa).
- T5: D1 üreteci blok düzeyinde çalışır: doküman ayrıştırılır, bloklar ham metinleriyle `"\n\n"` ile yeniden yazılır (`v00.md`). Her sürüm Rust parser'ıyla yeniden ayrıştırılır ve hedeflenen blok listesini (version_id dizisi) birebir vermezse işlem örneği yeniden çekilir (en fazla 50; sonra hata). Böylece gerçek ile parser asla sessizce ayrışamaz.
- T5: Op tanımları: `rephrase_*` = Paragraph/ListItem (≥10 kelime) bloğunda kelimelerin %5–15 / %15–30'unu dokümanın kendi sözcük dağarcığından (≥4 harf) farklı kelimelerle değiştirme (kelime sayısı korunur, wikilink'e dokunulmaz). `split` = ≥2 cümleli paragrafı rastgele cümle sınırından ikiye. `merge` = iki ardışık paragrafı tek boşlukla birleştirme. `move` = rastgele bloğu başka konuma. `insert` = örneklem dışındaki D3 dokümanlarından (8–60 kelime) bir paragraf. `delete` = rastgele blok. `dup_delete` = hazırlık adımında (puanlanmaz) 6 sabit kısa nottan dokümanda olmayan biri 3 kopya olarak, aralarında en az bir özgün blok kalacak 3 konuma eklenir; puanlanan adımda ilk kopya silinir.
- T5: Bir op örneği uygulanamazsa (ör. render geri dönüşü tutmadı) aynı op 50 kez yeniden denenir; hâlâ olmuyorsa op dokümanın o anki durumunda yapısal olarak imkânsız sayılır (ör. birleştirilecek ardışık paragraf kalmadı) ve yeni bir op çekilir. Yeniden çekilişler `truth.json` içinde `redraws` alanında kaydedilir (doküman başına en fazla 30, sonra hata). Bu, gerçekleşen op oranlarını çekiliş oranlarından hafifçe saptırabilir; gerçekleşen oranlar T06'da D1 için raporlanır.
- T5: `truth.json` adım kaydı: `{"step","op","scored","file","mapping"}`; `mapping` = `[[old_ref|null, [new_ref...]], ...]` yalnız işlemin konusu olan bloklar için (`merge`: iki giriş aynı yeni bloğa; `dup_delete`: kalan iki kopya eski→yeni, silinen kopya `[]`). Ref = `{"version_id","k"}`, `doc_path` dosya başında. Rastgelelik `random.Random(f"{SEED}:{doc_path}")`.
- T5: D1 doküman seçimi (uygunluk filtresi, `eligible`): ≥8 blok, ≥3 paragraf, ≥2 düzyazı bloğu (Paragraph/ListItem, ≥10 kelime), en az bir ≥2 cümleli paragraf, en az bir ardışık paragraf çifti ve blok-render geri dönüşü. Uygun D3 dokümanlarından `random.Random(SEED).sample` ile 200 seçilir; ilk 50 ayar, 150 test.
- T5: `rephrase_*` değiştirilecek kelime sayısını `round(oran × kelime sayısı)` (en az 1) olarak hedefler; değiştirilebilir aday kelime (≥3 harf, wikilink dışı) azsa gerçekleşen oran alt sınırın altında kalabilir.
- T6: H1-S2 paydasındaki "yeni blok" = gerçekte yeni soy alması gereken bloklar: `insert` bloğu, `split` parçaları ve `merge` sonucu (SPEC §5.3: split/merge sonuçları yeni soy alır). Pay = tahminde önceki snapshot'ta var olan bir soya bağlananlar. Tanım `eval/metrics.py` içinde ilk sonuçlardan önce yazıldı (taslak 17:05, ilk H1-S sonucu 17:20, 2026-10-13); yalnız `insert` için yanlış eşleme ayrıca `per_op` tablosunda raporlanır.
- T6: H1-S1 / H1-S4 koruma birimi = işlemin konusu olan bloklar (rephrase/move hedefi; dup_delete'te kalan iki kopya). Dokunulmayan bloklar S1'e girmez (kesin eşleşmeyle zaten korunurlar).
- T6: H1-S3: tahmin edilen her `BlockSplit`/`BlockMerged` olayı (puanlanan adımlarda, op türünden bağımsız) `(tür, kaynak soy kümesi, sonuç soy kümesi)` olarak gerçekle karşılaştırılır; tam eşitlik = doğru. Havuzlanmış P/R; tahmin yoksa P=0.
- T6: THETA × THETA_SPLIT ızgarası D1 ayar setinde **düz** çıktı (15 noktada aynı metrikler). Neden: D1 `split`/`merge` metinleri tam birleşimdir (`sim = 1.0`, her eşikte), rephrase'ler en fazla %30 kelime değiştirir; ayar setinde gözlenen en düşük güncelleme skoru `s = 0.7546` (rephrase_heavy, THETA=0.50/THETA_SPLIT=0.60 koşturmasında, gözden geçirici ölçümü), yani ızgaranın üst ucunun (0.70) üstünde. Eşleyicinin THETA'yı kullandığı ızgara dışı bir tanı noktasıyla (THETA_SPLIT'in bağlı olduğunun kanıtı D1 değil, D1'de split/merge skorları hep 1.0 olduğundan, Rust testi `theta_split_changes_the_outcome`'dur) (`THETA=0.95, THETA_SPLIT=0.99` → S1 0.996→0.463) gösterildi; bu nokta seçilemez, yalnız `h1s_tuning.json` `sensitivity_probe` alanında raporlanır. Düz ızgarada seçim kuralının eşitlik bozucusu (0.60, 0.70)'i seçer. T6 kabulündeki "ızgara boyunca sonuçlar değişiyor" ölçütü bu nedenle harfiyen sağlanmadı; ölçütün amacı (THETA'nın kullanıldığının kanıtı) tanı noktasıyla sağlandı. Veri veya eşik değiştirilmedi.
- T7: "Dokunulmuş" eski satırlar: `-a,b` (b>0) hunk'ı için a−1 … a+b (hunk satırları + iki yandaki bitişik satır); `-a,0` (saf ekleme, eski a ve a+1 satırları arasında) için a ve a+1. Bir blok, satır aralığı bu kümeye değiyorsa dokunulmuştur (`eval/metrics.py::touched_lines`).
- T7: H1-R2 = tüm eski blokların (dokunulmuş + dokunulmamış) yeni snapshot'ta soyu korunan oranı; `touched_retention` = dokunulmuş eski bloklarda aynı oran. Her (commit, dosya) çifti taze bir vault'ta değerlendirilir (eski metin → yeni metin).
- T7: Kayıp kategorileri öncelik sırasıyla: `dup_content` (blok `version_id`'si eski veya yeni dokümanda >1 kez), `type_change` (aynı normalize metin yeni dokümanda farklı blok türüyle var), `hunk_adjacent` (blok sırasında hemen komşusu dokunulmuş bir blok), `other`.
- T7: D2 commit listesi `git log --no-merges -n 300 <rev> -- 'src/*.md'` ile git'in **varsayılan geçmiş sadeleştirmesi** kullanılarak seçilir (`--full-history` değil; pencerede 3 commit fark eder). Builder git ayarlarını sabitler (`diff.algorithm=myers`, `color.ui=never`, `core.quotepath=off`, `core.autocrlf=false`).
**[H2]** (21 madde)
- T2: `Vault::block(version_id)` ve chunk metni ham markdown yerine `normalized_text` kullanır (ham metin içerik-adresli değildir, bkz. üstteki madde). Kod bloklarında normalize metin satır yapısını korur.
- T4: `structural` chunk'ta her `Heading` bloğu yeni bir chunk açar ve kendi bölümünün (kendi başlığı eklenmiş `heading_path`) ilk bloğu olur; başlık bir önceki bölümün sonuna eklenmez. Aynı başlık yolu art arda tekrarlansa da (iki ayrı `## X`) chunk'lar ayrıdır.
- T4: `chunk_id = blake3(version_id'ler "\n")`; aynı dokümanda aynı version_id listesi ikinci kez çıkarsa (`k ≥ 1`) `"\n#<k>"` eklenir, böylece indeks anahtarı `(doc_path, chunk_id)` tekildir. `fixed` stratejisinde de aynı kural.
- T4: `fixed` stratejisi blokların normalize metinlerini `"\n\n"` ile birleştirip token dizisini kurar, 512'lik pencereleri 448 adımla keser; pencere metni token baytlarından (geçersiz UTF-8 kayıpsız değil, `from_utf8_lossy`) çözülür. `heading_path` penceredeki ilk bloğunkidir. `token_count` her iki stratejide chunk metninin `encode_ordinary` token sayısıdır (Python `tiktoken` ile aynı tanım).
- T4: Chunk metni: `heading_path` " > " ile, ardından `"\n\n"`, ardından bloklar `"\n\n"` ile. Başlık yolu boşsa önek yok.
- T8: Embedding modeli HF revizyonu `1110a243fdf4706b3f48f1d95db1a4f5529b4d41`'e sabitlendi; `torch.set_num_threads(4)`, batch 64, CPU. Aynı metinler aynı sırada kodlanır (iki koşturmada aynı batch bileşimi).
- T8: LanceDB tablosunda vektör indeksi (IVF/PQ) **kurulmaz**; `flat` LanceDB üzerinden kesin (kaba kuvvet) kosinüs aramasıdır. ~6000 chunk için gerekli değil ve indeks kurulumu (k-means) rastgelelik getirirdi [H4].
- T8: Tüm sıralamalarda (BM25 aday kesimi dahil) eşit skorlar anahtar (`doc_path::chunk_id`) sırasıyla bozulur. BM25 sıralamasına yalnız skoru > 0 olan chunk'lar girer (sorgu terimi içermeyen chunk'ın RRF'ye "sıralı" girmesi hataydı, T8 testinde bulundu).
- T8: Metrik tanımları (`eval/metrics.py`): ilgililik = chunk `lineage_ids` ∩ gerçek soylar ≠ ∅; gerçek grupları single'da 1, multi'de 2 (A, B). `recall@k` ve `MRR` **v1 tanımları** (EVAL): recall = her grubun soylarından ilk k chunk'ın taşıdıkları oranının gruplar üzerinden ortalaması; MRR = verilen sıralamada (ilk 10) ilk ilgili chunk'ın 1/rank'ı, yoksa 0. `full_cover@10` = multi'de ilk 10 her iki gruba da değiyorsa 1; single'da recall@10. `nDCG@10`: ikili gain, log2 indirim, IDCG = `min_cover` ilgili chunk; **v1'den fark:** v1'de `min_cover` gerçek soylarını örten en küçük chunk kümesiydi (küme örtüsü) ve geçerli sıralamalar >1 verdiği için 1.0'da kırpılıyordu; v2'de kırpma yasak olduğundan `min_cover = ideal_relevant_count = min(10, indeksteki ilgili chunk sayısı)` (standart ikili nDCG; `metrics.py` içinde, dondurulur). Bu yüzden v2 nDCG ölçeği v1 ile birebir karşılaştırılamaz; H2-5'in v1 değeri (0.061) bağlam içindir. Gecikme: koşturucu ölçülen soruların önüne 10 ayrı ısınma sorgusu koyar; `latency_summary` ilk 10 ölçümü atar. H4-2 `max_relative_difference`: NaN/sonsuz → hata; tür uyuşmazlığı veya eşit olmayan sayısal olmayan değer → sonsuz fark. Bootstrap: eşleştirilmiş, 10.000 örnek, `default_rng(SEED)`, %2.5/%97.5 yüzdelikleri.
- T10: Graf ayrıntıları (v1 grafı ile aynı yorum): `knn` ağırlığı `0.3 · cos` (her chunk'ın 5 en yakın komşusu, iki yönden gelen katkılar toplanır); wikilink hedefi dosya adı (uzantısız), bulunamazsa büyük/küçük harf duyarsız eşleme; `[[D#h]]`'de h, D'nin chunk `heading_path`'lerinde (normalize, herhangi bir derinlikte) aranır, hiçbirinde yoksa D'nin tamamı. "Aynı üst başlık" = `heading_path[:-1]` eşit ve boş değil (v2'de başlık kendi chunk'ının `heading_path`'inin son elemanıdır; aynı bölümün 512 sınırıyla bölünmüş chunk'ları da bu gruba girer). Kendine kenar yok.
- T11: Ego alt grafı budaması: düğümler sıçrama sırasıyla kabul edilir; bir sıçrama içinde, kabul edilmiş düğümlere bağlanan en güçlü kenar ağırlığına göre (eşitlikte anahtar). Tohumlar her zaman tutulur. PPR: sütun-stokastik geçiş matrisi (kenar ağırlıkları), çıkışı olmayan düğümlerin kütlesi yeniden başlatma vektörüne döner; `r ← 0.85·(P r + dangling·v) + 0.15·v`, `v` tohumlarda eşit.
- T11: `graph_hybrid`: tohumlar `hybrid`'in RRF birleşik listesinin ilk 5'i; `rrf_norm = RRF / tohumların maks RRF`; birleşik listede (flat 50 ∪ BM25 50) olmayan alt graf düğümleri için `rrf_norm = 0.5·cos_norm`, `cos_norm = max(0, cos) / alt graftaki maks cos`. `graph_flat`: `ALPHA·cos + (1−ALPHA)·ppr/ppr_max` (ham kosinüs, SPEC). `hier`: doküman embedding'leriyle ilk 20 doküman, bu dokümanların chunk'larında kesin kosinüs ilk 50 + BM25 (skor > 0) ilk 50 → RRF.
- T11: `ALPHA_DEFAULT` `mdkernel/graph_search.py` içinde; T17 `--phase tune` seçimi oraya yazar ve `--phase test` eşitliği doğrular. Başlangıç değeri 0.6 (ayar öncesi yer tutucu).
- T11: Graf stratejileri ve `hier` yalnız soğuk indeks/graf üzerinde çalışır (sıcak katman H3 için `flat`/`hybrid`'dedir). `hier` içindeki BM25 tüm korpusun istatistiklerini (IDF) kullanır, puanlanan satırları seçilen 20 dokümanla sınırlar. SPEC formülü gereği yalnız ego alt grafındaki düğümler puanlanır; bu yüzden ALPHA=1'de bile `graph_hybrid` `hybrid`'e indirgenmez (rapor yorumunda dikkate alınır).
- T12: SDK uyumu: `anthropic==1.13.0`'da `messages.create` artık `temperature` argümanı almıyor (TypeError); API `claude-sonnet-4-6` için hâlâ kabul ediyor, bu yüzden `extra_body={"temperature": 0}` ile gönderilir. `questions.py` henüz dondurulmamışken yapıldığı için istisna prosedürü gerekmez.
- T12: Yeniden üretim notu: temperature 0'da aynı istem aynı soruyu döndürebileceğinden, 2. ve 3. denemede istemin sonuna sabit bir İngilizce düzeltme cümlesi eklenir (`RETRY_LEAK` / `RETRY_NAME`, kodda sabit).
- T12: Gerçek grubu = sorunun üretildiği chunk'ın tüm blokları (structural chunk'ın `lineage_ids`'i → `(doc_path, version_id, k)`); multi'de A ve B iki grup. Multi çiftleri: A chunk'ı ≥30 token ve B dokümanına wikilink içerir; B chunk'ı hedef dokümanın ≥30 tokenlı chunk'larından (`#başlık` varsa o başlık altındakilerden) tohumlu seçilir; her doküman çifti bir kez.
- T12: Dosya adı filtresi harfiyen uygulanır: dosya adı gövdesi (uzantısız, `-`/`_` → boşluk, küçük harf) soru metninde geçiyorsa yeniden üretilir. D3'te adlar noktalı hiyerarşiktir (`dendron.topic.publish`), bu yüzden filtre nadiren tetiklenir (5 tetiklenme).
- T12: Bölünme: ayar = 25 single + 25 multi (toplam 50, tohumlu), test = kalan (173 single + 124 multi). `make eval-h2-tune` yalnız ayar kimliklerini, `--phase test` yalnız test kimliklerini okur.
- T12 (gözden geçirici bulgusu, rapora taşınacak): Multi çiftlerinin ~%49'unda A pasajı fiilen içeriksiz bir transclusion/bağlantı bloğudur; bu sorular çoğu zaman yalnız B ile cevaplanabilir. EVAL §D4'e harfiyen uygundur (≥30 token, wikilink'li çift) ve veri değiştirilmedi; H2 multi sonuçları (H2-1, H2-4, H2-5) bu sınırlamayla yorumlanır. Ayrıca 198 single sorunun ~4'ü bağlamsız ("this release") ve belirsizdir.
- T15: Karar kuralında "H2 geçti" = H2-1..H2-5'in tümü; "kısmi H2" = H2-1, H2-2, H2-4 (ve H2-5) geçip yalnız H2-3'ün kalması.
**[H3]** (3 madde)
- T8: Sıcak katmandaki her anahtar soğuktaki aynı anahtarı her zaman gölgeler (sıcak top-k'da olmasa bile); soğuk arama, gölgelenebilecek/gizlenebilecek giriş sayısı kadar fazla aday çeker. Sıcak katmanın BM25 skoru soğuk modelin istatistikleriyle (idf, avgdl, k1, b) hesaplanır; soğuk sözlükte olmayan terimler `compact()`'a kadar sıfır katkı yapar.
- T13: H3-1 v1 ile aynı tanımla ölçülür: D3'ün geçici kopyasında 200 doküman (+5 ölçülmeyen ısınma), her birine bir `rephrase_light` düzenlemesi (dondurulmuş D1 işlemi); süre = dosya yazımı → `Watcher.process` (ingest → yeniden chunk → embed → sıcak katman) → `hot.search(yeni bloğun ≥5 kelimelik ilk cümlesi, 10)` dönüşü. Model yüklemesi dahil değil (önceden yüklü). hit@1/hit@10 bilgi amaçlıdır; sıcak katmana chunk eklenmemesi veya boş arama hata. İlk taslakta "sonda paragrafı birleşik indekste ilk 10'a girene kadar" ölçülmüştü; büyük chunk'larda sonda cümlesi seyreldiği için bu indeksleme gecikmesini değil sıralama kalitesini ölçüyordu (25/50 vaka hiç ilk 10'a girmedi) ve v1 ile karşılaştırılamazdı; bırakıldı.
- T13: H3-2: tüm D3'te snapshot soyları kümesi = `fixed` chunk soyları = `structural` chunk soyları (kayıp + kazanç toplamı, iki strateji ve strateji değişimi için); chunk üretimi öncesi/sonrası `events_hash` eşitliği `log_equal`.
**[H4]** (7 madde)
- T2: Nesne deposu yalnız anahtarının belirlediği alanları saklar. `objects/<vid>.json` = `{version_id, block_type, normalized_text, heading_level}`; `snapshots/<sid>.json` = `{snapshot_id, version_ids}`. Gerekçe: SPEC §4 "aynı kimlik farklı içerik → Err" kuralı, ham metin (yalnız boşluk/satır kırılımı farklı, aynı `version_id`; rust-lang/book'ta paragraf yeniden kırma commit'leri) veya snapshot'a özgü alanlar (doc_path, soy, ebeveyn; aynı içerikli iki doküman veya geri alma commit'i aynı `snapshot_id`) saklansaydı meşru girdilerde hata üretirdi. Oluşum alanları (soy, ebeveynler, köken, frontmatter, ebeveyn snapshot) olay günlüğündedir; `heading_path` blok sırasından yeniden hesaplanır.
- T2: `SnapshotCommitted` olayı tam sıralı `(lineage_id, version_id)` listesini, `frontmatter` ve `parent_snapshot` alanlarını taşır. `replay` blok olaylarını uygulayıp her commit'te canlı soy kümesinin bu listeyle eşit olduğunu ve `snapshot_id`'nin yeniden hesaplandığını doğrular.
- T2: `events_hash` = tüm `events.jsonl` baytlarının blake3 hex ilk 32 karakteri.
- T3: §5.1 `k` sayacı, belgede **daha önce kullanılmış** (canlı veya silinmiş) bir soy kimliğine denk gelirse bir sonraki `k`'ya geçer. Formül yalnız sayaçla uygulansaydı, var olan bir bloğun ikinci kopyası sonradan eklendiğinde yeni kopya canlı soyla aynı kimliği alırdı (çakışma), silinmiş bir bloğun içeriği geri geldiğinde ölü soy diriltilirdi. Taze bir belgede formül birebir aynıdır (test `lineage_ids_follow_formula`).
- T8: Embedding modeli HF revizyonu `1110a243fdf4706b3f48f1d95db1a4f5529b4d41`'e sabitlendi; `torch.set_num_threads(4)`, batch 64, CPU. Aynı metinler aynı sırada kodlanır (iki koşturmada aynı batch bileşimi).
- T14: `make eval-h4` `data/d3`'ü siler ve yeniden kurar; kaynak git klonu (`data/src/dendron-site`) önbellek olarak yerinde kalır ve sabit rev'e yeniden checkout edilir. Tam bağımsızlık (klon dahil) H4-2 / `make verify`'da iki temiz dizinde sağlanır. Hash uyuşmazlığı hata değil, H4-1 "geçmedi" sonucudur.
- T15 (gözden geçirici turu 1 sonrası): H4-2 karşılaştırması sonuç dosyalarının (`h1s_tuning, h1s, h1r, h2_tuning, h2, h3, h4`) **tüm** JSON yapraklarını (listeler eleman eleman) kapsar; yalnız `eval/compare_runs.py::LATENCY_PATHS`'te sabit listelenen duvar saati gecikme alanları hariçtir (`h2: H2-3, latency_ms`; `h3: H3-1, summary.p50/p95, latency.latencies_ms, latency.rows[*].ms`). Bir koşturmada olup diğerinde olmayan alan hata. (Önceki dar alan listesi maddesinin yerine geçer.)
**[-]** (12 madde)
- T0: `ASSUMPTIONS.md`, `FREEZE_EXCEPTIONS.md`, `STOP.md`, `EVAL_FREEZE_ISSUES.md` dosyaları `docs/progress/` altında tutulur (CLAUDE.md yol vermiyor).
- T0: Python sürümü 3.12'ye sabitlendi (`requires-python = ">=3.12,<3.13"`); `networkx==3.7` 3.11'i desteklemiyor. CLAUDE.md "3.11+" koşulu sağlanır.
- T0: Rust çekirdek crate'inin kütüphane adı `mdkcore`; Python modül adı `mdkernel_core` bağlama crate'ine ayrıldığı için çakışmayı önler.
- T0: `uv` projesi `package = false`; uzantı modülü yalnız `maturin develop --release` ile kurulur (`make build`). `uv sync` uzantıyı kaldırabildiği için `make setup` sırayla ikisini koşturur.
- T5: macOS ld (ld-27037, Xcode 27) bu uzantı modülü için 8 bayta hizalanmamış LINKEDIT string havuzu üretti (`stroff=0x4B363C`); dyld modülü "mis-aligned LINKEDIT string pool" ile reddetti. `strip -x` ve `-ld_classic` işe yaramadı; rust-lld yolu makineye bağlı mutlak yol gerektiriyor. Çözüm: `blake3` crate'i `pure` özelliğiyle (C/asm SIMD nesneleri yok; hash çıktısı aynıdır) — hizalama düzeldi. Bu kalıcı bir kök neden düzeltmesi değil; bu yüzden `make build` uzantıyı derledikten hemen sonra `import mdkernel_core` yapar ve sorun dönerse yüksek sesle başarısız olur.
- T7: D2 metinleri git'ten bayt olarak okunur ve yazılır (satır sonu çevirisi yok); blob dosya adı git blob sha'sıdır.
- T9: `make freeze-check` H2 bölümü gerçek embedding ve LanceDB araması içerir; aynı makinede bit düzeyinde tekrarlanır. Saf bölüm (sabit girdiyle tüm metrik fonksiyonları) makineden bağımsızdır. Dondurma istisnası kontrolünde (`FREEZE_EXCEPTIONS.md`) önce/sonra karşılaştırması aynı makinede yapılır.
- T9: Dondurma kapsamı genişletildi: `eval/freeze_check.py` ve `tests/fixtures/freeze_fixture/**` (`expected.json` dahil) da `T9: freeze eval` sonrası değiştirilmez; değişirse CLAUDE.md'deki istisna prosedürü uygulanır. Aksi halde istisna kaydının dayandığı ölçüm aracı değiştirilebilirdi.
- T10: İndeks + graf önbelleği `data/d3_index/<strategy>/` (D3 `events_hash`, model revizyonu ve graf parametreleriyle damgalı; uyuşmazlıkta sıfırdan kurulur). `data/d3` dışında tutulur ki H4 yeniden kurulumu silmesin.
- T15: Rapor `reports/runs/<RUN>/FINAL.md` ve `figs/` üretir, aynısını `reports/FINAL.md` ve `reports/figs/`'e kopyalar. Serbest yorum (§8) `reports/commentary.md`'den okunur; rakam içerirse rapor hata verir ("sayı içermez" kuralı mekanik). Eşikler ve karar kuralı `eval/report.py` içinde `00_GOAL.md`'den aynen kodlanmıştır; H4-2 raporda "doğrulamada" durumundadır ve karar satırları H4-2'ye koşullu yazılır.
- T15: Rapor başlığındaki veri manifest hash'leri koşturmanın `manifest.json` kaydından alınır; fazlar arasında veya güncel commit'li manifest'lerle uyuşmazsa ya da fazlar arasında araç sürümleri farklıysa rapor hata verir. T12 park yolunda `h2.json` bulunmayacağı için rapor da hata verir (T17 zaten T12'yi bekler; park yaşanmadı).
- T18: Nihai koşturmanın ham çıktıları (`reports/runs/<RUN>/*.json`, `FINAL.md`, `figs/`) `.gitignore`'a rağmen `git add -f` ile commit'lenir; böylece `reports/FINAL.md`'deki her sayı git'teki ham JSON'a izlenebilir. Diğer ara koşturmalar yerel kalır.

**Park:** Park edilen görev yok.

**Dondurma istisnaları:**

# Dondurma İstisnaları

(Boş. Girdi şartları CLAUDE.md §Ölçüm dürüstlüğü 2.)

Not (T12): `eval/datasets/questions.py` SDK 1.x'teki `temperature` argüman değişikliğine dosya dondurulmadan **önce** uyarlandı (`extra_body`); bu bir dondurma istisnası değildir (ASSUMPTIONS T12).

## 8. Serbest yorum (ajan; sayı içermez)
**Soy kararlılığı.** Sentetik düzenlemelerde koruma, tekrar-blok çapalama ve bölme/birleştirme tespiti eşiklerin üstünde; gerçek tarihçede dokunulmamış blok koruması da eşiği geçiyor. Hipotezi düşüren tek ölçüm sentetik yanlış eşleme. Kaynağı neredeyse tamamen birleştirme işlemi: birleşen iki paragraftan kısa olanı yeni bloğun yüzde on beşinden kısa kaldığında birleştirme adayı oluşmuyor, benzerlik ataması uzun parçanın soyunu yeni bloğa devrediyor. Spesifikasyondaki artık kontrolü yalnız bölme yönünde tanımlı; birleştirme için simetrik bir artık kontrolü bir sonraki turun açık sorusu. Bu turda kural gereği eşleyici değiştirilmedi.

**Sentetik ölçümün geçerlilik sınırı.** Eşik ızgarası ayar setinde tamamen düz çıktı: bölme ve birleştirme metinleri özgün metnin tam birleşimi, yeniden yazımlar ise ızgaranın üst ucunun hep üstünde skor alıyor. Eşleyicinin eşikleri kullandığı ızgara dışı bir tanı noktasıyla gösterildi; ama ayar adımı fiilen etkisiz kaldı ve seçim eşitlik bozucudan geldi. Bu nedenle sentetik koruma ve bölme/birleştirme sonuçları kolay bir düzenleme dağılımını ölçüyor; düzenlenmiş parçalarla bölme ve birleştirme içeren daha zor bir sentetik set gelecek turda gerekli.

**Gerçek tarihçe.** Dokunulmamış bloklardaki kayıpların tamamı tekrar içerik kategorisinde ve hepsi aynı kapanış etiketi bloğu. Bağlam çapası önceki turdaki kayıp türünü büyük ölçüde gideriyor, ancak aynı belgede birebir aynı blok çok kez geçtiğinde eşleyici hâlâ düzenlenen kopyaların soyunu tutup düzenlenmeyenlerinkini bırakabiliyor.

**Yapı-farkında arama.** Graf karma strateji, sözcüksel-yoğun karma tabana karşı çok-adımlı sorularda önceden kayıtlı kazancı sağlamadı; güven aralığı sıfırı içeriyor. Tek-adımlı sorulardaki kayıp eşik içinde, gecikme oranı da eşiğin rahatça altında. Yorum için üç sınırlama var: çok-adımlı soruların yaklaşık yarısında ilk pasaj içeriksiz bir gömme bağlantısı, bu yüzden soru fiilen tek pasajla cevaplanabiliyor; seçilen karışım ağırlığı ızgaranın ucunda ve çok-adımlı sorularda ayar eğrisi bu ağırlığa neredeyse duyarsız; nDCG ideal tanımı önceki turdan farklı olduğu için önceki turla karşılaştırma yalnız bağlamsal. Graf bileşeninin bu veri ve soru setinde sözcüksel aramanın üstüne ölçülebilir bir şey eklediğine dair kanıt yok.

**Artımlı indeksleme ve determinizm.** Yazmadan aranabilir hale gelme süresi eşiğin çok altında, strateji değişimi hiçbir soy kaybettirmiyor ve olay günlüğünü değiştirmiyor. Veri kümesi sıfırdan yeniden kurulduğunda olay günlüğü bit düzeyinde aynı; bu oturumdaki tekrarlı koşturmalarda kalite sayıları da birebir aynı çıktı. İki bağımsız temiz klonda resmi karşılaştırma doğrulama aşamasına aittir.

**Karar.** Önceden kayıtlı kural gereği soy kararlılığı hipotezi geçmediği için veri modeli ve eşleyici yeniden ele alınır; arama ve sıcak katman sonuçları saklanır. Arama hipotezi de ayrıca geçmedi; graf arama, eşleyici düzeltildikten sonra bile çekirdeğe bağlanmadan araştırma kolunda kalmalı.
