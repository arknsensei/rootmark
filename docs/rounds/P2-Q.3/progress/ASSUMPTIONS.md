# P2-Q.3 — Varsayımlar

- **[-] A1 (R0):** GOAL/TASKS'in "gerekirse `server/src/main.rs`"i bu depoda `server/src/bin/rootmark.rs`'tir (CLI'ın tek giriş dosyası; `ingest-dir` orada). Ürün sınırı (Q3-6) bu dosyayı `main.rs` yerine okur.
- **[arch] A2 (R0):** "Alt dizin kökü" (`ingest-dir V V/sub`) için kaldırma kapsamı vault'a göreli yoldur (`sub/…`); çekirdeğin alımı `doc_path`'i `dir`'e göre yazmayı sürdürür (çekirdek değişmez). `dir` vault'un dışındaysa kaldırma yapılmaz (DESIGN §4).
- **[-] A3 (R0):** Uzlaştırma yalnız `--watch`'ta koşar; `--watch`'sız `serve --mcp stdio` dosya sistemini gözlemlemez (bugünkü gibi). Claude Code için kılavuzun önerdiği kurulum `--watch`'lıdır; kılavuz bunu R4'te açıkça yazar.
- **[-] A4 (R0):** Turun açılış PR'ı #64 tur başlarken birleşmemişti; kullanıcı bu oturumda `--admin` birleştirmeyi onayladı ("Evet, --admin birleştir"), `ROUND_START` birleşmiş `main` (`325d8b9`).
