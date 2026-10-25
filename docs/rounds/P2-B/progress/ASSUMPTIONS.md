# P2-B — Varsayımlar

Etiketler: `[spec]` spesifikasyonu ilgilendirir, `[arch]` mimariyi, `[-]` yalnız tur içi uygulama. Hiçbiri spec §1–6 ya da GOAL ile çelişmez; çelişki görülseydi görev park edilirdi.

## B0
- **B0-1 [-]** `ROUND_START` = `6a5e911` (Blok 1 kapanışı, PR #27'nin birleşmiş `main` HEAD'i).
- **B0-2 [-]** Uygulama orkestratörün oturumunda koşar (`claude --bg` izni reddedildi); DEV_PROCESS §2 disiplini aynen (K-P2B.2).
- **B0-3 [-]** (b) "`required: true` görünmez": önerilen göçün planından `required` anahtarı kaldırılır (`false` yazmak onu bir doğrulama göçü gibi gösterirdi; doğrulama planları `required: false` taşır). Öteki planlar ve `status.migrations` bayt eşit.
- **B0-4 [-]** (b) 0.2.0'a geri dönüş satırı `v0.2.0` etiketinin kodu okunarak yazıldı: 0.2.0'ın tarif okuyucusu bilinmeyen alanı reddeder (`[[promotion.rules]]`, `[promotion] version`), `policy` aktörlü `Promoted`'ı ve `Promoted`/`Demoted`'ın `meta.source`'unu reddetmez (`check_event_actor` geçişleri atlar, `check_builtin_policy` `rules`'u tanımaz ama reddetmez).
- **B0-5 [arch]** (c8) GOAL'ün iki seçeneğinden "tanımdan çıkar" seçildi: okunamayan tarif matrisin hatasıdır (`status.migrations.error`; `rootmark migrate --dry-run` çıkış 1). `status` okunamayan tarifte zaten `policy_violation` döner (tarif yükleyicisi matristen önce koşar); hata `migrate --dry-run` ve `migrate::matrix` ile görünür.
- **B0-6 [-]** (c11) Test iki politikada (`unverified`, `canonical`) koşar: dış paragraf izleyicinin katmanını alır (`unverified` / `canonical`), hiçbir dış soy `promoted`'da değildir, ajanın iki yeni soyu terfi eder.
