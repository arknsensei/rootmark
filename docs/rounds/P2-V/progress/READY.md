# P2-V — READY

**doğrulama için hazır**

- Dal `p2-v/round`; PR #35 (`main`'e tek PR). Turun son kod/belge commit'i **`0cb08cf`** (P2-V/V6); ölçülen commit **`43055ee`** (P2-V/V3; sonrası istisnalı araç düzeltmeleri F3–F7 ve kayıt/belge). Bu READY commit'i yalnız kayıt.
- **Karar kuralı: tur KALDI** (V-3, V-4; V-6 ve V-7 değerleriyle) — eski zincir yürürlükte kalır; karar talepleri K-P2V-T1…T4 (`reports/rounds/P2-V/FINAL.md`).
- **Kademe 2 CI koşuları:** V4a zincirler **40613401174** → karşılaştırma **40633593589** (D eşit, 788 021 alan); V4b zincirler **40613409396** → karşılaştırma **40633603941** (D farkı: işlemciye bağlı vektör sayısalları); eski zincirin kademe 2 iş akışıyla koşusu (V0m, `352f790`) **40613392932** → **40633614107**. Eski zincirin referansı (V0, eski `verify.yml`) **40582139980**.
- **Kademe 1:** PR #35'in koşusu **40638224562** (yeşil, 2 068 s), dal koşusu 40613417655.
- **Mac kümesi:** RUN `20261028T013053Z-43055ee` (`reports/rounds/P2-V/mac/`).
- Envanter `reports/rounds/P2-V/inventory.json` (ön-kayıt, `ae65686`).
- Doğrulayıcı istemi: `docs/rounds/P2-V/PROMPT.md` (ikinci istem; doğrulanan commit PR birleştikten sonraki `main`).
