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

## A3
- [spec] C1 fixture'larının beklenen değerleri v3-final çekirdeğinin çıktısıdır (TASKS A3); tür adları v3-final yazımıyla (`Heading`, `Human`) kayıtlıdır (K15-T).
- [spec] C1-06 hata mesajları (geçici yol içerir) fixture'a girmez; yalnız `ok`/`error` sonucu ve dosya baytlarının değişmediği. C1-15 mesajı (yol içermez) aynen kayıtlıdır.
- [-] Gözlem: `[X, A, Y] → [X, A', Y']` (A ve komşusu Y birlikte hafif düzenlenmiş) v3'te `BlockSplit(A → A', Y') + BlockDeleted(Y)` üretiyor; Y'nin soyu kopuyor. v3-final davranışı; P1-A'da değiştirilmez. Holdout/split_hard sonuçları yorumlanırken dikkate alınmalı (A5).

## A4
- [arch] `research/` prototip kökünün yerini alır (`eval/`, `mdkernel/`, `tests/`, `data/`, `reports/runs/`); dondurulmuş kod `ROOT = parents[2]` ile `research/`'ü kök görür, böylece dondurulmuş dosyalarda yol değişikliği gerekmez.
- [arch] `research/tests/test_bindings*.py` CLI↔Python eşdeğerliğini sınamak için `target/release/mdk` ikilisini alt süreç olarak çağırır; bu bir import/paket bağımlılığı değildir, `check_deps.sh` import tarayıcısı Python import'larını ve yol erişimlerini denetler.
- [-] `make regress` çıktıları `research/reports/runs/<RUN>/` (prototip düzeni) altında üretilir ve `reports/rounds/P1-A/runs/<RUN>/`'a kopyalanır.
- [-] Prototipin rapor üreteçleri (`eval/report.py`, `eval/report_v3.py`) dondurulmuş olarak taşındı ama P1-A zincirinde koşmaz; P1-A raporu A7'de ayrı üreteçle.

## A5
- [spec] Holdout uygunluk kuralı: ≥3 blok, ≥1 paragraf, ≥1 düzyazı blok, render geri dönüşü (D1-hard kuralı easy ∪ hard dışında yalnız 5 doküman bırakıyor). Op tablosu ve op fonksiyonları D1-hard'ınkiyle aynı; uygulanamayan op D1-hard'daki gibi yeniden çekilir, bu yüzden gerçekleşen dağılım sapar (manifest `realised`).
- [spec] `split_hard` op adları dondurulmuş metriklerin tanıdığı adlardır (`split_edit`, `merge_edit`, `dup_delete_adjacent`); varyant `variant` alanında. `dup_delete_adjacent` (%10) dondurulmuş amacın (S1+(1−S2)+S3+S4) tanımlı olması için dilimde tutuldu.
- [spec] "≤ 8 kelimelik parçalar": iki parçalı bölmede kısa parça ≤ 8 kelime; "iki parçası da düzenlenmiş": her parçaya `rephrase_light`; "≥ %15 düzenlemeli birleştirme": birleşik metnin kelimelerinin ≥ %15'i değişmiş (`rephrase_heavy` aralığı, ölçülerek zorunlu).
- [-] A-7 ızgarasında THETA derlenmiş v3 değeridir (0.50); ızgara yalnız THETA_SPLIT'i değiştirir.
