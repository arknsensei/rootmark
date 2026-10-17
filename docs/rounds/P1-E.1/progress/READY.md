# P1-E.1 — doğrulama için hazır

- Commit: `2842f6e` (E1.5 TAMAMLANDI; resmi RUN'ın ölçtüğü commit `7ea12dc6543b5d0cc1fc6257aec4095735ff1543` `P1-E.1/E1.5-pre`, sonrası yalnız RUN kaydı, FINAL ve ilerleme kayıtları)
- Resmi RUN: `20261017T140538Z-7ea12dc` (`reports/rounds/P1-E.1/runs/20261017T140538Z-7ea12dc/`; çift: A `20261017T140538Z-7ea12dc`, B `20261017T143440Z-7ea12dc`, `verify.sh pair` çıkış 0, 103 189 alan fark 0, yapıtlar bayt eşit)
- FINAL: `reports/rounds/P1-E.1/FINAL.md` — **E1-1..E1-9 geçti**; önceki turların kaydedicilerinde 10 tarihsel FAIL (K-E1.4), sayılan 0. Karar kuralı: doğrulama "Güvenilir" derse insan adımı (`git tag v0.1.0`, `release.yml` tetikleme, `docs/RELEASE.md` kontrol listesi). **`v0.1.0` etiketi atılmadı, `release.yml` tetiklenmedi.**
- Gerçek Obsidian e2e (K-E1.3) yerelde koşar: `scripts/plugin_e2e_real.sh` (ayrı `--user-data-dir`, CDP; Obsidian yoksa "atlandı" kaydı, R15 FAIL).
- Açık karar talepleri: K-E2-T (Demote). K-E5-T K-E1.1 ile kapandı.
- Kapanan P1-E doğrulama koşulu ve önerileri: `status` sözleşme ihlali (K-E1.2, E1-1/E1-2/E1-8), R7 SIGPIPE (E1-4), gerçek Obsidian e2e kapısı (K-E1.3, E1-3/E1-9), yayım hedefi (K-E1.1, E1-5).
- Süreç notu: E1-6'nın 29 karar hariçi geliştirme RUN'ı (`20261017T132817Z-b77f291`) görüldükten sonra, resmi RUN'dan önce yazıldı (E1.5.md; emsal K-C1.6, P1-E E7).

Doğrulama yönergesi: `docs/rounds/P1-E.1/VERIFY.md`.
