# P2-0.2 — Görevler

Sıra: X0 → X1 → X2 → X3 → X4 → X5 → X6. Progress `docs/rounds/P2-0.2/progress/`. Commit öneki `P2-0.2/X<n>:`. Dal `p2-0.2/round` (K-P2.1).

## X0 — Tur açılışı
**Yapar:** `ROUND_START` (= PR #12'nin birleştiği `main` commit'i); DECISIONS: tur açılışı; P2-0.1 doğrulamasının Ö1 → X1/X4, Ö2 → X2, Ö3 → X3, F4 → X3, F6/F7 → kayıt (bilinen sınır); D-B test-3 tohumu, bağlantı biçimi dağılımı, D-A′ ek senaryolar **ölçümden önce** `ASSUMPTIONS.md`'ye; regresyon referansı P2-0.1 resmi RUN'ı `20261019T001516Z-21b16ea`.
**Kabul:** `check_frozen.sh`, `check_deps.sh`; tohum ve dağılım commit'li, veri yok.

## X1 — Bağlantı denetimi ve katman taşıma (T1, T2)
**Yapar:** `server`: `link_update_of` fark kuralı (hedef + eşit-etiket dönüşümü; kodlu yol; gömme), `renamed_from` katman taşıma (blok blok, yükseltme yok, düşürme yok), notun kendi içindeki yeniden yazım aynı tasdikte; `clients/obsidian`: `onlyCanonical` ön koşulu kalkar, yeniden adlandırma her durumda tasdik edilir; CLI `--renamed-from` aynı kural; spec §7.1 metni, CLIENT_CONTRACT §2.6; SPEC_EDITS.
**Kabul:** birim testler (etiket dönüşümü kabul; etiket + başka metin ret; kendine bağlantı; `unverified` bloklu notun yeniden adlandırması: katmanlar aynen; `proposal` aynen); gerçek Obsidian duman denemesi (ölçüm değil) 6 biçim; gözden geçirici + güvenlik incelemesi (aklama: yeniden adlandırmayla yükselme 0).

## X2 — D-C determinizmi (T3)
**Yapar:** `run_p2_0_1.py` D-C üreteci (ya da P2-0.2 kopyası): `apart`/`same_doc_after` tiklerde insan alımı günlüğe girene kadar `quiesce`, sonra ajan; `collide` değişmez; FREEZE_EXCEPTIONS girdisi (önce/sonra: `collide` tiklerin kaydı bit eşit); `verify.sh`'e yük altında ikinci çift (`VERIFY_LOAD=1`: zincirler boyunca `nproc` kadar `yes >/dev/null`).
**Kabul:** yüksüz ve yüklü çift D-C `events.jsonl` bayt eşit (en az iki tekrar); gözden geçirici onayı.

## X3 — Süreç, CRITERION, okuyucu (T4)
**Yapar:** DEV_PROCESS §5 damga kuralı; `docs/SPEC_EDITS_CRITERION.md` bilinmeyen alan netleştirmesi; spec §8 okuyucu kuralı (bilinmeyen alan yok sayılır, bilinmeyen değer reddedilir); `core`/`server` tasdik günlüğü okuyucusu (`deny_unknown_fields` kalkar; bilinen alanın bilinmeyen değeri hata); ileri uyumluluk testi; spec §7.1/§8 durum damgası "Draft (P2-0.2'nin doğrulamasına kadar)"; sürüm CRITERION ile.
**Kabul:** test: `{"x_future": 1}` ekli satır okunur, `partial: false` reddedilir; CRITERION metni gözden geçirici onaylı.

## X4 — Veri setleri ve ölçüm
**Yapar:** `research/eval/datasets/p2_0_2_sets.py` (test-3 vault üreteci: bağlantı biçimleri, kendine bağlantı, çift yeniden adlandırma, klasöre taşıma; D-A′ ek senaryolar); `run_p2_0_2.py --phase test` (ayar yok; W `Z5_tuning.json`'dan); D-A, D-A2, D-A′, D-B test-3 (gerçek Obsidian), D-B2, D-C, D-C′, D-E; `p2_0_2_checks.json`; biçim başına yeniden adlandırma raporu.
**Kabul:** H-P2-0.2a..k ölçüldü; tohum X0'daki; önceki test setleri okunmadı.

## X5 — Doğrulama zinciri
**Yapar:** `verify.sh`: conformance → regress (P2-0.1 referansı) → D-E → D-A/D-A2 → D-C → gerçek Obsidian kümeleri (yoksa "atlandı") → tamper (etiket + metin değişikliği, sahte katman taşıma, bilinmeyen değerli günlük satırı, MCP attest) → pair compare; `VERIFY_LOAD=1` ikinci çift.
**Kabul:** H-P2-0.2i (iki çift), kasıtlı vakalar.

## X6 — Rapor ve hazır
**Yapar:** `report_p2_0_2.py`; `reports/rounds/P2-0.2/FINAL.md` (hipotez tablosu, biçim başına tablo, karar kuralı dalı, NORTH_STAR önerisi); damga **konmaz** (X3 kuralı); `READY.md`; tek PR.
**Kabul:** izlenebilirlik; karar kuralı; NORTH_STAR değiştirilmez.
