# P2-L — Görevler

Sıra: L0 → L1 → L2 → L3 → L4 → L5. Progress `docs/rounds/P2-L/progress/`. Commit öneki `P2-L/L<n>:`. Dal `p2-l/round` (yalnız birleşmiş `main`'den, K-P2.2). Gözden geçirici: tur sonunda tek sefer (L5), görev başına değil (K-P2.4); güvenlik incelemesi yalnız L3 (filtre ile yol dışına sızma) ve L0 (b).

## L0 — Açılış, kapanış kalıntıları, hızlı kip, 0.2.0
**Yapar:** `ROUND_START`; DECISIONS tur açılışı; GOAL 1 (a)–(d). **Yayım (e) ayrı dalda:** `release/0.2.0` dalı `main`'den: sürüm dizgileri, CHANGELOG, RELEASE.md; PR aç (`gh pr create --base main --head release/0.2.0 --title "release: 0.2.0"`); **birleşmesini bekle** (`gh pr view --json state` 60 s aralıkla, en çok 2 saat; birleşmezse STOP.md'ye yaz ve L1'e geç, yayımı L5'e bırak); birleşince `git fetch && git tag -a v0.2.0 -m "rootmark 0.2.0" <main HEAD> && git push origin v0.2.0`, `gh workflow run release.yml --ref v0.2.0`, koşu bitince artefaktları indir, `gh attestation verify`, `gh release create v0.2.0` (notlar CHANGELOG'dan; arşivler, wheel, eklenti dosyaları, `install.sh`/`install.ps1`, `SHA256SUMS`), `release_check.sh`; kayıt `reports/rounds/P2-L/release_0_2_0.json`. Sonra `p2-l/round`'u `origin/main`'e yeniden tabanla.
**Kabul:** L-11; `check_frozen`, `check_deps`; `make verify FAST=1` ≤ 20 dk ve temiz.

## L1 — `uni_fold_tr`
**Yapar:** `index/src/tokenize.rs` ikinci tokenizer (Unicode `Alphabetic|Numeric` dizileri, NFC, Türkçe katlama: `İ→i`, `I→ı`, sonra `to_lowercase`), tantivy'e kayıt, manifest `lexical.tokenizer`, `[index] tokenizer = "mdk_lower_alnum" | "uni_fold_tr"`; `index_rebuild` göçü tokenizer değişiminde tetiklenir; `status.index.format`/`expected_format` tokenizer'ı içerir.
**Kabul:** L-1; eski varyant bayt bayt aynı (C2 200 chunk referansı); eski indeks → `migration_required` (L-8'in birimi).

## L2 — Çok dilli embedder
**Yapar:** `index/src/embed.rs` ikinci model kaydı (ONNX, sabit sha256+blake3, `scripts/fetch_model.sh` + `make models`), `embedder_id` manifest'te; `[index] embedder`; önbellek anahtarı model kimliğini içerir (zaten); C-10 eşdeğeri birim test (kesin kosinüs, iki koşuda aynı vektör).
**Kabul:** model manifest'i; birim testler; varsayılan değişmedi.

## L3 — Yol öneki filtresi
**Yapar:** `search.path_prefix: [string]` (MCP + HTTP); normalize (NFC, `/` ayracı, `..` ret → `invalid_arguments`), önek eşleşmesi doküman yolunda; sıralamadan önce; `query_hash = H(... + "\n" + filtre)` yalnız filtre varsa (filtresiz hash değişmez); `trace` kaydı `filter` alanı; CLIENT_CONTRACT §2 ve spec §8 bilgilendirici not (CRITERION). Güvenlik: önek vault dışını adresleyemez; filtre `include_proposal`'ı etkilemez.
**Kabul:** L-7; `contract_check.sh`; güvenlik incelemesi.

## L4 — Türkçe ölçüm
**Yapar:** `research/eval/datasets/d_tr.py` (kaynak indirme sabit revizyon, sha256, lisans notu; D-TR ≥ 300 TR + ≥ 100 TR/EN), sorgu üreteci (D3 yöntemi, sabit tohum; LLM yok), `run_p2l.py` (12 hücre: 3 kip × 2 tokenizer × 2 embedder), İngilizce referans Δ (L-3, L-4), `report_p2l.py`; sonuçlar `p2l_checks.json`.
**Kabul:** L-2..L-6 ölçüldü; veri ve tohum ön-kayıtlı (ölçümden önce commit); `make verify FAST=1` adımı.

## L5 — Zincir, rapor, hazır
**Yapar:** `verify.sh`: P2-L adımı (L-8 üç vault, L-9 tam çift bir kez, L-10), `compare_p2l` belirlenimli alanlar; gözden geçirici (tek tur, bütün kabul ölçütleri); FINAL (kriter tablosu, 12 hücre tablosu, varsayılan tokenizer kararı, yayım kaydı); READY; tek PR.
**Kabul:** L-9, L-10; izlenebilirlik; karar kuralı.
