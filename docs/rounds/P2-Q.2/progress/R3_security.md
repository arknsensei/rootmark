# R3 (f) — Açık güvenlik uyarıları

## Durum (2026-11-08, R3; `gh api repos/arknsensei/rootmark/{dependabot,code-scanning}/alerts?state=open`)
| Uyarı | Paket | Manifest | Önem | Kapsam | Aralık / düzeltme | Gerekçe ve plan |
|---|---|---|---|---|---|---|
| Dependabot #80 | `minimatch` 10.2.6 (npm) | `clients/obsidian/package-lock.json` | orta | geliştirme | `>= 7.0.0, < 11.0.1` → 11.0.1 (GHSA-g3hv-75wg-m249, "potential denial of service via globparts amplification") | Yalnız geliştirme araçlarının bağımlılığı (`eslint` ^10.2.5, `@eslint/config-array` ^10.2.4, `@typescript-eslint/typescript-estree` ^10.2.2); eklentinin `main.js`'ine girmez. 11.0.1'in yayımı 2026-10-27T22:26Z → 14 gün kuralı (`check_dep_age`) **2026-11-10T22:26Z'den sonra** sağlanır. Plan: o tarihten sonra `package.json` `overrides` `"minimatch": "11.0.1"`, `npm install` (kilit), `make lint-ts` ve `make test-ts` temiz olmalı (bağımlılar ^10 ister; 11'in API'si eslint'i bozarsa gerekçeyle açık kalır). |
| Dependabot #81 | `minimatch` 10.2.6 (npm) | `clients/vscode/package-lock.json` | orta | geliştirme | aynı | aynı (VS Code iskeleti) |
| CodeQL | — | — | — | — | — | açık uyarı **0** (#1 PR #51 ile `fixed`, #2–#4 K-P2Q1.2 ile kapatıldı) |

`rmcp` 3.6.0'ın yükseltmesi yeni bir Dependabot uyarısı açmadı (Rust tarafında açık uyarı yok).

## Son durum (2026-11-10T22:4xZ, R3 (e))
- `clients/obsidian/package.json` ve `clients/vscode/package.json` `overrides`: `"minimatch": "11.0.1"` (yayım 2026-10-27T22:26:03Z; commit 14 gün sonra). `npm install` → iki kilitte `minimatch` 10.2.6 → 11.0.1.
- 11.0.1 yeni bir dolaylı bağımlılık getirir: `@isaacs/nonrecursive-glob-match` `^1.0.1`. `npm`'in seçtiği 1.1.0 2026-11-05'te yayımlandı (5 gün; 14 gün kuralının ruhuna aykırı — `check_dep_age` yalnız doğrudan bağımlılıkları denetler). Bu yüzden o da `overrides` ile **1.0.1**'e sabitlendi (yayım 2026-10-27T04:24Z, ≥ 14 gün; `^1.0.1`'i karşılar). `brace-expansion` 5.0.12 (2026-09-14) değişmedi.
- Kapılar: `make lint-ts` temiz, `make test-ts` 164/164 + 12/12, `npm audit` iki istemcide 0, `scripts/check_dep_age.sh` exit 0, eklenti derlemesi (`npm run build`) exit 0. Ürün kodu ve `clients/*/src` değişmedi.
- Dependabot #80/#81 varsayılan dalı (`main`) tarar; turun PR'ı birleşince kapanmaları beklenir (birleşme insan adımı). CodeQL açık uyarı 0.
