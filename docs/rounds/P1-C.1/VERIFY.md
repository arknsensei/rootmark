# P1-C.1 — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P1-C.1/VERIFICATION.md.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; kalite alanları fark 0; `index/` `diff -r` **istisnasız** eşit (hiçbir dosya hariç tutulmadan).
- **V2 Varyant kanıtı (C1-1):** `research/eval/bm25_lucene.py` formülünü kendin oku ve tantivy'nin belgelenmiş BM25'iyle karşılaştır; `run_c1_variant.py`'ı koştur; 4 hücre Δ'larını kendin hesapla; ayrıca eski `rank_bm25` ile farkın P1-C'deki Δ'ya (multi −0.0136) eşit olduğunu teyit et.
- **V3 `include_proposal` (C1-2/3):** kendi proposal'lı vault'unda 100 sorgu × 3 mod: canonical öneki bayt eşit; proposal'lar etiketli, ×0.5, sonda; `k=200`'de canonical küme aynı.
- **V4 Salt okunur (C1-4):** taze kurulmuş indekste 50 `search` sonra `find index -newer <marker>` boş; `.tantivy-meta.lock` yok.
- **V5 Boş sorgu (C1-5):** `""`, `"   "`, `"\n"` → `Err`; `traces.jsonl` satır sayısı değişmez.
- **V6 Manifest (C1-6):** `lexical.*` alanları; bir alanı sil → `index status` hata.
- **V7 Uyumluluk:** 31/31 (C3-05 dahil), `spec_version` CRITERION kuralına uygun; C3-0x değişen beklenen değerleri `SPEC_EDITS.md` gerekçeleriyle birebir; kendin yeniden üret, `git status` temiz.
- **V8 Referans (C1-9):** `quality_ref_rust.json` FROZEN'da; P1-C `c3_quality.json` ile alan alan eşit.
- **V9 Dondurma, bağımlılık, karar kuralı:** `check_frozen.sh`, `check_deps.sh`; kuralı GOAL'den türet; FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri.
