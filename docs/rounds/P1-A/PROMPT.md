# P1-A — `/goal` İstemleri

## Ana ajan
```
Bu depo mdkernel monoreposudur. Dış sohbet geçmişi yoktur; tek gerçek kaynak depodaki belgelerdir.
Önce CLAUDE.md, sonra NORTH_STAR.md, spec/SPEC.md (§1–6 ve §12), spec/CONFORMANCE.md (Level 1), docs/ARCHITECTURE.md, docs/PACKAGES.md, docs/DEV_PROCESS.md'yi oku.
Aktif tur P1-A: docs/rounds/P1-A/GOAL.md ve TASKS.md. Görevleri A0'dan A7'ye sırayla yürüt; park kuralı dışında atlama yok.
Her görevde DEV_PROCESS §2 disiplini: progress kaydı, kabul ölçütleri tek tek, gözden geçirici alt-ajan, görev başına commit (P1-A/A<n>: ...).
Kırmızı çizgiler: GOAL.md çıkış kriterleri değişmez; spec §1–6 ve fixture'lar değişmez; MatcherVersion::V2 ve V3 çıktıları v3-final ile bayt bayt aynı kalır; çalışma zamanı kodu yalnız Rust; bağımlılık yönü core ← index ← server; sessiz başarısızlık yok; H2 koşturulmaz; API çağrısı yok.
Çıkış kriteri tutmuyorsa optimize etme, raporla, devam et. Durma koşulunda STOP.md yaz, görevi PARK et; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.
A7 bittiğinde docs/rounds/P1-A/progress/READY.md'ye "doğrulama için hazır", commit sha ve RUN kimliğini yaz, dur. Doğrulamayı sen yapma.
```

## Doğrulama ajanı (ayrı oturum, temiz klon, ana depo yolu verilmez)
```
Sen bağımsız doğrulama ajanısın; geliştirme oturumunun bağlamına ve ana depo dizinine erişimin yok, varsa bile kullanma.
docs/DEV_PROCESS.md §5 ve docs/rounds/P1-A/VERIFY.md'yi oku; V1'den V9'a sırayla uygula; scripts/verify.sh'i iki ayrı dizinde koştur. Kod değiştirme.
Tek çıktın reports/rounds/P1-A/VERIFICATION.md.
```
