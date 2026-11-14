# D-1 — İnsan adımları

Ajan bu adımların hiçbirini yapmaz; yalnız aşağıdaki **sınama komutlarıyla okur**. Gizli değişkenlerin değerleri sohbete, kayda ya da commit'e girmez; sınamalar yalnız adları ve varlığı okur. Ajan beklerken bağımsız görevlere (R1–R5) devam eder. Durum sütunu ajanın son okumasıdır.

## Adım 1 — Hesap ve depo hazırlığı (ilk gün; R0, 2026-11-14)
Ajanın son okuması (2026-11-14, salt okuma): `rootmark-ai/rootmark` var, herkese açık, boş (dal yok); `rootmark-ai/homebrew-tap` yok (404); laboratuvarda ve ürün deposunda gizli değişken yok; `docs.rootmark.ai` CNAME yok.

| # | Adım | Beklenen sonuç | Ajanın sınaması (salt okuma) | Durum |
|---|---|---|---|---|
| 1a | `rootmark-ai/rootmark`: herkese açık, **boş bırakın** (README/lisans eklemeyin; ilk commit aktarımın kök commit'i olacak, DESIGN §1.4). Ayarlar → Actions → General: "Allow all actions" (ya da GitHub + doğrulanmış yaratıcılar), Workflow permissions "Read repository contents" (iş akışları kendi izinlerini yazar). Issues açık. Dal koruması **ilk aktarımdan sonra** (adım 3). | depo var, `size` 0, dal yok | `gh api repos/rootmark-ai/rootmark --jq '{visibility,size}'`; `gh api repos/rootmark-ai/rootmark/branches --jq length` → 0 | depo var, boş ✔ |
| 1b | `rootmark-ai/homebrew-tap` deposunu açın: herkese açık, **README ile başlatılmış** (`main` dalı olsun: tap PR'ının tabanı). | depo var, `main` dalı var | `gh api repos/rootmark-ai/homebrew-tap --jq .default_branch` → `main` | yok (404) |
| 1c | Aktarım yetkisi: bir SSH anahtar çifti üretin (`ssh-keygen -t ed25519 -N "" -C rootmark-transfer -f rootmark_transfer`); **açık** anahtarı `rootmark-ai/rootmark` → Settings → Deploy keys'e "Allow write access" ile ekleyin (başlık `lab-transfer`); **gizli** anahtarı laboratuvar `arknsensei/rootmark` → Settings → Secrets and variables → Actions → **`PRODUCT_REPO_DEPLOY_KEY`** olarak ekleyin; yerel anahtar dosyalarını silin. | laboratuvarda gizli değişken adı var; ürün deposunda yazma izinli bir dağıtım anahtarı | `gh secret list -R arknsensei/rootmark` (ad `PRODUCT_REPO_DEPLOY_KEY`); `gh api repos/rootmark-ai/rootmark/keys --jq '.[] \| {title, read_only}'` → `read_only: false` | yok |
| 1d | Docker Hub (`rootmark` kullanıcı hesabı): `rootmark/rootmark` deposunu açın (public); Account settings → Personal access tokens → yeni belirteç, izin **Read & Write**; değerini ürün deposu `rootmark-ai/rootmark` → Secrets → **`DOCKERHUB_TOKEN`**; Variables → **`DOCKERHUB_USERNAME`** = `rootmark`. | ürün deposunda gizli değişken adı ve değişken var; Docker Hub deposu var | `gh secret list -R rootmark-ai/rootmark`; `gh variable list -R rootmark-ai/rootmark`; `curl -s https://hub.docker.com/v2/repositories/rootmark/rootmark/ \| jq .name` | yok |
| 1e | Tap belirteci: `rootmark-ai` hesabında **ince taneli** kişisel erişim belirteci — Repository access: yalnız `rootmark-ai/homebrew-tap`; Permissions: Contents **Read and write**, Pull requests **Read and write**; süre ≤ 1 yıl. Değeri ürün deposu → Secrets → **`HOMEBREW_TAP_TOKEN`**. | gizli değişken adı var | `gh secret list -R rootmark-ai/rootmark` (ad `HOMEBREW_TAP_TOKEN`) | yok |
| 1f | DNS (`rootmark.ai` alan adı): `docs` → `CNAME` → `rootmark-ai.github.io.`; GitHub → `rootmark-ai` hesabı Settings → Pages → "Add a domain" → `docs.rootmark.ai` (ya da `rootmark.ai`) doğrulaması için verilen **TXT** kaydı (`_github-pages-challenge-rootmark-ai.…`). Sitenin yayına girmesi (Pages'i etkinleştirmek, özel alan adı, HTTPS) adım 3'tedir. | CNAME ve TXT çözülüyor | `dig +short CNAME docs.rootmark.ai` → `rootmark-ai.github.io.`; `dig +short TXT _github-pages-challenge-rootmark-ai.docs.rootmark.ai` (ya da GitHub'ın verdiği ad) dolu | yok |
| 1g | `security@rootmark.ai` rol adresi (yönlendirme ya da posta kutusu). Hangi kişisel adrese yönlendiği hiçbir kayda yazılmaz. | adres posta alıyor | `dig +short MX rootmark.ai` dolu (ajan posta göndermez) | — |

Bitince bu dosyada her satırın "Durum"unu siz değiştirmeyin; ajan (ya da orkestratör) sınamaları koşar ve yazar.

## Adım 2 — Laboratuvar 0.4.0 (R6'da yazılacak)
Yer tutucu: `release/0.4.0` PR'ının birleşmesi, laboratuvar `release.yml` (doğrulama), etiket `v0.4.0`. Ayrıntılar R1–R5 bitince eklenir.

## Adım 3 — Aktarım ve ürün yayımı (R6'da yazılacak)
Yer tutucu: `transfer.yml` (`tag=v0.4.0`, `bootstrap=true`), ürün `main`'inin kök commit'ten oluşturulması, dal koruması, ürün etiketi `v0.4.0`, ürün `release.yml`, tap PR'ının birleşmesi, Pages + alan adı + HTTPS, `ghcr.io` paketinin herkese açık yapılması.

## Adım 4 — İsteğe bağlı (ürün yayımlandıktan sonra; çıkış kriteri değil)
Laboratuvar Pages yönlendirmesi ve `arknsensei/rootmark` → `arknsensei/rootmark-lab` (K17, D0-K4).
