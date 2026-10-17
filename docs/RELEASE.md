# Yayım kontrol listesi (mdk 0.1.0; P1-E/E5, P1-E.1 K-E1.1)

**Hedef (K-E1.1):** 0.1.0'ın tek yayım hedefi `aarch64-apple-darwin`'dir. `x86_64-unknown-linux-gnu` ve `x86_64-pc-windows-msvc` CI'da derlenir, `mdk conformance` koşturulur ve artefakt saklanır, ama yayım kriteri değildir ("CI bilgi"); `x86_64-apple-darwin` hedef değildir. Kurulum paketleri (installer, imzalama, dağıtım kanalları) faz 2'nin konusudur.

Her madde `scripts/release_check.sh [<dist>]` ile betikle doğrulanır (madde kimlikleri R1…R15 ve R4a; betik her biri için `ok`/`FAIL` yazar, çıkış 0 yalnız hepsi geçerse). `scripts/release.sh` paketi `dist/`'e kurar (hedef kayıtları `scripts/release_target.sh` ile, CI'daki kayıtla aynı biçimde; gerçek Obsidian e2e'si `dist/plugin_e2e_real/`); hiçbir şey yayımlamaz, etiket atmaz. Betik hiçbir boru hattını erken çıkan bir okuyucuyla (`grep -q`) bitirmez: liste önce bir dosyaya alınır (P1-E doğrulaması V7, E1-4).

| # | Madde | Doğrulama (release_check.sh) |
|---|---|---|
| R1 | Tek sürüm: bütün crate'ler ve eklenti 0.1.0 | `Cargo.toml` (`core`, `index`, `server`, `bindings/python`) ve `clients/obsidian/{package,manifest}.json` sürümleri eşit |
| R2 | Spec sürümü CRITERION'ın güncel değeri | `server/src/tools.rs SPEC_VERSION` = `core/tests/conformance.rs` `spec_version` = `spec/SPEC.md` başlığındaki sürüm |
| R3 | `mdk --version` / `--spec-version` doğru | yerel ikili `mdk 0.1.0` / spec sürümü |
| R4 | Yayım ikilisi kendi `mdk conformance`'ında 34/34 | `dist/release.json`: `required_targets` = [`aarch64-apple-darwin`], `required_ok`; o kaydın `version` = `mdk 0.1.0`, `spec_version`, `conformance` 34/34, `sha256`; diğer hedefler bilgi: kayıtları varsa aynı biçimde, yoksa nedeniyle `skipped` (K-E1.1) |
| R4a | Paket güncel | `dist/release.json`: `version` = 0.1.0, `worktree_dirty` false, kaydedilen commit ile HEAD arasında paket girdileri aynı (`git diff --quiet <commit> HEAD -- Cargo.toml Cargo.lock core index server bindings clients/obsidian spec models/MANIFEST.json scripts/release.sh scripts/linux_cross_build.sh scripts/fetch_model.sh packaging LICENSE README.md`); yalnız kayıt (ör. `docs/rounds/`) commit'leri güncelliği bozmaz |
| R5 | `mdk conformance` = `make conformance` | `scripts/conformance_equiv.sh` test-test eşit |
| R6 | Arşivler ve SHA256SUMS | `dist/SHA256SUMS` her arşivi kapsar, `shasum -c` geçer |
| R7 | Python wheel | `dist/wheel/*-0.1.0-*.whl` (dağıtım adı `mdkernel_bindings`) içinde `mdkernel_core` modülü (liste dosyaya alınıp aranır; SIGPIPE yok, E1-4) |
| R8 | Obsidian paketi (mağaza başvurusu yok) | `dist/obsidian/mdkernel-0.1.0.zip` içinde yalnız `main.js`, `manifest.json`, `styles.css`; `main.js` < 500 KB; `package.json`'da çalışma zamanı bağımlılığı yok |
| R9 | Spec yayım dizini | `dist/spec-<ver>/` içinde `SPEC.md`, `CONFORMANCE.md`, `LICENSE`, `fixtures/MANIFEST.json`, `conformance.json` (level 3, 34/34, manifest ok) |
| R10 | Lisans | kökte `LICENSE` Apache-2.0 metni (`Apache License` / `Version 2.0, January 2004`), arşivlerde de |
| R11 | Belgeler | `CHANGELOG.md`'de `## 0.1.0`, `docs/THREAT_MODEL.md`, bu dosya |
| R12 | cargo-binstall ve Homebrew taslağı | `server/Cargo.toml [package.metadata.binstall]`; `dist/homebrew/mdk.rb` sürüm ve sha256 dolu |
| R13 | CI derleme | `.github/workflows/release.yml` yalnız `workflow_dispatch` ile tetiklenir; matris `aarch64-apple-darwin` (zorunlu) ve `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (`continue-on-error`); `x86_64-apple-darwin` yok; her artefaktta `mdk conformance` (K-E1.1) |
| R14 | Etiket yok | `git tag -l v0.1.0` boş: etiketi bu tur atmaz |
| R15 | Gerçek Obsidian e2e (K-E1.3) | `dist/plugin_e2e_real/plugin_e2e_real.json`: 11/11, E1-9 bağlantı adımı geçti, `pass: true`; Obsidian ikilisi yoksa kayıt `skipped` olur ve R15 FAIL'dir (atlanma yayım için geçme sayılmaz) |

## İnsan adımları (betik yapmaz)
1. P1-E.1 bağımsız doğrulaması (`reports/rounds/P1-E.1/VERIFICATION.md`) **"Güvenilir"** demeden hiçbiri yapılmaz (P1-E'nin hükmü "şu koşullarla güvenilir" idi; koşulu P1-E.1 kapattı).
2. `server/Cargo.toml`'a `repository` alanını yaz (binstall `{ repo }` ve Homebrew URL'si için).
3. `.github/workflows/release.yml`'ı elle tetikle (workflow_dispatch; başka tetik yok). `release-json` artefaktındaki `dist/release.json`'da `aarch64-apple-darwin` kaydı `ok` olmalı (`--version`, `--spec-version`, `mdk conformance` 34/34, `sha256`); yerel `dist/release.json` ile sürüm, spec sürümü ve uyumluluk sonucu karşılaştırılır (hash'ler farklı makinelerde farklı olabilir; kaydedilir). Linux ve Windows işlerinin sonucu bilgi olarak kaydedilir (K-E1.1).
4. `git tag -a v0.1.0 -m "mdk 0.1.0"` ve itme; GitHub release'e arşivleri, wheel'i, eklenti zip'ini ve `SHA256SUMS`'ı ekle.
5. `dist/homebrew/mdk.rb`'yi tap'e koy; `cargo binstall mdkernel-server` ile dene.
6. Obsidian community store başvurusu bu yayımın kapsamında değildir (GOAL P1-E).
