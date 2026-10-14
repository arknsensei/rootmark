# P1-B — Görevler

Sıra: B0 → B1 → B2 → B3 → B4 → B5 → B6 → B7 → B8 → B9. Progress: `docs/rounds/P1-B/progress/`. Commit öneki `P1-B/B<n>:`.

## B0 — Tur açılışı ve altyapı önerileri (Ö1, Ö2)
**Yapar:** `git tag p1a-final` (READY commit'i). Ö1: `research/FROZEN.md` kendi satırı; `scripts/check_frozen.sh` tur başlangıç sha'sını `docs/rounds/<id>/progress/ROUND_START` dosyasından okur ve FROZEN.md tablosunun o sha'dan beri yalnız satır eklediğini (`git log -p`) doğrular. Ö2: `core/tests/conformance` fixture kökünü `MDK_FIXTURES` ya da çalışma dizininden çözer; `make conformance` `MDK_FIXTURES=$(CURDIR)/spec/fixtures`; `conformance.json` `fixtures_root`, `fixtures_manifest_hash` alanları.
**Kabul:** B-7 ve B-8 kasıtlı testleri geçer (betikle: `scripts/test_tamper.sh` iki yeni vaka); `check_frozen.sh` temizde 0.

## B1 — Katman ve geçiş olayları
**Yapar:** `Block.layer` zorunlu alan (v3-final nesnelerinde yoksa yükleme sırasında `author_kind`'dan türetilir, nesne yeniden yazılmaz); `EventKind::{LayerChanged, Promoted, Demoted}`; replay katman durumunu kurar; `Vault::layer_of(lineage) -> Layer`.
**Kabul:** C2-01, C2-08 fixture'ları; B-5 (v3-final vault açılır, `events_hash` değişmez, replay eşit); B-10 tohumlu 1000'lik dizi testi (`core/tests/layers_fuzz.rs`).

## B2 — Terfi hook'u ve budama
**Yapar:** `PromotionPolicy` trait, `PromotionRequest { lineage, from, to, actor, reason }`, `Decision::{Allow, Deny(reason)}`; `Deny`, `HumanOnly`; `Vault::promote(req, &dyn PromotionPolicy)`; hook yoksa derleme düzeyinde imkânsız, yanlış actor → `Err`; `Promoted.meta = {actor, reason, policy_version}` zorunlu; `prune_inactive`.
**Kabul:** C2-02, C2-03, C2-04; B-6'nın terfi vakaları.

## B3 — Kontrol noktası
**Yapar:** `Checkpoint` tipi ve formül (spec §3); `Vault::checkpoint(name?)`, `checkpoints()`, `snapshot_at`, `block_at`; `.mdkernel/checkpoints/<cid>.json` write-once; `CheckpointCreated` olayı.
**Kabul:** C2-05, C2-06; aynı vault durumunda iki `checkpoint()` çağrısı aynı `checkpoint_id` (deterministik); sonraki ingest'ten sonra `snapshot_at` eski snapshot'ı döner.

## B4 — Tüketim izi
**Yapar:** `.mdkernel/traces.jsonl`; `record_trace`; `trace_id` formülü; `Vault::read_blocks(refs, consumer, checkpoint)` her çağrıda iz yazar; `traces(since_seq)`.
**Kabul:** C2-07; iz satırı tohumlu testte bayt bayt beklenen; aynı okuma iki kez → iki iz (append-only), aynı `trace_id` değil (`at_seq` farklı).

## B5 — Ham günlük ve sürüm tetikleyici
**Yapar:** `.mdkernel/journal/`; `Vault::journal_append(doc, bytes)`; `VersionTrigger` trait + `IdleOrDistance`; `Vault::maybe_version(doc, trigger, now)` → ingest ya da `None`; çekirdek sınırları ve `Err` davranışı; "sürümsüz içerik indekslenmez" garantisi için `Vault::indexable_snapshot(doc)` yalnız sürümlenmiş snapshot döner.
**Kabul:** sınır vakaları `Err` (B-6'nın 3 vakası); tohumlu zaman dizisiyle tetikleyici testi (idle ve mesafe ayrı ayrı); günlük budama `retention_hours` altında siliyor, üstünde silmiyor; budama olay üretmiyor.

## B6 — Politika tarifi ve CLI
**Yapar:** `policy.toml` yükleyici (`serde` + sınır doğrulama); `mdk` komutları (GOAL listesi); `blame` çıktısı: blok sırasıyla `(lineage_id, version_id, layer, author_kind, source, seq_created, seq_last)`; `--at <checkpoint>` desteği.
**Kabul:** B-6 tam 6 vaka betikli; her CLI komutu için uçtan uca test (`core/tests/cli_governance.rs`); `blame --at` sonraki olayları göstermiyor.

## B7 — Spec hizalama ve C2 fixture'ları
**Yapar:** `spec/fixtures/C2-01..08/`; beklenen değerler mevcut çekirdekten üretilir ve `MANIFEST.json`'a eklenir (C1 fixture'ları değişmez); spec §7, §8, §11 metni yalnız açıklık düzeltmeleri; her düzeltme `docs/rounds/P1-B/progress/SPEC_EDITS.md`'de satır satır. Anlam değişikliği gerekiyorsa görev park, `DECISIONS.md`'ye talep.
**Kabul:** B-1, B-2; `SPEC_EDITS.md` gözden geçirici tarafından "yalnız açıklık" diye onaylı.

## B8 — Holdout-split (Ö3) ve regresyon
**Yapar:** `research/eval/datasets/synthetic_edits_v4.py` (yeni dosya): `d1hs`; manifest; `run_holdout_split.py`; `make regress` P1-A seti + `d1hs`; `FROZEN.md` yeni satırlar.
**Kabul:** B-9; gerçekleşen oranlar GOAL Ö3 alt sınırlarını sağlıyor (raporda); `d1hs ∩ d1ho ∩ easy ∩ hard = ∅`; B-3.

## B9 — Doğrulama zinciri, rapor, hazır
**Yapar:** `scripts/verify.sh`: conformance (L2) → regress → holdout → holdout-split → governance fuzz → tamper (B-6, B-7, B-8) → pair compare (`traces.jsonl` ve `checkpoints/` bayt karşılaştırması dahil); `report_p1b.py`; `reports/rounds/P1-B/FINAL.md`; `READY.md`.
**Kabul:** B-4; her sayı izlenebilir; karar kuralı çıktısı.
