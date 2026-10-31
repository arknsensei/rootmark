# P2-V — Bağımsız Doğrulama (kısa; `docs/rounds/P2-V/VERIFY.md` VV1–VV5; K-P2V.1)

Ayrı oturum, temiz klon (doğrulayıcının scratchpad'i), kod değiştirilmedi. Yerel koşturmalar o klonda; kademe 2 CI'da kendi tetiklememle.

**Doğrulanan commit: `44441df` (P2-V'nin birleşmiş `main` commit'i, PR #35 rebase ile; ağacı `a4d5d01` ile bayt bayt aynı: `git diff a4d5d01 44441df` boş).** Ölçümler PR #35'in başı `a4d5d01` (READY) üzerinde yapıldı, çünkü doğrulama başladığında PR birleşmemişti; rebase yalnız commit kimliklerini yeniden yazdı, içerik değişmedi. Ölçülen ürün kodu `ROUND_START` (`352f790`) ile aynıdır (VV4). Bu VERIFICATION `main`'den açılan ayrı bir dalda girer.

**Kademe 2 koşum: `41128634412`** (`gh workflow run verify.yml --ref p2-v/round -f sha=a4d5d01270e98dda2e68386cab990a882ab3f67d`; iş akışı `main`'de henüz eski sürümüyle durduğu için `--ref p2-v/round`).

## VV1 — Kademe 2'yi kendim tetikledim; D karşılaştırmasını kendi betiğimle yeniden yaptım
- **Koşu:** iki zincir yeşil (A 9 295 s, B 8 988 s; ikisi de `exit 0`, commit `a4d5d01`), karşılaştırma işi yeşil. `tier2.json`: `chains_wall_s` 9 397, `compare_s` 1 482, **`wall_s` 11 014 (3 sa 04 dk)**, `pass: true`. `tier2 pair`: 788 015 alan, D farkı 0, bayt farkı 0. `tier2 fails`: sayılan **0/0** (sınıflandırıcının sayımı 16/16). `eval.portable`: 133 dosya, iki zincirde 0 bulgu.
- **Dondurulmuş karşılaştırıcının kendi hükmü kırmızı** (17 alan + `p2_0_2/d_a/events.jsonl`): D-A yarışı yine çıktı (`d_a.json` satır 20: `new_blocks`, `pass_0a`, `unverified`) ve bunu taşıyan kayıtlar. Hepsi envanterin adıyla kenara alındı (`Z4`, `Z7`, `Z11`, `Z20`, `Z28`).
- **Kendi betiğim** (`research/eval`'e dayanmaz; iki RUN'ın bütün dosyaları, JSON yaprak yaprak, gerisi bayt bayt; envanterin girdileri yalnız dosya/alan önekiyle tam eşleşmeyle kenara): 1 342 dosya, 1 452 906 yaprak. Kenara alınmayan 608 yaprak farkı ve 12 bayt farkı var. Hepsini tek tek sınıfladım:
  - **Dondurulmuş karşılaştırıcının dosyalarında** (97 dosya): `h3_v3.json` gecikmeleri (409), `p1d1/d1_5`, `d1_6` ve `D1-6/measured` süreleri, `release.json` `targets›0›sha256` (`compare_p1e1.py:67–71`'in belgeli istisnası: ikilide derleme dizininin mutlak yolu), `p2a_checks.json`'ın beş alanı (kenara alınan alanları adlandıran `differing_fields` girdileri; envanterin gömülü fark kuralı). **Bunların dışında D farkı yok:** `tier2 pair`'in "D farkı 0" hükmünü bağımsız olarak doğruluyorum.
  - **Karşılaştırılmayan dosyalarda:** `manifest.json` `run_id`, `regress_vs_*` `run_b`, günlükler, `server.token`, `c8_latency.json`, `p2_0_2/d_b2.json` gecikmeleri ve günlükleri, `p2_0_3_checks.json` (D-A yarışının yansıması ve 0.3f gecikmesi). **Mutlak host yolları:** `conformance.json`, `conformance_full.json` ve `conformance_p1c1.json`'ın `fixtures_root`'u (`/home/runner/work/_temp/v/A/repo/spec/fixtures`), `p1d/inspector/*`. Bunlar çifte girmiyor, bu yüzden `eval.portable`'ın kapsamı dışında (aşağıda Ş2).
- **Gömme vektörleri bu koşuda A↔B eşit.** `p1c_index` vektör dosyalarında, C-10 `min_cosine`'da ve düz skorlarda fark yok. FINAL'in V4b'de gördüğü işlemci farkı burada çıkmadı. İki koşucu muhtemelen aynı işlemci ailesine düştü. `verify.yml` CPU modelini kaydetmediği için bunu doğrulayamadım (Ş1).
- **Karşılaştırma dışı liste = envanter:** `tier2_pair.json` `not_compared` 46 girdi. `inventory.json`'un `pair_exclusions` (40) + `byte_exclusions` (6) listesiyle birebir aynı (id, dosya, alan). Hiçbir girdide `*`, `?`, `[` yok. Çıktının "karşılaştırma dışı" satırı yalnız eşleşen 5 girdiyi adlandırıyor.

## VV2 — Hiçbir kriter kaybolmadı
- **V0'ın kriter kimliklerini kendim çıkardım.** V0 referansının (koşu `40582139980`, yapıt `verify-pair`, `352f790`) A RUN'ındaki her `*_checks.json` dosyasından, kendi tanımımla: üst düzey kimlikler ve `criteria` nesnesinin anahtarları. Sonuç **138**. Benim kademe 2 koşumda A'da 138, B'de 138 (A = B), Mac kümesinde 23. **Kaybolan 0.** Aracın tanımıyla (`historical.criteria`: `pass` taşıyan nesne) de 134 = envanterin 134'ü = `criteria_A.json`; kaybolan 0. İki tanım arasındaki 11 kimlik (`A-1`, `A-5`, `A-6`, `B-1/2/3/6/10`, `C-6`, `C-8`, `p2a_recorders`) `pass` alanı taşımıyor. Araç bunları kriter saymıyor; kayıtları çiftte bayt bayt karşılaştırılıyor. Eski zincirde de öyleydi, P2-V'nin getirdiği bir şey değil.
- **Z eşiklerini kaynaklarından kendim buldum.** `make verify-mac`'in uyguladığıyla aynılar:
  - D-10 p95 < 1000 ms ve katman doğru (`docs/rounds/P1-D/GOAL.md:48`; `verify_mac.py` `threshold_p95_ms: 1000`)
  - H-P2-0.2a = 0, a′ 200/200 (`P2-0.2/GOAL.md:31–32`); 0.3a = 0, 0.3a′ 200/200 (`P2-0.3/GOAL.md:17–18`)
  - H-P2-0.2d D-C ≥ 0.99 (`P2-0.2/GOAL.md:36`)
  - H-P2-0.2f ve 0.3f ≤ 0.01, p95 < 500 ms (`P2-0.2/GOAL.md:38`, `P2-0.3/GOAL.md:25`)

  Mac kümesi geçti/kaldı kararını dondurulmuş kaydedicilerin formülüyle veriyor. Örneğin 0.2a'nın formülü, sızıntının yanında `pass_0a`'yı da ister (`run_p2_0_2_checks.py:186–190`). "Eşik yok" dört küme (0.2i, 0.3i, `logs_vs_p2a`, 0.3b′) envanterde gerekçeleriyle yazılı. 0.2i/0.3i'nin GOAL'deki eşiği "iki zincirin günlükleri bayt eşit"tir; tek makinede tekrarlarla ölçülemediği için "eşik yok" sayılmaları savunulabilir.

## VV3 — Kasıtlı bozmalar (hepsi kopyalarda; gerçek RUN'lar dokunulmadı)
| | Bozma | Beklenen | Sonuç |
|---|---|---|---|
| (a) | B'nin kopyasında `p2r/det.json` `archived_traces›traces›0›at_seq` 0 → 1 | karşılaştırma kırmızı | `compare_p2r pair` çıkış 3, `tier2 pair --frozen` **çıkış 3**, D farkı tam olarak `p2r/det.json:archived_traces›traces›0›at_seq`. Kendi betiğim de tek ek fark olarak gördü (609 = 608 + 1). **Yakalandı.** |
| (b) | A'nın kopyasında `p2r/volume.json`'a zincirin iş dizini (`/home/runner/work/_temp/v/A/repo/target/x`) | taşınabilirlik kırmızı | `eval.portable --host host.json` **çıkış 3**, bulgu `p2r/volume.json` (`path`: `/home/runner`, `/home/runner/work/_temp`, …). **Yakalandı.** |
| (c) | Mac kümesinde Z-DA H-P2-0.2a′. Kontrol: 3. tekrarın üç kopyası → 3/3 geçti. Bozma: bir tekrarın `d_a.json` `scenarios_0a_prime` 200 → 199 | Mac kümesi kırmızı | `verify_mac rep` o tekrarda **FAIL**, `summary` 0.2a′ **2/3, pass false**, `mac_checks.pass` false. **Yakalandı.** |
| (d) | B'nin kopyasında `p2r_checks.json`'dan `R-1` silindi | kayıp tespit edilir | `tier2 equiv B B′ --mac mac_checks.json`: `criteria_old` 134, `criteria_new` 133, **`criteria_lost: [p2r_checks.json:R-1]`**, çıkış 3. **Yakalandı.** |

**Bozma dışı bir bulgu (gözden geçiricinin B1'i, bağımsız teyit):** `tier2 fails` D1-12 ve D-9'u "türev" diye kenara alıyor. Bunu sınıflandırıcının ilk 20 farkına bakarak yapıyor. Benim koşumda D1-12'nin tam fark listesi **12 046** girdi; bunların **6 032**'si adlı Z/O dosyalarının ve C1-8 kaydının dışında (`p1c_index.json` vektör dosyaları, `p1c_checks.json:C-10›min_cosine`; yayım Mac'ine bağlı referans değerler). Doğru bir korumayla D1-12 ve D-9 her iki zincirde sayılırdı. Bu koşunun "sayılan FAIL 0"ı da FINAL'deki gibi **geçerli değil.** Gerçek değer 2/2 olurdu.

## VV4 — Ürün kodu ve dondurma
- `git diff 352f790..a4d5d01 -- core index server bindings clients spec conformance` **boş**.
- **Uyumluluk:** yerel FAST çiftinde `fast/conformance.json` 39/39, seviye 3, C3-06 dahil. Kademe 2'nin `conformance_p1d1.json`'u iki zincirde 39/39, C3-06 dahil.
- `check_frozen`: OK (tablo `352f790`'dan beri yalnız ekleme).
- **Değişen ölçüm dosyaları ve istisnalar.** Değişen her ölçüm dosyası ya dondurmaya ilk kez `P2-V/V3:`'te giriyor ya da bir istisnaya bağlı:
  - `release_check.sh` F1, `verify.sh` F2 (ikisi de yalnız `43055ee`)
  - `tier2.py` F3 ve F5, `verify_mac.sh` F4, `portable.py` F6, `verify.yml` F7
  - `ci.yml` ve `Makefile` dondurma listesinde değil.

  **Önce/sonra kanıtlarını bağımsız denetledim:**
  - F3: iki sürümün AST'si eşit (`ast.dump`, kendi denetimim).
  - F1: diff yalnız R17'nin iki `detail` metni.
  - F2: diff yalnız `fast/conformance.json`'un `fixtures_root` göreli yazımı.
  - F4: diff yalnız `summary`'nin çıkış kodu yolu.
  - F5–F7: kanıt kaydı mevcut (`v4/local_recompare_V4a.json`). Benim koşumda `--frozen` yolu çalıştı ve kendi karşılaştırmamla tutarlı (VV1).
- **Envanter:** `inventory.json` ve `.md`'nin `git log`'unda yalnız `ae65686` (V1) var. Bu commit V2'den (`84b8d23`) önce; sonra değişmemiş.

## VV5 — Yerel kademeler, karar kuralı, vaat
- **`make verify FAST=1`** (temiz klon, `a4d5d01`): **1 038 s**, hızlı çift **18 550 alan eşit**. Kendi betiğimle A↔B farkı yalnız RUN kimliği (`manifest.json` ve `fast/c3_quality.log`'un ilk satırı).
- **`make verify-mac`** (`20261030T212809Z-a4d5d01`, 3 tekrar): **8 229 s** (kurulum 160 s), çıkış 3. Makine sessiz değildi (yük ~5,5).

  | Küme | Kriter | Tutan tekrar | Değerler / not |
  |---|---|---|---|
  | Z-D10 | D-10 | 0/3 | p95 3 217 / 1 865 / 1 666 ms; katman doğru, SIGTERM'de kayıp yok |
  | Z-DA | 0.2a, 0.3a | 0/3 | iki tekrarda harness durdu; 3. tekrarda sızıntı 0, `pass_0a` tutmadı (D-A yarışı) |
  | Z-DA | a′ | 1/3 | |
  | Z-DC | 0.2d | 0/3 | 3. tekrarda D-C 0.735, ajan `canonical` 3 |
  | Z-DB2 | 0.2f, 0.3f | 0/3 | |
  | O-DB4, O-DC′, O-DA′ | gerçek Obsidian kümeleri | 0/3 | `run_p2_0_3` üç tekrarda durdu |
  | O-REL | E-7, E1-4 | 0/3 | kararla kapanmış kalışlar: K-E5-T 2/4, R4 |
  | O-REL | E1-3, E1-9 | 3/3 | |
  | O-REL | tamper | 2/3 | |
  | O-C18 | C1-8 | 3/3 | |

  Harness'lerin durma nedenleri `mac_checks.json`'da yok (`log_tail` boş); benim günlüğümde iki tanesi var:
  - `D-C: the human's write of agent-c/c083.md not ingested within 15.0 s`
  - `rootmark attest …glossary.md: HTTP 403 … is not a rename of the content of …`

  Tablo FINAL'in V-6 tablosuyla aynı aileyi gösteriyor.
- **Karar kuralını GOAL'den kendim türettim.** FINAL'in değerleri ve benim ölçümlerim:

  | Kod | Geçti/kaldı | Gerekçe |
  |---|---|---|
  | V-1 | **geçti** | 26 adım, sınıflı kayıtlar, Z/O kaynak eşikli ya da "eşik yok"; tek commit `ae65686`, V2'den önce |
  | V-2 | **geçti** | diff boş; 39/39, C3-06 |
  | V-3 | **kaldı** | FINAL'in V4b'sinde adı geçmeyen D farkları: işlemciye bağlı vektörler + R14. Benim koşumda vektörler eşit; ama eşitlik koşucu şansına bağlı, kriteri kurtarmaz |
  | V-4 | **kaldı** | Süre: FINAL'de 9 786 / 10 582 s, benim koşumda 11 014 s, üçü de > 9 000 s. Sayılan FAIL: türev koruması kırpılmış listeye bakıyor; doğru korumayla 2/2 (VV3 notu) |
  | V-5 | **geçti** | Kapsamı "çifte giren kayıtlar"; benim koşumda 0/0; enjeksiyon yakalandı. Not Ş2 |
  | V-6 | **kaldı** | Z kümeleri 0/3; süre 8 229 s > 1 800 s |
  | V-7 | **kaldı** | PR koşusu 40638224562: 2 068 s > 1 800 s. CI kaydı: 05:28:42 → 06:03:19. Yerel FAST 1 038 s, kriterin ölçtüğü CI süresi değil |
  | V-8 | **geçti** | Belgeler, F1–F7 kanıtlı, `check_frozen` OK, gözden geçirici KOŞULLU KABUL ve koşulları kapandı |

  Dal: V-3 kaldı → "V-2, V-3 ya da V-5 kaldı → tur kaldı; eski zincir yürürlükte kalır; karar talebi". V-4 ve V-7 kaldı → "tur kaldı; karar talebi". V-6'nın özel dalı uygulanmaz, çünkü geri kalan geçmedi. **FINAL'in türettiği dalla aynı.** Kademeler bilgidir; blok sonunun hükmü eski tam çifttir. K-P2V-T1..T4 açık.
- **Vaat sorusu: yeni kademelerle "iki bağımsız klon aynı sonucu üretir" iddiası eskisi kadar güçlü mü?** İki bakımdan **daha güçlü:** iki zincir artık ayrı makinelerde koşuyor, eskisi gibi tek makinede sırayla değil; kayıtlardaki host yolları denetleniyor. Eski çift, işlemciye bağlı vektörleri hiç göremezdi; yenisi gördü (V4b). Dört yerde **zayıf ya da açık:**
  1. Kademe 2'nin yeşili koşucuların aynı işlemciye düşmesine bağlı. Benim koşumda vektörler eşitti, V4b'de değildi. NORTH_STAR ilke 2'nin vektör baytları için bugün doğru olmadığı bulgusu (K-P2V-T1) duruyor.
  2. "Sayılan FAIL 0" türev korumasının kırpılmış listesine dayanıyor (B1).
  3. Z/O kümeleri bayt eşitliğinden çıktı ve Mac kümesinde ölçülüyor. Ama Mac kümesi bu makinede bunların hiçbirini eşiğinde tutamıyor. Yani bu kümelerin determinizm iddiası bugün hiçbir kademede "geçti" ile desteklenmiyor. Ölçülüyor ve kaydediliyor, ama tutmuyor.
  4. Eski çiftten devralınan bir kör nokta: `p2_0_3_checks.json` ve `p1a_checks.json` hiçbir karşılaştırmaya girmiyor; bunlar kriterlerin deterministik yarılarını da taşıyor (`rules`, `seed`, `p2a_recorders`).

  Bu eksiklerin hiçbiri FINAL'de gizlenmemiş: hepsi ya FINAL'de, ya gözden geçiricide, ya da karar taleplerinde yazılı. 4. madde yalnız burada.

## Sonuç
**İhlal:** yok. Ürün kodu, spec ve uyumluluk değişmedi. Dondurulmuş dosyalar istisnalı. Envanter ölçümden önce dondurulmuş. Hiçbir kalış optimize edilmemiş.

**Fark / şüphe:**
- **Ş1:** Kademe 2'nin sonucu koşucunun işlemcisine bağlı ve `verify.yml` CPU modelini kaydetmiyor. Yeşil bir koşunun aynı işlemci ailesinde mi koştuğu yapıttan anlaşılamıyor.
- **Ş2:** Çifte girmeyen kayıtlarda mutlak host yolları kalıyor (`conformance.json`, `conformance_full.json`, `conformance_p1c1.json` `fixtures_root`; `p1d/inspector/*`). V-5'in lafzı karşılanıyor, GOAL'ün "kayıtlarda host'a özgü değer yok" ilkesi tümüyle karşılanmıyor.
- **Ş3:** `mac_checks.json` harness durmalarının nedenini taşımıyor (`log_tail` boş). Ayrıca kararı veren alanı (`pass_0a`) değerlerin arasında göstermiyor: 0.2a "sızıntı 0, kaldı" diye okunuyor.
- **Ş4:** Türev koruması (B1) bu koşuda da sayımı sıfırlıyor (VV3 notu).

**Kriter başına:** V-1 geçti · V-2 geçti · V-3 kaldı · V-4 kaldı · V-5 geçti · V-6 kaldı · V-7 kaldı · V-8 geçti. **Tur kaldı; FINAL'in dalı doğru.**

**Hüküm: şu koşullarla güvenilir.** FINAL'in ölçümleri, değerleri ve karar kuralının dalı bağımsız olarak doğrulandı. Kademelerin araçları kasıtlı bozmaların dördünü de yakaladı. Hüküm şu iki koşula bağlı:
1. Kademeler yürürlüğe girmeden önce türev koruması tam listeye bakacak biçimde düzeltilmeli (K-P2V-T4 B1).
2. İşlemciye bağlı vektörler için karar verilmeli (K-P2V-T1).

Bu iki koşul kapanmadan kademe 2'nin "yeşil"i bir blok hükmü olarak kullanılamaz; yalnız bilgi olarak kullanılabilir. FINAL de bunu söylüyor.

**Öneriler (≤ 3):**
1. **K-P2V-T4'ü kademeler yürürlüğe girmeden kapatın**, istisna prosedürü ve yeni envanterle:
   - B1: koruma kriterin kendi kaydındaki tam regress listesine baksın.
   - B3: p1e1 tamper'ın deterministik yarısı kendi kriteri olsun.
   - Ayrıca `p2_0_3_checks.json`'ın deterministik alanları çifte alınsın.
2. **K-P2V-T1 (a)'yı P2-C'den önce yapın:** işlemciden bağımsız gömme sayısalları. Köprü olarak `verify.yml` her zincirde CPU modelini (`/proc/cpuinfo` `model name`) `host.json`'a yazsın; karşılaştırma iş iki model farklıysa bunu hükümle birlikte açıkça bildirsin.
3. **Mac kümesini ve host denetimini tamamlayın:**
   - `mac_checks.json` harness hatasının son satırlarını ve kararı veren alanları taşısın.
   - `eval.portable` çifte girmeyen kayıtları da tarasın; `conformance*.json` `fixtures_root` göreli yazılsın.
   - Mac kümesi K-P2V-T2 (a) uyarınca sessiz makinede bir kez koşulsun. Bu doğrulamada makine de sessiz değildi (yük ~5,5).
