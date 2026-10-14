# P1-B — `/goal` İstemleri

## Ana ajan
```
Bu depo mdkernel monoreposudur; tek gerçek kaynak depodaki belgelerdir. P1-A tamamlandı ve doğrulandı (reports/rounds/P1-A/). Aktif tur P1-B.
Önce CLAUDE.md, NORTH_STAR.md, spec/SPEC.md (§3, §7, §8, §11, §12), spec/CONFORMANCE.md (Level 2), docs/ARCHITECTURE.md §4.1 ve §5, docs/PACKAGES.md, docs/DEV_PROCESS.md, core/CLAUDE.md'yi oku.
Sonra docs/rounds/P1-B/GOAL.md ve TASKS.md. Görevleri B0'dan B9'a sırayla yürüt; park kuralı dışında atlama yok. B0'da docs/rounds/P1-B/progress/ROUND_START dosyasına tur başlangıç commit sha'sını yaz.
Her görevde DEV_PROCESS §2 disiplini: progress kaydı, kabul ölçütleri tek tek, gözden geçirici alt-ajan, görev başına commit (P1-B/B<n>: ...).
Kırmızı çizgiler: GOAL.md çıkış kriterleri değişmez; spec §1–6 ve C1 fixture'ları değişmez; spec §7/8/11 yalnız açıklık için ve SPEC_EDITS.md'ye satır satır kaydederek değişir, anlam değişikliği gerekiyorsa park et ve DECISIONS.md'ye karar talebi yaz; MatcherVersion::V2/V3 çıktıları ve eşleyici sabitleri değişmez; v3-final vault'ları açılmaya devam eder; çalışma zamanı yalnız Rust; bağımlılık yönü core ← index ← server; sessiz başarısızlık ve sessiz varsayılana düşme yok; arama/MCP/istemci kodu yok; API çağrısı yok.
Çıkış kriteri tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md yaz, görevi PARK et; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.
B9 bittiğinde docs/rounds/P1-B/progress/READY.md'ye "doğrulama için hazır", commit sha ve RUN kimliğini yaz, dur. Doğrulamayı sen yapma.
```

## Doğrulama ajanı (ayrı oturum, temiz klon, ana depo yolu verilmez)
```
Sen bağımsız doğrulama ajanısın; geliştirme oturumunun bağlamına ve ana depo dizinine erişimin yok, varsa bile kullanma.
docs/DEV_PROCESS.md §5 ve docs/rounds/P1-B/VERIFY.md'yi oku; V1'den V11'e sırayla uygula; scripts/verify.sh'i iki ayrı dizinde koştur. Kod değiştirme.
Tek çıktın reports/rounds/P1-B/VERIFICATION.md.
```
