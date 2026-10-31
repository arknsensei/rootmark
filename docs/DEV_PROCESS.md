# Geliştirme Süreci

Prototip turlarında işe yaramış disiplin, ürün geliştirmeye uyarlanmıştır. Her tur bir `/goal` koşturmasıdır ve bu belge o koşturmanın anayasasıdır.

## 1. Tur anatomisi
Her tur `docs/rounds/<id>/` altında dört dosya ile tanımlanır:
- `GOAL.md` — tur amacı, kapsam, **önceden kayıtlı çıkış kriterleri**, yapılmayacaklar.
- `TASKS.md` — sıralı görevler; her görevde Yapar / Kabul / Üretir.
- `VERIFY.md` — bağımsız doğrulayıcı yönergesi (temiz klon, çift koşturma, kasıtlı bozma).
- `PROMPT.md` — ana ajan ve doğrulayıcı için kopyala-yapıştır `/goal` istemleri.

İki tür tur vardır (K-P2.4): **hipotez turu** (NORTH_STAR düzeyinde bir iddia; prototip disiplini: D-set, önceden kayıtlı eşik, tam çift koşturma, tam bağımsız doğrulama) ve **geliştirme turu** (birim/entegrasyon testleri, kalite kapıları, önceden kayıtlı çıkış kriterleri; D-set yok; gerçek Obsidian yalnız e2e). Geliştirme turları bloklar halinde ilerler; bağımsız doğrulama blok sonunda bir kez, dar kapsam. Tur başladıktan sonra `GOAL.md` değişmez. Yeni ihtiyaç bir sonraki tura gider ve `docs/DECISIONS.md`'ye yazılır.

`GOAL.md` yazım kuralları (K-D1.9, P1-D.1):
- **Spec sürümü öngörülmez.** Bir GOAL spec düzenlemesi isterse sürüm numarası yazmaz, yalnız "sürüm `docs/SPEC_EDITS_CRITERION.md` kuralıyla" der. Sürümü, düzenlemeler yapıldıktan sonra CRITERION belirler ve `SPEC_EDITS.md` kaydeder.
- **Karar kuralı tamdır.** Her karar kuralı, adlandırdığı dalların yanında bir varsayılan dal taşır: "**tanımlanmamış kalış → tur kaldı, karar talebi**". Kuralda adı geçmeyen bir kriter kalırsa tur kalmış sayılır ve `DECISIONS.md`'ye karar talebi yazılır. Hiçbir kalış sessizce "sonraki tur başlar" sonucuna düşmez.

## 2. Ajan disiplini (kök `CLAUDE.md` bunu uygular)
- Okuma sırası: `NORTH_STAR.md` → `spec/SPEC.md` → `docs/ARCHITECTURE.md` → `docs/PACKAGES.md` → aktif tur dosyaları → ilgili paketin `CLAUDE.md`'si → `docs/rounds/<id>/progress/`.
- Görev başı/sonu kayıtları, gözden geçirici alt-ajan (yalnız kabul ölçütleri + değişen dosya listesi ile; kod yazmaz), görev başına commit (`<round>/<task>: ...`). Gözden geçirici: hipotez turlarında görev başına en fazla bir tur (K-P2.3); geliştirme turlarında tur sonunda tek tur, READY öncesi (K-P2.4).
- Dal ve PR (K-P2.1): `main` korumalıdır (PR + CI, doğrusal geçmiş). Tur tek bir dalda (`<tur-id>/round`) ilerler; görev başına commit dalda, her görev sonunda dal uzağa itilir; tur sonunda READY.md ile tek PR açılır. PR'ı orkestratör birleştirir; doğrulayıcı birleşmiş `main`'i klonlar, VERIFICATION.md ayrı PR ile girer. Bir tur yalnız birleşmiş `main`'den açılır; `ROUND_START` bir `main` commit'ini ya da etiketi gösterir (rebase birleştirmesi dal commit'lerini yeniden yazar, K-P2.2).
- Park kuralı: yalnız durma koşulunda; bağımlılar işaretlenir; park yerinden devam.
- Soru sorulmaz; varsayım etiketli (`[spec] [arch] [-]`) olarak `ASSUMPTIONS.md`'ye yazılır. Spesifikasyonla çelişen varsayım yapılmaz; çelişki görülürse görev park edilir ve `DECISIONS.md`'ye karar talebi yazılır.

## 3. Kalite kapıları (her tur, her commit)
- `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`; Python için `pytest`, `ruff`; TypeScript için `tsc`, `eslint`, `vitest`.
- `make conformance` — claim edilen seviyeye kadar tüm uyumluluk testleri.
- **Doğrulama zinciri üç kademedir (P2-V, K-P2V.1).** Her kademe temiz klon, iki bağımsız zincir ve karşılaştırmadır; sıfır olmayan çıkış başarısızlıktır. **Yürürlük:** P2-V.1'in karar kuralına bağlıdır (K-P2V.3; P2-V kaldı, `reports/rounds/P2-V/FINAL.md`); kademeler yürürlüğe girene kadar blok sonunun hükmü eski tam çifttir (`scripts/verify.sh pair`, dondurulmuş `compare_p2r pair`, bayt eşitliği), kademelerin çıktısı yanında bilgi olarak kaydedilir.
  - **Kademe 1 — tur içi kapı, her PR'da:** `make verify FAST=1` (yalnız aktif bloğun kontrolleri + uyumluluk + son resmi RUN'a regresyon + tek çift; K-P2.4 (3)). CI'da `ci.yml`'nin `fast` işleri `main`'e açılan her PR'da koşar: iki `ubuntu-latest` koşucusunda `verify.sh chain-fast`, dondurulmuş `eval.compare_p2r fast`, kayıtlarda host değeri denetimi (`eval.portable --fast`); hedef ≤ 30 dk. Yerelde isteğe bağlıdır.
  - **Kademe 2 — tam çift, yalnız CI'da (blok sonu ve haftalık):** `verify.yml`: tam zincir (`verify.sh chain`, commit'in kendi zinciri; hiçbir adım önbellekten alınmaz, atlanmaz) iki `ubuntu-latest` koşucusunda paralel, üçüncü iş karşılaştırır. Hüküm yalnız deterministik (D) sınıftadır: `eval.tier2 pair` dondurulmuş `eval.compare_p2r.pair`'in her alanını karşılaştırır, envanterin (`reports/rounds/P2-V/inventory.json`) adını verdiği zamanlamaya duyarlı (Z) ve gerçek Obsidian (O) alanlarını kenara alır ve çıktıda "karşılaştırma dışı" diye listeler; sayılan FAIL `eval.tier2 fails` (Z/O kriterleri Mac kümesinindir; referans RUN'ı yayım Mac'inde kaydedilmiş, değeri o donanıma bağlı kriterler — C1-8, E-7, E1-4 — O'dur: kayıtları iki zincirde karşılaştırılır, ölçütleri Mac kümesinde); host değeri `eval.portable`. Tetikleme haftalık zamanlayıcı ve `workflow_dispatch` (`sha` girdisi: o commit'in zinciri bugünün karşılaştırmasıyla); hedef ≤ 2 sa 30 dk. Tam `make verify` çifti yerelde koşulmaz (geliştiricinin makinesi saatlerce sessiz kalmaz).
  - **Mac kümesi — blok sonunda bir kez ve Z/O kümesine dokunan turlarda:** `make verify-mac`: envanterin Z ve O kümeleri, her biri 3 tekrar, kendi turunun eşiğiyle (`<RUN>/mac/rep<i>/`, özet `<RUN>/mac_checks.json`); gerçek Obsidian, izleme gecikmesi ve yayım Mac'ine bağlı referanslar burada ölçülür.
  - **Envanter ön-kayıttır.** Zincire kayıt ekleyen bir tur, kaydın sınıfını (D/Z/O) ve Z/O ise eşiğini GOAL'ünde yazar; envanter ve kademe 2'nin karşılaştırıcısı dondurulmuştur, değişiklik yeni bir envanter dosyası ve §4 istisna prosedürüyle olur. Bir kümenin bayt eşitliğinden çıkması onun ölçülmemesi değildir: Mac kümesinde kendi eşiğiyle ve tekrarla ölçülür; hiçbir kriter kaybolmaz.
  - **Kayıtlarda host'a özgü değer yoktur** (mutlak yol, kullanıcı adı, geçici dizin, host adı); ürünün çıktısındaki mutlak yol kayda depo köküne göreli yazılır; `eval.portable` (`scripts/check_records_portable.sh`) bulursa kırmızıdır.
- Kalite regresyonu: `research/` içindeki holdout sentetik set ve v2 `hybrid` sayıları her turda yeniden koşturulur; sayı değişiyorsa açıklanmak zorundadır.
- Sessiz başarısızlık yasak: çözülemeyen referans, eksik dosya, manifest uyuşmazlığı → hata.

## 4. Dondurma politikası
- `spec/SPEC.md` §1–6 ve uyumluluk fixture'ları yalnız karar kaydıyla ve spec sürüm artışıyla değişir.
- Ölçüm kodu (`research/eval/**`, `scripts/verify.sh`, dondurma denetleyicileri) tur içinde dondurulur; istisna: `FREEZE_EXCEPTIONS.md` girdisi + önce/sonra bit eşitliği + gözden geçirici onayı. Denetleyicilerin kendisi dondurma listesindedir ve derlenmiş ayarlanabilir sabitlere bağlı olamaz.
- Eşikler ve çıkış kriterleri tur başladıktan sonra değişmez.

## 5. Bağımsız doğrulama
- Ayrı oturum, temiz klon, ana çalışma dizinine okuma dahil erişim yok.
- Çift koşturma, kasıtlı bozma testleri, izlenebilirlik, karar kuralının bağımsız türetimi.
- **Tam çifti CI'dan alır (P2-V, K-P2V.1).** Doğrulayıcı kademe 2'yi kendi tetiklemesiyle başlatır (`gh workflow run verify.yml --ref main -f sha=<doğrulanan commit>`), iki zincirin yapıtını (`tier2-A`, `tier2-B`: bütün RUN dizini, günlük, `host.json`, sınıflandırıcı çıktısı) indirir ve D sınıfının karşılaştırmasını dondurulmuş karşılaştırıcıya dayanmadan **kendi betiğiyle** yeniden yapar; karşılaştırma dışı listenin envanterle birebir aynı olduğunu ve desen içermediğini denetler. `make verify FAST=1`'i ve `make verify-mac`'i yerelde bir kez koşar; aynı tam çifti yerelde yeniden koşmaz.
- Çıktı: `reports/rounds/<id>/VERIFICATION.md`; hüküm "güvenilir / şu koşullarla / değil" + en fazla 3 öneri. Geliştirme turlarında VERIFY "dar kapsam"dır: yalnız kararı belirleyen kriterler, 2–3 saat; hipotez turlarında tam kapsam (K-P2.3). Öneriler bir sonraki turun `GOAL.md`'sinde ele alınır ya da `DECISIONS.md`'de reddedilme gerekçesiyle kapatılır.
- **Spec durum damgası (K-P2-02.3; P2-0.1 doğrulaması Ö3, K-P2-01.6).** Bir spec bölümünün durum damgası "Validated", bağımsız doğrulamanın **"güvenilir"** hükmünden — ya da "şu koşullarla güvenilir" hükmünün koşulları bir sonraki turda kapanıp bağımsız olarak teyit edildikten sonra — konur. Turun ana ajanı FINAL'de damga koymaz: karar kuralının dalını ve damga önerisini yazar, spec'in damgasını "Draft (<tur>'un doğrulamasına kadar)" bırakır. Damgayı, doğrulamadan sonraki kapanış commit'i (orkestratör) ya da sonraki turun açılış görevi koyar ve `SPEC_EDITS.md`'ye açıklık satırı olarak yazar; doğrulamanın hükmüne atıf zorunludur.

### VERIFY.md şablonu (kademeli zincirle; P2-V)
```
# <tur> — Bağımsız Doğrulama (<dar|tam> kapsam)
Ayrı oturum, temiz klon (kendi scratchpad'in), ana çalışma dizinine erişim yok; kod değiştirilmez.
- V1 — Kademe 2'yi kendin tetikle: `gh workflow run verify.yml --ref main -f sha=<sha>`; yapıtı indir; D alanlarını A↔B kendi betiğinle karşılaştır; karşılaştırma dışı liste = envanter (desen yok). Koşu kimliğini yaz.
- V2 — Kademe 1 ve Mac kümesi: `make verify FAST=1` bir kez; turun Z/O kümesine dokunuyorsa (ya da blok sonuysa) `make verify-mac` bir kez (3 tekrar); süreler.
- V3.. — Turun kendi kriterleri: kararı belirleyen kriterler, kendi vakalarınla; kasıtlı bozmalar.
- Vn — Karar kuralını GOAL'den kendin türet, FINAL ile karşılaştır; vaat sorusu: GOAL'ün ölçmediği ama NORTH_STAR'ı ilgilendiren bir şey gördün mü.
- Sonuç: ihlal/fark/şüphe; kriter başına geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
```

## 6. Karar kayıtları
`docs/DECISIONS.md`: `K<n>` numaralı, tarihli, gerekçeli, etkilenen belgeler listeli. Mimari belgedeki K1–K8 ilk girdilerdir. "Sonuç görüldükten sonra alınan karar" olarak işaretleme zorunludur (K4 örneği).

## 7. Paket `CLAUDE.md` şablonu
```
# <paket> — ajan notları
Sorumluluk: <PACKAGES.md satırı>
Bağımlılık yönü: <core ← index ← server ← clients>; ters yön yasak.
Dil ve araçlar: <...>
Dondurulmuş yüzeyler: <spec bölümleri, public API>
Test komutları: <...>
Bu pakette yapılmayacaklar: <...>
```

## 8. Sürüm ve yayım
Faz 1 sonunda `rootmark 0.1.0`: uyumluluk Seviye 3 raporu, `make verify` çıktısı, değişiklik günlüğü, spec sürümü. Sonrası SemVer; spec kırıcı değişiklik → majör.
