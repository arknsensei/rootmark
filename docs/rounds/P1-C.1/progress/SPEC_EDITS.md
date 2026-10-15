# P1-C.1 — Spec düzenlemeleri

Ölçüt: `docs/SPEC_EDITS_CRITERION.md` (K-B1.3): **açıklık** ⇔ (a) hesaplanan hiçbir değeri değiştirmez ve (b) daha önce uyumlu bir uygulamayı uyumsuz kılmaz. Bölüm durumu (K-C0.3) spec'teki güncel damgadan: §9 **Draft** (`spec/SPEC.md` satır 3, tur başında).

| # | Görev | Yer | Düzenleme | (a) | (b) | Sınıf |
|---|---|---|---|---|---|---|
| S1 | C1.0 | §9, yeni madde 5 | `proposal` istenince `canonical` sonuçlar istenmemiş halle **aynı** (chunk, sıra, skor) MUST; `proposal`'lar ayrı geçişte, istatistikleri yalnız checkpoint'in `canonical` chunk'larından, `proposal` etiketli, skor ×0.5, her `canonical`'dan sonra MUST | **sağlamaz** (`include_proposal` açıkken sıralamalar değişir: P1-C'de canonical sıralaması proposal'ların istatistiklere ve RRF adaylarına girmesiyle değişiyordu, doğrulama Ş1) | **sağlamaz** (P1-C başvuru uygulaması 1.0.1'e göre uyumluydu, 1.0.2'ye göre değil) | **anlam değişikliği** → K-C1.3 |
| S2 | C1.0 | §9, yeni madde 6 | boş / yalnız boşluk (Unicode `White_Space`; C1.0 gözden geçirme notu 5) sorgu `Err` MUST, iz yazılmaz | **sağlamaz** (P1-C boş sorguya 10 hit ve bir iz satırı üretiyordu, doğrulama Ş3) | **sağlamaz** (boş sorguyu yanıtlayan uygulama 1.0.1'e göre uyumluydu) | **anlam değişikliği** → K-C1.5 |
| S3 | C1.0 | §9, yeni madde 7 | "Informative: lexical scoring is declarative …" — `lexical.{variant,idf,k1,b,tokenizer}` manifest kaydı, başvuru değerleri | sağlar (bilgilendirici) | sağlar (bilgilendirici not hiçbir uygulamayı bağlamaz) | **açıklık** (K-C1.1) |
| S4 | C1.0 | §9, son madde (P1-C S1'in bilgilendirici notu) | istatistikler "uygun chunk'lar" → "`canonical` chunk'lar"; proposal'lar kendi aralarında sıralanır (hybrid: her tarafın ilk 50 proposal'ı), her katmandan en çok `k`; "sorgu yalnız indeks dosyalarını okur" (K-C1.4) | sağlar (bilgilendirici; normatif değer S1'den gelir) | sağlar | **açıklık** |
| S5 | C1.0 | başlık + durum satırı | `v1.0.1` → `v1.0.2`, sürüm cümlesi | sağlar | sağlar | **açıklık** (sürüm damgası; artışın gerekçesi aşağıda) |
| S6 | C1.4 | durum satırı | §9 "Draft" → "Validated" **yalnız** GOAL karar kuralı sağlanırsa (C1-1 ∧ diğerleri geçti) | sağlar | sağlar | **açıklık** (karar kuralına bağlı damga; P1-B E4 emsali) — **uygulandı** (C1.4): resmi RUN `20261015T094412Z-b9f7622`'de C1-1..C1-9 geçti; durum satırı "Validated for §9 (P1-C, P1-C.1; Level 3)", başlık "§9 (Validated, P1-C.1)" |

Uyumluluk: `spec/CONFORMANCE.md` Seviye 3'e **C3-05** satırı (K-C1.3). `core/tests/conformance.rs` `spec_version` `1.0.1` → `1.0.2` (dondurma istisnası P1-C.1/F1). C3-01..04'ün beklenen değerlerinden K-C1.3 gereği değişenler C1.3'te aşağıya eklenir.

## Sürüm kararı (K-C0.3: CRITERION'un sürüm kuralına atıfla)

**Spec sürümü 1.0.2** (GOAL K-C1.3 notu ve C1-7 `1.1.0` öngörür; PROMPT: "CRITERION farklı diyorsa CRITERION kazanır, gerekçesiyle").

1. S1 ve S2 (b)'yi sağlamaz → CRITERION gereği **anlam değişikliği**: karar kaydı (K-C1.3, K-C1.5) + spec sürümü artar + etkilenen fixture'lar yalnız o kararla yeniden üretilir.
2. CRITERION'un sürüm kuralı iki düzey tanır: "**yama**: hesaplanan değeri değişmeyen **ya da yalnız Validated/Draft bölümleri etkileyen**; **majör**: §1–6 ya da kırıcı değişiklik". Ara (minör) düzey yoktur; `1.1.0` bu kuraldan türetilemez.
3. S1 ve S2 yalnız §9'u etkiler; §9'un damgası **Draft**'tır (K-C0.3: damga spec'ten). §1–6'ya dokunulmaz. → yama koşulu sağlanır.
4. "Kırıcı değişiklik → majör" ile çakışma: (b)'yi sağlamayan her düzenleme bir anlamda kırıcıdır; CRITERION'un kendi yeniden sınıflandırma tablosu ve emsal K-B1.2/K-B1.3 bunu çözer: P1-B.1'de §7/§8'deki (b)'yi sağlamayan değişiklikler (E1 küçük harf katman, E2 `returned` biçimi, K-B1.2 `trace_id` formülü — C2-07 beklenen değeri değişti) **yama** olarak `1.0.0 → 1.0.1` yapıldı. Draft bir bölümün doğrulanmadan önce daraltılması aynı sınıftadır. DEV_PROCESS §8'in "kırıcı → majör" kuralı `mdk 0.1.0` yayımından sonraki SemVer içindir (henüz yayım yok, K-C8-T).
5. Sonuç: `1.0.1 → 1.0.2`. C1-7'deki `spec_version 1.1.0` bu kuralla `1.0.2` olarak okunur ve FINAL'de böyle raporlanır (ASSUMPTIONS C1.0-1).

## Fixture değişiklikleri (C1.3; K-C1.3, dondurma istisnası P1-C.1/F2)

Kural (PROMPT): C3-01..04 beklenen değerleri yalnız canonical-yalnız istatistik / K-C1.3 gereği değişebilir. Eski ve yeni `expected.json` işlem işlem karşılaştırıldı:

- **Bayrak kapalı her arama gözlemi bayt eşit** (C3-01..04'ün bütün `include_proposal: false` aramaları; ingest, checkpoint, build, view, traces gözlemleri de). Değişen işlemlerin hepsi `include_proposal: true` aramalarıdır ve her birinde değişen tek alan `rankings`'tir.
- Yeni dosyalarda bayrak açık her sıralamanın canonical öneki, aynı checkpoint/mod/`k`/sorgularla yapılan bayrak kapalı ikizine eşit: **50/50** (ikizi olan 5 arama × 10 sorgu).
- Değişikliğin üç nedeni (sorgu düzeyinde, eski ↔ yeni): (i) **liste uzunluğu / proposal kısmı** — eski `finalize` ağırlıklı skorla ortak ilk `k`'yı alıyordu, yeni her katmandan `k` (C1.2-1); proposal sayısı ör. C3-02 hybrid 3 → 30; (ii) **canonical kısım kısalmıştı** — eski listede proposal'lar canonical yer kaplıyordu, yeni canonical liste eskinin uzantısı (11 sorgu: eski canonical liste yeninin öneki); (iii) **canonical sıra değişti** — eski karışık istatistikler (lexical: `N`, `df`, ortalama uzunluk proposal'larla) ve hybrid'de proposal'larla paylaşılan ilk 50 RRF adayı canonical sırayı kaydırıyordu (11 sorgu; P1-C doğrulamasının Ş1'i).

| Fixture | Değişen işlemler (hepsi bayrak açık) | Nedenler |
|---|---|---|
| C3-01 | op5 lexical | (i) proposal 3 → 7, 2 sorgu |
| C3-02 | op5 hybrid | (i) 3 → 30; (ii); (iii) 1 sorgu |
| C3-03 | op4/11/15/18 flat; op5/12/16/19 lexical; op6/13/17/20 hybrid | (i) her biri; (ii); (iii) op6/13/17 1, op19 2, op20 2 sorgu |
| C3-04 | op8 flat, op10 lexical, op12 hybrid | (i); (ii); (iii) op10 1, op12 2 sorgu |
| C3-05 | yeni | K-C1.3 özellik vakası: 3 mod × 8 sorgu, `canonical_prefix_of` üçünde `equal: true`; proposal hit 13 / 32 / 32 (boş değil), `derived` 0 |
