# D-0 — Bağımsız Doğrulama (kısa; `docs/rounds/D-0/VERIFY.md` VR1–VR4)

**Doğrulanan commit:** `d7a280669bc6827f83d83d12402914bf966bc3a7` (`main`; PR #60, rebase ile birleşti 2026-11-12T14:47:50Z). Ağacı PR #60'ın başı `89901dd`'ninkiyle eşit (`e71a2985…`); `c1abc66..89901dd` yalnız `docs/rounds/D-0/progress/READY.md`. Turun CI koşusu `43229369117` (PR, `c1abc66`) 10/10 iş yeşil; `main` itmesi `43240925770` (`d7a2806`) yeşil.

Ayrı oturum, kendi scratchpad'imde GitHub'dan temiz klon, ana çalışma dizinine erişim yok, depo kodu değiştirilmedi. Kişisel içerik okunmadı; PQ2-1 sınaması kendi ürettiğim sentetik kasada. Kayıt defterlerine yalnız `GET` (ve `.io` için whois) attım. Kademe 2 ve Mac kümesi VERIFY gereği koşulmadı (ürün kodu değişmedi, VR1). **Marka sorgusu yapılmadı** (görev verenin talimatı: D0-3 ürün sahibinin kararıyla ertelendi; yalnız karar kuralının işlenişi denetlendi). Doğrulama betikleri depoya girmedi (scratchpad): `vr2_requery.py`, `vr3_classify.py`, `vr3_build.sh`, `vr4_probe.py`.

## VR1 — Sınır

- **`git diff b6f5b7b..d7a2806 --stat`:** 49 dosya, +9685/−1. `docs/`, `reports/rounds/D-0/`, `research/d0/` dışında tek yol `server/CLAUDE.md`; değişikliği tek satır: `rmcp =3.5.0` → `=3.6.0` (PQ2-4). Ürün kodu, spec, fixture, `packaging/`, `scripts/`, iş akışları, `Cargo.*`, istemci paket dosyaları değişmedi.
- **Sıra (D0-1):** `main`'de `f47735d` R0 (`DESIGN.md`, yazar zamanı 2026-11-11T20:18:31Z) → `6128686` R1 (ilk tarama kaydı, 20:24:11Z). Rebase SHA'ları değiştirdi; FINAL'in andığı `7306155` (R0) ve `1fc1123` (R1) PR #60'ın dalında var (`pull/60/head`), zamanlar aynı. R1 kaydına göre ilk tarama koşusu 20:19:48Z'de başladı — `DESIGN.md` commit'inden 77 s sonra. Depodaki `registry_scan.json` ikinci koşudur (20:47:24Z → 20:50:37Z, `script_commit` `d90bd16`). **Önce: evet.**
- **Gizli bilgi ve kişisel içerik:** eklenen satırlarda token kalıpları (`gh*_`, `github_pat_`, `AKIA…`, özel anahtar başlığı), parola, kişisel e-posta, `/Users/<x>/`, `/home/<x>/` tarandı. Tek e-posta rol adresi `security@rootmark.ai`; yol olarak yalnız kılavuzun kurgu kullanıcıları (`ada`, `rmhuman`); `arknsensei` herkese açık depo sahibinin GitHub adı (depo URL'si). **Bulgu yok.**
- **Dışarıya yazan eylem:** raporun §9'u ile tutarlı; kayıt defterlerinde turun bıraktığı bir iz görmedim (VR2'deki iki yeni hesap ürün sahibinin insan adımı, aşağıda).

## VR2 — Tarama

Kendi betiğim (`vr2_requery.py`; URL'leri ve sınıflamayı kendisi kurar, turun betiğini çağırmaz): 6 seçilmiş hücre + 14 rastgele hücre (tohum `20261112`), 2026-11-12T19:12–19:13Z. Turun satırı 2026-11-11T20:47–20:50Z.

| Ad | Sütun | Rapor | Benim | Aynı mı | Açıklama |
|---|---|---|---|---|---|
| `rootmark` | pypi | dolu / 200 ("Rootmark SDK -- agent instrumentation", 0.1.5) | dolu / 200 (aynı özet, 0.1.5) | evet | |
| `rootmark` | github | dolu / 200 (User `RootMark`, 2018-03-08, 0 depo) | dolu / 200 (aynı) | evet | |
| `rootmark-ai` | github | boş / 404 | **dolu / 200** — **User** `rootmark-ai`, oluşturma 2026-11-11T21:30:55Z, 1 herkese açık depo; `orgs/rootmark-ai` 404 | hayır | zaman farkı: hesap taramadan 40 dk sonra açıldı (ürün sahibinin insan adımı). **Kurum değil, kullanıcı hesabı** (Öneri 1) |
| `rootmark` | dockerhub | boş / 404 | **dolu / 200** — `type: User`, katılım 2026-11-11T21:45:40Z | hayır | zaman farkı: taramadan sonra açıldı (insan adımı); kurum değil, kullanıcı (Öneri 1) |
| `rootmark` | dom_ai | dolu / 200 (2026-11-10 → 2028-11-10) | dolu / 200 (aynı tarihler) | evet | |
| `rootmark` | dom_com | dolu / 200 (2004-03-24 → 2027-03-24) | dolu / 200 (aynı) | evet | |
| `rootmark-core` | brew_cask | boş / 404 | 503 → tam liste (`api/cask.json`, 7 827 cask): yok | evet (boş) | formulae.brew.sh bugün bilinmeyen adlara 503 dönüyor; uydurma kontrol adı `zzqnonexist-x7` de 503, `firefox` 200. Tam listeyle kesinleştirildi |
| `rootmark-core` | brew_formula | boş / 404 | 503, yeniden denemede 404; tam liste (8 731): yok | evet | aynı |
| `rootmark-bindings` | brew_cask | boş / 404 | 503 → tam liste: yok | evet | aynı |
| `rootmarkhq` | openvsx_ns | boş / 404 | boş / 404 | evet | |
| `rootmark-ai` | vscode_ext | boş / 404 | boş / 404 | evet | |
| `rootmark-server` | dockerhub | boş / 404 | boş / 404 (`/v2/orgs/` de 404) | evet | |
| `rootmarkhq` | dom_org | boş / 404 | boş / 404 | evet | |
| `rootmarkhq` | pypi | boş / 404 | boş / 404 | evet | |
| `rootmark-core` | dom_dev | boş / 404 | boş / 404 | evet | |
| `getrootmark` | openvsx_ns | boş / 404 | boş / 404 | evet | |
| `rootmarkhq` | github | boş / 404 | boş / 404 | evet | |
| `rootmark-core` | dom_com | boş / 404 | boş / 404 | evet | |
| `rootmark-server` | dom_dev | boş / 404 | boş / 404 | evet | |
| `rootmark-index` | dom_org | boş / 404 | boş / 404 | evet | |

**Sonuç:** 20 hücrenin 18'i aynı (üçü Homebrew'un bugünkü 503'ü yüzünden tam listeyle kesinleştirildi). Farklı iki hücrenin ikisi de zamanla açıklanıyor: hesaplar taramadan sonra ürün sahibince açıldı (GOAL: "ürün sahibi hesapları açarken bu turu beklemez"). Turun satırlarında hata yok. Ancak raporun ve kanalların planı bu adları **kurum** olarak varsayıyor (D0-K3 "GitHub kurumu `rootmark-ai`", D0-K6 "Docker Hub kurum adı"); bugün ikisi de **kullanıcı** hesabı (Öneri 1).

**Marka:** VERIFY "dört kaynaktan en az birini kendin sorgula" diyor; görev verenin talimatıyla **koşulmadı**. Karar kuralının işlenişi denetlendi (VR4 sonu).

## VR3 — Aktarım planı

Kendi sınıflandırıcım (`vr3_classify.py`): `git ls-tree -r` (`core.quotepath=off`), DESIGN §3'ün desen dili (`dir/` önek, öteki desenler `fnmatch` ya da tam yol, en uzun desen kazanır, eşit uzunlukta iki sınıf = çift), `transforms.md`'nin ana tablosu. Turun `classify.py`'sini kullanmadım.

| Commit | Toplam | Giden | Kalan | Dönüşen | Sınıflandırılmamış | Çift | Ölü desen |
|---|---|---|---|---|---|---|---|
| `b6f5b7b` (ROUND_START) | 16 584 | 589 | 15 351 | 644 | **0** | 0 | 0 |
| `d7a2806` (bugünkü `main`) | 16 631 | 589 | 15 398 | 644 | **0** | 0 | 0 |

ROUND_START sayıları raporla birebir aynı; bugünkü `main`'de fark yalnız turun eklediği 47 dosya, hepsi "kalan". (İlk koşumda `core.quotepath` açıkken Türkçe adlı bir dosya — `docs/rounds/…/dizin adı/t7.md` — eşleşmedi; bu benim betiğimin hatasıydı, düzeltilince 0.)

**Sızıntı (giden + dönüşen, 1 233 dosya, ürün yollarıyla):** `research/`, `reports/`, `docs/rounds/`, `docs/archive/` öneki 0; `.rootmark` bileşeni 0 (K dönüşümü `index_ort/vault/.rootmark/` → `dot-rootmark/` uygulanınca); `server.token` 0; anahtar adları (`*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa*`, `id_ed25519*`, `.env*`, `*secret*`) 0; içerikte ev dizinim ve kullanıcı adım 0 dosya; token kalıpları 0. `/Users/<x>/`, `/home/<x>/`: yalnız `docs/GUIDE.md`'de `ada` ve `rmhuman` (`transforms.md` istisnalarında gerekçeli). E-posta: yalnız `support@dendron.so` (D0F-8'in MIT verisinde, kuruluş adresi). **Sızıntı 0.**

**Ayrı dizinde derleme** (`vr3_build.sh`, kendi `CARGO_TARGET_DIR`'ım):

| Küme | `cargo build --release` | `clients/obsidian` (`npm ci && npm run build`) | `clients/vscode` |
|---|---|---|---|
| A — yalnız izin listesi (589) | **düştü**: `server/Cargo.toml` yok | geçti | geçti |
| B — izin listesi + dönüşenler ham halleriyle ürün yollarında (1 233) | geçti (32 s; `rootmark 0.3.2`) | geçti | geçti |

**Eksik kalan (A):** `server/Cargo.toml` dönüşen kümede (M: `repository` alanı) olduğu için yalnız izin listesiyle çalışma alanı açılmıyor; başka eksik yok — `core`, `index`, `bindings` ve istemciler izin listesiyle tam. Turun `export_check.sh`'i de B kümesini derliyor (dönüşenler uygulanmadan, "D-1 ağacının alt sınırı") ve sonucu aynı (`export_check.json`: 31 s, iki istemci ok). REPORT §3'ün "giden 589 … giden küme ayrı dizinde derleniyor" cümlesi B kümesi için doğru, 589'luk küme için değil (Fark 2, Öneri 2).

Yöntem, geçmiş politikası (temiz geçmiş, sürüm başına tek commit), bayt eşitliği (ağaç kimliği) ve insan adımları `TRANSFER_PLAN.md` §1–6'da yazılı.

## VR4 — PQ2-1 ve öneriler

**PQ2-1, kendi probum (`vr4_probe.py`), yayımlanmış 0.3.2:** `gh release download v0.3.2` → `rootmark-0.3.2-aarch64-apple-darwin.tar.gz`, `SHA256SUMS` ile `OK`; `rootmark --version` = `rootmark 0.3.2`; model `scripts/fetch_model.sh` (sha256 ok). Kasa kendi ürettiğim sentetik kasa (16 not, turunkinden farklı içerik; hedef not `garden/kestrel.md`, 4 blok, eşsiz sözcük aramada 2 blok). Her koşu taze kopya: `serve --watch` hazır → sil ya da taşı (`archive/kestrel-moved.md`) → durdur → `serve --watch` yeniden, hazır + 10 s → `events.jsonl`'de o yolun `BlockDeleted` sayısı ve aramada o yolun isabeti. macOS arm64 (kendi Mac'im), 2026-11-12.

| Durum | Koşu | Sonuç |
|---|---|---|
| sil + SIGKILL (işlemden 0,501–0,507 s sonra) | 6 | **6/6 kalıcı** — durdurmadan önce `BlockDeleted` 0, yeniden başlatmadan sonra 0, aramada 2 isabet |
| taşı + SIGKILL | 6 | **6/6 kalıcı** — eski yol aramada 2, yeni yol 2 (iki kopya canlı) |
| sil + SIGTERM 5 s (kontrol) | 3 | 3/3 alındı (durdurmadan önce `BlockDeleted` 4, aramada 0) |
| taşı + SIGTERM 5 s (kontrol) | 3 | 3/3 alındı (eski yol 0, yeni yol 2) |
| **ek:** sil + SIGTERM 0,5 s (bekleme dolmadan düzgün durma) | 3 | 3/3 kapandı — `BlockDeleted` durdurmadan önce 0, durunca 4: SIGTERM bekleyen kaldırmayı boşaltıyor |

**Raporla aynı:** SIGKILL koşularının 12/12'si kalıcı (raporda macOS 20/20), kontroller alındı; kendiliğinden düzelen 0, sınama geçersizi 0. Ek koşu, gözden geçiricinin N4'te açık bıraktığı sorunun yarısını yanıtlıyor: SIGTERM ~0,5 s'de bile bekleyen kaldırmayı yazıyor, yani hata yalnız SIGKILL'de (ve sunucu kapalıyken yapılan değişikliklerde, D0F-5). Gerçek istemci dizisini (SIGINT → SIGTERM → ~0,5 s → SIGKILL) koşmadım.

**Önerilerin dayanağı (karar listesi D0-K1..K18, örnekleme):**
- D0-K1..K3: kayıt defteri satırlarına dayanıyor (VR2'de doğrulandı); "marka taraması temizse" koşulu ve "karar açık" ibaresi kuralla uyumlu. D0-K3 ve D0-K6'nın "kurum" varsayımı bugün tutmuyor (F1, Öneri 1).
- D0-K7: model 90 871 704 bayt (≈ 91 MB), lisans `apache-2.0` (`model_licenses.json`, HF API; kendi indirdiğim kopya 97 MB disk), yerinde.
- D0-K8: HTTP'nin yalnız 127.0.0.1 olduğu kodda (`server/src/http.rs` satır 3, 60, 223); yerinde.
- D0-K10: PQ2-1'in kalıcılığı benim koşumda da var; gerekçe yerinde.
- D0-K18 ve D0F-2: 0.3.2 arşivi yalnız `rootmark`, `LICENSE`, `README.md`, `models/MANIFEST.json`, `models/fetch_model.sh` içeriyor (`tar -tzf`): üçüncü taraf bildirimi yok; `support@dendron.so` MIT verisinde (VR3). Yerinde.

**P2-Q.2 doğrulamasının üç önerisi:** `CHANNELS.md` §5'te her biri D-1'deki yeriyle ve ölçülebilir bir D-1 kriteriyle yazılı — Ö1 `check_dep_age.sh` (M dönüşümü, `transforms.md` satırıyla tutarlı) + "dolaylı dahil yaş ihlali 0"; Ö2 tek `SHA256SUMS`, `SHA256SUMS.local` kalkar + "MISMATCH 0"; Ö3 yayım kapısında R21 (`claude_probe.py`, ≤ 7 gün) + `RELEASING.md` insan adımı. **Yerli yerinde.**

## Karar kuralının bağımsız türetimi

GOAL'den: D0-3'ün eşiği "4/4; not var". `R2_trademark_human.md` (ürün sahibinin 2026-11-12 yanıtı): H2 ertelendi, kanıtlı satır yok → D0-3 **kaldı**. Kural: "**D0-2 ya da D0-3 kaldı** (eksik ya da kanıtsız satır) → tur kaldı; eksik satırlar tamamlanmadan isim kararı verilmez." Ertelemenin ayrı bir dalı yok ("D0-6 koşulamadı" dalı yalnız PQ2-1 içindir); tanımlanmamış kalış da "tur kaldı"ya varır. Öteki kriterler (aşağıdaki tablo) geçti; ürün sınırı ihlali ve dışarıya yazan eylem yok. **Benim dalım: "tur kaldı; isim kararları (D0-K1, K2, K3, K15) H2'den sonra" — FINAL'inkiyle aynı.** FINAL ürün sahibinin "doğrulanmamış RootMark Inc" gözlemini satır saymamış, `trademark.md`'ye ve D0-K1'e risk olarak yazmış: kuralla uyumlu. FINAL'in "isimden bağımsız kararlar (D0-K10, K18) ürün sahibinin takdirinde" cümlesi GOAL'ün yasakladığı bir şey değil; kural yalnız isim kararlarını bekletir.

**Vaat sorusu (NORTH_STAR):** PQ2-1/D0F-5, "dosyalar gerçek kaynaktır" sözünü doğrudan kırıyor (aramada olmayan not, `blame`'de silinmiş içerik canlı) ve ilk halka açık sürümün varsayılan kipinde (MCP stdio, istemcinin SIGKILL'i) oluşabiliyor; raporun 0.3.3 önerisi bu açıdan yerinde.

## Sonuç

**İhlal:** yok. Ürün sınırı korunmuş, gizli bilgi ve kişisel içerik yok, dışarıya yazan eylem yok.

**Farklar (hükmü değiştirmez):**
- **F1 — Hesap türleri:** GitHub `rootmark-ai` ve Docker Hub `rootmark` taramadan sonra açılmış; ikisi de **kullanıcı** hesabı, plan kurum varsayıyor (VR2).
- **F2 — "Giden küme"nin tanımı:** 589 dosyalık izin listesi tek başına derlenmiyor (`server/Cargo.toml` dönüşenlerde); raporun "derleniyor" cümlesi dönüşenlerin ham halleriyle eklendiği 1 233 dosyalık küme için doğru (VR3).
- **F3 — SHA'lar:** FINAL ve READY dalın SHA'larını anıyor (`7306155`, `1fc1123`, `c1abc66`); rebase birleştirmesinden sonra `main`'de bunlar `f47735d`, `6128686`, `a4278a2`. Zamanlar ve ağaç aynı; önceki turlarda da böyleydi.

**Şüphe:** yok.

| Kod | Hüküm | Gerekçe (bağımsız) |
|---|---|---|
| D0-1 | **geçti** | `DESIGN.md` (20:18:31Z) ilk tarama koşusundan (20:19:48Z) ve ilk tarama kaydından (20:24:11Z) önce commit'li (VR1) |
| D0-2 | **geçti** | 216/216 satır; kendi sorgumla 20 hücrenin 18'i aynı, 2 fark zamanla açıklanıyor; erişilemeyen 4 hücre nedeniyle yazılı (VR2) |
| D0-3 | **kaldı** (ertelendi) | 0/4 kanıtlı satır, not var; ürün sahibinin kararı; kural doğru işlenmiş (kendim sorgulamadım, talimat gereği) |
| D0-4 | **geçti** | kendi sınıflandırıcımla sınıflandırılmamış 0, çift 0, sızıntı 0 (ROUND_START ve bugünkü `main`); dönüşenlerle birlikte küme ayrı dizinde derleniyor; yöntem yazılı (VR3; F2) |
| D0-5 | **geçti** | kod adları (CHANNELS §1), kanallar ve gizli değişken adları (§2, §6), P2-Q.2'nin üç önerisi (§5) yazılı (VR4) |
| D0-6 | **geçti** | PQ2-1 kendi probumla 0.3.2'de yeniden üretildi: SIGKILL 12/12 kalıcı, SIGTERM kontrolü 6/6 alındı (VR4) |
| D0-7 | **geçti** | karar listesi D0-K1..K18, README iskeleti, PQ2-4, sınır temiz, PR'da kademe 1 yeşil (`43229369117`), gözden geçirici KABUL |

**Karar kuralının dalı (bağımsız):** "D0-2 ya da D0-3 kaldı → tur kaldı" — FINAL'le aynı.

## Hüküm: **güvenilir**

Turun ölçtüğü her şey (tarama, sınıflandırma, sızıntı, derleme, PQ2-1) bağımsız olarak aynı sonucu verdi; farklar ya zamanla açıklanıyor ya da sözcük düzeyinde. Turun "kaldı" sonucu karar kuralının doğru uygulanmasıdır, raporun güvenilirliğini düşürmez.

## Öneriler (≤ 3)

1. **Hesap türlerini D-1'den önce netleştir.** GitHub `rootmark-ai` bugün bir kullanıcı hesabı (kurum değil) ve Docker Hub `rootmark` da kullanıcı. D-1'in planı (D0-K3, D0-K6, D0-K11, TRANSFER_PLAN §6, CHANNELS §2) kurum varsayıyor: üye/ekip izinleri, kurum düzeyinde gizli değişkenler, `ghcr.io` paket görünürlüğü ve köken tasdiki için kurum ayarları. Ürün sahibi hesabı kuruma çevirsin (GitHub "Turn into an organization" ya da yeni kurum) ya da D-1 planı kullanıcı hesabına göre güncellensin; karar K-D0.x'e yazılsın.
2. **D-1'in aktarım kriterinde "giden küme"yi dönüşenlerle birlikte tanımla.** "İzin listesi + dönüşenler uygulanmış haliyle" ağacın derlenmesi ve testlerinin geçmesi ölçülsün (bugünkü denetim dönüşenleri ham haliyle ekliyor, `research/data` yolları dönüşmeden testler koşulamaz).
3. **0.3.3'ün regresyon kümesine dört durumu birden koy.** SIGKILL (sil ve taşı), SIGTERM ~0,5 s (bugün geçiyor; düzeltme bunu bozmamalı), gerçek istemci dizisi SIGINT → SIGTERM → ~0,5 s → SIGKILL ve D0F-5'in sunucu kapalıyken düzenle/sil/ekle durumları; `pq2_1_probe.py` ve `offline_probe.py` ürün testine (`server/tests/`) çevrilsin ki aktarımla ürün deposuna da gitsin.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
