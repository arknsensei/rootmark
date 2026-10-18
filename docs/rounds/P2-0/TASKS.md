# P2-0 — Görevler

Sıra: Z0 → Z1 → Z2 → Z3 → Z4 → Z5 → Z6 → Z7 → Z8. Progress `docs/rounds/P2-0/progress/`. Commit öneki `P2-0/Z<n>:`.

## Z0 — Tur açılışı ve P1-E.1 önerileri
**Yapar:** `ROUND_START` (= `v0.1.0`); `RELEASE.md` insan adımı 1 → VERIFICATION bağı; `contract_check.sh` `status` §2 tüm üst düzey alanlar; bayat/bozuk indekste kapanış sıkıştırması notla atlanır (çıkış 0) ve sözleşmeye yazılır; resmi RUN bayt yapıtlarının hash'leri `reports/rounds/<id>/runs/<RUN>/artifacts.sha256` olarak kaydedilir (geriye dönük P1-E, P1-E.1). DECISIONS: tur açılışı, ROADMAP v2 yürürlük.
**Kabul:** `contract_check.sh` kasıtlı alan kaldırma kırmızı; `release_check.sh` 17/17 (yeni madde); `check_frozen.sh`, `check_deps.sh`.

## Z1 — Spec ve uyumluluk tasarımı
**Yapar:** spec §7 `unverified` katmanı, tasdik eşleşmesi (W), `unattested_fs` aktörü, politika; §8 `attestations.jsonl` (yalnızca-ekleme, `attestation_id = H(doc_path + "\n" + content_hash + "\n" + at_seq)`); CONFORMANCE C2-11/12, C3-08 tanımları; `SPEC_EDITS.md`; sürüm CRITERION ile. Fixture'lar Z4'te.
**Kabul:** gözden geçirici "yalnız tanımlanan genişleme"; sürüm gerekçesi yazılı.

## Z2 — Çekirdek ve sunucu
**Yapar:** `core`: `Layer::Unverified`, `actor.kind = unattested_fs`, tasdik günlüğü, eşleştirme penceresi (W parametre, `policy.toml [attestation] window_s`), geç tasdik `LayerChanged`; `server`: `POST /v1/attest` (insan kanalı, HMAC oturum anahtarı), `rootmark attest`, `status.attestation` alanı; izleyici tasdik eşleştirmesi; MCP'de `attest` **yok** (test).
**Kabul:** birim testler (pencere içi/dışı/geç, çoklu hızlı kayıt, yeniden adlandırma); MCP `attest` çağrısı `tool_not_found`; eski vault yükleme canonical.

## Z3 — İstemci tasdiki
**Yapar:** Obsidian plugin: `vault.on('modify')` + kayıt sonrası `attest` (aşama 1: oturum anahtarı); blame'de `unverified` işareti; proposal listesinde `unverified` sekmesi ve tek tık terfi; durum çubuğunda `attested/unattested` sayacı. `src/attest.ts` yalnız sözleşme uçlarıyla.
**Kabul:** `vitest`; gerçek Obsidian e2e'de 20 kayıt → 20 tasdik; md yazmama kuralı korunuyor (statik tarama).

## Z4 — Fixture'lar ve uyumluluk
**Yapar:** C2-11, C2-12, C3-08 fixture'ları; `gen_fixtures_*`; `make conformance` 37/37.
**Kabul:** H-P2-0h; beklenenler yeniden üretilebilir, `git status` temiz.

## Z5 — Veri setleri ve ölçüm
**Yapar:** `research/eval/datasets/p2_0_sets.py` (D-A üreteci, D-B senaryo listesi, D-C karışım, D-E vault'lar); `scripts/agent_fs_writer.py`; gerçek Obsidian harness genişletmesi (D-B 300 oturum, otomatik kayıt zamanlaması); `run_p2_0.py --phase tune` (W taraması D-B ayar) → `tuning.json`, `W` sabitle; `--phase test`.
**Kabul:** ayar/test ayrımı; fail-fast; D-A 200 senaryonun her biri beklenen katmanı doğrulayan tek bir fonksiyonla ölçülür (kod gösterilir); H-P2-0a..0d, 0f.

## Z6 — Anahtar zinciri alt-hipotezi (macOS)
**Yapar:** `server`: tasdik gizi macOS Keychain'de, ACL yalnız Obsidian imzalı uygulamasına; plugin tarafı; `scripts/keychain_probe.sh` (terminal sürecinden okuma denemesi, 20 kez); sonuç H-P2-0e. Olmazsa gerekçeli rapor, aşama 1'e dön.
**Kabul:** 0e ölçüldü (geçti/kaldı); hangi aşamanın etkin olduğu `status`'ta.

## Z7 — Doğrulama zinciri
**Yapar:** `verify.sh`: conformance → regress (P1-E.1 referansı) → D-E → D-A → D-C → D-B (gerçek Obsidian varsa; yoksa "atlandı" ve 0b/0c ölçülmedi diye raporla, karar kuralı bekler) → tamper (sahte tasdik, yanlış hash, token'sız attest, MCP attest) → pair compare (`attestations.jsonl` dahil).
**Kabul:** H-P2-0g; kasıtlı vakalar.

## Z8 — Rapor ve hazır
**Yapar:** `report_p2_0.py`; `FINAL.md` (hipotez tablosu, karar kuralı, tehdit modeli güncelleme taslağı); `READY.md`.
**Kabul:** izlenebilirlik; karar kuralı; NORTH_STAR değişikliği **yapılmaz**, yalnız önerilir (insan kararı).
