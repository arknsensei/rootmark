# P2-V.1 — Görevler

Genel kurallar:
- Sıra: E0 → E1 → E2 → E3 → E4 → E5. E0'ın seçimi commit'lenmeden E1'e, E3'ün envanteri commit'lenmeden E3'ün araç değişikliklerine geçilmez.
- Progress `docs/rounds/P2-V.1/progress/`. Commit öneki `P2-V.1/E<n>:`.
- Dal `p2-v.1/round`, yalnız birleşmiş `main`'den (K-P2.2). `ROUND_START` = bu turu açan PR'ın birleşmiş `main` HEAD'i.
- Yama turu, geliştirme dozu (K-P2.4): gözden geçirici E1 sonunda (gömme değişikliği; başlık "kalite ve determinizm") ve E5'te (tek tur; başlık "ölçülen şey gevşedi mi").
- Uzun koşular CI'da: tam çift yalnız `verify.yml` ile; yerelde tam `make verify` koşulmaz. Yerelde yalnız FAST ve Mac kümesi.
- Dondurulmuş kodda her değişiklik `docs/rounds/P2-V.1/progress/FREEZE_EXCEPTIONS.md`'ye önce/sonra kanıtıyla (DEV_PROCESS §4; başlıklar `## F<n> `).
- CI tetiklemeleri `gh workflow run <iş akışı> --ref p2-v.1/round [-f sha=<sha>]`; yapıtlar `gh run download` ile `reports/rounds/P2-V.1/ci/<run-id>/` altına (yalnız kayıtlar ve karşılaştırma).

## E0 — Açılış ve fizibilite
**Yapar:**
- `ROUND_START`; DECISIONS'ta K-P2V1.1 (açılış).
- İki işlemcinin vektörleri: P2-V V4b'nin (`reports/rounds/P2-V/v4/`, CI koşuları) yapıtları hâlâ indirilebiliyorsa onlar; değilse sabit korpusu `ubuntu-latest` ve `ubuntu-24.04-arm`'da gömen tek seferlik bir `workflow_dispatch` işi (yapıtı ham `f32` vektörler + CPU modeli).
- Her aday yöntem için (GOAL Kapsam 1: (y) adımlar 2⁻¹⁰…2⁻²⁰, (k), (s), birleşimler): farklı bileşen sayısı, farklı vektör sayısı, C-10 `min_cosine`, kalite referansının Δ'sı. Ayrıca iki işlemcinin ham vektörlerinde bileşen başına |δ| dağılımı (en büyük, ortalama, sıfır olmayan oranı) ve her adım için beklenen sınır çarpışması sayısı (bileşen sayısı × ortalama |δ| / adım), E0 korpusu ve 10⁵ doküman ölçeği için. Bir yöntem için ölçüm yapılamıyorsa (örn. (k) yapılandırılamıyor) "denenemedi + neden".
- `reports/rounds/P2-V.1/e0/feasibility.md` + ham sayılar JSON; seçim ve gerekçe; commit `P2-V.1/E0: fizibilite ve seçim (ön-kayıt)`.

**Kabul:** V1-1'in ilk yarısı; seçilen yöntem E0 korpusunda gözlenen 0 fark **ve** 10⁵ doküman ölçeğine beklenen sınır çarpışması < 1 (her yöntem için ikisi de `feasibility.md`'de). Hiçbir yöntem ikisini birden vermiyorsa STOP.md + karar talebi, E1 başlamaz; E3 ve E4 bağımsız sürer.

## E1 — Gömme determinizmi (ürün)
**Yapar:**
- Seçilen yöntem `index/src/embed.rs` ve/veya `index/src/vectors.rs`'te; embedder kimliği ya da indeks biçim sürümü artar; `server/`'de yalnız göç kaydı.
- Testler: aynı metin → aynı bayt (birim); göç ileri + kuru koşturma + geri alma (P2-A deseni); eski indeks `migration_required`.
- C-10, P1-C ve P1-C.1 kalite kriterlerini yeniden ölç; sıralama değişikliklerini listele (`reports/rounds/P2-V.1/e1/quality.json`).
- Gömme değişikliğinin zincirde değiştireceği alan aileleri (vektör dosyaları, düz kip skorları, C-10, D1-3 `after_hash`, D-TR, taşıyan regress alanları) — E3 envanterine girecek liste.
- Gözden geçirici (başlık: "kalite ve determinizm iddiası").

**Kabul:** V1-3, V1-4; `make test`, `make lint`, `make conformance` temiz.

## E2 — İşlemciler arası kanıt
**Yapar:**
- `ci.yml`'a (ya da ayrı iş akışına) matris işi: `ubuntu-latest`, `ubuntu-24.04-arm`, `macos-14`; her biri sabit korpusun indeksini kurar, vektör dosyalarının ve vektör türevli kayıtların blake2b'sini ve CPU modelini yapıta yazar; ayrı bir iş üçünü karşılaştırır, farklıysa kırmızı.
- Kademe 1'in bir parçası (her PR'da).

**Kabul:** V1-2 (dal koşusunda ve PR'da 3/3 eşit).

## E3 — Araç düzeltmeleri ve yeni envanter
**Yapar:**
- Önce `reports/rounds/P2-V.1/inventory.json` + `inventory.md`: P2-V envanteri aynen + R14, B3'ün yeni kriter kimliği, B6'nın adlandırmaları, E1'in beklenen fark aileleri, CPU modeli alanları (karşılaştırma dışı değil, bilgi); commit `P2-V.1/E3: envanter (ön-kayıt)`.
- Sonra istisna prosedürüyle: B1, B2, B3, B6, R14, CPU köprüsü (`host.json` + karşılaştırma işinin yazdığı hüküm), Ş2 (`eval.portable` bütün kayıtlar; `fixtures_root` göreli), Ş3 (`mac_checks.json` harness hatası ve karar alanları).
- Her düzeltme için kasıtlı bozma ya da önce/sonra kanıtı (`reports/rounds/P2-V.1/e3/`): B1 için D1-12'nin tam listesinde Z/O dışı bir girdi → sayılır.

**Kabul:** V1-8; envanter commit'i araç değişikliklerinden önce.

## E4 — Süre
**Yapar:**
- Kademe 2 karşılaştırıcısını hızlandır (profil, sonra değişiklik); aynı girdide önce/sonra çıktısı bayt eşit.
- `make verify-mac`: deterministik O kriterleri tekrarsız; tarihsel kalışlar (E-7, E1-4, H-P2-0.3b) "tarihsel" etiketiyle, kayıt eşitliğiyle.
- Kademe 2'yi turun güncel commit'inde iki kez tetikle; `make verify-mac` bir kez; FAST CI süresi.

**Kabul:** V1-6, V1-7, V1-9, V1-10 değerleri kaydedildi (geçti ya da kaldı; kalırsa optimize etme, karar kuralı).

## E5 — Gözden geçirici, rapor, hazır
**Yapar:**
- Gözden geçirici tek tur; başlık: "ölçülen şey gevşedi mi" (her araç değişikliği, her yeni karşılaştırma dışı alan, E1'in fark aileleri).
- `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- DEV_PROCESS §3 ve kök `CLAUDE.md`'deki yürürlük notunu karar kuralının dalına göre güncelle.
- `reports/rounds/P2-V.1/FINAL.md` (kriter tablosu, karar kuralının dalı, CI koşu kimlikleri, CPU modelleri, süreler); READY; tek PR.

**Kabul:** V1-5, V1-11; READY "doğrulama için hazır" + commit sha + kademe 2 ve E2 CI koşu kimlikleri.
