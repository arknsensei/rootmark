# VERIFICATION — Bağımsız Doğrulama (T19)

Doğrulayan: bağımsız ajan (ana oturumun bağlamı yok). Yalnız git içeriği (çıplak klon `source.git`, HEAD `c5e722d4b7cb`) ve ağdan indirilenler kullanıldı; ana ajanın yerel `data/`'sına, önbelleğine veya ana depo dizinine dokunulmadı. HuggingFace model önbelleği kullanıldı (VERIFY'da izinli). Anthropic API çağrılmadı (D4 git'te). Kod değiştirilmedi.

Ana ajanın nihai koşturması (git): `RUN = 20261013T154851Z-f133d0b` (`docs/progress/T18.md`, `reports/FINAL.md`, `reports/runs/20261013T154851Z-f133d0b/`).
Doğrulama koşturmaları: A `20261013T160648Z-c5e722d`, B `20261013T161214Z-c5e722d` (aynı makinede, iki ayrı temiz dizinde).

Etiketler: `[OK]`, `[FARK]`, `[ŞÜPHE]`, `[İHLAL]`.

## V1 — Çift temiz koşturma `[OK]`

Komut: `bash ref/scripts/verify.sh pair <source.git> t19/A t19/B` → çıkış kodu 0.

| Zincir | Kurulum (`make setup`, Rust+maturin dahil) | Veri (`d2 d3 d1`, manifest doğrulamalı) | freeze-check | Eval fazları + rapor | Toplam |
|---|---|---|---|---|---|
| A | 11 s | 42 s | 28 s | 244 s | **325 s** |
| B | 11 s | 42 s | 29 s | 252 s | **334 s** |

Çiftin toplam duvar saati 10 dk 59 s. Her zincir taze `git clone` yaptı, D2 (`rust-lang/book`) ve D3 (`dendron-site`) kaynaklarını GitHub'dan yeniden klonladı, üç veri setini commit'li manifest'lere karşı doğruladı, `freeze-check`'i geçti, 7 fazı (h1s tune/test, h1r, h2 tune/test, h3, h4) koşturdu ve `make report` ile rapor üretti. H2 atlanmadı (D4 git'te). Her iki koşturmanın `manifest.json`'unda 7 fazın hepsi `git_sha = c5e722d4b7cb`, `worktree_clean = true`.

Not (sınırlama, ihlal değil): iki zincir aynı makinede koştu ve uv/cargo/HF önbelleklerini paylaştı. EVAL §H4 "aynı makinede iki ayrı dizinde" seçeneğine izin veriyor; farklı bir makinede bit düzeyinde eşitlik bu doğrulamada test edilmedi.

## V2 — H4 (determinizm)

**H4-1 `[OK]`**: üç hash eşit.
- `data/MANIFEST.d3.json` `events_hash`: `e6945bbc2da9c47376e017e815d7e51b`
- A `h4.json` (yeniden kurulum): `e6945bbc2da9c47376e017e815d7e51b`
- B `h4.json` (yeniden kurulum): `e6945bbc2da9c47376e017e815d7e51b`

**H4-2 `[OK]`**: `eval/compare_runs.py A B` → **maks. göreli fark = 0.0**. 7 sonuç dosyasında 22 359 JSON yaprağı karşılaştırıldı (dosya başına fark 0.0, `differing_fields` boş). Hariç tutulan alanlar yalnız `LATENCY_PATHS`'te sabit listelenen duvar saati gecikmeleri (`h2: H2-3, latency_ms`; `h3: H3-1, summary.p50/p95, latency.latencies_ms, latency.rows[*].ms`).

**git FINAL.md ↔ A FINAL.md `[OK]`**: run kimlikleri maskelenerek `diff` alındı. Farklı satırlar yalnızca şunlar: başlıktaki commit (`f133d0b` ↔ `c5e722d`; aradaki commit'ler yalnız rapor/doküman/run çıktıları ekliyor, kod değişikliği yok), H2-3 oranı, H3-1 p95, §4.3 gecikme tablosu ve §5 p50/p95. Bütün kalite sayıları birebir aynı. Ek olarak git'teki ham JSON'larla A'nın JSON'ları `compare_runs.py` ile karşılaştırıldı: 22 359 alan, maks. fark 0.0.

Gecikmede eşik **durumları** aynı:

| | git | A | B | Eşik | Durum (üçü) |
|---|---|---|---|---|---|
| H2-3 | 1.775889 | 1.714993 | 1.837496 | ≤ 3.0 | GEÇTİ |
| H3-1 p95 ms | 21.108710 | 22.928482 | 21.870433 | < 1000 | GEÇTİ |

(B'de `graph_flat/structural` p95 65.46 ms, A'da 26.41 ms. Bu alan bir eşiğe girmiyor; tekil bir gecikme sıçraması.)

## V3 — Dondurma `[OK]`

- `T9: freeze eval` = `a480432`. `git log -- eval/metrics.py eval/truth.py eval/datasets/ eval/freeze_check.py tests/fixtures/freeze_fixture` sonucunda T9 sonrasında tek commit var: `9e17e01` (T12). Bu commit yalnızca **yeni** dosya `eval/datasets/questions.py`'yi ekliyor (+296). `git diff a480432 HEAD` T9'da var olan dondurulmuş dosyaların (`metrics.py`, `truth.py`, `freeze_check.py`, `datasets/{__init__,common,git_history,synthetic_edits,vault}.py`, `freeze_fixture/**`) hiçbirinde değişiklik göstermiyor. CLAUDE.md kuralına göre yeni dosyalar eklendikleri commit'ten sonra dondurulmuş sayılır. `questions.py`'nin sonradan değiştiği bir commit yok.
- `FREEZE_EXCEPTIONS.md` boş. Dondurma istisnası gerektiren bir değişiklik de yok; SDK uyarlaması (`extra_body` temperature) dosya dondurulmadan önce yapılmış.
- `make freeze-check` A'da ve B'de geçti: `freeze-check OK (d489a0ed44b15d3aeb272e16cdd4a5af)` (iki zincirde aynı özet).

## V4 — Eşikler ve karar `[OK]`

- `git log -- docs/00_GOAL.md` → yalnız `ddb3518 T0: skeleton`. İlk commit'ten sonra değişiklik yok.
- Karar kuralını `00_GOAL.md`'den kendim yeniden türettim (A = B = git sayılarıyla):
  - H1: H1-S2 = 0.037954 > 0.02 → **KALDI**. Öteki H1 alt ölçümleri geçti. Sonuç: **H1 geçmedi**.
  - H2: H2-1 0.008964 < 0.03, H2-4 0.024194 < 0.03 ve H2-5 0.028967 < 0.05 kaldı; H2-2 −0.018821 ≥ −0.02 ve H2-3 geçti. "Kısmi H2" istisnası yalnız H2-3'ün tek başına kaldığı durum için tanımlı, burada uygulanmaz. Sonuç: **H2 geçmedi**.
  - H3: H3-1 ve H3-2 (0) geçti → **H3 geçti**.
  - H4: H4-1 eşit, H4-2 = 0 (bu doğrulama) → **H4 geçti**.
  - Uygulanan satır: **"H1 geçmedi → veri modeli/eşleyici yeniden ele alınır; H2/H3 saklanır."**
- `FINAL.md` §2 aynı satırı veriyor ve satırı H4-2'ye koşullu yazıyor. H4-2 bu doğrulamada geçtiği için koşul sağlandı ve FINAL.md'deki karar artık koşulsuz geçerli. Bana göre FINAL.md §8'deki "graf arama araştırma kolunda tutulsun" önerisi kural çıktısı değil; FINAL.md de bunu açıkça yazıyor.

## V5 — Sızıntı ve ayar/test

**8+ kelime sızıntısı `[OK]`**: A'nın `data/d3/.mdkernel/objects/` nesnelerinden gerçek metinleri çözdüm. Tüm 347 soruyu (198 single, 149 multi) programatik olarak taradım: soru ile her gerçek bloğun normalize metni arasındaki en uzun ortak ardışık kelime dizisi (küçük harf, `[a-z0-9]+`) **en fazla 7**, ≥ 8 olan **yok**. Dağılım: 0:2, 1:46, 2:110, 3:81, 4:55, 5:30, 6:15, 7:8. `random.Random(20261013).sample(tüm sorular, 20)` ile çekilen 20 örneğin bütün gerçek referansları `mdk resolve` ile soy kimliğine çözüldü (çözülemeyen yok).

**Dosya adı filtresi `[OK]` (notlu)**: 20 örneği elle inceledim. Hiçbir soru hedef dokümanın dosya adı gövdesini (`-`/`_` → boşluk) içermiyor. Tüm 347 soruda programatik taramada da isabet sıfır. Not: D3 adları noktalı hiyerarşik (`dendron.topic.pod`) olduğu için harfiyen uygulanan filtre neredeyse hiç tetiklenmiyor. 20 örneğin 5'inde adın **son segmenti** soruda geçiyor (`pod`, `workspace`, `extend`, `tutorial`, `github`). Bunlar konunun doğal sözcükleri; kural harfiyen sağlanıyor ve ana ajan bunu ASSUMPTIONS T12'de kaydetmiş. Bağlamsız sorular var (ör. s100 "image featured in the Details section", s009 "this week's CROP vote"); bu da T12 incelemesinde not edilmiş.

**Test setini önceden okuyan kod yolu `[ŞÜPHE]` (düşük)**: Güncel `run_h2.py`'de `--phase tune` yalnız `load_split("tune")` çağırıyor. Bu fonksiyon önce satırdaki `id`'yi regex ile okuyor ve test kayıtlarını ayrıştırmadan atlıyor. `--phase test` önce `ALPHA_DEFAULT == tuning.json` eşitliğini doğruluyor, test sorularını ondan sonra yüklüyor. `mdkernel/` ve `eval/` altında `data/d4` okuyan başka yol yok (soruları üreten `questions.py` hariç). Ancak ana ajanın asıl ayar koşturmasındaki `1f63132` sürümünde `load_split` **tüm** satırları `json.loads` ile ayrıştırıp test kayıtlarını sonra atıyordu. Yani test kayıtları belleğe ayrıştırıldı ama hiçbir hesaba girmedi; T17 incelemesinin "Düşük 3" bulgusuyla `f133d0b`'de düzeltildi. Sonuç etkisi yok: güncel kodla bağımsız yeniden ayarlama A'da ve B'de aynı eğriyi verdi ve aynı ALPHA = 0.9'u seçti (`h2_tuning.json` farkı 0); aksi halde `--phase test` hata verirdi. `tests/test_run_h2.py::test_expected_counts_come_from_the_split_file` test bölünmesini okuyor ama `f133d0b`'de, yani ALPHA sabitlendikten sonra eklendi.

**Bölünme kesişimleri `[OK]`**: D4 ayar 50 / test 297, kesişim ∅, birleşim = 347 sorunun tamamı, tekrar yok. D1 ayar 50 / test 150, kesişim ∅. Ayrıca `run_h2 --phase test` içinde `tune_ids ∩ test = ∅` kontrolü var. Not: D4'te EVAL hedefi 200/150 iken 198/149 soru var; EVAL "3 deneme, sonra atla" kuralına uygun.

## V6 — Fail-fast kontrolleri (kasıtlı bozma) `[OK]`

A'nın kopyasında (`t19/A_corrupt`) kod değiştirmeden yalnız veri bozuldu. Kopya sonra silindi.

| Bozma | Komut | Sonuç |
|---|---|---|
| `data/d4/single.jsonl` s000'de bir `version_id` → `0123…cdef` | `make eval-h2 RUN=x` | Hata, ama doğrulama bozmaya ulaşmadan duruyor: `malformed RUN 'x'` |
| aynı | `make eval-h2 RUN=20261013T160648Z-c5e722d` (gerçek ayar dosyası mevcut) | **Hata**: `TruthError: unresolvable truth dendron.alternatives.roam-vs-dendron.md:0123…#0` |
| `MANIFEST.d3.json` `events_hash` 1 hane değişti | `make d3` | **Hata**: `MANIFEST.d3.json mismatch in fields: ['events_hash']` |
| `MANIFEST.d3.json` bir doküman blake3'ü 1 hane değişti | `make d3` | **Hata**: `mismatch in fields: ['docs']` |
| `reports/runs/<RUN>/h2.json` silindi (dizinde git'ten gelen başka tam bir run da var) | `make report RUN=<RUN>` | **Hata**: `run … is missing ['h2.json']`. Başka run'a geri düşmedi. |

Hiçbiri sessizce çalışmadı.

## V7 — İzlenebilirlik `[OK]`

`FINAL.md`'den 12 sayıyı git'teki `reports/runs/20261013T154851Z-f133d0b/*.json` içinde `json.load` ile buldum. Hepsi 6 haneli yuvarlamayla birebir:

| FINAL.md | JSON alanı | Ham |
|---|---|---|
| H1-S2 0.037954 | `h1s.json:H1-S2` | 0.037953795379537955 |
| H1-S3 0.961083; P 1.0 / R 0.9251 / 284-284-307 | `h1s.json:H1-S3`, `H1-S3_detail` | 0.961082910321489; 1.0 / 0.92508 / 284,284,307 |
| merge yanlış eşleme 0.1229 | `h1s.json:per_op.merge.false_match` | 0.12290502793296089 |
| H1-R1 0.999843 | `h1r.json:H1-R1` | 0.9998433399913838 |
| H1-R2 0.978662 | `h1r.json:H1-R2` | 0.9786619917835387 |
| H2-1 0.008964 | `h2.json:H2-1` | 0.00896412256494511 |
| H2-2 −0.018821 | `h2.json:H2-2` | −0.01882082210334679 |
| H2-4 0.024194 | `h2.json:H2-4` | 0.02419354838709678 |
| H2-5 0.028967 | `h2.json:H2-5` | 0.028966844718882834 |
| hybrid structural single/multi nDCG 0.6740 / 0.4936 | `h2.json:cells` | 0.67396 / 0.49358 |
| bootstrap gh−h multi Δ 0.0090 [−0.0024, 0.0210] | `h2.json:bootstrap` | 0.008964 [−0.002385, 0.020965] |
| H3-1 p95 21.108710 | `h3.json:H3-1.p95_ms` | 21.108710439875708 |
| ALPHA 0.9, eğri ucu 0.5710 | `h2_tuning.json` | 0.9, 0.5710430626820245 |

H1-R1 kayıp kategorileri: FINAL `dup_content 8, hunk_adjacent 0, other 0, type_change 0` = `h1r.json:loss_categories`. `untouched_lost = 8 = 51066 − 51058` ile de tutarlı; `other_count = 0`.

## V8 — Metrik uyumu

`eval/metrics.py` ile `03_EVAL.md`'yi satır satır karşılaştırdım (H1 koruma/yanlış eşleme/F1, R1/R2/touched, kayıp kategorileri ve `other` örnekleri, H2 ilgililik/recall/MRR/nDCG/full_cover, gecikme ısınması, bootstrap, H3, H4). Elle örnekleri A'nın ortamında çalıştırdım:

| Örnek | Beklenen | `metrics.py` |
|---|---|---|
| nDCG, ilgili chunk'lar 1. ve 3. sırada, `min_cover=2` | (1 + 1/log2 4) / (1 + 1/log2 3) = 0.91972 | 0.9197207891 ✓ |
| nDCG, 3 ilgili ilk 3'te, `min_cover=3` | 1.0 | 1.0 ✓ |
| **nDCG > 1 girdisi**: 3 ilgili ilk 3'te, `min_cover=2` | hata | `MetricError: nDCG 1.3066 > 1` ✓ (kırpma yok) |
| full_cover, multi, yalnız A kapsanmış | 0 | 0.0 ✓ |
| recall multi, A tam / B yok | 0.5 | 0.5 ✓ |
| single'da full_cover = recall@10 | eşit | 0.5 = 0.5 ✓ |
| H1-S4 dup_delete, kalan 2 kopya korunmuş | 1.0 | 1.0 ✓ |
| H1-S4 dup_delete, biri kaymış | 0.5 | 0.5 ✓ |
| boş payda | hata | `MetricError` ✓ |

- `[OK]` H1 tanımları EVAL ile uyumlu. "Dokunulmamış" = hunk aralığı ve iki yanındaki bitişik satır dışında (`-a,b` → a−1…a+b; `-a,0` → a, a+1). Bu git `-U0` anlamıyla doğru. S2 paydası (insert + split parçaları + merge sonucu) SPEC'e uygun; ASSUMPTIONS T6'da ilk sonuçtan önce tanımlandığı kaydedilmiş.
- `[FARK]` nDCG IDCG'sindeki `min_cover`: EVAL bu terimi v1 anlamıyla (en küçük örtü kümesi) kullanıyor; v2 kodu `min_cover = min(10, indeksteki ilgili chunk sayısı)` kullanıyor (standart ikili nDCG). Fark ASSUMPTIONS T8'de açıkça yazılmış, dondurmadan önce yapılmış ve EVAL'in ">1 hata, kırpma yok" şartını doğru biçimde karşılıyor (v1 tanımı kırpmasız >1 verirdi). Sonuç etkisi: H2-5'in v1 değeriyle (0.061) karşılaştırması yalnız bağlamsal. H2 eşik durumlarını değiştirmez; eşikler v2 içindeki farklar üzerinde.
- Bilgi: `per_op[*].event_recall` aynı gerçek olayla iki tahmini çift sayabilir (havuzlanmış S3 bunu doğru yapıyor). Yalnız bilgi tablosunu etkiliyor, eşiği etkilemiyor.

## V9 — Rust/Python tutarlılığı `[OK]`

- `mdk ingest-dir` ile `tests/fixtures/mini_vault` → `events_hash = 8914f9b418ae360d49c85c951050f432`. Python `mdkernel_core.Vault.ingest_dir` → aynı hash. Ayrıca iki `events.jsonl` bayt düzeyinde aynı (`cmp`).
- `mdk chunk` (structural) ile 5 dokümanın (blends, glossary, roast-log, roasting, safety; 11 chunk) `token_count` değerleri, Python `Vault.chunk` değerleriyle aynı. Python `tiktoken` `cl100k_base` `encode_ordinary(text)` uzunluğu da aynı. Mini vault'un tüm structural ve fixed chunk'larında (45 chunk) üçü birebir eşit.

## V10 — Varsayımlar ve park

- `ASSUMPTIONS.md` etiketli: [H1] 20, [H2] 21, [H3] 3, [H4] 4, [-] 12 madde (FINAL.md §7 çok etiketli maddeleri birden fazla grupta listeliyor).
- `STOP.md` yok, `EVAL_FREEZE_ISSUES.md` yok, park edilen görev yok (`PARK:` yalnız T15 incelemesinde park yolunun tartışmasında geçiyor). `FREEZE_EXCEPTIONS.md` boş.
- Sonucu etkileyebilecek varsayımlar:
  1. **[H1] S2 paydası ve merge davranışı**: H1'i düşüren tek ölçüm S2. Kaynağı neredeyse tamamen `merge` (per_op yanlış eşleme 0.1229). Tanım SPEC'e uygun ve ilk sonuçtan önce sabitlenmiş; bu yüzden H1 kararı sağlam.
  2. **[H1] Düz THETA ızgarası**: D1'de split/merge metinleri tam birleşim, rephrase skorları ızgaranın üstünde. Ayar adımı fiilen etkisiz kaldı ve seçimi eşitlik bozucu yaptı. Sentetik S1/S3/S4 kolay bir dağılımı ölçüyor; ana ajan bunu raporlamış.
  3. **[H2] Multi soruların ~%49'unda A pasajı içeriksiz transclusion**: H2-1/H2-4/H2-5'in "geçmedi" sonucunu yorumlarken önemli bir sınırlama. Kural sonucu değişmiyor (H1 zaten geçmedi).
  4. **[H2] ALPHA ızgara ucunda (0.9)**: seçim kuralı önceden sabit, eğri multi'de düz.
  5. **[H2] nDCG `min_cover` tanımı** (V8 FARK).

## Sonuç

- **İhlal: 0 · Fark: 1 · Şüphe: 1**
  - FARK: V8 nDCG `min_cover` tanımı v1'den farklı (belgelenmiş; yalnız v1 karşılaştırmasını etkiliyor).
  - ŞÜPHE (düşük): V5'te ana ajanın ilk ayar koşturmasının kodu test kayıtlarını ayrıştırıp atıyordu. Hiçbir hesaba girmedi; bağımsız yeniden ayarlama aynı ALPHA'yı verdi.
- **Hüküm: güvenilir.** Kalite sayılarının tamamı iki bağımsız temiz klonda bit düzeyinde yeniden üretildi (H4-2 = 0, 22 359 alan) ve git'teki ana ajan koşturmasıyla birebir aynı (maks. fark 0). H4-1 üç yönlü eşit. Gecikme eşik durumları aynı. Fail-fast kontrollerinin hepsi çalıştı. Dondurma ve eşikler bozulmamış. Kapsam sınırı: iki koşturma aynı makinede yapıldı; farklı makinede eşitlik bu doğrulamanın dışında.
- **Karar kuralına katılıyorum.** H4 geçti (H4-1 ve H4-2), bu yüzden öteki sonuçlar geçerli. H1 geçmedi (H1-S2 = 0.0380 > 0.02). Uygulanan satır: **"veri modeli/eşleyici yeniden ele alınır; H2/H3 saklanır."** H2 ayrıca geçmedi (H2-1, H2-4, H2-5); kısmi-H2 istisnası uygulanmaz.
- **Ana ajana öneriler (en fazla 3):**
  1. `scripts/verify.sh pair` H4-2 ≠ 0 olsa bile 0 ile çıkıyor (`compare_runs.py` farkı yalnız yazıyor). `H4-2_equal == false` durumunda betik sıfır olmayan kodla çıkmalı; `make verify` tek başına hüküm verebilsin.
  2. Eşleyicinin yeniden ele alınmasında birleştirme için simetrik bir artık kontrolü ve düzenlenmiş parçalarla split/merge içeren daha zor bir D1 kurulmalı; şu anki THETA ızgarası düz olduğu için ayar adımı fiilen bilgi taşımıyor.
  3. Bir sonraki H2 turunda, içeriksiz A pasajlı multi çiftleri baştan elenmeli (ör. A chunk'ında wikilink/transclusion dışında asgari içerik şartı) ve dosya adı filtresi noktalı hiyerarşik adların son segmentini de kapsamalı; ayrıca H4-2 farklı bir makinede de koşturulmalı.
