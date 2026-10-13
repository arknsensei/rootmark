# 00 — Hedef, Hipotezler, Önceden Kayıtlı Eşikler (v2)

## Bağlam

v1 (Python prototip) üç hipotezi test etti. Sonuç: H1-S1/S2/R2 ve H3 güçlü biçimde geçti; H1-S3 spesifikasyondaki eşleyici sıralama hatası yüzünden 0 çıktı; H1-R1 birebir aynı içerikli tekrar bloklar yüzünden 0.9997'de kaldı; H2-1 geçti ama rastgele soy kimlikleri yüzünden temiz ortamda yeniden üretilemedi; H2-3 gecikme tam-graf PageRank yüzünden kaldı; `hybrid` (BM25+ANN) ölçülen her hücrede en iyi çıktı.

v2, çekirdeği **Rust** ile sıfırdan yazar, v1'in hatalarını giderir ve tüm hipotezleri temiz ortamda yeniden üretilebilir biçimde yeniden ölçer. v2 çıktısı `reports/FINAL.md` + `reports/VERIFICATION.md`'dir.

## Hipotezler

**H1 — Soy kararlılığı.** Markdown'a kimlik yazmadan, diff ile blok soyu kararlı takip edilebilir; bölme/birleştirme olayları tespit edilebilir; tekrar bloklar komşu bağlamıyla çapalanabilir.

**H2 — Yapı-farkında arama.** Wikilink grafı ve başlık hiyerarşisi, aynı embedding ile, güçlü bir taban çizgisine (`hybrid`) karşı çok-adımlı sorularda ölçülebilir kazanç sağlar ve kabul edilebilir gecikmede çalışır.

**H3 — Artımlı indeksleme ve blok/chunk ayrımı.** Yazma 1 saniye altında aranabilir olur; chunk stratejisi değişimi soy kaybetmez. (v1'de geçti; Rust çekirdekle yeniden teyit.)

**H4 — Determinizm.** Aynı girdi, aynı komutla, farklı makinede bit düzeyinde aynı soy kimlikleri, olay günlüğü ve kalite metriklerini üretir.

## Önceden kayıtlı eşikler (DEĞİŞTİRİLEMEZ)

| Kod | Ölçüm | Eşik | v1 |
|---|---|---|---|
| H1-S1 | Sentetik soy koruma (`rephrase_*`, `move`) | ≥ 0.95 | 0.999 |
| H1-S2 | Sentetik yanlış eşleme | ≤ 0.02 | 0.009 |
| H1-S3 | Bölme/birleştirme F1 | ≥ 0.80 | 0.000 (spec hatası) |
| H1-S4 | Tekrar-blok senaryosunda soy koruma (yeni op `dup_delete`) | ≥ 0.95 | — |
| H1-R1 | Gerçek git tarihçesi, dokunulmamış blok koruma | ≥ 0.999 | 0.9997 |
| H1-R2 | Gerçek git tarihçesi, genel koruma | ≥ 0.85 | 0.988 |
| H2-1 | `graph_hybrid` vs `hybrid`, multi-hop ΔnDCG@10 | ≥ +0.03 | — |
| H2-2 | `graph_hybrid` vs `hybrid`, single-hop ΔnDCG@10 | ≥ −0.02 | — |
| H2-3 | `graph_hybrid` p95 / `hybrid` p95 gecikme | ≤ 3.0 | 6.5 (flat tabanlı) |
| H2-4 | `graph_hybrid` vs `hybrid`, multi-hop Δfull-cover@10 | ≥ +0.03 | — |
| H2-5 | `graph_flat` vs `flat`, multi-hop ΔnDCG@10 (v1 ile karşılaştırma) | ≥ +0.05 | 0.061 |
| H3-1 | Yazma→aranabilir p95 | < 1000 ms | 29 ms |
| H3-2 | Yeniden-chunk soy kaybı | = 0 | 0 |
| H4-1 | İki bağımsız temiz koşturmada olay günlüğü hash eşitliği (D3 tamamı) | = eşit | — |
| H4-2 | İki bağımsız temiz koşturmada tüm kalite metriklerinde maksimum göreli fark | = 0 | H2'de üretilemedi |

v1'e göre değişen eşikler ve nedenleri: H1-R1 "=1.00" → "≥ 0.999" (56k blokta kenar durumları için tam eşitlik hipotez hakkında bilgi vermiyor; kalan her kayıp kategorize raporlanır). H2 tabanı `flat` → `hybrid` (v1 `hybrid`'in baskın olduğunu gösterdi; grafın sözcüksel aramanın üstüne bir şey ekleyip eklemediği asıl soru). H2-5, v1 sonucuyla doğrudan karşılaştırma için korundu.

Gecikme eşikleri (H2-3, H3-1) doğrulamada **durum** karşılaştırılır, sayı değil; gecikme makineye bağlıdır.

## Karar kuralı (önceden kayıtlı)

- H4 geçmedi → başka hiçbir sonuç geçerli sayılmaz; önce determinizm düzeltilir ve her şey yeniden koşturulur.
- H1 ∧ H2 ∧ H3 ∧ H4 geçti → çekirdek veri modeli (`01_SPEC.md` §1–7) **dondurulur** ve gerçek çekirdek aşamasına (sync, plugin yüzeyi, sunucu) geçilir.
- H1 geçmedi → veri modeli/eşleyici yeniden ele alınır; H2/H3 saklanır.
- H1 geçti, H2 geçmedi → çekirdek "versiyonlu md çekirdeği" olarak ilerler; graf arama araştırma kolu olarak ayrılır, çekirdeğe bağlanmaz.
- H3 geçmedi → sıcak katman yeniden ele alınır, mimari değişmez.

Kısmi H2: H2-1, H2-2, H2-4 geçip yalnız H2-3 kalıyorsa "H2 kalite geçti, gecikme kaldı" diye raporlanır ve gecikme bir sonraki tura mühendislik borcu olarak yazılır; bu durumda çekirdek aşamasına geçilir ama graf arama "kanıtlanmış, optimize edilmemiş" etiketi alır.

## Yapılmayacaklar
UI, sync, CRDT, çok tenant, plugin API, sunucu, Rust'ta embedding. Embedding modeli karşılaştırması. Ürünleşme.

## Başarı tanımı
`FINAL.md` her eşik için ham JSON'a izlenebilir sayı + durum + karar kuralı çıktısı içerir; **bağımsız ajan temiz klonda `make verify` ile aynı kalite sayılarını bit düzeyinde** (gecikmeler hariç) üretir ve `VERIFICATION.md` bunu belgeler.
