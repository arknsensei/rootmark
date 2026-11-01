# P2-V.1 — Bağımsız Doğrulama (dar kapsam; `docs/rounds/P2-V.1/VERIFY.md` VW1–VW5)

Ayrı oturum, kendi scratchpad'imde temiz klon (`git clone https://github.com/arknsensei/rootmark.git`); ana çalışma dizinine dokunulmadı, kod değiştirilmedi. Tarih 2026-11-01.

**Doğrulanan commit: `11c8dfb`** — P2-V.1'in birleşmiş `main` commit'i (PR #39, rebase birleştirmesi; ağacı `210a7f9`, PR head'i `695f952`'ninkiyle **birebir eşit**, `git rev-parse <sha>^{tree}`). Ölçümler ve CI tetiklemeleri PR birleşmeden önce `695f952` üzerinde yapıldı (aynı ağaç); ölçülen ürün/araç kodu `58377de` ile aynı — `git diff 58377de 695f952` yalnız `reports/`, `docs/` ve kök `CLAUDE.md`'nin yürürlük notu.

Kendi tetiklediğim CI koşuları (araçlar `p2-v.1/round`'un, yani doğrulanan commit'in; `--ref main` P2-V'nin eski araçlarını kullanırdı):
- **Kademe 2: `41292205176`** (`gh workflow run verify.yml --ref p2-v.1/round -f sha=695f952…`).
- **E2 + kademe 1: `41292207466`** (`gh workflow run ci.yml --ref p2-v.1/round`; E2'nin üç `embed-cpu` işi, FAST çifti, test işleri).

## VW1 — İşlemciler arası

**E2'yi kendim tetikledim (`41292207466`):** üç koşucu — AMD EPYC 7763 (x86_64 Linux), Neoverse-N2 (aarch64 Linux), Apple M1 (Virtual) (macOS arm64; `macos-15`). CPU modelleri yapıtın `cpu.json`'larından okundu ve üçü farklı. Yapıtları indirip **kendi betiğimle** (bütün dosyalar, sha256 + bayt) karşılaştırdım: iki modelin `vectors.f32` (973 824 bayt), `embedders.jsonl`, `manifest.json`, düz kip skorları (`flat.jsonl`), `*.build.json` ve `corpus.txt` — **11 dosyanın hepsi üç koşucuda bayt eşit**. Farklı olanlar yalnız `cpu.json` ve onun özetini taşıyan `hashes.json` (beklenen). CI'nın `e2_compare.json`'ı da aynı hükmü verir (`equal: true`, `processors_differ: true`).

**Beşinci işlemci (yerel):** aynı commit'in `scripts/embed_cpu.sh`'ını bu makinede (Apple M4 Max, SME2; ORT'nin başka çekirdek seçtiği işlemci) koştum: iki modelin 8 vektör/türev dosyası CI'nın x86_64 çıktısıyla **bayt eşit** (`cmp`; 51 s).

**Kademe 2'de işlemciler arası:** kendi tetiklemem `41292205176`'da zincirler **farklı işlemcilere** düştü — A: AMD EPYC 9V45, B: AMD EPYC 7763 (`host.json`, `tier2.json` `same_processor: false`). Bu, turun kendi kademe 2 koşularında olmayan bir kanıttır (ikisinde de dört zincir EPYC 7763'tü, FINAL F3). Bütün zincirde (P1-C D3 dizini, p1d vault'ları, sıcak katman) 6 `.f32` dosyasının hepsi A↔B bayt eşit; D farkı 0 (VW2). P2-V'de aynı durumda 209 281 alan farklıydı.

**E0'ın ham sayılarından yeniden hesap** (`reports/rounds/P2-V.1/e0/feasibility.json`, kendi betiğimle):
- Ölçek: 6 063 parça / 401 doküman = 15,1197 parça/doküman; 10⁵ doküman × 15,1197 × 384 = **580 596 508** bileşen (dosyadaki değerle aynı).
- ORT, Xeon ↔ EPYC (minilm): ortalama |δ| 1,6988e-8 → beklenen çarpışma k = 10'da **10 100**, k = 20'de **10 342 417**; öteki x86↔ARM çiftleri 10 891–10 907 (k = 10). Tablodaki değerlerle aynı. Yuvarlama (y) hiçbir adımda gözlenen 0 vermiyor (en az 34 minilm, 29 multilingual) ve beklenen ≥ 10⁴ → V1-1'i geçemez.
- (r) `detbert`: iki modelde, 6 çiftin hepsinde farklı bileşen 0, ortalama |δ| 0 → beklenen **0 < 1**.
- ORT'nin Neoverse ↔ M1 çifti 0 fark verir (ARM'lar aynı çekirdek); x86 bağımsızlığı yalnız (r) ile sağlanıyor. Seçimin gerekçesi sayılardan yeniden türetilebiliyor.
- Sıra: seçim `7955b6e` (E0) → embedder'a bağlama `f25cbaa` (E1); E0 ölçüm commit'i `3fdab2e`. **V1-1'in "E1'den önce commit'li" yarısı tutuyor.**

**VW3 (d) burada:** M1 koşucusunun `minilm-l6-v2/vectors.f32`'sinde tek bileşenin (38 417.) son mantis bitini çevirdim: `e2_compare.py` önce `hashes.json does not match its files` ile çıkış 1 verdi (bütünlük denetimi); `hashes.json`'ı tutarlı yeniden yazınca **çıkış 3, "DIFFERENT (1 files)"**, farklı dosya adıyla. Kendi betiğim de aynı dosyayı gösterdi. **Yakalandı.**

## VW2 — Kademe 2 (`41292205176`)

- Zincir çıkışları **0 / 0**, ikisi de `695f952`. Duvar **9 085 s** (zincirler 8 801 s, karşılaştırma 144 s) ≤ 11 700 s.
- Karşılaştırma işi **kırmızı** (`pass: false`), FINAL'in iki koşusuyla aynı nedenlerle: `tier2 pair` **D farkı 0** (788 913 alan, `equal: true`); `tier2 fails` **sayılan 1 / 1** (`p1e1_checks.json:tamper·det`, `tamper_p1e1.json pass false`); `eval.portable` A ve B'de bulgu (`conformance.json` `fixtures_root` = `/home/runner/work/_temp/v/<A|B>/repo/spec/fixtures`). D-9 ve D1-12 tam listeleriyle türev (18 056 / 18 058 fark).
- **Kendi A↔B karşılaştırmam** (`compare_p2r`/`tier2`'yi kullanmadan): iki RUN'ın 1 342 dosyasının hepsi, JSON'lar yaprak yaprak (1 460 895 yaprak), ötekiler bayt bayt; envanterin adları birebir uygulandı (desen yok; gömülü fark kuralı dahil). Dondurulmuş karşılaştırmanın kapsamında (97 dosya + 33 bayt kalemi) envanterin adlarıyla açıklanmayan farklar yalnız şunlar ve hepsi karşılaştırıcıların kendi belgeli duvar saati/ortam alanları (`research/eval/DETERMINISTIC_FIELDS.md`): `h3_v3.json` `H3-1`, `latency/latencies_ms`, `summary/p50|p95`; `p1d1/d1_5.json`/`d1_6.json` `elapsed_*`, `ratio`; `p1d1_checks.json` `D1-6/measured`; `p1e1/release/release.json` `targets/0/sha256` (`P1E1_INFO`). **D farkı 0 — CI'nın hükmüyle aynı.** Kenara alınanlar Z1, Z3, Z4, Z7, Z10, Z11, Z20 (gömülü dahil) ve bayt Z28 (`p2_0_2/d_a/events.jsonl`). Kapsam dışındaki farklar (günlükler, `server.token`, `manifest.json` `run_id`, `c8_latency.json`, `regress_vs_*` `run_b`, `p2_0_2/d_b2.json` gecikmeleri, `conformance.json` `fixtures_root`, `p2_0_3_checks.json`) bilgidir; vektör türevli tek bir fark yok.
- **Karşılaştırma dışı liste = envanter:** `tier2_pair.json` `not_compared` 48 kimlik = `inventory.json` `pair_exclusions` (40) + `byte_exclusions` (8), küme eşit; hiçbir değerde `*`, `?`, `[` yok. Tek içerik farkı O13: envanterde madde kimliği `R15` (B6), çıktıda çözülmüş konumu `items›15` — iki zincirin `release_check.json`'ında R15 gerçekten 15. sırada (R4a 3, R7 7, R17 17). Tutarlı.
- **`host.json` ↔ hüküm:** A `AMD EPYC 9V45 96-Core Processor`, B `AMD EPYC 7763 64-Core Processor`; `tier2.json` ikisini ve `same_processor: false`'u yazıyor; karşılaştırma günlüğü "different". Tutarlı.
- **Karşılaştırıcının önce/sonra eşitliği (F6, V1-7'nin son yarısı):** aynı girdide (bu koşunun iki RUN'ı) `ROUND_START`'ın `compare_p2r pair`'i **749 s**, `695f952`'ninki **7 s**; çıktı JSON'u ve stderr **bayt eşit** (`cmp`).

## VW3 — Kasıtlı bozmalar (bu koşunun RUN kopyaları üzerinde; sınıflandırıcı `eval.historical_p2r` bozulmuş kopyada yeniden koşuldu, sonra `eval.tier2`)

Taban (bozmasız A): sayılan 1 (`tamper·det`) — CI ile aynı.
- **(a) B1 — D1-12'nin tam listesinde Z/O dışı girdi.** D1-12'nin `regress_vs_p1d.differing_fields`'ına `p1e_checks.json:E-2›pass` (a true, b false) eklendi; sınıflandırıcının kırpılmış 20'lik listesinde **yok** (doğrulandı), tam listede var. `tier2 fails`: **D1-12 sayıldı** (`n_not_set_aside` 2, tam olarak eklenen iki yaprak). İlk denememde seçtiğim ad (`p1b_checks…`) sıralamada ilk 20'ye düştü; B1'i sınamadığı için ikinci ad kullanıldı. **Yakalandı.**
- **(b) B2 — C1-8'in sıralama kimliği.** Çift: B'nin `p1c1_checks.json` C1-8 kaydına `rankings_200›flat›0›0›1` (chunk kimliği) farkı eklendi → `tier2 pair` **çıkış 3, `equal: false`**, iki yaprak. Türev kuralı: D1-12'nin gömdüğü C1-8 kaydına aynı kimlik yaprağı eklendi → **D1-12 sayıldı**; aynı kayıttaki skor yaprağı (`…›0›0›3`) zaten vardı ve kenarda kaldı (C1-8'in adları yalnız skorları örtüyor). **Yakalandı.**
- **(c) B3 — p1e1 tamper'ın deterministik yarısı.** `tamper_p1e1.json` `pass` → true: sayılan **0** (yeni kimlik gerçekten bu alanı okuyor); geri false: sayılan 1. Ama `pass` bu zincirde zaten **false** (iki vaka beklenmedik: "p1e-final server" ve "fixture regenerated with the p1d-final mdk"; FINAL ve K-P2V1-T3 ile aynı). Kontrol vakasını bozdum (`cases[0]` accepted → rejected, `controls_ok` 1 → 0, `pass` false kaldı): `tier2 fails` **değişmedi** (yine yalnız `tamper·det`, aynı gerekçe) — bozma `fails`'te görünmez; yalnız çift yakalar (`tier2 pair` çıkış 3, üç yaprak). **Kısmen yakalandı** (bkz. Ş2).
- **(d) Vektör bileşeni → E2 kırmızı.** VW1'de. **Yakalandı.**

## VW4 — Ürün sınırı ve kalite

- `git diff 749a4a5..695f952 -- core bindings clients spec` **boş**. `server/` farkı: `src/migrate.rs` (eski ORT `embedder_id` tablosu `OLD_EMBEDDER_IDS`, `index_embedder_old`, `index_rebuild`'in uygulanma koşulu ve matris notu), `src/tools.rs` (`index_readable`'ın göç ipucu), `tests/p2v1_migrate.rs` ve `tests/fixtures/index_ort/` — yalnız göç tanıma. Ürünün öteki değişikliği `index/` (`detbert.rs`, `embed.rs`, `models.rs`, `store.rs`) ve `models/MANIFEST*.json` `embedder_id_rule` (`+ "+fixed1"`); `Cargo.lock`'tan `ort`/`ndarray` ağacı çıktı.
- `make conformance`: **level 3, 39/39** (yerel). `cargo test --release` `p2v1_migrate` 1/1, `p2v1_embed` 4/4 (ilk denemem klonda model dosyaları olmadığı için durdu; `make models models-multilingual` sonrası geçti).
- **Kalite (zincir kayıtlarından, kendi koşumda; P1-C resmî RUN'ı `20261015T014417Z-9aa326b` ile kendi betiğimle):** C-10 `min_cosine` **0,9999999999992675** (≥ 0,999; iki zincirde aynı). `rankings_200` 200 sorgu × 3 kip = 600 sıralamada `(doc_path, chunk_id, layer)` dizisi değişen **0**; en büyük |Δskor| **5,5e-7**. `c3_quality.json` 4 838 yaprağın yalnız biri farklı: `index/embedder_id` (`…/e73168f9` → `…/e73168f9+fixed1`). C1-1 ve C1-9 geçti. C-3 ve C-11 `pass: false` — sınıflandırıcıda **tarihsel** ("P2-0.2 resmî RUN'ında, kaydı değişmemiş"), değerleri P1-C ile aynı. C-8 p95: A (EPYC 9V45) **35,8 ms**, B (EPYC 7763) **49,7 ms** (< 100).
- **Kendi göç senaryom** (`fixtures/index_ort`'tan bağımsız): `ROUND_START`'ın (`749a4a5`) ikilisini ayrı `CARGO_TARGET_DIR`'da derledim; `research/data/d_notes`'tan 23 not ile vault kurdum (`ingest-dir`, `index build`; `embedder_id` `minilm-l6-v2/e73168f9`). Yeni ikili: `index status` ve `search` **"index embedder_id … holds an older rootmark's vectors (ONNX Runtime numerics, not "fixed1"); rebuild it"** hatası; `migrate --dry-run` hiçbir baytı değiştirmedi; `migrate` üç göç uyguladı (`index_rebuild` 13 dosya, `log_readers` ve `checkpoints` 0 dosya), `events_hash` aynı; göçmüş indeks aynı notlardan **taze kurulan indeksle bayt eşit** (`diff -r`); üç göçü ters sırayla geri alınca vault `.rootmark/migrations/` kayıtları dışında **göç öncesiyle bayt eşit**, `events_hash` aynı. (İlk denemede geri almayı yalnız son göç için yaptığım için fark gördüm; göçlerin ters sırayla geri alınması gerekiyor — ürün bulgusu değil.)

## VW5 — Yerel kademeler ve karar kuralı

- **`make verify FAST=1`** (yerel M4 Max): çıkış 0, **1 253 s**, çift eşit (18 550 alan). Koşu sırasında aynı makinede derleme ve testler de çalıştı (süre bilgi).
- **Kademe 1 CI (`41292207466`):** bütün işler yeşil; iş akışı **33 dk 48 s** (FAST zincirleri 32:42 / 24:32, karşılaştırma 1:00) ≤ 40 dk; E2 3/3.
- **`make verify-mac`:** 1. koşu **çıkış 2, 3 678 s, `mac_checks.json` yazılmadı**: 1. tekrarın `serve_watch`'ında D-10 testi "edit 0 of espresso.md not searchable after 10 s" ile kaydını (`d10.json`) yazmadan çöktü; `scripts/verify_mac.sh:104` eksik kayıtta kümeyi durdurur. O sırada `fseventsd` %100 CPU'daydı (bu makinede bilinen yük). 2. koşu: çıkış 0, **7 279 s** (kurulum 161 s; tekrarlar 3 354 / 1 957 / 1 803 s), aracın hükmü `pass`: C1-8 1/1; E-7, E1-4, H-P2-0.3b tarihsel kayıt eşit; E1-3, E1-9, tamper e2e 3/3; Z kümeleri 0/3 (D-10 p95 4 322 / 2 823 / 4 296 ms, eşik 1 000; P2-0.2/0.3 kümeleri harness'in `attest` 403 "denied" durmasıyla); **tarihsel olmayan gerçek Obsidian O kriterleri H-P2-0.3b-yük, 0.3c, 0.3d, 0.3e 0/3** (aynı harness durması; Ş3'ün `log_tail`'i hatayı taşıyor). FINAL'deki desenle aynı.
- **Kendi tetiklediğim ve FINAL'in kayıtlarıyla eski ↔ yeni (V1-6):** P2-V V0m (`40613392932`, `352f790`) A zinciri ↔ kendi kademe 2'min A'sı, `eval.tier2 equiv`: kriterler **135 ⊆ 135**, kaybolan 0, D bayt farkı 0; adlı farklar E1 216 975, Z/O 93, H 2, commit 3; **adı geçmeyen 1: `p1e1/release/release_check.json:items›7›detail`** (R7, tekerlek adı `…-manylinux_2_34_x86_64.whl`; iki zincirimde de bu ad). FINAL ve K-P2V1-T2 ile aynı.
- **V1-11 denetimleri (yerel):** `check_frozen` OK (tablo `749a4a5`'ten beri yalnız eklenmiş), `check_deps` OK, `check_dep_age` 133 bağımlılık ≥ 14 gün; `reports/rounds/P2-V/inventory.json` turda değişmedi; P2-V.1 envanteri yalnız `0a62484`'te yazıldı, araçlar (`febd3a7`, `5978419`) ondan sonra ve envanter sonra değişmedi.

### Kriterler (bağımsız okuma)
| Kod | Sonuç | Dayanak (bu doğrulamanın) |
|---|---|---|
| V1-1 | **geçti** | E0 sayıları yeniden hesaplandı (gözlenen 0, beklenen 0 < 1; (y) ≥ 10⁴); seçim `7955b6e` < E1 `f25cbaa` |
| V1-2 | **geçti** | E2 `41292207466` 3/3 bayt eşit (kendi betiğim); yerel M4 Max eşit; kademe 2'de EPYC 9V45 ↔ 7763 eşit; PR koşusu `41252646332` (FINAL) yeşil |
| V1-3 | **geçti** | C-10 0,99999999999927; 600 sıralamada değişiklik 0; C-3 yalnız `embedder_id`; C1-1, C1-9 geçti; C-3/C-11 tarihsel |
| V1-4 | **geçti** | Kendi vault'umla `ROUND_START` ikilisi → bayat tanıma → göç → taze kurulumla bayt eşit → geri alma bayt eşit; `p2v1_migrate` geçti |
| V1-5 | **geçti** | `core bindings clients spec` farkı boş; `server/` yalnız göç tanıma; conformance 39/39; `events_hash` göçte ve geri almada aynı |
| V1-6 | **kaldı** | Adı geçmeyen D farkı 1 (R7 `detail`); kriterler 135 ⊆ 135 |
| V1-7 | **kaldı** | Sayılan FAIL 1 (`tamper·det`); D farkı 0, çıkışlar 0, CPU kayıtlı, 9 085 s ≤ 11 700, karşılaştırıcı önce/sonra bayt eşit (süre yarısı geçti) |
| V1-8 | **kaldı** | `portable --all` iki zincirde bulgu (`conformance.json` `fixtures_root`); B1, B2 kasıtlı bozmayla yakalandı; B3 yalnız `pass` alanını okuyor (Ş2); B6 madde çözümü doğru |
| V1-9 | **geçti** | `41292207466` yeşil, 33 dk 48 s; E2 3/3 |
| V1-10 | **kaldı** | Tarihsel olmayan gerçek Obsidian O kriterleri 0/3 iken aracın hükmü `pass`; bunu veren karar yok (K-P2V1-T5); ayrıca 1. koşu D-10'da tamamen durdu (Ş1) |
| V1-11 | **geçti** | Denetleyiciler temiz; envanter sırası ve değişmezliği; F6 önce/sonra kendim teyit ettim; yürürlük notu `CLAUDE.md` ve DEV_PROCESS §3'te; gözden geçirici kaydı (`E5.review.md`) 2. tur KABUL |

### Karar kuralı (GOAL'den bağımsız türetim)
V1-1 ve V1-2 geçti → "V1-1 ya da V1-2 kaldı" dalı uygulanmaz (E1 bu dalla engellenmez). V1-3 geçti. **V1-6 kaldı → "V1-4, V1-5 ya da V1-6 kaldı → tur kaldı; eski zincir yürürlükte; karar talebi."** V1-7'nin sayılan FAIL yarısı, V1-8 ve V1-10 kaldı → "V1-7'nin geri kalanı, V1-8, V1-9, V1-10, V1-11 kaldı → tur kaldı; karar talebi." V1-7'nin süre yarısı geçti. **Sonuç: tur kaldı, eski tam çift yürürlükte, kademeler yürürlükte değil — FINAL ile aynı dal, aynı dört kriter.** Karar talepleri K-P2V1-T2..T5 bu dört kalışı adlarıyla karşılıyor.

### Vaat sorusu (NORTH_STAR ilke 2, "her makinede aynı sonuç")
**Vektör indeksi için artık doğru**, gözlenebildiği kadarıyla: bu doğrulamada beş işlemcide (EPYC 7763, EPYC 9V45, Neoverse-N2, Apple M1, yerel M4 Max), turun kendi kayıtlarında ek olarak Xeon 8370C ve Xeon 6973P-C'de; x86_64 (AVX2+FMA) ve iki ARM ailesinde; iki modelde vektör ve düz kip skoru baytları eşit. Yöntem yuvarlamaya değil işlem sırasının tanımına dayanıyor (|δ| = 0, şans sıfırı değil). **Doğru olmadığı bir yer kalıyor:** envanterin C1-8 adlarına göre sıcak katmanın dört sözcüksel skor yaprağı son ULP'de platform `libm`'ine bağlı (macOS ↔ glibc; Linux koşucuları arasında eşit) — sözcüksel skorlar işletim sistemleri arasında bayt eşit değil. Bunu kendim ölçmedim (iki zincirim de Linux); E1'in değil P2-V'den beri var olan bir durum ve GOAL bunu ölçmüyor. Bunun dışında NORTH_STAR ilke 8 ("yüz binlerce doküman") için sabit sıralı gömmenin bedeli ölçülmüş ama hükme girmemiş: M4 Max'te ORT'nin ~5 katı; göç her eski indeks için tam yeniden gömmedir (FINAL K-P2V1-B1).

## İhlal / fark / şüphe
- **İhlal yok.** Ürün sınırı, dondurma, envanter sırası, karar kuralının dalı FINAL'deki gibi; FINAL'deki her sayı kendi koşumda aynı ya da aynı sınıfta yeniden üretildi (Z/O değerleri doğası gereği farklı).
- **Ş1 (yeni): Mac kümesinin harness'i D-10 testinin çökmesinde bütün kümeyi kaybediyor.** `serve_watch.rs:161`'in 10 s koruması kaydı yazmadan panikliyor; `verify_mac.sh:104` eksik kayıtta `exit 1` — 3 678 s sonra `mac_checks.json` yok. "Harness durması başarısız tekrar sayılır" ilkesi bu yolda uygulanmıyor. Yavaşlayan gömme (detbert) izleyici gecikmelerine ek baskıdır; ilk koşuda `fseventsd` %100'deydi. İkinci koşu tamamlandı.
- **Ş2 (yeni): `tamper·det` sabit FAIL, deterministik bozmaları maskeliyor.** Kriterin değeri P2-B'den beri Linux'ta false; bu yüzden kademe 2 her tetiklemede kırmızı ve `pass`'i false bırakan bir bozma `fails`'te yeni sinyal üretmiyor (yalnız çift yakalıyor). K-P2V1-T3'ün (a) seçeneği (tarihsel/H) bu maskeyi kalıcı kılar.
- **Fark:** kendi kademe 2'mde zincirler farklı işlemcilere düştü (turun koşularında düşmemişti) — işlemciler arası kanıtı kademe 2 içinde de sağlıyor; bilgi.
- **Şüphe yok** ürün iddiasında (determinizm, kalite, göç).

## Hüküm: **şu koşullarla güvenilir**
Turun raporu (FINAL) ölçümleri, değerleri ve karar kuralının dalı bakımından bağımsız olarak doğrulandı; ürün iddiası (işlemciden bağımsız gömme, kalite aynı, göç) turun kendi kanıtından daha geniş bir işlemci kümesinde tuttu; kasıtlı bozmaların dördü yakalandı (c kısmen). Koşullar: (1) K-P2V1-T2..T5 kararlaştırılmadan kademeler yürürlüğe girmez (FINAL'in dalı); (2) Mac kümesinin D-10 yolunda kayıt kaybı (Ş1) kademeler yürürlüğe girmeden giderilir ya da kararla kabul edilir.

## Öneriler (≤ 3)
1. **K-P2V1-T3'te (b) ya da (c), yalnız (a) değil:** `tamper·det`'in nedenini (Linux'ta "p1d-final mdk ile yeniden üretilen fixture" kontrolü) gidermek kademe 2'yi yeşile döndürür ve deterministik yarıyı gerçek bir dedektör yapar (Ş2); T2 ve T4 için (a) yeni ölçüm gerektirmeden yeterli görünüyor.
2. **Mac kümesi harness'i (§4 istisnasıyla):** `serve_watch`'ın D-10 ve SIGTERM durumları zaman aşımında da kayıt yazsın (ölçülmüş gecikmeler + "zaman aşımı" kararı) ya da `verify_mac.sh` eksik `d10.json`'ı o tekrarın başarısız sonucu olarak kaydetsin; K-P2V1-T5 ile birlikte karara bağlansın (Ş1).
3. **P2-C'de ön-kayıtlı hız/ölçek eşiği ve NORTH_STAR notu:** sabit sıralı gömme M4 Max'te ORT'nin ~5 katı; ilk kurulum ve her göç tam yeniden gömme — ilke 8 için eşik ve SIMD/bloklama ya da doküman başına paralellik (her vektör tek iş parçacığında, sıra sabit) ölçülsün; NORTH_STAR ilke 2'ye "vektör baytları işlemciden bağımsız; sözcüksel skorların son ULP'si platform `libm`'ine bağlı" notu insan kararıyla eklensin.

## Koşu ve kayıt kimlikleri
Kademe 2 (kendi) **`41292205176`** (A EPYC 9V45, B EPYC 7763; 9 085 s); E2 + kademe 1 (kendi) **`41292207466`** (33 dk 48 s); eski zincir referansı V0m `40613392932`; yerel FAST RUN'ları `20261101T091804Z-695f952` / `20261101T091805Z-695f952`; Mac kümesi RUN `20261101T103947Z-695f952` (2. koşu; 1. koşu `20261101T093801Z-695f952` kayıtsız durdu); P1-C resmî RUN `20261015T014417Z-9aa326b`.
