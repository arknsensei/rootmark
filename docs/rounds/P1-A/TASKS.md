# P1-A — Görevler

Sıra: A0 → A1 → A2 → A3 → A4 → A5 → A6 → A7. Progress: `docs/rounds/P1-A/progress/`.

## A0 — Kaynak teyidi ve etiket
**Yapar:** Prototip deposu `prototype/` alt modülü ya da `git subtree` ile içe alınır (`v3-final` etiketi; yoksa prototip deposunda HEAD'e `v3-final` etiketi atılır ve sha kaydedilir). `docs/rounds/P1-A/progress/SOURCE.md`: prototip sha, v2/v3 manifest hash'leri.
**Kabul:** `git log` prototip geçmişini içeriyor (subtree) ya da sabit sha kayıtlı.

## A1 — Monorepo iskeleti
**Yapar:** `Cargo.toml` workspace (`core`, `index`, `server`, `bindings/python`); `index` ve `server` boş crate (yalnız `lib.rs`/`main.rs` + README); `clients/README.md`; `research/pyproject.toml`; `Makefile` hedefleri (`test`, `lint`, `conformance`, `verify`, `regress`); CI (`.github/workflows/ci.yml`: test, lint, conformance, bağımlılık yönü kontrolü); paket `CLAUDE.md`'leri (DEV_PROCESS §7 şablonu).
**Kabul:** `cargo build --workspace`, `make test` boş geçer; `scripts/check_deps.sh` ters bağımlılığı yakalıyor (kasıtlı test: `core`'a `index` bağımlılığı ekleyip betiğin hata verdiğini göster, geri al).

## A2 — `core/` taşıma ve API hizalama
**Yapar:** Prototip `core/` → `core/`; public tipler `spec` terimleriyle (`author_kind`, `layer` alanı **eklenir ama yalnız `canonical`/`proposal` varsayılanı yazılır, geçiş yok**); `Checkpoint`, `Trace` tipleri yalnız tanım; `MatcherVersion::{V2,V3}`; `mdk` CLI `server/` yerine şimdilik `core/src/bin/mdk.rs`'de kalır (P1-D'de taşınır).
**Kabul:** prototip Rust testleri geçer; A-5 hash'leri; `events.jsonl` biçimi değişmedi (v3-final fixture ile bayt eşitliği).

## A3 — Uyumluluk fixture'ları ve testleri
**Yapar:** `spec/fixtures/C1-xx/` (girdi md + beklenen JSON); `core/tests/conformance.rs` her C1 testi; `make conformance` → `reports/conformance.json` (`level_claimed: 1`).
**Kabul:** A-1; her fixture'ın beklenen değeri `v3-final` çekirdeğinden üretildi ve `SOURCE.md`'de kayıtlı; fixture'lar dondurulmuş.

## A4 — `research/` taşıma
**Yapar:** Prototip `eval/`, `mdkernel/` (Python), `tests/` → `research/`; `bindings/python` → `mdkernel_core` wheel; `scripts/verify.sh` yeni yollarla; dondurma listesi `research/FROZEN.md` (denetleyiciler dahil); `make regress`: v3 D1-easy/hard, D2, H3, H4-V2 → `reports/rounds/P1-A/`.
**Kabul:** A-3 birebir (v3-final JSON'larıyla alan alan); `freeze-check`, `freeze-check-v3` geçer ve denetleyiciler sabit-bağımsız (test: sabitler değiştirilince denetleyici sonucu değişmiyor).

## A5 — Holdout seti ve bölme eşiği dilimi
**Yapar:** `research/eval/datasets/synthetic_edits_v3.py` (yeni dosya): holdout (SEED `20261013:holdout`, 100 doküman, D1-hard dağılımı) ve `split_hard` dilimi (GOAL kapsamı). `research/eval/run_holdout.py`, `run_split_grid.py`. Manifest'ler.
**Kabul:** A-2; A-7 raporu; holdout ∩ easy ∩ hard = ∅ (programatik); `split_hard` parçalarının hiçbiri eski metnin tam alt dizisi değil.

## A6 — Doğrulama zinciri
**Yapar:** `scripts/verify.sh`: klon → setup → conformance → regress → holdout → split-grid → compare (iki dizin); çıkış kodu disiplini.
**Kabul:** A-4; kasıtlı bozma: bir fixture beklenen değeri değiştirilince `make conformance` hata; bir holdout truth `version_id` sıfırlanınca `TruthError`.

## A7 — Rapor ve hazır
**Yapar:** `reports/rounds/P1-A/FINAL.md` (yalnız RUN'dan; A-1..A-7 tablosu; karar kuralı); `docs/rounds/P1-A/progress/READY.md`.
**Kabul:** her sayı JSON'a izlenebilir; READY.md sha + RUN içeriyor.
