# P2-Q.4 — Görevler

Genel kurallar:
- **Sıra:** R0 → R1 → R2 → R3 → R4 → R5. R0'ın `DESIGN.md` ve `TESTS.md`'si commit'lenmeden ürün kodu yazılmaz. R2 ve R3, R1'i beklerken yürüyebilir (farklı yollar), ama commit'ler görev sırasıyla.
- **Kayıt ve dallar:** progress `docs/rounds/P2-Q.4/progress/`. Commit öneki `P2-Q.4/R<n>:`. Dal `p2-q.4/round`, yalnız birleşmiş `main`'den (K-P2.2). Yayım hazırlığı ayrı dal `release/0.3.4`.
- **Gözden geçirici:** yama turu, araç dozu (K-P2.4): yalnız R5'te, tek tur.
- **Testler:** yeni testler yeni dosyalarda (`server/tests/`, gerekirse `index/tests/`). Test düzeneği değişikliği (port, SIGKILL yolu) iddiaları gevşetmez; değişen her düzenek satırı R0'da adıyla. Dondurulmuş dosyalar: DEV_PROCESS §4.
- **Yük altında tekrar:** yerel çok tekrarlı koşular (≥ 200) eşzamanlı yükle (ör. ikinci bir `cargo test --workspace` ya da `stress`) yapılır; yük ve makine `progress/` kaydında. CI'da tekrar 20.
- **CI'ın son hali:** READY'den önce ve sonra PR'ın son commit'inin CI'ı `gh pr checks <n>` ile okunur. Kalış varsa koşu kimliğiyle FINDINGS ve FINAL'e yazılır; "yeniden koşunca geçti" kaydı silmez.
- **CI yapılandırması:** `.github/workflows/**` değişirse R0'ın listesine önceden adıyla (K-P2Q1.4).
- **İnsan adımı ve sorular:** `HUMAN_STEP.md`; soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye; spec'in tanımlamadığı durumda PARK + karar talebi.
- **Vault:** yalnız sentetik kasa ya da D-NOTES kopyası (depo dışında, scratchpad). Kişisel vault ve qorbit'e dokunulmaz. Kayıtlarda token, içerik ve host değeri yok.
- **Push:** 403 ("denied to rootmark-ai") verirse `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push …`; sürerse `HUMAN_STEP.md`.

## R0 — Açılış, yeniden üretim, tasarım
**Yapar:**
- `ROUND_START`; DECISIONS K-P2Q4.1 (açılış).
- **PQ3-9'un yeniden üretimi 0.3.3 ile:** `watch_restart.rs`'in A kümesi (`a1`/`a2`, iki politika) ve C1, macOS'ta yük altında ≥ 200, Linux'ta (Docker) ≥ 200 tekrar. Kalan tekrarlarda olay sırası kaydı: izleyicinin aldığı ham olaylar (yol, tür, zaman sırası), uzlaştırmanın kararları, bekleme zamanlayıcıları (içerik yok). Kayıt `reports/rounds/P2-Q.4/r0/`, özet `progress/R0_repro.md`.
- **PQ3-7:** enjeksiyonla belirlenimli üretim tasarımı (R2'de ürün koduyla gelir) ya da P2-Q.3'ün `r5` kaydı ve kod okumasıyla gerekçe.
- `progress/DESIGN.md`:
  - PQ3-9'un kök nedeni (kanıtla) ve düzeltmesi; nedenin doğrudan testi;
  - `install`'ın yeni sırası (ör. yeni dizini yan yana yazıp eskisini yedek adla `rename`, sonra yeniyi `rename`, en son yedeği silmek; ya da manifest'in en son yazılması) ve başlangıçta eksik dizinin denetimi (`start_index`: hangi dosyalar, hangi ileti, `auto_rebuild`);
  - enjeksiyon değişkeni: adı, okunduğu yerler, ürün varsayılanında etkisizliği;
  - PQ3-6: üç yer ve not satırı;
  - düzenek: port (stderr'den), C1'in zorunlu SIGKILL yolu;
  - ön-kayıtlı testler.
- `progress/TESTS.md`; kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi (sürüm dizgileri 0.3.4, E1-4/R14).

**Kabul:** Q4-1.

## R1 — PQ3-9
**Yapar:** DESIGN'daki düzeltme; nedeni hedefleyen yeni test; A ve C kümeleri yerelde yük altında macOS ≥ 200, Linux ≥ 200. `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs`, `watch_restart.rs` aynen geçer.

**Kabul:** Q4-2.

## R2 — PQ3-7
**Yapar:** `index/src/store.rs` `install` ve başlangıç denetimi; enjeksiyon testleri (ör. `server/tests/index_install_kill.rs`): her enjeksiyon noktasında ≥ 20 tekrar, iki platform; dizin baytlarının temiz kurulumla eşitliği; eksik dizin iletisi.

**Kabul:** Q4-3.

## R3 — PQ3-6, PQ3-8, PQ3-4
**Yapar:** `NotFound` (üç yer) ve izinsiz dizin testi; `restartkit`/`watchkit` portu; C1'in zorunlu SIGKILL yolu iki platformda. `progress/R3.md`.

**Kabul:** Q4-4; Q4-5.

## R4 — Belgeler, güvenlik ve 0.3.4
**Yapar:**
- (a) Kılavuz; (b) `CHANGELOG.md` `## 0.3.4`; (c) `progress/R4_security.md`.
- (d) Kademe 2 bir tetikleme (`gh workflow run verify.yml --ref p2-q.4/round -f sha=<sha>`); `make verify-mac` bir kez (önce `fseventsd` sağlığı).
- (e) `release/0.3.4`, `release_check` (R1–R20), PR; `HUMAN_STEP.md` (birleşme, `release.yml`, etiket, sürüm).
- (f) Yayımlanmış 0.3.4 ile: R1 ve R2'nin durumları iki platformda kısa tekrarla; gerçek Claude Code oturum sonu sınaması ≥ 3 oturum (`claude -p --mcp-config … --strict-mcp-config`, D-NOTES kopyası, sürüm kayıtlı) → `progress/R4_claude_code.md`; D-NOTES senaryosu Mac, Docker ve CI'da (`pilot.yml`), 0.3.3 kayıtlarıyla karşılaştırma.

**Kabul:** Q4-6, Q4-7, Q4-8'in D-NOTES yarısı, Q4-9, Q4-10.

## R5 — Rapor, gözden geçirici, hazır
**Yapar:**
- Kapılar: `make test` iki kez, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- `FINDINGS.md`: PQ3-9, PQ3-7, PQ3-6, PQ3-8, PQ3-4; yeni bulgular.
- Gözden geçirici tek tur (GOAL başlığı).
- `reports/rounds/P2-Q.4/FINAL.md`: Q4-1..Q4-11, karar kuralının dalı, koşu kimlikleri, "koştu/geçti".
- READY; tek PR: `gh pr create --base main --head p2-q.4/round --title "P2-Q.4: öldürülünce kaçan kaldırma, atomik dizin ve 0.3.4" --body-file reports/rounds/P2-Q.4/FINAL.md`. PR'ın CI'ı bitince son commit'in sonucu READY'ye; READY'den sonraki her commit'in CI'ı da okunur.

**Kabul:** Q4-8, Q4-11; READY'de "doğrulama için hazır" + commit sha + PR'ın son commit'inin CI koşu kimliği.
