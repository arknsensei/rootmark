# P1-B.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-B.1/F<n>` anar). Eski satırlar silinmez ve değişmez.

## F1 · B1.0 · `scripts/check_frozen.sh` — noktalı tur kimliği
- **Ne:** yeniden dondurma satırının notundaki istisna atfı kalıbı `[A-Z0-9]+-[A-Z0-9]+/F[0-9]+` → `[A-Z0-9]+-[A-Z0-9]+(\.[0-9]+)?/F[0-9]+`. Başka değişiklik yok (`git diff p1b-final -- scripts/check_frozen.sh`: 1 satır).
- **Neden:** bu turun kimliği `P1-B.1`; eski kalıp `P1-B.1/F<n>` atfını hiç eşleştirmez (`P1-B` sonrası `.`), dolayısıyla bu turun her yeniden dondurma satırı (B1.2 `spec/fixtures/**`, `conformance.rs`; B1.3 `check_v3_compat.sh`) "cites no freeze exception" ihlali verirdi.
- **Önce/sonra bit eşitliği:** `p1b-final` tablosuyla (değişiklikten önce, aynı ağaç) eski ve yeni denetleyicinin çıktısı satır satır aynı, tek fark denetleyicinin kendi satırı (değişmiş dosya → bu girdinin yeniden dondurma satırıyla kapanır). Eski atıfların hepsi (`P1-B/F1`…`P1-B/F8`) yeni kalıpla aynı dizgiye eşleşir (`(\.[0-9]+)?` boş). Kayıt: B1.0.md.
- **Gözden geçirici onayı:** B1.0.review.md.

## F2 · B1.2 · `spec/fixtures/**` — C2-07 beklenen değeri (K-B1.2)
- **Ne:** `spec/fixtures/C2-07/expected.json` (iki `trace_id` ve `traces` sonucundaki aynı iki değer) ve `spec/fixtures/MANIFEST.json`'da yalnız `C2-07/expected.json` satırı (`ead96c1e…` → `49b9bf5b…`). Üretim: `bash scripts/gen_fixtures_c2.sh` (C2 mevcut çekirdekten, C1 + manifest dondurulmuş v3-final üretecinden).
- **Neden:** K-B1.2 (spec 1.0.1 §8 ayırıcılı `trace_id`); GOAL B1.2 "C2-07 beklenen değeri yeniden üretim, `MANIFEST.json`". C1 fixture'ları değişmez.
- **Önce/sonra bit eşitliği:** `git diff --stat p1b-final -- spec/fixtures` → yalnız bu iki dosya (4+/4−); C1 `expected.json`'ları v3-final üreteciyle bayt eşit yeniden üretildi; eski C2-07 değerleri eski formülle, yeni değerler yeni formülle Python `blake3` ile bağımsız hesaplandı (`H(cid+"claude-code"+qh+s)` → `052ae79a…`/`8e2df27f…` = eski; `H(cid+"\n"+"claude-code"+"\n"+qh+"\n"+s)` → `17536b4d…`/`f4ee3c57…` = yeni). Kayıt: B1.2.md.
- **Gözden geçirici onayı:** B1.2.review.md.

## F3 · B1.2 · `core/tests/conformance.rs` — `spec_version` 1.0.1
- **Ne:** rapor alanı `"spec_version": "1.0"` → `"1.0.1"` (tek satır).
- **Neden:** GOAL B1.2 "`conformance.json` `spec_version 1.0.1`".
- **Önce/sonra bit eşitliği:** P1-B resmi RUN'ının `conformance.json`'ı ile yeni rapor: C2-07 dışındaki 24 girdinin `{id,result,actual_hash,expected_hash}` JSON'u bayt eşit (sha256 `0d929657…`; Seviye 1 görünümü `27ba0ed8…` iki tarafta aynı); `tests`, `fixtures_root`, `fixtures_manifest_hash`, `spec_version` dışındaki tüm alanlar eşit; 25/25, `level_claimed 2`.
- **Gözden geçirici onayı:** B1.2.review.md.

## F4 · B1.3 · `scripts/check_v3_compat.sh` — karışık yazarlı v3-final vault'ları (B1-1)
- **Ne:** P1-B bölümünden **sonra** yeni bir bölüm: `scripts/v3final-mixed-gen/scenarios.json`'daki 9 senaryo v3-final kütüphane API'siyle **o anda** yeniden üretilir (`scripts/gen_v3final_mixed.sh <temp>`; commit'lenen fixture'lar kullanılmaz), her biri mevcut `mdk` ile açılır: her doküman için `replay` `snapshot_id` v3-final `mdk replay` ile eşit, `events_hash` ve `events.jsonl` baytları değişmez, `blame` `[author_kind, layer, layer_source]` satırları `expect_blame` ile eşit. Sonuç isteğe bağlı ikinci argümana (`<mixed-out.json>`) yazılır; herhangi bir senaryo kalırsa çıkış 1. Başlık yorumu ve `mixed_out` değişkeni dışında P1-B bölümünün satırları aynen (`git diff p1b-final -- scripts/check_v3_compat.sh`: yalnız eklenen satırlar). `Makefile` `p1b-checks` ikinci çıktıyı RUN'a `v3_compat_mixed.json` olarak yazar.
- **Neden:** GOAL B1.3; P1-B doğrulamasının 1. önerisi ("`check_v3_compat.sh`'e v3-final API'siyle üretilmiş Human→Agent, Human→Transcript ve Synthetic→Agent senaryoları eklenmeli … F-istisnası ile").
- **Önce/sonra bit eşitliği (eski senaryo):** aynı ağaçta değişiklikten önceki ve sonraki betik: stdout ve `<out.json>` `cmp` ile **bayt eşit**, ayrıca P1-B resmi RUN'ının `v3_compat.json`'ı ile bayt eşit (12 doküman, `events_hash` `f3cb01242e8cb7123e417a72a196250f`, `pass true`); stderr'in eski satırı (`check_v3_compat: OK (12 documents)`) aynı, yeni bölüm kendi satırını ekler. Bu `p1b_checks.json` B-5 alanının değişmemesini sağlar (B1-6).
- **Ayırt edicilik:** aynı yeni betik `p1b-final` çekirdeğiyle (worktree) → çıkış 1, karışık 0/9: Human→Agent, Human→Transcript, Synthetic→Agent ve karışık vault P1-B doğrulamasındaki F1 hatasını (`snapshot committed before LayerChanged`) verir; mevcut çekirdekle 9/9.
- **Gözden geçirici onayı:** B1.3.review.md.
