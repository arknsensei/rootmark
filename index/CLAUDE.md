# index — ajan notları
Sorumluluk: sözcüksel (tantivy), vektör deposu + kesin arama, embedder trait + ONNX varsayılan, hibrit RRF, kontrol noktasına göre sorgu, sıcak katman (docs/PACKAGES.md `index`).
Bağımlılık yönü: core ← index ← server ← clients; `index` yalnız `core`'a bağlıdır. Ters yön yasak (`scripts/check_deps.sh`).
Dil ve araçlar: Rust 1.92.0; P1-C'de `tantivy`, `ort`.
Dondurulmuş yüzeyler: yok (P1-A'da yalnız trait tanımları, `src/lib.rs`).
Test komutları: `cargo test -p mdkernel-index --release`.
Bu pakette yapılmayacaklar: P1-A'da implementasyon (tantivy/ONNX P1-C); ANN (determinizm garantisi olmadan); graf sıralama (K3).
