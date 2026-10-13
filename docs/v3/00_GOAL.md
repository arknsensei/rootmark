# v3 — Hedef, Eşikler, Karar Kuralı

v3, aynı depoda (`v2-final` etiketinden sonra) üçüncü ölçüm turudur. Kapsam dar: **H1'i kapatmak** ve v2 sonuçlarının korunduğunu göstermek. H2 koşturulmaz. `docs/ARCHITECTURE.md` K4 kararı bu turda uygulanır ve ölçülür.

## Değişen şey ve neden
- v2'de H1 yalnız S2 (yanlış eşleme 0.038 > 0.02) ile kaldı; kaynak birleştirme işlemiydi: kısa parça asgari parça oranının altında kaldığı için birleştirme adayı oluşmuyor, benzerlik ataması uzun parçanın soyunu devrediyordu.
- v2 THETA ızgarası düz çıktı: sentetik split/merge metinleri tam birleşimdi, ayar adımı bilgi taşımadı.
- K4 (ARCHITECTURE §10): bölme ve birleştirmede **baskın parça soyu korur**. Bu bir tasarım kararıdır; v2 sonuçları görüldükten sonra alınmıştır ve gerçek tanımını değiştirir. Bu nedenle S2 tanımı yeniden yazılmış, ve v2 davranışı bayrakla korunarak v2 sayılarının birebir yeniden üretilebildiği ayrıca gösterilmektedir (H4-V2).

## Hipotezler
- **H1 (yeniden):** K4 semantiğiyle ve simetrik birleştirme kontrolüyle eşleyici, hem kolay hem **zor** sentetik düzenlemelerde ve gerçek tarihçede soyu kararlı takip eder.
- **H3, H4 (regresyon):** Eşleyici değişikliği artımlı indeksleme ve determinizmi bozmaz; v2 davranışı bayrakla birebir yeniden üretilir.

## Önceden kayıtlı eşikler (DEĞİŞTİRİLEMEZ)

| Kod | Ölçüm | Küme | Eşik |
|---|---|---|---|
| H1-S1 | Soy koruma (`rephrase_*`, `move`) | D1-easy (K4 gerçek) | ≥ 0.95 |
| H1-S2 | Yanlış eşleme (K4 tanımı, EVAL_DELTA §Metrikler) | D1-easy | ≤ 0.02 |
| H1-S3 | Bölme/birleştirme olay F1 | D1-easy | ≥ 0.80 |
| H1-S4 | Tekrar-blok koruma (`dup_delete`) | D1-easy | ≥ 0.95 |
| H1-H1 | Soy koruma (`rephrase_*`, `move`, `split_edit`, `merge_edit` baskın parçalar) | **D1-hard** | ≥ 0.90 |
| H1-H2 | Yanlış eşleme | D1-hard | ≤ 0.03 |
| H1-H3 | Bölme/birleştirme olay F1 | D1-hard | ≥ 0.70 |
| H1-H4 | Tekrar-blok koruma (`dup_delete_adjacent`) | D1-hard | ≥ 0.90 |
| H1-R1 | Gerçek tarihçe, dokunulmamış blok koruma | D2 | ≥ 0.999 |
| H1-R2 | Gerçek tarihçe, genel koruma | D2 | ≥ 0.85 |
| H1-G | THETA×THETA_SPLIT ızgarasında farklı amaç değeri sayısı (geçerlilik kontrolü) | D1-hard ayar | ≥ 3 |
| H3-1 | Yazma→aranabilir p95 | D3 | < 1000 ms |
| H3-2 | Yeniden-chunk soy kaybı | D3 | = 0 |
| H4-1 | Olay günlüğü hash, D3 yeniden kurulum, eşleyici v3 | D3 | = manifest (v3) |
| H4-V2 | Eşleyici v2 bayrağıyla D3 `events_hash` = `e6945bbc2da9c47376e017e815d7e51b` **ve** v2 `h1s.json` kalite alanları birebir | D3, D1-v2 | = eşit |
| H4-2 | İki temiz koşturmada kalite metrikleri maks. göreli fark | tümü | = 0 |

H1-G bir hipotez değil, ölçümün geçerliliği şartıdır: ızgara yine düzse D1-hard yeterince zor değildir ve H1-H* sonuçları "kanıt" sayılmaz, "bilgi" sayılır.

## Karar kuralı
- H4-V2 geçmedi → v2 davranışı korunamamış; sonuçlar geçersiz, önce bu düzeltilir.
- H4-1 ∧ H4-2 geçmedi → determinizm bozuldu; sonuçlar geçersiz.
- H1-G geçmedi → H1-H* bilgi amaçlı; karar yalnız H1-S*, H1-R* ile verilir ve "zor set kanıtı eksik" notu düşülür.
- H1-S* ∧ H1-R* ∧ (H1-G geçtiyse H1-H*) ∧ H3 ∧ H4 geçti → **`01_SPEC.md` §1–7 (K4 ve v3 eşleyiciyle) dondurulur; çekirdek fazı 1 başlar.**
- H1-S2 veya H1-H2 kaldı, diğerleri geçti → eşleyici bir tur daha; diğer sonuçlar saklanır.
- Başka bir H1 alt ölçümü kaldı → veri modeli yeniden ele alınır.

## Yapılmayacaklar
H2 koşturması; graf, varlık, akış işleri; yeni primitifler (katman, terfi, kontrol noktası); sync; UI.
