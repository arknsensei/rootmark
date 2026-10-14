# P1-B — Varsayımlar

Etiketler: `[spec]` spesifikasyon yorumu, `[arch]` mimari/süreç yorumu, `[-]` yerel uygulama kararı.

- **B0-1 [arch] Dondurulmuş dosyaların tur görevince değiştirilmesi = yeniden dondurma satırı.** P1-B görevleri P1-A'da dondurulmuş dosyaları açıkça değiştirir (Ö1 `check_frozen.sh`, Ö2 `conformance.rs`, `test_tamper.sh`, `spec/fixtures/MANIFEST.json`, `verify.sh`). Ö1 "satır silinmez/gevşetilmez" dediği için eski satır yerinde kalır; aynı kalıpla yeni satır eklenir, `check_frozen.sh` sonraki satırı esas alır ve bir `FREEZE_EXCEPTIONS.md` girdisi (`P1-B/F<n>`) şart koşar. Alternatif (eski satırın commit önekini değiştirmek) Ö1'i ihlal ederdi.
- **B0-2 [-] Etkin tur.** `check_frozen.sh` tüm `docs/rounds/*/progress/ROUND_START` dosyalarını denetler (her biri için yalnız-ekleme); en az biri yoksa hata.
- **B0-3 [-] Fixture kökü yedeği.** `MDK_FIXTURES` yoksa `<cwd>/../spec/fixtures` (cargo testleri `core/` içinde koşturur); kök ve kaynağı (`fixtures_root_source`) rapora yazılır, `MANIFEST.json` yoksa hata — sessiz düşme değil.
