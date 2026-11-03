# Q1 (c) — README'nin komutları (aday ikiliyle; Q2'de yayımlanmışla yeniden)

`release/0.3.0` (`49fd5cd`) README'si; aday ikili `rootmark 0.3.0` (yerel derleme), `spec/fixtures/mini_vault` kopyası.

| Komut (README) | Sonuç (aday) | Not |
|---|---|---|
| `rootmark ingest-dir vault vault` | çıkış 0 | sahne bölümü |
| `rootmark serve --vault vault --mcp stdio --http 127.0.0.1:0 --watch` | çıkış 0; `http listening on 127.0.0.1:<port>`, MCP akışı kapanınca `stopped` | sahne bölümü |
| `rootmark --version` / `--spec-version` | `rootmark 0.3.0` / `2.0.3` | kurulum bloğu |
| `rootmark conformance >/dev/null` | `rootmark conformance: level 3, 39/39 (spec 2.0.3)` | kurulum bloğu |
| `curl … install.sh`, `curl … SHA256SUMS`, `grep … \| shasum -a 256 -c`, `bash install.sh --version 0.3.0`, `export …` | **Q2'de** (sürüm yayımlanınca) | kurulum bloğu; kılavuz §1.1 ile aynı |
| `gh attestation verify <file> -R arknsensei/rootmark` | **Q2'de** | |
| `cargo build --release -p rootmark-server --bin rootmark`, `make models` | çıkış 0 (yayım dalının derlemesi; modeller yerel) | kaynaktan |
| `cargo binstall rootmark-server` | koşulmadı: crate yayımlanmadı, README "once the crate is published" der | |
| `rootmark migrate <vault>`, `--dry-run` | çıkış 0 | |
| `rootmark migrate --rename-store <vault>` | yeni bir vault'ta çıkış 1, `migration rename_store does not apply to vault` — beklenen: README onu yalnız "a vault from before the project's rename" için söyler | |

Eskimiş iddiaların düzeltilmesi: `rootmark status` (CLI'da yok) → "the HTTP/MCP `status`"; sürüm 0.2.0 → 0.3.0; spec rozeti 2.0.0 → 2.0.3, `--spec-version` 2.0.2 → 2.0.3; Obsidian bölümü BRAT'tan elle kuruluma ve kılavuza; Türkçe vault'ta `make models-multilingual` (depo komutu, kurulu kullanıcıda yok) → kılavuz §1.2; "Running rootmark" bağlantısı üstte.
