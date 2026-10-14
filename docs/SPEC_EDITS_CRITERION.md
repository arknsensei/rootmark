# Spec düzenlemeleri — "yalnız açıklık" ölçütü (K-B1.3)

`spec/SPEC.md`'de bir düzenleme ancak aşağıdaki iki koşulu **birlikte** sağlıyorsa **yalnız açıklık** sayılır:

- **(a)** Hesaplanan hiçbir değeri değiştirmez (kimlikler, hash'ler, olay baytları, sorgu sonuçları; spec'in harfi harfine okunmasıyla hesaplanan değer aynı kalır).
- **(b)** Daha önce uyumlu olan hiçbir uygulamayı uyumsuz kılmaz (düzenlemeden önceki metne göre geçerli olan bir yazım, serileştirme ya da davranış seçeneği düzenlemeden sonra da geçerlidir).

İkisinden biri sağlanmıyorsa düzenleme **anlam değişikliği** ya da **serileştirme kararı**dır:
- karar kaydı (`docs/DECISIONS.md`, `K…`) gerekir,
- spec sürümü artar (yama: hesaplanan değeri değişmeyen ya da yalnız Validated/Draft bölümleri etkileyen; majör: §1–6 ya da kırıcı değişiklik — docs/DEV_PROCESS.md §8),
- etkilenen uyumluluk fixture'ları yalnız o kararla yeniden üretilir.

Bilgilendirici not ("Informative: …") hiçbir uygulamayı bağlamadığı için (a) ve (b)'yi sağlar.

## Uygulama
Her tur, spec düzenlemelerini `docs/rounds/<tur>/progress/SPEC_EDITS.md` tablosunda satır satır kaydeder ve her satırı bu ölçüte göre sınıflandırır: **açıklık** / **serileştirme kararı** / **anlam değişikliği** (karar numarasıyla).

## Yeniden sınıflandırma (P1-B, K-B1.3)
| P1-B düzenlemesi | (a) | (b) | Sınıf |
|---|---|---|---|
| E1 — katman değerleri küçük harf | sağlar | **sağlamaz** (PascalCase katman yazan bir uygulama önceki metinle uyumluydu) | serileştirme kararı → K-B1.3, spec 1.0.1 normatif |
| E2 — `returned` nesne biçimi; `+` düz birleştirme; `at_seq` ondalık | sağlar | **sağlamaz** (dizi biçiminde demet yazan bir uygulama önceki metinle uyumluydu) | serileştirme kararı → K-B1.3, spec 1.0.1 normatif; formülün kendisi K-B1.2 ile değişti |
| E3 — §11 bilgilendirici saklama notu | sağlar | sağlar | açıklık |
| E4 — durum damgası (Draft → Validated) | sağlar | sağlar | açıklık (karar kuralına bağlı damga) |
