# P1-C.1 — Bağımsız Doğrulama

- Yönerge: `docs/rounds/P1-C.1/VERIFY.md`, `docs/DEV_PROCESS.md` §5.
- Doğrulanan commit: `4856262` (READY; ölçülen commit `b9f7622`). `b9f7622..4856262` arasında kod/araştırma/fixture farkı yok: yalnız `docs/`, `reports/` ve `spec/SPEC.md` durum damgası (2 satır: §9 Draft → Validated; SPEC_EDITS S6).
- Ortam: temiz klon (oturum scratchpad'i, `git clone` ana depodan), macOS (Darwin 27.0.0), tantivy `=0.26.2`, model `minilm-l6-v2/e73168f9` (fetch + manifest doğrulandı).
- Kendi RUN'larım: A `20261015T104848Z-4856262`, B `20261015T110914Z-4856262` (`scripts/verify.sh pair <klon> A B`).
- Süreç notu (şeffaflık): klonu kurmadan önce ana dizinden yalnız `VERIFY.md`, `GOAL.md`, `READY.md` ve `DEV_PROCESS.md` §5 okundu (ağaç temizdi, `4856262`'deki içerikle aynı). Bundan sonra yapılan bütün okumalar ve koşturmalar klonda ve kendi çalışma dizinlerimde yapıldı. Ana dizine yazılan tek dosya bu rapor.

## V1 · Çift koşturma — geçti
- `verify.sh pair` **çıkış 0**: zincir A 1 240 s, zincir B 1 245 s; her iki manifest'te `worktree_clean` false olan faz yok.
- `pair_compare.json` (`compare_p1c1 pair`): **99 341 alan, `equal: true`, `differing_excluded: {}`, `bytes_equal: true`**.
- `index/` **istisnasız** `diff -r`: `A/p1c_work/<RUN>/.mdkernel/index` ↔ `B/…/index` **eşit**. Her iki tarafta 18 dosya (gizli `.managed.json` dahil), `*.lock` yok. Ayrıca determinizm vault'unun **tamamı** (`.mdkernel/` + vault dosyaları) `diff -r` ile eşit.
- Ek: kendi RUN A'm ile commit'lenmiş resmi RUN `20261015T094412Z-b9f7622` arasında `compare_p1c1 pair` → 99 341 alan eşit, baytlar eşit.
- Kasıtlı bozmalar (zincir içinde): `test_tamper` 12/12, P1-C 6/6, `test_p1c1.sh` 45/45.
- Kendi bağımsız kontrolüm: kendi vault'umda indeksi iki kez ayrı ayrı kurdum, `diff -r` eşit. tantivy segment adları rastgele değil.

## V2 · Varyant kanıtı (C1-1) — geçti (bir gözlemle)
- **Formül okuması.** `research/eval/bm25_lucene.py`: `idf = ln(1 + (N − n + 0.5)/(n + 0.5))`, `k1 1.2`, `b 0.75`, `score = Σ idf·tf·(k1+1)/(tf + k1(1 − b + b·dl/avgdl))`, `dl` = token sayısı, f64. tantivy 0.26.2 kaynağı (`src/query/bm25.rs`) aynı idf'i, aynı `K1`/`B`'yi ve `weight = idf·(1+K1)`, `tf/(tf + K1(1 − B + B·fieldnorm/avg))`'yi kullanır. **İki fark var:** tantivy'de (i) `fieldnorm` 256 kovalı kayıplı kodlamadır (`fieldnorm/code.rs`, 40 tokenın üstü yuvarlanır), (ii) `Score = f32`'dir. Rust indeksi (`index/src/search.rs::bm25_with`) skoru tantivy'nin skorlayıcısıyla değil, kendisi hesaplar: tam `lexical_len`, f64. Python ile Rust aynı formülü uygular; ikisi de tantivy'nin gerçek skorlayıcısı değildir. Bu FINAL'de "fieldnorm'suz" diye belirtilmiş.
- **Koşturma.** `run_c1_variant` zincirde koştu (A ve B, `c1_variant.json` A↔B eşit). Δ'ları `c1_variant.json` ve P1-C `c3_quality.json`'dan kendim hesapladım:

| Hücre | Δ nDCG@10 | Δ recall@10 | Δ MRR | sorgu düzeyinde eşit |
|---|---|---|---|---|
| hybrid/structural/single | 0 | 0 | 0 | 297/297 |
| hybrid/structural/multi | 0 | 0 | 0 | 297/297 |
| flat/structural/single | 0 | 0 | 0 | 297/297 |
| flat/structural/multi | 0 | 0 | 0 | 297/297 |

- **`rank_bm25` ile fark = P1-C Δ'sı.** lucene − okapi nDCG@10: hybrid multi **−0.013603** (P1-C `c3_quality.json` `delta` −0.013603101688; GOAL'deki −0.0136), hybrid single +0.002898 (P1-C +0.002898), flat 0/0. Birebir eşit.
- **Kasıtlı bozma (benim).** `evaluate()` bellekte, hiçbir dosya yazmadan:
  - `k1 = 1.5` → C1-1 **kaldı** (hybrid single Δ nDCG −0.0091, MRR −0.0105; multi +0.0042). Ölçüt duyarlı.
  - tantivy'ye sadık varyant (fieldnorm kodlaması + f32; benim yeniden uygulamam) → C1-1 **kaldı** (hybrid multi Δ MRR +0.0045, recall@10 +0.0040; single nDCG −0.0011; hybrid sorgu eşitliği 277/297).
- **Gözlem Ş1:** K-C1.1'in beş manifest alanı (`variant, idf, k1, b, tokenizer`) skoru yeniden üretmeye yetmiyor. Belge uzunluğu normalizasyonu (tam token sayısı, kodlamasız) kaydedilmiyor. `lucene-bm25` adı, tantivy'nin kendi BM25'i ile 0.001 eşiği içinde aynı sonucu vermiyor. GOAL'ün yazılı ölçütü (K-C1.1 formülü) karşılanıyor, yani ihlal değil. Ama "bildirimsel skorlama" iddiası eksik bildirilmiş.
- **Gözlem Ş2:** 4 hücreden 2'si (`flat`) BM25 içermez, varyanttan bağımsız olarak geçer. C1-1'in asıl kanıtı iki `hybrid` hücresi. O ikisi de sorgu düzeyinde 297/297 tam eşit.
- Not: `bm25_lucene.py` "Rust kodu okunmadı" diyor. FINAL ise işlem sırasının Rust'ınkine bilerek eşitlendiğini söylüyor (C1.1 gözden geçirme notu 1). Bu, kanıtın bağımsızlığını biraz zayıflatıyor. Sorgu düzeyinde tam eşitlik ve iki bozmanın da kalması sonucu yine taşıyor.

## V3 · `include_proposal` (C1-2/3) — geçti
Kendi vault'um: depo belgeleri (`spec/`, `docs/*.md`, `NORTH_STAR.md`; 10 belge) insan içeriği. Ayrıca `propose` ile 12 ajan yazımı: 3 birebir ikiz kopya (`agent/twin_*`), mevcut 8 belgenin paragraf eklenmiş sürümleri, sorguyla aynı cümleyi taşıyan 1 belge. Checkpoint `t0`. 100 sorgu (başlık ve cümle parçalarından deterministik seçim + 5 elle yazılmış sorgu; biri hiç eşleşmeyen tokenlı) × 3 mod. Her durumda `k=10` ve `k=200`, bayraklı ve bayraksız.
- canonical önek bayraksız sonuca bayt eşit: **300/300**.
- öneki izleyen her hit `layer: proposal` ve sayısı ≤ k: **300/300**. Hiçbir canonical bir proposal'dan sonra gelmiyor.
- `k=200`: canonical küme ve sıra bayrakla aynı: **300/300**.
- ×0.5: 9 388 proposal hit. İkiz kopyalarda (aynı chunk, aynı istatistik) `flat`/`lexical` proposal skoru = 0.5 × canonical ikizin skoru, **tam eşitlik 2 440/2 440**. `hybrid`'de 2 × skor, ulaşılabilir bir RRF toplamına (`1/(60+a)` [+ `1/(60+b)`], a, b ≤ 50) 1e-12 içinde eşit: **3 800/3 800**.

## V4 · Salt okunur (C1-4) — geçti
Taze kurulmuş indeks, işaret dosyası, ardından 50 `search` (3 mod, yarısı `--include-proposal`, 10'u `--at` olmadan; 50 iz satırı yazıldı). `find index -newer marker` **boş**, `*.lock` **yok** (`.tantivy-meta.lock` dahil), kopyayla `diff -r` **eşit**.

## V5 · Boş sorgu (C1-5) — geçti
`""`, `"   "`, `"\n"` × 3 mod = 9 çağrı. Hepsi çıkış 1, `error: empty query: a query must hold a non-whitespace character (spec §9)`. `traces.jsonl` 6 → 6 satır. `events.jsonl` + `traces.jsonl` sha256 değişmedi.

## V6 · Manifest (C1-6) — geçti
`manifest.json` → `lexical = {variant "lucene-bm25", idf "ln(1 + (N - n + 0.5) / (n + 0.5))", k1 1.2, b 0.75, tokenizer "mdk_lower_alnum"}`, `format mdkernel-index/2`. 12 bozmanın hepsinde `index status` çıkış 1:
- beş alandan her birinin silinmesi
- `lexical`'ın silinmesi
- `variant`, `b`, `idf` ve `tokenizer` değerlerinin değiştirilmesi
- bilinmeyen alan eklenmesi
- `k1`'in dizgi yazılması

Geri yüklenen manifest açılıyor.

## V7 · Uyumluluk — geçti
- `make conformance` (klonda): **level 3, 31/31**, C3-05 `pass`, `fixtures_manifest ok`, `spec_version "1.0.2"`.
- **Sürüm, CRITERION kuralıyla:** CRITERION iki düzey tanır. Yama: "hesaplanan değeri değişmeyen ya da yalnız Validated/Draft bölümleri etkileyen". Majör: §1–6 ya da kırıcı. Minör düzey yoktur. S1/S2 (b)'yi sağlamaz, yani anlam değişikliğidir: karar + sürüm artışı gerekir. Yalnız §9'u etkilerler ve §9 tur başında Draft'tı. Sonuç: **1.0.1 → 1.0.2 kurala uygun**. Emsal: P1-B.1'de (b)'yi sağlamayan §7/§8 düzenlemeleri 1.0.0 → 1.0.1 yapılmıştı. GOAL `1.1.0` öngörüyordu. PROMPT açıkça "CRITERION kazanır" dediği için bu bir ihlal değil, gerekçesi SPEC_EDITS.md'de (bkz. Ö2).
- **C3-0x değişiklikleri, SPEC_EDITS ile birebir.** `p1c-final` ↔ HEAD `expected.json`'ları işlem işlem karşılaştırıldı:
  - Değişen işlemler tam olarak tablodakiler: C3-01 op5; C3-02 op5; C3-03 op4/5/6/11/12/13/15/16/17/18/19/20; C3-04 op8/10/12. Hepsi `include_proposal: true`, değişen tek alan `rankings`. Bayrak kapalı hiçbir gözlem değişmemiş.
  - proposal sayıları tablodakiyle aynı: 3 → 7 (lexical), 3 → 30 (hybrid/flat), C3-04'te 2 → 4/20.
  - neden (ii), canonical kısım uzamış: **11 sorgu**. Neden (iii), canonical sıra değişmiş: **11 sorgu**; dağılım C3-02 op5 1, C3-03 op6/13/17 1'er, op19 2, op20 2, C3-04 op10 1, op12 2. Tabloyla aynı.
  - Bayraklı sıralamanın canonical öneki bayraksız ikize eşit: **50/50**. Yeni listelerde proposal'lar hep canonical'lardan sonra geliyor.
- **Kendim yeniden ürettim:** `scripts/gen_fixtures_c3.sh` (C3 + `gen_fixtures.sh` ile bütün C1 ve MANIFEST), ardından `git status` **temiz**.
- Test değişiklikleri (`p1c-final..HEAD`) incelendi. `core/tests/conformance.rs` yalnız `spec_version` (F1). `eval_c3.rs` yalnız yeni `canonical_prefix_of` gözlemi (F3). `LEVEL3`/gen sayıları (F4/F5). `hybrid.rs::proposal_weight_and_order_by_hand` yeni `finalize` anlamına (katman başına k, K-C1.3 / ASSUMPTIONS C1.2-1) göre yeniden yazılmış ve ×0.5 ile sıralama iddialarını koruyor. Testi geçirmek için zayıflatılmış bir doğrulama görmedim.

## V8 · Referans (C1-9) — geçti
- `research/FROZEN.md` satır 106: `research/data/quality_ref_rust.json` (`P1-C.1/C1.4-pre:` commit `c337cf7`). `check_frozen.sh` OK.
- P1-C resmi `c3_quality.json` ile alan alan: `cells` 24/24 yaprak eşit. `per_query` 4 158/4 158 yaprak eşit. Referansın atladıkları yalnız `trace_id` (2 × 297 = 594 yaprak, kalite alanı değil) ve `index`'in kurulum sayaçları (`embedded`, `reuse`). `checkpoint_id`, `n_questions`, `records`/`snapshots`/`embedder_id` eşit.
- Zincir içinde kendi RUN'larımda C1-9 `pass` (A ve B).

## V9 · Dondurma, bağımlılık, karar kuralı — geçti
- `check_frozen.sh` OK (tablo P1-C ve P1-C.1 `ROUND_START`'tan beri yalnız ekleme). `check_deps.sh` OK. `test_check_deps.sh` OK (ters bağımlılık yakalanıyor). Dondurma istisnaları F1–F8 `FREEZE_EXCEPTIONS.md`'de, her biri bir FROZEN satırıyla.
- K-C1.6 hariç tutma listesi `6623ff1`'de (08:40Z) yazılmış, resmi RUN 09:44Z'de. Yani ölçümden önce. 10 hariç alanın hepsi K-C1.6'nın (1)–(6) maddelerinden. `c3_quality.json` hariç tutulmamış.
- **Kuralı GOAL'den türettim:**
  - C1-1 kaldı → varyant açıklaması yanlış; P1-D D3 başlamaz; karar talebi.
  - C1-1 geçti ∧ C1-2..C1-9 geçti → K-C8-T kapanır, §9 Validated, 0.1.0 yayım engeli kalkar, P1-D tam başlar.
  - Benim RUN'larımda (A, B) C1-1..C1-9 hepsi `true`, yani ikinci dal. **FINAL ile aynı.** `spec/SPEC.md` damgası (S6) buna uygun.
- Kuraldaki boşluk: "C1-1 geçti ∧ başka bir kriter kaldı" durumu GOAL'de tanımlı değil. Bu turda gerçekleşmedi.

## Sonuç

**İhlal:** yok.

**Fark:** kendi çift koşturmamda ve resmi RUN'la karşılaştırmada 0. `index/` istisnasız eşit.

**Şüphe / gözlem:**
- Ş1: `lucene-bm25` bildirimi skoru yeniden üretmeye yetmiyor. Uzunluk normalizasyonu kaydedilmiyor, tantivy'ye sadık uygulama C1-1'i 0.0045'e kadar kaçırıyor.
- Ş2: C1-1'in `flat` hücreleri varyanttan bağımsız.
- Diğer: `include_proposal` yanıtı 2k'ya kadar hit dönebiliyor (ASSUMPTIONS C1.2-1; spec metni "sona eklenir" diyor, uzunluk sınırını normatif olarak koymuyor). Spec sürümü GOAL'deki 1.1.0 yerine CRITERION kuralıyla 1.0.2.

**Hüküm: Güvenilir.** C1-1..C1-9 bağımsız olarak yeniden üretildi. Karar kuralının sonucu (K-C8-T kapanır, §9 Validated, 0.1.0 yayım engeli kalkar, P1-D tam başlar) doğru.

**Öneriler (≤ 3):**
1. **Ö1 (Ş1):** `lexical` manifest'ine ve spec §9 bilgilendirici notuna belge uzunluğu normalizasyonunu ekleyin (ör. `dl: "token count, exact (no fieldnorm quantization)"`, hesap f64). Ya da varyant adını bunu yansıtacak biçimde değiştirin. Gerekirse bunu bir K kararıyla yapın; bildirim, skoru başka bir uygulamanın yeniden üretebileceği kadar tam olmalı.
2. **Ö2:** CRITERION ile tur şablonlarını hizalayın. Ya CRITERION'a minör düzey ekleyin ya da GOAL'lerde sürüm sayısı öngörmeyin. Karar kurallarına "ana kriter geçti ∧ başka bir kriter kaldı" dalını ekleyin.
3. **Ö3:** P1-D D3 (search aracı) başlamadan önce `include_proposal` yanıt uzunluğunu (katman başına ≤ k, toplam ≤ 2k) spec §9'da normatif yazın ve istemci sözleşmesine koyun. Şu an yalnız ASSUMPTIONS'ta duruyor.
