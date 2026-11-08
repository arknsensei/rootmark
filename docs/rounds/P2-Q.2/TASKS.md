# P2-Q.2 — Görevler

Genel kurallar:
- **Sıra:** R0 → R1 → R2 → R3 → R4 → R5. R1 ve R2 bağımsızdır, sırayla yapılır. R3'ün belge işleri beklerken sürebilir; `minimatch` 2026-11-10'dan sonra. R0'ın `DESIGN.md` ve `TESTS.md`'si commit'lenmeden R1/R2'nin kodu yazılmaz.
- **Kayıt ve dallar:** progress `docs/rounds/P2-Q.2/progress/`. Commit öneki `P2-Q.2/R<n>:`. Dal `p2-q.2/round`, yalnız birleşmiş `main`'den (K-P2.2). Yayım hazırlığı ayrı dal `release/0.3.2`.
- **Gözden geçirici:** yama turu, araç dozu (K-P2.4): yalnız R5'te, tek tur.
- **Testler:** yeni testler yeni dosyalarda. Dondurulmuş test ve kaydedicilere dokunulmaz; değişmesi gerekirse DEV_PROCESS §4.
- **CI yapılandırması:** `.github/workflows/**` değişirse R0'ın "değişmesi beklenen kayıtlar" listesine o değişiklikten **önce** adıyla eklenir (P2-Q.1 Q1-4 dersi).
- **İnsan adımı ve sorular:** insan adımı beklemek durma değildir (`HUMAN_STEP.md`). Soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye. Spec'in tanımlamadığı bir durum: PARK + karar talebi (uydurma yok).
- **Gerçek Claude Code sınaması:** yalnız D-NOTES kopyası bir vault'ta (depo dışında, scratchpad). Kişisel vault ve qorbit'e dokunulmaz. Kayıtlarda token, içerik ve host değeri yok.

## R0 — Açılış ve tasarım
**Yapar:**
- `ROUND_START`; DECISIONS K-P2Q2.1 (açılış).
- PQ1-1'in yeniden üretimi: yayımlanmış `v0.3.1` + yüklü Claude Code, başsız (`claude -p --mcp-config <dosya> --strict-mcp-config`, hata ayıklama günlüğü). İstemcinin `initialize` ve `tools/list` isteklerinin biçimi araya giren bir kaydediciyle (stdio'yu kopyalayan küçük bir betik; içerik alanları yazılmadan) kaydedilir: `progress/R0_repro.md` ve `progress/r0_requests.json`.
- `progress/DESIGN.md`:
  - kök neden: rmcp'nin doğrulama koşulu, istemcinin protokol sürümü ve `_meta`'sı;
  - aday rmcp sürümlerinin (≥ 14 gün) değişiklik notlarının ve kaynağının okunması: hangisi sorunu çözüyor, API değişiklikleri neler;
  - seçilen yöntem ve alternatifi;
  - uyumluluk matrisi;
  - PQ1-4'ün yöntemi ve değişmezi.
- `progress/TESTS.md`: ön-kayıtlı testler ve beklenen sonuçlar.
- Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi: sürüm dizgileri (0.3.2), E1-4/R14 (etiket), değişecekse CI yapılandırması. Beklenmeyen değişiklik = adsız fark.

**Kabul:** Q2-1; Q2-2 (i).

## R1 — PQ1-1: MCP uyumluluğu
**Yapar:**
- `DESIGN.md`'deki yöntem: `rmcp` yükseltmesi (`server/Cargo.toml`, `Cargo.lock`; `check_deps`, `check_dep_age` temiz) ya da `mcp.rs`'te eski oturum modeli.
- 0.3.1'in `tools/list` çıktısı (adlar, şemalar) önce kaydedilir; sonra bayt eşitlik testi.
- Yeni entegrasyon testi (`server/tests/mcp_compat.rs`): matrisin her hücresi; `initialize` → `tools/list` → `search` → `propose` → günlükte ajan aktörü; `r0_requests.json`'daki gerçek istemci biçimleri dahil. CI'da `ubuntu-latest` ve macOS'ta koşar.
- Gerçek Claude Code başsız sınaması (yerel ikili): `progress/R1_claude_code.md`.

**Kabul:** Q2-2 (ii) ve (iii)'ün R1 yarısı; Q2-3.

## R2 — PQ1-4: kaldırma beklemesi
**Yapar:**
- Kaldırmanın alınma anı, eski yolun geç tasdik penceresinin yeni yolun alımından sonra dolmasına bağlanır.
- Yeni entegrasyon testi (`server/tests/watch_hold.rs` gibi), macOS ve Linux'ta ≥ 20 tekrar: yeni yolun alımı test yapılandırmasıyla `W`'den uzun geciktirilir, geç tasdikli yeniden adlandırma yeni yolu yükseltir, eski yol bir kez kapanır.
- `watch_remove.rs`, `watch_newdir.rs` ve `serve_watch.rs` aynen geçer.

**Kabul:** Q2-4.

## R3 — Belgeler ve güvenlik
**Yapar:**
- (a) Kılavuz §4.1: güncel Claude Code ile bağlanma ve doğrulama (sınanmış); Known issues'tan PQ1-1 çıkar.
- (b) Kılavuzun "Removals are taken a little late" maddesine `unverified` altında tasdikli yeniden adlandırmanın eski yolunun `unattested_fs` ile kapandığı eklenir (P2-Q.1 doğrulaması Öneri 3a).
- (c) `CHANGELOG.md` 0.3.2 taslağı.
- (d) `docs/DEV_PROCESS.md`'ye anahtar saklama kuralı: `~/.config/rootmark-lab/`, `600`, `README.txt`; scratchpad yasak.
- (e) `minimatch` `overrides` (2026-11-10'dan sonra; `make lint-ts`, `make test-ts` temiz). Değilse gerekçe.
- (f) Uyarıların son durumu `progress/R3_security.md`'ye yazılır (`gh api` dependabot ve code-scanning listeleri).

**Kabul:** Q2-7.

## R4 — Doğrulama koşuları ve 0.3.2
**Yapar:**
- (a) Kademe 2 bir tetikleme (`gh workflow run verify.yml --ref p2-q.2/round -f sha=<sha>`): yapıtlar, `tier2 pair/fails`, `portable`; değişen kayıtların R0 listesiyle karşılaştırması.
- (b) `make verify-mac` bir kez. Önce `fseventsd` sağlığı `ps` ile; sağlıksızsa `HUMAN_STEP.md`.
- (c) `release/0.3.2` dalı, R1–R20, PR; `HUMAN_STEP.md` (birleşme, `release.yml`, etiket, sürüm).
- (d) Sürüm yayımlanınca:
  - gerçek Claude Code sınaması yüklü 0.3.2 ile (`progress/R4_claude_code.md`);
  - D-NOTES senaryosu P2-Q'nün sürücüsüyle Mac, Docker ve CI'da (`pilot.yml`, bir koşu). Mac ↔ Docker ↔ CI ve 0.3.1 kayıtlarıyla karşılaştırma.

  Kayıtlar `reports/rounds/P2-Q.2/` altına.

**Kabul:** Q2-2 (iii)'ün R4 yarısı, Q2-5, Q2-6'nın D-NOTES yarısı, Q2-8.

## R5 — Rapor, gözden geçirici, hazır
**Yapar:**
- Kapılar: `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- `FINDINGS.md`: PQ1-1 ve PQ1-4 kapandı, yeni bulgular.
- Gözden geçirici tek tur (GOAL başlığı).
- `reports/rounds/P2-Q.2/FINAL.md`: Q2-1..Q2-9, karar kuralının dalı, koşu kimlikleri, "koştu/geçti" ayrımı.
- READY; tek PR: `gh pr create --base main --head p2-q.2/round --title "P2-Q.2: MCP uyumluluğu ve 0.3.2" --body-file reports/rounds/P2-Q.2/FINAL.md`.

**Kabul:** Q2-6, Q2-9; READY'de "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
