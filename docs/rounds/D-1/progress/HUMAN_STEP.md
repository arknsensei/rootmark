# D-1 — İnsan adımları

Ajan bu adımların hiçbirini yapmaz; yalnız aşağıdaki **sınama komutlarıyla okur**. Gizli değişkenlerin değerleri sohbete, kayda ya da commit'e girmez; sınamalar yalnız adları ve varlığı okur. Ajan beklerken bağımsız görevlere (R1–R5) devam eder. Durum sütunu ajanın son okumasıdır.

## Adım 1 — Hesap ve depo hazırlığı (ilk gün; R0, 2026-11-14)
**Adım 1 TAMAM** (ajanın yeniden okuması, 2026-11-14 akşam; 1c'yi kullanıcının onayıyla orkestratör yaptı). İlk okuma (sabah): depo boş, tap 404, gizli değişken yok, CNAME yok.

| # | Adım | Beklenen sonuç | Ajanın sınaması (salt okuma) | Durum |
|---|---|---|---|---|
| 1a | `rootmark-ai/rootmark`: herkese açık, **boş bırakın** (README/lisans eklemeyin; ilk commit aktarımın kök commit'i olacak, DESIGN §1.4). Ayarlar → Actions → General: "Allow all actions" (ya da GitHub + doğrulanmış yaratıcılar), Workflow permissions "Read repository contents" (iş akışları kendi izinlerini yazar). Issues açık. Dal koruması **ilk aktarımdan sonra** (adım 3). | depo var, `size` 0, dal yok | `gh api repos/rootmark-ai/rootmark --jq '{visibility,size}'`; `gh api repos/rootmark-ai/rootmark/branches --jq length` → 0 | ✔ (public, boş, dal 0, issues açık) |
| 1b | `rootmark-ai/homebrew-tap` deposunu açın: herkese açık, **README ile başlatılmış** (`main` dalı olsun: tap PR'ının tabanı). | depo var, `main` dalı var | `gh api repos/rootmark-ai/homebrew-tap --jq .default_branch` → `main` | ✔ (public, `main`) |
| 1c | Aktarım yetkisi: bir SSH anahtar çifti üretin (`ssh-keygen -t ed25519 -N "" -C rootmark-transfer -f rootmark_transfer`); **açık** anahtarı `rootmark-ai/rootmark` → Settings → Deploy keys'e "Allow write access" ile ekleyin (başlık `lab-transfer`); **gizli** anahtarı laboratuvar `arknsensei/rootmark` → Settings → Secrets and variables → Actions → **`PRODUCT_REPO_DEPLOY_KEY`** olarak ekleyin; yerel anahtar dosyalarını silin. | laboratuvarda gizli değişken adı var; ürün deposunda yazma izinli bir dağıtım anahtarı | `gh secret list -R arknsensei/rootmark` (ad `PRODUCT_REPO_DEPLOY_KEY`); `gh api repos/rootmark-ai/rootmark/keys --jq '.[] \| {title, read_only}'` → `read_only: false` | ✔ (laboratuvar `PRODUCT_REPO_DEPLOY_KEY`; ürün `lab-transfer` yazma izinli) |
| 1d | Docker Hub (`rootmark` kullanıcı hesabı): `rootmark/rootmark` deposunu açın (public); Account settings → Personal access tokens → yeni belirteç, izin **Read & Write**; değerini ürün deposu `rootmark-ai/rootmark` → Secrets → **`DOCKERHUB_TOKEN`**; Variables → **`DOCKERHUB_USERNAME`** = `rootmark`. | ürün deposunda gizli değişken adı ve değişken var; Docker Hub deposu var | `gh secret list -R rootmark-ai/rootmark`; `gh variable list -R rootmark-ai/rootmark`; `curl -s https://hub.docker.com/v2/repositories/rootmark/rootmark/ \| jq .name` | ✔ (`DOCKERHUB_TOKEN`, `DOCKERHUB_USERNAME`; Docker Hub deposu orkestratörün okumasıyla 200) |
| 1e | Tap belirteci: `rootmark-ai` hesabında **ince taneli** kişisel erişim belirteci — Repository access: yalnız `rootmark-ai/homebrew-tap`; Permissions: Contents **Read and write**, Pull requests **Read and write**; süre ≤ 1 yıl. Değeri ürün deposu → Secrets → **`HOMEBREW_TAP_TOKEN`**. | gizli değişken adı var | `gh secret list -R rootmark-ai/rootmark` (ad `HOMEBREW_TAP_TOKEN`) | ✔ (`HOMEBREW_TAP_TOKEN`) |
| 1f | DNS (`rootmark.ai` alan adı): `docs` → `CNAME` → `rootmark-ai.github.io.`; GitHub → `rootmark-ai` hesabı Settings → Pages → "Add a domain" → `docs.rootmark.ai` (ya da `rootmark.ai`) doğrulaması için verilen **TXT** kaydı (`_github-pages-challenge-rootmark-ai.…`). Sitenin yayına girmesi (Pages'i etkinleştirmek, özel alan adı, HTTPS) adım 3'tedir. | CNAME ve TXT çözülüyor | `dig +short CNAME docs.rootmark.ai` → `rootmark-ai.github.io.`; `dig +short TXT _github-pages-challenge-rootmark-ai.docs.rootmark.ai` (ya da GitHub'ın verdiği ad) dolu | ✔ (CNAME → `rootmark-ai.github.io.`, TXT var) |
| 1g | `security@rootmark.ai` rol adresi (yönlendirme ya da posta kutusu). Hangi kişisel adrese yönlendiği hiçbir kayda yazılmaz. | adres posta alıyor | `dig +short MX rootmark.ai` dolu (ajan posta göndermez) | ✔ (MX var; adres sınanmadı) |

Bitince bu dosyada her satırın "Durum"unu siz değiştirmeyin; ajan (ya da orkestratör) sınamaları koşar ve yazar.

## Adım 2 — Laboratuvar 0.4.0 (R6; ajan `release/0.4.0` dalını ve PR'ını hazırlar)
**Durum (2026-11-14):** PR **#73** (`release/0.4.0` → `main`, "rootmark 0.4.0") açık; laboratuvar `release_check` bu dalda 21/21 + ayrı R21 kapısı (K-D1.2; ilk koşu F1'le 22/22 idi) (`reports/rounds/D-1/r6/release_check.json`; R21: Claude Code 2.1.325, kayıt 2026-11-14 — 7 günlük pencere 2026-11-21'de kapanır, sonra R21 yeniden koşulmalı). Ön koşul: #73'ün CI'ı yeşil. K-D1-T1 kapandı (K-D1.2, b): R21 ayrı kapı; düzeltme #73'e tek push'la gitti; #73 yeşil ve birleşti (aşağıdaki Durum sütunu).

| # | Adım | Beklenen sonuç | Ajanın sınaması | Durum |
|---|---|---|---|---|
| 2a | PR #73'ü (`release/0.4.0`) **rebase** ile birleştirin (0.3.4'teki #69 gibi; turun PR'ından önce). | `main`'de sürüm 0.4.0, `research/transfer/`, `product/overlay/` | `git show origin/main:Cargo.toml \| grep '^version'` → `0.4.0` | ✔ #73 CI 16/16 (koşu 43628292002, head 42d7ce2); `main` = 9bca7de, ağacı 42d7ce2'ninkiyle aynı (4124d0b7…), sürüm 0.4.0 |
| 2b | Laboratuvar `release.yml`'ini birleşmiş `main`'de tetikleyin (laboratuvarın kendi doğrulaması; GitHub sürümü **yayımlanmaz**). | koşu yeşil | `gh run list -R arknsensei/rootmark -w release -L 1` | tetiklendi: koşu 43633859618 (9bca7de), son okuma sürüyor |
| 2c | Birleşmiş `main` commit'ine etiket: `git tag -a v0.4.0 -m "rootmark 0.4.0" <sha> && git push origin v0.4.0` | etiket var | `git ls-remote --tags origin v0.4.0` | bekliyor (2b ve `main` push CI'ı yeşil olunca) |

## Adım 3 — Aktarım ve ürün yayımı (adım 1 ve 2 bittikten sonra)
| # | Adım | Beklenen sonuç | Ajanın sınaması |
|---|---|---|---|
| 3a | Laboratuvarda **Actions → transfer → Run workflow**: `tag = v0.4.0`, `bootstrap = true` (ilk aktarım). | iş yeşil; özet: iki koşu aynı ağaç kimliği; ürün deposunda `transfer/v0.4.0` dalı, kök commit | `gh run list -R arknsensei/rootmark -w transfer -L 1`; `gh api repos/rootmark-ai/rootmark/branches` |
| 3b | Ürün deposunda varsayılan dalı bu commit'ten oluşturun: `gh api -X POST repos/rootmark-ai/rootmark/git/refs -f ref=refs/heads/main -f sha=<transfer/v0.4.0'ın commit'i>`; Settings → General → Default branch `main`. | `main` = aktarım commit'i | `gh api repos/rootmark-ai/rootmark/commits/main --jq .commit.message` (`Lab-Commit`, `Lab-Tag`, `Transfer-Tree`) |
| 3c | `main`'e push `ci`, `transfer-verify` ve `pages`'i başlatır: `ci` ve `transfer-verify` yeşil olmalı. Ardından dal koruması (`main`: PR zorunlu, gerekli denetimler `dependency direction and age`, `test + lint + conformance (macos-15)`, `test + lint + conformance (ubuntu-latest)`, `transfer verify`; doğrusal geçmiş; zorla itme yok). | iki iş akışı yeşil; kural seti etkin | `gh run list -R rootmark-ai/rootmark -L 5`; `gh api repos/rootmark-ai/rootmark/rulesets` |
| 3d | Pages: Settings → Pages → Source **GitHub Actions**; Custom domain `docs.rootmark.ai`; DNS doğrulandıktan sonra **Enforce HTTPS**. `pages` iş akışını yeniden koşun. | `https://docs.rootmark.ai/` 200 | `curl -sI https://docs.rootmark.ai/` |
| 3e | Ürün etiketi: `git tag -a v0.4.0 -m "RootMark 0.4.0" <ürün main commit'i>` ürün deposuna itilir (ya da `gh api -X POST repos/rootmark-ai/rootmark/git/refs -f ref=refs/tags/v0.4.0 -f sha=<sha>` — hafif etiket de olur). | etiket var | `gh api repos/rootmark-ai/rootmark/git/ref/tags/v0.4.0` |
| 3f | Ürün deposunda **Actions → release → Run workflow**: `tag = v0.4.0`. GitHub sürümü, `image` ve `tap` işleri bu koşunun içindedir. | koşu yeşil; sürüm 0.4.0 varlıklarıyla; iki kayıt defterinde aynı özet; tap deposunda PR | `gh release view v0.4.0 -R rootmark-ai/rootmark`; `docker buildx imagetools inspect …`; `gh pr list -R rootmark-ai/homebrew-tap` |
| 3g | `ghcr.io` paketini herkese açık yapın (rootmark-ai → Packages → rootmark → Package settings → Change visibility → Public). | anonim `docker pull ghcr.io/rootmark-ai/rootmark:0.4.0` çalışır | `docker logout ghcr.io; docker pull …` |
| 3h | Tap PR'ını birleştirin. | `brew install rootmark-ai/tap/rootmark` çalışır | ajanın R6 sınaması |

Bunlardan sonra ajan R6'nın "yayımlanmış kanallardan sınama"sını (temiz Linux konteyneri ve bu Mac) koşar.

## Adım 5 — Turun dalı `main`'in gerisinde (R7, turun PR'ından önce)
#73 rebase ile birleşti, commit sha'ları değişti: `d-1/round`'un 25 commit'inin 23'ü `main`'de başka sha'larla var. Ajan zorla itmez. Yerel deneme (2026-11-14; dal itilmedi, silindi): `git rebase origin/main` çatışmasız; yinelenen 23 commit "patch contents already upstream" diye düşüyor, `main`'in üstünde yalnız turun kayıt commit'leri kalıyor. Yeniden tabanlanmış dal ile eski `d-1/round` arasındaki fark yalnız #73'ün sürüm satırları (CHANGELOG, Cargo/npm sürümleri, RELEASE.md başlığı).

Ajan R7'de kayıtlarını bitirince yeniden tabanlamayı yerelde yapar ve **yeni** bir dal olarak iter: `d-1/round-final` (zorla itme değil). Sonra iki yoldan biri insanındır:

| # | Adım | Beklenen sonuç | Ajanın sınaması |
|---|---|---|---|
| 5a (önerilen) | `git fetch origin && git push --force-with-lease=d-1/round:<ajanın yazdığı eski sha> origin origin/d-1/round-final:refs/heads/d-1/round`; sonra ajan `gh pr create … --head d-1/round` komutunu GOAL'deki gibi koşar | `d-1/round` = `d-1/round-final`, `main`'in üstünde | `git merge-base --is-ancestor origin/main origin/d-1/round` |
| 5b | Zorla itme istenmezse ajan PR'ı `--head d-1/round-final` ile açar (GOAL'deki `--head`'den sapma; ASSUMPTIONS'a yazılır) | PR `main`'e tek PR | `gh pr list -R arknsensei/rootmark --head d-1/round-final` |

## Adım 4 — İsteğe bağlı (ürün yayımlandıktan sonra; çıkış kriteri değil)
Laboratuvar Pages yönlendirmesi: `docs/rounds/D-1/progress/lab_redirect/index.html`'i laboratuvarın Pages sitesinin `spec/index.html`'i olarak yayımlamak (laboratuvar `pages.yml`'ini bir kez bu dosyayla koşturmak), sonra `arknsensei/rootmark` → `arknsensei/rootmark-lab` (K17, D0-K4). Eski rozetin `badge.json`'u yönlendirilemez (bilinen sınır).
