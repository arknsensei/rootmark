# P2-Q.3 — Görevler

Genel kurallar:
- **Sıra:** R0 → R1 → R2 → R3 → R4 → R5. R1 ve R2 aynı alım yolunu paylaşır, sırayla yapılır. R3 bağımsızdır, R1/R2'yi beklerken yürüyebilir. R0'ın `DESIGN.md` ve `TESTS.md`'si commit'lenmeden R1/R2'nin kodu yazılmaz.
- **Kayıt ve dallar:** progress `docs/rounds/P2-Q.3/progress/`. Commit öneki `P2-Q.3/R<n>:`. Dal `p2-q.3/round`, yalnız birleşmiş `main`'den (K-P2.2). Yayım hazırlığı ayrı dal `release/0.3.3`.
- **Gözden geçirici:** yama turu, araç dozu (K-P2.4): yalnız R5'te, tek tur.
- **Testler:** yeni testler yeni dosyalarda (`server/tests/`). Dondurulmuş test ve kaydedicilere dokunulmaz; değişmesi gerekirse DEV_PROCESS §4.
- **CI yapılandırması:** `.github/workflows/**` değişirse R0'ın "değişmesi beklenen kayıtlar" listesine o değişiklikten **önce** adıyla eklenir (K-P2Q1.4). Turun PR'ı kod değiştirdiği için CI tam koşar (K-D0.3 yalnız belge PR'larını hızlandırır).
- **İnsan adımı ve sorular:** insan adımı beklemek durma değildir (`HUMAN_STEP.md`). Soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye. Spec'in tanımlamadığı bir durum: PARK + karar talebi (uydurma yok).
- **Vault:** sınamalar yalnız sentetik kasada ya da D-NOTES kopyasında (depo dışında, scratchpad). Kişisel vault ve qorbit'e dokunulmaz. Kayıtlarda token, içerik ve host değeri yok.
- **Push:** bu makinede git'in kimlik bilgisi başka bir hesaba düşebiliyor (D-0 kapanışında 403, "denied to rootmark-ai"). Push 403 verirse kalıcı ayara dokunmadan `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push …` kullanılır; sürerse `HUMAN_STEP.md`.

## R0 — Açılış ve tasarım
**Yapar:**
- `ROUND_START`; DECISIONS K-P2Q3.1 (açılış).
- Hataların yayımlanmış `v0.3.2` ile yeniden üretimi: `research/d0/pq2_1_probe.py`, `offline_probe.py` (ve Docker betikleri) macOS ve Linux'ta, kısa tekrarla (her durum ≥ 3). Kayıt `reports/rounds/P2-Q.3/r0/`, özet `progress/R0_repro.md`.
- `progress/DESIGN.md`:
  - uzlaştırmanın algoritması: dosya listesi ↔ günlüğün canlı dokümanları; eksik dosya, farklı bayt, yeni dosya, taşıma; belirlenimli sıra;
  - izleyiciyle sıra: uzlaştırma ile ilk izleyici olaylarının yarışı (D0F-6: macOS FSEvents başlatmaya çok yakın olayı bazen iletir) — aynı değişikliğin iki kez alınmaması;
  - spec §7.1 okuması: başlangıçta tasdik penceresi yok; `unverified` ve `canonical` politikalarında sonuç; aktör;
  - PQ2-1'in kapanışı: SIGKILL'le kaybolan bekleyen kaldırmanın bir sonraki başlangıçta alınması (ya da gerekçeli başka yöntem, ör. bekleyen kaldırmaların kalıcı kaydı) ve neden;
  - `ingest-dir`'in silme kuralı ve kök sınırı;
  - performans: büyük vault'ta başlangıç süresi (D-NOTES ölçeğinde ölçülür, `TESTS.md`'de eşik);
  - ön-kayıtlı testler.
- `progress/TESTS.md`: GOAL R1 (a)–(f), R2 ve R4 (f) testleri, beklenen sonuçlar, tekrar sayıları.
- Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi: sürüm dizgileri (0.3.3), E1-4/R14 (etiket), değişecekse CI yapılandırması, D-NOTES'ta uzlaştırmanın yeni olayları (varsa, gerekçeyle).

**Kabul:** Q3-1.

## R1 — Başlangıç uzlaştırması
**Yapar:**
- `serve.rs`'te başlangıç uzlaştırması (`DESIGN.md`'deki gibi); alım `tools.rs`'in mevcut dış değişiklik ve `watch_removed` yolu üzerinden.
- Yeni entegrasyon testleri (ör. `server/tests/watch_restart.rs`): GOAL R1 (a)–(f); macOS ve Linux'ta (CI: `ubuntu-latest` ve macOS), her durum ≥ 20 tekrar (tekrar sayısı CI süresini aşarsa yerelde ≥ 20, CI'da azaltılmış ve `TESTS.md`'de gerekçeli). SIGKILL testi gerçek süreç sinyaliyle (`kill -9` ya da `Child::kill`), alt süreçte koşan `rootmark serve` ile.
- `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs` aynen geçer.

**Kabul:** Q3-2; Q3-4.

## R2 — `ingest-dir` silmeleri
**Yapar:**
- `ingest-dir <kök>`: kökün altında dosyası olmayan canlı dokümanlar kapatılır; kökün dışı aynen.
- Yeni test (ör. `server/tests/ingest_dir_remove.rs`): silme alınır; alt dizin kökü; değişiklik yokken yeni olay 0.

**Kabul:** Q3-3.

## R3 — `check_deps`'in macOS düşüşü
**Yapar:**
- #61'in koşusu `43241761070`'ın günlüğü (`gh run view 43241761070 --log`), betiğin okuma yolları, `make lint`'in hedef sırası; hipotezler (ör. `make test`'in bıraktığı dosyalar, `npm ci`'nin yazdıkları, `grep`/`sed` farkları, `pipefail` ile boru kapanışı) tek tek sınanır; macOS'ta (yerel ve gerekirse CI'da `workflow_dispatch` ile) yeniden üretme denemesi.
- Bulunursa düzeltme ve sınama; bulunmazsa betik düştüğünde uç listesini ve okunan dosyaları yazan tanı çıktısı. `test_check_deps.sh` geçer.
- `progress/R3_check_deps.md`.

**Kabul:** Q3-7.

## R4 — Belgeler, güvenlik ve 0.3.3
**Yapar:**
- (a) Kılavuz (D0F-7) ve (b) `CHANGELOG.md` 0.3.3 taslağı.
- (c) Uyarıların son durumu `progress/R4_security.md` (`gh api` dependabot ve code-scanning).
- (d) Kademe 2 bir tetikleme (`gh workflow run verify.yml --ref p2-q.3/round -f sha=<sha>`): yapıtlar, `tier2 pair/fails`, `portable`; değişen kayıtların R0 listesiyle karşılaştırması. `make verify-mac` bir kez (önce `fseventsd` sağlığı `ps` ile; sağlıksızsa `HUMAN_STEP.md`).
- (e) `release/0.3.3` dalı, R1–R20 (`release_check`), PR; `HUMAN_STEP.md` (birleşme, `release.yml`, etiket, sürüm).
- (f) Sürüm yayımlanınca: R1 (a), (c), (d) kısa tekrarla iki platformda; gerçek Claude Code oturum sonu sınaması (`claude -p --mcp-config … --strict-mcp-config`, D-NOTES kopyası; oturumun son saniyesinde bir not silinir; sonraki oturumda `search` onu döndürmez; Claude Code sürümü kayıtlı) → `progress/R4_claude_code.md`; D-NOTES senaryosu Mac, Docker ve CI'da (`pilot.yml`, bir koşu), 0.3.2 kayıtlarıyla karşılaştırma. Kayıtlar `reports/rounds/P2-Q.3/` altına.

**Kabul:** Q3-5, Q3-6'nın D-NOTES yarısı, Q3-8, Q3-9.

## R5 — Rapor, gözden geçirici, hazır
**Yapar:**
- Kapılar: `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- `FINDINGS.md`: D0F-1, D0F-5, D0F-7, `check_deps` düşüşü; yeni bulgular.
- Gözden geçirici tek tur (GOAL başlığı).
- `reports/rounds/P2-Q.3/FINAL.md`: Q3-1..Q3-10, karar kuralının dalı, koşu kimlikleri, "koştu/geçti" ayrımı.
- READY; tek PR: `gh pr create --base main --head p2-q.3/round --title "P2-Q.3: başlangıç uzlaştırması ve 0.3.3" --body-file reports/rounds/P2-Q.3/FINAL.md`.

**Kabul:** Q3-6, Q3-10; READY'de "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
