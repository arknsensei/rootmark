# P2-V.1 — Gömme Determinizmi ve Kademelerin Tamamlanması (Yama Turu)

Başlangıç: P2-V karar kuralına göre kaldı (V-3, V-4, V-6, V-7; `reports/rounds/P2-V/FINAL.md`); bağımsız doğrulama "şu koşullarla güvenilir" (`reports/rounds/P2-V/VERIFICATION.md`), iki koşulu: türev korumasının düzeltilmesi (K-P2V-T4 B1) ve işlemciye bağlı vektörler için karar (K-P2V-T1). K-P2V.3 dört karar talebini kararlaştırdı; bu tur onları uygular. Kademeler (K-P2V.1) bu turun karar kuralıyla yürürlüğe girer; o zamana kadar blok sonunun hükmü eski tam çifttir. Hedef süre 2–3 gün.

Sorun: (1) Aynı platformun iki koşucusu farklı işlemciye düşünce gömme vektörleri bayt bayt farklı (V4b: 209 281 alan; ≤ 4e-7); NORTH_STAR ilke 2 ("her makinede aynı sonuç") türev vektör indeksinin baytları için bugün doğru değil. ORT zaten tek iş parçacığı ve `deterministic_compute` ile çalışıyor (`index/src/embed.rs`); fark işlemcinin seçtiği çekirdekten (SIMD yolu) geliyor. (2) Kademe 2'nin türev koruması kırpılmış listeye bakıyor (B1) ve P2-V'nin gözden geçiricisinin öteki araç bulguları açık (B2, B3, B6). (3) Kademe 2 2 sa 43 dk – 3 sa 04 dk, FAST CI'da 33–35 dk, Mac kümesi ~2,3 sa sürdü.

## İlkeler (bu turun değişmezleri)
- **Ürün değişikliği yalnız gömmenin sayısal çıktısıdır** (`index/` içinde embedder ve vektör yazımı) ve onun indeks biçimi/göç sonucu. Spec §1–6, uyumluluk fixture'ları, `events_hash`, olay biçimi, sözcüksel indeks değişmez. Spec §9/§10'a dokunması gerekirse SPEC_EDITS girdisiyle ve yalnız bilgilendirici metinde.
- **Hiçbir kriter kaybolmaz, hiçbir eşik gevşemez** (P2-V ilkeleri aynen). Bu turun yeniden yazdığı süre eşikleri yalnız K-P2V.3'te kararlaştırılanlardır (kademe 2 ≤ 3 sa 15 dk, FAST CI ≤ 40 dk); başka eşik değişmez.
- **Önce ölç, sonra seç.** Gömme yönteminin seçimi (yuvarlama adımı ya da başka yol) E0'ın ölçümüne dayanır ve seçim ölçümden sonra, uygulamadan önce commit'lenir.
- **Dondurulmuş ölçüm kodunun her değişikliği istisna prosedürüyle** (DEV_PROCESS §4). Yeni envanter (`reports/rounds/P2-V.1/inventory.json`) değişikliklerden önce commit'lenir ve sonra değişmez; P2-V'nin envanteri aynen kalır.
- Gömme değişikliğinin zincirde değiştireceği alanlar (vektör dosyaları, düz kip skorları, C-10 `min_cosine`, D1-3 `after_hash`, D-TR değerleri ve bunları taşıyan regress alanları) **ölçümden önce aile aile adlandırılır**; adı geçmeyen fark tanımlanmamış farktır.
- Sessiz başarısızlık yok; kayıtlarda host'a özgü değer yok.

## Kapsam
1. **E0 — Açılış ve fizibilite.** `ROUND_START`; DECISIONS'ta açılış (K-P2V1.1). İki farklı işlemcide üretilmiş vektörler üzerinde (P2-V V4b'nin iki zinciri; yapıt yoksa aynı korpusu `ubuntu-latest` ve `ubuntu-24.04-arm` koşucularında gömen tek seferlik bir CI işi) aday yöntemlerin fark sayısı:
   - (y) normalize edilmiş vektörü sabit bir ızgaraya yuvarlamak (adımlar 2⁻¹⁰…2⁻²⁰; her adım için farklı yuvarlanan bileşen sayısı ve kalite etkisi);
   - (k) ORT/MLAS çekirdeklerini işlemciden bağımsız bir yola sabitlemek (yapılandırma ya da derleme bayrağı; mümkünse);
   - (s) gömmenin son adımlarını (havuzlama, normalizasyon) sabit sıralı skaler Rust'ta yapmak;
   - ya da bunların birleşimi.
   Çıktı `reports/rounds/P2-V.1/e0/feasibility.md`; **seçim ve gerekçesi E1'den önce commit'lenir.** Hiçbir yöntem korpusta 0 fark vermiyorsa: STOP.md ve karar talebi (K-P2V-T1 (b) kapsam kararı).
2. **E1 — Gömme determinizmi (ürün).** Seçilen yöntem `index/`'te; embedder kimliği ya da indeks biçim sürümü değişir; eski indeks `migration_required` verir ve `rootmark migrate` ile yeniden kurulur (P2-A çerçevesi: ileri göç, kuru koşturma, geri alma). C-10 ve P1-C/P1-C.1 kalite kriterleri kendi eşikleriyle yeniden ölçülür; sıralamalardaki her değişiklik raporlanır.
3. **E2 — İşlemciler arası kanıt.** CI'da üç koşucuda (`ubuntu-latest` x86_64, `ubuntu-24.04-arm` aarch64, `macos-14` arm64) sabit bir korpusun (P1-C korpusu ya da onun belirlenmiş alt kümesi) indeksi kurulur; vektör dosyalarının ve vektör türevli kayıtların baytları karşılaştırılır. Kademe 1'in bir parçası olur (her PR'da).
4. **E3 — Araç düzeltmeleri (istisna prosedürüyle; K-P2V-T4, doğrulamanın Ş1–Ş3'ü).**
   - B1: türev koruması kriterin kendi kaydındaki tam regress listesine bakar ve sınıflandırıcının izin listesini uygular.
   - B2: `_criterion_fields` bir Z/O kriterinin yalnız Z/O alanlarını kenara alır; deterministik iç alanlar (C1-8'in sıralama kimlikleri ve sırası) karşılaştırmada kalır.
   - B3: p1e1 `tamper`'ın deterministik yarısı ayrı bir kriter kimliğiyle sayılır.
   - B6: O13, H1, C2 dizi sırasıyla değil adla; gerçek Obsidian kayıtlarının çift sınıfı tekilleştirilir.
   - R14: `detail` (en yeni `ROUND_START`) commit'e bağlı listeye.
   - Köprü (K-P2V-T1 (c)): her zincir CPU modelini `host.json`'a yazar; karşılaştırma işi iki modeli ve aynı/farklı olduğunu hükümle birlikte yazar.
   - `mac_checks.json` harness hatasının son satırlarını ve kararı veren alanları taşır (Ş3); `eval.portable` çifte girmeyen kayıtları da tarar, `conformance*.json` `fixtures_root` göreli yazılır (Ş2).
   - Yeni envanter `reports/rounds/P2-V.1/inventory.json` + `inventory.md`: P2-V envanterinin üstüne bu turun eklemeleri ve E1'in beklenen fark aileleri; E3'ün araç değişikliklerinden önce commit'lenir.
5. **E4 — Süre.** Karşılaştırıcı (kademe 2'nin compare işi, ~25 dk) istisna prosedürüyle hızlandırılır; çıktısı bayt bayt aynı kalır (önce/sonra kanıtı aynı girdide). Mac kümesinde deterministik O kriterleri tekrarsız koşar (K-P2V-T2 (b)); kararla kapanmış kalışlar (E-7, E1-4, H-P2-0.3b) "tarihsel" etiketiyle, kayıt eşitliğiyle raporlanır, ham eşikle değil (K-P2V-T2 (c)).
6. **E5 — Rapor, hazır.** Gözden geçirici tek tur, iki başlık: "gömme değişikliği kaliteyi ya da determinizm iddiasını zayıflattı mı" ve "araç değişiklikleri ölçüleni gevşetti mi". FINAL; READY; tek PR.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| V1-1 | E0 fizibilitesi ve seçim E1'den önce commit'li; seçilen yöntem E0 korpusunda farklı bileşen 0 | commit'li; 0 |
| V1-2 | İşlemciler arası: E2'nin üç koşucusunda vektör dosyaları ve vektör türevli kayıtlar bayt eşit; turun PR'ında ve son commit'inde | eşit (3/3) |
| V1-3 | Kalite: C-10 ve P1-C/P1-C.1 kalite kriterleri kendi GOAL eşiklerinde; sıralama değişiklikleri adlarıyla raporlandı | eşiklerde |
| V1-4 | Göç: P2-L/P2-R dönemi bir vault'un indeksi `migration_required` → `rootmark migrate` → taze kurulumla bayt eşit; geri alma eski indeksi geri getirir | eşit; geri alındı |
| V1-5 | Ürün sınırı: `git diff ROUND_START..HEAD -- core bindings clients spec` boş (spec'e dokunulduysa yalnız SPEC_EDITS'li bilgilendirici metin); `server/`'de yalnız göç kaydı (biçim sürümü/embedder kimliği tablosu); `make conformance` 39/39, C3-06 ve `events_hash` aynen | boş; yalnız göç; aynen |
| V1-6 | Hiçbir kriter kaybolmadı: P2-V V0 referansının kriter kimlikleri ⊆ kademe 2 ∪ Mac kümesi; B3'ün yeni kimliği eklendi; D alanlarında eski↔yeni farklar yalnız E1'in önceden adlandırılmış ailelerinde | ⊆; adlı |
| V1-7 | Kademe 2: turun son commit'inde iki ayrı tetikleme; ikisinde de çıkış 0, D farkı 0, düzeltilmiş türev korumasıyla sayılan FAIL 0; `host.json` CPU modelleri kayıtlı; tetikleme başına duvar süresi; karşılaştırıcının önce/sonra çıktısı aynı girdide bayt eşit | 0; ≤ 3 sa 15 dk; eşit |
| V1-8 | Araç bulguları: B1, B2, B3, B6, R14, Ş2, Ş3 her biri için kasıtlı bozma ya da önce/sonra kanıtı; taşınabilirlik denetleyicisi bütün kayıtlarda 0 bulgu | hepsi; 0 |
| V1-9 | Kademe 1 CI'da: turun PR'ında FAST ve E2 işleri yeşil; FAST süresi | ≤ 40 dk |
| V1-10 | Mac kümesi: turun son commit'inde; deterministik O kriterleri 1 tekrar ve geçti; Z kümeleri 3 tekrar, değerleriyle raporlandı; tarihsel kalışlar kayıt eşitliğiyle; süre raporlandı | geçti; raporlandı |
| V1-11 | Süreç: FREEZE_EXCEPTIONS önce/sonra kanıtlı; envanter commit'i E3'ten önce ve sonra değişmemiş; `check_frozen`, `check_deps`, `check_dep_age` temiz; DEV_PROCESS §3 ve kök CLAUDE.md'deki yürürlük notu; gözden geçirici kabul | hepsi |

Z kümelerinin kaynak eşikleri bu turun hükmüne girmez (K-P2V.3: sessiz koşu yapılmaz); değerleri raporlanır ve eşiğin altında kalanlar P2-C'ye ürün bulgusu (izleme gecikmesi, pencere yarışı) olarak taşınır.

## Karar kuralı
- **Hepsi geçti** → kademeler (K-P2V.1) yürürlüğe girer; P2-V doğrulamasının iki koşulu kapanır; NORTH_STAR ilke 2 vektör indeksi için yeniden doğrudur (notu FINAL'de, değişiklik insan kararı); kısa bağımsız doğrulama (VW1–VW5); P2-C ardından yeni kademelerle açılır.
- **V1-1 ya da V1-2 kaldı** → işlemciden bağımsız gömme bu yöntemlerle kurulamadı; E1 birleşmez (ürün kodu `ROUND_START`'taki haliyle kalır); araç düzeltmeleri (E3, E4) ayrı PR olarak sürer; karar talebi: K-P2V-T1 (b) kapsam kararı (vektör baytları "işlemci ailesine bağlı türev" diye O'ya, NORTH_STAR notu).
- **V1-3 kaldı** → yöntem kaliteyi düşürüyor; değerler raporlanır, optimize edilmez; karar talebi (daha ince adım ya da başka yöntem).
- **V1-4, V1-5 ya da V1-6 kaldı** → tur kaldı; eski zincir yürürlükte; karar talebi.
- **V1-7 süre kısmı kaldı** (geri kalan geçti) → değer raporlanır; kademeler yürürlüğe girer, kademe 2'nin süresi için karar talebi.
- **V1-7'nin geri kalanı, V1-8, V1-9, V1-10, V1-11 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Embedder modelinin, tokenizer'ın, sözcüksel indeksin, hibrit birleşim kuralının değişmesi.
- Spec §1–6, uyumluluk fixture'ları, NORTH_STAR (öneri FINAL'de, değişiklik insan kararı).
- Herhangi bir turun eşiğinin değişmesi (K-P2V.3'ün iki süre eşiği dışında); kriter silinmesi.
- Joker ya da desenle karşılaştırma dışı bırakma.
- Mac kümesinin sessiz makinede koşulması (K-P2V.3); Z kümeleri için ürün düzeltmesi (P2-C).
- Self-hosted ya da ücretli koşucu, depoya secret.
- Yayım.
