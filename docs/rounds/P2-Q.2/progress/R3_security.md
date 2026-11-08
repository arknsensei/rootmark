# R3 (f) — Açık güvenlik uyarıları

## Durum (2026-11-08, R3; `gh api repos/arknsensei/rootmark/{dependabot,code-scanning}/alerts?state=open`)
| Uyarı | Paket | Manifest | Önem | Kapsam | Aralık / düzeltme | Gerekçe ve plan |
|---|---|---|---|---|---|---|
| Dependabot #80 | `minimatch` 10.2.6 (npm) | `clients/obsidian/package-lock.json` | orta | geliştirme | `>= 7.0.0, < 11.0.1` → 11.0.1 (GHSA-g3hv-75wg-m249, "potential denial of service via globparts amplification") | Yalnız geliştirme araçlarının bağımlılığı (`eslint` ^10.2.5, `@eslint/config-array` ^10.2.4, `@typescript-eslint/typescript-estree` ^10.2.2); eklentinin `main.js`'ine girmez. 11.0.1'in yayımı 2026-10-27T22:26Z → 14 gün kuralı (`check_dep_age`) **2026-11-10T22:26Z'den sonra** sağlanır. Plan: o tarihten sonra `package.json` `overrides` `"minimatch": "11.0.1"`, `npm install` (kilit), `make lint-ts` ve `make test-ts` temiz olmalı (bağımlılar ^10 ister; 11'in API'si eslint'i bozarsa gerekçeyle açık kalır). |
| Dependabot #81 | `minimatch` 10.2.6 (npm) | `clients/vscode/package-lock.json` | orta | geliştirme | aynı | aynı (VS Code iskeleti) |
| CodeQL | — | — | — | — | — | açık uyarı **0** (#1 PR #51 ile `fixed`, #2–#4 K-P2Q1.2 ile kapatıldı) |

`rmcp` 3.6.0'ın yükseltmesi yeni bir Dependabot uyarısı açmadı (Rust tarafında açık uyarı yok).

## Son durum
R3 (e)'den sonra güncellenir.
