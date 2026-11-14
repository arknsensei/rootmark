# D-1 — Durma kaydı

## 2026-11-14 · R6 PARK — R21 ile dondurulmuş E1-4 okumasının çelişkisi
- **Koşul:** CLAUDE.md "spec ile çelişki" sınıfı (dondurulmuş ölçüm kodu ve ön-kayıtlı envanter, R0'ın değişmesi beklenen kayıtlarında adıyla yoktu).
- **Gözlem:** PR #73'ün macOS işi `make test`'in pytest aşamasında iki kalış (R21'in `RELEASE.md` satırı → `test_p1e1_tools::test_release_ids_cover_r1_to_r16`, `test_p2v1_tools::test_the_mac_reading_…`); üçüncüsü (`test_transfer::test_host_home_and_private_terms`) aracın kendi hatası, düzeltildi (D1F-13).
- **Karar talebi:** `docs/DECISIONS.md` K-D1-T1 (seçenekler a, b, c; ajanın önerisi b).
- **Park edilen:** R6'nın PR #73 birleşmesi ve sonrası. Bağımsız işler: yok (R0–R5 tamam; R6'nın kalanı insan adımı ve bu kararı bekliyor).
- **Devam:** karar gelince ilgili seçenek uygulanır, yerelde tam `make test` + `make lint`, #73'e tek push.
- **Kapanış (2026-11-14):** K-D1.2 (kullanıcı kararı, seçenek b): R21 ayrı kapı (`scripts/release_gate_claude.sh`), `release_check.sh` F1 öncesi baytlarına döndü (D-1/F2); D1-10 lafzen kaldı (FINAL). R6 PARK'tan çıktı.

## 2026-11-14 · R6 PARK (3a öncesi) — `v0.4.0` kendi listeleriyle aktarılamıyor
- **Koşul:** dışarıya yazan adım (3a) öncesi yerel yarıda engel; çözüm etiketi (insanın) ya da aktarım yolunu değiştirir → karar talebi.
- **Gözlem:** `v0.4.0`'dan iki yerel koşu + CI benzetimi: ağaç aynı (623553b9…), sızıntı 0, ama sınıflandırılmamış 1 (`scripts/release_gate_claude.sh`, K-D1.2'nin eklediği) → `ok: false`; `transfer.yml` ürün deposuna yazmadan kalırdı.
- **Karar talebi:** `docs/DECISIONS.md` K-D1-T2 (a, b, c; ajanın önerisi a).
- **Park edilen:** 3a ve sonrası. Bağımsız iş: düzeltme dalı `release/0.4.0-fix` ve yerel tam kapılar (PR karar gelince).
- **Kapanış (2026-11-15):** K-D1.3 (kullanıcı kararı, seçenek a): düzeltme PR'ı (`release/0.4.0-fix`) ve laboratuvar `v0.4.0` etiketinin yeniden atılması; sıra HUMAN_STEP adım 2′. R6 PARK'tan çıktı.

## 2026-11-15 · R6 PARK (3b sonrası, 3f öncesi) — ürün `release.yml` geçersiz YAML
- **Koşul:** düzeltmenin ürüne giden yolu dışa yazan, kısmen yıkıcı insan adımları ve D0-K5/D1-2 sapması içeriyor → karar talebi.
- **Gözlem:** D1F-17 (kaplama `release.yml:296` tırnaksız ad; actionlint'te tek YAML hatası); düzeltme ve kalıcı test `d-1/round`'da.
- **Karar talebi:** `docs/DECISIONS.md` K-D1-T3 (a, b, c; ajanın önerisi b).
- **Park edilen:** 3c'nin dal koruması, 3d–3h.
