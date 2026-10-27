# P2-V — Doğrulama Altyapısı: Kademeli Zincir (Altyapı Turu)

Başlangıç: Blok 2 kapandı; P2-R karar kuralına göre kaldı (R-1 (b), R-6 lafzen, R-10), K-P2R.4 ve K-P2R.5. Tam `make verify` çifti üç yerel denemede ve haftalık CI'da (`40265181721`) temiz çıkmadı; farkların hiçbiri ürün yüzeyinde değil, hepsi ölçüm tasarımından: zamanlamaya duyarlı kümeler (D-A, P2-0.2'nin `serve --watch` kümeleri, gerçek Obsidian) bayt eşitliği karşılaştırmasında; bir kayıt (`release_check.json` R17 `detail`) host'un klon yolunu yazıyor. Bu tur ölçüm aracını onarır ve kademelere ayırır; **ürün kodu değişmez**. K-P2V.1. Hedef süre 1–2 gün.

Sorun: Her tur kendi denetimlerini zincire ekledi, hiçbiri çıkmadı. Bir zincir ~76–80 dk; çift, geliştiricinin makinesinde ~2,5–3 saat sessizlik ister ve yük, uyku ya da sanal makine yüzünden bozulur. Bayt eşitliği "aynı girdi → aynı çıktı" iddiasını ölçmek içindir; gecikme ve pencere yarışı ölçen kümeler bu iddiaya ait değildir, kendi eşikleriyle ölçülmelidir. Doğrulayıcı aynı çifti yeniden koşar; böylece aynı 2,5 saat bir blokta iki üç kez harcanır.

## İlkeler (bu turun değişmezleri)
- **Hiçbir kriter kaybolmaz, hiçbir eşik gevşemez.** Eski tam zincirin kaydettiği her kriter kimliği yeni kademelerin birinde kaydedilir. Bir kümenin bayt eşitliğinden çıkması, onun ölçülmemesi değil, kendi eşiğiyle ve tekrarla ayrı ölçülmesidir.
- **Ürün kodu, spec ve uyumluluk fixture'ları değişmez:** `core/`, `index/`, `server/`, `bindings/`, `clients/`, `spec/`, `conformance` fixture'ları; `events_hash`, C3-06, 39/39 aynen.
- **İki bağımsız yeniden üretim ilkesi korunur.** Tam çiftte önbellekli yeniden kullanım (girdisi değişmeyen eski denetimi koşmamak) yoktur; tam çift iki temiz klonda her şeyi koşar. Süre kazancı tam çifti CI'a taşımaktan gelir, atlamaktan değil.
- **Dondurulmuş ölçüm kodunun her değişikliği istisna prosedürüyle** (DEV_PROCESS §4: `FREEZE_EXCEPTIONS.md` girdisi, önce/sonra kanıtı, gözden geçirici onayı). Sınıflandırma, değişikliklerden önce commit'lenir ve sonra değişmez.
- Kayıtlarda host'a özgü değer yok (mutlak yol, kullanıcı adı, geçici dizin, host adı); varsa hata.
- Sessiz başarısızlık yok: atlanan bir adım "atlandı + neden" olarak kaydedilir, kaybolmaz.

## Kapsam
1. **V0 — Açılış.** `ROUND_START`; DECISIONS'ta tur açılışı (K-P2V.2). Eski zincirin referansı: `verify.yml`'ı `ROUND_START`'ta CI'da tetikle (eski `verify.sh`, değişmeden) ve yapıtını sakla — V-3'ün eşitlik karşılaştırmasının referansıdır (kırmızı biteceği beklenir; K-P2R.5'in iki nedeni). Blok 2 doğrulamasının önerilerinden doğrulama altyapısına ait olanlar bu tura, ötekiler DECISIONS'a P2-C için.
2. **V1 — Envanter ve sınıflandırma (ön-kayıt).** Tam zincirin her adımı ve her kayıt dosyası/alan kümesi için: süre (3. denemenin ve CI koşusunun günlüklerinden), girdiler (hangi ikili, hangi veri, hangi kaydedici), platform gereksinimi ve sınıf:
   - **D** — deterministik; bayt eşitliği çift karşılaştırmasında kalır.
   - **Z** — zamanlamaya duyarlı (pencere, gecikme, yarış); bayt eşitliğinden çıkar, Mac kümesinde kendi eşiğiyle ölçülür.
   - **O** — gerçek Obsidian ya da macOS'a özgü yol (FSEvents) gerekir; Mac kümesinde.
   - **H** — host'a bağlı değer sızıyor; kayıt düzeltilir (değer kaldırılır ya da göreli/normalize yazılır), sonra D olur.
   Her Z ve O kümesi için: onu getiren turun GOAL'ündeki eşik, adıyla ve kaynağıyla; eşiği yalnız bayt eşitliği olan kümeler "eşik yok" diye işaretlenir. Çıktı `reports/rounds/P2-V/inventory.json` + `inventory.md`; **V2'den önce commit'lenir ve sonra değişmez.**
3. **V2 — Kademeler.**
   - **Kademe 1 (tur içi):** `make verify FAST=1` aynen anlamıyla (aktif tur + uyumluluk + son RUN'a regresyon + tek çift). CI'da `main`'e açılan her PR'da koşar (yeni iş, `ubuntu-latest`); yerelde isteğe bağlı.
   - **Kademe 2 (blok sonu, haftalık):** tam çift **yalnız CI'da**; A ve B iki ayrı koşucuda paralel, üçüncü iş karşılaştırır. Karşılaştırma yalnız D sınıfını bayt bayt karşılaştırır; Z/O alanları adlı listeyle (joker yok, gerekçeli, V1'den) karşılaştırma dışıdır ve çıktıda "karşılaştırma dışı: <liste>" olarak yazılır. Tetikleme: haftalık zamanlayıcı + `workflow_dispatch` (girdi: commit sha).
   - **Mac kümesi:** `make verify-mac` — Z ve O kümeleri, her biri **3 tekrar**, kendi eşiğiyle; her tekrar ayrı kaydedilir (`reports/runs/<RUN>/mac/`), özet `mac_checks.json`. Gerçek Obsidian ve izleme gecikmesi burada. Blok sonunda bir kez ve Z/O kümesine dokunan turlarda.
4. **V3 — Host sızıntısı.** R17'nin `detail`'i ve V1'de H bulunan bütün alanlar düzeltilir. Bir denetleyici (`scripts/check_records_portable.sh` ya da eşdeğeri) çift karşılaştırmasına giren bütün kayıtlarda iş dizini, `$HOME`, `$TMPDIR`, kullanıcı adı ve host adı arar; bulursa kırmızı. Kasıtlı bozma: bir kayda yol enjekte et → yakalanır.
5. **V4 — Eşdeğerlik kanıtı.** Aynı ürün commit'inde (ROUND_START'ın ürün kodu) eski zincirin CI referansı (V0) ile yeni kademe 2 çıktısı: D sınıfı bütün alanlar bayt eşit; farklar yalnız V1'in Z/O/H listesindeki alanlarda ve adlarıyla. Kriter kimliği kümesi: eski ⊆ (kademe 2 ∪ Mac kümesi). Yeni tam çift CI'da **iki ayrı tetiklemede** yeşil.
6. **V5 — Süreç belgeleri.** DEV_PROCESS §3 (kademeler, hangisinin nerede ve ne zaman koştuğu), §5 (doğrulayıcı tam çifti kendi `workflow_dispatch`'iyle alır, yapıtı indirip karşılaştırmayı kendi koşturmasıyla yeniden yapar, FAST'ı ve Mac kümesini yerelde bir kez koşar; aynı çifti yerelde yeniden koşmaz); VERIFY şablonu; kök `CLAUDE.md` "Kalite kapıları" satırı (tur bitişi: FAST + CI'daki kademe 1; blok bitişi: kademe 2 + Mac kümesi); ROADMAP.
7. **V6 — Rapor, hazır.** Gözden geçirici tek tur (V6), ayrı başlık: "ölçülen şey gevşedi mi" (her Z/O çıkarımı, her H düzeltmesi, her karşılaştırma dışı alan). FINAL; READY; tek PR.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| V-1 | Envanter: tam zincirin her adımı ve çift karşılaştırmasına giren her kayıt dosyası sınıflı (D/Z/O/H), Z/O için kaynak eşik ya da "eşik yok"; V2'den önce commit'li, sonra değişmemiş (`git log`) | hepsi |
| V-2 | Ürün kodu ve spec değişmedi: `git diff ROUND_START..HEAD -- core index server bindings clients spec` boş; `make conformance` 39/39, C3-06 ve `events_hash` aynen | boş; aynen |
| V-3 | Hiçbir kriter kaybolmadı: eski zincirin (V0 referansı) kriter kimlikleri ⊆ kademe 2 ∪ Mac kümesi; D sınıfı alanlar eski↔yeni bayt eşit; farklar yalnız V1'in Z/O/H listesinde | ⊆; eşit |
| V-4 | Kademe 2: turun son commit'inde iki ayrı CI tetiklemesi; ikisinde de karşılaştırma çıkışı 0, D sınıfında fark 0, sayılan FAIL 0; tetikleme başına duvar süresi | 0; ≤ 2 sa 30 dk |
| V-5 | Host sızıntısı: denetleyici kademe 2'nin iki tetiklemesinde 0 bulgu; R17 düzeltildi; kasıtlı yol enjeksiyonu yakalandı | 0; yakalandı |
| V-6 | Mac kümesi: `make verify-mac` turun son commit'inde, 3 tekrar; her Z/O kümesi kaynak eşiğini 3/3 tekrarda karşılar; "eşik yok" kümeleri değerleriyle raporlanır | 3/3; süre ≤ 30 dk |
| V-7 | Kademe 1 CI'da: turun PR'ında FAST işi yeşil; süre | ≤ 30 dk |
| V-8 | Süreç: DEV_PROCESS §3/§5, VERIFY şablonu, kök CLAUDE.md, ROADMAP güncel; her dondurulmuş dosya değişikliği FREEZE_EXCEPTIONS'ta önce/sonra kanıtıyla; `check_frozen`, `check_deps`, `check_dep_age` temiz; gözden geçirici kabul | hepsi |

## Karar kuralı
- **Hepsi geçti** → kademeler yürürlüğe girer; P2-R'nin R-10'u V-4 ile kapanır (K-P2R.5); kısa bağımsız doğrulama (VV1–VV5) başlar; P2-C onun ardından yeni kademelerle açılır.
- **V-6 kaldı** (geri kalan geçti) → bir Z/O kümesi sessiz makinede de kendi eşiğini tutmuyor: bu **ürün bulgusudur** (izleme gecikmesi ya da pencere yarışı), ölçüm sorunu değil; değerler raporlanır, optimize edilmez; karar talebi (ürün düzeltmesi P2-C'den önce mi, P2-C'de mi); kademeler yine yürürlüğe girer, doğrulama başlar.
- **"Eşik yok" kümeleri** V-6'yı kaldırmaz; her biri için karar talebi (eşik tanımı ya da kümenin emekliye ayrılması) yazılır.
- **V-2, V-3 ya da V-5 kaldı** → tur kaldı; eski zincir yürürlükte kalır; karar talebi.
- **V-4 ya da V-7 kaldı** → tur kaldı; karar talebi.
- **V-1 ya da V-8 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Ürün kodu, spec, uyumluluk fixture'ları, NORTH_STAR.
- Herhangi bir turun eşiğinin, kriterinin ya da GOAL'ünün değişmesi; kriter silinmesi.
- Joker ya da desenle karşılaştırma dışı bırakma; her alan adıyla ve gerekçesiyle.
- Tam çiftte önbellekli yeniden kullanım.
- Linux'ta gerçek Obsidian (AppImage/Xvfb) — kapsam dışı; gerekirse sonraki bir karar.
- Self-hosted koşucu, ücretli koşucu, depoya secret.
- Z kümelerinin "düzeltilmesi" için ürün koduna dokunmak (V-6 kalırsa karar talebi).
- Yayım.
