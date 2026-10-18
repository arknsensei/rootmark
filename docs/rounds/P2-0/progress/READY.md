# P2-0 — doğrulama için hazır

- **Commit:** RUN'ı ve FINAL'i taşıyan commit `e7058e0`; resmi `make verify`'ın ölçtüğü commit `e378be2`. Ürün ve ölçüm kodu `fa02519`'dan beri değişmedi; sonrası yalnız rapor üreteci metni (F16, F17) ve belgeler.
- **Resmi RUN:** `20261018T161850Z-e378be2` (`reports/rounds/P2-0/runs/20261018T161850Z-e378be2/`); çift: A `20261018T161850Z-e378be2`, B `20261018T170104Z-e378be2`, `make verify` dal sabitken çıkış 0, 106 701 alan fark 0, yapıtlar bayt eşit. Ayar RUN'ı: `20261018T122523Z-aa61274` (W = 0.5 s; `Z5_tuning.json`).
- **Resmi sayılmayan koşu:** ilk `make verify` (A `20261018T140007Z-fa02519`) B klonu koşu sırasında eklenen commit'leri aldığı için 3 ile bitti; resmi değil, commit'lenmedi (ASSUMPTIONS Z8-1).
- **FINAL:** `reports/rounds/P2-0/FINAL.md` — 0a, 0a′, 0c, 0f, 0g, 0h geçti; **0b kaldı** (931/1890 = 0.49; 927'si yeniden adlandırma yolundan, ASSUMPTIONS Z5-5), **0d kaldı** (333/340 = 0.979; 7'si `collide`, Z5-6); 0e ölçülmedi (Z6 PARK, `STOP.md`); regress'te bir fark (R16 sayımı, K-P2-0.3). **Karar kuralı: tanımlanmamış kalış → tur kaldı, karar talebi** (`docs/DECISIONS.md` K-P2-0.6-T). NORTH_STAR değiştirilmedi; öneri FINAL'de.
- **Açık maddeler:** K-P2-0.6-T (insan kararı); `scripts/keychain_probe.sh` (H-P2-0e, insan koşturur); spec §7.1 "Draft" kalır, kodda varsayılan `unattested = "unverified"` durur (yayım yok).
- **Kalite kapıları:** `make test` (cargo 256, pytest 232 + 1 atlanan, vitest 139) ve `make lint` `fa02519`'da temiz (sonrası yalnız rapor üreteci, HEAD'de ruff temiz, ve belgeler); `make conformance` 37/37; `make verify` `e378be2`'de temiz.
- **Doğrulama yönergesi:** `docs/rounds/P2-0/VERIFY.md`. D-B değerleri gerçek Obsidian zamanlamasına bağlıdır ve çift karşılaştırmasının dışındadır (`P2_0_TIMING`); bağımsız bir koşuda küçük sapmalar beklenir (ilk koşu: 0b 932, p95 287 ms; resmi: 931, 270 ms).

doğrulama için hazır
