# D-1 — Tasarım (R0)

`ROUND_START` = `87a042e` (PR #72 sonrası `main`). Bu belge R1'in ilk kod commit'inden önce commit'lenir (D1-1). Girdiler: `reports/rounds/D-0/` (TRANSFER_PLAN, CHANNELS, README_SKELETON, `transfer/`), K-D0.4 (D0-K2..K18), K-P2Q3.3, K-P2Q4.3, K-P2Q4.4.

## 0. Değişmesi beklenen laboratuvar kayıtları (K-P2Q1.4; adıyla, değişiklikten önce)
| Kayıt | Değişiklik | Görev | Neden |
|---|---|---|---|
| `.github/workflows/ci.yml` | `deps` işi `scripts/check_frozen.sh` ve `scripts/check_compare_fields.sh`'i de koşar (Python: `uv` + `numpy`/`blake3` yalnız, `UV_NO_SYNC=1`); başka iş, süzgeç ve gerekli denetim adı aynen | R0 | K-P2Q4.4 (4): yalnız dondurulmuş `.md`/`reports` değiştiren PR dondurma denetiminden kaçıyordu |
| `.github/workflows/transfer.yml` (yeni) | laboratuvarın aktarım iş akışı (`workflow_dispatch`, girdiler `tag`, `bootstrap`; §2.1); başka iş akışı değişmez | R1 | GOAL R1 |
| `scripts/release_check.sh` (dondurulmuş) | yeni madde **R21** (güncel Claude Code ile `claude_probe.py` kaydı ≤ 7 gün); öteki 21 madde aynen | R0 | GOAL R6 / Ö3; DEV_PROCESS §4, `FREEZE_EXCEPTIONS.md` D-1/F1; `research/FROZEN.md`'ye yeniden dondurma satırı |
| `research/FROZEN.md` | yalnız bir satır eklenir (`release_check.sh`, D-1/F1) | R0 | §4 |
| `docs/RELEASE.md` | R21 satırı ve insan adımı (Claude Code sınaması); 0.4.0 başlığı, adım 1'in doğrulama raporları, ürün deposuna yayımın yeri | R0 (R21), R6 (0.4.0) | R21; laboratuvar 0.4.0 için GitHub sürümü yayımlamaz (TRANSFER_PLAN §3.3) |
| `research/p2q2/claude_probe.py` | özet kayda `recorded_at` (UTC, saniye) ve `--out-file` | R0 | R21'in yaşı kayıttan okunur (dosya zamanı değil) |
| Sürüm dizgileri | 0.3.4 → **0.4.0**: `Cargo.toml` ×4 (`core`, `index`, `server`, `bindings/python`), `Cargo.lock`'un kendi crate satırları, `clients/obsidian/{package,package-lock,manifest}.json`, `clients/vscode/package{,-lock}.json` (varsa), `server/src` içindeki sürüm sabiti yoksa yok; `CHANGELOG.md` `## 0.4.0` | R6, `release/0.4.0` dalında | K-P2Q4.4 (5) |
| E1-4/R14 (etikete bağlı okuma) | `v0.4.0` etiketi atıldıktan sonra bir sonraki `ROUND_START`'a kadar R14 farklı okunur (beklenen fark) | R6 | K-P2Q1.4 (2) |
| `scripts/check_dep_age.sh` | npm kilitlerinin bütün `packages` girdileri (dolaylı dahil) ≥ 14 gün; istisnalar `scripts/dep_age_exceptions.txt` | R2 | Ö1 |
| `scripts/dep_age_exceptions.txt` | gerekirse gerekçeli satırlar (Ö1'in yeni ihlalleri) | R2 | Ö1 |
| `docs/GUIDE.md` | değişmez (ürün yüzü kaplamadadır); yalnız laboratuvar kılavuzunda bir hata bulunursa düzeltilir ve kaplama `SOURCES.json` ile güncellenir | — | |
| `.gitignore` | `product/overlay` için bir şey gerekmez; `out/` (ürün betiklerinin çıktı dizini) eklenir | R1 | M dönüşümü `out/` yazar |

Ürün sınırı (D1-5): `core index server bindings spec clients/*/src` içinde yalnız sürüm dizgileri. Laboratuvar `make lint` aynen.

## 1. Aktarım aracı (`research/transfer/`, Python; çalışma zamanı değil)
```
research/transfer/
  __init__.py
  lists/allowlist.txt   # D-0'dan; D-1 düzeltmeleri §1.2
  lists/stays.txt
  lists/transforms.md   # | desen | dönüşüm | ürün yolu |  + İstisnalar
  classify.py           # D-0 classify.py'nin desen mantığı (aynı fonksiyonlar), commit'ten
  rewrite.py            # M dönüşümleri: yol → saf fonksiyon (bytes → bytes); her kural tam bir kez eşleşmeli
  leaks.py              # sızıntı sınıfları (§1.5)
  build.py              # giriş noktası: python3 -I -m transfer.build --ref <etiket|commit> --out <dir>
  verify.py             # TRANSFER.json ↔ etiket blob'ları ↔ kaplamalar; ağaç kimliği karşılaştırması
```
- **Sınıflandırma:** `classify.py`'nin `read_list`, `read_transforms`, `matches`, `dest_of` mantığı aynen (`research/d0/classify.py` dondurulmuş değil; D-0 kaydının üreteci olduğu için dokunulmaz, kopya `research/transfer/classify.py`'de; test ikisinin aynı sonucu verdiğini `ROUND_START`'ta denetler). En uzun desen kazanır; sınıflandırılmamış, çift sınıflı, ürün yolu çakışması, ölü desen > 0 → hata.
- **Girdi:** bir laboratuvar etiketi ya da commit; araç yalnız `git ls-tree`/`git cat-file` ile o commit'in blob'larını okur (çalışma ağacını değil). Kaplamalar ve listeler **aynı commit'ten** okunur (etiket ağacı belirler).
- **Dönüşümler** (`transforms.md`, D-0 türleri):
  - **= (aynen):** blob baytları, kip (100644/100755) aynen.
  - **K (konum):** baytlar ve kip aynen, yol değişir (`research/data/{tok_ref,embed_ref}.json`, `p2_0*_old_vaults/` → `testdata/…`; `server/tests/fixtures/index_ort/vault/.rootmark/` → `…/dot-rootmark/`).
  - **M (mekanik):** `rewrite.py`'de yol başına kural listesi `(eski, yeni, beklenen sayı)`; her kural blob'da tam beklenen sayıda eşleşmezse hata (sessiz başarısızlık yok — kaynak değişmişse araç durur). Kurallar §1.3.
  - **Y (kaplama):** `product/overlay/<ürün yolu>` baytları ve kipi; `product/overlay/SOURCES.json` kaplamanın kaynağının blob kimliğini taşır; etiketteki kaynak blob'u farklıysa hata (`overlay <yol>: source <lab yol> changed (<eski>→<yeni>); update the overlay`).
  - **Yalnız-ürün kaplaması:** `SOURCES.json`'da `"source": null`; laboratuvar kaynağı yok (`RELEASING.md`, `Dockerfile`, iş akışları…).
- **`product/` kendisi laboratuvarda kalır** (`stays.txt`: `product/`); kaplama dizinindeki her dosya `SOURCES.json`'da adıyla bulunmalı (fazla ya da eksik → hata).
- **`TRANSFER.json`** (ürün kökünde; kendisi listelenmez): `{"format": 1, "lab_repo": "arknsensei/rootmark", "lab_commit": <sha>, "lab_tag": <etiket|null>, "files": [{"path", "mode", "sha256", "origin": "lab:<yol>@<blob>" | "overlay:<yol>@<blob>", "transform": "=" | "K" | "M" | "Y"}]}`; yola göre sıralı, `ensure_ascii=False`, `indent=1`, sonda `\n`; zaman damgası yok. M satırının `origin`'i laboratuvar kaynağıdır (`lab:`), `sha256` çıktınınkidir.
- **Ağaç kimliği:** geçici çıplak depo (`git init --bare <out>/tree.git`); her dosya `git hash-object -w --stdin`, yollar `GIT_INDEX_FILE` ile `git update-index --add --cacheinfo <kip>,<blob>,<yol>`, `git write-tree`. Kipler yalnız 100644/100755 (sembolik bağlantı varsa hata); satır sonları baytlarla aynen (dönüştürme yok, `core.autocrlf=false`). Çıktı: `<out>/tree/` (dosyalar), `<out>/tree.git` (nesneler; `transfer.yml` commit'i buradan yazar), `<out>/transfer_result.json` (`tree`, sayılar, sızıntılar). Aynı girdiden iki koşu aynı kimlik (test).
- **Belirlenimlilik:** dosya sırası `LC_ALL=C` yol sırası; ortam (kullanıcı, saat, dizin) çıktıya girmez; Python `-I`, yalnız standart kitaplık.

### 1.2 Liste düzeltmeleri (D-0 → D-1)
- `stays.txt` + `product/`, `research/transfer/` zaten `research/` altında; `scripts/release.sh` → kalır (laboratuvarın yerel yayım kapısı; ürünün paketi CI'da, §3.2); `.github/workflows/transfer.yml` kalır.
- Y'ye geçenler (D-0'da M): `SECURITY.md` (İngilizce özet), `docs/GUIDE.md` (ürün kılavuzu: depo URL'si + PQ3-3/PQ3-5/PQ4-9 notları + Docker/Homebrew), `NOTICE` (K18 + imajdaki model), `.github/workflows/release.yml` ve `pages.yml` (ürün yüzü laboratuvarınkinden yapıca farklı: tek `SHA256SUMS`, wheel/eklenti CI'da, tap; mdBook sitesi), `scripts/contract_check.sh` (İngilizce sözleşmenin tablolarını okur; kaynak laboratuvarınki).
- Yeni yalnız-ürün kaplamaları: `RELEASING.md`, `Dockerfile`, `.dockerignore`, `.github/workflows/{transfer-verify,image}.yml`, `.github/ISSUE_TEMPLATE/feature.md` (isteğe bağlı), `testdata/THIRD_PARTY.md`, `about.toml` + `about.hbs` (`THIRD_PARTY_LICENSES` üreteci), `site/` (mdBook kaynakları, §5), `product/transfer` aracı ürün deposuna **gitmez** — `transfer-verify` laboratuvarı `Lab-Commit`'te klonlayıp laboratuvarın aracını koşar.
- **`THIRD_PARTY_LICENSES` ağaçta değildir:** sürüm başına ürün `release.yml`'inde `cargo about generate` ile üretilir (sürümün `Cargo.lock`'undan), arşivlere, imaja ve sürüm varlıklarına girer, `SHA256SUMS`'ta. Gerekçe: ağaçta tutulsa her bağımlılık değişiminde elle güncellenecek ve yayımdakiyle sapabilirdi.

### 1.3 M kuralları (her biri beklenen eşleşme sayısıyla)
| Yol | Kural |
|---|---|
| 8 test dosyası (`index/tests/{lexical,embed,p2_l_tokenize}.rs`, `server/tests/{p2_b_b0,p2_b_b1,p2_p_channel,p2_p_p0,promotion_rules}.rs`) | `../research/data/` → `../testdata/`; yorumlardaki `research/data/` → `testdata/` |
| `server/tests/p2v1_migrate.rs` | `ort_vault`'un dönüşüne kopyadan sonra `dot-rootmark` → `.rootmark` yeniden adlandırması (tek ekleme; testin gözlemi aynen) |
| `server/Cargo.toml` | `repository = "https://github.com/arknsensei/rootmark"` → `rootmark-ai/rootmark` |
| `packaging/install.sh`, `install.ps1`, `homebrew/rootmark.rb.in` | `arknsensei/rootmark` → `rootmark-ai/rootmark` |
| `CHANGELOG.md` | laboratuvar göreli bağlantıları → `https://github.com/arknsensei/rootmark/blob/<lab_tag>/<yol>`; eski sürümlerin indirme URL'leri **aynen** (laboratuvarda kalırlar) |
| `scripts/check_deps.sh` | `research/` denetimi çıkar |
| `scripts/check_dep_age.sh` | `research/pyproject.toml` kısmı çıkar (npm dolaylı bağımlılıkları R2'de laboratuvar betiğine eklenir, ürüne aynen gider) |
| `scripts/scene.sh`, `scene_agent.sh`, `plugin_e2e.sh` | varsayılan çıktı `reports/rounds/…` → `out/…` |
| `.github/dependabot.yml` | `uv` (`/research`) bloğu çıkar |
Kuralların kesin metni `rewrite.py`'dedir; R1'de etiket ağacındaki gerçek dizgilerle yazılır (D-0 tablosu `scene_agent.sh` vb. için yer tutucudur).

### 1.4 Ürün deposu boşken ilk aktarım
Ürün deposunda `main` yok (D-1 açılışında boş, dalsız); boş depoda PR açılamaz (taban dal yok) ve "temiz geçmiş: ilk commit ilk sürüm" (D0-K5) boş bir başlangıç commit'ine izin vermez. Seçilen yol: `transfer.yml` girdisi `bootstrap: true` iken kök commit'i `transfer/v0.4.0` dalına iter ve PR açmaz; **insan** ürün deposunda varsayılan dalı bu commit'ten `main` olarak oluşturur (`gh api -X POST repos/rootmark-ai/rootmark/git/refs -f ref=refs/heads/main -f sha=<commit>` — HUMAN_STEP'te), dal korumasını sonra açar. `transfer-verify.yml` `main`'e push ve `workflow_dispatch` ile de koşar; ilk commit'in doğrulaması bu koşudur. Sonraki sürümlerde ebeveyn ürün `main`'i, PR `transfer/vX.Y.Z → main`.

### 1.5 Sızıntı denetimi (ürün ağacında, baytlar üzerinde; `leaks.py`)
| Sınıf | Kural | İstisna |
|---|---|---|
| `lab_prefix` | yol `research/`, `reports/`, `docs/rounds/`, `docs/archive/`, `product/` ile başlar | yok |
| `rootmark_dir` | `.rootmark` bir yol bileşeni | yok |
| `server_token` | ad `server.token` | yok |
| `key_pattern` | ad `*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa*`, `id_ed25519*`, `.env*`, `*secret*` | yok |
| `secret_content` | içerikte belirteç desenleri: `gh[pousr]_[A-Za-z0-9]{36,}`, `github_pat_…`, `dckr_pat_…`, `-----BEGIN [A-Z ]*PRIVATE KEY-----`, `AKIA[0-9A-Z]{16}`, `sk-ant-…`, `xox[baprs]-…` | yok |
| `host_user` | çalışan kullanıcının adı (sözcük sınırıyla) ya da ev dizini içerikte | koşucu adları `runner` ve kod içi `rmhuman`/`ada` gibi kurgular `transforms.md` İstisnalar'da |
| `home_path` | `/Users/<x>/`, `/home/<x>/` | `transforms.md` İstisnalar (`ada`, `rmhuman`, gerekirse `runner` — CI yolu örneği) |
| `email` | e-posta deseni | `@rootmark.ai` rol adresleri, `example.com`/`example.org`/`example.net` adresleri, `git@github.com`; ötekiler yalnız `transforms.md` "E-posta istisnaları"nda dosya + adresle, gerekçeli (ör. bağımlılık lisans metinleri, fixture'ların kurgu adresleri) |
| `admin_mail` | ürün sahibinin bilinen kişisel/yönetici adresleri | adres değeri kayda yazılmaz; araç `~/.config/rootmark-lab/leak_terms.txt` (600, varsa) satırlarını arar, CI'da `LEAK_TERMS` gizlisi yoksa yalnız genel desenler — sayılar kayda, değerler değil |
| `turkish_process` | laboratuvarın Türkçe süreç belgelerinin adları (`NORTH_STAR.md`, `DECISIONS.md`, `DEV_PROCESS.md`, `ROADMAP.md`, `HANDOFF.md`, `ARCHITECTURE.md`, `PACKAGES.md`, `RELEASE.md`, `SPEC_EDITS_CRITERION.md`, `HEADLESS_CHANNEL.md`) ürün yolu olarak | yok |
| `broken_link` | ürün ağacındaki `.md`'lerin göreli bağlantısı ağaçta olmayan bir yola | fixture notları (D-0 kuralı) |
Herhangi biri gerekçesiz > 0 → aktarım durur (çıkış ≠ 0). Kod yorumlarındaki tur/karar atıfları (`K-P2B.1`) sızıntı değildir (TRANSFER_PLAN §1.1).

## 2. İş akışları: `transfer.yml` (laboratuvar) ve `transfer-verify.yml` (ürün)
### 2.1 `transfer.yml` (laboratuvar; `workflow_dispatch`; girdiler `tag` (zorunlu, `vX.Y.Z`), `bootstrap` (bool))
1. `actions/checkout` `fetch-depth: 0`; `tag` laboratuvarda var ve `vX.Y.Z` biçiminde; aracın kodu **etiketin** ağacından (`git worktree add` etiket) koşar.
2. `python3 -I -m transfer.build --ref <tag> --out A`; ikinci kez `--out B`; `A/transfer_result.json.tree` = `B`'ninki, değilse hata (ilk 20 farklı yol).
3. Ürün deposu `PRODUCT_REPO_DEPLOY_KEY` (yalnız ürün deposuna yazan dağıtım anahtarı; `webfactory/ssh-agent` yok — anahtar `ssh-agent`'a adımın içinde yüklenir, diske yazılmaz) ile `git ls-remote`; `main` varsa ebeveyn o, yoksa `bootstrap` doğru olmalı (değilse hata).
4. `git commit-tree <tree> [-p <main>]`; yazar ve işleyen `github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>` (kişisel e-posta yok); tarih `Lab-Commit`'in işleyen tarihi (belirlenimli: aynı girdi aynı commit kimliği). Mesaj: `rootmark X.Y.Z` + boş satır + `Lab-Commit: <sha>`, `Lab-Tag: vX.Y.Z`, `Transfer-Tree: <tree>`.
5. `git push <ssh url> <commit>:refs/heads/transfer/vX.Y.Z` (zorla değil; dal varsa ve farklıysa hata).
6. PR: dağıtım anahtarı API'ye yazamaz → PR'ı insan açar (`HUMAN_STEP`: `gh pr create -R rootmark-ai/rootmark --base main --head transfer/vX.Y.Z`) ya da bir GitHub App kullanılırsa iş açar. **Karar:** dağıtım anahtarı (en az yetki; App kurulumu insan için daha uzun); PR'ın açılması insan adımı (TASKS'ın "PR açar"ı bu yüzden HUMAN_STEP'e taşınır — ASSUMPTIONS A1).
7. Kayıt: iş özeti (`$GITHUB_STEP_SUMMARY`) ve yapıt `transfer-<tag>` (`transfer_result.json`, `TRANSFER.json`; ağaç değil).
Hata yolları: etiket yok; sınıflandırma/sızıntı/kaplama hatası; iki koşu farklı; ürün `main`'i beklenmeyen; dal var ve farklı; push reddi. Hepsi çıkış ≠ 0, ileti adıyla.

### 2.2 `transfer-verify.yml` (ürün; `pull_request` → `main`, `push` → `main`, `workflow_dispatch`)
1. Ürün commit'ini `fetch-depth: 1`; mesajından `Lab-Commit`, `Lab-Tag`, `Transfer-Tree` (yoksa hata; ürün `main`'ine aktarım dışı commit girmez).
2. Laboratuvarı herkese açık URL'den `Lab-Commit`'te klonlar (`git clone --filter=blob:none` + `checkout`); etiket `Lab-Tag` o commit'i gösterir (değilse hata).
3. `python3 -I -m transfer.build --ref <Lab-Commit> --out T`; `T.tree` = `Transfer-Tree` = `git rev-parse HEAD^{tree}` (üçü eşit).
4. `verify.py`: `TRANSFER.json`'ın her `lab:` satırı etiket blob'una, her `overlay:` satırı kaplamaya eşit; ağaçta `TRANSFER.json`'da olmayan dosya 0; sızıntı denetimi üretilmiş ağaçta yeniden (§1.5).
Yetki: `contents: read`. Gizli değişken yok.

## 3. Ürün deposunun iş akışları (TRANSFER_PLAN §3.2; hepsi `product/overlay/.github/workflows/`)
### 3.1 `ci.yml` (ürün)
`deps` (`check_deps.sh`, `check_dep_age.sh` — npm dolaylı dahil), `test` (macos-15: `make models`, `make test`, `make lint`, `make conformance`), `test-linux` (ubuntu-latest, aynı). Ürün `Makefile`'ı yalnız ürün hedefleri: `build`, `test` (cargo + istemcilerin `vitest`), `lint` (`cargo fmt --check`, `clippy -D warnings`, istemcilerin `tsc`/`eslint`, `check-deps`, `check-client-types`), `conformance`, `models`, `models-multilingual`, `clients`. Laboratuvara özgü hedefler (`verify*`, `regress`, `p*-checks`, `lint-py`, `check-frozen`) yok. Belge süzgeci (K-P2Q4.3) ürüne gitmez: ürün `main`'i yalnız aktarım PR'larıyla değişir.

### 3.2 `release.yml` (ürün; `workflow_dispatch`, girdi `tag`)
İşler: `licenses` (`cargo about generate about.hbs > THIRD_PARTY_LICENSES`, `cargo-about` ≥ 14 gün sabit sürüm) → `binary` (matris: `aarch64-apple-darwin` macos-15, `x86_64-unknown-linux-gnu` ubuntu-24.04 zorunlu; `x86_64-pc-windows-msvc` bilgi; arşivde `rootmark`, `LICENSE`, `NOTICE`, `README.md`, `THIRD_PARTY_LICENSES`, `models/MANIFEST.json`, `models/fetch_model.sh`; `release_target.sh check` ve `install.sh --from` sınaması laboratuvardaki gibi) → `wheel` (maturin, ubuntu; `rootmark_bindings-X.Y.Z-*.whl`) → `plugin` (`clients/obsidian` `npm ci && npm run build` → `rootmark-obsidian-X.Y.Z.zip` + `main.js`, `manifest.json`, `styles.css`) → `record` (tek `SHA256SUMS`: arşivler, `install.sh`, `install.ps1`, wheel, eklenti zip'i, `main.js`, `manifest.json`, `styles.css`, `THIRD_PARTY_LICENSES`; `sha256sum -c`; `actions/attest-build-provenance` hepsine; `gh release create vX.Y.Z` — etiket insanın attığı etiket olmalı, yoksa hata; notlar CHANGELOG'un `## X.Y.Z` bölümü) → `image` (`uses: ./.github/workflows/image.yml`, `with: tag`) → `tap` (formül → tap PR, §6). `SHA256SUMS.local` yok. İzinler iş başına en az (`contents: write` yalnız `record`; `id-token`, `attestations` yalnız tasdik eden işlerde; `packages: write` yalnız `image`).

### 3.3 `image.yml` (ürün; `workflow_call` + `workflow_dispatch`, girdi `tag`)
Yayımlanmış sürümden `rootmark-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` + `SHA256SUMS` indirir, `sha256sum -c` ve `gh attestation verify` (imaj, sürümün ikilisiyle aynı baytları taşır); `docker/setup-buildx-action`, `docker/login-action` iki kez (`ghcr.io`: `GITHUB_TOKEN`; Docker Hub: `vars.DOCKERHUB_USERNAME` + `secrets.DOCKERHUB_TOKEN`), `docker/build-push-action` `platforms: linux/amd64`, `push: true`, etiketler `ghcr.io/rootmark-ai/rootmark:{X.Y.Z,X.Y,latest}` ve `rootmark/rootmark:{…}` **tek yapımda** (aynı manifest özeti iki kayıt defterine), `provenance: false`, `sbom: false` (özet belirlenimli kalsın; köken ayrıca tasdikle); `actions/attest-build-provenance` `subject-name: ghcr.io/rootmark-ai/rootmark`, `subject-digest`, `push-to-registry: true` (Docker Hub için de ayrı tasdik `subject-name: index.docker.io/rootmark/rootmark`); son adım iki kayıt defterinden `docker buildx imagetools inspect` özetlerini karşılaştırır (farklıysa hata).

### 3.4 `pages.yml` (ürün; `push` → `main` ve `workflow_dispatch`)
`mdbook` (sabit sürüm, `cargo install --locked mdbook@<≥14 gün>` önbellekle ya da yayımlanmış ikili + sha256), `rootmark` yapımı ve `rootmark conformance` → `site/src/spec/conformance.json` + `badge.json` (bugünkü gibi ölçülmüş), kaynak belgeler ağaçtan kopyalanır (§5), `mdbook build`, iç bağlantı denetimi (betik: üretilmiş HTML'deki göreli `href`'ler hedefte var; kırık > 0 → hata), `actions/upload-pages-artifact`, `deploy-pages`. `CNAME` (`docs.rootmark.ai`) site çıktısına.

## 4. Konteyner imajı (`product/overlay/Dockerfile`)
- Taban: `debian:trixie-slim@sha256:<özet>` (R4'te güncel özet okunur ve sabitlenir; `research/p2q/scenario.json`'daki özetle karşılaştırılır).
- Katmanlar: (1) `ca-certificates` yok (ağ yok, HTTP yok) — yalnız tabanın kendisi; (2) `COPY` sürüm arşivinden çıkarılmış `rootmark` → `/usr/local/bin/rootmark`; (3) varsayılan model `/usr/share/rootmark/models/minilm-l6-v2/` — yapım sırasında ayrı bir aşamada (`debian:trixie-slim` + `curl`, `jq`) `fetch_model.sh`'in manifest denetimiyle indirilir, son aşamaya yalnız model dosyaları kopyalanır; modelin `LICENSE` (Apache-2.0 metni) ve `ATTRIBUTION` (kaynak, revizyon, lisans; `models/MANIFEST.json`'dan) aynı dizine; (4) `/usr/share/doc/rootmark/{LICENSE,NOTICE,THIRD_PARTY_LICENSES}`.
- Kullanıcı: `rootmark` uid/gid 10001, ev `/home/rootmark`; `USER 10001:10001`; `WORKDIR /vault`; `VOLUME /vault`; `ENV ROOTMARK_MODELS=/usr/share/rootmark/models`.
- `ENTRYPOINT ["rootmark"]`, `CMD ["serve","--vault","/vault","--mcp","stdio","--watch"]` (PQ3-5: `--watch`'lı). `HEALTHCHECK` yok (stdio).
- `LABEL org.opencontainers.image.{source=https://github.com/rootmark-ai/rootmark, version, licenses=Apache-2.0, revision=<ürün commit'i>}` (tarih etiketi yok: özet belirlenimli kalsın).
- Boyut tahmini: taban ~75 MB + ikili ~25 MB + model ~91 MB ≈ 190 MB.
- Yerel yapım (R4): sürüm yokken arşiv laboratuvar ağacından Linux çapraz yapımıyla (`scripts/linux_cross_build.sh`, `release.sh`'in yolu) üretilir; Dockerfile girdisi `--build-arg ARCHIVE=…` değil, yapım bağlamında `dist/rootmark-linux.tar.gz` (adı sabit). Sınama: kılavuzun dört adımı (`conformance`, `ingest-dir`, `index build`, MCP stdio oturumu: `propose` + `search`) ve `--watch` (birimde düzenleme → `search` yeni metni) Linux'ta (Docker, `--platform linux/amd64`) ve macOS Docker Desktop'ta; `id -u` = 10001; model lisansı dosyada.

## 5. Belge sitesi (`product/overlay/site/`, mdBook)
- `site/book.toml`, `site/src/SUMMARY.md`, `site/theme/` yok (varsayılan tema), `site/src/CNAME`? (mdBook `[output.html] cname = "docs.rootmark.ai"`).
- Sayfalar: `introduction.md` (README'nin özü), `install.md` (macOS, Linux, Docker, Homebrew), `quickstart.md`, `guide.md` ← `docs/GUIDE.md` (ürün kaplaması; `pages.yml` kopyalar), `git.md` (**"How RootMark differs from git"**), `spec/index.md` ← `spec/SPEC.md`, `spec/conformance.md` ← `spec/CONFORMANCE.md` + ölçülmüş rozet, `contract.md` ← `docs/CLIENT_CONTRACT.md`, `threat-model.md` ← `docs/THREAT_MODEL.md`, `changelog.md` ← `CHANGELOG.md`, `security.md` ← `SECURITY.md`. Kaynağı ağaçta olan sayfalar `site/src`'de **tutulmaz**; `pages.yml` (ve yerel `make site`) kopyalar → tek kaynak.
- "git ile fark" sayfası iddia → atıf tablosu taslağı (kesin hali `progress/R3_claims.md`):
  | İddia | Atıf (laboratuvar, `v0.4.0` etiketli mutlak URL) | Koşul |
  |---|---|---|
  | git dosya/satır düzeyinde iz tutar; RootMark blok kimliği ve soyunu düzenleme, bölme, birleştirme ve taşımada izler | prototip v1–v3 ve P1-A (C1 17/17), `reports/rounds/P1-A/FINAL.md` | sentetik + 56k bloklu gerçek tarihçe |
  | onaylanmamış (proposal/unverified) içerik aramaya girmez | P1-B/P1-D kriterleri, `spec/SPEC.md` §7–8, C2 | varsayılan arama |
  | ajan yazımı canonical içeriği değiştiremez (`append_only`) | P1-D FINAL | MCP kanalı |
  | her cevap dayandığı blokları kaydeder (`trace`) | P1-B (§11), P1-E sahnesi | `search`/`read_block` |
  | dosya erişimli ajanın doğrudan yazımı canonical sayılmaz | P2-0.3 FINAL, K-P2-03.1, NORTH_STAR ölçülen koşul | **yalnız** `[attestation] unattested = "unverified"` ve tasdikli istemciyle; varsayılan `canonical` |
  | aynı dosyalar her makinede aynı indeks ve günlük | P2-V.1 (üç platform bayt eşit), P1-A çift koşturma | |
  | git'in verdiği: tam sürüm geçmişi, dallar, uzak eşitleme — RootMark bunları vermez (sync yok) | NORTH_STAR "değiliz", ROADMAP P2-C | |
- Rozet: `pages.yml`'in ölçtüğü `conformance.json`'dan (`badge.json`), elle sayı yok.
- Bağlantı denetimi: yerelde `mdbook build` + betik; kırık iç bağlantı 0.
- Laboratuvar Pages yönlendirmesi (K17): `product/` dışında, laboratuvar `pages.yml`'inin yeniden adlandırmadan önce koşacağı statik sayfa; R3'te hazırlanır (`docs/rounds/D-1/progress/lab_redirect/`), yayımı insan adımı (laboratuvar `pages.yml` bu turda değişmez — değişirse §0'a önceden yazılır).

## 6. Homebrew tap
- Formül şablonu (ürün yüzü): `packaging/homebrew/rootmark.rb.in` (M: depo adı) — `on_macos/on_arm` ve **`on_linux/on_intel`** blokları (iki arşiv), `install`: `bin.install "rootmark"`, `pkgshare.install "models"`, `doc.install "LICENSE", "NOTICE", "THIRD_PARTY_LICENSES"`; `caveats` modeli indirme; `test`: `--version` ve `rootmark conformance` (39/39 — fixture'lar ikilide gömülü). Linux bloğu laboratuvar şablonunda yok → ürün şablonu **Y** kaplamasına geçer (kaynak laboratuvar şablonu).
- Ürün `release.yml`'in son işi `tap`: `SHA256SUMS`'tan iki arşivin özeti → `Formula/rootmark.rb`; `HOMEBREW_TAP_TOKEN` ile `rootmark-ai/homebrew-tap`'te `formula/rootmark-X.Y.Z` dalı ve PR (`gh pr create -R rootmark-ai/homebrew-tap`); birleştirme insanın.
- Yerel sınama (R5): geçici tap dizini (`brew tap-new --no-git local/rootmark-test` değil — yalnız scratchpad'de `Formula/rootmark.rb`, `url "file://…"`), `HOMEBREW_NO_INSTALL_FROM_API=1 brew install --formula ./rootmark.rb`? Homebrew yerel dosya formülünü reddedebilir → yöntem: `brew tap-new` scratchpad'deki bir yerel tap'e (yerel git deposu, uzak yok) ve `brew install <tap>/rootmark`, `brew test`, `brew uninstall`, `brew untap`. Kullanıcının mevcut Homebrew kurulumuna yalnız bu geçici tap ve formül dokunur; sonunda kaldırılır.

## 7. Yayım kapısı R21 (`release_check.sh`, D-1/F1)
- Kayıt: `<dist>/claude_probe/*.json` — `research/p2q2/claude_probe.py --out <dist>/claude_probe` çıktısı (yeni alan `recorded_at`).
- Geçer: en az bir kayıt için `exit == 0`, `rootmark_version == "rootmark <ver>"`, `tools_listed_count == 11` ve `tools_match_0_3_1`, `tool_calls` içinde `propose`, `agent_events ≥ 1`, `server_errors` boş, `recorded_at` var ve şimdiden en çok 7 gün (604 800 s) önce (gelecekte değil). Kayıt yok ya da eski → FAIL (kasıtlı bozma: `recorded_at` 8 gün önce, alan yok, dizin yok).
- "Güncel Claude Code": kayıttaki `claude_code_version` sınama anındaki `claude --version` ile aynı (yerel insan adımı; kurulu değilse R21 FAIL).
- Öteki 21 madde aynen (önce/sonra: aynı `dist/` ile `items[]` farkı yalnız R21'in eklenişi).

## 8. Gizli değişkenler ve dışarıya yazma
Adlar: laboratuvar `PRODUCT_REPO_DEPLOY_KEY`; ürün `DOCKERHUB_TOKEN`, `HOMEBREW_TAP_TOKEN`, değişken `DOCKERHUB_USERNAME`. Değerleri ajan görmez; `gh secret list` yalnız adları okur. Ajan ürün ve tap deposuna, kayıt defterlerine, DNS'e yazmaz; laboratuvarda yalnız `d-1/round`, `release/0.4.0` ve R0'ın geçici bozma dalını iter (geçici dal silinir). `gh` çağrıları etkin hesabı değiştirmeden komut başına belirteçle (orkestratör notu, 2026-11-14).

## 9. Sınamaların yeri
Vault: yalnız sentetik kasa (`spec/fixtures` mini vault'u, `scripts/scene.sh`'in vault'u) ya da D-NOTES kopyası, scratchpad'de. Kayıtlar `reports/rounds/D-1/r<n>/`; host değeri yok (`scripts/check_records_portable.sh` desenleri).
