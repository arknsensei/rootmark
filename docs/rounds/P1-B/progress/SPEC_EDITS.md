# P1-B — spec/SPEC.md düzeltmeleri (§7, §8, §11; yalnız açıklık)

Kural (GOAL P1-B, PROMPT): §7/§8/§11 metni uygulamayla hizalanırken yalnız açıklık için değişir; her değişiklik burada satır satır. Anlam değişikliği gerektiren konular düzeltme yapılmadan `docs/DECISIONS.md`'ye karar talebi olarak yazıldı (K17-T…K20-T). §1–6 değişmedi.

| # | Bölüm / satır | Önce | Sonra | Neden yalnız açıklık |
|---|---|---|---|---|
| E1 | §7, satır 89 | ``Every version has `layer ∈ {canonical, proposal, derived}`. Default on ingest:`` | ``Every version has `layer ∈ {canonical, proposal, derived}` (serialized in lowercase, as written here). Default on ingest:`` | Değer kümesi aynı; yalnız yazım biçimi belirtildi (K15-T "Layer küçük harf, tutarlılık P1-B'de"). §2.2/§4.3 tür adlarının PascalCase yazımı (K15-T) **değiştirilmedi** (§1–6 donmuş). |
| E2 | §8, satır 98 | ``returned: [(doc_path, lineage_id, version_id)], at_seq}`. `trace_id = H(checkpoint_id + consumer + query_hash + at_seq)`.`` | ``returned: [(doc_path, lineage_id, version_id)], at_seq}`; each `returned` entry is serialized as an object with the keys `doc_path`, `lineage_id`, `version_id`. `trace_id = H(checkpoint_id + consumer + query_hash + at_seq)`, where `+` is plain string concatenation and `at_seq` is written in decimal.`` | Demet gösteriminin JSON yazımı (K15-T'nin "returned demet ↔ nesne" sorusu) ve formüldeki `+`'nın düz birleştirme olduğu (formül metni aynen; §3'teki diğer formüller ayırıcıyı açıkça yazar, bu yazmaz) belirtildi; hesaplanan değer metnin harfi harfine okumasıyla aynı. |
| E3 | §11, satır 113 | ``policies supply parameters within those bounds.`` | ``policies supply parameters within those bounds, and a parameter outside them is rejected, not clamped. (Informative: the reference implementation's minimum journal retention is 24 hours.)`` | "Parametreler sınırlar içinde" ifadesinin karşılığı: sınır dışı parametre geçerli bir tarif değildir — ARCHITECTURE §4.4 ("çekirdek sınırları içinde parametrik tarif; sınır dışı → ret", kaynak sırasında spec'ten sonra ama bu konuda tek açık hüküm) ve GOAL P1-B ile aynı. 24 saat bilgilendirici (normatif değil) notla. |

Damga: tur sonunda karar kuralı sağlanırsa (B9) §7, §8, §11 başlıkları "(Draft)" → "(Validated in P1-B)" ve durum satırı buna göre güncellenir; bu değişiklik de aşağıya E4 olarak eklenir.

## Düzeltme yerine karar talebi (anlam taşıyan yorumlar)
- K17-T — ingest'in katmanı yalnız düşürmesi, devralınan soy için `LayerChanged` (ASSUMPTIONS B1-1).
- K18-T — `event_seq` = içerik ofseti; `at_seq` = iz satır sırası (B3-1, B4-1).
- K19-T — budama = `meta.prune` taşıyan `Demoted`; indirme hedefleri (B2-4).
- K20-T — `d1hs` kaynak korpusu (ASSUMPTIONS B8-1).
