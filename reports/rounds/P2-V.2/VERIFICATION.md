# P2-V.2 — Bağımsız Doğrulama (kısa; `docs/rounds/P2-V.2/VERIFY.md` VY1–VY4)

**Doğrulanan commit:** `8ed02efa8e73ebcac946a8ce5fb993cd23fc73ff` (`main`; PR #43, rebase ile birleşti 2026-11-03T10:14:00Z). Ağacı PR head'i `7d8a9b7`'ninkiyle eşit (`44b7d1b0…`); kademe 1'in koştuğu `236c879`'dan sonra yalnız dört belge/rapor dosyası değişti (`docs/ROADMAP.md`, `progress/READY.md`, `progress/Y5.md`, `FINAL.md`). Rebase nedeniyle FINAL'deki commit'ler `main`'de başka adla: Y1 `93db1c6` → `6229677`, Y2 `46b8df8` → `c70da90`, Y5 `236c879` → `98a6825` (ağaç eşit, `e404873d…`).

Ayrı oturum, kendi scratchpad'imde temiz klon (`git clone` GitHub'dan, `8ed02ef`), ana çalışma dizinine erişim yok, kod değiştirilmedi. Kademe 2 tetiklenmedi; Mac kümesi 3 tekrar koşulmadı (VERIFY). Bütün karşılaştırmalar kendi betiklerimle (yaprak düzeyinde JSON farkı, `cmp`, `hashlib.blake2b`); depodaki araçlar yalnız VERIFY'ın adını verdiği komutlar için (`tier2 pair/fails/criteria/equiv`, `eval.portable --all`, `verify_mac reread`, `verify_mac.sh`) koşturuldu; `research/p2v2/reread.py`/`before_after.py`/`name_diff.py` kullanılmadı.

## VY1 — Envanter
Kendi programım (`invdiff.py`: iki JSON'un bütün yapraklarını yol›değer olarak karşılaştırır) `reports/rounds/P2-V.1/inventory.json` ↔ `reports/rounds/P2-V.2/inventory.json`:
- Yaprak 32 336 → 32 415; **silinen 0**, **değişen 6**, **eklenen 79**.
- Değişen 6: üst bilgi (`round`, `task`, `round_start`, `base.file`, `base.rule`) ve `observed_outside_e1›0›status` ("…YOK…" → "adlandırıldı: E1-W (K-P2V1.2)"; eski metin `status_p2v1`'de aynen).
- Eklenen 79, yalnız dört ad ve onların kayıt satırları: **E1-W** (`e1_families.named›0`, R7 `detail`, `p1e1/release/release_check.json`, karar K-P2V1.2 (K-P2V1-T2 (a)), `release_check.sh:71` `*.whl` notu; `counts.E1-W` = 1), **`tamper·det` tarihsel okuma** (`derived_criteria›0` mevcut girdisine `reading: historical`, `reference` {koşu 41239746833 A, blake2b `9aa33c4f…`, `pass` false, `reason` "generated with the p1d-final mdk", iki vaka metni}, `decision` K-P2V1-T3 (c), `until: P2-C`), **H5** (`host_fields›4`, `conformance.json` `fixtures_root`, `class: measured_host_value`, K-P2V1-T4 (a)), **`reported_only`** (`mac_reading.reported_only›0..4`: H-P2-0.3b-yük, H-P2-0.3c, H-P2-0.3d, H-P2-0.3e, `p1e1_checks.json:tamper`; her biri `carry: P2-C`, K-P2V1-T5 (a) şart 1); artı `rules›8`, `rules›9` (T3, T5) ve `p2v2_changes›0..5`, `base.p2v1_base`.
- **Eşikler aynen:** beş `reported_only` eşiği P2-V.1 envanterinin `mac_sets` satırlarıyla karakter karakter aynı (`≤ 0.02 (yüklü çift…)`, `p95 < 500 ms`, `D-C ≥ 0.99; D-C′ ≥ 0.95 ve ajan canonical = 0`, `ajan canonical = 0; sahte 10/10`, tamper e2e metni). Kriter kimlikleri ve karar kimlikleri K-P2V1.2'nin metniyle birebir; başka ad yok.
- `tamper·det` referansının blake2b'si (`9aa33c4f192b0440…be9f2dad`) kendi indirdiğim `41239746833/tier2-A/run/p1e1/tamper_p1e1.json`'ın blake2b'siyle aynı.
- **P2-V ve P2-V.1 envanterleri bayt bayt aynen:** `P2-V/inventory.json` blake2b `a0b14d77…` (`1696e34`'te, `89c4b28`'de, HEAD'de aynı), `P2-V.1/inventory.json` `08c235c8…` (`d7fc44b`, `89c4b28`, HEAD aynı); `89c4b28..HEAD` aralığında ikisine (ve `.md`'lerine) dokunan commit 0.
- **Önce commit'li, sonra değişmemiş:** `git log -- reports/rounds/P2-V.2/inventory.json` tek commit `6229677` (Y1); araç commit'i `c70da90` (Y2) ondan sonra; HEAD'deki dosya `6229677`'dekiyle bayt eşit (`29fa8a95…`). `inventory.md` de tek commit (`6229677`).

## VY2 — Yeniden okumayı kendim yaptım
### İndirdiğim yapıtlar (`gh run download`, her yapıt kendi boş dizinine; blake2b = `hashlib.blake2b`, 64 bayt)
| Koşu (commit) | Dosya | Bayt | Manifestle | Commit'li manifestin blake2b'si (= K-P2V2.1) | Kendi dosya listemin blake2b'si (`yol blake2b\n` sıralı) |
|---|---|---|---|---|---|
| 41239746833 (`58377de`) | 2 712 / 2 712 | 478 598 711 | eksik 0, fazla 0, farklı 0 | `84dffd4284f388accc169ab9972197a1709e6d08297d6aee51ce0db1cea8eeaea2f1c3ed2797bae6e65c11826676a2bf2a40850ee32edbd7eb2ff0cb375b4b4a` | `92b182999f7e7bd12aad0fa7e4b6438baf40d80dcea01bec77634eb145bd634e757a595898f65dc8782e371b6ae5a46a61691e2631fba9070734e38eb86a0ee5` |
| 41239754895 (`58377de`) | 2 712 / 2 712 | 478 590 734 | 0, 0, 0 | `7b06c811380d74f69958eafb2cd4e0c53ea70e5da1f845b88822776c28f6a54f88260c0b632de5f805ede1fcc5975c9aaeafdbd9c8ffb0cdd05b7de94e6488ca` | `fa66655b613c2761d9492aff3ddff5a8246efdca4df95ee821cf28ee81fdc7577da320e9c291af8d1e0e439bb8b879c3ce3120ef7b74664ba09d2a67b890da4e` |
| 41292205176 (`695f952`) | 2 712 / 2 712 | 478 588 133 | 0, 0, 0 | `1b3bf5700eb38211ef2c776b5f81dbfc056e57bfa75733476ea24534db77eba4c08db6719d091e50f7bff31fc93b88b6ac4ac1f32d7451d4d6cc0774d3ba8cee` | `831b8ccc6848f63e0f53b1228ccbc4e9238ecfe5a3df931e218e30a74c1658876118b8a7635b223e73acd11a0d0e872f28b731b6ca50e2cfbecd3da541260496` |
| 40613392932 V0m (`352f790`; iş akışının ref'i `43055ee`, `commit` dosyası `352f790…`) | 2 704 / 2 704 | 478 040 405 | 0, 0, 0 | `dd9563123df982027192e31c696fff5d667f25e5f330f6f98bc6f47cdb5ba60a2b20f7c3665e25079a60ebf980bbf8b5c0d373eadd8a27133b21e09fb4ae80bd` | `d37ebc9e7de84f1823a1a729b5824e8f4e3e2fcad560c2c099111ce7ca2e5908624849832059ae17922ff1160fa43175fb72564407de5e1a172d05c3562a2caa` |

Depoya commit'li üst kayıtlar (`tier2-{A,B}/{commit,exit,host.json,…}`, `tier2-compare/*`) indirdiklerimle `cmp` eşit. Yapıtlar 2027-01-26/30'a kadar indirilebilir (`expired=false`).

### Kademe 2 (her koşu, her envanter; kendi sürücüm, araçların CLI'ı doğrudan)
| Koşu | `tier2 pair` D farkı (alan / bayt; alan) | `tier2 fails` sayılan FAIL A / B — sonra (önce) | `portable --all` bulgu A / B — sonra (önce); atlanan | Kriterler A / B |
|---|---|---|---|---|
| 41239746833 | **0 / 0** (788 890) | **0 / 0** (1 / 1: `tamper·det`); `tamper·det` tarihsel, `record_equal: true` | **0 / 0** (4 / 4, hepsi `conformance.json` `fixtures_root`); `skipped_by_name` H5 | 135 / 135 |
| 41239754895 | **0 / 0** (788 907) | **0 / 0** (1 / 1) | **0 / 0** (4 / 4); H5 | 135 / 135 |
| 41292205176 | **0 / 0** (788 913) | **0 / 0** (1 / 1) | **0 / 0** (4 / 4); H5 | 135 / 135 |

- **`tier2 equiv`** (V0m A ↔ 41239746833 A, `--mac` P2-V.1 kaydı): sonra **adsız D farkı 0** (`d_fields_differing` {}), `pass` true, adlı E1 216 976, Z/O 133, H 2, commit 3; önce adsız 1 (`p1e1/release/release_check.json:items›7›detail`), E1 216 975, `pass` false.
- **Kriter kümesi:** her koşuda her zincirin 135 kimliği (`tamper·det` dahil) P2-V.1'in commit'li `criteria_*.json`'uyla (41292205176 için CI yapıtı `tier2-compare/criteria_*.json`) aynı küme; kaybolan 0.
- **"Önce" (yeni araç + P2-V.1 envanteri) ↔ P2-V.1'in kayıtları** (`reports/rounds/P2-V.1/ci/{41239746833,41239754895}/compare/*`, `…/equiv_v0m.json`; 41292205176 için CI'ın `tier2-compare/*`'ı — değişmemiş araçla CI'da üretilmiş): 22 dosyadan **19 bayt eşit** (equiv, yoğunlaştırılıp depo serileştiricisiyle yazılınca `cmp` eşit), **3** `tier2_pair.json` yalnız `frozen_result_read` yaprağında farklı (komut satırındaki `--frozen` yolunun yankısı; FREEZE_EXCEPTIONS'ta sayılan alan), **0 başka fark**. Bu, F1 ve F2'nin önce/sonra kanıtını kendi koşumla teyit eder (yeni araç eski çıktıyı üretir).
- **"Sonra" (P2-V.2 envanteri) ↔ commit'li `reports/rounds/P2-V.2/reread/<koşu>/*`:** aynı sonuç (19 bayt eşit, 3 yalnız yankı).
- **Önce ↔ sonra yaprak farkı (kendi programım), yalnız dört adda:** `tier2_pair.json` 2 yaprak (`inventory`, `frozen_result_read`; hüküm alanları aynı); `tier2_fails_{A,B}.json` 18 yaprak (`counted`, `counted_fails`, `pass`, `historical`, `derived_criteria›0`) — **T3**; `portable_{A,B}.json` 23 yaprak (`findings` — düşenlerin hepsi `fixtures_root` —, `pass`, `skipped_by_name`) — **T4**; `criteria_*` 0; `equiv` 4 yaprak (`d_fields_differing` R7 `detail` düştü, `pass`, `named_counts›E1` +1) — **T2**. Üç koşuda aynı.

### Mac kaydı (`verify_mac reread`, `reports/rounds/P2-V.1/mac/20261031T204025Z-58377de/mac_checks.json`)
- P2-V.1 envanteriyle çıktı kayda **bayt eşit** (`cmp`) — F3'ün önce/sonra kanıtı teyit.
- P2-V.2 envanteriyle çıktı commit'li `reread/mac_reread.json`'a ve `mac_values.json`'a bayt eşit; çıkış 0.
- Kendi programımla (`maccmp.py`, kanonik JSON): **23/23 kriterin `values`'ı kayıtla eşit**, kimlik kümesi aynı.
- **`reported_only` beşi** `in_verdict: false`, `pass: null`, değer ve eşikleriyle: H-P2-0.3b-yük 0/3, H-P2-0.3c 0/3, H-P2-0.3d 0/3, H-P2-0.3e 0/3, tamper e2e 2/3. C1-8 **1/1** geçti; tarihsel E-7, E1-4, H-P2-0.3b **kayıt eşit**; Z kümeleri 3 tekrar değerleriyle, hükmün dışında; E1-3 3/3, E1-9 3/3 hükümde. **Hüküm geçti** (`in_verdict`: C1-8, E1-3, E1-4, E1-9, E-7).

## VY3 — Kasıtlı bozmalar (kopyalarda; asıl kayıtlar değişmedi — bozmalardan sonra `tamper_p1e1.json` blake2b'si yine `9aa33c4f…`)
- **(a) `tamper·det`** (`tier2 fails`, 41239746833 A): bozmasız → sayılan 0, `record_equal: true`. Envanter kopyasında referans **metni** (`reason`) değişti → **sayılan 1** (`differs: [failing_cases]`), çıkış 3. Referans **blake2b**'si değişti → sayılan 1 (`differs: [blake2b]`). RUN kopyasında kaydın bir vakası bozuldu, `pass` false kaldı (Ş2 türü deterministik bozma) → sayılan 1 (`differs: [blake2b, failing_cases]`). Tarihsel okuma gerçek bir karşılaştırmadır; Ş2'nin kör noktası bu okumayla kapanıyor.
- **(b) H5 sızmıyor** (`portable --all`, 41239746833 A kopyası): `p1b_checks.json`'a host yolu taşıyan alan (`planted_root`) → **4 bulgu, `p1b_checks.json`'da**, çıkış 3; `conformance.json`'da `fixtures_root` dışında bir alan (`fixtures_root_copy`) → **4 bulgu, `conformance.json`'da**, çıkış 3. İkisinde de `skipped_by_name` yalnız H5.
- **(c) Mac kaydı** (`reread`, kayıt kopyası): `reported_only` H-P2-0.3c'nin bir değeri geçen bir değere çevrildi → `reread` onu raporlar (1/3), `in_verdict: false`, **hüküm geçti kalır**; tamper e2e'nin üç değeri `pass: false` → 0/3, hüküm geçti kalır. Deterministik O C1-8'in değeri bozuldu (`pass` false, `differing_outside_e1` dolu) → **hüküm kaldı**; ek olarak E1-3'ün bir tekrarı bozuldu → hüküm kaldı (E1-3 artık hükümde; FINAL "bundan sonra" notuyla tutarlı). Gözlem: yalnız `differing_outside_e1` bozulup değerin `pass`'i true bırakılırsa `reread` hükmü geçti der — `reread` kaydedilmiş değer başına `pass`'i okur, kaydedicinin formülünü yeniden uygulamaz (bkz. öneri 1).
- **(d) Y4 (b)'nin tekrarı** (kendi klonumda, `8ed02ef`; `VERIFY_MAC_REPS=1 ROOTMARK_D10_DEADLINE_MS=1 bash scripts/verify_mac.sh <boş dizin>` — `make verify-mac`'in tarifinin aynısı): RUN `20261103T102853Z-8ed02ef` (tekrar `…102854Z-8ed02ef`). `p1d/d10.json` **var**, `timeout: true`, `deadline_ms` 1, `stopped_at` {edit 0, espresso.md}, `edits` 0, `pass` false (blake2b `1e701f894a03c79b…`); `sigterm.json` var (`pass` true); serve_watch'tan sonraki adımlar koştu (release_local, release + release_check, tamper e2e; öteki kümeler kayıtlı); **`mac_checks.json` var** (23 kriter, `reps` 1, `pass` false; blake2b `b239139827b7e283…`), D-10 satırı 0/1, hata ve günlük kuyruğuyla; **çıkış 3**. Süre **3 057 s** (kurulum ~198 s, tekrar 2 859 s; FINAL'de 3 780 s). Kayıt yollar depo-göreli. Harness durmaları (`run_p2_0_2`, `run_p2_0_3`) kayıtlı, kümeyi düşürmedi — Ş1 giderilmiş.

## VY4 — Sınır, süreç, karar kuralı
- `git diff 89c4b28..8ed02ef -- core index bindings clients spec` **boş** (0 satır). `server/` farkı yalnız `server/tests/serve_watch.rs` (+75 −18): eşik düğmesi `d10_deadline()` (varsayılan 10 000 ms = önceki `from_secs(10)`; koşul `elapsed >= deadline` önceki `assert!(elapsed < 10 s)`'nin tersi), zaman aşımında kayıt + süreci öldürme + panik, SIGTERM başlangıç beklemesinde kayıt, `write_out` yardımcısı; başarı yolunun kayıt anahtarları aynı. Ürün davranışına dokunan satır yok (`server/src` farksız).
- `check_frozen` OK (tablo `89c4b28`'den beri yalnız eklemeli), `check_deps` OK, `check_dep_age` 133 bağımlılık, en az 14 gün, kalış 0. `make lint`: clippy, fmt, ruff, eslint/tsc, `check_deps`, `test_check_deps`, `check_frozen`, `check_compare_fields` temiz; `check-client-types` ilk denemede klonda model dosyaları olmadığından (`serve` başlamıyor) kaldı, modeller (`models verify` ok) konunca **temiz**. `make conformance` **39/39** (seviye 3; hem Y4 klonumda hem bu klonda). `make test` yerelde koşturulmadı; CI'ın test işleri yeşil (aşağıda).
- **FREEZE_EXCEPTIONS** her girdi kendim: F1, F2 — VY2'de yeni araç + P2-V.1 envanteri P2-V.1'in kayıtlarını bayt bayt üretti (yalnız sayılan yankı); F3 — `reread` P2-V.1 envanteriyle `mac_checks.json`'u bayt bayt üretti; F4 — `verify_mac.sh`'ın yorum dışı farkı yalnız `reps` ataması (+ doğrulama) ve dört kayıt denetimi (kayıt varken aynı, yokken `--error` + devam; `cp` aynı koşulun içinde), `bash -n` temiz, `Makefile` farksız, gerçek yol VY3 (d); F5 — fark yukarıda, gerçek yol VY3 (d); F6 — `verify.yml`'nin yorum dışı satırları `89c4b28` ile bayt eşit. `research/FROZEN.md` satırları ve `check_frozen` temiz.
- **Yürürlük metinleri:** DEV_PROCESS §3 "Yürürlük: kademeler P2-V.2 ile yürürlüktedir…", kök `CLAUDE.md` kalite kapıları, ROADMAP §2 (P2-V.2 satırı) ve §4 tutarlı; "yürürlükte değil" ifadesi DEV_PROCESS, `CLAUDE.md`, ROADMAP ve HANDOFF'ta yok; HANDOFF kademelerden söz etmiyor. `tier2.INVENTORY`, `portable.INVENTORY`, `verify_mac.INVENTORY` = `reports/rounds/P2-V.2/inventory.json`.
- **CI:** PR #43 koşusu `41538014984` (`236c879`, pull_request): 10 işin hepsi `success`; iş akışı 22:31:41 → 23:08:35 = **36 dk 54 s** (≤ 40 dk); FAST A/B, karşılaştırma, E2 üç koşucu + karşılaştırma, test + lint + conformance (ubuntu-latest, macos-14), bağımlılık yönü ve yaşı. Birleşmiş `main`'in push koşusu `41617540218` (`8ed02ef`) de **success**.
- **Gözden geçirici:** `docs/rounds/P2-V.2/progress/Y5.review.md` KABUL (okundu; tek tur).

## Kriterler (GOAL P2-V.2) — benim ölçümümle
| Kod | Sonuç | Dayanak |
|---|---|---|
| V2-1 | **geçti** | VY1: tek commit `6229677`, araçlardan önce, sonra değişmedi; fark yalnız dört ek + üst bilgi; P2-V/P2-V.1 envanterleri bayt eşit |
| V2-2 | **geçti** | VY2: D farkı 0 (3/3), sayılan FAIL 0 (A, B, 3/3), bulgu 0 (A, B, 3/3), equiv adsız 0, kriter kümesi aynı (135, `tamper·det` dahil) |
| V2-3 | **geçti** | VY2 Mac: değerler 23/23 eşit; `reported_only` 5, `in_verdict: false`; C1-8 1/1; tarihsel üçü kayıt eşit; Z 3 tekrar değerleriyle; hüküm geçti |
| V2-4 | **geçti** | VY3 (d): `d10.json` `timeout: true`, `mac_checks.json` var, öteki kümeler kayıtlı, çıkış 3; düğmesiz varsayılanlar 3 ve 10 000 (kaynakta okundu), `Makefile` farksız; (a) `pytest tests/test_p2v2_tools.py` klonumda **12/12** geçti |
| V2-5 | **geçti** | VY2: yeni araç + P2-V.1 envanteri → 19/22 bayt eşit, 3 yalnız sayılan yankı, 0 fark; Mac `reread` bayt eşit |
| V2-6 | **geçti** | VY4: ürün farkı boş, `server/` yalnız test dosyası, conformance 39/39, lint temiz; test CI'da yeşil (yerelde koşulmadı) |
| V2-7 | **geçti** | VY4: metinler tutarlı, üç aracın envanter varsayılanı P2-V.2 |
| V2-8 | **geçti** | VY4: FREEZE_EXCEPTIONS F1–F6 kanıtları teyit, üç denetleyici temiz, PR CI yeşil (36 dk 54 s), gözden geçirici KABUL |

**Karar kuralı (GOAL'den kendim türettim):** V2-1..V2-8 geçti → "Hepsi geçti" dalı: kademeler (K-P2V.1) PR #43'ün birleşmesiyle (`8ed02ef`) yürürlükte; P2-V.1 doğrulamasının iki koşulu kapandı; blok sonunun hükmü kademe 2 (`verify.yml`, iki tetikleme) + `make verify-mac`; eski tam çift koşulmaz; P2-C yeni kademelerle açılır. FINAL'in dalıyla **aynı**. Bu doğrulamanın hükmü "güvenilir değil" olmadığından askıya alma dalı uygulanmaz.

## Sonuç
- **İhlal:** yok. **Fark:** FINAL'in sayıları ve dalı bağımsız olarak yeniden üretildi; tek farklar bilgi düzeyinde — Y4 (b) süresi 3 057 s (FINAL 3 780 s), FINAL'in andığı commit'ler rebase yüzünden `main`'de başka sha'larla (yukarıda eşlemesi).
- **Şüphe / gözlem:** (1) `verify_mac reread` kaydedilmiş değer başına `pass`'i okur; ayrıntı alanları `pass`'le çelişecek biçimde bozulursa yakalamaz (VY3 (c) ek sınama) — kaydı `rep` yazdığı için bugünkü kullanımda risk düşük, ama "yeniden okuma" adı yeniden hesaplama izlenimi veriyor. (2) Yerel `make lint`/`make conformance` model dosyaları olmadan `check-client-types` ve C3'te kalıyor; temiz klonda `make models` önkoşul (belgede açık değil).

**Hüküm: güvenilir.**

## Öneriler (≤ 3)
1. P2-C'de `verify_mac reread`, deterministik formülü olan O kriterleri (C1-8: `differing_outside_e1 == []`) için değer başına `pass`'i kayıttaki ölçümlerden yeniden hesaplasın ya da `--help`/belgede "kaydedilmiş `pass`'i okur" diye açıkça yazsın.
2. Rebase ile birleşen turlarda kapanış commit'i (orkestratör) FINAL/DECISIONS'taki dal sha'larını `main`'deki karşılıklarıyla eşleyen bir satır eklesin (bu turda `93db1c6`→`6229677`, `46b8df8`→`c70da90`, `236c879`→`98a6825`); izlenebilirlik bugün yalnız ağaç eşitliğiyle kuruluyor.
3. DEV_PROCESS §5'in doğrulayıcı adımlarına "temiz klonda `make setup` + `make models` (ve `clients/obsidian` için `npm ci`) önce" notu eklensin; aksi halde `make lint`'in `check-client-types` adımı ve `make conformance` ortam yüzünden kalıyor.

## Koşu kimlikleri ve yerel kayıtlar
İndirilen: 41239746833, 41239754895, 41292205176, 40613392932. CI: PR #43 `41538014984`, `main` push `41617540218`. Yerel Y4 (b) RUN `20261103T102853Z-8ed02ef` / `20261103T102854Z-8ed02ef` (scratchpad; depoya alınmadı). Kademe 2 tetiklenmedi; Mac kümesi 3 tekrar koşulmadı.
