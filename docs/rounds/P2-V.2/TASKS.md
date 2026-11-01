# P2-V.2 — Görevler

Genel kurallar:
- Sıra: Y0 → Y1 → Y2 → Y3 → Y4 → Y5. Y1'in envanteri commit'lenmeden Y2'nin araç değişikliklerine geçilmez; Y3 Y2'den sonra (yeni araçla), ama her Y3 çıktısının yanında P2-V.1 envanteriyle alınmış "önce" çıktısı da olur.
- Progress `docs/rounds/P2-V.2/progress/`. Commit öneki `P2-V.2/Y<n>:`.
- Dal `p2-v.2/round`, yalnız birleşmiş `main`'den (K-P2.2). `ROUND_START` = bu turu açan PR'ın birleşmiş `main` HEAD'i.
- Mini tur, araç dozu (K-P2.4): gözden geçirici yalnız Y5'te (tek tur).
- Yeni ölçüm yok: kademe 2 tetiklenmez; Mac kümesi 3 tekrar koşulmaz; tam `make verify` yerelde koşulmaz. Yerelde yalnız FAST, pytest ve Y4'ün tek tekrarlı enjekte koşusu.
- Dondurulmuş kodda her değişiklik `docs/rounds/P2-V.2/progress/FREEZE_EXCEPTIONS.md`'ye önce/sonra kanıtıyla (DEV_PROCESS §4; başlıklar `## F<n> `); `research/FROZEN.md` satırları.
- Yapıtlar `gh run download <koşu> -n tier2-A -n tier2-B -D reports/rounds/P2-V.2/ci/<koşu>/` ile; depoya yalnız RUN kayıtları (JSON) alınır, `chain.log` ve büyük ikili dosyalar `.gitignore` ile dışarıda; her indirilen dosyanın blake2b'si `reports/rounds/P2-V.2/ci/<koşu>/manifest.json`'da.

## Y0 — Açılış
**Yapar:**
- `ROUND_START`; DECISIONS'ta K-P2V2.1 (açılış: `ROUND_START`, indirilen koşular `41239746833`, `41239754895`, `41292205176` ve varsa V0m `40613392932`; manifest blake2b'leri; V0m indirilemiyorsa ASSUMPTIONS Y0-1: equiv okuması `reports/rounds/P2-V.1/ci/41239746833/equiv_v0m.json`'daki fark listesi üzerinedir).
- P2-V.1'in commit'li karşılaştırma çıktıları (`reports/rounds/P2-V.1/ci/<koşu>/compare/*.json`, `equiv_v0m.json`, `mac/.../mac_checks.json`) "önce" referansı olarak listelenir (`progress/Y0.md`).

**Kabul:** yapıtlar yerde, manifest commit'li; K-P2V2.1 yazıldı.

## Y1 — Yeni envanter (ön-kayıt)
**Yapar:**
- `research/p2v2/inventory.py` (dondurulmuş değil): tabanı `reports/rounds/P2-V.1/inventory.json` aynen; dört ek (GOAL Kapsam 2): E1-W (`e1_families` + `observed_outside_e1` durumu), `tamper·det` tarihsel okuma (`derived_criteria` girdisine `reading: "historical"`, `reference` = P2-V.1 kaydının `pass` ve gerekçe metni, `decision: "K-P2V1.2"`, `until: "P2-C"`), H5 (`host_fields`), `mac_reading.reported_only` (beş kriter, kimlik + P2-V eşiği + `carry: "P2-C"`). `rules`'a iki satır: "tarihsel okumalı türev kriter kayıt eşitliğiyle sayılır", "`reported_only` kriterleri değerleriyle raporlanır, hükme girmez".
- `reports/rounds/P2-V.2/inventory.json` + `inventory.md` (dört ek, gerekçe, karar kimlikleri, P2-V.1'den programla fark: yalnız ekler; `research/tests/test_p2v2_tools.py`'de bunu doğrulayan test).
- Commit `P2-V.2/Y1: envanter (ön-kayıt)`; bundan sonra dosya değişmez.

**Kabul:** V2-1'in ilk iki parçası (önce commit'li; programla fark yalnız 4 ek).

## Y2 — Araç değişiklikleri (istisna prosedürüyle)
**Yapar (her biri FREEZE_EXCEPTIONS'ta ayrı F girdisi, önce/sonra kanıtı `reports/rounds/P2-V.2/y2/`):**
- F1 `research/eval/tier2.py`: envanter varsayılanı `reports/rounds/P2-V.2/inventory.json`; `fails` türev kriterin `reading: "historical"` girdisini kayıt eşitliğiyle okur (iki zincirde ve referansla aynı → sayılmaz, "tarihsel" satırı; farklı → sayılır). Önce/sonra: P2-V.1 envanteriyle (`--inventory`) üç koşunun `compare/tier2.json`, `tier2_pair.json`, `tier2_fails_A/B.json` bayt eşit.
- F2 `research/eval/portable.py`: envanterin `host_fields` girdilerinden `class: "measured_host_value"` olanları adıyla atlar ve `skipped_by_name` listesinde raporlar. Önce/sonra: P2-V.1 envanteriyle `portable_A/B.json` bayt eşit.
- F3 `research/eval/verify_mac.py`: envanter varsayılanı P2-V.2; `reported_only` kriterleri `in_verdict: false` ile değer ve eşikleriyle; Ş1: `rep` eksik `d10.json`/`sigterm.json` ya da `timeout: true` kaydını o tekrarın başarısız sonucu olarak hata kuyruğuyla yazar; `reread --checks <mac_checks.json> --out <json>` alt komutu: kayıtlı değerleri değiştirmeden yeni okumayla hükmü üretir. Önce/sonra: P2-V.1 envanteriyle `reread` P2-V.1 `mac_checks.json`'u (değerler ve hüküm) bayt eşit üretir.
- F4 `scripts/verify_mac.sh`: satır 104/109/114/117'deki `exit 1`'ler yerine `errs+=(--error ...)` + devam; `reps=${VERIFY_MAC_REPS:-3}`; envanter yolu yorumu. Önce/sonra: düğmesiz koşuda komutlar ve sıra aynı (`bash -n`, diff'in yalnız bu satırlarda olduğu).
- `server/tests/serve_watch.rs` (dondurulmuş değil; ürün değil): D-10 ve SIGTERM bekleme eşiği `ROOTMARK_D10_DEADLINE_MS` (varsayılan 10 000); zaman aşımında `assert!` öncesi `d10.json`/`sigterm.json` yazılır (`timeout: true`, `pass: false`, o ana kadarki gecikmeler, hangi düzenlemede durduğu); sonra test yine başarısız olur.
- `research/FROZEN.md` satırları; `tests/test_p2v2_tools.py`: F1 tarihsel okuma (referansla aynı → sayılmaz; farklı → sayılır), F2 adıyla atlama (başka host yolu → yine bulgu), F3 eksik kayıt → başarısız tekrar, `reread` değerleri değiştirmiyor.

**Kabul:** V2-5; `uv run pytest` temiz; FREEZE_EXCEPTIONS F1–F4.

## Y3 — Yeniden okuma
**Yapar:**
- Üç koşunun her biri için `reports/rounds/P2-V.2/reread/<koşu>/`: `tier2_pair.json`, `tier2_fails_A.json`, `tier2_fails_B.json`, `portable_A.json`, `portable_B.json`, `tier2_criteria.json`; `41239746833` için ayrıca `equiv_v0m.json` (V0m yapıtı yoksa `equiv_reading.json`: P2-V.1'in fark listesi üzerinde adların uygulanması, yöntem yazılı).
- `reports/rounds/P2-V.2/reread/mac_reread.json` (`verify_mac reread` P2-V.1 Mac kaydı üzerinde).
- `reports/rounds/P2-V.2/reread/summary.md`: koşu başına D farkı, sayılan FAIL, bulgu, adsız fark; Mac hükmü; "önce" (P2-V.1 envanteri) ile "sonra" (P2-V.2) farkının yalnız dört adda olduğu satır satır.

**Kabul:** V2-2, V2-3 değerleri kaydedildi (geçti ya da kaldı; kalırsa ad eklenmez, karar kuralı).

## Y4 — Ş1 testi
**Yapar:**
- (a) pytest (Y2'de yazıldı) geçiyor.
- (b) `VERIFY_MAC_REPS=1 ROOTMARK_D10_DEADLINE_MS=1 make verify-mac` (boş çalışma dizini); çıktı `reports/rounds/P2-V.2/y4/mac_checks.json` + `verify_mac.log` + `d10.json`; süre.
- (c) `bash scripts/verify_mac.sh` düğmesiz davranışının aynı olduğunu diff ve `reps`/eşik varsayılanlarıyla göster (`y4/defaults.md`); tam koşu yapılmaz.

**Kabul:** V2-4.

## Y5 — Yürürlük, gözden geçirici, rapor, hazır
**Yapar:**
- DEV_PROCESS §3: "Yürürlük" paragrafı → kademeler P2-V.2 ile yürürlükte (K-P2V1.2, K-P2V2.1); blok sonu hükmü kademe 2 iki tetikleme + Mac kümesi; eski tam çift koşulmaz. Kök `CLAUDE.md` kalite kapıları satırı. ROADMAP §2 P2-V.2 satırı (sonuç), §4 notu. `docs/HANDOFF.md` yürürlük notu (varsa). `verify.yml`, `verify_mac.sh` yorumlarındaki envanter yolu.
- `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- Gözden geçirici tek tur; başlık: "adlar K-P2V1.2'nin dışına taştı mı; önce/sonra kanıtı bayt eşit mi; Ş1 yolu kayıt yazıyor ve küme devam ediyor mu; yürürlük metinleri tutarlı mı".
- `reports/rounds/P2-V.2/FINAL.md` (kriter tablosu V2-1..V2-8, karar kuralının dalı, koşu kimlikleri, süreler); READY; tek PR (`gh pr create --base main --head p2-v.2/round --title "P2-V.2: kademelerin yürürlüğe girmesi" --body-file reports/rounds/P2-V.2/FINAL.md`).

**Kabul:** V2-6, V2-7, V2-8; READY "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
