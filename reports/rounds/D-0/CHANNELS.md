# D-0 — Kod adları ve dağıtım kanalları (R4)

R1'in taraması (`registry_scan.json`) ve R3'ün aktarım planına (`TRANSFER_PLAN.md`) dayanır. Bu belge D-1'in kanal planıdır; D-0 hiçbir kanala yayımlamaz, hiçbir hesap ya da ad almaz. Gizli değişkenlerin yalnız **adları** yazılır.

## 1. Kod adlarının kaderi

| Kod adı | Bugün | Öneri | Gerekçe | Göç / uyumluluk |
|---|---|---|---|---|
| İkili | `rootmark` | **aynen kalır** | ürün adıyla aynı; Homebrew formülü, crates.io, npm, Docker Hub'da `rootmark` boş (R1) | yok |
| Crate'ler | `rootmark-core`, `rootmark-index`, `rootmark-server` | **aynen kalır** | crates.io'da üçü de boş (R1); kaynakta yüzlerce atıf | yok |
| Python dağıtımı / modülü | dağıtım `rootmark-bindings` (wheel `rootmark_bindings`), modül `rootmark_core` | **aynen kalır** | PyPI'da `rootmark` başka bir etkin projenin (ajan enstrümantasyonu, 2026-08'den beri); `rootmark-bindings` boş (R1); modül adı o projenin içe aktarma adıyla çakışmaz | yok |
| Laboratuvar Python paketi | `research/pyproject.toml` `name = "rootmark"` | aynen (yayımlanmaz) | yalnız laboratuvarda yerel kurulum; ürün deposuna gitmez (R3) | yok |
| Vault deposu | `<vault>/.rootmark/` | **aynen kalır** | spec ve bütün vault'ların biçimi; değişirse her kullanıcıya göç (`migrate --rename-store` emsali, `mdkernel` → `rootmark`) | yok |
| Spec adı ve sürümü | "rootmark specification", 2.0.3 (§1–6 donmuş) | **aynen kalır** | ad ürünle aynı; sürüm spec'in kendi sürümü (PACKAGES) | yok; spec'in yayın adresi değişir (TRANSFER_PLAN §3.4) |
| Ortam değişkenleri | `ROOTMARK_MODELS`, `ROOTMARK_MATCHER`, `ROOTMARK_DEFAULT_TOKENIZER` | aynen | kullanıcı yapılandırmaları | yok |
| MCP sunucu adı | `serverInfo.name = "rootmark"` | aynen | istemci yapılandırmaları (`.mcp.json`) bu adı kullanır | yok |
| `status` alanı | `mdk_version` (eski proje adının kalıntısı) | **şimdilik kalır**; bir sonraki sözleşme sürümünde `rootmark_version` eklenip `mdk_version` kullanımdan kaldırılabilir | alan istemci sözleşmesinde (CLIENT_CONTRACT) ve iki istemcide; adı değiştirmek sözleşme değişikliğidir | değişirse: iki alan bir sürüm birlikte, sonra eski alan kalkar (P2-C ya da sonrası; D0F-4) |
| Obsidian eklenti kimliği | `id: "rootmark"`, ad `rootmark` | **aynen kalır** | topluluk listesinde ve kaldırılanlar listesinde yok (R1) | yok |
| VS Code uzantı kimliği | `rootmark-local.rootmark-vscode` (iskelet, yayımlanmamış) | yayımlanacağı zaman yayıncı `rootmark-ai` → kimlik `rootmark-ai.rootmark` (öneri) | yayıncı `rootmark-ai` ve `rootmark-ai.rootmark` Marketplace ve Open VSX'te boş (R1); bugünkü `rootmark-local` yalnız yerel geliştirme yayıncısı | yayımlanmadığı için göç yok; D-1'de değil (§2) |

**Ürün adının yazımı:** belgelerde "RootMark" (K-D0.1), komut ve kimliklerde `rootmark`. Laboratuvarın bugünkü belgeleri "rootmark" yazar; ürün kaplaması (`README`, site) "RootMark"ı kullanır, kod ve çıktılar değişmez.

## 2. Kanallar (D-1 planı)

| Kanal | Ad | D-1'de yayım? | Gerekçe | Yöntem | İnsan adımları | Gizli değişken adları |
|---|---|---|---|---|---|---|
| GitHub sürümü | `rootmark-ai/rootmark` sürümleri | **evet** | kurulumun ana yolu (`install.sh`, arşivler, `SHA256SUMS`, köken tasdiki) | ürün `release.yml` (`workflow_dispatch`), `actions/attest-build-provenance` | kurum ve depo açmak; etiket; tetikleme | yok (`GITHUB_TOKEN`, `id-token: write`, `attestations: write`) |
| `ghcr.io` | `ghcr.io/rootmark-ai/rootmark` | **evet** | kurum adına bağlı, ek hesap gerektirmez; köken tasdiki aynı iş akışında | ürün `image.yml` (yayım etiketinde), `docker/build-push-action`, imaj köken tasdiki (`attest-build-provenance` `push-to-registry`) | paket görünürlüğünü "public" yapmak (ilk yayımdan sonra bir kez, kurum ayarı) | yok (`GITHUB_TOKEN`, `packages: write`) |
| Docker Hub | `rootmark/rootmark` (kurum `rootmark` — Docker kimliği tire almaz; R1: `rootmark` boş, `rootmark-ai` geçersiz ad) | **isteğe bağlı** (K-D0.1) — öneri: evet, `ghcr.io` ile aynı özet (digest) | `docker pull rootmark/rootmark` keşfedilebilirliği | aynı iş akışında ikinci hedef; aynı imaj özeti iki kayıt defterine | Docker Hub hesabı ve `rootmark` kurumu; kurumda erişim belirteci (yalnız "Read & Write", tek depo) | `DOCKERHUB_USERNAME` (değişken), `DOCKERHUB_TOKEN` (gizli) |
| Homebrew tap | depo `rootmark-ai/homebrew-tap`, formül `Formula/rootmark.rb` → `brew install rootmark-ai/tap/rootmark` | **evet** | `packaging/homebrew/rootmark.rb.in` hazır; homebrew-core bilinirlik ister (sonra) | ürün `release.yml`'in son işi formülü doldurur (bugün `release.sh` yapıyor) ve tap deposuna PR açar | tap deposunu açmak; PR'ları birleştirmek | `HOMEBREW_TAP_TOKEN` (yalnız tap deposuna `contents: write` ince taneli belirteç) ya da bir GitHub App |
| PyPI | `rootmark-bindings` | **hayır (D-1'de)**; öneri: ad, ilk gerçek kullanıcı ihtiyacında yayımlanır | bağlama bugün yalnız `research/`'ün yüzeyi ("used by research/ only", README); kararlı API sözü yok; boş bir yer tutucu yayımlamak PyPI'ın ad işgali kuralına aykırı | yayımlanırsa: **trusted publishing** (OIDC, ürün deposunun `release.yml`'i, ortam `pypi`) | PyPI hesabı, projede "pending publisher" tanımı | yok (trusted publishing) |
| crates.io | `rootmark-core`, `rootmark-index`, `rootmark-server` | **hayır (D-1'de)** | kütüphane API'si kararlı değil; `cargo binstall rootmark-server` crates.io kaydı ister → D-1'de belgelerden "yayımlandığında" notuyla kalır | yayımlanırsa: crates.io **trusted publishing** (GitHub Actions OIDC) ya da `CARGO_REGISTRY_TOKEN` | crates.io hesabı; ad sahipliği | `CARGO_REGISTRY_TOKEN` (yalnız trusted publishing kullanılmazsa) |
| npm | `@rootmark/*` kapsamı (boş, R1) | **hayır** | istemciler npm paketi değil (Obsidian eklentisi zip/BRAT, VS Code VSIX); `bindings/node` faz 2 | — | (isteğe bağlı) `rootmark` npm kurumunu ad koruması için açmak — yalnız ürün sahibinin kararıyla | — |
| Obsidian | topluluk eklentisi `rootmark` | **hayır (D-1'de)**; BRAT ve sürüm varlıklarıyla dağıtım sürer, depo `rootmark-ai/rootmark` | mağaza başvurusu ayrı bir inceleme süreci; GOAL P1-E'den beri kapsam dışı | BRAT: sürüm varlıkları `main.js`, `manifest.json`, `styles.css` | (sonra) `obsidian-releases` deposuna başvuru PR'ı — insan | — |
| VS Code Marketplace / Open VSX | yayıncı `rootmark-ai`, uzantı `rootmark-ai.rootmark` | **hayır** | VS Code istemcisi iskelet (faz 2) | yayımlanırsa `vsce`/`ovsx` | yayıncı hesapları | `VSCE_PAT`, `OVSX_PAT` (ileride) |
| Belge sitesi | **`docs.rootmark.ai`** (kılavuz, spec, sözleşme, tehdit modeli, değişiklik günlüğü); `rootmark.ai` kökü aynı siteye ya da bir tanıtım sayfasına | **evet** | spec'in kalıcı adresi laboratuvar adından bağımsız olur (TRANSFER_PLAN §3.4) | ürün deposunun `pages.yml`'i (GitHub Pages); üreteç **mdBook** (Rust araç zinciri; depoya Python/Node araç dili eklemez); uyumluluk rozeti bugünkü gibi ölçülmüş `conformance.json`'dan | DNS: `docs` için `CNAME` → `rootmark-ai.github.io`; kök için GitHub Pages A/AAAA kayıtları; Pages'te özel alan adı ve HTTPS; alan adı doğrulaması (TXT) | yok |
| Güvenlik iletişimi | `security@rootmark.ai` (rol adresi) | **evet** (`SECURITY.md` kaplaması) | kişisel adres yerine rol | — | posta kutusu/yönlendirme açmak | — |

## 3. Konteyner imajı

- **Taban:** `debian:trixie-slim`, özetiyle sabit (bugün `research/p2q/scenario.json`'daki özet; P2-Q'nün kılavuz sınaması bu imajla koştu). İkili glibc ≥ 2.38 ister (trixie 2.41). Distroless (`gcr.io/distroless/cc-debian13`) alternatif: daha küçük, kabuksuz; `ingest-dir` gibi betik kullanımını zorlaştırır → öneri trixie-slim.
- **Mimari:** **`linux/amd64`** (bugünkü yayım hedefi `x86_64-unknown-linux-gnu`). `linux/arm64` için yeni bir yayım hedefi (`aarch64-unknown-linux-gnu`) gerekir; gömme vektörleri x86_64 ve aarch64 Linux'ta bayt eşit (P2-V.1) → teknik engel yok; D-1'e "ikinci mimari" olarak önerilir, karar listesinde (D0-K7). Apple silicon kullanıcıları için arm64 imajı emülasyonsuz çalışır (P2-Q: emüle `amd64` ~90× yavaş, Rosetta açıkken kabul edilebilir).
- **Modeller:** varsayılan model (`minilm-l6-v2`, 90,9 MB) **imajda**: ağsız çalışır, sürüm başına sabit, `ROOTMARK_MODELS=/usr/share/rootmark/models`, `fetch_model.sh`'in manifest denetimiyle yapım sırasında indirilir; lisans metni ve atıf imajda (TRANSFER_PLAN §5). Çok dilli model (479,4 MB) imajda **değil**: ayrı etiket (`:X.Y.Z-multilingual`) ya da birim üzerine `fetch_model.sh` (kılavuz §1.2). Tahmini boyut: taban ~75 MB + ikili 23 MB + model 91 MB ≈ 190 MB (sıkıştırılmamış).
- **Kullanıcı:** kök olmayan `rootmark` (uid 10001); vault `/vault` birimi. Bağlanan dizinin sahibi farklı olduğunda `docker run --user "$(id -u):$(id -g)"` (kılavuz notu).
- **Ağ — kısıt (bulgu D0F-3):** HTTP API yalnız `127.0.0.1`'e bağlanır (`server/src/http.rs`, tasarım gereği: "no external interface"). Konteynerin içindeki `127.0.0.1` dışarıdan `-p` ile yayımlanamaz. Bu yüzden imajın hızlı başlangıcı **MCP stdio**'dur (ajan istemcisi konteyneri `docker run -i` ile başlatır); HTTP yalnız Linux'ta `--network host` ile. Konteyner içinde dış arayüzü açan bir bayrak ürün ve tehdit modeli değişikliğidir (token'ın ağ üzerinden kullanımı) — D-1'in kapsamına girip girmeyeceği karar listesinde (D0-K8).
- **Sağlık denetimi:** stdio kipinde `HEALTHCHECK` yok (süreç istemcinin); `--network host` + HTTP kipinde `HEALTHCHECK CMD ["rootmark","--version"]` yerine `GET /v1/status` belirteç ister → D-1 `rootmark` içinde belirteçsiz bir canlılık ucu eklemedikçe `--version` ile yetinilir.
- **Kılavuzun hızlı başlangıcı (`docker run` biçimi, öneri):**
  ```sh
  IMG=ghcr.io/rootmark-ai/rootmark:X.Y.Z
  docker run --rm "$IMG" conformance >/dev/null                     # level 3, 39/39
  docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" ingest-dir /vault /vault
  docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" index build /vault
  # an agent's MCP config: command "docker", args:
  #   ["run","-i","--rm","--user","<uid>:<gid>","-v","/home/ada/notes:/vault","ghcr.io/rootmark-ai/rootmark:X.Y.Z",
  #    "serve","--vault","/vault","--mcp","stdio","--watch"]
  ```
  D-1'in çıkış kriteri ("imaj `docker run` ile kılavuzun hızlı başlangıcını koşuyor") bu dört adımla sınanabilir; `--watch` konteynerde bağlanan birimde çalışır (Linux'ta inotify; macOS Docker Desktop'ta dosya olayları birim üzerinden iletilir — D-1'de sınanmalı).
- **Etiketler:** `X.Y.Z`, `X.Y`, `latest`; özet (digest) sürüm notunda; köken tasdiki imaj özetine.

## 4. PQ2-1 ve kanallar
Claude Code (ve stdio'lu her MCP istemcisi) sunucuyu oturum sonunda SIGKILL'le bitirebilir; konteynerde `docker run -i --rm`'in durdurulması da aynı yoldur. R5'in sonucu (REPORT §5) imajın varsayılan kipinin (stdio + `--watch`) PQ2-1'e açık olduğunu gösterir; düzeltmenin sırası karar listesinde (D0-K10).

## 5. P2-Q.2 doğrulamasının önerileri → D-1 planı
| Öneri | D-1'deki yeri |
|---|---|
| **Ö1** — `check_dep_age`'i dolaylı npm bağımlılıklarına genişletmek | `scripts/check_dep_age.sh` (laboratuvar ve ürün; M dönüşümü, `transforms.md`): iki `package-lock.json`'ın bütün `packages` girdileri ≥ 14 gün, istisnalar `dep_age_exceptions.txt`'de gerekçeyle. Ürün `ci.yml`'in `deps` işinde ve aktarım aracının ön koşulunda koşar. D-1 GOAL'ünde bir kriter: "kilitlerin dolaylı bağımlılıkları dahil yaş ihlali 0". |
| **Ö2** — `SHA256SUMS.local`'ı yayımlanmış varlıklarla tutarlı yapmak | ürün `release.yml` bütün varlıkları (arşivler, kurulum betikleri, **wheel ve eklenti zip'i de**) CI'da üretir ve tek bir `SHA256SUMS` yazar; `SHA256SUMS.local` kalkar; kılavuz §5.1 eklenti zip'ini `SHA256SUMS` ile denetler. `RELEASING.md` (ürün) adımı buna göre. D-1 kriteri: "sürümün her varlığı tek `SHA256SUMS`'ta, `sha256sum -c` hatasız; MISMATCH 0". |
| **Ö3** — her yayımdan önce gerçek Claude Code sınaması | laboratuvarın yayım kapısına yeni madde (R21: `research/p2q2/claude_probe.py` güncel Claude Code ile, sonuç kaydı yayımdan ≤ 7 gün önce; kayıt yoksa `release_check.sh` FAIL) ve ürünün `RELEASING.md`'sinde insan adımı. CI'da gerçek istemci yok (kimlik gerektirir) → yerel, insanın başlattığı adım. D-1 kriteri: "D-1 sürümünden önce güncel Claude Code ile araç listesi ve `propose` geçti". |

## 6. Gizli değişkenler (yalnız adlar, özet)
Laboratuvar: `PRODUCT_REPO_DEPLOY_KEY`. Ürün: `DOCKERHUB_TOKEN` (+ değişken `DOCKERHUB_USERNAME`), `HOMEBREW_TAP_TOKEN`; ileride `CARGO_REGISTRY_TOKEN` (trusted publishing yoksa), `VSCE_PAT`, `OVSX_PAT`. PyPI, `ghcr.io`, köken tasdiki ve GitHub Pages gizli değişken istemez (OIDC / `GITHUB_TOKEN`). Değerler yalnız GitHub'ın gizli değişkenlerinde; sohbet, kayıt ya da commit'e girmez.
