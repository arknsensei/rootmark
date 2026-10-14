# P1-B — spec/SPEC.md düzeltmeleri (§7, §8, §11; yalnız açıklık)

Kural (GOAL P1-B, PROMPT): §7/§8/§11 metni uygulamayla hizalanırken yalnız açıklık için değişir; her değişiklik burada satır satır. Anlam değişikliği gerektiren konular düzeltme yapılmadan `docs/DECISIONS.md`'ye karar talebi olarak yazıldı (K17-T…K22-T). §1–6 değişmedi.

| # | Bölüm / satır | Önce | Sonra | Neden yalnız açıklık |
|---|---|---|---|---|
| E1 | §7, satır 89 | ``Every version has `layer ∈ {canonical, proposal, derived}`. Default on ingest:`` | ``Every version has `layer ∈ {canonical, proposal, derived}` (serialized in lowercase, as written here). Default on ingest:`` | Değer kümesi aynı; yalnız yazım biçimi belirtildi (K15-T "Layer küçük harf, tutarlılık P1-B'de"). §2.2/§4.3 tür adlarının PascalCase yazımı (K15-T) **değiştirilmedi** (§1–6 donmuş). |
| E2 | §8, satır 98 | ``returned: [(doc_path, lineage_id, version_id)], at_seq}`. `trace_id = H(checkpoint_id + consumer + query_hash + at_seq)`.`` | ``returned: [(doc_path, lineage_id, version_id)], at_seq}`; each `returned` entry is serialized as an object with the keys `doc_path`, `lineage_id`, `version_id`. `trace_id = H(checkpoint_id + consumer + query_hash + at_seq)`, where `+` is plain string concatenation and `at_seq` is written in decimal.`` | Demet gösteriminin JSON yazımı (K15-T'nin "returned demet ↔ nesne" sorusu) ve formüldeki `+`'nın düz birleştirme olduğu (formül metni aynen; §3'teki diğer formüller ayırıcıyı açıkça yazar, bu yazmaz) belirtildi; hesaplanan değer metnin harfi harfine okumasıyla aynı. |
| E3 | §11, satır 113 | ``policies supply parameters within those bounds.`` | ``policies supply parameters within those bounds. (Informative: the reference implementation's minimum journal retention is 24 hours.)`` | Yalnız bilgilendirici (normatif olmayan) not; bağlayıcı metin aynen. İlk sürümdeki "a parameter outside them is rejected, not clamped" cümlesi B7 gözden geçirmesinde anlam değişikliği sayıldı (sınıra kırpan bir uygulamayı uyumsuz kılardı) ve geri alındı → K21-T. |

| E4 | durum satırı (satır 3); §7 (87), §8 (95), §11 (111) başlıkları | `Draft for §7–§11 (to be validated in Phase 1).`; `(Draft)` ×3 | `Validated for §7, §8, §11 (Phase 1 round P1-B, Oct 2026; conformance Level 2); Draft for §9–§10 (to be validated in Phase 1).`; `(Validated, P1-B)` ×3 | GOAL P1-B "tur sonunda bu bölümler Draft → Validated damgası alır" ve karar kuralı (B-1..B-8 ∧ B-10 geçti; resmi RUN `20261014T135400Z-7735a07`, FINAL.md). Metin içeriği değişmedi; açık karar talepleri K17-T…K22-T damgadan bağımsız olarak açık. |

## Düzeltme yerine karar talebi (anlam taşıyan yorumlar)
- K17-T — ingest'in katmanı yalnız düşürmesi, devralınan soy için `LayerChanged` (ASSUMPTIONS B1-1).
- K18-T — `event_seq` = içerik ofseti; `at_seq` = iz satır sırası (B3-1, B4-1).
- K19-T — budama = `meta.prune` taşıyan `Demoted`; indirme hedefleri (B2-4).
- K20-T — `d1hs` kaynak korpusu (ASSUMPTIONS B8-1).
- K21-T — sınır dışı politika parametresi: ret mi, kırpma mı (B7 gözden geçirmesi; E3'ten geri alındı).
- K22-T — `trace_id` düz birleştirmesinde çakışma (B7 gözden geçirmesi).
