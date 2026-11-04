# R3 — Güvenlik uyarıları (CodeQL, Dependabot)

Kaynak: `gh api repos/arknsensei/rootmark/code-scanning/alerts?state=open` ve `…/dependabot/alerts?state=open` (2026-11-05, dal açılırken).

## CodeQL (4 açık)
| # | Kural | Yer | Karar |
|---|---|---|---|
| 1 | `actions/missing-workflow-permissions` | `.github/workflows/release.yml:30` (`binary` işi) | **düzeltildi:** iş akışının en üstüne `permissions: contents: read`; `record` işi kendi yetkilerini (`id-token: write`, `attestations: write`) zaten taşıyor. Uyarı, değişiklik `main`'e girip CodeQL yeniden taradığında kapanır (birleşmeden sonra denetlenir). |
| 2, 3, 4 | `js/bad-code-sanitization` | `clients/obsidian/e2e/real/run.ts:170, 171, 247` | **gerekçeyle "test kodu" olarak kapatılır (düzeltilmez).** Gerekçe: (i) dosya dondurulmuş gerçek Obsidian e2e harness'idir (`research/FROZEN.md`, P2-L/F2); düzeltme DEV_PROCESS §4 istisnası ve R15'in (gerçek Obsidian 11/11) önce/sonra yeniden koşusunu gerektirir — bu yama turunun kapsamı dışında bir risk/maliyet; (ii) `J()` = `JSON.stringify`, girdileri harness'in kendi sabit dizgileridir (sorgu sözcükleri, `DOC`, checkpoint adları) ve CDP üzerinden harness'in kendi geçici Obsidian profilinde değerlendirilir; dışarıdan gelen girdi yok, yayımlanan eklentide (`main.js`) yok. Kapatma GitHub'da bir dış sistem yazımıdır; ajanın oturumunda otomatik mod bunu reddetti → **insan adımı** (`HUMAN_STEP.md`), komut aşağıda. |

İnsan adımı komutu (her uyarı için, 2 3 4):

```sh
for n in 2 3 4; do gh api -X PATCH repos/arknsensei/rootmark/code-scanning/alerts/$n \
  -f state=dismissed -f dismissed_reason="used in tests" \
  -f dismissed_comment="P2-Q.1/R3 (Q1-7): frozen real-Obsidian e2e harness; J() = JSON.stringify of constant test strings evaluated over CDP in a throwaway Obsidian profile; not shipped. docs/rounds/P2-Q.1/progress/R3_security.md"; done
```

## Dependabot (2 açık)
| # | Paket | Yer | Önem | Karar |
|---|---|---|---|---|
| 80 | `minimatch` 10.2.6 (GHSA-g3hv-75wg-m249, "denial of service via globparts amplification", etkilenen `>= 7.0.0, < 11.0.1`) | `clients/obsidian/package-lock.json` | orta | **gerekçeyle açık.** Düzeltme 11.0.1, yayım 2026-10-27T22:26Z; deponun 14 gün kuralı (`check_dep_age`) 2026-11-10'dan önce izin vermez (bugün 2026-11-05). Paket dolaylı ve yalnız geliştirme kapsamında (`eslint` 10.12.0 ve `typescript-eslint` 8.71.0 üzerinden; `scope: development`): yalnız lint sırasında deponun kendi glob desenleriyle çalışır, eklentinin `main.js`'ine girmez, yayımlanmaz. 2026-11-10'dan sonra `overrides` ile 11.0.1 (eslint uyumluysa; `make lint-ts`, `make test-ts`) — P2-Q.1'in birleşmesinden sonraki ilk bakım (D-0 açılışı) ya da bir Dependabot PR'ı. |
| 81 | `minimatch` (aynı) | `clients/vscode/package-lock.json` | orta | aynı gerekçe (VS Code istemci iskeleti, yayımlanmaz). |

Yayımlanan Rust ikilisinde (`Cargo.lock`) uyarı yok.
