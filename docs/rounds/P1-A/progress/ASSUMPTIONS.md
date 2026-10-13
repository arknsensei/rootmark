# P1-A Varsayımları

Etiketler: `[spec]` spesifikasyon yorumu, `[arch]` mimari/paket yapısı, `[-]` diğer.

## A1
- [arch] `index` crate'i P1-A'da yalnız trait tanımları taşır (`LexicalIndex`, `Embedder`, `VectorIndex`, `Fusion`); `server` yalnız boş `lib.rs` (ikili yok; `mdk` adı `core/src/bin/mdk.rs` ile çakışmasın diye).
- [-] `make test-ts` / `lint-ts`, `clients/*/package.json` yoksa bunu açıkça yazıp geçer (P1-A'da istemci yok).

## A2
- [spec] `Layer` olay günlüğüne yazılmaz; `Block.layer` `provenance.author_kind`'dan türetilir. Gerekçe: GOAL A-5 ve TASKS A2 kabulü `events.jsonl` biçiminin v3-final ile bayt eşit kalmasını şart koşar; katman olayları (`LayerChanged`, `Promoted`) P1-B kapsamındadır.
- [spec] `synthetic` → `canonical` (spec §7 "as declared"; P1-B öncesi beyan mekanizması yok, sentetik değerlendirme verisi insan düzenlemesinin yerine geçer).
- [spec] spec §5 "MUST record `matcher_version` and parameters in `meta` of every event" — v3-final yalnız `matcher_version` yazar; THETA/THETA_SPLIT yazılmaz. Davranış değiştirilmedi (kırmızı çizgi: V2/V3 çıktıları v3-final ile bayt bayt aynı). Varsayılan parametrelerde `matcher_version` parametreleri belirler (v2 → 0.60/0.70, v3 → 0.50/0.65); varsayılan dışı parametre yalnız ölçüm ızgaralarında kullanılır. Karar talebi: `docs/DECISIONS.md` K14-T.
- [-] `mini_vault` fixture'ı `spec/fixtures/mini_vault/` altına taşındı (çekirdek testleri ve C1-17 ortak); `core/tests/*.rs` içindeki yol `../tests/fixtures/mini_vault` → `../spec/fixtures/mini_vault` (yalnız yol; test mantığı değişmedi).
