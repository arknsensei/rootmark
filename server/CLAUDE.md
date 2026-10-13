# server — ajan notları
Sorumluluk: `mdk` CLI, MCP sunucusu (`rmcp`), HTTP API, dosya izleyici, politika tarif dosyası yükleyici (docs/PACKAGES.md `server`).
Bağımlılık yönü: core ← index ← server ← clients; `server` yalnız `core` ve `index`'e bağlıdır. Ters yön yasak.
Dil ve araçlar: Rust 1.92.0.
Dondurulmuş yüzeyler: yok (P1-A'da boş crate; `mdk` CLI şimdilik `core/src/bin/mdk.rs`'de, P1-D'de buraya taşınır).
Test komutları: `cargo test -p mdkernel-server --release`.
Bu pakette yapılmayacaklar: P1-A'da MCP/HTTP kodu; çekirdek mantığı kopyalamak.
