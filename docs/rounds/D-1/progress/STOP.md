# D-1 — Durma kaydı

## 2026-11-14 · R6 PARK — R21 ile dondurulmuş E1-4 okumasının çelişkisi
- **Koşul:** CLAUDE.md "spec ile çelişki" sınıfı (dondurulmuş ölçüm kodu ve ön-kayıtlı envanter, R0'ın değişmesi beklenen kayıtlarında adıyla yoktu).
- **Gözlem:** PR #73'ün macOS işi `make test`'in pytest aşamasında iki kalış (R21'in `RELEASE.md` satırı → `test_p1e1_tools::test_release_ids_cover_r1_to_r16`, `test_p2v1_tools::test_the_mac_reading_…`); üçüncüsü (`test_transfer::test_host_home_and_private_terms`) aracın kendi hatası, düzeltildi (D1F-13).
- **Karar talebi:** `docs/DECISIONS.md` K-D1-T1 (seçenekler a, b, c; ajanın önerisi b).
- **Park edilen:** R6'nın PR #73 birleşmesi ve sonrası. Bağımsız işler: yok (R0–R5 tamam; R6'nın kalanı insan adımı ve bu kararı bekliyor).
- **Devam:** karar gelince ilgili seçenek uygulanır, yerelde tam `make test` + `make lint`, #73'e tek push.
