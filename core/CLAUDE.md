# core — ajan notları
Sorumluluk: parser, blok, soy (matcher v3; v2 bayrakla), olay günlüğü, nesne deposu, chunk; P1-B'den itibaren katman/terfi, kontrol noktası, tüketim izi (docs/PACKAGES.md `core`).
Bağımlılık yönü: core ← index ← server ← clients; `core` hiçbir iç paketi bilmez. Ters yön yasak (`scripts/check_deps.sh`).
Dil ve araçlar: Rust 1.92.0 (`rust-toolchain.toml`), `Cargo.lock` kilitli; `pulldown-cmark`, `blake3`, `strsim`, `serde`.
Dondurulmuş yüzeyler: spec §1–6 davranışı ve `spec/fixtures/` (C1); `MatcherVersion::V2` ve `V3` çıktıları v3-final ile bayt bayt aynı (mini_vault `events_hash` v2 `8914f9b4…`, v3 `808a41f2…`); `events.jsonl` biçimi; eşleyici sabitleri (V2 0.60/0.70, V3 0.50/0.65).
Test komutları: `cargo test -p mdkernel-core --release`, `make conformance`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
Bu pakette yapılmayacaklar: md dosyasına kimlik/meta veri yazmak; kimliğe zaman damgası veya rastgelelik; sessiz başarısızlık; indeks/sunucu kodu; P1-A'da katman geçişi (yalnız varsayılan katman).
