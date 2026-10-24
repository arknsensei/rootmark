# P2-P — Görevler

Sıra: P0 → P1 → P2 → P3 → P4 → P5. Progress `docs/rounds/P2-P/progress/`. Commit öneki `P2-P/P<n>:`. Dal `p2-p/round` (yalnız birleşmiş `main`'den, K-P2.2; `ROUND_START` = P2-L birleşmiş `main` HEAD). Geliştirme turu dozu (K-P2.4): gözden geçirici yalnız P5'te tek tur; güvenlik incelemesi P1 ve P2 için P5'in gözden geçiricisinde ayrı başlık. `make verify FAST=1` görev sonlarında; tam `make verify` P5'te bir kez.

## P0 — Açılış, P2-L kalıntıları, K-P2L-T1
**Yapar:** `ROUND_START`; DECISIONS: tur açılışı, K-P2L-T1 kapanışı (seçenek (a); insan onayı GOAL'dedir); P2-L FINAL not 8'in a1–a4 bulguları (kod + test) ve K-P2L-T2 (a) belgesi (README "Türkçe vault'lar" ya da PACKAGES); göç adımı `policy_attestation_1` (`server` ya da `core`'un göç çerçevesi, P2-A; `.rootmark/migrations/<id>/policy.toml.before`; `--dry-run` planında; `--rollback`); `status.migrations` `policy` satırı; `status.attestation.implicit`; `serve` uyarı satırı; CHANGELOG "Unreleased / Upgrading from 0.1.0"; `docs/RELEASE.md` ve `PACKAGES.md` gerekiyorsa.
**Kabul:** P-1 (üç vault testi: 0.1.0 tarifi, 0.2.0 tarifi, tarifsiz); P-9; `check_frozen`, `check_deps`; `make verify FAST=1`.

## P1 — `[[promotion.rules]]`
**Yapar:** `core/src/policy.rs`: `[promotion]` tablosuna `version` ve `rules` (yükleme doğrulaması, hata metinleri alan adlı); `core`: `policy` aktörlü `Promoted` için okuma kuralı (`reason` `rule:` öneki, `policy_version` boş değil) ve `vault.rs` replay denetimi; `server`: kural eşleyici (`propose`'dan sonra, ilk eşleşen kural, soy kimliği sırası, her soy için `Promoted`), `propose` yanıtı olayları taşır, `rootmark policy lint`; `tools.rs` araç açıklamaları; CLIENT_CONTRACT §2.1/§2.4 (kurallı terfi, `Promoted.meta` alanları, `lint`); ARCHITECTURE §4.4 bir cümle.
**Kabul:** P-2, P-3; kuralsız tarifle hiçbir davranış değişmedi (mevcut testler aynen); `events_hash` değişmedi (P2-A'nın üç eski vault'u).

## P2 — Güvenlik vakaları
**Yapar:** S1–S7 testleri (`server/tests/promotion_rules.rs`; S1 izleyiciyle, `unverified` ve `canonical` politikada; S6 C3-06 koşusu + `events_hash`; S6b kurallı tarifle sıralama farkı yalnız önek içi belgelerde); THREAT_MODEL "Yönetişim garantileri"ne kural maddesi (tasdiksiz yazım ve `unverified` hiçbir kuralla terfi etmez; kural yalnız `propose`'un kendi soylarına bakar); `docs/SECURITY.md` gerekiyorsa.
**Kabul:** P-4, P-5; testler kalıcı (`make test`).

## P3 — Başsız insan kanalı
**Yapar:** THREAT_MODEL maddesi (bot süreci insan kanalı; ayrı OS hesabı; token yalnız o hesaba; tek `human_id` açık sınır); `docs/HEADLESS_CHANNEL.md`; spec §7 "Provenance of events" CRITERION açıklığı (2.0.3; `docs/SPEC_EDITS.md` satırı; okuma kuralı); `Promoted`/`Demoted.meta.source` = `X-Consumer` (HTTP) / `--source` (CLI), yoksa alan yok; CLIENT_CONTRACT §2.4, §4.2; `history` çıktısı; eski günlük okuma testi.
**Kabul:** P-7; spec §1–6 diff boş; `make conformance` 39/39.

## P4 — Sahne
**Yapar:** `server/tests/scene_collector.rs` (GOAL 5; MCP ve CLI); iki koşturmada `events_hash` ve `trace` bayt eşitliği; sahnenin kısa anlatımı `docs/HEADLESS_CHANNEL.md`'ye ya da `docs/CLIENT_CONTRACT.md` §4.1 ajan yönergesine (kurallı vault'ta ajan bilmeli: önek içi `propose` doğrudan canonical olur).
**Kabul:** P-3, P-6 (sahne kısmı).

## P5 — Zincir, rapor, hazır, blok sonu
**Yapar:** `verify.sh`: P2-P adımı (P-1 üç vault, P-5 S6, P-6 sahne; dondurma istisnası kaydı önce/sonra); tam `make verify` bir kez; gözden geçirici (tek tur, bütün kabul ölçütleri; güvenlik başlığı P1/P2); FINAL (kriter tablosu, karar kuralı dalı, K-P2L-T1 sonucu, blok sonu doğrulayıcı için P2-L + P2-P özet); READY; tek PR.
**Kabul:** P-6 (tam çift), P-8; izlenebilirlik; karar kuralı.
