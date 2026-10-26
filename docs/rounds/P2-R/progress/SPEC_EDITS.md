# P2-R — Spec düzenlemeleri (docs/SPEC_EDITS_CRITERION.md)

| # | Bölüm | Düzenleme | (a) | (b) | Sınıf |
|---|---|---|---|---|---|
| E1 | §8 "Consumption trace" | Bilgilendirici not (P2-R/R2; GOAL P2-R 3): "append-only" bir satırın nasıl yazıldığını sınırlar; başvuru uygulaması operatörün açık isteğiyle saklama sınırının gerisindeki en eski satırları bayt bayt, aynı sırayla `.rootmark/` içindeki, kendisinin silmediği bir arşive taşıyabilir; `at_seq` mutlaktır (ilk canlı satırın `at_seq`'i kaydedilir, yeni iz sayımı sürdürür), hiçbir `trace_id` değişmez, canlı satırlar arşivden sonra birleştirilince eski dosyadır; dosyası arşivlenmiş checkpoint'e sorgu reddedilir, başka türlü yanıtlanmaz. | sağlar | sağlar | açıklık (bilgilendirici not) |

**Sürüm:** yalnız açıklık satırı → sürüm değişmez, **2.0.3** (CRITERION "Sürüm numarası", ilk madde; "Frozen bir bölüme eklenen bilgilendirici not açıklıktır" — §8 Validated). GOAL P2-R 3 ve K-P2R.1 "(CRITERION ile, 2.0.4)" der; DEV_PROCESS §1 (K-D1.9) gereği sürümü GOAL değil CRITERION belirler ve bir bilgilendirici not sürüm artırmaz (ASSUMPTIONS R2-1). §1–6 ve öteki bölümler değişmedi; durum damgası değişmedi (§8'in damgası Blok 2 doğrulamasından sonra, DEV_PROCESS §5).
