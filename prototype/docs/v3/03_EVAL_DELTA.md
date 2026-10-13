# v3 — Ölçüm Farkı (`03_EVAL.md` üzerine)

Dondurma kuralı gereği v2'nin `eval/metrics.py`, `eval/truth.py`, `eval/datasets/*.py` dosyaları **değişmez**. v3 yalnız yeni dosya ekler: `eval/datasets/synthetic_edits_v2.py`, `eval/metrics_v3.py`, `eval/run_h1s_v3.py`, `eval/run_h1r_v3.py`, `eval/run_h4_v2repro.py`. Bunlar `T23: freeze eval v3` sonrası dondurulur.

## Veri setleri

### D1-easy (K4 gerçek) — `data/d1k4/`
v2 D1 ile **aynı SEED, aynı doküman seçimi, aynı operasyon dizisi** (`synthetic_edits_v2.py` v2 üretecini çağırarak aynı metinleri üretir; metin dosyaları bayt bayt eşit olmalı, test edilir). Yalnız `truth.json` K4 semantiğiyle yazılır: `split` → en uzun parça eski soy, diğerleri yeni; `merge` → sonuç en uzun eskinin soyu. Manifest `MANIFEST.d1k4.json`.

### D1-hard — `data/d1h/`
D3'ten `random.Random(f"{SEED}:hard")` ile **farklı** 200 doküman (easy ile kesişim boş; ayar 50 / test 150). 10 adım/doküman. Operasyonlar:

| Op | Oran | Tanım | Gerçek (K4) |
|---|---|---|---|
| `rephrase_light` | 15% | %5–15 kelime | aynı soy |
| `rephrase_heavy` | 15% | %15–30 | aynı soy |
| `rephrase_extreme` | 10% | %30–45 kelime değişimi, cümle sırası karıştırılabilir | aynı soy |
| `split_edit` | 15% | cümle sınırından ikiye böl, **sonra bir parçaya `rephrase_light`** | baskın (en uzun, düzenleme sonrası) eski soy; diğeri yeni |
| `merge_edit` | 15% | iki ardışık paragrafı birleştir, **sonra birleşik metne `rephrase_light`** | en uzun eskinin soyu |
| `split_three` | 5% | üçe böl | baskın eski soy, ikisi yeni |
| `move_edit` | 10% | taşı + `rephrase_light` | aynı soy, moved |
| `dup_delete_adjacent` | 10% | 3 kopya **ardışık** (araya özgün blok konmadan) eklenir, ortadaki silinir | kalan 2 kopya soyu korur |
| `insert` | 5% | başka dokümandan paragraf | yeni |

Parçalar "tam birleşim" olmasın diye `split_*`/`merge_*` daima düzenleme içerir. Render geri dönüşü, yeniden çekiliş ve `redraws` kaydı v2 ile aynı. Manifest `MANIFEST.d1h.json`.

### D2, D3 — değişmez (aynı rev, aynı manifest).

## Metrikler (`eval/metrics_v3.py`)
- **Koruma (S1/H1):** op'un konusu olan bloklardan gerçekte eski soyu taşıması gerekenlerin tahminde de taşıyan oranı. `split_*`/`merge_*` için yalnız **baskın** parça sayılır.
- **Yanlış eşleme (S2/H2) — K4 tanımı:** payda = gerçekte **yeni** soy alması gereken bloklar: `insert` blokları ve `split_*`'in baskın olmayan parçaları. Pay = tahminde var olan bir soya bağlananlar. (`merge` sonucu artık paydada değildir; bu değişiklik `00_GOAL.md`'de açıkça kayıtlıdır.) Ayrıca **bilgi amaçlı** v2 tanımıyla S2 de raporlanır.
- **Olay F1 (S3/H3):** `BlockSplit`/`BlockMerged` olayı doğru ⇔ `(tür, kaynak soy kümesi, sonuç soy kümesi, dominant)` gerçekle tam eşit. Havuzlanmış P/R.
- **S4/H4:** `dup_delete*` kalan kopyalar.
- **H1-G:** ızgaradaki farklı amaç değeri sayısı (6 ondalık yuvarlama).
- **R1/R2:** v2 ile aynı tanım, v2 `metrics.py`'daki fonksiyonlar `metrics_v3.py` tarafından **içe aktarılarak** kullanılır (kopyalanmaz).
- **H4-V2:** `run_h4_v2repro.py`: D3'ü `--matcher v2` ile sıfırdan ingest → `events_hash` == `e6945bbc2da9c47376e017e815d7e51b`; v2 D1 (`data/d1`, v2 üreteci, v2 truth) üzerinde `--matcher v2` ve v2 `metrics.py` ile H1-S1..S4 → git'teki `reports/runs/20261013T154851Z-f133d0b/h1s.json` ile alan alan eşit.

## Koşturma
`make eval-h1s-v3 RUN= --phase tune|test` (ızgara D1-hard ayar; test D1-easy(K4) + D1-hard test), `make eval-h1r-v3 RUN=`, `make eval-h3 RUN=`, `make eval-h4 RUN=` (eşleyici v3, yeni manifest `events_hash_v3`), `make eval-h4-v2repro RUN=`, `make report-v3 RUN=`.
`make verify` v3 zincirini kapsayacak şekilde genişletilir (`scripts/verify.sh` değişir; v2 fazları da koşturulmaya devam eder).

## Rapor
`eval/report_v3.py`, `docs/v3/05_REPORT_TEMPLATE.md` şablonu (v2 şablonundan türetilir, H2 bölümü yoktur, H1-easy/hard/real ve H4-V2 bölümleri vardır). Kural: yalnız `RUN=`, elle sayı yok, eşikler `00_GOAL.md`'den aynen kodlanır.
