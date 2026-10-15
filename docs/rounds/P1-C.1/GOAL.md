# P1-C.1 — İndeks Kapanışı

Kısa tur. P1-C'nin C-3 kalışını (K-C8-T) ve doğrulamanın Ş1/Ş2'sini kapatır. Bitmeden P1-D'nin D3 (search aracı) görevi başlayamaz; D0–D2 başlayabilir.

## Kararlar (C1.0'da DECISIONS.md'ye)
- **K-C1.1 Sözcüksel skorlama normatif değil, bildirimseldir.** Spec §9 BM25'i "veya eşdeğeri" diye tanımlar; varyant seçimi uygulamanındır ama **indeks manifest'ine yazılmak zorundadır**: `lexical.variant` (`"lucene-bm25"`), `idf` formülü (metin), `k1`, `b`, tokenizer kimliği. Spec §9'a bilgilendirici not + 1.0.1 içinde kalır (K-B1.3 ölçütü: değer değiştirmiyor, uyumluluğu bozmuyor).
- **K-C1.2 Referans değişimi, kanıtla.** Prototip `rank_bm25` referansı araştırma tabanıydı, spec değil. Yeni regresyon referansı Rust `hybrid` sayılarıdır; ancak **şu kanıt** olmadan değişmez: `research/` içinde tantivy varyantının (K-C1.1 formülü) Python'da yeniden uygulanması + aynı RRF, P1-C'nin `hybrid` nDCG/recall/MRR değerlerini her hücrede ±0.001 içinde üretmeli. Üretmiyorsa fark varyant değil hatadır ve tur "kaldı" döner.
- **K-C1.3 `include_proposal` anlamı.** Bayrak açıkken canonical sonuçlar bayrak kapalıyken elde edilenle **bayt eşittir**; proposal'lar yalnız canonical üzerinden hesaplanmış istatistiklerle **ayrı** puanlanır, ×0.5 ile etiketli olarak **sona** eklenir. Spec §9'a normatif cümle; C3-05 özellik vakası eklenir (Seviye 3). Sürüm 1.0.1 → **1.0.2** (CRITERION yama kuralı: önceki uyumlu uygulama bu özelliği sağlamıyorsa uyumsuz hale gelir → yama değil… bkz. not). *Not:* K-B1.3 (b) gereği bu uyumluluğu daraltan bir değişikliktir; CRITERION'un sürüm kuralına göre **1.1.0** olur. Ajan CRITERION'u uygular, gerekçeyi yazar.
- **K-C1.4 Sorgu salt okunurdur.** `search` indeks dizinine hiçbir dosya yazmaz (tantivy salt okunur açılış); yazıyorsa hata. `index/` bayt eşitliği artık istisnasız.
- **K-C1.5 Boş sorgu reddedilir** (`Err`), iz yazılmaz. Spec §9 normatif, 1.1.0 içinde.

## Çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| C1-1 | Python tantivy-varyant BM25 + RRF vs Rust `hybrid`, 4 hücre, nDCG/recall/MRR | \|Δ\| ≤ 0.001 |
| C1-2 | `include_proposal=true` canonical öneki vs `false` sonucu, 1000 sorgu × 3 mod, proposal'lı vault | bayt eşit 1000/1000 |
| C1-3 | Proposal hit'ler etiketli, ×0.5, hepsi canonical'lardan sonra | property 1000 |
| C1-4 | `search` sonrası `index/` dizini `diff -r` değişmemiş (`.lock` dahil hiçbir dosya) | eşit |
| C1-5 | Boş / yalnız boşluk sorgu | `Err`, iz satırı yok |
| C1-6 | İndeks manifest'inde `lexical.{variant,idf,k1,b,tokenizer}` | var; eksikse `index status` hata |
| C1-7 | C1 17/17, C2 9/9, C3 **5/5**, `spec_version 1.1.0`, `level_claimed 3` | geçer |
| C1-8 | Çift koşturma; P1-C kalite alanları (C-3 ve C3-0x hariç tutma listesi kararla) | fark 0 |
| C1-9 | Regresyon referansı güncelleme: `research/data/quality_ref_rust.json` yazılır, FROZEN'a eklenir; yeniden koşturma referansla fark | 0 |

## Karar kuralı
C1-1 kaldı → varyant açıklaması yanlış, hata var; P1-D D3 başlamaz, karar talebi. C1-1 geçti ∧ diğerleri geçti → K-C8-T kapanır, spec §9 "Validated", 0.1.0 yayım engeli kalkar, P1-D tam başlar.

## Görevler
- **C1.0** `git tag p1c-final`; `ROUND_START`; DECISIONS ekleri; spec §9 1.1.0 (K-C1.1 not, K-C1.3/K-C1.5 normatif cümleler), `SPEC_EDITS.md`, CONFORMANCE C3-05.
- **C1.1** `research/eval/bm25_lucene.py` (yeni): tantivy formülü; `run_c1_variant.py`; C1-1 raporu.
- **C1.2** `index`: canonical-yalnız istatistikler, ayrı proposal geçişi; `lexical.*` manifest alanları; salt okunur açılış; boş sorgu `Err`.
- **C1.3** C3-05 fixture; `gen_fixtures_c3.sh`; C3-01..04 beklenen değerleri **değişirse** (canonical-yalnız istatistik nedeniyle) yeniden üretilir ve değişen her fixture `SPEC_EDITS.md`'de gerekçeli listelenir.
- **C1.4** `verify.sh` genişletme (C1-2..C1-6 kasıtlı testler dahil: `.lock` üretimi, boş sorgu, manifest alanı silme); `quality_ref_rust.json`; rapor; `READY.md`.

## Yapılmayacaklar
Yeni mod, reranker, ANN; MCP; eşleyici; embedder değişikliği.
