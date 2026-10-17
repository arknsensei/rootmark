# P2-A — iskelet (P1-E/E6)

Bu dizin yalnız faz 2 planlamasının yeridir; tur yönergeleri (`GOAL.md`, `TASKS.md`, `VERIFY.md`, `PROMPT.md`) P1-E'nin karar kuralı ve doğrulaması sonrasında yazılır. ROADMAP P2-A: blok içi CRDT, bloklar arası nesne alışverişi (sync), tenant = vault + anahtar + politika.

Planlamaya girdi olarak açık kayıtlar (yalnız işaret; içerik `docs/DECISIONS.md`'de):
- K-E5-T — yayım ikilileri: Intel macOS (sabitlenmiş `ort`'ta hazır onnxruntime yok) ve Windows (CI tetiklenmedi).
- K-E2-T — sözleşmede `demote` aracı yok (istemcide Demote).
- K14-T, K15-T, K17-T, K18-T, K19-T, K20-T, K21-T — P1-A/P1-B'den açık spec talepleri.
- `docs/THREAT_MODEL.md` "Bilinen açık sınırlar" (TOCTOU, token = insan varsayımı, dondurulmuş değerlendiricilerin geçici yolları).
- P1-E doğrulamasının önerileri (`reports/rounds/P1-E/VERIFICATION.md`, yazıldığında).
