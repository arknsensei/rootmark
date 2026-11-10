# P2-Q.2 — Rapor: MCP uyumluluğu, kaldırma beklemesi ve 0.3.2

`ROUND_START` = `76977e0`; dal `p2-q.2/round` (birleşmiş `main` `c544a9e`'nin üstünde; yeniden temellendirme öncesi uç yerelde `backup/p2-q.2-round-pre-rebase`). Yayım: `release/0.3.2` → PR #55 (`7081416`) ve `release.yml` düzeltmesi PR #56 (`c544a9e`); `release.yml` `42574633881`; `v0.3.2` `c544a9e`'de yayımlı (12 dosya). Gözden geçirici (tek tur): **KABUL** (`docs/rounds/P2-Q.2/progress/R5.review.md`).

## Özet
- **PQ1-1 kapandı.** `rmcp` `=3.5.0` → `=3.6.0` (crates.io, 2026-10-20, ≥ 14 gün; fork/yama yok; `mcp.rs` değişmedi). rmcp 3.5.0, 2026-07-28+ sürüm bildiren bir `_meta`'nın bütün alanlarını istiyordu; Claude Code 2.1.319 önce `server/discover`'ı tam 2026-07-28 meta'sıyla gönderiyor, sonra `initialize` 2025-11-25'e düşüyor. 3.6.0 keşfi `-32022` ile yanıtlıyor ve oturumu kilitlemiyor. Gerçek Claude Code 2.1.319 hem yerel ikiliyle (R1) hem yayımlanmış 0.3.2 ile (R4) 11/11 araç listeledi; `search` ve `propose` başarılı, öneri `proposal` katmanında, `source = claude-code/2.1.319`.
- **PQ1-4 kapandı (test edilen mekanizma için).** Kaldırmanın beklemesi toplu işin alımından sonra başlıyor. Yeni yolun alımı yapay olarak `W`'den uzun geciktirilince eski kodda geç tasdik reddediliyordu (H1/H5 kaldı); düzeltmeyle macOS ve Linux'ta H1 20/20. Gerçek Obsidian'ın 0.3b oranı ise düşmedi, yükseldi (PQ2-2, `reported_only`; gözden geçirici B1) — nedeni ayrıştırılmadı, P2-C'ye.
- **0.3.2 yayımlandı**; D-NOTES Mac ↔ Docker ↔ CI ↔ 0.3.1 kayıtları bayt eşit (`events_hash` `173c13df…`, vektörlerde ULP 0): iki düzeltme D-NOTES'un hiçbir baytını değiştirmedi.
- **Karar kuralının dalı: hepsi geçti** (Q2-9'un PR kademe 1 yarısı `READY.md`'de). Sıradaki D-0 (isim ve depo), sonra D-1.

## Kriterler
"Koştu" ile "geçti" ayrı sayılır: her satırda koşan adımların sayısı ve geçenler.

| Kod | Sonuç | Kanıt |
|---|---|---|
| Q2-1 | **geçti** | `DESIGN.md` ve `TESTS.md` R0'da (R1'in ilk kod commit'inden önce); kök neden istemcinin kaydedilmiş istek biçimiyle (`progress/r0_requests.json`, kaydedici `research/p2q2/mcp_tap.py`; içerik ve host değeri yok) ve rmcp 3.5.0'ın kaynağıyla (DESIGN §1); yöntem `rmcp =3.6.0`, alternatif (eski oturum modeli) gerekçeyle reddedildi (§2); fork 0 |
| Q2-2 | **geçti** | (i) 0.3.1 + Claude Code 2.1.319: 0 araç, `-32602` (`claude_code/r0_v0.3.1.json`) — koştu 1, beklendiği gibi kaldı. (ii) `mcp_compat.rs` M1–M11 (10 test fonksiyonu): rmcp 3.5.0 ile 6/10 (M7–M10 kaldı), 3.6.0 ile 10/10 macOS ve Linux Docker (`r1/`, `r2/linux_arm64_docker_8fd0703.log`); CI `ubuntu-latest` ve macOS: turun PR'ı (`READY.md`). (iii) gerçek Claude Code 2.1.319: R1 yerel ikili (`claude_code/r1_local.json`) ve R4 yayımlanmış 0.3.2 (`claude_code/r4_v0.3.2.json`) — her biri CC1 11/11, CC2 `search`, CC3 `propose` (`author_kind = Agent`, `proposal`) koştu 3 / geçti 3 |
| Q2-3 | **geçti** | `tools/list` normalize JSON ve ham baytlar 0.3.1 ile eşit (`server/tests/fixtures/p2q2/`, yayımlanmış 0.3.1 ikilisinden); Inspector 2.9.0 şemalar 11/11, çağrılar 11/11; `mcp_schemas`, `contract`, `p1d1_server`, `p1d1_criteria`, `watch_remove`, `watch_newdir`, `serve_watch` aynen geçti; M1 (2025-06-18) ve M2 (2025-11-25) `_meta`'sız çalışıyor; `make conformance` 39/39 |
| Q2-4 | **geçti** | `watch_hold.rs` H1–H5: eski kodda H1 ve H5 kaldı (`r2/watch_hold_macos_before_fix.log`), düzeltmeyle macOS 5/5 (**H1 20/20**) ve Linux Docker arm64 5/5 (H1 20/20); `watch_remove` 9/9, `watch_newdir` 3/3 iki platformda; CI: turun PR'ı |
| Q2-5 | **geçti** | kademe 2 tek tetikleme `42515401135` (`d6eb311`): zincir çıkışları 0, D farkı 0, sayılan FAIL 0/0, `portable` bulgu 0/0; commit'ler arası farklar yalnız R0 listesindekiler (sürüm dizgileri, E1-4/R14) (`tier2/42515401135/`). Mac kümesi bir kez (RUN `20261108T193444Z-d6eb311`, 3 tekrar): hüküm `pass`; değişen O değerleri adıyla (O-DB4 0.3b 0.2693/0.2697/0.2922, 0.3c p95, D-10 p95) (`mac_set/`). CI yapılandırması değişikliği (`release.yml` `macos-15`) kademe 2'den sonra, DESIGN §6 ekiyle değişiklikten önce adıyla |
| Q2-6 | **geçti** | `git diff 76977e0..HEAD -- core index bindings spec 'clients/*/src'`: yalnız üç `version` satırı; `server/` farkı `Cargo.toml`, `src/serve.rs` (izleyici) ve yeni testler (`mcp.rs` değişmedi); dondurulmuş dosyalar değişmedi (`check_frozen` 0); eski vault'lar göçsüz (conformance 39/39, D-NOTES 0.3.1 kaydıyla eşit); D-NOTES 0.3.2 Mac ↔ Docker ↔ CI `42575714093` ↔ 0.3.1 bayt eşit (`pilot/compare.json`) |
| Q2-7 | **geçti** | kılavuz §4.1 Claude Code 2.1.319 ile sınandı, Known issues'tan PQ1-1 çıktı; Öneri 3a yazılı; anahtar kuralı DEV_PROCESS §2'de; CHANGELOG `## 0.3.2`; `rmcp` 3.6.0 19 günlükken alındı; Dependabot #80/#81 `minimatch` 11.0.1 `overrides` ile giderildi (dolaylı `@isaacs/nonrecursive-glob-match` 1.0.1'e sabit; uyarılar `main`'e birleşince kapanır), CodeQL açık 0 (`progress/R3_security.md`) |
| Q2-8 | **geçti** | yerel paket `release_check` **21/21** (`release/`); `release.yml` `42574633881` zorunlu iki hedef `ok`; `v0.3.2` yayımlı (insan, 12 dosya) |
| Q2-9 | geçti (PR CI'ı ile kesinleşir) | `701a595`'te `make test` 0 (cargo 450/0, pytest 316, vitest 164 + 12), `make lint` 0, `make conformance` 39/39, `make verify FAST=1` 0 (RUN `20261110T225334Z-701a595`, 1279 s, çift eşit), `check_frozen` 0, `check_deps` 0, `check_dep_age` 0 (`r5/gates.json`); gözden geçirici KABUL; turun PR'ında kademe 1: `READY.md` |

## Karar kuralı
GOAL'ün dalları sırayla: Q2-1 geçti (park yok); Q2-2 ve Q2-3 geçti → PQ1-1 kapandı, sözleşme aynen; Q2-4 geçti → PQ1-4 kapandı; Q2-5, Q2-6, Q2-7, Q2-8 geçti; Q2-9 PR'ın kademe 1'iyle → **"Hepsi geçti"**: PQ1-1 ve PQ1-4 kapandı, 0.3.2 yayımlı, ajan yolu güncel Claude Code ile çalışıyor. Sıradaki D-0, sonra D-1 (K-D0.1). Kısa bağımsız doğrulama birleşmeden sonra.

## Koşu kimlikleri
| Ne | Kimlik |
|---|---|
| Kademe 2 | `verify.yml` `42515401135` (`d6eb311`) |
| Mac kümesi | RUN `20261108T193444Z-d6eb311` |
| Yayım | `release.yml` `42532598520` (macos-14 imajı emekli, açılamadı; PQ2-5), `42574633881` (başarılı) |
| D-NOTES CI | `pilot.yml` `42575714093` |
| `make verify FAST=1` | RUN `20261110T225334Z-701a595` |
| Turun PR'ının CI'ı | `READY.md` |

## Kararlar
K-P2Q2.1 (açılış). Tur içinde yeni karar talebi yok.

## Bulgular
`reports/rounds/P2-Q.2/FINDINGS.md`: PQ1-1 ve PQ1-4 kapandı; yeni PQ2-1 (Claude Code'un SIGKILL'i bekleyen kaldırmaları kaybettiriyor → P2-C), PQ2-2 (0.3b oranı yükseldi, `reported_only` → P2-C/P2-H), PQ2-3 (watchkit port yarışı → P2-C), PQ2-4 (`server/CLAUDE.md` eski rmcp sürümü), PQ2-5 (`release.yml` macos-14, düzeltildi), PQ2-6 (`minimatch`'in dolaylı bağımlılığı, düzeltildi; `check_dep_age` önerisi), PQ2-7 (izleyici ve test kapsamı, küçük → P2-C).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
