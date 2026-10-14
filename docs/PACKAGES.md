# Paket Haritası ve Dil Kararları

Tek monorepo. Dil kararı bileşen başına nihaidir; "önce X sonra Y" geçişi yoktur. Fazlı olan yalnız plugin çalışma zamanı ve istemcilerin kapsamıdır.

## Dizin yapısı

```
mdkernel/
  NORTH_STAR.md
  CLAUDE.md                 # kök ajan kuralları (DEV_PROCESS'ten türetilir)
  spec/                     # yayımlanan spesifikasyon + uyumluluk testleri ve fixture'ları
  docs/                     # ARCHITECTURE, PACKAGES, ROADMAP, DEV_PROCESS, DECISIONS, rounds/<id>/
  core/                     # Rust crate: mdkernel-core
  index/                    # Rust crate: mdkernel-index
  server/                   # Rust crate: mdkernel-server → `mdk` ikilisi
  bindings/python/          # pyo3 → mdkernel_core wheel
  bindings/node/            # napi-rs → @mdkernel/core (faz 2)
  clients/obsidian/         # TypeScript Obsidian eklentisi
  clients/vscode/           # TypeScript (faz 2)
  research/                 # Python: eval, deneyler; çalışma zamanı dışı
  scripts/                  # verify.sh, conformance.sh, release.sh
  reports/                  # koşturma ve doğrulama çıktıları
```

## Paketler

| Paket | Dil | Sorumluluk | Bağımlılık | Durum | Dağıtım |
|---|---|---|---|---|---|
| `spec` | Markdown + fixture | normatif tanım, uyumluluk testleri | — | v1.0 §1–6 donmuş, §7/§8/§11 doğrulandı (P1-B), §9–10 taslak | yayım (Apache-2.0) |
| `core` | Rust | parser, blok, soy (matcher v3), olay günlüğü, nesne deposu, chunk, katman/terfi, kontrol noktası, tüketim izi, sürüm tetikleyici sınırları, politika hook'ları | `pulldown-cmark`, `blake3`, `strsim`, `serde` | **nihai** | crates.io |
| `index` | Rust | sözcüksel (tantivy), vektör deposu + kesin arama, embedder trait + ONNX varsayılan (`ort`), hibrit RRF, kontrol noktasına göre sorgu, sıcak katman | `core`, `tantivy`, `ort` | **nihai** | crates.io |
| `server` | Rust | `mdk` CLI, MCP sunucusu (`rmcp`), HTTP API, dosya izleyici, politika tarif dosyası yükleyici | `core`, `index` | **nihai** | tek ikili (Linux/macOS/Windows), Homebrew/cargo-binstall |
| `bindings/python` | Rust→Python | `research` ve Python kullanıcıları için `core`+`index` yüzeyi | `core`, `index` | nihai | wheel (PyPI) |
| `bindings/node` | Rust→Node | istemciler için yerel bağlama (MCP/HTTP yetmezse) | `core` | faz 2 | npm |
| `clients/obsidian` | TypeScript | blame paneli, proposal görünümü, promote/demote, checkpoint seçici, trace görüntüleyici; çekirdeğe MCP/HTTP ile | `server` | faz 1 minimal | community plugin |
| `clients/vscode` | TypeScript | aynı yüzey | `server` | faz 2 | marketplace |
| `research` | Python | ölçüm altyapısı (v1–v3 mirası), varlık grafı, akış durumu deneyleri | `bindings/python` | kalıcı, yayımlanmaz | — |

Bağımlılık yönü kesin: `core ← index ← server ← clients`. `clients` yalnız `server`'ın MCP/HTTP sözleşmesine bağlıdır; `research` yalnız bağlamaya. Ters yönde bağımlılık reddedilir (CI kontrolü).

## Plugin sınıfları ve çalışma zamanı

| Sınıf | Faz 1 | Faz 2 |
|---|---|---|
| Karar/politika (terfi, budama, sürüm tetikleyici, sıralama ağırlığı) | bildirimsel tarif dosyası (`policy.toml`), çekirdek sınırları içinde parametre | WASM (`wasmtime`) ile prosedürel politika |
| Pipeline zenginleştirici (chunker, embedder, varlık çıkarıcı, reranker) | yalnız dahili trait implementasyonları (ONNX embedder, structural chunker) | WASM, içerik-adresli önbellek zorunlu |
| İstemci eklentisi | Obsidian | VS Code, diğer |

## v2/v3 prototip kodunun kaderi
`core/` prototipteki `core/` crate'inden başlar (matcher v3 dahil), yeniden yazılmaz; API yüzeyi `spec/` ile hizalanırken temizlenir. `research/` prototipteki `eval/` + `mdkernel/` Python kodunu devralır. Prototip `bindings/` → `bindings/python/`. Prototip `server`/`index` yoktu; sıfırdan.

## Dağıtım artefaktları ve sürümleme
- Tek sürüm numarası (SemVer) tüm Rust crate'leri ve `mdk` ikilisi için; `spec` kendi sürümünü taşır ve `mdk --spec-version` ile bildirilir.
- Her sürüm: uyumluluk raporu (`conformance.json`, seviye), `make verify` çift koşturma çıktısı, değişen karar kayıtları.
- İstemciler kendi sürümleriyle, desteklenen `mdk` sürüm aralığını bildirir.
