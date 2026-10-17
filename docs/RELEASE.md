# Yayım kontrol listesi (mdk 0.1.0; P1-E/E5)

Her madde `scripts/release_check.sh [<dist>]` ile betikle doğrulanır (madde kimlikleri R1…R14; betik her biri için `ok`/`FAIL` yazar, çıkış 0 yalnız hepsi geçerse). `scripts/release.sh` paketi `dist/`'e kurar; hiçbir şey yayımlamaz, etiket atmaz.

| # | Madde | Doğrulama (release_check.sh) |
|---|---|---|
| R1 | Tek sürüm: bütün crate'ler ve eklenti 0.1.0 | `Cargo.toml` (`core`, `index`, `server`, `bindings/python`) ve `clients/obsidian/{package,manifest}.json` sürümleri eşit |
| R2 | Spec sürümü CRITERION'ın güncel değeri | `server/src/tools.rs SPEC_VERSION` = `core/tests/conformance.rs` `spec_version` = `spec/SPEC.md` başlığındaki sürüm |
| R3 | `mdk --version` / `--spec-version` doğru | yerel ikili `mdk 0.1.0` / spec sürümü |
| R4 | Her yayım ikilisi kendi `mdk conformance`'ında 34/34 | `dist/release.json`: her hedefin kaydı `ok`; olmayan hedef nedeniyle listelenir (E-7) |
| R5 | `mdk conformance` = `make conformance` | `scripts/conformance_equiv.sh` test-test eşit |
| R6 | Arşivler ve SHA256SUMS | `dist/SHA256SUMS` her arşivi kapsar, `shasum -c` geçer |
| R7 | Python wheel | `dist/wheel/*-0.1.0-*.whl` (dağıtım adı `mdkernel_bindings`) içinde `mdkernel_core` modülü |
| R8 | Obsidian paketi (mağaza başvurusu yok) | `dist/obsidian/mdkernel-0.1.0.zip` içinde yalnız `main.js`, `manifest.json`, `styles.css`; `main.js` < 500 KB; `package.json`'da çalışma zamanı bağımlılığı yok |
| R9 | Spec yayım dizini | `dist/spec-<ver>/` içinde `SPEC.md`, `CONFORMANCE.md`, `LICENSE`, `fixtures/MANIFEST.json`, `conformance.json` (level 3, 34/34, manifest ok) |
| R10 | Lisans | kökte `LICENSE` Apache-2.0 metni (`Apache License` / `Version 2.0, January 2004`), arşivlerde de |
| R11 | Belgeler | `CHANGELOG.md`'de `## 0.1.0`, `docs/THREAT_MODEL.md`, bu dosya |
| R12 | cargo-binstall ve Homebrew taslağı | `server/Cargo.toml [package.metadata.binstall]`; `dist/homebrew/mdk.rb` sürüm ve sha256 dolu |
| R13 | CI çapraz derleme | `.github/workflows/release.yml` dört hedefi içerir |
| R14 | Etiket yok | `git tag -l v0.1.0` boş: etiketi bu tur atmaz |

## İnsan adımları (betik yapmaz)
1. P1-E bağımsız doğrulaması (`reports/rounds/P1-E/VERIFICATION.md`) **"Güvenilir"** demeden hiçbiri yapılmaz.
2. `server/Cargo.toml`'a `repository` alanını yaz (binstall `{ repo }` ve Homebrew URL'si için).
3. `.github/workflows/release.yml`'ı elle tetikle (workflow_dispatch); dört işin sonucunu `dist/release.json` ile karşılaştır (macOS x86_64 işi K-E5-T kararına kadar düşer).
4. `git tag -a v0.1.0 -m "mdk 0.1.0"` ve itme; GitHub release'e arşivleri, wheel'i, eklenti zip'ini ve `SHA256SUMS`'ı ekle.
5. `dist/homebrew/mdk.rb`'yi tap'e koy; `cargo binstall mdkernel-server` ile dene.
6. Obsidian community store başvurusu bu yayımın kapsamında değildir (GOAL P1-E).
