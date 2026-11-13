# R4 (c) — Güvenlik (2026-11-13)

- Dependabot: açık uyarı **0** (`gh api repos/{owner}/{repo}/dependabot/alerts?state=open`).
- CodeQL (code scanning): açık uyarı **0** (`…/code-scanning/alerts?state=open`).
- Secret scanning bu depoda kapalı (API 404; önceki turlardaki gibi; bu tur değiştirmedi).
- `npm audit --omit=dev`: `clients/obsidian` 0, `clients/vscode` 0.
- Bu tur bağımlılık eklemedi ve sürüm değiştirmedi (`Cargo.toml`/`Cargo.lock`'ta dış crate farkı yok; `check_dep_age` ve `check_deps` R5'te).
- Turun güvenlikle ilgili yüzü: yeni test ayarları (`ROOTMARK_TEST_ABORT_AT`, `ROOTMARK_WATCH_TEST_STOP_DELAY_MS`) yalnız ortam değişkeninden okunur, tanınmayan değer hatadır, verilmezse etkisizdir; bir saldırgan süreci ortam değişkeni verebiliyorsa sunucuyu zaten başlatıp durdurabilir (THREAT_MODEL'in yerel süreç sınırı). `index.old`/`index.build` `.rootmark/` altındadır (vault'un izinleriyle); başlangıçta yalnız bu iki sabit adı siler.
