# P2-V — Görevler

Genel kurallar:
- Sıra: V0 → V1 → V2 → V3 → V4 → V5 → V6. V1 commit'lenmeden V2'ye geçilmez.
- Progress `docs/rounds/P2-V/progress/`. Commit öneki `P2-V/V<n>:`.
- Dal `p2-v/round`, yalnız birleşmiş `main`'den (K-P2.2). `ROUND_START` = bu turu açan PR'ın birleşmiş `main` HEAD'i.
- Altyapı turu, geliştirme dozu (K-P2.4): gözden geçirici yalnız V6'da, tek tur; başlığı "ölçülen şey gevşedi mi".
- Uzun koşular CI'da: tam çift yalnız `verify.yml` (ya da yeni kademe 2 iş akışı) ile; yerelde tam `make verify` koşulmaz. Yerelde yalnız FAST ve Mac kümesi.
- Dondurulmuş kodda her değişiklik `FREEZE_EXCEPTIONS.md`'ye önce/sonra kanıtıyla (DEV_PROCESS §4).
- CI tetiklemeleri: `gh workflow run <iş akışı> --ref <dal> [-f sha=<sha>]`, izleme `gh run watch`/`gh run view`; yapıtlar `gh run download` ile `reports/rounds/P2-V/ci/<run-id>/` altına (yalnız kayıtlar ve karşılaştırma; büyük ağaçlar değil).

## V0 — Açılış ve eski zincirin referansı
**Yapar:**
- `ROUND_START`; DECISIONS'ta K-P2V.2 (açılış; Blok 2 doğrulamasının altyapıyla ilgili önerileri varsa bu tura alındığı, ötekilerin P2-C'ye bırakıldığı).
- `verify.yml`'ı `ROUND_START`'ta tetikle (eski `verify.sh` değişmeden); yapıtı indir; kırmızıysa farklı alanları listele ve K-P2R.5'in iki nedeniyle karşılaştır (`reports/rounds/P2-V/v0/old_chain.md`).

**Kabul:** referans koşunun kayıtları depoda; farkların K-P2R.5 teşhisiyle uyumu yazılı (uyumsuzsa STOP.md: teşhis yanlış olabilir, karar talebi).

## V1 — Envanter ve sınıflandırma (ön-kayıt)
**Yapar:**
- `verify.sh`'ın tam zincirinin her adımı: ad, süre (P2-R 3. denemesinin ve V0 koşusunun günlüklerinden), çağırdığı hedef, girdiler, platform gereksinimi.
- Çift karşılaştırmasına giren her kayıt dosyası ve alan kümesi: sınıf D/Z/O/H ve gerekçe. Z/O için onu getiren turun GOAL'ündeki eşik (dosya ve satırla) ya da "eşik yok".
- Kriter kimliklerinin tam listesi (bütün `*_checks.json`'lardan) — V-3'ün referansı.
- `reports/rounds/P2-V/inventory.json` + `inventory.md`; commit `P2-V/V1: envanter (ön-kayıt)`.

**Kabul:** V-1. Bu commit'ten sonra envanter dosyaları değişmez (yanlışlık bulunursa yeni dosya + ASSUMPTIONS + FINAL'de not; eskisi aynen).

## V2 — Kademeler
**Yapar:**
- Kademe 1: CI'a PR'da koşan FAST işi (`ci.yml`'a iş ya da ayrı iş akışı); `make verify FAST=1`'in anlamı aynen.
- Kademe 2: tam zincir A ve B ayrı koşucularda paralel, üçüncü iş karşılaştırır; karşılaştırıcı D sınıfını bayt bayt, Z/O alanlarını V1'in adlı listesiyle dışarıda bırakır ve bunu çıktıya yazar. Haftalık zamanlayıcı ve `workflow_dispatch` (sha girdisi).
- Mac kümesi: `make verify-mac`; Z/O kümeleri 3 tekrar, kaynak eşikleriyle; `mac_checks.json`.
- Disk: koşucu başına tek zincir (P2-A/GH-0'daki disk sorununu ortadan kaldırır); gerekirse eski adımın disk temizliği aynen.

**Kabul:** üç kademe tek başına çalışır; FAST yerelde ve CI'da yeşil; dondurma istisnaları önce/sonra kanıtlı.

## V3 — Host sızıntısı
**Yapar:** R17 `detail` ve V1'in bütün H alanları; taşınabilirlik denetleyicisi kademe 2'nin karşılaştırma adımında; kasıtlı yol enjeksiyonu testi.

**Kabul:** V-5'in yerel kısmı (denetleyici temiz, enjeksiyon yakalanıyor).

## V4 — Eşdeğerlik
**Yapar:**
- Kademe 2'yi turun güncel commit'inde CI'da iki kez tetikle (iki ayrı `workflow_dispatch`).
- V0 referansı ile yeni kademe 2 çıktısı: D alanları eşit mi; kriter kimlikleri ⊆; farkların hepsi V1 listesinde mi (`reports/rounds/P2-V/v4/equivalence.json`).
- `make verify-mac` turun güncel commit'inde, 3 tekrar.

**Kabul:** V-3, V-4, V-5, V-6 değerleri kaydedildi (geçti ya da kaldı; kalırsa optimize etme, karar kuralı).

## V5 — Süreç belgeleri
**Yapar:** DEV_PROCESS §3 ve §5; VERIFY şablonu (doğrulayıcı tam çifti CI'dan alır ve karşılaştırmayı kendisi yeniden yapar); kök `CLAUDE.md` "Kalite kapıları"; ROADMAP §2 (P2-V satırı) ve §4.

**Kabul:** V-8'in belge kısmı.

## V6 — Gözden geçirici, rapor, hazır
**Yapar:**
- Gözden geçirici tek tur; başlık: her Z/O çıkarımı, her H düzeltmesi, karşılaştırma dışı her alan — "ölçülen şey gevşedi mi".
- `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- `reports/rounds/P2-V/FINAL.md` (kriter tablosu, karar kuralının dalı, CI koşu kimlikleri, süreler: eski zincir ↔ kademe 1 ↔ kademe 2 ↔ Mac kümesi); READY; tek PR.

**Kabul:** V-7, V-8; READY "doğrulama için hazır" + commit sha + kademe 2 CI koşu kimlikleri.
