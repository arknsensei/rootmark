# 03 — Ölçüm Sözleşmesi (v2)

Metrik kodu `eval/metrics.py`, gerçek çözümleme `eval/truth.py`; `T9: freeze eval` sonrası dondurulur. Komutlar: `make d2 d3 d1`, `make eval-h1s RUN=`, `make eval-h1r RUN=`, `make eval-h2-tune RUN=`, `make eval-h2 RUN=`, `make eval-h3 RUN=`, `make eval-h4 RUN=`, `make report RUN=`, `make verify`. `RUN` verilmezse hata.

## Veri ve determinizm

Tüm veri `data/` altında ve git-ignored; **manifest'ler git'te**:
- `data/MANIFEST.d2.json`: depo URL, sabit rev, seçilen commit listesi, dosya başına blake3.
- `data/MANIFEST.d3.json`: depo URL, sabit rev, doküman listesi ve blake3'leri, ingest sonrası `events_hash`.
- `data/MANIFEST.d1.json`: D1 üreteç sürümü + SEED + çıktı dosyalarının blake3'leri.
- `data/d4/*.jsonl` ve `data/splits/*.json`: **git'te** (soru üretimi API gerektirdiği için). Gerçekler içerik-referanslı (aşağı).

Her `make d*` hedefi üretimden sonra manifest'i doğrular (ilk üretimde yazar, `--write-manifest` ile). Uyuşmazlık → hata.

### Gerçek (truth) referansı
Hiçbir gerçek dosyası `lineage_id` içermez. Gerçek birimi:
```json
{"doc_path": "notes/foo.md", "version_id": "<32 hex>", "k": 0}
```
`k` = snapshot'ta aynı `version_id`'li bloklar arasındaki sıra. `eval/truth.py:resolve(vault, ref) -> lineage_id`, `Vault.resolve_truth` üzerinden; **tek** çözülemeyen referans koşturmayı durdurur.

## Veri setleri

### D2 — Gerçek git tarihçesi (H1-R)
`https://github.com/rust-lang/book`, rev `1500248d8f23` (v1 ile aynı; manifest'te tam sha). `src/*.md`'ye dokunan son 300 non-merge commit, yalnız `M` dosyalar. `git diff -U0` hunk aralıkları. "Dokunulmamış" = hiçbir satırı hunk aralığında değil **ve** bir satırı hunk sınırına bitişik değil (v1'de bitişik satır vakası bulundu; tanım daraltıldı ve `ASSUMPTIONS.md`'ye yazılacak).

### D3 — Vault (H2, H3, H4, D1 kaynağı)
`https://github.com/dendronhq/dendron-site`, `vault/`, rev `fbc07769235b` (v1'de doğrulanmış: 1012 doküman, %53 wikilink). Yedek: `obsidianmd/obsidian-help` yalnız kabul koşulu sağlanırsa. Kabul: ≥200 doküman, ≥1000 structural chunk, ≥%40 wikilink'li. Ingest sırası: `doc_path` sözlük sırası (deterministik).

### D1 — Sentetik düzenlemeler (H1-S)
D3'ten `SEED` ile 200 doküman (ayar 50 / test 150). 10 adım/doküman. Operasyonlar ve oranlar:

| Op | Oran | Gerçek |
|---|---|---|
| `rephrase_light` (%5-15 kelime) | 25% | aynı soy |
| `rephrase_heavy` (%15-30) | 15% | aynı soy |
| `split` (cümle sınırından 2'ye) | 10% | split |
| `merge` (2 ardışık paragraf) | 10% | merge |
| `move` | 15% | aynı soy, moved |
| `insert` | 10% | yeni |
| `delete` | 5% | deleted |
| `dup_delete` | 10% | bkz. aşağı |

`dup_delete` (yeni, H1-S4): dokümana aynı içerikli 3 kopya kısa blok eklenir (önceki adımda, hazırlık), bu adımda **ilk** kopya silinir; gerçek: kalan 2 kopyanın soyları değişmez. v1'in H1-R1 kaybını doğrudan modelleyen senaryo. Hazırlık adımı sayılmaz.

Gerçek dosyası `truth.json`: her adım için `{"op":..., "mapping": [[old_ref, [new_ref,...]], ...]}`; ref'ler `(version_id, k)` biçiminde (doc_path sabit).

### D4 — Soru seti (H2)
`claude-sonnet-4-6`, `temperature=0` (SDK parametre adı farklıysa uyumluluk istisnası prosedürü). Single 200 chunk, multi 150 wikilink'li (A,B) çifti; çift başına yalnız bir doküman çifti; chunk ≥ 30 token. İstemler **İngilizce** (vault İngilizce; v1'de çeviri yapıldı, bu kez baştan İngilizce yazılır ve `eval/datasets/questions.py` içinde sabit string olarak durur):
- single: "Based only on the passage below, write one natural user question whose answer is contained entirely in the passage. Do not mention the document title or file name. Return only the question."
- multi: "Two passages are given. Write one question that requires BOTH passages to answer and cannot be answered from either alone. Do not mention titles or file names. Return only the question."
Sızıntı: soru, gerçek metindeki 8+ kelimelik ardışık diziyi içeriyorsa yeniden üret (3 deneme, sonra atla). Ek filtre: soru, hedef dokümanın dosya adı gövdesini (tire/alt çizgi boşluğa çevrilmiş) içeriyorsa yeniden üret. Bölünme: ayar 50 / test kalan. Gerçekler içerik-referanslı.

## Metrikler

### H1
- Koruma = gerçekte aynı soy ∧ tahminde aynı soy / gerçekte aynı soy. S1: `rephrase_*`, `move`. **S4: yalnız `dup_delete`.**
- Yanlış eşleme = gerçekte farklı/yeni iken var olan soya bağlanan / toplam yeni blok.
- Split/merge F1: olay doğru ⇔ soy kümesi gerçekle tam eşit; havuzlanmış P/R.
- R1 dokunulmamış koruma; R2 genel koruma (tanım: eşleyici eski soyu yeni snapshot'ta tutuyor). **Hunk içi** bilgi metriği: `touched_retention` (eski soy var mı) — tek tanım, v1'deki ikilik kaldırıldı.
- **Kayıp kategorizasyonu (zorunlu, H1-R1):** her dokunulmamış soy kaybı `{dup_content, hunk_adjacent, type_change, other}` ile etiketlenir; `other` ≥ 5 vaka ise ilk 5'i raporda listelenir (doc, version_id).

### H2
İlgililik: chunk `lineage_ids` ∩ gerçek ≠ ∅. k=10.
- Recall@5/10, MRR (v1 tanımları).
- nDCG@10: ikili gain, log2 indirimi, IDCG = `min_cover` adet ilgili chunk; 1.0 kırpması **kaldırıldı** (IDCG hesabı doğruysa gereksiz; >1 çıkarsa bu bir hatadır ve test yakalar).
- **full_cover@10 (yeni, H2-4):** multi-hop'ta ilk 10 chunk hem A'nın hem B'nin en az bir soyuna değiyorsa 1, değilse 0; single'da recall@10 ile aynı.
- Gecikme p50/p95, 10 ısınma hariç, sorgu embedding'i dahil.
- Bootstrap: `graph_hybrid − hybrid` ve `graph_flat − flat` için eşleştirilmiş, 10.000 örnek, %95 GA.
- ALPHA duyarlılık eğrisi: ayar seti, tüm ızgara, `graph_hybrid/structural/{single,multi}` nDCG; rapor + PNG.

Ölçüm hücreleri: 5 strateji × 2 chunk stratejisi × {single, multi}. Eşikler `structural` üzerinde.

### H3
v1 ile aynı; H3-1'e model yükleme dahil değil (model önceden yüklü), bu `ASSUMPTIONS.md`'de açık.

### H4
`make eval-h4 RUN=`: `data/d3` silinir, D3 sıfırdan ingest edilir, `events_hash` ile `MANIFEST.d3.json` karşılaştırılır (H4-1). H4-2 doğrulama aşamasında ölçülür: `make verify` iki ayrı temiz klonda (veya aynı makinede iki ayrı dizinde) tam zinciri koşturur ve `reports/runs/<a>` ile `<b>` kalite JSON'larını alan alan karşılaştırır (gecikme alanları hariç tutulur, liste `eval/compare_runs.py` içinde sabit).

## Koşturma ve rapor
Her `make eval-*` → `reports/runs/<RUN>/<hipotez>.json` + `manifest.json` (güncellenir). `make report RUN=` yalnız o dizini okur; eksik dosya → hata. `FINAL.md` şablonu `05_REPORT_TEMPLATE.md`.
