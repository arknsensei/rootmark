# P1-C — Bağımsız Doğrulama Raporu

- **Doğrulanan commit:** `44d2bdc` (READY.md commit'i). Klonların HEAD'i bu commit, çalışma ağaçları temiz. READY'nin gösterdiği `2058bda` ve ölçülen kod commit'i `9aa326b` bu commit'in ataları. `9aa326b..44d2bdc` arasında kod tarafında yalnız `research/eval/report_p1c.py` (2 satır, F9 tablo kaçışı) ve `research/FROZEN.md` (1 satır) değişiyor; gerisi `docs/` ve resmi RUN dosyaları.
- **Resmi RUN:** `20261015T014417Z-9aa326b` (`reports/rounds/P1-C/runs/…`).
- **Tarih:** 2026-10-15 (UTC ~07:25–08:40).
- **Ortam:** macOS 27.0.1 (Darwin 27.0.0 arm64) · rustc 1.92.0 · Python 3.13.5. Kendi Python araçlarım için ayrı bir venv kurdum: `blake3`, `numpy`, `onnxruntime`, `tokenizers`, `sentence-transformers` 6.1.0, `torch` 2.14.1. A/B zincirlerinin ortamını kullanmadım.
- **Yöntem:** Depo `git clone` ile scratchpad'e alındı (`clone/`). Bütün koşturmalar ayrı kopyalarda yapıldı:
  - `PA/`, `PB/` — `verify.sh pair` zincirleri; kaynak `clone/`.
  - `cur/` — HEAD kopyası. Model dosyaları **kendi indirdiğim** dosyalar. V2–V9'daki kendi testlerim burada koştu.
  - `hf/`, `hfst/` — HuggingFace'ten doğrudan indirilen model dosyaları (ONNX ve PyTorch safetensors).
  - Kendi vault'larım: `v4/` (D3 kopyaları), `v5/` (kendi metinlerim, proposal ve derived'li), `v6/`, `v8/`, `kc01/`, `c209/`, `v3c3/`.

  Doğrulama sonunda `clone`, `cur`, `PA/repo`, `PB/repo` temiz (`git status --porcelain` 0 satır, HEAD `44d2bdc`).
- **Süreç notu (dürüstlük):** Klonu almadan önceki ilk komutla, ana dizindeki commit'li `docs/rounds/P1-C/VERIFY.md` dosyasını okudum ve `docs/rounds/P1-C/` dizinini listeledim. Bu, DEV_PROCESS §5'teki "okuma dahil erişim yok" kuralından sapma. Okunan dosya HEAD'de commit'li ve çalışma ağacı temizdi. Doğrulamanın geri kalanı yalnız klonlardan yürüdü. Ana dizine yazılan tek dosya bu rapor (commit'lenmedi).
- **Kendi araçlarım** (hiçbiri depoya girmedi):
  - `hfdl.py` — model indirme + sha256.
  - `v4.py` — determinizm testleri; yalnız CLI.
  - `v5gen.py`, `v5.py` — katman vault'u ve testleri.
  - `v6.py` — iz formülü.
  - `c209.py` — bozuk iz satırları.
  - `kc01.py` — `LayerChanged` silme vakaları.
  - `v3c3.py` — C3 fixture'larını uyumluluk koşucusu yerine CLI ile yeniden oynatır.
  - `v7cos.py`, `v7st.py` — kosinüsler, kendi ST hesabım.
  - `cmpjson.py` — JSON yapraklarını tam eşitlikle karşılaştırır. Hariç tuttuğu yapraklar: gecikme (`*_ms`, `p50`, `p95`, `latencies_ms`), yollar (`fixtures_root`, `root`, `run_b`) ve `run_id`. Hariç tutulan farklı yaprakların listesi aşağıda.

Etiketler: `[OK] [FARK] [ŞÜPHE] [İHLAL]`

---

## V1 — Çift koşturma `[OK]`
**Komut:** `bash clone/scripts/verify.sh pair <scratch>/clone <scratch>/PA <scratch>/PB`
- Çıkış kodu **0**.
  - A: RUN `20261015T073220Z-44d2bdc`.
  - B: RUN `20261015T075209Z-44d2bdc`; B zinciri 1162 s.
- `pair_compare.json` (compare_p1c): `n_fields 99053`, `max_relative_difference 0.0`, `C-6_equal true`, `H4-2_equal true`, `bytes_equal true`.
- Kendi karşılaştırmam (`cmpjson.py`, 34 JSON dosyası, 80194 kalite yaprağı):
  - **A↔B:** 0 fark.
  - **A↔resmi RUN:** 0 kalite farkı. Yalnız 17 `manifest.json:phases.*.git_sha` farklı (`44d2bdc` ↔ `9aa326b`); bu beklenen, çünkü iki commit arasında ölçüm kodu değişmiyor.
  - Hariç tutulup farklı çıkan yapraklar yalnız şunlar: `c8_latency` ve `h3_v3` gecikmeleri, `fixtures_root`, `models_verify.root`, `regress_*.run_b`, `manifest.run_id`.
- **İndeks baytları:** `diff -r PA/p1c_work/<RUN>/.mdkernel/index PB/p1c_work/<RUN>/.mdkernel/index` → eşit (`lexical`, `vectors`, `hot`, `manifest.json`, `records.jsonl`, `snapshots.jsonl`). `p1c_index.json`'daki 18 dosya hash'i A = B = resmi RUN.
- **200 sorgu sıralaması:** `p1c_index.json:rankings_200` (flat, lexical, hybrid; her biri 200 sorgu, skorlarıyla) ve `rankings_after_compact` A = B = resmi RUN, birebir.
- **Temiz ağaç:** iki RUN'da 17 fazın hepsi `git_sha 44d2bdc`, `worktree_clean true`.
- **Kasıtlı bozma:** iki zincirde de `tamper_p1c` 6/6 ret, `test_tamper` 12/12.

## V2 — Model `[OK]`
- **Kendi indirmem:** `hfdl.py`, `huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/1110a243…/<dosya>` adresinden indirdi. HF API bu revizyonu çözüyor. Dört dosyanın boyutu, sha256'sı ve (kendi venv'imdeki `blake3` ile) blake3'ü `models/MANIFEST.json` ile birebir:
  - `model.onnx` — 90405214 bayt, blake3 `e73168f9e1f4…`
  - `tokenizer.json` — 466247 bayt, blake3 `82483bb4…`
  - `sentence_bert_config.json` — 53 bayt
  - `1_Pooling/config.json` — 190 bayt
- `cur/models/`'a kendi dosyalarımı koydum; `make models` → `ok true`, `embedder_id minilm-l6-v2/e73168f9`.
- **Manifest bozma** (`cur` içinde; her biri `git checkout` ile geri alındı) — `make models` her durumda **hata**:

| Bozma | Sonuç |
|---|---|
| `model.onnx` blake3 1 hex | `blake3 … != manifest …` |
| `tokenizer.json` blake3 | `blake3 … != manifest …` |
| `tokenizer.json` size | `size 466247 != manifest 466248` |
| `sentence_bert_config.json` sha256 | `fetch_model: sha256 … != manifest` (yeniden indirip reddediyor) |
| `revision` + sha256 | `curl 404`, Error 56 |
| dosya girdisi silindi | `no entry for tokenizer.json` |
| boş manifest `{}` | Error 1 |
| yerel dosyaya 1 bayt eklendi, doğrudan `mdk-index models verify` | `size 54 != manifest 53`, çıkış 1 |

- **`embedder_id` türetimi:** `index/src/models.rs:54` → `format!("{}/{}", manifest.name, manifest.files["model.onnx"].blake3[..8])`. Değer, `verify()` dosyanın blake3'ünü manifest'le eşleştirdikten sonra manifest'ten türetiliyor; kodda sabit `e73168f9` yok.
- Not (Ş3): yerel model dosyası bozulursa `make models` hata vermiyor; sha256 uyuşmazlığını görüp dosyayı yeniden indiriyor ve onarıyor (bkz. Şüpheler).

## V3 — Uyumluluk `[OK]` (sayı farkı ADDENDUM'la açıklanıyor)
- `make conformance` (cur): `level_claimed 3`, `spec_version 1.0.1`, **30/30** (C1 17, C2 9, C3 4). VERIFY 29/29 diyor; ADDENDUM K-C0.2 C2-09'u ekliyor (17 + 9 + 4 = 30). Bu tutarlı, ihlal değil.
- **Fixture'ları yeniden üretme:** `scripts/gen_fixtures_c3.sh` (C3 `expected.json` + MANIFEST, frozen v3-final üreteci) ve `scripts/gen_fixtures_c2.sh` ardışık koşturuldu → `git status` temiz.
- **Bağımsız C3 yeniden üretimi** (`v3c3.py`): `case.c3.json` işlemleri uyumluluk koşucusu kullanılmadan yalnız `mdk`/`mdk-index` CLI ile oynatıldı (ingest-dir, propose, ingest, demote, checkpoint, build, rebuild, search). Her `snapshot_id`, `checkpoint_id`, demote `lineage_id`, build sayısı ve **her arama sıralaması** (`doc_path`, `chunk_id`, `layer`) `expected.json` ile birebir:
  - C3-01: 5/5
  - C3-02: 5/5
  - C3-03: 20/20
  - C3-04: 11/11
  - `derived` hit: 0
  - Yeniden oynatılmayan işlemler: `traces` ve `view`.
- **ADDENDUM — C2-09 kendi bozuk satırlarımla:** her vakada `at_seq` ve `trace_id` geçerli olacak şekilde yeniden hesaplandı, böylece ret yalnız hedef alandan geliyor. `mdk trace list` ve `mdk-index search` ikisi de çıkış 1:
  - `query_hash` 31 hex → ret (`is not 32 lowercase hex`)
  - `query_hash` 33 hex → ret
  - `query_hash` hex olmayan (`g`×32) → ret
  - `query_hash` büyük harf → ret
  - `consumer` içinde `\n` → ret (`contains a newline`)
  - Kontrol: aynı biçimde geçerli satır → açılıyor (çıkış 0).
- **ADDENDUM — K-C0.1 sınır vakası kendi kurulumumla:** insan yazısı + ajanın iki paragrafı düzenlemesi → aynı snapshot'ta iki `LayerChanged`. Yeniden yazımın sadakati doğrulandı: satırlar yazarın JSON biçimiyle bayt eşit yeniden üretiliyor.

| Vaka | Sonuç |
|---|---|
| iki `LayerChanged`, ilki silinip yeniden numaralı | `Err` (`does not match the replayed layer rule`) |
| iki `LayerChanged`, ikincisi silinip yeniden numaralı | `Err` (`snapshot committed before LayerChanged`) |
| iki `LayerChanged`, ilki silinip `seq` boşluğu bırakıldı | `Err` |
| tek `LayerChanged` silinip yeniden numaralı | **açılıyor**; ilgili blok `proposal`, `layer_source derived` (K-C0.1 kabul edilmiş sınır) |
| tek `LayerChanged` silinip boşluk bırakıldı | `Err` |

## V4 — Determinizm (kendi testlerim) `[OK]`
D3 vault'unun kopyası kullanıldı (`index` hariç). Checkpoint `v4cp` = `368158345bfc…`; C-3'ün checkpoint'iyle aynı. Yalnız CLI kullanıldı.
- **100 tekrar:** 5 D4 sorusu × 3 mod × 100 = 1500 arama, sıralama + `layer` + skorun `repr`'i ile karşılaştırıldı → 0 fark.
- **İndeks silinip yeniden kurulunca:** 30 soru × 3 mod = 90 sıralama → 0 fark. İndeks dosyaları da eşit; tek fark aşağıdaki kilit dosyası.
- **Farklı dizinde kurulum** (`v4/elsewhere/deeper/vault-b`): 90 sıralama → 0 fark. İki taraf da arama yaptıktan sonra `diff -r index/` → **bayt eşit**.
  - Ara durumda görülen tek fark `lexical/.tantivy-meta.lock`: build sonunda kilitler siliniyor, ilk aramada tantivy okuyucusu 0 baytlık kilidi yeniden yaratıyor. Denetleyici (`run_p1c_checks.index_files`) bu dosyayı "the reader's empty lock file is not data" diyerek açıkça hariç tutuyor (Ş2).
- **`compact` öncesi/sonrası:**
  - 8 dokümanı düzenleyip `index watch` ile sıcağa aldım; `v4cp2` checkpoint'i kuruldu.
  - 32 sorgu × 3 mod × 2 checkpoint = 192 sıralama, compact öncesi ve sonrası → 0 fark.
  - Eski checkpoint'in sonuçları sıcak düzenlemelerden etkilenmedi: 90/90 temel çizgiyle eşit.
  - `status`: sıcak 8 → 0, soğuk 5998 → 6006.

## V5 — Katman `[OK]` (bkz. Ş1)
Vault'u kendi metinlerimle kurdum (`v5gen.py`): 30 insan dokümanı, 8 ajan dokümanı (`propose`), 6 insan dokümanında ajan düzenlemesi. Ardından `policy.toml` (`human_only`) ile 14 blok `derived`'e indirildi: 3 dokümanda birer blok, bir dokümanın bütün blokları ve bir proposal bloğu. `cp2`'de bloklar: canonical 281, proposal 73, derived 14.
- 100 tohumlu rastgele sorgu × 3 mod × {varsayılan, `--include-proposal`} × {cp1, cp2}. `k=10` ve `k=200` ile koşturuldu; `k=10`'da canonical'lar listeyi doldurduğu için proposal görünmüyor.
  - Her hit'in katmanını kendim hesapladım: `blame --at`'ten blokların en düşük katmanı. Bu değer etiketle 0 uyuşmazlık veriyor.
- **Varsayılan:** `proposal` hit **0** (k=200, cp2'de 23145 hit, hepsi canonical).
- **`--include-proposal`:** proposal hit'ler `layer: "proposal"` etiketli (cp2'de 7144) ve hepsi canonical'lardan sonra ("proposal'dan sonra canonical" 0).
- **Skor ×0.5:** vault'un bir kopyasında bütün proposal soyları `promote` edildi; aynı chunk'ın flat ve lexical skorları karşılaştırıldı. 5667 çiftin **hepsinde** oran tam 0.5.
- **`derived`:** hiç dönmedi. Hiçbir hit'in blokları `derived` değil; iki seçenekte, iki checkpoint'te, `index build` öncesinde de sonrasında da 0. Demote, mevcut indeks yeniden kurulmadan sorgulandığında da `derived` sızmıyor.
- **`--at`:** cp2'den sonra ingest edilen `late.md` (benzersiz kelimeler) cp2'de hiçbir modda görünmüyor; cp3'te üç modda da 1. sırada.

## V6 — İz `[OK]`
- 5 arama: Unicode sorgu, boş sorgu ve aynı sorgunun tekrarı dahil; farklı modlar ve tüketiciler. Her arama `traces.jsonl`'a **tam 1 satır** ekledi.
- **Formüller:**
  - `query_hash = blake3(query)[:32]` — 5/5 eşit; hepsi 32 küçük harf hex.
  - `trace_id = H(checkpoint_id \n consumer \n query_hash \n at_seq)` — 5/5 kendi hesabımla eşit.
  - Aramanın stdout'undaki `trace_id` ve `checkpoint_id` iz satırıyla aynı.
  - `at_seq` 0, 1, 2, … artıyor (K18-T).
  - `returned` anahtarları `{doc_path, lineage_id, version_id}`. Kümesi, hit'lerin `lineage_ids`'iyle birebir (30/30).
- **Hata yolları sessiz değil:**
  - `--consumer` içinde `\n` → çıkış 1, iz satırı eklenmedi.
  - İndeks yokken arama → çıkış 1 (`no index … run mdk index build`), iz yok.
  - İndekslenmemiş snapshot'lı yeni checkpoint → `snapshot … is not indexed` hatası; `index status` `stale_docs` alanında gösteriyor.

## V7 — Kalite (C-3, C-10) `[OK]` ölçüm yeniden üretildi; C-3 **kaldı**
- **C-3, A çıktısından (`c3_quality.json:per_query`) kendi hesabım:** hücre ortalamaları; referans v2-final `20261013T154851Z-f133d0b/h2.json` hücreleri. Soru kimlikleri `splits/d4.json` test bölümüyle birebir (297 soru, tekrar yok).

| Hücre | n | nDCG@10 | v2 | Δ | |Δ| ≤ 0.01 |
|---|---|---|---|---|---|
| hybrid/structural/single | 173 | 0.676857 | 0.673959 | **+0.002898** | ✓ |
| hybrid/structural/multi | 124 | 0.479977 | 0.493580 | **−0.013603** | ✗ |
| flat/structural/single | 173 | 0.526144 | 0.526144 | 0.000000 | (bilgi) |
| flat/structural/multi | 124 | 0.361294 | 0.361294 | 0.000000 | (bilgi) |

  - Δ değerleri FINAL'deki +0.0029 ve −0.0136 ile aynı.
  - Flat hücrelerinde nDCG, recall@10 ve MRR Δ 0. Bu, K-C8-T'nin teşhisiyle (fark sözcüksel tarafta) tutarlı.
- **C-10, 50 kosinüsü kendim hesapladım** (`v7cos.py`, `v7st.py`):
  - Rust ONNX vektörleri, D3 indeksimden (`vectors.f32`; kayıt sırası `embedders.jsonl` ile doğrulandı) ↔ `embed_ref.json`: en düşük **0.9999999999992166**; 50/50 ≥ 0.999. FINAL ile aynı.
  - Kendi Python `onnxruntime` hesabım (kendi indirdiğim model, mean pooling, L2, tek iş parçacığı) ↔ `embed_ref`: 0.99999999999921.
  - Kendi Python hesabım ↔ Rust: 0.99999999999997.
  - **Referansın kendisi:** kendi kurduğum `sentence-transformers` 6.1.0 + PyTorch, HF'den indirdiğim `model.safetensors` ile aynı 50 metni gömdü. `embed_ref.json` ile en düşük kosinüs 0.999999999999815, en büyük eleman farkı 8.9e-8. Dosya gerçekten ST çıktısı; ONNX'ten kopyalanmamış.
  - 3 chunk 256 tokende kesiliyor (`max_seq_length 256`); üç yolda da aynı biçimde.

## V8 — Sıcak katman `[OK]`
- **C-8 durumu:** A RUN'da 200 düzenleme için p50 28.9 ms, p95 **33.5 ms** < 100 ms (`pass true`, `threads 1`). Resmi RUN'da p95 32.2 ms. Durum her iki RUN'da "geçti"; sayı makineye bağlı.
- **Yalnız etkilenen chunk'lar embed ediliyor** (`index watch` sayacı, kendi vault'umda 4 chunk'lık `h20.md`):

| Değişiklik | `events` | `embedded` |
|---|---|---|
| değişiklik yok | 0 | 0 |
| yeni bölüm eklendi | 3 | 1 |
| bir bölümde bir paragraf düzenlendi | 2 | 1 |
| iki ayrı bölümde birer paragraf düzenlendi | 3 | 2 |

  Bunun ardından `status` sıcak 4 kayıt gösteriyor.

## V9 — Dondurma, bağımlılık yönü, F-istisnaları, SPEC_EDITS `[OK]`
- `scripts/check_frozen.sh` → `OK`; tablo P1-B ve P1-C ROUND_START'tan beri append-only.
- `scripts/check_deps.sh` → `OK (core ← index ← server ← clients; research → bindings/python)`.
- **`cargo tree`:**
  - `mdkernel-index`'in normal bağımlılıklarındaki tek iç crate `mdkernel-core`.
  - `index/Cargo.toml`'da dev-dependency yok.
  - `index`'e bağımlı olanlar yalnız `mdkernel-server` ve `mdkernel-bindings`.
  - `core`, `index`'e de `server`'a da bağımlı değil.
  - Not: `mdk index …` ve `mdk search …`, aynı dizindeki `mdk-index` ikilisini **çalışma zamanında çalıştırıyor** (`core/src/bin/mdk.rs:504–520`). Bu crate bağımlılığı değil, belgelenmiş bir yönlendirme; GOAL CLI'nin P1-D'de taşınacağını söylüyor.
- **Ölçüm kodu (`7053760..HEAD`):**
  - `research/eval/` altında yalnız **yeni dosyalar** var (`compare_p1c`, `gen_embed_ref`, `gen_tok_ref`, `report_p1c`, `run_c3_quality`, `run_p1c_checks`); mevcut eval dosyası değişmemiş.
  - `scripts/verify.sh` (+42/−4) F8 ile, `report_p1c.py` (2 satır) F9 ile kayıtlı.
  - `FREEZE_EXCEPTIONS.md` F1–F9 var.
  - Fixture değişiklikleri yalnız ekleme: C2-09, C3-01..04 ve MANIFEST satırları (+17, silme yok).
- **Spec:** `7053760..HEAD` diff'i §9'a tek bir "Informative: …" maddesi ekliyor; §1–§8 değişmemiş. Durum satırı §9–§10 için "Draft" diyor.
- **SPEC_EDITS S1:** CRITERION'a göre bilgilendirici not (a) ve (b)'yi sağlar → "açıklık". Sınıflandırma doğru; sürüm 1.0.1 (K-C0.3) doğru.
- GOAL, TASKS, ADDENDUM ve VERIFY'ın son değişikliği ROUND_START commit'i `7053760`; sonrasında değişmemiş.

## V10 — Karar kuralı `[OK]`
- **GOAL'den türetim:** C-1..C-11 hepsi geçerse §9 Validated olur. C-3 ya da C-10 kalırsa karar talebi açılır, P1-D başlar, 0.1.0 bekler. C-4/C-5/C-6 kalırsa P1-D başlamaz. C-7 kalırsa P1-D başlamaz.
- **Bağımsız sonuçlar:**
  - Kalan tek kriter C-3 (V7; hybrid multi Δ −0.0136).
  - C-10 geçti (V7).
  - C-4, C-5, C-6, C-9 geçti (V1, V4).
  - C-7 geçti (V5; A RUN'da da 1000 sorgu, varsayılan proposal/derived 0).
  - C-1 ve C-2 (C-12, C-13 dahil) geçti (V3).
  - C-8 geçti (V8).
  - C-11: A↔resmi RUN ve A↔B 0 fark; `regress_vs_p1b1` geçti.
- **Türetilen karar:** karar talebi (DECISIONS K-C8-T yazılmış). **P1-D başlar, 0.1.0 yayımı bekler.** §9 "Draft" kalır; `level_claimed 3`.
- **FINAL ile karşılaştırma:** birebir aynı.

---

## Sonuç

### İhlaller
- Yok.

### Farklar
- Yok.
  - Bilgi: VERIFY V3 "29/29" diyor, gerçek 30/30. ADDENDUM C2-09 ile tutarlı.
  - Bilgi: A↔resmi RUN'da yalnız `git_sha` alanları farklı; ölçüm kodu iki commit arasında değişmiyor.

### Şüpheler
- **Ş1 — `--include-proposal` canonical kısmının sıralamasını da değiştiriyor.** Proposal'lar listeye "sona eklenmiyor"; dahil edilince canonical sonuçlar da değişiyor. Kendi vault'umda canonical top-10'un öneki, varsayılanla karşılaştırınca şöyle:
  - hybrid: **29/30** soruda farklı
  - lexical: 5/30 soruda farklı
  - flat: 0/15

  Canonical listesi kısalmıyor; fark sıra ve küme farkı. `k=200`'de hybrid'de 15 soruda 154 canonical chunk listeden düşüyor. Sebepler:
  - **lexical:** BM25 istatistikleri (N, n, ortalama uzunluk) uygun chunk kümesinden hesaplanıyor; proposal'lar bu kümeye girince canonical skorlar değişiyor.
  - **hybrid:** RRF'nin her taraftan aldığı ilk 50 aday proposal'larla paylaşılıyor.

  Spec §9'un bilgilendirici notuyla ("statistics … over the chunks eligible") ve normatif metinle çelişmiyor. Ancak "proposal etiketli, ×0.5, canonical'dan sonra" ifadesi, bir istemcinin bekleyeceği "canonical sonuçlar aynı kalır" özelliğini vermiyor. P1-D'de MCP bu bayrağı dışarı açacağı için önemli.
- **Ş2 — Okuma, indeks dizinine yazıyor.** İlk `search` `lexical/.tantivy-meta.lock` (0 bayt) yaratıyor; `build` bunu siliyor. "`index/` bayt eşit" yalnız aynı okuma durumundaki iki dizin için tutuyor. Denetleyici dosyayı açıkça hariç tutuyor ve `diff -r` iki taraf da okunduktan sonra eşit; yani ölçüm doğru. Yine de "salt okunur sorgu" beklentisiyle ve dışarıdan `diff -r` yapacak bir doğrulayıcıyla sürtüşme yaratıyor.
- **Ş3 (bilgi) — `make models` bozuk yerel model dosyasını hata vermeden yeniden indiriyor.** sha256 uyuşmazlığında `fetch_model.sh` dosyayı HF'den yeniden çekip onarıyor; sonuç manifest'le doğrulandığı için sessiz başarısızlık değil. Kasıtlı bozma testinin "model file changed" vakası bu yüzden `make models` değil, doğrudan `mdk-index models verify` ile yapılıyor. İkisi de ayrı ayrı doğru.
  - Ek bilgi: boş sorgu (`""`) hata değil, sonuç döndürüyor (hybrid, 10 hit) ve iz yazıyor. Spec bu konuda sessiz.

### Hüküm
**Güvenilir.**

FINAL.md'deki her ölçüm bağımsız olarak yeniden üretildi:
- **Çift koşturma:** 0 fark; indeks `diff -r` eşit; 200 × 3 sıralama A = B = resmi RUN.
- **Model:** kendi indirdiğim dosyalarla 4/4 hash eşit; 8/8 manifest bozması hata.
- **Uyumluluk:** level 3, 30/30, 1.0.1. C3 beklenen değerleri hem üreteçle hem kendi CLI oynatmamla birebir. C2-09 5/5 ret; K-C0.1 vakaları beklenen davranışta.
- **Determinizm:** 1500 tekrar, yeniden kurulum, farklı dizin ve compact'ta 0 fark.
- **Katman:** proposal varsayılanda 0; etiketli; ×0.5 5667/5667; derived 0; `--at` doğru.
- **İz:** 5/5 formül, satır başına bir iz.
- **Kalite:** C-3 Δ'ları ve C-10 kosinüsleri kendi hesabımla aynı; `embed_ref` gerçek ST çıktısı.
- **Sıcak katman:** C-8 geçti; embed sayacı yalnız değişen chunk'ları sayıyor.
- **Dondurma ve bağımlılık:** temiz.

Karar kuralının sonucu **P1-D başlar, 0.1.0 yayımı bekler** (C-3 kaldı; K-C8-T). Bu, FINAL ile aynı.

### Öneriler (en fazla 3)
1. **`include_proposal` anlamını P1-D'den önce karar kaydıyla sabitleyin (Ş1).** Önerilen anlam: canonical sıralama yalnız canonical aday kümesinden hesaplanır, yani bayrak kapalıyken elde edilenle bayt eşittir; proposal'lar ayrı hesaplanıp ×0.5 ile sona eklenir. Hangisi seçilirse seçilsin, Seviye 3'e bunu sınayan bir özellik vakası ekleyin: "bayrak açıkken canonical önek = bayrak kapalıyken sonuç" ya da tersi, açıkça.
2. **K-C8-T'yi 0.1.0'dan önce kapatın.** Bağımsız bulgular: flat Δ = 0 (nDCG, recall ve MRR), ONNX ↔ ST kosinüsü ≥ 0.99999999999921. Fark yalnız sözcüksel tarafta. Seçeneklerden (a/b/c) biri, eşiği önceden kayıtlı yeni bir turda seçilmeli. (b) ya da (c) seçilirse BM25 varyantı indeks `manifest.json`'una yazılmalı (şu an yalnız `k1`/`b` var, IDF biçimi yok).
3. **İndeks dizinini okumaya karşı kararlı yapın (Ş2).** Sorgu yolunda tantivy kilidinin dosyaya yazılmasını önleyin (salt okunur açılış) ya da bu dosyayı `.managed.json` ve spec §9'un bilgilendirici notunda "veri değil" olarak tanımlayın. Böylece dış doğrulayıcının düz `diff -r`'ı her durumda anlamlı olur.
