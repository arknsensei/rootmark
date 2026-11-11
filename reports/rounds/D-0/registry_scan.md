# D-0 — Kayıt defteri taraması (okunur tablo)

Kaynak: `registry_scan.json` (betik `research/d0/registry_scan.py` @ `7306155`; koşu 2026-11-11T20:19:48Z → 2026-11-11T20:22:54Z UTC). Her hücrenin URL'si, zamanı, yanıt kodu ve gövde sağlaması JSON'dadır. Bu tablo JSON'dan üretilir; elle düzenlenmez.

Hücre: 216/216; eksik 0; kanıtsız 0; dolu 5, boş 207, erişilemedi 4.

## Matris

| sütun | `rootmark` | `rootmark-bindings` | `rootmark-core` | `rootmark-index` | `rootmark-server` | `rootmark-ai` | `getrootmark` | `rhizomark` | `rootmarkhq` |
|---|---|---|---|---|---|---|---|---|---|
| `crates.io` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `pypi` | dolu | boş | boş | boş | boş | boş | boş | boş | boş |
| `testpypi` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `npm_pkg` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `npm_org` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `npm_user` | boş | boş | boş | boş | erişilemedi | erişilemedi | erişilemedi | boş | erişilemedi |
| `dockerhub` | boş | boş (geçersiz ad) | boş (geçersiz ad) | boş (geçersiz ad) | boş (geçersiz ad) | boş (geçersiz ad) | boş | boş | boş |
| `ghcr` | dolu | boş | boş | boş | boş | boş | boş | boş | boş |
| `github` | dolu | boş | boş | boş | boş | boş | boş | boş | boş |
| `brew_formula` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `brew_cask` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `brew_tap` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `obsidian_id` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `obsidian_name` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `vscode_publisher` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `vscode_ext` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `openvsx_ns` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `openvsx_ext` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_ai` | dolu | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_com` | dolu | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_dev` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_io` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_app` | boş | boş | boş | boş | boş | boş | boş | boş | boş |
| `dom_org` | boş | boş | boş | boş | boş | boş | boş | boş | boş |

## Dolu ve erişilemeyen hücreler

| ad | sütun | UTC | kod | özet | not |
|---|---|---|---|---|---|
| `rootmark` | `pypi` | 2026-11-11T20:19:49Z | 200 | {"canonical_name": "rootmark", "summary": "Rootmark SDK -- agent instrumentation", "latest_version": "0.1.5", "first_upload": "2026-08-17", "last_upload": "2026-10-24", "release_count": 7, "project_urls": ["https://github.com/IvanAdactic/rootmark", "https:/… |  |
| `rootmark` | `ghcr` | 2026-11-11T20:19:52Z | 401 | {"public_image_rootmark": false, "token_status": 403, "namespace_owner": {"login": "RootMark", "type": "User", "created": "2018-03-08", "public_repos": 0}, "namespace_owner_from": "github row"} | namespace belongs to the GitHub account of the same name; no public image <ns>/rootmark |
| `rootmark` | `github` | 2026-11-11T20:19:51Z | 200 | {"login": "RootMark", "type": "User", "created": "2018-03-08", "public_repos": 0} |  |
| `rootmark` | `dom_ai` | 2026-11-11T20:19:55Z | 200 | {"registered": "2026-11-10", "expires": "2028-11-10", "registrar": "GoDaddy.com, LLC", "status": ["client delete prohibited", "client renew prohibited", "client transfer prohibited", "client update prohibited", "add period"]} |  |
| `rootmark` | `dom_com` | 2026-11-11T20:19:55Z | 200 | {"registered": "2004-03-24", "expires": "2027-03-24", "registrar": "Squarespace Domains LLC", "status": ["client delete prohibited", "client transfer prohibited"]} |  |
| `rootmark-server` | `npm_user` | 2026-11-11T20:21:04Z | 403 | {} | www.npmjs.com answered 403 (browser check; also for control names in R0); user existence has no public API -> HUMAN_STEP.md |
| `rootmark-ai` | `npm_user` | 2026-11-11T20:21:33Z | 403 | {} | www.npmjs.com answered 403 (browser check; also for control names in R0); user existence has no public API -> HUMAN_STEP.md |
| `getrootmark` | `npm_user` | 2026-11-11T20:22:02Z | 403 | {} | www.npmjs.com answered 403 (browser check; also for control names in R0); user existence has no public API -> HUMAN_STEP.md |
| `rootmarkhq` | `npm_user` | 2026-11-11T20:22:39Z | 403 | {} | www.npmjs.com answered 403 (browser check; also for control names in R0); user existence has no public API -> HUMAN_STEP.md |

## Yakın alan: GitHub'da adında `rootmark` geçen depolar

`https://api.github.com/search/repositories?q=rootmark+in:name&sort=stars&per_page=30` · 2026-11-11T20:22:55Z · 200 · toplam 10

| depo | tanıtım (ilk 120) | oluşturma | son itme | yıldız |
|---|---|---|---|---|
| `arknsensei/rootmark` | Block-level lineage, provenance and a deterministic derived index for Markdown written by people and agents | 2026-10-17 | 2026-11-11 | 1 |
| `rootmarkjoy/rootmarkjoy` |  | 2022-12-02 | 2024-01-25 | 1 |
| `bmosoane/rootmark` |  | 2026-10-21 | 2026-10-21 | 0 |
| `Sketchify-Dev/rootmark` | Verifiable data and query middleware for the Portaldot V3 chain. One query across EVM and ink! events, every result carr | 2026-10-27 | 2026-11-08 | 0 |
| `bpstr/rootmark` | Rootmark turns Markdown repositories into static websites. | 2026-09-06 | 2026-09-07 | 0 |
| `northgardtracker/rootmark` | X-ray for AGENTS.md and coding-agent instruction drift. | 2026-06-21 | 2026-06-23 | 0 |
| `kgreen1200/rootmark` |  | 2026-11-08 | 2026-11-08 | 0 |
| `002sofianjeri-debug/RootMarket` |  | 2026-07-07 | 2026-07-09 | 0 |
| `rootmarkexe/rootmarkexe` |  | 2025-09-11 | 2026-11-09 | 0 |
| `002sofianjeri-debug/RootMarket-Backend` |  | 2026-07-07 | 2026-07-07 | 0 |

## Yakın alan: web siteleri

| URL | UTC | kod / hata | yönlenme |
|---|---|---|---|
| https://rootmark.com/ | 2026-11-11T20:22:56Z | 200 | https://www.rootmark.com/ |
| https://rootmark.dev/ | 2026-11-11T20:23:08Z | URLError |  |
| https://rootmark.ai/ | 2026-11-11T20:23:29Z | 503 |  |
