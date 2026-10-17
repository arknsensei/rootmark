# server — ajan notları
Sorumluluk: `rootmark` CLI, MCP sunucusu (`rmcp`), HTTP API, dosya izleyici, politika tarif dosyası yükleyici (docs/PACKAGES.md `server`).
Bağımlılık yönü: core ← index ← server ← clients; `server` yalnız `core` ve `index`'e bağlıdır. Ters yön yasak (`scripts/check_deps.sh`).
Dil ve araçlar: Rust 1.92.0; `rmcp =3.5.0` (resmi MCP Rust SDK, stdio), `axum =0.8.9`, `tokio =1.53.1`, `notify =8.2.0`, `schemars =1.2.1` (doğrudan bağımlılıklar ≥ 14 gün önceki sürümlere sabit, TASKS D0).
Dondurulmuş yüzeyler: `rootmark` alt komutları ve JSON çıktıları (P1-D/D0'da `core`'dan taşındı, bayt eşit); `rootmark index …`/`rootmark search …` `rootmarkindex::cli`'dir (ikinci süreç yok). spec §7–§10 davranışı çekirdekte ve indekstedir; sunucu yalnız dışarı açar.
Test komutları: `cargo test -p rootmark-server --release` (CLI testleri `server/tests/cli_*.rs`; indeks testleri model dosyalarını ister, `make models`).
Bu pakette yapılmayacaklar: çekirdek/indeks mantığını kopyalamak; `propose` dışında markdown yazan yol; markdown'a kimlik/meta veri; `promote` varsayılan politikasını gevşetmek; 127.0.0.1 dışına bağlanma; istemci kodu; yeni spec bölümü.
