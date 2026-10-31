# P2-V.1/E1 — Zincirde beklenen fark aileleri (ön-kayıt; kademe 2 ölçümünden önce)

GOAL P2-V.1 İlkeler: "Gömme değişikliğinin zincirde değiştireceği alanlar … ölçümden önce aile aile adlandırılır; adı geçmeyen fark tanımlanmamış farktır." Bu liste E1'de, kademe 2'nin turdaki ilk tetiklemesinden önce commit'lenir; E3'ün envanteri (`reports/rounds/P2-V.1/inventory.json`) bu ailelerin alanlarını ad ad taşır (desen yok). İki karşılaştırma ayrıdır:

- **A ↔ B (aynı commit, iki koşucu; V1-2, V1-7):** bu ailelerin hiçbiri farklı olmamalı — sayısal yol artık işlemciden bağımsız. P2-V'de V4b'nin 209 281 alanlık farkının hepsi buradadır; P2-V.1'de 0 beklenir.
- **eski ↔ yeni (P2-V V0 referansı `352f790` ↔ turun son commit'i; V1-6) ve zincirin dondurulmuş referans RUN'larına karşı regress'leri:** yalnız bu ailelerde fark beklenir.

## Mekanizma
`embedder_id` `+fixed1` etiketini taşır (ASSUMPTIONS E1-1) ve vektörler artık `detbert`'in baytlarıdır: ORT'ninkinden bileşen başına ≤ ~3e-7 farklı (E0: kosinüs ≥ 0,999999999998). Değeri vektörlerin kendisinden ya da kosinüs skorlarından gelen her alan, ve kimliği taşıyan her dize değişir. Değeri yalnız sıralamadan (kimlik ve sıra) gelen alanlar değişmez: E1'de 200 sorgu × 3 kipte sıralama değişikliği **0** (`e1/quality.json`), hibrit (RRF sıralarından) ve sözcüksel skorlar aynı, D3 + D4'ün bütün kalite hücreleri ve sorgu başına değerleri aynı (`c3_quality.json` 4 838 yapraktan yalnız `index›embedder_id` farklı).

## Aileler
| Kod | Aile | Kayıtlar (bilinen konumlar) | Kaynak |
|---|---|---|---|
| **E1-V** | Vektör baytları | RUN'daki vault'ların `.rootmark/index/vectors/vectors.f32`'si ve `embedders.jsonl`'u; indeks `manifest.json`'unun bunları özetleyen alanları; kaydedicilerin indeks dosyası özetleri (`p1c_index.json` `files_build`/`files_rebuild` vb.) | P2-V V4b `d_bytes_differing` ve `p1c_index.json` 5 819 alan |
| **E1-E** | `embedder_id` dizgesi | `models_verify.json`, `c3_quality.json:index›embedder_id`, `c10_embedding.json:embedder_id`, `p1c_checks.json` (`models›embedder_id` ve C-10 kaydı), `p1d/scene/scene.json`, `p1d/http/status_*.json`, `p1d/inspector/calls/status.json`, `p1d/mcp_e2e/run*/prep_index.json` ve `transcript.jsonl`, `p2l/d_tr.json`, `p2l/english.json`; bunları dondurulmuş referans RUN'lara karşı gömen regress girdileri | V4b RUN'ında ORT kimliğini taşıyan dosyalar (`e73168f9`/`ca55a866` araması) |
| **E1-S** | Düz kip (kosinüs) skorları | `p1c_index.json:rankings_200›flat›<q>›<r>›3` (ve aynı kaydın yeniden kurulum/sıcak katman sıralamalarının düz skorları), `p1c1_checks.json` C1-8'in bunu gömen regress girdileri; skor taşıyan başka arama kayıtlarının yalnız `score` yaprakları | V4b `p1c_index.json`, `p1c1_checks.json` |
| **E1-C10** | C-10 kosinüsleri | `c10_embedding.json:chunks›<i>›cosine`, `min_cosine`; `p1c_checks.json:C-10›min_cosine` | V4b `p1c_checks.json` (1 alan) |
| **E1-D13** | D1-3 `after_hash` | `p1d1/d1_3.json` (bayt artefaktı ve `after_hash` alanları), `p1d1_checks.json` D1-3 kaydı | V4b `p1d1/d1_3.json` (2 alan + bayt) |
| **E1-DTR** | D-TR kayıtlarının vektör özetleri ve kimlikleri | `p2l/d_tr.json:builds›<varyant>/<model>›manifest›{vectors_blake3,embedders_blake3,embedder_id}` ve `…›index›embedder_id` (4 kurulum × 4 = 16 yaprak), `p2l/english.json:<varyant>›index›embedder_id` (2). E1'de ölçüldü (`e1/quality.json` `p2l_vs_p2l_official_run`): 67 572 + 9 604 yapraktan yalnız bunlar farklı; L-2…L-6'nın değerleri ve sıralamaları aynı | V4b `p2l/d_tr.json` (4 alan: aynı kurulumların `vectors_blake3`'ü) |
| **E1-R** | Bunları taşıyan regress alanları | `differing_fields`/`excluded_differences` altında yukarıdaki ailelerden birini adlandıran gömülü girdiler: `p1c1_checks.json` (C1-8), `p1d_checks.json` (D-9), `p1d1_checks.json` (D1-12), `p1e_checks.json`, `p1e1_checks.json`, `p2_0_2_checks.json`, `p2a_checks.json`, `p1e1/release/release_check.json` (V4b'de 1 alan) | V4b `d_fields_differing_by_file` |

## Beklenmeyenler (fark çıkarsa tanımlanmamış farktır)
- Sıralamaların kimlik ve sırası (üç kip), hibrit ve sözcüksel skorlar, kalite hücreleri ve sorgu başına ölçütler (C-3, C1-1, C1-9'un 618 değeri, P2-L'nin İngilizce referansı).
- Olay günlükleri, `events_hash`, kimlikler, snapshot'lar, uyumluluk raporları (39/39), göç kayıtlarının biçimleri.
- Yukarıdaki ailelerin dışında kalan her dosya ve alan.

## Not: dondurulmuş referansa karşı regress'ler
Zincirin regress'leri (C1-8, D-9, D1-12, …) dondurulmuş resmî RUN'lara karşıdır; o RUN'lar ORT ile (çoğu yayım Mac'inde) kaydedildi. E1-V/E/S/C10/D13/DTR alanları bu referanslardan artık **her makinede aynı biçimde** farklıdır (önce yalnız yayım Mac'i dışındaki makinelerde). Bu farklar sayılan FAIL'e dönmemelidir: E3'ün envanteri onları referansa bağlı (O) alanlar olarak ad ad taşır ve türev koruması (B1 düzeltmesi) tam listeye bakar; adı geçmeyen bir fark sayılır.
