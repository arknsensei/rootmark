# D-1 — Görevler

Genel kurallar:
- **Sıra:** R0 → R1 → R2 → R3 → R4 → R5 → R6 → R7. R0'ın `DESIGN.md`'si commit'lenmeden R1'in kodu yazılmaz. R2, R3, R4, R5 birbirinden bağımsızdır; bir insan adımını beklerken öteki görevlere geçilir. R6 R1–R5'in hepsini ve insan adımlarını ister.
- **Kayıt ve dallar:** progress `docs/rounds/D-1/progress/`, kayıtlar `reports/rounds/D-1/`. Commit öneki `D-1/R<n>:`. Dal `d-1/round`, yalnız birleşmiş `main`'den (K-P2.2). Yayım hazırlığı ayrı dal `release/0.4.0`.
- **Gözden geçirici:** yalnız R7'de, tek tur (K-P2.4).
- **CI yapılandırması:** laboratuvarın `.github/workflows/**` değişiklikleri R0'ın "değişmesi beklenen kayıtlar" listesine o değişiklikten **önce** adıyla eklenir (K-P2Q1.4). Ürün deposunun iş akışları laboratuvarda `product/overlay/.github/workflows/` altında kaplamadır; laboratuvarın CI'ını değiştirmez.
- **Dondurulmuş dosyalar:** `scripts/release_check.sh` ve öteki dondurulmuş dosyalar yalnız DEV_PROCESS §4'ün istisna prosedürüyle (`progress/FREEZE_EXCEPTIONS.md`, kasıtlı bozma sınaması).
- **İnsan adımı ve sorular:** insan adımı beklemek durma değildir (`HUMAN_STEP.md`; birden çok adım numaralı, her birinde beklenen sonuç ve sınama komutu). Soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye. Spec'in ya da kararların tanımlamadığı bir durum: PARK + karar talebi (uydurma yok).
- **Dışarıya yazma:** ajan ürün deposuna, tap deposuna, Docker Hub'a, `ghcr.io`'ya ve DNS'e yazmaz; ürün deposunda dal açmaz, PR açmaz (bunu laboratuvarın `transfer.yml`'i insanın tetiklemesiyle yapar). Okuma isteği serbest.
- **Vault:** sınamalar yalnız sentetik kasada ya da D-NOTES kopyasında (depo dışında, scratchpad). Kişisel vault ve qorbit'e dokunulmaz. Kayıtlarda token, içerik ve host değeri yok.
- **Push:** push 403 verirse kalıcı ayara dokunmadan `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push …`; sürerse `HUMAN_STEP.md`. Zorla itme insanın adımıdır.

## R0 — Açılış ve tasarım
**Yapar:**
- `ROUND_START`; DECISIONS K-D1.1 (açılış).
- `progress/DESIGN.md`:
  - aktarım aracı: `research/transfer/` modülleri, `classify.py`'nin desen mantığının paylaşımı, dönüşüm türleri (K/M/Y; `reports/rounds/D-0/transfer/transforms.md`), `TRANSFER.json` biçimi, `product/overlay/SOURCES.json` (kaplama → kaynağın blob kimliği), ağaç kimliğinin hesabı (`git mktree`/`write-tree`; dosya kipleri), sızıntı denetiminin sınıfları ve istisnaları;
  - `transfer.yml` (laboratuvar) ve `transfer-verify.yml` (ürün): girdiler, yetki (`PRODUCT_REPO_DEPLOY_KEY`), commit mesajı alanları, hata yolları;
  - ürün deposunun iş akışları (TRANSFER_PLAN §3.2): `ci.yml`, `release.yml` (tek `SHA256SUMS`; wheel ve eklenti CI'da), `image.yml`, `pages.yml`, `transfer-verify.yml`; tap PR'ı;
  - imaj: taban özeti, katmanlar, model ve lisansı, kullanıcı, varsayılan komut, boyut tahmini;
  - site: mdBook yapısı, sayfa listesi, "git ile fark" sayfasının iddia → atıf tablosu (taslak), rozetin kaynağı;
  - **değişmesi beklenen laboratuvar kayıtları** adıyla: sürüm dizgileri (0.4.0), E1-4/R14 (etiket), laboratuvar `ci.yml` (dondurma denetimlerinin `deps`'e taşınması), `release_check.sh` (R21, §4).
- Laboratuvar `ci.yml`: `deps` işi `scripts/check_frozen.sh` ve `scripts/check_compare_fields.sh`'i de koşar (gerekirse `fetch-depth: 0` ve Python kurulumu). Kasıtlı bozma: yalnız dondurulmuş bir `.md`'yi değiştiren geçici dal (itilmez ya da itilip silinir; kayıt `reports/rounds/D-1/r0/`) `deps`'te kalır. Laboratuvar `make lint` aynen.
- `progress/FREEZE_EXCEPTIONS.md`: `release_check.sh`'e R21 (DEV_PROCESS §4).
- `HUMAN_STEP.md` (adım 1, ilk gün): hesap ve depo hazırlığı — (a) `rootmark-ai/rootmark` deposu (boş, herkese açık, varsayılan dal `main`; dal koruması: PR + CI + doğrusal geçmiş); (b) `rootmark-ai/homebrew-tap` deposu (boş); (c) dağıtım anahtarı ya da GitHub App → laboratuvar gizli değişkeni `PRODUCT_REPO_DEPLOY_KEY`; (d) Docker Hub `rootmark` hesabında `rootmark/rootmark` deposu ve erişim belirteci → ürün deposu gizli değişkeni `DOCKERHUB_TOKEN`, değişken `DOCKERHUB_USERNAME`; (e) tap için ince taneli belirteç → ürün deposu gizli değişkeni `HOMEBREW_TAP_TOKEN`; (f) DNS: `docs.rootmark.ai` `CNAME` → `rootmark-ai.github.io`, Pages alan adı doğrulaması (TXT); (g) `security@rootmark.ai` rol adresi (yönlendirme). Her biri için ajanın okuyarak yapacağı sınama (ör. `gh api repos/rootmark-ai/rootmark`, `gh secret list -R …` yalnız adlar, `dig`). Değerler sohbete, kayda ya da commit'e girmez.

**Kabul:** D1-1.

## R1 — Aktarım aracı ve `transfer.yml`
**Yapar:**
- `research/transfer/` (Python, `research/` içinde; çalışma zamanı değil): etiket ya da commit → ürün ağacı (geçici dizin), `TRANSFER.json`, sızıntı denetimi, ağaç kimliği; belirlenimli (dosya sırası, kipler, satır sonları); `SOURCES.json` eşleşmezse hata.
- Testler (`research/tests/`): iki koşu aynı kimlik; kaynağı değişmiş kaplama → hata; sızıntı sınıflarının her biri için kasıtlı bozma (ör. `reports/` altından bir dosyanın izin listesine girmesi, ev dizini dizgisi, `.rootmark` bileşeni) → hata.
- Laboratuvar `.github/workflows/transfer.yml` (`workflow_dispatch`, girdi `tag`): ağacı üretir, ikinci kez üretip kimliği karşılaştırır, ürün deposunda `transfer/<tag>` dalına tek commit (ebeveyn ürün `main`'i; ilk aktarımda kök commit), PR açar. Ürün deposu boşken ilk aktarımın yolu DESIGN'da.
- Aktarılmış ağacın tek başına sınaması (D-0 doğrulamasının Öneri 2'si): `ROUND_START`'ta ve turun son ağacında, ağaç laboratuvar dışında bir dizinde: `cargo build --release --locked --workspace`, `cargo test --locked --workspace`, `make conformance` (ürün `Makefile`'ı), iki istemcide `npm ci && npm test && npm run build`. Kayıt `reports/rounds/D-1/r1/`.
- Sınıflandırma sayıları (`classification.json`) turun son ağacında; sınıflandırılmamış 0.

**Kabul:** D1-2'nin yerel yarısı, D1-3.

## R2 — Ürün kaplamaları
**Yapar:**
- `product/overlay/` altında GOAL Kapsam 3'ün dosyaları; her kaplamanın kaynağı (varsa) `SOURCES.json`'da.
- `README.md`: README_SKELETON'dan; ürün iddiaları atıflı; hızlı başlangıç komutları `reports/rounds/D-1/r2/readme_quickstart.sh` ile yazıldığı gibi koşar (sentetik kasa).
- Ürün `release.yml`: laboratuvar `release.yml` + `release.sh`'in ürün yüzü; wheel ve eklenti zip'i CI'da üretilir; tek `SHA256SUMS`; `actions/attest-build-provenance` her arşiv için; `THIRD_PARTY_LICENSES` (ör. `cargo about generate`, sürüm başına) arşivlere girer; tap PR'ı son iş (R5).
- `NOTICE`, `testdata/THIRD_PARTY.md` (K18).
- `scripts/check_dep_age.sh` (laboratuvar ve ürün): iki `package-lock.json`'ın bütün `packages` girdileri ≥ 14 gün; istisnalar `dep_age_exceptions.txt`'de gerekçeyle (Ö1). Laboratuvarda yeni ihlal çıkarsa: istisna gerekçeli ya da sürüm yükseltmesi değil sabitleme (ürün davranışı değişmez).
- Ürün `ci.yml` ve `transfer-verify.yml`.

**Kabul:** D1-4'ün yerel yarısı (aktarılmış ağaçta ürün `ci.yml`'inin komutları yerelde geçer), D1-11'in ağaç yarısı.

## R3 — Belge sitesi
**Yapar:**
- mdBook kaynakları kaplamada (`product/overlay/site/` ya da DESIGN'daki yer); sayfalar GOAL Kapsam 4.
- "git ile fark" sayfası: iddia → atıf tablosu (`progress/R3_claims.md`): her iddia için laboratuvarın kaydı ya da kararı (yol + `v0.4.0` etiketli mutlak URL) ve iddianın geçtiği koşul. Atıfsız iddia 0.
- Kılavuz: laboratuvar `docs/GUIDE.md`'nin İngilizce ürün yüzü; PQ3-3, PQ3-5 (`--watch`'sız sunucu kapalıyken değişiklikleri almaz; ajan yapılandırmaları `--watch`'lı), PQ4-9 (`ROOTMARK_MODELS`) notları; Docker ve Homebrew kurulumu.
- Ürün `pages.yml`; yerelde `mdbook build` ve bağlantı denetimi (kırık iç bağlantı 0); rozet ölçülmüş `conformance.json`'dan.
- Laboratuvar Pages'i için yönlendirme sayfası (K17; yayımı insan adımı, R6'dan sonra).

**Kabul:** D1-9'un yerel yarısı.

## R4 — Konteyner imajı
**Yapar:**
- `product/overlay/Dockerfile` ve `image.yml` (GOAL Kapsam 5).
- Yerelde imajı yapıp kılavuzun dört adımını betikle koşmak (`reports/rounds/D-1/r4/quickstart_{linux,macos}.json`): Linux (Docker, `linux/amd64`) ve macOS Docker Desktop; `--watch` sınaması: konteyner çalışırken birimdeki bir notu düzenle, `search` yeni metni döndürür (iki platform, aralıklı; D0F-6). İmajın boyutu, kullanıcı (`id -u` ≠ 0), model lisans dosyası kayıtta.

**Kabul:** D1-7'nin yerel yarısı.

## R5 — Homebrew tap'i
**Yapar:**
- Formül şablonunun ürün yüzü; ürün `release.yml`'inin son işi (özetleri yayımlanmış arşivlerden; tap deposuna PR, `HOMEBREW_TAP_TOKEN`).
- Yerelde: geçici bir tap dizininde formülün yerel arşivle `brew install --build-from-source`'suz kurulup `brew test`'i geçmesi (yöntem DESIGN'da); kayıt `reports/rounds/D-1/r5/`.

**Kabul:** D1-8'in Homebrew yarısının yerel sınaması.

## R6 — 0.4.0'ın yayımı ve yayımlanmış kanallardan sınama
**Yapar:**
- Laboratuvar: `release/0.4.0` (sürüm dizgileri, CHANGELOG `## 0.4.0` — "ilk halka açık sürüm; ürün davranışı 0.3.4 ile aynı"), PR, `release_check` 22/22 (R21 için güncel Claude Code ile `claude_probe.py`, D-NOTES kopyası), `HUMAN_STEP.md`: birleşme, laboratuvar `release.yml` (laboratuvarın kendi doğrulaması), etiket `v0.4.0`. Laboratuvar 0.4.0 için GitHub sürümü yayımlamaz (TRANSFER_PLAN §3.3).
- `HUMAN_STEP.md`: `transfer.yml`'i `v0.4.0` ile tetiklemek; ürün PR'ında ürün CI'ı ve `transfer-verify` yeşil olunca birleştirmek (doğrusal); ürün etiketi `v0.4.0`; ürün `release.yml` (GitHub sürümü, imaj, tap PR'ı); tap PR'ını birleştirmek; Pages'i etkinleştirmek, özel alan adı ve HTTPS; `ghcr.io` paketini herkese açık yapmak.
- Yayımlanmış kanallardan sınama (GOAL Kapsam 7; kayıtlar `reports/rounds/D-1/r6/`): temiz Linux konteynerinde ve macOS'ta `install.sh`; `brew install rootmark-ai/tap/rootmark` + `brew test`; iki kayıt defterinin özet eşitliği (`docker buildx imagetools inspect`); yayımlanmış imajla dört adım (Linux ve macOS Docker Desktop) ve `--watch`; Obsidian BRAT varlıkları; `sha256sum -c`; `gh attestation verify -R rootmark-ai/rootmark` (arşivler ve imaj); `docs.rootmark.ai` sayfaları (200) ve bağlantı denetimi; gerçek Claude Code ile imajın MCP stdio'su (`propose` + `search`, ≥ 1 oturum, Claude Code sürümü kayıtlı).
- Ağaç kimliği üç yerde (yerel, `transfer.yml`, `transfer-verify`) ve ürün `main`'inin ağacı.
- Güvenlik: iki depoda Dependabot ve CodeQL (`progress/R6_security.md`).
- Laboratuvarın yeniden adlandırılması ve Pages yönlendirmesi: `HUMAN_STEP.md`'de son adım (isteğe bağlı; ürün yayımlandıktan sonra; çıkış kriteri değil).

**Kabul:** D1-2, D1-4, D1-6, D1-7, D1-8, D1-9, D1-10, D1-11.

## R7 — Rapor, gözden geçirici, hazır
**Yapar:**
- Kapılar: `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- `D1-5`: `git diff ROUND_START..HEAD -- core index server bindings spec clients/*/src` (yalnız sürüm dizgileri), `tools/list` 0.3.4 ile bayt eşit.
- `FINDINGS.md`: yeni bulgular; D-0'dan devredilenlerin kapanışı (D0F-2, D0F-8, Ö1–Ö3, D-0 Öneri 2, PQ3-3, PQ3-5, PQ4-9).
- Gözden geçirici tek tur (GOAL başlığı).
- `reports/rounds/D-1/FINAL.md`: D1-1..D1-12, karar kuralının dalı, koşu kimlikleri, "koştu/geçti" ayrımı, ürün deposunun ve sürümün adresleri.
- READY; tek PR: `gh pr create --base main --head d-1/round --title "D-1: ilk halka açık RootMark sürümü (0.4.0)" --body-file reports/rounds/D-1/FINAL.md`. READY'den sonra itilen her commit'in CI'ı okunur ve FINAL'e yazılır.

**Kabul:** D1-1, D1-5, D1-12; READY'de "doğrulama için hazır" + commit sha + PR'ın son commit'inin CI koşu kimliği.
