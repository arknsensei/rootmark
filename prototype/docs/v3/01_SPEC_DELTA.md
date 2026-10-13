# v3 — Spesifikasyon Farkı (`01_SPEC.md` üzerine)

Yalnız burada yazılanlar değişir. `01_SPEC.md`'nin kalanı aynen geçerli.

## §5.1 Soy kimliği — değişmez
Formül aynı. Yeni soy yalnız **baskın olmayan** parçalar ve gerçekten yeni bloklar için üretilir.

## §5.3 Eşleyici v3 adımları (`matcher_version = "v3"`)

Eşleyici `MatcherVersion::{V2, V3}` ile çağrılır. `V2` v2 davranışını **bayt bayt** korur (H4-V2 bunu test eder). Varsayılan `V3`.

1. **Kesin eşleşme** — değişmez.
2. **Bölme adayları.** Eşleşmemiş eski `o` için ardışık yeni `n_i..n_j` (2 ≤ uzunluk ≤ 4), `sim(o, concat) ≥ THETA_SPLIT`. **Asgari parça oranı kaldırıldı**; yerine her parça ≥ 1 kelime. En yüksek `sim` kazanır.
   **K4:** en uzun parça (karakter sayısı; eşitlikte ilk) `o`'nun soyunu devralır, `parents=[o.version_id]`; diğer parçalar yeni soy (§5.1), `parents=[o.version_id]`. Olay `BlockSplit`, `meta.dominant = <lineage_id>`.
3. **Birleştirme adayları.** Simetrik: ardışık eski `o_i..o_j` (2–4) ↔ tek yeni `n`, `sim(concat(o), n) ≥ THETA_SPLIT`. **K4:** `n`, en uzun eskinin soyunu devralır; `parents = [o_i..o_j]`. Olay `BlockMerged`, `meta.dominant`.
4. **Benzerlik ataması** — değişmez (THETA).
5. **Artık kontrolü, bölme yönü** — değişmez.
6. **Artık kontrolü, birleştirme yönü (yeni).** Adım 4'te `(o, n)` eşleşmişse ve `o`'nun hemen komşusu `o'` eşleşmemiş (silinmiş görünen) ise: `sim(concat(o, o') veya concat(o', o), n) > sim(o, n) + 0.10` ise adım 4 eşleşmesi geri alınır ve `BlockMerged` üretilir (`n` baskın olanın soyunu devralır; baskın = daha uzun eski). En fazla 3 eski blok birleşimi denenir.
7. **Kalanlar** — değişmez.

Parametre ızgarası (ayar, D1-hard): `THETA ∈ {0.50, 0.55, 0.60, 0.65, 0.70}` × `THETA_SPLIT ∈ {0.55, 0.65, 0.75}`. Seçim kuralı: `S1 + (1−S2) + S3 + S4` (D1-hard ayar seti üzerinde H1-H* karşılıkları) maksimum; eşitlikte (0.60, 0.65)'e en yakın. Seçilen değer `V3` sabitleri olur; `V2` sabitleri (0.60, 0.70) dokunulmaz.

## §7 API
`Vault::ingest(..., matcher: MatcherVersion)`; CLI `--matcher v2|v3` (varsayılan v3); Python bağlaması `matcher="v3"`. `events.jsonl` her olayda `meta.matcher_version` taşır; bir vault içinde karışık sürüm **yasak** (ilk olayın sürümü vault'a kilitlenir, farklı sürümle ingest → `Err`).

## Değişmezler (ek)
- Aynı girdiyle `V2` çağrısı, v2-final etiketindeki çekirdekle bayt bayt aynı `events.jsonl` üretir (H4-V2).
- `V3`'te `BlockSplit`/`BlockMerged` olaylarında `meta.dominant`, `lineage_ids` içinde yer alır ve `parents` boş değildir.
