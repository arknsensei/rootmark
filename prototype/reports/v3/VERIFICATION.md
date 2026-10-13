# mdkernel-v3 — Bağımsız Doğrulama (T28)

Doğrulayıcı: ayrı oturum, ana oturum bağlamı yok. Tek kaynak: çıplak klon `origin.git` (HEAD `19d4b09`, `v2-final` etiketi = `5c53be4`). Ana çalışma deposuna okuma dahil erişilmedi. Kod değiştirilmedi; kasıtlı bozmalar yalnız A'nın kopyasında (`Acopy/`) ve kopya koşturma dizinlerinde yapıldı. Talimatlar: `docs/04_VERIFY.md` (v2 V1–V10) + `docs/v3/04_VERIFY_DELTA.md`.

Koşturmalar:
- A: `20261013T184210Z-19d4b09` — 450 s (18:40:32 → 18:48:02 UTC)
- B: `20261013T184941Z-19d4b09` — 491 s (18:48:02 → 18:56:13 UTC; B sırasında doğrulayıcının kopya üzerindeki işleri paralel koştu, yalnız gecikmeyi etkiler)
- `verify.sh pair` toplam 941 s, çıkış kodu 0.
- Ana ajanın RUN'ı (karşılaştırma için, git'te): `20261013T182004Z-f82dabd` (`docs/progress/v3/T27.md`).

Kapsam notu: v3 turunda H2 koşturulmaz (`docs/v3/00_GOAL.md`, `CLAUDE_APPEND.md`). Bu yüzden v2 V5'in D4/H2 maddeleri ve V6'nın `eval-h2` maddesi **kapsam dışıdır**; `[FARK]` sayılmaz. Yerlerine v3 karşılıkları uygulandı (aşağıda).

## V1 — Çift temiz koşturma
- [OK] `bash scripts/verify.sh pair origin.git A B` → çıkış 0. Her zincir: klon → `make setup` → `d2 d3 d1 d1k4 d1h` (tüm manifestler doğrulandı) → `freeze-check` OK `d489a0ed44b15d3aeb272e16cdd4a5af` → `freeze-check-v3` OK `d138cddcde1d0978a020a6fe8df2f610` → v2 fazları (H1-S tune/test, H1-R; eşleyici v2) → v3 fazları (H1-S v3 tune/test, H1-R v3, H3, H4, H4-V2) → `report-v3`. A ve B'de her fazın `manifest.json` kaydı `git_sha 19d4b09…`, `worktree_clean: true`.
- [OK] Kasıtlı test: A'nın koşturma dizininin kopyasında `h1s_v3.json:hard.H1-H1` 0.9882451721… → 0.988246 yapıldı; `verify.sh compare <A/repo> <kopya> <B run>` → **çıkış 3**, `H4-2_equal: false`, maks. göreli fark 8.38e-07, "error: H4-2 comparison failed". Ek: kopyadan `h4_v2repro.json` silinince `compare` → çıkış 1.

## V2 — H4 (determinizm) ve V2-ek (H4-V2)
- [OK] **H4-1 (v3):** `MANIFEST.d3.json:events_hash_v3` = `7335baebefba377c84574928dd4cc05c`; A yeniden kurulum = `7335baeb…`; B = `7335baeb…` → üçü eşit. v2 günlüğü (aynı kurulum) A ve B'de `e6945bbc2da9c47376e017e815d7e51b` = manifest.
- [OK] **H4-2:** `compare_runs --set v3` A ↔ B: 9 dosya, 1577 kalite alanı, maks. göreli fark **0.0** (dosya başına 0.0). Dışlananlar yalnız `h3_v3.json` gecikme yolları. Ek olarak A ↔ ana RUN (`20261013T182004Z-f82dabd`, git'teki ham JSON): 1577 alan, fark 0.0.
- [OK] **H4-V2 koşul 1:** `--matcher v2` ile D3 sıfırdan ingest `events_hash` = `e6945bbc2da9c47376e017e815d7e51b` (A ve B, `h4_v2repro.json:H4-V2.d3`).
- [OK] **H4-V2 koşul 2:** v2 D1 + v2 metrikleri, `v2-final:reports/runs/20261013T154851Z-f133d0b/h1s.json` ile 35 alan, fark 0 (A ve B). Bağımsız çapraz kontrol: zincirin v2 fazının ürettiği `h1s.json` (A) `git show v2-final:…/h1s.json` ile Python `==` ile tümden eşit (S1 1.0, S2 0.037953795…, S3 0.961082910…, S4 1.0).
- [OK] Git'teki `reports/v3/FINAL.md` (RUN `20261013T182004Z-f82dabd`) ile A'nın `FINAL.md`'si: `diff` yalnız RUN/commit satırı ve H3-1 gecikme satırları (p95 21.42 → 21.33 ms; B'de 21.98 ms). Kalite sayıları birebir; gecikme eşik durumu (< 1000 ms → GEÇTİ) her üçünde aynı. A ↔ B FINAL farkı da yalnız gecikme.

## V3 — Dondurma
- [OK] v2 dondurması: `git diff a480432 HEAD -- eval/metrics.py eval/truth.py eval/datasets/{__init__,common,git_history,synthetic_edits,vault}.py eval/freeze_check.py tests/fixtures/freeze_fixture` → **boş**. `eval/datasets/questions.py` yalnız T12 commit'inde (eklendiği commit). `make freeze-check` A ve B'de OK `d489a0ed…`.
- [OK] v3 dondurması: `T23: freeze eval v3` commit'i `aa3a2e2`, mesaj tam olarak bu (ek satır yok). Sonrasında değişiklik: `eval/metrics_v3.py` (yalnız aa3a2e2), `eval/datasets/synthetic_edits_v2.py` (yalnız 5ff5797, dondurmadan önce), `tests/fixtures/freeze_fixture_v3/**` (yalnız aa3a2e2) → değişmemiş. Dondurmadan sonra eklenenler eklendikleri commit'ten sonra değişmemiş: `eval/run_h1s_v3.py` (ddf2b36), `eval/run_h1r_v3.py` (49b5783), `eval/run_h4_v2repro.py` (146f45a). `make freeze-check-v3` A ve B'de OK `d138cddc…`.
- [OK] `FREEZE_EXCEPTIONS.md` v3/T24 girdisi (`eval/freeze_check_v3.py`): (a) girdi, neden ve diff var; (b) önce/sonra bit eşitliği kaydı var; (c) gözden geçirici onayı (`T24.review.md`, "ONAYLANDI") var. Bağımsız yeniden üretim: dondurma anındaki denetleyici (`git show aa3a2e2:eval/freeze_check_v3.py`) güncel çekirdekle kopyada koşturuldu → `FAILED: output 9e8389668db180218e2e0ce7a254c6d7` (kayıttaki değer); güncel denetleyici → `d138cddc…` = `expected.json`. Not: istisna sınıfı ("bağımlılık API değişikliği") tam uymuyor; ancak dosya CLAUDE.md v3 dondurma listesinde değil (`run_*_v3.py` kalıbına girmez), dondurulmuş metrik/üreteç/fixture dosyaları değişmedi ve değişiklik ölçüm sonuçlarını etkilemiyor. Prosedür şeffaflık için eksiksiz uygulanmış → ihlal değil.

## V4 — Eşikler ve karar (V4-ek)
- [OK] `docs/v3/00_GOAL.md` tek commit'te: `5c53be4 v3: directives` (= `v2-final` etiketi, T20'den önce); sonrasında değişiklik yok. `docs/00_GOAL.md` de yalnız T0 commit'inde.
- [OK] Karar kuralını kendi koşturmalarımdan türettim (A; B ve ana RUN aynı):
  - H4-V2 geçti; H4-1 geçti; H4-2 = 0 (bu doğrulama) geçti → sonuçlar geçerli.
  - H1-G = 4 ≥ 3 (15 hücre, 6 ondalıkta 4 farklı amaç değeri; kendim saydım) → H1-H* **kanıt** sayılır.
  - H1-S1 1.000 ≥ 0.95; S2 0.000 ≤ 0.02; S3 1.000 ≥ 0.80; S4 1.000 ≥ 0.95.
  - H1-H1 0.988245 ≥ 0.90; H2 0.000 ≤ 0.03; H3 0.996721 ≥ 0.70; H4 1.000 ≥ 0.90.
  - H1-R1 0.999843 ≥ 0.999 (51066 dokunulmamış blokta 8 kayıp, tümü `dup_content`); R2 0.981725 ≥ 0.85.
  - H3-1 p95 21.3 ms < 1000; H3-2 = 0.
  - → **"`01_SPEC.md` §1–7 (K4 ve v3 eşleyiciyle) dondurulur; çekirdek fazı 1 başlar."** FINAL.md §2 çıktısıyla aynı (FINAL'deki H4-2 koşulu bu doğrulamayla karşılandı).
- [OK] H1-G şartı doğru uygulanmış: `report_v3.py` H1-G'yi H1-H*'in kanıt/bilgi statüsü için kullanıyor; seçim kuralı (amaç maks.; eşitlikte (0.60, 0.65)'e en yakın) elle kontrol edildi: θ=0.50 satırları 3.9860 ile eşit, (0.50, 0.65) uzaklığı 0.10 < 0.141 → seçim doğru.

## V5 — Sızıntı ve ayar/test (v3 karşılığı)
- Kapsam dışı: D4 soru sızıntısı, dosya adı filtresi, `run_h2 --phase test` (H2 koşturulmaz).
- [OK] Ayar/test ayrımı: `phase_tune` yalnız `splits("hard")["tune"]` üzerinde değerlendirir (D1-easy ve test bölünmesine dokunmaz); `phase_test` önce derlenmiş V3 sabitlerini `h1s_v3_tuning.json` seçimiyle karşılaştırır. Kopyada seçim 0.55'e değiştirilince → `RuntimeError: compiled V3 THETA/THETA_SPLIT (0.5, 0.65) != tuning selection`, çıkış 2.
- [OK] Bölünme kesişimleri (programatik, A verisi): D1-easy tune 50 / test 150, kesişim 0; D1-hard tune 50 / test 150, kesişim 0; `docs.json` `split` etiketleri `splits.json` ile tutarlı.
- [OK] Eşleyici v3 kodu (`core/src/lineage.rs`, T21 `0fbdf33`) D1-hard üretecinden (T22) önce commit'lenmiş; sonra yalnız T24 sabit commit'i (`b017755`) değiştirmiş.

## V6 — Fail-fast (kasıtlı bozma, A'nın kopyasında) ve V6-ek
- [OK] `MANIFEST.d3.json` bir doküman hash'i değiştirildi → `make d3` hata (`MANIFEST.d3.json (v2 fields) mismatch in ['docs']`, çıkış 2).
- [OK] `events_hash` (v2) değiştirildi → `make d3` hata (`mismatch in ['events_hash']`).
- [OK] `events_hash_v3` değiştirildi → `make d3` hata (`events_hash_v3 7335… != MANIFEST.d3.json 8335…`).
- [OK] `reports/runs/<RUN>/h4_v2repro.json` silindi → `make report-v3 RUN=<RUN>` hata (`FileNotFoundError: … missing ['h4_v2repro.json']`).
- [OK] (v3 karşılığı, v2'deki `version_id` bozma) `data/d1h/docs/150/truth.json` içinde bir `version_id` sıfırlandı → `make eval-h1s-v3 PHASE=test` manifest hatası; manifest girdisi de güncellenince → `TruthError: unresolvable truth …`, çıkış 2. Sessiz sıfır yok.
- Kapsam dışı: `data/d4/single.jsonl` → `make eval-h2` (H2 yok).
- [OK] V6-ek: `diff -rq data/d1/docs data/d1k4/docs` → yalnız `truth.json` farkları (184/200); tüm `v*.md` metinleri bayt bayt eşit; `splits.json` ve `docs.json` da bayt bayt eşit. `truth.json`'u aynı kalan 16 dokümanda split/merge op'u yok (beklenen).
- [OK] Easy/hard doküman kesişimi (kaynak doküman adına göre): 200 ∩ 200 = **0**.

## V7 — İzlenebilirlik
- [OK] Git'teki FINAL.md'den 12 sayı ana RUN JSON'larında bulundu: H1-H1 0.988245 (`h1s_v3.json:hard.H1-H1`), H1-H3 0.996721 ve tp 456 / tahmin 456 / gerçek 459 (`hard.H1-H3_detail`), v2-tanımlı S2 0.506601/payda 606 ve 0.532483/payda 862 (`*_v2def`, `n_false_match_v2def`), paydalar 799/299/300 ve 1191/403/330 (`n_retention`, `n_false_match`, `n_dup`), rephrase_extreme 0.9136 (`per_op`), merge_edit olay recall 0.9850, R1 0.999843 / R2 0.981725 / çift 836 / dokunulmamış 51066 (`h1r_v3.json`), H1-G 4 (`h1s_v3_tuning.json`).
- [OK] H1-R1 kayıp kategorileri: `dup_content 8, hunk_adjacent 0, other 0, type_change 0` = `h1r_v3.json:loss_categories`; `untouched_lost` 8 ile tutarlı (1 − 8/51066 = 0.999843).

## V8 — Metrik uyumu (V8-ek, `metrics_v3.py` elle örnekler)
- [OK] Merge (A baskın, B; sonuç A soyunda) + 1 insert: merge `new_linked = []` (paydada değil), S2 paydası 1 (yalnız insert), H1-H2 = 0.0; merge koruma `[True]`.
- [OK] Split, baskın olmayan parça var olan B soyuna bağlanmış: H1-H2 = 1.0 (> 0); aynı kayıt + 1 doğru insert → 0.5.
- [OK] Doğru split (baskın A, diğer yeni X): H1-H2 0.0, H1-H3 1.0.
- [OK] Olay F1: split `meta.dominant` = X (yanlış) → F1 0.0; `dominant` yok → 0.0; merge `dominant` = B (yanlış) → 0.0. Dominant eşitsizliği doğru sayılmıyor.
- [OK] H1-G: `[1, 1.0000001, 2, 3]` → 3 (6 ondalık yuvarlama). Boş payda → `MetricError` (fail-fast).
- [OK] `metrics_v3.py` ↔ EVAL_DELTA: S1/H1 op kümeleri GOAL tablosuyla aynı (easy: rephrase_*, move; hard: + rephrase_extreme, move_edit, split_edit/merge_edit baskın); R1/R2 v2 `metrics.py`'dan içe aktarılıyor (kopya yok).

## V9 — Rust/Python tutarlılığı (V9-ek)
- [OK] `tests/fixtures/mini_vault`: `v2-final` etiketinden derlenen `mdk ingest-dir` → `8914f9b418ae360d49c85c951050f432`; HEAD `mdk ingest-dir --matcher v2` → `8914f9b4…`; ayrıca iki vault'un `events.jsonl` dosyaları **bayt bayt eşit** (`cmp`). `--matcher v3` → `808a41f21a43e121d7d688fee9f5bc24` (farklı), iki koşturmada aynı; varsayılan (MDK_MATCHER yok) = v3.
- [OK] Python `mdkernel_core.Vault(matcher="v2"/"v3")` aynı dosya sırasıyla: `8914f9b4…` / `808a41f2…`; `matcher_lock()` doğru. `matcher_defaults`: v2 (0.6, 0.7), v3 (0.5, 0.65).
- [OK] Karışık sürüm: v2 vault'una `--matcher v3` ingest → `error: vault is locked to matcher v2; ingest with v3 refused`, çıkış 1.
- [OK] 5 doküman (12 chunk) için `mdk chunk` ↔ Python `Vault.chunk` `token_count` eşit ve `tiktoken cl100k_base` ile aynı.

## V10 — Varsayımlar ve park
- [OK] `STOP.md`, `EVAL_FREEZE_ISSUES.md` yok; park edilen görev yok; T20–T27 kendi ve gözden geçirici kayıtları TAMAMLANDI. FREEZE_EXCEPTIONS: yalnız v3/T24 (V3'te değerlendirildi).
- Sonucu etkileyebilecek varsayımlar (hepsi `ASSUMPTIONS.md`'de etiketli ve FINAL §8'de raporlu): D1-hard uygunluk kuralı gevşetildi (205 adaydan 200 seçildi); `split_three` yan cümle sınırından da bölünebiliyor; `split_edit`'te bir parça, `split_three`'te iki parça, `merge_edit`'in ~%53'ünde bir eski blok aynen kalıyor (zor setin "zorluğu" sınırlı); H4-1 (v3) ilk ingest olduğu için esasen kurulum determinizmini sınıyor, eşleyici farkını değil.
- [ŞÜPHE] **D1-easy bağımsız bir sınama seti değil:** metinleri, bölünmeleri ve test dokümanları v2 D1 ile bayt bayt aynı; K4 ve v3 eşleyici tasarımı bu setteki v2 sonuçları (S2 = 0.038, birleştirme kaynaklı) görüldükten sonra yapıldı. GOAL bunu açıkça kabul ediyor. Etkiyi azaltan: D1-hard, eşleyici kodu commit'lendikten sonra üretildi ve onda da tüm eşikler geçiyor. Sonuç geçersiz değil, ama H1-S* "doğrulama" değil "düzeltmenin hedef sette çalıştığı" olarak okunmalı.
- [ŞÜPHE] **H1-G harfiyen geçiyor, ama zayıf:** 4 farklı değerin tümü THETA'dan geliyor; THETA_SPLIT ızgarada hiçbir metriği değiştirmiyor; H1-H2 tüm hücrelerde 0, H1-H3 tümünde aynı, H1-H4 tümünde 1. Farkı yalnız H1-H1 (pratikte `rephrase_extreme`) yaratıyor. Seçilen THETA=0.50 ızgaranın alt sınırında. THETA_SPLIT_V3 = 0.65 veriden değil eşitlik kırma kuralından (ajan bunu kendisi kaydetmiş). Yani D1-hard, bölme/birleştirme parametresi açısından hâlâ ayrım yapmıyor.

## Sonuç
- **İhlal: 0 · Fark: 0 · Şüphe: 2.** (Kapsam dışı: H2'ye bağlı V5/V6 maddeleri.)
- **Hüküm: güvenilir, şu koşullarla.** Ölçüm zinciri bağımsız olarak iki kez yeniden üretildi (H4-1, H4-2 = 0, H4-V2 eşit). Ana ajanın kalite sayıları birebir doğrulandı. Dondurma kuralları korunmuş, fail-fast yolları çalışıyor. Koşullar: (1) H1-easy sonuçları, tasarımı yönlendiren setin aynısında elde edildi; (2) D1-hard ızgarası yalnız THETA'ya duyarlı, bölme eşiği doğrulanmış değil.
- **Karar kuralı:** Katılıyorum. Kendi türetmem: tüm H1-S*, H1-H* (H1-G=4 → kanıt), H1-R*, H3, H4-1, H4-2, H4-V2 geçti → `01_SPEC.md` §1–7 (K4 + v3 eşleyici) dondurulur; çekirdek fazı 1 başlar.
- **K4 tanım değişikliği raporda açıkça işaretlenmiş mi: EVET.** FINAL.md başında kalın "K4 tanım değişikliği (sonuç görüldükten sonra alınmış tasarım kararı)" bloğu var; v2 tanımlı S2 (0.506601 / 0.532483) bilgi amaçlı ayrıca raporlanmış.

### Öneriler (en fazla 3)
1. Çekirdek fazı 1'de, v3 eşleyicisini ayar ve tasarımda hiç kullanılmamış yeni bir sentetik sette (yeni SEED, D3 dışı veya yeni D3 dilimi) bir kez sınayın. Böylece D1-easy'nin tasarımla iç içe olmasından gelen şüphe kapanır.
2. THETA_SPLIT'in davranışı değiştirdiği bir zorluk bölgesi ekleyin (parçaların ikisinin de düzenlendiği bölme, kısa parçalar, ≥%15 düzenlemeli birleştirme) ve THETA ızgarasını 0.50'nin altına genişletin. Seçilen değer şu anda sınırda ve bölme eşiği eşitlik kırmadan geliyor.
3. FREEZE_EXCEPTIONS prosedürüne "dondurma denetleyicisinin kendisi" sınıfını açıkça ekleyin; denetleyiciler ayarlanabilir derlenmiş sabitlere bağlı olmasın (T24'teki durum). Ayrıca `eval/freeze_check*.py` dosyalarını dondurma listesine resmen alın.
