# Yayım kontrol listesi (rootmark 0.1.0; P1-E/E5, P1-E.1 K-E1.1)

**Hedef (K-E1.1):** 0.1.0'ın tek yayım hedefi `aarch64-apple-darwin`'dir. `x86_64-unknown-linux-gnu` ve `x86_64-pc-windows-msvc` CI'da derlenir, `rootmark conformance` koşturulur ve artefakt saklanır, ama yayım kriteri değildir ("CI bilgi"); `x86_64-apple-darwin` hedef değildir. Kurulum paketleri (installer, imzalama, dağıtım kanalları) faz 2'nin konusudur.

Her madde `scripts/release_check.sh [<dist>]` ile betikle doğrulanır (madde kimlikleri R1…R20 ve R4a; betik her biri için `ok`/`FAIL` yazar, çıkış 0 yalnız hepsi geçerse). `scripts/release.sh` paketi `dist/`'e kurar (hedef kayıtları `scripts/release_target.sh` ile, CI'daki kayıtla aynı biçimde; gerçek Obsidian e2e'si `dist/plugin_e2e_real/`); hiçbir şey yayımlamaz, etiket atmaz. Betik hiçbir boru hattını erken çıkan bir okuyucuyla (`grep -q`) bitirmez: liste önce bir dosyaya alınır (P1-E doğrulaması V7, E1-4).

| # | Madde | Doğrulama (release_check.sh) |
|---|---|---|
| R1 | Tek sürüm: bütün crate'ler ve eklenti 0.1.0 | `Cargo.toml` (`core`, `index`, `server`, `bindings/python`) ve `clients/obsidian/{package,manifest}.json` sürümleri eşit |
| R2 | Spec sürümü CRITERION'ın güncel değeri | `server/src/tools.rs SPEC_VERSION` = `core/tests/conformance.rs` `spec_version` = `spec/SPEC.md` başlığındaki sürüm |
| R3 | `rootmark --version` / `--spec-version` doğru | yerel ikili `rootmark 0.1.0` / spec sürümü |
| R4 | Yayım ikilisi kendi `rootmark conformance`'ında 37/37 | `dist/release.json`: `required_targets` = [`aarch64-apple-darwin`], `required_ok`; o kaydın `version` = `rootmark 0.1.0`, `spec_version`, `conformance` 37/37, `sha256`; diğer hedefler bilgi: kayıtları varsa aynı biçimde, yoksa nedeniyle `skipped` (K-E1.1) |
| R4a | Paket güncel | `dist/release.json`: `version` = 0.1.0, `worktree_dirty` false, kaydedilen commit ile HEAD arasında paket girdileri aynı (`git diff --quiet <commit> HEAD -- Cargo.toml Cargo.lock core index server bindings clients/obsidian spec models/MANIFEST.json scripts/release.sh scripts/linux_cross_build.sh scripts/fetch_model.sh packaging LICENSE README.md`); yalnız kayıt (ör. `docs/rounds/`) commit'leri güncelliği bozmaz |
| R5 | `rootmark conformance` = `make conformance` | `scripts/conformance_equiv.sh` test-test eşit |
| R6 | Arşivler ve SHA256SUMS | `dist/SHA256SUMS` her arşivi kapsar, `shasum -c` geçer |
| R7 | Python wheel | `dist/wheel/*-0.1.0-*.whl` (dağıtım adı `rootmark_bindings`) içinde `rootmark_core` modülü (liste dosyaya alınıp aranır; SIGPIPE yok, E1-4) |
| R8 | Obsidian paketi (mağaza başvurusu yok) | `dist/obsidian/rootmark-0.1.0.zip` içinde yalnız `main.js`, `manifest.json`, `styles.css`; `main.js` < 500 KB; `package.json`'da çalışma zamanı bağımlılığı yok |
| R9 | Spec yayım dizini | `dist/spec-<ver>/` içinde `SPEC.md`, `CONFORMANCE.md`, `LICENSE`, `fixtures/MANIFEST.json`, `conformance.json` (level 3, 37/37, manifest ok) |
| R10 | Lisans | kökte `LICENSE` Apache-2.0 metni (`Apache License` / `Version 2.0, January 2004`), arşivlerde de |
| R11 | Belgeler | `CHANGELOG.md`'de `## 0.1.0`, `docs/THREAT_MODEL.md`, bu dosya |
| R12 | cargo-binstall ve Homebrew taslağı | `server/Cargo.toml [package.metadata.binstall]`; `dist/homebrew/rootmark.rb` sürüm ve sha256 dolu |
| R13 | CI derleme | `.github/workflows/release.yml` yalnız `workflow_dispatch` ile tetiklenir; matris `aarch64-apple-darwin` (zorunlu) ve `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (`continue-on-error`); `x86_64-apple-darwin` yok; her artefaktta `rootmark conformance` (K-E1.1) |
| R14 | Etiket insan adımıdır | `v0.1.0` etiketi yok, ya da (P2-0/Z0'dan beri) var ve R16'nın raporu "Güvenilir", etiketin commit'i en yeni `ROUND_START`'ın kendisi ya da atası (etiketi bir tur atmaz; 0.1.0'ı GH-0'da insan attı, K-GH.4) |
| R15 | Gerçek Obsidian e2e (K-E1.3) | `dist/plugin_e2e_real/plugin_e2e_real.json`: 11/11, E1-9 bağlantı adımı geçti, `pass: true`; Obsidian ikilisi yoksa kayıt `skipped` olur ve R15 FAIL'dir (atlanma yayım için geçme sayılmaz) |
| R16 | İnsan adımı 1'in bağı (P2-0/Z0; P1-E.1 doğrulaması öneri 1) | aşağıdaki insan adımı 1'in adını verdiği `reports/rounds/<tur>/VERIFICATION.md` var ve son `**Hüküm: …**` satırı "Güvenilir" |
| R17 | Kurulum betikleri (P2-A/A3) | `packaging/install.sh` (Linux x86_64, macOS arm64) ve `packaging/install.ps1` (Windows x86_64) var; `install.sh` bu paketin arşivini yerel bir `SHA256SUMS` ile (model indirmeden) kurar, kurulan ikili `rootmark <ver>` der; bir baytı değişmiş arşiv reddedilir ve hiçbir şey kurulmaz |
| R18 | CI kurulum, sağlama, köken (P2-A/A3) | `release.yml`'in her işi kendi arşivini kurulum betiğiyle (`install.sh --from`, `install.ps1 -From`) kurar ve kurulan ikiliyle `--version` + `rootmark conformance` koşturur; son iş bütün arşivlerin `SHA256SUMS`'ını yazar ve `actions/attest-build-provenance` ile köken tasdiki üretir (`id-token: write`, `attestations: write`) |
| R19 | İmzalama insan adımıdır (P2-A/A3) | aşağıdaki insan adımları Apple notarization'ı ve Windows Authenticode'u adlandırır; köken doğrulaması `gh attestation verify` yazılıdır |
| R20 | Eylemler sabit (P2-A/A3) | her iş akışının her `uses:`'u tam sürüme sabit (`@vX.Y.Z`; araç zinciri eylemi `dtolnay/rust-toolchain@<toolchain>` hariç); yaşları `scripts/check_dep_age.sh` (≥ 14 gün) |

## İnsan adımları (betik yapmaz)
1. P1-E.1 bağımsız doğrulaması (`reports/rounds/P1-E.1/VERIFICATION.md`) **"Güvenilir"** demeden hiçbiri yapılmaz (P1-E'nin hükmü "şu koşullarla güvenilir" idi; koşulu P1-E.1 kapattı).
2. `server/Cargo.toml`'a `repository` alanını yaz (binstall `{ repo }` ve Homebrew URL'si için).
3. `.github/workflows/release.yml`'ı elle tetikle (workflow_dispatch; başka tetik yok). `release-json` artefaktındaki `dist/release.json`'da `aarch64-apple-darwin` kaydı `ok` olmalı (`--version`, `--spec-version`, `rootmark conformance` 37/37, `sha256`); yerel `dist/release.json` ile sürüm, spec sürümü ve uyumluluk sonucu karşılaştırılır (hash'ler farklı makinelerde farklı olabilir; kaydedilir). Linux ve Windows işlerinin sonucu bilgi olarak kaydedilir (K-E1.1).
4. `git tag -a v0.1.0 -m "rootmark 0.1.0"` ve itme; GitHub release'e arşivleri, wheel'i, eklenti zip'ini ve `SHA256SUMS`'ı ekle. Eklentinin `main.js`, `manifest.json` ve `styles.css` dosyaları release'e **ayrıca** tek tek eklenir (Obsidian BRAT bunları zip'ten değil, release varlıklarından indirir; README "Obsidian (BRAT)").
5. `dist/homebrew/rootmark.rb`'yi tap'e koy; `cargo binstall rootmark-server` ile dene.
6. Obsidian community store başvurusu bu yayımın kapsamında değildir (GOAL P1-E).

## Kurulum (P2-A/A3; K-E1.1'in hedefleri değişmez)
Yayım hedefi yine yalnız `aarch64-apple-darwin`'dir (K-E1.1); Linux `x86_64` ve Windows `x86_64` arşivleri CI'da derlenir, kurulur, sınanır ve artefakt olarak saklanır, kurulum betikleri onlar için de çalışır. Yayımlanabilir paketler (bir yayımda release varlığı olarak eklenir):
- `rootmark-<ver>-<target>.tar.gz` (Linux, macOS) ve `rootmark-<ver>-x86_64-pc-windows-msvc.zip`; içinde ikili, `LICENSE`, `README.md`, `models/MANIFEST.json` ve `models/fetch_model.sh` (model dosyaları arşivde değildir; kurulum indirir ve manifest'le denetler).
- `SHA256SUMS` (bütün arşivler ve iki kurulum betiği) ve her birinin GitHub köken tasdiki (artifact attestation; `SHA256SUMS` arşivle aynı yerden gelir: bütünlüğü kanıtlar, kökeni tasdik kanıtlar).
- `install.sh` (önce indir, doğrula, sonra çalıştır: `curl -fsSLO …/v<ver>/install.sh`, `curl -fsSLO …/v<ver>/SHA256SUMS`, `gh attestation verify install.sh -R arknsensei/rootmark`, `grep ' install.sh$' SHA256SUMS | sha256sum -c`, `bash install.sh --version <ver>`; boru ile `curl … | bash` kökeni doğrulamaz) — arşivi ve `SHA256SUMS`'ı indirir, SHA-256'yı denetler, `~/.local/bin/rootmark` ve `~/.local/share/rootmark/models` kurar, modeli indirir; `ROOTMARK_MODELS=~/.local/share/rootmark/models`.
- `install.ps1`: `powershell -ExecutionPolicy Bypass -File install.ps1 -Version <ver>` — aynısı Windows için (`%LOCALAPPDATA%\rootmark`).
- `cargo binstall rootmark-server` (`server/Cargo.toml [package.metadata.binstall]`; arşiv adları ve iç dizin düzeniyle eşleşir; yayım ve crates.io insan adımıdır).

**Kurulum doğrulama (betikle: R17, R18; elle):**
1. `release.yml` koşusunun `release-json` artefaktında her hedefin `<target>.json` (`ok`) ve `<target>.install.json` (`ok`, `conformance` 39/39) kaydı.
2. `release-files` artefaktı indirilir: `sha256sum -c SHA256SUMS` (macOS: `shasum -a 256 -c`) geçer.
3. Her arşiv için `gh attestation verify <arşiv> -R arknsensei/rootmark` "Verification succeeded" der.
4. Temiz bir makinede `install.sh` / `install.ps1` ile kurulum, `rootmark --version`, `rootmark conformance` 39/39.

**İmzalama (insan adımı; bu turda yapılmaz, GOAL P2-A):**
- **Apple notarization:** Developer ID Application sertifikasıyla `codesign --options runtime --timestamp --sign "Developer ID Application: …" rootmark`, `xcrun notarytool submit rootmark.zip --keychain-profile <profil> --wait`; tek dosyalık bir ikiliye bilet zımbalanamaz (`stapler` yalnız paket/uygulama), Gatekeeper çevrimiçi denetler. Sertifika ve parola hiçbir zaman depoya girmez.
- **Windows Authenticode:** bir kod imzalama sertifikasıyla (EV ya da Azure Trusted Signing) `signtool sign /fd SHA256 /tr <zaman damgası sunucusu> /td SHA256 rootmark.exe`, ardından zip ve `SHA256SUMS` yeniden üretilir.
- İmzalama yapılırsa imzalı ikililer yeni arşivlerdir: `SHA256SUMS` ve köken tasdiki imzalı arşivler için yeniden üretilmelidir (CI'da bir insan onaylı iş ya da yerelde), yoksa R6/R18 eşleşmez.

