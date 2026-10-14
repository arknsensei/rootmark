# P1-B.1 — spec/SPEC.md düzenlemeleri (1.0 → 1.0.1)

Ölçüt: `docs/SPEC_EDITS_CRITERION.md` (K-B1.3). §1–6 metni değişmedi (`git diff p1b-final -- spec/SPEC.md` yalnız aşağıdaki satırlar).

| # | Bölüm / satır | Önce | Sonra | (a) değer | (b) uyum | Sınıf |
|---|---|---|---|---|---|---|
| S1 | başlık (1) | `# mdkernel Specification v1.0` | `# mdkernel Specification v1.0.1` | sağlar | sağlar | sürüm (K-B1.2 gereği yama sürümü) |
| S2 | durum satırı (3) | … `Draft for §9–§10 (to be validated in Phase 1).**` | aynı + ` Version 1.0.1 (Phase 1 round P1-B.1): §7 undeclared synthetic → canonical; §8 trace_id with separators and input validation; §7/§8 serialization normative. §1–§6 are unchanged.` | sağlar | sağlar | açıklık (sürüm notu) |
| S3 | §7 (89) | ``(serialized in lowercase, as written here)`` | ``; layers MUST be serialized in lowercase, as written here`` | sağlar | **sağlamaz** | serileştirme kararı — K-B1.3 (P1-B E1 normatif) |
| S4 | §7 (89) | `` `synthetic` → as declared.`` | `` `synthetic` → as declared, and `canonical` if no layer is declared.`` | sağlar (başvuru uygulaması zaten canonical) | **sağlamaz** (bildirimsiz synthetic'e başka katman veren uygulama önceki metinle uyumlu sayılabilirdi) | anlam değişikliği — K-B1.4 |
| S5 | §8 (98) | ``each `returned` entry is serialized as an object`` | ``each `returned` entry MUST be serialized as an object`` | sağlar | **sağlamaz** | serileştirme kararı — K-B1.3 (P1-B E2 normatif) |
| S6 | §8 (98) | ``trace_id = H(checkpoint_id + consumer + query_hash + at_seq)`, where `+` is plain string concatenation and `at_seq` is written in decimal.`` | ``trace_id = H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" + at_seq)`, where `at_seq` is written in decimal. `query_hash` MUST be exactly 32 lowercase hexadecimal characters and `consumer` MUST NOT contain `"\n"`; a trace violating either MUST be rejected.`` | **sağlamaz** (her `trace_id` değişir; C2-07) | **sağlamaz** | anlam değişikliği — K-B1.2 (C2-07 yeniden üretildi) |

Not: S6'daki "lowercase" ASSUMPTIONS B12-1'de gerekçelendirildi (§3 kimlikleri tek yazımlı küçük harf hex).
