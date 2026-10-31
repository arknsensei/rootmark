# P2-V.1 — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda E1 sonunda ("kalite ve determinizm iddiası") ve E5'te ("ölçülen şey gevşedi mi").

## F1 · E1 · `index/tests/embed.rs` — `embedder_id` kuralının iddiası
- **Ne:** `embedder_id_is_derived_from_the_manifest`'in ilk `assert_eq!`'i: beklenen kimlik `"minilm-l6-v2/" + blake3[:8]` → `"minilm-l6-v2/" + blake3[:8] + "+fixed1"` (etiket sabit yazılı; kütüphanenin `NUMERICS` sabitinden alınmaz, gözden geçirici koşulu 2). Dosyanın öteki satırları (C-10 eşiği 0.999 ve kaydı, 100 tekrar bayt eşitliği, toplu ≡ tek tek, birim uzunluk, manifest uyuşmazlıkları) aynen (`E1_f1.diff`: yalnız bu iddia).
- **Neden:** GOAL P2-V.1 Kapsam 2: "embedder kimliği ya da indeks biçim sürümü değişir"; seçilen kimlik (ASSUMPTIONS E1-1). Test kuralın kendisini sınar; kural değişince iddia yeni kuralı söyler. Biçim sürümünü değiştirmek beş dondurulmuş ölçüm dosyasını değiştirmeyi gerektirirdi.
- **Önce/sonra:** önce (`ROUND_START`): kimlik `minilm-l6-v2/e73168f9`, test o kuralla geçer; sonra: kimlik `minilm-l6-v2/e73168f9+fixed1`, test yeni kuralla geçer ve `m.embedder_id()` ile eşitlik iddiası (ikinci satır) değişmeden geçer. C-10 testi değişmeden geçer (`min_cosine` 0.9999999999992675 ≥ 0.999; E1'in RUN'ı, `reports/rounds/P2-V.1/e1/c10_embedding.json`). `cargo test -p rootmark-index --release --test embed` 6/6.
- **Gözden geçirici onayı:** `E1.review.md`.
