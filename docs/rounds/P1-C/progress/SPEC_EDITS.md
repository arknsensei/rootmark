# P1-C — Spec düzenlemeleri

Ölçüt: `docs/SPEC_EDITS_CRITERION.md` (K-B1.3): **açıklık** ⇔ (a) hesaplanan hiçbir değeri değiştirmez ve (b) daha önce uyumlu bir uygulamayı uyumsuz kılmaz. Sürüm kararı (K-C0.3): yalnız açıklık → spec sürümü değişmez (1.0.1); bölüm durumu spec'teki güncel damgadan (§9 "Draft").

| # | Görev | Yer | Düzenleme | (a) | (b) | Sınıf |
|---|---|---|---|---|---|---|
| S1 | C7 | §9, son madde | "Informative: …" — başvuru uygulamasının BM25 parametreleri ve istatistiklerin checkpoint'in uygun chunk'larından alınması, kesin kosinüs, RRF `1/(60 + rank)` ilk 50, `(doc_path, chunk_id)` eşitlik kırma, chunk katmanı (blokların checkpoint'teki en düşük katmanı), proposal (varsayılan dışarıda; istenirse etiketli, ×0.5, canonical'dan sonra), derived asla, arama başına bir iz | sağlar (bilgilendirici; hiçbir normatif değer değişmez) | sağlar (bilgilendirici not hiçbir uygulamayı bağlamaz; §9'un MUST'ları aynı) | **açıklık** |

Uyumluluk: `spec/CONFORMANCE.md` Seviye 3 satırları (C3-01..C3-04) değişmedi; fixture'lar `spec/fixtures/C3-0x/` (C7, P1-C/F5). C2-09 satırı K-C0.2 kararıyla eklendi (C0; spec metni değil).
