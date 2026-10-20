# P2-A — Görevler

Sıra: A0 → A1 → A2 → A3 → A4 → A5 → A6. Progress `docs/rounds/P2-A/progress/`. Commit öneki `P2-A/A<n>:`. Dal `p2-a/round` (K-P2.1; yalnız birleşmiş `main`'den, K-P2.2). Gözden geçirici alt-ajan görev başına en fazla bir tur; ikinci tur yalnız bir kabul ölçütü düştüyse (K-P2.3).

## A0 — Tur açılışı ve P2-0 dizisinin kapanış işleri
**Yapar:** `ROUND_START` (= PR #14'ün birleştiği `main` commit'i); DECISIONS: tur açılışı, P2-0.2 doğrulamasının önerileri (Ö1 → A0 b/c/d, Ö2 → K-P2-02.5 uygulanışı A0 a, Ö3 → A0 e); `config::default_recipe` `[attestation] unattested = "canonical"` + yorum; `status.attestation` gösterir; (A) düzeltmesi (eski yolun başına karşı denetim) + birim test; `--attest-log`; harness yeniden deneme (dondurma istisnası, önce/sonra kanıt); bilgi ölçümü: D-B test-3 tek zincir (gerçek Obsidian), `reports/rounds/P2-A/info_d_b_test3.json` (eşik yok).
**Kabul:** A-7; yeni vault `status` canonical; P2-0.2'nin `unverified` davranışı seçilince aynen (C2-11..14 geçer); `check_frozen`, `check_deps`.

## A1 — Göç çerçevesi
**Yapar:** `core`/`server`: göç kayıt defteri (`Migration` trait: `id`, `applies(vault) -> bool`, `plan(vault) -> Plan`, `apply`, `rollback`), `.rootmark/migrations/<id>/` geri alma kaydı (kopya + manifest + SHA-256), `rootmark migrate [--dry-run] [--rollback <id>] <vault>`, çıktı JSON (plan, uygulanan, bayt etkisi); kayıtlı göçler: `rename_store` (mevcut, çerçeveye taşınır), `index_rebuild` (`mdkernel-index/3` → güncel manifest biçimi; göç yeniden kurar, `events_hash` dokunulmaz), `log_readers` (2.0.0→2.0.2 doğrulama; bayt değişmez), `checkpoints` (kayıt, no-op). `events_hash`'i değiştirecek göç → hata.
**Kabul:** A-1 birim/entegrasyon testleri; idempotent (ikinci koşu no-op); `--dry-run` sonrası `git status`/bayt eşit.

## A2 — Sürüm matrisi ve "göç gerekli"
**Yapar:** `status.migrations` (yapıt başına biçim/beklenen/durum), hata kodu `migration_required` (sözleşme §3; `index_stale` yalnız manifest'le uyuşmayan indeks için kalır), `contract_check.sh` yeni alanlar; `historical` sınıflandırıcısına "göç gerekli" kategorisi (kaydediciler değişmez; K-E1.4 listesi aynen); CLIENT_CONTRACT ve spec bilgilendirici not; sürüm CRITERION ile.
**Kabul:** A-3; `contract_check.sh` kasıtlı alan kaldırma kırmızı; eski indeksli vault `status` → `migration_required` ve `rootmark migrate` yol gösterir.

## A3 — Kurulum ve CI
**Yapar:** `release.yml`: Windows `fetch_model.sh` düzeltmesi (ya da PowerShell eşdeğeri), Linux/Windows artefaktlarında `rootmark conformance`, `actions/attest-build-provenance`, `SHA256SUMS`; `dist/install.sh`, `dist/install.ps1`; `cargo binstall` meta verisi; eylem sürümleri (Node 24; ≥ 14 gün kuralı `check_dep_age.sh`); RELEASE.md: imzalama/notarization insan adımları, kurulum doğrulama maddeleri (`release_check.sh` yeni maddeler); `release.yml` dispatch'i **bu turda ajan tetikler** (etiket değil; K-GH.4 emsali) ve sonucu kaydeder.
**Kabul:** A-4; `gh attestation verify` çıktısı kayıtta; `release_check.sh` yeni maddeleri yeşil.

## A4 — Intel macOS ve K-GH.6
**Yapar:** `ort` sürüm taraması (hangi sürümde `x86_64-apple-darwin` hazır ikili; sabitleme ≥ 14 gün); varsa `release.yml`'e bilgi hedefi, yoksa kayıt; karar talebi metni. `propose` dizin bileşeni varyant reddi (her platform; `paths.rs`), CLIENT_CONTRACT §3, Linux CI işinde test yeşil; `ci.yml`'de Linux işi "information" etiketi kalkar (zorunlu yapma ruleset insan adımı, STOP.md'ye).
**Kabul:** A-5, A-6.

## A5 — Doğrulama zinciri
**Yapar:** `verify.sh`: göç adımı (6 vault: dry-run → göç → eşitlikler → geri alma → bayt eşit) → conformance → regress (P2-0.2 referansı) → tamper (bozuk geri alma kaydı → geri alma reddi; `events_hash` değiştiren sahte göç → ret; eski indeks + `auto_rebuild=false` → `migration_required`) → tek yüksüz çift; `compare_p2a` belirlenimli alan listesi.
**Kabul:** A-8; kasıtlı vakalar.

## A6 — Rapor ve hazır
**Yapar:** `report_p2a.py`; `reports/rounds/P2-A/FINAL.md` (kriter tablosu, karar kuralı dalı, bilgi ölçümü ayrı başlıkta, platform karar talebi varsa); `READY.md`; tek PR.
**Kabul:** izlenebilirlik; karar kuralı; NORTH_STAR değiştirilmez.
