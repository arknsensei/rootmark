# D-0 — Dağıtım: İsim, Depo ve Aktarım Planı (karar + rapor turu)

Başlangıç: P2-Q.2 "hepsi geçti" ile kapandı (`reports/rounds/P2-Q.2/FINAL.md`, PR #57). Bağımsız doğrulamanın hükmü **"güvenilir"** (`reports/rounds/P2-Q.2/VERIFICATION.md`, PR #58). 0.3.2 yayımlı (`v0.3.2` → `c544a9e`). K-D0.1'deki sıraya göre sıradaki tur D-0'dır. K-P2Q2.2 turu açar.

D-0 bir **karar ve rapor turudur**: ürün kodu yazılmaz, hiçbir kayıt defterine paket yayımlanmaz, hesap ya da depo açılmaz. Çıktısı, ürün sahibinin isim, depo ve kayıt defterleri kararını verebileceği bir rapor ve D-1'in uygulayacağı yazılı bir aktarım planıdır.

## Tur öncesi girdiler (K-P2Q2.2; orkestratörün 2026-11-10 ön taraması, bu turda yeniden yapılır)
- **Alan adı:** ürün sahibi `rootmark.ai`'yi aldı ve bu alan adında bir yönetici posta kutusu açtı.
- **PyPI `rootmark` dolu:** "Rootmark SDK — agent instrumentation" adında etkin bir proje (2026-08'den beri, son sürüm 0.1.5, 2026-10-24). Yakın alanda. Bugünkü wheel'in adı `rootmark_bindings` (`rootmark-bindings`); PyPI'da boş.
- **GitHub `rootmark` adı dolu:** 2018'den beri boş bir kullanıcı hesabı (depo yok). Kurum adayı `rootmark-ai` boş.
- **Boş görünenler:** crates.io (`rootmark`, `rootmark-core`, `rootmark-index`, `rootmark-server`), npm (`rootmark`, `@rootmark/*`), Docker Hub (`rootmark`), Homebrew formülü, Obsidian eklenti kimliği `rootmark`.
- **Alan adları:** `rootmark.com` 2004'ten beri kayıtlı. `rootmark.dev`'i PyPI'daki proje kendi sitesi olarak gösteriyor ama alan adı yanıt vermiyor.
- **Ad önerisi (orkestratör):** ürün adı RootMark kalır; PyPI'da `rootmark-bindings`; öteki kayıt defterlerinde `rootmark`. Karar ürün sahibinindir (D0-7).
- **İnsan adımları (hesaplar):** ürün sahibi hesapları açarken bu turu beklemez. Turun raporu, hangi hesabın açıldığını `progress/HUMAN_STEP.md` aracılığıyla ürün sahibinden öğrenir ve kaydeder. Ajan hiçbir hesabı açmaz, hiçbir adı yayımlamayla "kapmaz".

## İlkeler
- **Ürün kodu değişmez.** `core`, `index`, `server`, `bindings`, `clients/*/src`, `spec`, fixture'lar, `packaging/`, `scripts/`, `.github/workflows/**`, `Cargo.toml`/`Cargo.lock` ve istemcilerin paket dosyaları bu turda değişmez. İstisna: PQ2-4 için `server/CLAUDE.md`'deki `rmcp` sürüm satırı (belge). Değişiklikler `docs/`, `reports/rounds/D-0/` ve gerekiyorsa `research/d0/` (yalnız tarama ve sınama betikleri) altında olur.
- **Dışarıya yazan eylem yok.** Ajan kayıt defterlerine yalnız okuma isteği atar (herkese açık API, `GET`). Hesap açmaz, ad rezerve etmez, paket, imaj ya da etiket yayımlamaz, GitHub'da depo ya da kurum açmaz, alan adı DNS'ine dokunmaz. Bunlar insan adımıdır ve D-1'in işidir.
- **Her tarama satırı kanıtlı.** Kayıt defteri ya da marka taramasının her satırı şunları taşır: sorgulanan URL (ya da elle sorgulanan arayüzün adı), sorgu zamanı (UTC), yanıt kodu ve yanıtın içeriksiz özeti (dolu/boş, sahibi, son sürüm tarihi). Erişilemeyen kaynak "erişilemedi" yazılır, uydurulmaz. Yanıtı tarayıcı gerektiren kaynaklar (ör. TÜRKPATENT) `HUMAN_STEP.md` ile insana verilir.
- **Kişisel içerik yok.** Kişisel vault ve qorbit'e dokunulmaz. PQ2-1 sınaması yalnız D-NOTES kopyasında ya da sentetik bir kasada.
- **Gizli bilgi yok.** Raporda, kayıtlarda ve PR'da token, parola, kurtarma kodu ya da kişisel e-posta adresi geçmez. İletişim adresleri için yalnız rol adı önerilir (ör. `security@`).
- **Aktarım ölçülebilir olmalı.** Laboratuvardan ürün deposuna aktarım planı, hangi yolların gideceğini bir **izin listesi** olarak yazar ve bugünkü `main` üzerinde sayılarla gösterir (dosya sayısı, gidenler, kalanlar, sınıflandırılmamış 0). D-1'in çıkış kriteri ("otomatik ve doğrulanabilir, bayt eşit aktarım") bu listeye dayanır.
- Sessiz başarısızlık yok; markdown'a kimlik/meta veri yazılmaz.

## Kapsam
1. **R0 — Açılış.** `ROUND_START`; DECISIONS K-D0.2 (açılış). Tarama yöntemi ve kaynak listesi `progress/DESIGN.md`'de, taramadan önce.
2. **R1 — İsim ve kayıt defterleri taraması.** Aşağıdaki her ad ve her kayıt defteri için kanıtlı satır (İlkeler):
   - Adlar: `rootmark`, `rootmark-bindings`, `rootmark-core`, `rootmark-index`, `rootmark-server`, `rootmark-ai`, `getrootmark`; R1'in bulgusuna göre en çok iki yedek ad (ör. raporda gerekçeli bir alternatif).
   - Kayıt defterleri: crates.io, PyPI, TestPyPI, npm (paket ve kurum/kapsam), Docker Hub (kullanıcı/kurum), `ghcr.io` (GitHub kurum ya da kullanıcı adına bağlı), GitHub (kullanıcı/kurum), Homebrew (homebrew-core formülü ve cask; tap adı), Obsidian topluluk eklentileri (kimlik ve ad), VS Code Marketplace ve Open VSX (yayıncı ve uzantı kimliği), alan adları (`rootmark.ai`, `.com`, `.dev`, `.io`, `.app`, `.org`; RDAP).
   - PyPI'daki `rootmark` projesinin ve yakın alandaki GitHub projelerinin (ör. AGENTS.md ile ilgili `rootmark` adlı depo) içeriksiz özeti: ne yaptığı, ne zamandır etkin, bizimle karışma riski.
3. **R2 — Marka taraması.** TÜRKPATENT, EUIPO (TMview), USPTO ve WIPO Global Brand Database'de "rootmark" ve yakın yazımlar; ilgili Nice sınıfları (9, 42). Sonuç her kaynak için kanıtlı satır. Ajanın erişemediği arayüzler `HUMAN_STEP.md` ile insana: hangi sayfada hangi sorgu, sonucun nasıl yazılacağı. **Hukuki görüş değildir**; rapor bunu açıkça yazar.
4. **R3 — Depo ayrımı ve aktarım planı.**
   - **İzin listesi:** ürün deposuna gidecek yollar (çalışma zamanı kodu, istemciler, spec, kurulum, İngilizce kullanıcı belgeleri, lisans dosyaları), laboratuvarda kalacaklar (`research/`, `reports/`, `docs/rounds/`, Türkçe süreç belgeleri, dondurulmuş ölçüm kodu) ve dönüşecekler (ör. `README.md`, `CLAUDE.md` dosyaları, `Makefile`'ın laboratuvara özgü hedefleri). Bugünkü `main` üzerinde sayılar: toplam, giden, kalan, dönüşen, **sınıflandırılmamış 0**.
   - **Aktarım yöntemi:** sürüm başına aktarım (laboratuvar etiketi → ürün deposu commit'i), geçmiş politikası (temiz geçmiş mi, süzülmüş geçmiş mi; gerekçe), aktarımın bayt eşitliğinin nasıl doğrulanacağı, aktarımı kimin tetiklediği (insan adımı).
   - **Ürün deposunun yeri ve CI'ı:** kurum ve depo adı (öneri ve alternatif), hangi iş akışlarının ürün deposunda koşacağı (yayım, köken tasdiki, CI), hangilerinin laboratuvarda kalacağı (kademe 2, Mac kümesi, dondurulmuş ölçümler). Sürüm etiketlerinin ve GitHub sürümlerinin hangi depoda olacağı. Bugün README'deki `arknsensei.github.io/rootmark/spec/` rozet ve bağlantılarının kaderi.
   - **Laboratuvar deposunun kaderi:** adı (`rootmark` kalır mı, `rootmark_dev`/`rootmark-lab` mı), görünürlüğü, yönlendirmeler.
   - **Lisans ve üçüncü taraf:** Apache-2.0 ve `NOTICE`; modellerin (`models/`, MiniLM ailesi) lisansları ve konteyner imajında ya da ilk açılışta indirmenin lisans sonucu.
5. **R4 — Kod adları ve kanallar.**
   - **Kod adlarının kaderi:** `rootmark` ikilisi, crate adları, `rootmark_bindings` Python modülü, `.rootmark/` dizini, spec adı ve sürümü, Obsidian eklenti kimliği, VS Code uzantı kimliği. Her biri için "aynen kalır" ya da "değişir" ve değişirse göç ve uyumluluk notu (K-D0.1).
   - **Dağıtım kanalları planı (D-1 için):** `ghcr.io` ve Docker Hub imaj adları; imaj taban dağıtımı ve mimarileri; modeller imajda mı ilk açılışta mı (boyut, lisans, ağ gereksinimi); Homebrew tap adı (`packaging/homebrew/rootmark.rb.in`'in ürün deposundaki yeri); PyPI (`rootmark-bindings` ya da R1'in önerisi), crates.io ve npm'e yayımlanıp yayımlanmayacağı ve gerekçesi; belge sitesi (`rootmark.ai` altında hangi alt alan adı, barındırma). Her kanal için gereken **insan adımları** ve **gizli değişken adları** (yalnız ad; değer asla), mümkün olan yerde trusted publishing.
   - **P2-Q.2 doğrulamasının önerileri D-1 planına:** Öneri 1 (npm dolaylı bağımlılıklarını da 14 gün kuralına sokmak), Öneri 2 (`SHA256SUMS.local`'ı yayımlanmış varlıklarla tutarlı yapmak), Öneri 3 (her yayımdan önce gerçek Claude Code sınaması). Her biri için D-1'de nereye gireceği.
6. **R5 — PQ2-1 sınaması (ürün kodu değişmez).** Yayımlanmış 0.3.2 ile, D-NOTES kopyası ya da sentetik bir kasada: `serve --watch` açıkken bir not silinir ya da taşınır, kaldırma beklemesi dolmadan sunucu SIGKILL ile öldürülür, sunucu `--watch` ile yeniden başlatılır. Soru: silinen notun blokları yeni başlangıçta kapanıyor mu (`BlockDeleted`, aramada yok)? ≥ 10 tekrar; macOS ve Linux (Docker). Betik `research/d0/`'da, kayıt `reports/rounds/D-0/pq2_1/`. Sonuç PQ2-1'in **kalıcı** mı yoksa **bir sonraki başlangıçta kendiliğinden düzelen** bir hata mı olduğunu belirler; D-1'den önce düzeltme gerekip gerekmediği karar talebine girer (D0-6).
7. **R6 — README iskeleti ve belgeler.**
   - Ürün README'sinin iskeleti (İngilizce): amaç, kapsam, ne değildir, hızlı başlangıç (ikili, Homebrew, Docker), bağlantılar. `reports/rounds/D-0/README_SKELETON.md`. Bugünkü `README.md` değişmez; iskelet D-1'de ürün deposunun README'si olur.
   - PQ2-4: `server/CLAUDE.md`'deki `rmcp` sürümü `=3.6.0`.
8. **R7 — Rapor, gözden geçirici, hazır.**
   - `reports/rounds/D-0/REPORT.md`: R1–R6'nın sonuçları ve **karar listesi** (D0-7): her karar için seçenekler, önerilen seçenek ve gerekçe. Ürün sahibi kararlarını bu listeden verir; kararlar K-D0.x olarak tur kapandıktan sonra orkestratör tarafından yazılır.
   - Gözden geçirici tek tur. Başlık: "her tarama satırı kanıtlı mı; izin listesi bütün dosyaları sınıflandırıyor mu ve ürüne kişisel içerik ya da laboratuvar kaydı sızıyor mu; öneriler gerekçeli mi; dışarıya yazan eylem yapılmış mı".
   - READY; tek PR.

## Önceden kayıtlı çıkış kriterleri
"Koştu" ile "geçti" ayrı sayılır. Kriter, koşan **her** adımın geçmesini ister.

| Kod | Ölçüm | Eşik |
|---|---|---|
| D0-1 | `DESIGN.md` (yöntem, kaynak listesi) R1'in ilk tarama kaydından önce commit'li. | önce |
| D0-2 | **Kayıt defteri taraması:** R1'in her ad × her kayıt defteri hücresi için kanıtlı satır (URL ya da arayüz, UTC zaman, yanıt kodu, özet); erişilemeyenler "erişilemedi" olarak ve nedeniyle. Tarama betikleri depoda, yeniden koşulabilir. | eksik hücre 0; kanıtsız satır 0 |
| D0-3 | **Marka taraması:** dört kaynak için kanıtlı satır (ajanın ya da `HUMAN_STEP.md` ile insanın sorgusu); "hukuki görüş değildir" notu. | 4/4; not var |
| D0-4 | **Aktarım planı:** izin listesi bugünkü `main`'in her izlenen dosyasını sınıflandırıyor (giden, kalan, dönüşen); sınıflandırılmamış 0. Giden kümede `research/`, `reports/`, `docs/rounds/`, kişisel içerik, `.rootmark/` ya da anahtar dosyası yok (betikle denetlenmiş). Aktarım yöntemi, geçmiş politikası, bayt eşitliği denetimi ve insan adımları yazılı. | 0; 0; yazılı |
| D0-5 | **Kod adları ve kanallar:** her kod adı için karar önerisi ve gerekiyorsa göç notu; her dağıtım kanalı için ad, insan adımları, gizli değişken adları (değersiz) ve yayım yöntemi; P2-Q.2 doğrulamasının üç önerisi D-1 planında yerli yerinde. | hepsi yazılı |
| D0-6 | **PQ2-1:** macOS ve Linux'ta ≥ 10 tekrar koştu; sonuç (kalıcı / bir sonraki başlangıçta düzeliyor / karışık) sayılarla; D-1'den önce düzeltme önerisi gerekçeli. | koştu; sayılarla |
| D0-7 | **Rapor ve süreç:** `REPORT.md`'de karar listesi (seçenek, öneri, gerekçe); README iskeleti; PQ2-4 düzeltildi; ürün sınırı (İlkeler) korunmuş — `git diff ROUND_START..HEAD` yalnız izinli yollarda; dışarıya yazan eylem 0; `make test`, `make lint` temiz, turun PR'ında kademe 1 yeşil; gözden geçirici kabul. | hepsi |

## Karar kuralı
- **Hepsi geçti** → rapor ürün sahibine sunulur. Ürün sahibi D0-7'nin karar listesinden kararlarını verir; orkestratör K-D0.x olarak yazar ve D-1'i açar. PQ2-1 kalıcıysa ve rapor D-1'den önce düzeltme öneriyorsa sıra ürün sahibinin kararıyla belirlenir (kısa yama turu 0.3.3 ya da D-1'in kapsamına alınması). Kısa bağımsız doğrulama birleşmeden sonra.
- **D0-2 ya da D0-3 kaldı** (eksik ya da kanıtsız satır) → tur kaldı; eksik satırlar tamamlanmadan isim kararı verilmez.
- **D0-4 kaldı** (sınıflandırılmamış dosya ya da giden kümede sızıntı) → tur kaldı; D-1 açılmaz.
- **D0-6 koşulamadı** → PQ2-1 açık kalır, karar talebi; tur öteki kriterlerle değerlendirilir.
- **Ürün sınırı ihlali ya da dışarıya yazan bir eylem** → tur kaldı; değişiklik geri alınır, karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Hesap, kurum, depo açmak; ad rezerve etmek; paket, imaj ya da sürüm yayımlamak; DNS değişikliği (insan adımı ve D-1).
- Ürün deposunu kurmak, aktarım aracını yazmak, Dockerfile, Homebrew formülü, belge sitesi (D-1).
- PQ2-1'in düzeltmesi (sonuca göre 0.3.3 ya da D-1); PQ2-2, PQ2-3, PQ2-7 (→ P2-C/P2-H).
- Ürün adını değiştirmek: rapor seçenekleri yazar, karar ürün sahibinindir.
- Kişisel içerikle ölçüm.
