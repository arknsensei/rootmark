# D-0 — Tasarım (R0; R1'in ilk tarama kaydından önce commit'lenir, D0-1)

`ROUND_START` = `b6f5b7b` (PR #59'un birleşmiş `main`'i). Bu belge yöntemi ve kaynakları **taramadan önce** sabitler. Kalibrasyon: uç noktaların dolu/boş davranışı R0'da yalnız **kontrol adlarıyla** sınandı (dolu bilinen `serde`, `requests`, `react`, `bitnami`, `redhat`, `jq`, `firefox`, `example.com`; boş bilinen uydurma `zzqnonexist-x7`); "rootmark" ya da listedeki herhangi bir ad R0'da sorgulanmadı. Kalibrasyonun sonuçları §1.3'teki "dolu/boş" sütununa işlendi.

## 1. R1 — Ad × kayıt defteri matrisi

### 1.1 Adlar
| Ad | Neden |
|---|---|
| `rootmark` | ürün adı (K-D0.1) |
| `rootmark-bindings` | bugünkü wheel dağıtım adı (`rootmark_bindings`) |
| `rootmark-core`, `rootmark-index`, `rootmark-server` | bugünkü crate adları |
| `rootmark-ai` | GitHub kurum adayı (K-P2Q2.2) |
| `getrootmark` | yaygın "get<ad>" kurum/alan adı kalıbı |
| `rhizomark` (yedek 1) | orkestratörün 2026-11-10 ön taramasında anılan alternatif ürün adı; R1 "rootmark"ta ciddi bir çakışma bulursa raporun alternatifi |
| `rootmarkhq` (yedek 2) | `rootmark-ai` alınamazsa ya da tireli ad istenmezse kurum/yayıncı adı kalıbı ("<ad>hq") |

Yedekler GOAL'ün "en çok iki" sınırındadır ve aynı koşuda, aynı yöntemle taranır. R1'in bulgusu başka bir yedek gerektirirse (ör. `rhizomark` da doluysa) bu belgeye ek yazılır ve gerekçesi `ASSUMPTIONS.md`'ye girer; sayı ikiyi geçmez.

### 1.2 Kayıt defterleri (sütunlar)
Matris **9 ad × 24 sütun = 216 hücre**dir; her hücre bir satır olur. Ad bir kayıt defterinin ad kuralına aykırıysa sorgu yine atılır ve satır `invalid_name` notu taşır (ör. Docker Hub kimliği tire almaz) — hücre boş bırakılmaz.

| # | Sütun | Sorgu (yalnız `GET`) | Dolu | Boş | Ad normalizasyonu / not |
|---|---|---|---|---|---|
| 1 | `crates.io` | `https://crates.io/api/v1/crates/<ad>` (User-Agent zorunlu) | 200 | 404 | büyük/küçük harf ve `-`/`_` eşdeğer (crates.io kuralı); yanıt kanonik adı verir |
| 2 | `pypi` | `https://pypi.org/pypi/<ad>/json` | 200 | 404 | PEP 503: küçük harf, `[-_.]+` → `-`; PyPI benzer adları ayrıca reddedebilir (API'de görünmez, raporda not) |
| 3 | `testpypi` | `https://test.pypi.org/pypi/<ad>/json` | 200 | 404 | aynı |
| 4 | `npm_pkg` | `https://registry.npmjs.org/<ad>` | 200 | 404 | yalnız küçük harf; npm noktalama farkıyla aynılaşan adları reddeder → ayrıca noktalaması silinmiş biçim (`rootmarkcore` …) sorgulanır, sonuç aynı satırda `punct_variant` |
| 5 | `npm_org` | `https://registry.npmjs.org/-/org/<ad>/package` | 200 | 404 | `@<ad>` kapsamı kurum olarak |
| 6 | `npm_user` | `https://www.npmjs.com/~<ad>` | — | — | kalibrasyonda 403 (tarayıcı denetimi, kontrol adlarında da); herkese açık API'de kullanıcı varlığı yok → satır `unreachable` + `HUMAN_STEP.md`; zayıf kanıt olarak `https://registry.npmjs.org/-/v1/search?text=maintainer:<ad>` toplamı yazılır (0 ≠ boş) |
| 7 | `dockerhub` | `https://hub.docker.com/v2/users/<ad>/` (kurumsa `/v2/orgs/<ad>`'a yönlenir) | 200 (yönlenme hedefi kaydedilir: kullanıcı/kurum) | 404 | Docker kimliği 4–30, `[a-z0-9]` |
| 8 | `ghcr` | anonim belirteç `https://ghcr.io/token?scope=repository:<ad>/rootmark:pull`, sonra `https://ghcr.io/v2/<ad>/rootmark/tags/list` | 200 (herkese açık imaj var) | 401/403/404 (herkese açık imaj yok) | `ghcr.io` ad alanı aynı adlı GitHub hesabınındır; ad alanının sahibi sütun 9'dan okunur, satır ikisini birlikte yazar |
| 9 | `github` | `https://api.github.com/users/<ad>` (`gh api` ile; kimlikli ama salt okuma `GET`) | 200 (`type` User/Organization, `created_at`, `public_repos`) | 404 | büyük/küçük harf duyarsız |
| 10 | `brew_formula` | `https://formulae.brew.sh/api/formula/<ad>.json` | 200 | 404 | homebrew-core |
| 11 | `brew_cask` | `https://formulae.brew.sh/api/cask/<ad>.json` | 200 | 404 | homebrew-cask |
| 12 | `brew_tap` | `https://api.github.com/repos/<ad>/homebrew-tap` ve `…/<ad>/homebrew-rootmark` | biri 200 | ikisi 404 | tap = GitHub deposu `<sahip>/homebrew-<tap>`; sahibi hesap sütun 9 |
| 13 | `obsidian_id` | `https://raw.githubusercontent.com/obsidianmd/obsidian-releases/master/community-plugins.json` (+ `community-plugins-removed.json`) | `id` eşit | yok | tek indirme, bütün adlar için; dosyanın sha256'sı satıra yazılır |
| 14 | `obsidian_name` | aynı dosyalar | `name` büyük/küçük harf duyarsız eşit | yok | |
| 15 | `vscode_publisher` | `https://marketplace.visualstudio.com/publishers/<ad>` | 200 | 404 | |
| 16 | `vscode_ext` | `https://marketplace.visualstudio.com/items?itemName=<ad>.rootmark` | 200 | 404 | uzantı kimliği `<yayıncı>.rootmark`; yayıncı = ad |
| 17 | `openvsx_ns` | `https://open-vsx.org/api/<ad>` | 200 | 404 | |
| 18 | `openvsx_ext` | `https://open-vsx.org/api/<ad>/rootmark` | 200 | 404 | |
| 19–24 | `dom_ai`, `dom_com`, `dom_dev`, `dom_io`, `dom_app`, `dom_org` | RDAP: IANA önyüklemesi `https://data.iana.org/rdap/dns.json` → yetkili sunucu `…/domain/<ad>.<tld>`; `.io` önyüklemede yok (kalibrasyon: `rdap.org` 404) → `whois -h whois.nic.io <ad>.io` (port 43, salt okuma) | 200 / whois kaydı | 404 / `Domain not found.` | yalnız durum, kayıt ve bitiş tarihi, kayıt şirketi; kayıt sahibi bilgisi yazılmaz |

### 1.3 Satır biçimi (`reports/rounds/D-0/registry_scan.json`)
Her satır: `name`, `registry`, `url` (ya da `whois:<sunucu>`), `queried_at` (UTC, ISO 8601, saniye), `http_status` (ya da whois için `null` ve `whois_status`), `state` (`taken` / `free` / `unreachable` / `invalid_name` ile birlikte `taken`|`free`), `summary` (içeriksiz: sahibin türü ve herkese açık tanıtıcısı ya da proje URL'si, oluşturma/ilk ve son sürüm tarihi, son sürüm numarası; e-posta, kişi adı ve gövde metni **yazılmaz**), `body_sha256` (yanıt gövdesinin sağlaması; kanıtın izi, içerik değil), `note`. Başlıkta: betiğin commit'i, koşu başlangıç/bitiş zamanı, Python sürümü, User-Agent (`rootmark-d0-registry-scan/1 (read-only)`; kişisel bilgi yok). Okunur tablo `registry_scan.md` aynı JSON'dan üretilir.

**Kurallar:** yanıt kodu 200/404 dışındaysa (429, 5xx, zaman aşımı) üç denemeye kadar artan beklemeyle yeniden sorulur; yine olmazsa satır `unreachable` ve nedeniyle (kod ya da istisna sınıfı) yazılır — uydurulmaz, boş sayılmaz. GitHub isteği `gh api` (yerel kimlik, salt okuma) ile; `gh` yoksa kimliksiz `urllib` ile. Betik bir kayıt defterine `GET` dışında istek atmaz (betikte tek istek işlevi; yöntem sabit `GET`). Betik yeniden koşulabilir: `python3 -I research/d0/registry_scan.py --out reports/rounds/D-0`.

### 1.4 Yakın alan (içeriksiz özet)
- PyPI `rootmark`: JSON'dan `summary`, `home_page`/`project_urls` (yalnız URL), ilk ve son sürüm tarihi, sürüm sayısı, son sürüm. Kişi adı ve e-posta yazılmaz.
- GitHub: `https://api.github.com/search/repositories?q=rootmark+in:name` (ilk 30, yıldıza göre) — her depo için `full_name`, `description`'ın ilk 120 karakteri (içerik değil, depo tanıtımı), `created_at`, `pushed_at`, yıldız; AGENTS.md ile ilgili `rootmark` deposu dahil. Bizimle karışma riski elle değerlendirilir (rapor).
- `rootmark.dev`, `rootmark.com`: RDAP satırları + `https://<alan>/` için tek `GET` (yanıt kodu ya da bağlantı hatası; gövde yazılmaz).

## 2. R2 — Marka taraması
**Kaynaklar:** TÜRKPATENT (marka araştırma), EUIPO TMview, USPTO (Trademark Search), WIPO Global Brand Database. **Sorgular:** `rootmark`, `root mark`, `rootmarks`, `rhizomark`; Nice sınıfları 9 ve 42 (bütün sınıflar da listelenir, 9/42 ayrıca işaretlenir); durum: başvuru, tescil, geçersiz.
**Yöntem:** önce her kaynak için belgelenmiş herkese açık bir `GET` uç noktası denenir (ör. arayüzün JSON arka ucu bir `GET` kabul ediyorsa). Arayüz tarayıcı (JavaScript, captcha, e-Devlet) gerektiriyorsa ya da arama yalnız `POST` ile yapılabiliyorsa ajan sorgulamaz: `HUMAN_STEP.md` sayfayı, sorguyu ve sonucun `progress/R2_trademark_human.md`'ye yazılış biçimini adım adım verir (kaynak, sorgu, tarih UTC, sonuç sayısı, her eşleşme için marka, sınıf, durum, sahip ülkesi, başvuru no; ekran görüntüsü yolu isteğe bağlı, depoya girmez). TÜRKPATENT baştan insan adımıdır (GOAL).
**Çıktı:** `reports/rounds/D-0/trademark.md`: kaynak başına satır (kim sorguladı, URL/arayüz, UTC, sonuç) ve **"Bu tarama hukuki görüş değildir"** notu.

## 3. R3 — İzin listesi ve denetim
**Biçim:** üç dosya `reports/rounds/D-0/transfer/`:
- `allowlist.txt` — ürün deposuna **aynen** (bayt eşit) gidenler;
- `stays.txt` — laboratuvarda kalanlar;
- `transforms.md` — dönüşerek gidenler: tablo (`| desen | dönüşüm | ürün deposundaki yol |`).
**Desen dili:** satır başına bir desen; `#` yorum; `dir/` önek (o dizinin altındaki her şey); başka her desen `fnmatch` (`*` `/`'u da eşler) ve tam yol olarak denenir. Bir dosyanın sınıfı onu eşleyen desenlerin **en uzununun** (en özel) sınıfıdır; aynı uzunlukta iki sınıf eşlerse **çift sınıflı** sayılır (hata).
**`research/d0/classify.py`** (`git ls-files` @ verilen commit; varsayılan `ROUND_START`): her dosyaya tek sınıf; `classification.json`: toplam, giden, kalan, dönüşen, sınıflandırılmamış (=0 olmalı), çift sınıflı (=0 olmalı), dizin başına sayılar, hiçbir dosyayı eşlemeyen ölü desenler. **Sızıntı denetimi** (giden + dönüşen küme): yol önekleri `research/`, `reports/`, `docs/rounds/`, `docs/archive/`; yol bileşeni `.rootmark`; adı `server.token`; anahtar desenleri (`*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa*`, `id_ed25519*`, `.env*`, `*secret*`); içerikte çalışan kullanıcının ev dizini ve kullanıcı adı (çalışma anında okunur, kayda **yalnız sayı** yazılır) ve genel `/Users/<x>/`, `/home/<x>/` dizgileri — ikincisi bulunursa dosya ve `<x>` listelenir, kurgu örnek (ör. kılavuzun `ada`'sı) olarak gerekçesi `transforms.md`'nin "istisnalar" bölümünde yazılı değilse hata. Çıkış 0 yalnız hepsi 0 ise.

## 4. R5 — PQ2-1 sınaması
**İkili:** `gh release download v0.3.2` (`rootmark-0.3.2-aarch64-apple-darwin.tar.gz`, `…-x86_64-unknown-linux-gnu.tar.gz`, `SHA256SUMS`), `shasum -a 256 -c` geçmezse dur. Model: `models/MANIFEST.json`'ın sabit sürümü (`scripts/fetch_model.sh`'in indirdiği, sha256 denetimli; mevcut yerel kopya varsa manifestle denetlenip kullanılır).
**Kasa:** sentetik (scratchpad'de üretilen, 20 not; içerik kişisel değil). Her tekrar taze kopya.
**Adımlar (bir tekrar):** (1) `ingest-dir` + `index build`; (2) `serve --vault V --http 127.0.0.1:<boş port> --watch` arka planda; `GET /v1/status` 200 olana ve `watching` satırı gelene kadar bekle; (3) **silme** ya da (ayrı tekrar) **taşıma** (`mv a.md sub/a-moved.md`, tasdiksiz); (4) olaydan sonra **0,5 s** (izleyicinin olayı gördüğü, kaldırma beklemesinin — `window_s` + 0,5 s = 2,5 s — dolmadığı an) `SIGKILL`; (5) `serve … --watch` yeniden; hazır olunca **10 s** bekle (bekleme ×4); (6) gözlem: `events.jsonl`'de o notun bloklarının `BlockDeleted` sayısı (öncesi/sonrası), notun eşsiz bir sözcüğüyle varsayılan `search` sonucu (bulunan blok sayısı), taşımada yeni yolun blokları, `events_hash`; sonra `SIGTERM` ile durdur.
**Karşılaştırma (kontrol):** aynı dizi (3)'ten sonra **5 s** bekleyip `SIGTERM` (düzgün durma).
**Ek gözlem (düzeltme yolu, karar için):** kalıcıysa, sunucu kapalıyken `rootmark ingest-dir V V` yeniden koşulduğunda blokların kapanıp kapanmadığı (tek ölçüm, her platformda).
**Tekrar:** her platformda {sil, taşı} × {SIGKILL, SIGTERM} × 10 = 40 koşu; macOS arm64 (yerel) ve Linux x86_64 (Docker, `research/p2q/scenario.json`'ın sabit `debian:trixie-slim` imajı, `linux/amd64`).
**Sınıflama (tekrar başına):** `recovered` (SIGKILL sonrası yeni başlangıçta `BlockDeleted` var ve aramada yok), `persistent` (yok ve aramada var), `other` (ör. SIGKILL'den önce kaldırma alınmış: sınama geçersiz, ayrı sayılır). **Beklenen gözlem** (koddan, `server/src/serve.rs` `run`: başlangıçta vault'u günlükle uzlaştıran bir tarama görünmüyor; izleyici yalnız yeni olayları alır): `persistent`. Sonuç sayılarla yazılır; beklentiyle çelişirse çelişki raporlanır.
**Kayıt:** `reports/rounds/D-0/pq2_1/{macos,linux}.json` (sayılar, sürüm, komutlar, zamanlar; içerik, mutlak yol, host adı yok) ve özet `summary.json`. Betik `research/d0/pq2_1_probe.py`.

## 5. Dışarıya yazan eylem yok
Bu turdaki her ağ isteği kayıt defterlerine ve marka kaynaklarına salt okuma `GET` (ve `.io` için whois sorgusu), GitHub'dan sürüm indirme (`gh release download`) ve turun kendi dalının itilmesi/PR'ıdır. Hesap, kurum, depo, ad, paket, imaj, etiket ve DNS işi yoktur; gerekenler `HUMAN_STEP.md`'dedir.
