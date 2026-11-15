# R6 — İki depoda güvenlik (D1-11)

Okuma: `gh api` (laboratuvar `arknsensei`, ürün `rootmark-ai`; salt okuma). Gizli değişkenlerin yalnız adları okunur.

## Laboratuvar `arknsensei/rootmark` (2026-11-15)
- Dependabot uyarıları açık (`vulnerability-alerts` 204), güvenlik güncellemeleri `enabled`. Açık uyarı **0** (tümü 77, hepsi `fixed`).
- CodeQL varsayılan kurulum `configured` (actions, javascript-typescript, python, rust; `default` sorgu takımı). `main` 66d14e3'ün analizleri: actions 0, javascript-typescript 3, python 0, rust 0 sonuç. Açık uyarı **0** (tümü 4: 3 `dismissed` — önceki turlarda gerekçeli —, 1 `fixed`).
- Gizli değer taraması (secret scanning) kapalı (laboratuvar; D1-11'in ölçütü değil).

## Ürün `rootmark-ai/rootmark` (2026-11-15)
- İlk okuma (3h'den sonra): Dependabot uyarıları **kapalı** (API 403 "Dependabot alerts are disabled"), güvenlik güncellemeleri `disabled`; CodeQL analizi **yok** (404 "no analysis found"). Gizli değer taraması ve itme koruması `enabled`, açık uyarı 0.
- `.github/dependabot.yml` ürün ağacında var (yalnız güvenlik güncellemeleri; sürüm güncellemeleri `open-pull-requests-limit: 0`).
- Ayar insanındır → HUMAN_STEP **3i** ✔: Dependabot uyarıları açık (204), CodeQL varsayılan kurulum `configured`, ilk analiz 43731780721. Güvenlik güncellemeleri (otomatik PR) **bilerek kapalı**: ürün `main`'i yalnız aktarımla dolar; uyarı çıkarsa düzeltme laboratuvarda yapılır, sonraki sürümle aktarılır. Not: ürünün `dependabot.yml` yorumu "security updates are enabled in the repository settings" der (laboratuvar için doğru, ürün için değil) → D1F-20.
- İlk CodeQL koşusu 43731780721 yeşil (rust, actions, javascript-typescript; e6d8ae5). Açık uyarılar (orkestratörün okuması): Dependabot 0, gizli değer taraması 0, CodeQL **3** — üçü `js/bad-code-sanitization` (medium), `clients/obsidian/e2e/real/run.ts` (#1–#3); rust 0, actions 0. Laboratuvarda aynı dosyada aynı kuralla #2–#4 olarak var ve "used in tests" gerekçesiyle kapatılmış (P2-Q.1/R3, Q1-7, kullanıcı kararı 2026-11-05). **Durum: 3 açık, laboratuvardaki gerekçeli kapatmanın eşi, kapatma bekliyor** (insan adımı, HUMAN_STEP 3j).
