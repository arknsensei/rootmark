# D-1 — Final raporu: ilk halka açık RootMark sürümü (0.4.0)

**Karar kuralının dalı: tur kaldı (D1-10 lafzen; K-D1.2) → karar talebi.** RootMark 0.4.0 yayımlandı; D1-10'un lafzen kalması yayımı durdurmaz (K-D1.2).

- D1-10: yayım kapısı tabloda 21/21 geçti. R21 (güncel Claude Code sınaması), dondurulmuş tüketicileri kırdığı için ayrı bir kapı oldu ve o da geçti. Kriter "22/22" dediği için lafzen kaldı.
- Öteki kriterler:
  - D1-1…D1-6 geçti.
  - D1-7, D1-8 ve D1-9 notla geçti (aşağıda).
  - D1-11, ürün deposundaki üç CodeQL uyarısının gerekçeli kapatılmasını (HUMAN_STEP 3j) bekliyor.
  - D1-12, turun PR'ının CI'ını bekliyor (`READY.md`).
- Ürün davranışı değişmedi: çalışma zamanı dizinlerinde yalnız dört sürüm satırı değişti; `tools/list` 0.3.4 ile bayt eşit.

## Adresler
- Ürün deposu: <https://github.com/rootmark-ai/rootmark>.
  - `main` = `transfer/v0.4.0` = `e6d8ae588eb332afef4c0ea4c097e1002914c2e4`. Kök commit, ebeveyni yok, ağacı `afdcc4589d18627f2dc8cd4085c802fc5c5ba8ec`.
  - Etiket `v0.4.0` bu commit'te.
- Sürüm: <https://github.com/rootmark-ai/rootmark/releases/tag/v0.4.0>.
  - "RootMark 0.4.0", 2026-11-15T14:44:29Z.
  - 12 varlık: üç arşiv, `SHA256SUMS`, `install.sh`/`install.ps1`, wheel, Obsidian zip + BRAT dosyaları, `THIRD_PARTY_LICENSES`.
- İmaj: `ghcr.io/rootmark-ai/rootmark:0.4.0` ve `rootmark/rootmark:0.4.0`.
  - İkisi aynı özet: `sha256:d574117090625b6e0b150c70be737f1c1079214b98e2a618bb71010bb0cacd4b`.
- Homebrew: `brew install rootmark-ai/tap/rootmark`. Tap `main` = 81be27b (PR #1).
- Belgeler: <https://docs.rootmark.ai> (HTTPS zorunlu).
- Laboratuvar: `arknsensei/rootmark`, etiket `v0.4.0` = `66d14e33fcbe3e61a06db84e300f3e0e9fc4ebdd`.

## Koşu kimlikleri
- **Başlangıç:** `ROUND_START` `87a042e`; dal `d-1/round`.
- **Laboratuvar PR'ları:**
  - #73: sürüm 0.4.0, birleşme 9bca7de.
  - #74: sınıflandırma düzeltmesi (K-D1.3), birleşme 545d0b2.
  - #75: ürün `release.yml` düzeltmesi (K-D1.4), birleşme 66d14e3.
  - `main` CI 43694986713, laboratuvar `release.yml` 43694996155 (66d14e3).
- **Aktarım:**
  - Laboratuvar `transfer.yml` 43703150416 (bootstrap).
  - Ürün `transfer-verify` 43705057071.
  - Ürün `main` CI 43705057064.
  - `pages` 43705057074 (yeniden koşuda yeşil).
- **Ürün yayımı:**
  - `release.yml` 43725125629: 3. denemede `success`. 1. ve 2. denemede yalnız tap işi düştü (D1F-19).
  - Ürün CodeQL ilk analizi: 43731780721.
- **Önceki denemeler (kayıtta görünür kalır):**
  - Ürünün ilk kök commit'i dd0465d (ağaç c61ce297). Bu ağaçta `release.yml` geçersiz YAML'dı (D1F-17), sonra K-D1.4 ile depo sıfırlandı.
  - Laboratuvar etiketi `v0.4.0` üç kez atıldı: 9bca7de → 545d0b2 → 66d14e3.
- **R21 kaydı:** `r6/claude_probe_r21_v0.4.0.json`, 2026-11-14T16:59:55Z. Claude Code 2.1.325.
- **Turun PR'ı ve son commit'inin CI'ı:** `READY.md`'de.

Yeniden tabanlamadan (K-D1.5) önceki sha'lar ile `main`'deki karşılıkları:

| Kayıtlarda | `main`'de |
|---|---|
| a46b49e (DESIGN) | ffe4bda |
| 3e243af (`ci.yml` deps) | f10562a |
| 263b85d | ace6540 |
| ee8b36d | aef08ca |
| 74e9b82 | 55fed12 |
| 825ac33 (sürüm) | 70abd99 |
| 2b4bde4 (K-D1.2 F2) | 8537843 |

## Çıkış kriterleri

| # | Sonuç | Kanıt |
|---|---|---|
| D1-1 | geçti | DESIGN (`ffe4bda`), R1'in ilk kod commit'inden önce. `ci.yml` `deps` işinde dondurma denetimleri var. Kasıtlı bozma `r0/frozen_probe.json`. |
| D1-2 | geçti | Ağaç `afdcc458…` beş yerde aynı:<br>• yerel iki koşu + CI benzetimi (`r6/transfer_local_v0.4.0_retag2.json`)<br>• `transfer.yml` günlüğü<br>• `transfer-verify` günlüğü (`ok`, sorun 0)<br>• ürün `main`'inin ağacı<br>• ajanın anonim klonda `transfer verify`'ı, 1253 satır bayt bayt (`r6/transfer_3i.json`)<br>Gözden geçirici aynı kimliği bağımsız iki koşuyla üretti. Kaynağı değişmiş kaplamada araç hata veriyor (R1 testleri). |
| D1-3 | geçti | Sınıflandırılmamış 0, çift sınıflı 0, yol çakışması 0, ölü desen 0. Sızıntı 0 (özel terim 3). Gerekçesiz istisna 0. Sınırlar D1F-15, D1F-16 (K-D1.6) ve D1F-23. |
| D1-4 | geçti | Ürün `main` CI 43705057064: cargo test/clippy/fmt, `conformance` 39/39, istemciler, `check_deps`, `check_dep_age` dolaylı dahil. |
| D1-5 | geçti | `git diff ROUND_START..HEAD -- core index server bindings spec clients/*/src`: yalnız dört `version` satırı. `tools/list` bayt eşit (`r6/tools_list_0.3.4_vs_0.4.0.json`). Laboratuvar `conformance` 39/39 (R7 kapıları). |
| D1-6 | geçti | `SHA256SUMS` 11 satır. `sha256sum -c` Linux'ta ve macOS'ta 11 OK, eksik 0. `gh attestation verify` 5 varlık + 2 imaj için exit 0. `THIRD_PARTY_LICENSES` arşivlerde ve imajda. `NOTICE` dendron MIT bildirimini taşıyor (`r6/published_channels.json`). |
| D1-7 | geçti (notla) | İki kayıt defterinde aynı özet. Yayımlanmış imajla dört adım, MCP, `--watch` 3/3, uid 10001, lisanslar: macOS Docker Desktop (ghcr) 8/8, Linux 8/8. Gerçek Claude Code 2.1.325 ile 1 oturumda `search` + `propose` SUCCEEDED (`r6/claude_probe_image_ghcr.json`). **Not:** Linux sınaması `docker:28-dind` içinde, Docker Desktop'ın Linux çekirdeğinde koştu; yerli bir Linux host değil. Ürün `image.yml`'in GitHub Linux koşucusundaki denetimi (`--version`, 39/39, uid) yerlidir. |
| D1-8 | geçti (notla) | `install.sh`: macOS arm64 ve temiz `ubuntu:24.04`. `brew install` + `brew test`: macOS ve Linuxbrew. BRAT varlıkları sürümde, zip `SHA256SUMS`'ta. Her yolda 0.4.0 ve 39/39. README blokları yazıldığı gibi 4/4. **Not:** Linux x86_64 yolları Docker Desktop'ın amd64 emülasyonunda koştu, yerli x86_64 host değil. |
| D1-9 | geçti (notla) | `docs.rootmark.ai`: zorunlu 9 sayfa 200, erişilen 12 sayfa 200, kırık iç bağlantı ve çapa 0. 16 laboratuvar atfı `v0.4.0`'da var (`r6/site_check.json`). İddia tablosu `progress/R3_claims.md`. **Not (D1F-22):** README'nin "Scale" satırı atıfsız. Bu satırı kapsam tanımı (NORTH_STAR §8'in tasarım sınırı) olarak sınıfladım, ölçüm iddiası olarak değil. "On every machine" ölçülen üç platformdan geniş. İkisi sonraki sürümün kaplamasında düzelir; yayımlanmış README değişmez. |
| D1-10 | **lafzen kaldı** | K-D1.2: `release_check` 21/21 + ayrı R21 kapısı ok (`r6/release_gate_claude.json`; kasıtlı bozma `release_gate_claude_tamper.json`). Kayıt yayımdan ≤ 7 gün önce. F1 ve F2 `FREEZE_EXCEPTIONS.md`'de. |
| D1-11 | bekliyor (3j) | **Laboratuvar:** Dependabot açık 0, CodeQL açık 0.<br>**Ürün:** Dependabot uyarıları açık, açık 0; gizli değer 0; CodeQL 3 açık. Üçü `js/bad-code-sanitization`, `clients/obsidian/e2e/real/run.ts`; laboratuvarda aynı üçü "used in tests" ile gerekçeli kapatılmış. Kapatma insanın (HUMAN_STEP 3j). Güvenlik güncellemeleri ürün deposunda bilerek kapalı (D1F-20).<br>**Sızıntı:** gizli değişkenlerin yalnız adları kayıtta; tur farkının 109 dosyasında belirteç, kişisel e-posta, gerçek ev yolu ve kişi adı 0 (gözden geçirici §5). |
| D1-12 | bekliyor (PR CI) | R7 kapıları yeniden tabanlanmış ağaçta (aşağıda). Gözden geçirici: KABUL (notlarla), `progress/R7.review.md`. PR'ın son commit'inin CI'ı `READY.md`'de. **Risk:** D1F-11 (PQ4-6, ubuntu `serve_watch` SIGTERM) PR CI'ında yeniden görülebilir. Görülürse test değiştirilmez; yeniden koşu insan kararıdır (K-P2Q4.4). |

**"Koştu" ile "geçti" ayrımı:** yukarıdaki her "geçti" bu turda koşulmuş bir sınamanın kaydına dayanır. Koşmuş ama kalmış olanlar:
- PR #73'ün ilk CI'ı (D1F-11, D1F-12).
- v0.4.0'ın ilk aktarımı (D1F-14).
- ürünün ilk `release.yml`'i (D1F-17).
- 3f'nin ilk iki denemesi (D1F-19).

Hepsinin düzeltmesi ve yeniden koşusu kayıtta.

## Karar talebi
D1-10 lafzen kaldı; karar kuralı bu durumda "tur kaldı; karar talebi" der. İnsandan istenen iki şey var:
1. D1-10'un kapanışı. Önerim: R21'i kalıcı olarak ayrı kapı tutmak ve kriterin metnini "tablo 21/21 + R21 kapısı" diye değiştirmek. Bu, K-D1.2'nin ilkesidir.
2. D1-11 ve D1-12'nin bekleyen kanıtları:
   - 3j'nin sonucu.
   - Turun PR'ının CI'ı.

## Bulgular
Bulgular `FINDINGS.md`'de (D1F-1…D1F-23); devredilenlerin kapanışı da aynı dosyada. Sonraki tura önerilenler:
- actionlint'in CI kapısı.
- `docker/*` eylemlerinin SHA'ya sabitlenmesi (D1F-10).
- Sızıntı denetiminin kör noktaları (D1F-15, D1F-16, D1F-23).
- Kaplama metinleri (D1F-20, D1F-22).
- Karar kimliği çakışması (D1F-21).
- `license.workspace` (D1F-8).
