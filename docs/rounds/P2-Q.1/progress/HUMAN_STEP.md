# P2-Q.1 — İnsan adımı

**Durum (2026-11-05, öğleden sonra): yayımın insan adımları tamamlandı.** PR #51 birleşti (`e74e1f8`), `release.yml` `41994456323` başarılı (`release.json`: zorunlu `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`, Windows da `ok`), `v0.3.1` `e74e1f8`'de etiketli, GitHub sürümü 12 dosyayla yayımlı. CodeQL açık uyarısı 0. Ajan R4 (d)'yi koşuyor (CI `pilot.yml` iki koşu, Docker, Mac, Obsidian sahnesi), sonra R5.

## Açık istekler (R5'ten önce)
1. **Tur dalının zorla itilmesi.** PR #51 tur dalının R0–R4 commit'lerini `main`'e rebase ile taşıdı; `p2-q.1/round` yerelde birleşmiş `main`'in (`e74e1f8`) üstüne yeniden temellendirildi (turun kendi dosyaları aynı; yeniden temellendirilmemiş hali `backup/p2-q.1-round-pre-rebase` yerel dalında). Ajanın zorla itmesi bu oturumda reddedildi. **Beklenen:** `git push --force-with-lease=p2-q.1/round:3284267 origin p2-q.1/round` (kullanıcı, ana çalışma dizininde) ya da bu itmeye izin. Turun PR'ı (R5) bundan sonra açılır.
2. **Kararın teyidi (tek satır).** K-P2Q1-T1 kararı (a) ve CodeQL #2–#4'ün kapatılması bu oturuma orkestratör oturumundan (`mdkernel-ce`) iletildi; `docs/DECISIONS.md` K-P2Q1.2 olarak "teyit bekleniyor" notuyla yazıldı. Kullanıcı bu oturumda "K-P2Q1.2 teyit" yazarsa not kaldırılır.
