# P2-V — Zincir envanteri ve sınıflandırma (V1, ön-kayıt)

`ROUND_START` `352f790`. Makine okunur kaynak `inventory.json` (bu dosya onun okunur biçimi); ikisi de V2'den önce commit'lenir ve sonra değişmez (GOAL P2-V V1, V-1). Kademe 2'nin karşılaştırıcısı (`eval.tier2`) ve Mac kümesi (`eval.verify_mac`) listeleri doğrudan `inventory.json`'dan okur.

## Sınıflar
- **D** — deterministik: aynı girdi → aynı çıktı; kademe 2 A↔B bayt/alan eşitliği
- **Z** — zamanlamaya duyarlı (pencere, gecikme, yarış): karşılaştırma dışı, Mac kümesinde kendi eşiğiyle 3 tekrar
- **O** — gerçek Obsidian ya da macOS'a özgü yol — ve referans RUN'ları yayım Mac'inde kaydedilmiş, değeri o donanıma/platforma bağlı kriterler (vektör sayısalları C1-8, yerel yayım hedefi E-7, yayım kontrol listesi E1-4): Mac kümesinde kendi eşiğiyle; gerçek Obsidian alanları karşılaştırma dışı, platforma bağlı kriterlerin kayıtları aynı platformdaki iki zincirde deterministik olduğu için karşılaştırmada kalır (yalnız kademe 2'nin FAIL sayımından düşer)
- **H** — host'a bağlı değer: kayıt düzeltilir (V3), sonra D

## Kurallar
- Karşılaştırma dondurulmuş `eval.compare_p2r.pair`'dir (eski tam çiftin her dosyası ve alanı); `eval.tier2 pair` yalnız aşağıda adı geçenleri kenara alır.
- Her girdi bir dosya ya da bir alan (alt ağacıyla); desen yok (`*`, `?`, `[` hata).
- Gömülü fark kuralı: başka kayıtların alan farklarını `<dosya>:<alan>` anahtarıyla taşıyan bir alan (`differing_fields`, `excluded_differences`) içinde bir girdi, adlandırdığı alan kenara alınmışsa kenara alınır (özyinelemeli).
- Dondurulmuş karşılaştırıcıların kendi zaman/ortam/bilgi alanları (DETERMINISTIC_FIELDS.md) aynen dışarıda kalır; bu listeye eklenmez.
- Kademe 2'nin sayılan FAIL'i: zincirin kendi sınıflandırıcısının (`eval.historical_p2r`) sayılan FAIL'lerinden `tier2_set_aside_criteria` (Mac kümesi ölçer) ve bildirdiği her farkı gömülü kuralla kenara alınan bir Z/O alanı olanlar düşülür; geri kalan sayılır.
- Eski ↔ yeni eşdeğerlik (V-3, `eval.tier2 equiv`): farklar yalnız Z/O, H ve `commit_bound` alanlarında; kriter kimlikleri eski ⊆ yeni ∪ Mac kümesi.

## Zincirin adımları (`scripts/verify.sh chain`, `352f790`) ve süreleri (s)
| Adım | Koşan | Girdiler | Platform gereksinimi | P2-R 3. deneme A | P2-R 3. deneme B | CI 40265181721 A | CI 40265181721 B | V0 40582139980 A | V0 40582139980 B |
|---|---|---|---|---|---|---|---|---|---|
| `clone` | `git clone` | kaynak depo (commit) | - | 3 | 3 | 1 | 1 | 0 | 0 |
| `setup` | `make setup` | Cargo.lock, uv.lock; Rust + maturin | - | 45 | 44 | 217 | 186 | 206 | 195 |
| `models` | `make models` | models/MANIFEST.json (ağ: HF) | ağ | 12 | 12 | 25 | 26 | 28 | 26 |
| `datasets` | `make datasets` | research/data manifest'leri | - | 58 | 60 | 23 | 22 | 23 | 21 |
| `freeze` | `make freeze-check freeze-check-v3; scripts/check_frozen.sh` | FROZEN.md, v2/v3 fixture'ları | - | 68 | 69 | 38 | 37 | 38 | 32 |
| `conformance` | `make conformance` | spec/fixtures, modeller | - | 18 | 18 | 29 | 30 | 29 | 29 |
| `holdout` | `make eval-holdout eval-split-grid` | d1ho, d1sh | - | 28 | 27 | 3 | 3 | 3 | 3 |
| `regress` | `make regress` | v2/v3 RUN'ları, P1-A RUN'ı | - | 362 | 363 | 213 | 217 | 136 | 135 |
| `p1a` | `make p1a-checks` | RUN kayıtları | - | 24 | 22 | 12 | 13 | 12 | 11 |
| `p1b` | `make p1b-checks` | v3-final ikilisi, fuzz tohumları | - | 45 | 46 | 64 | 64 | 66 | 60 |
| `p1b1` | `make p1b1-checks` | RUN kayıtları | - | 2 | 2 | 1 | 1 | 2 | 1 |
| `p1c` | `make p1c-checks` | D3, D4, model | vektör sayısalları donanıma bağlı (C1-8'in referansı yayım Mac'inde) | 511 | 511 | 1014 | 997 | 974 | 869 |
| `tamper_p1c` | `scripts/test_tamper_p1c.sh` | model manifest, fixture'lar | - | 23 | 22 | 12 | 12 | 12 | 11 |
| `p1c1` | `make p1c1-checks` | P1-C RUN'ı, CLI | - | 64 | 63 | 52 | 50 | 45 | 42 |
| `p1d` | `make p1d-checks` | MCP Inspector (ağ, Node), serve --watch (FSEvents/inotify), P1-C.1 RUN'ı | ağ; izleme gecikmesi (D-10) | 98 | 101 | 179 | 163 | 177 | 156 |
| `p1d1` | `make p1d1-checks` | p1d-final ağacı, P1-D RUN'ı | - | 149 | 152 | 278 | 279 | 276 | 257 |
| `p1e` | `make p1e-checks` | eklenti e2e (başsız), yerel yayım ikilisi, P1-D.1 RUN'ı | yerel yayım hedefi aarch64-apple-darwin (E-7) | 83 | 82 | 175 | 183 | 192 | 176 |
| `p1e1` | `make p1e1-checks` | yayım paketi (release.sh), gerçek Obsidian e2e ve kasıtlı bozmaları, P1-E RUN'ı | gerçek Obsidian (E1-3, E1-9 adımı, R15, tamper e2e); macOS arşivi (R17, E1-4) | 154 | 152 | 909 | 897 | 884 | 838 |
| `p2a` | `make p2a-checks (içinde p2-0-2-checks)` | serve --watch kümeleri D-A, D-A2, D-C, D-B2; altı eski vault; P2-0.2 RUN'ı | izleme gecikmesi ve pencere yarışı (D-A quiesce 0.4 s) | 835 | 952 | 1849 | 1904 | 1948 | 1904 |
| `p2_0_3` | `make p2-0-3-checks` | gerçek Obsidian: D-B test-4, D-A′, D-C′; P2-A RUN'ı | gerçek Obsidian (macOS) | 747 | 747 | 3 | 3 | 3 | 2 |
| `p2l` | `make p2l-checks p2l-quality` | çok dilli model (ağ), D-TR | ağ | 369 | 369 | 597 | 576 | 655 | 558 |
| `p2p` | `make p2p-checks` | sunucu testleri | - | 46 | 47 | 31 | 30 | 34 | 29 |
| `p2b` | `make p2b-checks` | D-NOTES, iki istemci (Node), başsız e2e | - | 173 | 174 | 185 | 183 | 184 | 156 |
| `p2r` | `make p2r-checks` | D-VOL tohumu | - | 464 | 468 |  |  | 661 | 572 |
| `tamper` | `scripts/test_tamper.sh` | RUN dosyaları, FROZEN.md | - | 193 | 195 | 126 | 141 | 113 | 115 |
| `summary` | `python -m eval.historical_p2r --summary` | P2-B resmi RUN'ı, listeler | - | 0 | 0 | 40 | 42 | 39 | 39 |
| **toplam** | | | | 4574 | 4701 | 6076 | 6060 | 6740 | 6237 |

Süre kaynağı: günlüklerdeki `[verify HH:MM:SS]` satırları (adımın başından sonrakinin başına). `p2r` adımı `ae7e692`'nin zincirinde yoktu (CI 40265181721).

## Karşılaştırma dışı alanlar (Z/O; adlı, gerekçeli)
| Kimlik | Sınıf | Küme | Dosya | Alan (alt ağacıyla) | Gerekçe |
|---|---|---|---|---|---|
| Z1 | Z | Z-D10 | `p1d/d10.json` | (bütün dosya) | D-10: dış md düzenlemesinden `search`'e gecikme (p95) ve geçiş kararı; serve --watch altında dosya olayı (FSEvents/inotify) ile ölçüm arasındaki zamanlama; kayıt girdiye ek olarak zamanlamanın fonksiyonu |
| Z2 | Z | Z-D10 | `p1d/sigterm.json` | (bütün dosya) | D-10'un SIGTERM yarısı: `serve --watch` durdurulurken yazılan değişikliğin kaydı (zamanlama) |
| Z3 | Z | Z-D10 | `p1d_checks.json` | `D-10` | D-10 kriterinin kaydı (Z1, Z2'den) |
| Z4 | Z | Z-DA | `p2_0_2/d_a.json` | (bütün dosya) | D-A: 200 senaryo, dondurulmuş `Served.quiesce(settle_s=0.4)`; FSEvents geç gelirse ölçüm alımdan önce (K-P2R.5 (1), P2-0 Z5 N5) |
| Z5 | Z | Z-DA | `p2_0_2/d_a2.json` | (bütün dosya) | D-A2: D-A ile aynı mekanizma (quiesce) |
| Z6 | Z | Z-DC | `p2_0_2/d_c.json` | (bütün dosya) | D-C: insan/ajan yazımı arasındaki gecikmeler ve çakışma (collide) tikleri; serve --watch altında dosya olayı (FSEvents/inotify) ile ölçüm arasındaki zamanlama; kayıt girdiye ek olarak zamanlamanın fonksiyonu |
| Z7 | Z | Z-DA | `p2_0_2_checks.json` | `H-P2-0·2a` | H-P2-0.2a: Z4, Z5'ten |
| Z8 | Z | Z-DA | `p2_0_2_checks.json` | `H-P2-0·2a′` | H-P2-0.2a′: Z4'ten |
| Z9 | Z | Z-DC | `p2_0_2_checks.json` | `H-P2-0·2d` | H-P2-0.2d: Z6'dan (D-C′ yarısı O, zincirde ölçülmez) |
| Z10 | Z | Z-DB2 | `p2_0_2_checks.json` | `H-P2-0·2f` | H-P2-0.2f: D-B2 (Obsidian dışı insan oturumları, `serve --watch`) yanlış `unverified` ve p95 gecikmesi |
| Z11 | Z | Z-DA | `p2_0_2_checks.json` | `regress›p2_0_1_sets›d_a` | P2-0.1 RUN'ına karşı D-A kümesi (Z4'ün regress'i) |
| Z12 | Z | Z-DA | `p2_0_2_checks.json` | `regress›p2_0_1_sets›d_a2` | P2-0.1 RUN'ına karşı D-A2 (Z5'in regress'i) |
| Z13 | Z | Z-DC | `p2_0_2_checks.json` | `regress›p2_0_1_sets›d_c` | P2-0.1 RUN'ına karşı D-C (Z6'nın regress'i) |
| Z14 | Z | Z-DA | `p2_0_2_checks.json` | `regress›p2_0_1_sets›equal` | Z11–Z13'ün birleşimi (`d_e_p2_0_1` ayrıca karşılaştırılır) |
| Z15 | Z | Z-DA | `p2_0_2_checks.json` | `regress›pass` | regress'in `pass`'ı Z alanlarına da bağlı; D içeriği alan alan (gömülü fark kuralı) karşılaştırılır |
| Z16 | Z | FAIL özeti | `p2_0_2_checks.json` | `fail_summary` | P2-0.2'nin sınıflandırıcısının FAIL özeti: Z/O kriterlerini sayar; D kriterlerinin kayıtları ayrı ayrı karşılaştırılır ve kademe 2 FAIL'leri D sınıfında yeniden sayar (`eval.tier2 fails`) |
| Z17 | Z | FAIL özeti | `p2a_checks.json` | `fail_summary` | P2-A sınıflandırıcısının FAIL özeti (Z16 gibi) |
| Z18 | Z | Z-DA | `p2a_checks.json` | `regress›p2_0_2_bytes` | D-A, D-A2, D-C günlüklerinin P2-0.2 RUN'ına bayt eşitliği (Z4–Z6'nın günlükleri) |
| Z19 | Z | Z-DA | `p2a_checks.json` | `regress›pass` | regress'in `pass`'ı (Z15 gibi) |
| Z20 | Z | Z-DA | `p2a_checks.json` | `regress›excluded_differences›p2_0_2_checks·json:regress›leaves` | P2-0.2 regress'inin karar dışlamalarındaki yaprak sayısı: Z alanlarının farkını sayar |
| Z21 | Z | Z-D10 | `p1d1_checks.json` | `D1-12›pass` | D1-12: P1-D RUN'ına karşı regress; `pass`'ı D-10'u taşır (P2-R FINAL not 9); D içeriği gömülü fark kuralıyla |
| Z22 | Z | Z-D10 | `p1d1_checks.json` | `D1-12›regress_vs_p1d›equal` | Z21'in `equal`'ı |
| Z23 | Z | Z-D10 | `p1e_checks.json` | `E-9›pass` | E-9: P1-D.1 RUN'ına karşı regress; gömülü farkları D-10'u ve D1-12'yi taşır |
| Z24 | Z | Z-D10 | `p1e_checks.json` | `E-9›regress_vs_p1d1›equal` | Z23'ün `equal`'ı |
| Z25 | Z | Z-D10 | `p1e1_checks.json` | `E1-6›pass` | E1-6: P1-E RUN'ına karşı regress; gömülü farkları D-10'u taşır |
| Z26 | Z | Z-D10 | `p1e1_checks.json` | `E1-6›regress_vs_p1e›equal` | Z25'in `equal`'ı |
| Z27 | Z | FAIL özeti | `p1e1/historical.json` | (bütün dosya) | P1-E.1 sınıflandırıcısının FAIL özeti (Z16 gibi; D-10'u ve gerçek Obsidian kayıtlarını sayar) |
| O1 | O | O-REL | `p1e1/release/plugin_e2e_real.json` | (bütün dosya) | gerçek Obsidian e2e (K-E1.3): ayrı örnek, CDP; Obsidian yoksa `skipped` |
| O2 | O | O-REL | `p1e1/release/access_log_real.json` | (bütün dosya) | O1'in erişim günlüğü |
| O3 | O | O-REL | `p1e1_checks.json` | `E1-3` | E1-3: O1'den |
| O4 | O | O-REL | `p1e1_checks.json` | `E1-9›connection_step` | E1-9'un e2e adımı (O1'den); `contract_check_pass` D, karşılaştırılır |
| O5 | O | O-REL | `p1e1_checks.json` | `E1-9›skipped` | E1-9'un e2e adımı ölçüldü mü (O1) |
| O6 | O | O-REL | `p1e1_checks.json` | `E1-9›pass` | E1-9'un `pass`'ı O1'e bağlı; D yarısı `contract_check_pass` |
| O11 | O | O-REL | `p1e1/tamper_p1e1_e2e.json` | (bütün dosya) | P1-E.1'in gerçek Obsidian kasıtlı bozmaları (`scripts/test_tamper_p1e1_e2e.sh`); Obsidian yoksa `real_cases_run: false` |
| O12 | O | O-REL | `p1e1_checks.json` | `tamper` | P1-E.1 `tamper`: `tamper_p1e1.json` (D, ayrıca karşılaştırılır) ile O11'in birleşimi |
| O13 | O | O-REL | `p1e1/release/release_check.json` | `items›15` | R15: yayım paketinin gerçek Obsidian e2e'si (O1); Obsidian yoksa `skipped` |
| O7 | O | O-DB4 | `p2_0_2_checks.json` | `H-P2-0·2b` | D-B test-3, gerçek Obsidian; K-P2.3'ten beri zincirde ölçülmez (`skipped`); ölçümü P2-0.3'ün D-B test-4'ü (Mac kümesi) |
| O8 | O | O-DB4 | `p2_0_2_checks.json` | `H-P2-0·2b′` | O7 gibi |
| O9 | O | O-DB4 | `p2_0_2_checks.json` | `H-P2-0·2c` | O7 gibi (tasdik gecikmesi) |
| O10 | O | O-DA′ | `p2_0_2_checks.json` | `H-P2-0·2e` | D-A′, gerçek Obsidian; O7 gibi, ölçümü P2-0.3'ün D-A′'sü |
| Z28 | Z | Z-DA | `p2_0_2/d_a/events.jsonl` | (bayt) | D-A günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |
| Z29 | Z | Z-DA | `p2_0_2/d_a/attestations.jsonl` | (bayt) | D-A günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |
| Z30 | Z | Z-DA | `p2_0_2/d_a2/events.jsonl` | (bayt) | D-A2 günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |
| Z31 | Z | Z-DA | `p2_0_2/d_a2/attestations.jsonl` | (bayt) | D-A2 günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |
| Z32 | Z | Z-DC | `p2_0_2/d_c/events.jsonl` | (bayt) | D-C günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |
| Z33 | Z | Z-DC | `p2_0_2/d_c/attestations.jsonl` | (bayt) | D-C günlüğü (H-P2-0.2i); Z4–Z6'nın vault'u |

## Host'a bağlı alanlar (H; V3'te düzeltilir)
| Kimlik | Dosya | Alan | Nerede | Düzeltme |
|---|---|---|---|---|
| H1 | `p1e1/release/release_check.json` | `items›17›detail` | scripts/release_check.sh R17 (dondurulmuş; istisna P2-V/F1) | paketin dizini ve geçici dizin yerine sabit metin (`the package's bin/`, `<tmp>`, `<dist>`, `<repo>`); Mac arşivi olmayan hostta klon yolunu yazıyordu (K-P2R.5 (2), CI 40265181721) |
| H2 | `conformance_p1d1.json` | `fixtures_root` | Makefile p1c-checks (RUN'a kopya) | depo köküne göreli (`spec/fixtures`); `reports/conformance.json` mutlak kalır (dondurulmuş `make conformance` denetimi onu okur); dondurulmuş karşılaştırıcılar bu alanı zaten `ortam` sayar (DETERMINISTIC_FIELDS) |
| H3 | `models_verify.json` | `root` | Makefile p1c-checks | depo köküne göreli (`index/../models`); çift karşılaştırmasında değil, Blok 2 doğrulaması önerisi 2 |
| H4 | `fast/conformance.json` | `fixtures_root` | scripts/verify.sh chain-fast (dondurulmuş; istisna P2-V/F2) | H2 gibi; kademe 1'in kaydı |

## Commit'e bağlı D alanları (yalnız eski ↔ yeni eşdeğerliğinde adıyla beklenen fark)
- C1 `p1e1/release/release.json` `commit` — paketin commit'i: girdisi commit kimliğinin kendisi (D: aynı commit'te iki zincir eşit)
- C2 `p1e1/release/release_check.json` `items›3›item` — R4 maddesinin metni paketin commit'ini adlandırır (`inputs of <sha> = HEAD`)

## Mac kümesi (`make verify-mac`; 3 tekrar, kaynak eşiği)
| Küme | Sınıf | Ölçüm | Kriter | Eşik | Kaynak |
|---|---|---|---|---|---|
| Z-D10 | Z | P1-D D-10 (`serve_watch` testi) | `p1d_checks.json` D-10 | p95 < 1000 ms, katman doğru (SIGTERM'de kayıp yok) | `docs/rounds/P1-D/GOAL.md:48` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_2_checks.json` H-P2-0.2a | = 0 (canonical sızıntısı) | `docs/rounds/P2-0.2/GOAL.md:31` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_2_checks.json` H-P2-0.2a′ | 200/200 | `docs/rounds/P2-0.2/GOAL.md:32` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_3_checks.json` H-P2-0.3a | = 0 | `docs/rounds/P2-0.3/GOAL.md:17` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_3_checks.json` H-P2-0.3a′ | 200/200 | `docs/rounds/P2-0.3/GOAL.md:18` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_2_checks.json` H-P2-0.2i | **eşik yok** — eşiği iki zincirin günlüklerinin bayt eşitliği; tekrarlar arası eşitlik raporlanır | `docs/rounds/P2-0.2/GOAL.md:40` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_3_checks.json` H-P2-0.3i | **eşik yok** — H-P2-0.2i gibi | `docs/rounds/P2-0.3/GOAL.md:27` |
| Z-DA | Z | P2-0.2 D-A, D-A2 (`serve --watch`, quiesce) | `p2_0_3_checks.json` logs_vs_p2a | **eşik yok** — D-A, D-A2, D-C günlüklerinin P2-A resmi RUN'ına bayt eşitliği; GOAL eşiği değil, kaydedicinin denetimi | `research/eval/run_p2_0_3.py (`_logs`)` |
| Z-DC | Z | P2-0.2 D-C (`serve --watch`) | `p2_0_2_checks.json` H-P2-0.2d | D-C ≥ 0.99 (D-C′ yarısı H-P2-0.3d) | `docs/rounds/P2-0.2/GOAL.md:36` |
| Z-DB2 | Z | P2-0.2 D-B2 (Obsidian dışı insan, `serve --watch`) | `p2_0_2_checks.json` H-P2-0.2f | ≤ 0.01; p95 < 500 ms | `docs/rounds/P2-0.2/GOAL.md:38` |
| Z-DB2 | Z | P2-0.2 D-B2 (Obsidian dışı insan, `serve --watch`) | `p2_0_3_checks.json` H-P2-0.3f | ≤ 0.01; p95 < 500 ms | `docs/rounds/P2-0.3/GOAL.md:25` |
| O-DB4 | O | P2-0.3 D-B test-4 (gerçek Obsidian) | `p2_0_3_checks.json` H-P2-0.3b | ≤ 0.01 | `docs/rounds/P2-0.3/GOAL.md:19` |
| O-DB4 | O | P2-0.3 D-B test-4 (gerçek Obsidian) | `p2_0_3_checks.json` H-P2-0.3b-yük | ≤ 0.02 (yüklü çift; Mac kümesi yüksüz koşar, değer bu eşikle raporlanır) | `docs/rounds/P2-0.3/GOAL.md:20` |
| O-DB4 | O | P2-0.3 D-B test-4 (gerçek Obsidian) | `p2_0_3_checks.json` H-P2-0.3b′ | **eşik yok** — GOAL: raporlanır (bilgi) | `docs/rounds/P2-0.3/GOAL.md:21` |
| O-DB4 | O | P2-0.3 D-B test-4 (gerçek Obsidian) | `p2_0_3_checks.json` H-P2-0.3c | p95 < 500 ms | `docs/rounds/P2-0.3/GOAL.md:22` |
| O-DC′ | O | P2-0.3 D-C′ (gerçek Obsidian) ve D-C | `p2_0_3_checks.json` H-P2-0.3d | D-C ≥ 0.99; D-C′ ≥ 0.95 ve ajan canonical = 0 | `docs/rounds/P2-0.3/GOAL.md:23` |
| O-DA′ | O | P2-0.3 D-A′ (gerçek Obsidian) | `p2_0_3_checks.json` H-P2-0.3e | ajan canonical = 0; sahte 10/10 | `docs/rounds/P2-0.3/GOAL.md:24` |
| O-REL | O | P1-E/P1-E.1 yerel yayım ikilisi ve yayım paketi, gerçek Obsidian e2e'si ve kasıtlı bozmaları (`release_local.sh`, `release.sh`, `release_check.sh`, `test_tamper_p1e1_e2e.sh`) | `p1e_checks.json` E-7 | 4/4 (yayım paketi; K-E5-T ile ertelendi: kayıtlı 2/4) | `docs/rounds/P1-E/GOAL.md:36` |
| O-REL | O | P1-E/P1-E.1 yerel yayım ikilisi ve yayım paketi, gerçek Obsidian e2e'si ve kasıtlı bozmaları (`release_local.sh`, `release.sh`, `release_check.sh`, `test_tamper_p1e1_e2e.sh`) | `p1e1_checks.json` E1-3 | 11/11 | `docs/rounds/P1-E.1/GOAL.md:16` |
| O-REL | O | P1-E/P1-E.1 yerel yayım ikilisi ve yayım paketi, gerçek Obsidian e2e'si ve kasıtlı bozmaları (`release_local.sh`, `release.sh`, `release_check.sh`, `test_tamper_p1e1_e2e.sh`) | `p1e1_checks.json` E1-4 | geçer (kontrol listesinin bütün maddeleri) | `docs/rounds/P1-E.1/GOAL.md:17` |
| O-REL | O | P1-E/P1-E.1 yerel yayım ikilisi ve yayım paketi, gerçek Obsidian e2e'si ve kasıtlı bozmaları (`release_local.sh`, `release.sh`, `release_check.sh`, `test_tamper_p1e1_e2e.sh`) | `p1e1_checks.json` E1-9 | 1/1 (e2e adımı; sözleşme yarısı kademe 2'de) | `docs/rounds/P1-E.1/GOAL.md:22` |
| O-REL | O | P1-E/P1-E.1 yerel yayım ikilisi ve yayım paketi, gerçek Obsidian e2e'si ve kasıtlı bozmaları (`release_local.sh`, `release.sh`, `release_check.sh`, `test_tamper_p1e1_e2e.sh`) | `p1e1_checks.json` tamper | gerçek Obsidian kasıtlı vakaları reddedilir, kontroller geçer (e2e yarısı; `tamper_p1e1.json` kademe 2'de) | `research/eval/run_p1e1_checks.py (`tamper`)` |
| O-C18 | O | P1-C.1 C1-8: P1-C kayıtları P1-C resmi RUN'ına karşı (vektörler ve skorlar referans Mac'in donanımına bağlı; zincirin `holdout`…`p1c1-checks` adımları) | `p1c1_checks.json` C1-8 | fark 0 (karar dışlamalarıyla) | `docs/rounds/P1-C.1/GOAL.md:22` |

Kademe 2'nin sayımında türev olarak (gömülü fark kuralıyla) kenara alınabilenler: D-9, D1-12, E-9, E1-6 ve iki regress — yalnız bildirdikleri her fark bir Z/O alanında ya da Z/O kriterinin kaydındaysa. Kademe 2'de sayılan FAIL'den kenara alınan ama Mac kümesinde ayrıca koşmayan dört kriter: `p2_0_2_checks.json` H-P2-0.2b, b′, c (D-B test-3) ve e (D-A′) — K-P2.3'ten beri hiçbir zincirde ölçülmez (`skipped`); ölçümleri P2-0.3'ün karşılıklarıdır (O-DB4, O-DA′). Kimlikleri kademe 2'nin kayıtlarında sürer.

## Kayıt dosyaları (çift karşılaştırmasına girenler)
| Dosya | Sınıf | Kenara alınan alanlar | H | Commit'e bağlı |
|---|---|---|---|---|
| `h1s_tuning.json` | D | - | - | - |
| `h1s.json` | D | - | - | - |
| `h1r.json` | D | - | - | - |
| `h1s_v3_tuning.json` | D | - | - | - |
| `h1s_v3.json` | D | - | - | - |
| `h1r_v3.json` | D | - | - | - |
| `h3_v3.json` | D | - | - | - |
| `h4_v3.json` | D | - | - | - |
| `h4_v2repro.json` | D | - | - | - |
| `holdout.json` | D | - | - | - |
| `split_grid.json` | D | - | - | - |
| `holdout_split.json` | D | - | - | - |
| `p1b_checks.json` | D | - | - | - |
| `governance/governance.json` | D | - | - | - |
| `p1b1_checks.json` | D | - | - | - |
| `c3_quality.json` | D | - | - | - |
| `p1c_checks.json` | D | - | - | - |
| `p1c_index.json` | D | - | - | - |
| `c1_variant.json` | D | - | - | - |
| `p1c1_checks.json` | D | - | - | - |
| `p1d_checks.json` | D | Z3 | - | - |
| `p1d/inspector/inspector.json` | D | - | - | - |
| `p1d/mcp_e2e/mcp_e2e.json` | D | - | - | - |
| `p1d/http/http_e2e.json` | D | - | - | - |
| `p1d/scene/scene.json` | D | - | - | - |
| `p1d/tamper_p1d.json` | D | - | - | - |
| `p1d/d10.json` | Z | - | - | - |
| `p1d/sigterm.json` | Z | - | - | - |
| `p1d1_checks.json` | D | Z21, Z22 | - | - |
| `conformance_p1d1.json` | D | - | H2 | - |
| `p1d1/p1d1.json` | D | - | - | - |
| `p1d1/d1_1.json` | D | - | - | - |
| `p1d1/d1_2.json` | D | - | - | - |
| `p1d1/d1_3.json` | D | - | - | - |
| `p1d1/d1_4.json` | D | - | - | - |
| `p1d1/d1_5.json` | D | - | - | - |
| `p1d1/d1_6.json` | D | - | - | - |
| `p1d1/d1_7.json` | D | - | - | - |
| `p1d1/d1_8.json` | D | - | - | - |
| `p1d1/d1_9.json` | D | - | - | - |
| `p1d1/d1_11.json` | D | - | - | - |
| `p1d1/d1_13.json` | D | - | - | - |
| `p1d1/v3_compat.json` | D | - | - | - |
| `p1d1/v3_compat_mixed.json` | D | - | - | - |
| `p1d1/tamper_p1d1.json` | D | - | - | - |
| `p1e_checks.json` | D | Z23, Z24 | - | - |
| `p1e/e0/e14.json` | D | - | - | - |
| `p1e/e0/e15.json` | D | - | - | - |
| `p1e/scene/scene.json` | D | - | - | - |
| `p1e/plugin_e2e/plugin_e2e.json` | D | - | - | - |
| `p1e/plugin_e2e/access_log.json` | D | - | - | - |
| `p1e/client.json` | D | - | - | - |
| `p1e/release/release_local.json` | D | - | - | - |
| `p1e/release/conformance_rootmark.json` | D | - | - | - |
| `p1e/release/conformance_equiv.json` | D | - | - | - |
| `p1e/tamper_p1e.json` | D | - | - | - |
| `p1e/check_compare_fields.json` | D | - | - | - |
| `p1e1_checks.json` | D | Z25, Z26, O3, O4, O5, O6, O12 | - | - |
| `p1e1/status/e1_1.json` | D | - | - | - |
| `p1e1/status/e1_2.json` | D | - | - | - |
| `p1e1/contract.json` | D | - | - | - |
| `p1e1/tamper_p1e1.json` | D | - | - | - |
| `p1e1/tamper_p1e1_e2e.json` | O | - | - | - |
| `p1e1/release/release.json` | D | - | - | C1 |
| `p1e1/release/release_check.json` | D | O13 | H1 | C2 |
| `p1e1/release/plugin_e2e_real.json` | O | - | - | - |
| `p1e1/release/access_log_real.json` | O | - | - | - |
| `p1e1/historical.json` | Z | - | - | - |
| `p1e1/check_compare_fields.json` | D | - | - | - |
| `p2_0_2_checks.json` | D | Z7, Z8, Z9, Z10, Z11, Z12, Z13, Z14, Z15, Z16, O7, O8, O9, O10 | - | - |
| `p2_0_2/rules.json` | D | - | - | - |
| `p2_0_2/d_e.json` | D | - | - | - |
| `p2_0_2/d_a.json` | Z | - | - | - |
| `p2_0_2/d_a2.json` | Z | - | - | - |
| `p2_0_2/d_c.json` | Z | - | - | - |
| `p2_0_2/tamper_p2_0_2.json` | D | - | - | - |
| `p2a_checks.json` | D | Z17, Z18, Z19, Z20 | - | - |
| `p2a/migrate_old.json` | D | - | - | - |
| `p2a/fixture.json` | D | - | - | - |
| `p2a/tamper.json` | D | - | - | - |
| `p2a/tests.json` | D | - | - | - |
| `p2l_checks.json` | D | - | - | - |
| `p2l_quality.json` | D | - | - | - |
| `p2l/d_tr.json` | D | - | - | - |
| `p2l/english.json` | D | - | - | - |
| `p2l/migrate.json` | D | - | - | - |
| `p2p_checks.json` | D | - | - | - |
| `p2b_checks.json` | D | - | - | - |
| `p2b/align.json` | D | - | - | - |
| `p2b/det.json` | D | - | - | - |
| `p2b/client_checks.json` | D | - | - | - |
| `p2b/obsidian_e2e.json` | D | - | - | - |
| `p2b/vscode_e2e.json` | D | - | - | - |
| `p2r_checks.json` | D | - | - | - |
| `p2r/volume.json` | D | - | - | - |
| `p2r/det.json` | D | - | - | - |
| `p2b/align_regress.json` | D | - | - | - |
| `governance/vault/.rootmark/checkpoints/` (bayt) | D | - | - | - |
| `governance/vault/.rootmark/events.jsonl` (bayt) | D | - | - | - |
| `governance/vault/.rootmark/traces.jsonl` (bayt) | D | - | - | - |
| `p1d/inspector/tools_list.json` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/cli_promote.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/transcript.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/transcript_b.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/vault/.rootmark/events.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/vault/.rootmark/proposals.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/vault/.rootmark/traces.jsonl` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/vault/agent/plan.md` (bayt) | D | - | - | - |
| `p1d/mcp_e2e/run1/vault/espresso.md` (bayt) | D | - | - | - |
| `p1d/scene/scene.json` (bayt) | D | - | - | - |
| `p1d1/d1_13.json` (bayt) | D | - | - | - |
| `p1d1/d1_3.json` (bayt) | D | - | - | - |
| `p1d1/d1_8.json` (bayt) | D | - | - | - |
| `p1e/plugin_e2e/access_log.json` (bayt) | D | - | - | - |
| `p1e/plugin_e2e/events.jsonl` (bayt) | D | - | - | - |
| `p1e/plugin_e2e/plugin_e2e.json` (bayt) | D | - | - | - |
| `p1e/plugin_e2e/proposals.jsonl` (bayt) | D | - | - | - |
| `p1e/plugin_e2e/traces.jsonl` (bayt) | D | - | - | - |
| `p1e/scene/events.jsonl` (bayt) | D | - | - | - |
| `p1e/scene/proposals.jsonl` (bayt) | D | - | - | - |
| `p1e/scene/scene.json` (bayt) | D | - | - | - |
| `p1e/scene/traces.jsonl` (bayt) | D | - | - | - |
| `p1e1/release/access_log_real.json` (bayt) | D | - | - | - |
| `p1e1/release/plugin_e2e_real.json` (bayt) | D | - | - | - |
| `p2_0_2/d_a/attestations.jsonl` (bayt) | Z | Z29 | - | - |
| `p2_0_2/d_a/events.jsonl` (bayt) | Z | Z28 | - | - |
| `p2_0_2/d_a2/attestations.jsonl` (bayt) | Z | Z31 | - | - |
| `p2_0_2/d_a2/events.jsonl` (bayt) | Z | Z30 | - | - |
| `p2_0_2/d_c/attestations.jsonl` (bayt) | Z | Z33 | - | - |
| `p2_0_2/d_c/events.jsonl` (bayt) | Z | Z32 | - | - |

Sınıfı D olan her dosyanın gerekçesi aynı: girdisi commit, veri manifest'leri ve tohumlar; içinde zaman alanı varsa dondurulmuş karşılaştırıcı onu zaten ayırır (DETERMINISTIC_FIELDS.md); gerçek Obsidian ya da `serve --watch` zamanlaması yok. Gözlenen çiftlerde (aşağıda) bu dosyaların hiçbiri farklı değildi.

## Kriter kimlikleri (134; V-3'ün referansı, P2-R 3. deneme A)
| Kimlik | Sınıf | Kademe | Mac kümesi |
|---|---|---|---|
| `p1b1_checks.json:B1-1` | D | tier2 | - |
| `p1b1_checks.json:B1-2` | D | tier2 | - |
| `p1b1_checks.json:B1-3` | D | tier2 | - |
| `p1b1_checks.json:B1-4` | D | tier2 | - |
| `p1b1_checks.json:B1-5` | D | tier2 | - |
| `p1b1_checks.json:B1-6` | D | tier2 | - |
| `p1b_checks.json:B-5` | D | tier2 | - |
| `p1c1_checks.json:C1-1` | D | tier2 | - |
| `p1c1_checks.json:C1-2` | D | tier2 | - |
| `p1c1_checks.json:C1-3` | D | tier2 | - |
| `p1c1_checks.json:C1-4` | D | tier2 | - |
| `p1c1_checks.json:C1-5` | D | tier2 | - |
| `p1c1_checks.json:C1-6` | D | tier2 | - |
| `p1c1_checks.json:C1-7` | D | tier2 | - |
| `p1c1_checks.json:C1-8` | O | mac | O-C18 |
| `p1c1_checks.json:C1-9` | D | tier2 | - |
| `p1c_checks.json:C-1` | D | tier2 | - |
| `p1c_checks.json:C-10` | D | tier2 | - |
| `p1c_checks.json:C-11` | D | tier2 | - |
| `p1c_checks.json:C-12` | D | tier2 | - |
| `p1c_checks.json:C-13` | D | tier2 | - |
| `p1c_checks.json:C-2` | D | tier2 | - |
| `p1c_checks.json:C-3` | D | tier2 | - |
| `p1c_checks.json:C-4` | D | tier2 | - |
| `p1c_checks.json:C-5` | D | tier2 | - |
| `p1c_checks.json:C-7` | D | tier2 | - |
| `p1c_checks.json:C-9` | D | tier2 | - |
| `p1d1_checks.json:D1-1` | D | tier2 | - |
| `p1d1_checks.json:D1-10` | D | tier2 | - |
| `p1d1_checks.json:D1-11` | D | tier2 | - |
| `p1d1_checks.json:D1-12` | D | tier2 | - |
| `p1d1_checks.json:D1-13` | D | tier2 | - |
| `p1d1_checks.json:D1-2` | D | tier2 | - |
| `p1d1_checks.json:D1-3` | D | tier2 | - |
| `p1d1_checks.json:D1-4` | D | tier2 | - |
| `p1d1_checks.json:D1-5` | D | tier2 | - |
| `p1d1_checks.json:D1-6` | D | tier2 | - |
| `p1d1_checks.json:D1-7` | D | tier2 | - |
| `p1d1_checks.json:D1-8` | D | tier2 | - |
| `p1d1_checks.json:D1-9` | D | tier2 | - |
| `p1d1_checks.json:tamper` | D | tier2 | - |
| `p1d_checks.json:D-1` | D | tier2 | - |
| `p1d_checks.json:D-10` | Z | mac | Z-D10 |
| `p1d_checks.json:D-11` | D | tier2 | - |
| `p1d_checks.json:D-2` | D | tier2 | - |
| `p1d_checks.json:D-3` | D | tier2 | - |
| `p1d_checks.json:D-4` | D | tier2 | - |
| `p1d_checks.json:D-5` | D | tier2 | - |
| `p1d_checks.json:D-6` | D | tier2 | - |
| `p1d_checks.json:D-7` | D | tier2 | - |
| `p1d_checks.json:D-8` | D | tier2 | - |
| `p1d_checks.json:D-9` | D | tier2 | - |
| `p1d_checks.json:scene` | D | tier2 | - |
| `p1d_checks.json:tamper` | D | tier2 | - |
| `p1e1_checks.json:E1-1` | D | tier2 | - |
| `p1e1_checks.json:E1-2` | D | tier2 | - |
| `p1e1_checks.json:E1-3` | O | mac | O-REL |
| `p1e1_checks.json:E1-4` | O | mac | O-REL |
| `p1e1_checks.json:E1-5` | D | tier2 | - |
| `p1e1_checks.json:E1-6` | D | tier2 | - |
| `p1e1_checks.json:E1-8` | D | tier2 | - |
| `p1e1_checks.json:E1-9` | O | mac | O-REL |
| `p1e1_checks.json:tamper` | O | mac | O-REL |
| `p1e_checks.json:E-1` | D | tier2 | - |
| `p1e_checks.json:E-10` | D | tier2 | - |
| `p1e_checks.json:E-12` | D | tier2 | - |
| `p1e_checks.json:E-13` | D | tier2 | - |
| `p1e_checks.json:E-14` | D | tier2 | - |
| `p1e_checks.json:E-15` | D | tier2 | - |
| `p1e_checks.json:E-16` | D | tier2 | - |
| `p1e_checks.json:E-2` | D | tier2 | - |
| `p1e_checks.json:E-3` | D | tier2 | - |
| `p1e_checks.json:E-4` | D | tier2 | - |
| `p1e_checks.json:E-5` | D | tier2 | - |
| `p1e_checks.json:E-6` | D | tier2 | - |
| `p1e_checks.json:E-7` | O | mac | O-REL |
| `p1e_checks.json:E-9` | D | tier2 | - |
| `p1e_checks.json:tamper` | D | tier2 | - |
| `p2_0_2_checks.json:H-P2-0.2a` | Z | mac | Z-DA |
| `p2_0_2_checks.json:H-P2-0.2a′` | Z | mac | Z-DA |
| `p2_0_2_checks.json:H-P2-0.2b` | O | mac | O-DB4 |
| `p2_0_2_checks.json:H-P2-0.2b′` | O | mac | O-DB4 |
| `p2_0_2_checks.json:H-P2-0.2c` | O | mac | O-DB4 |
| `p2_0_2_checks.json:H-P2-0.2d` | Z | mac | Z-DC |
| `p2_0_2_checks.json:H-P2-0.2e` | O | mac | O-DA′ |
| `p2_0_2_checks.json:H-P2-0.2f` | Z | mac | Z-DB2 |
| `p2_0_2_checks.json:H-P2-0.2h` | D | tier2 | - |
| `p2_0_2_checks.json:H-P2-0.2j` | D | tier2 | - |
| `p2_0_2_checks.json:H-P2-0.2k` | D | tier2 | - |
| `p2_0_2_checks.json:fail_summary` | D | tier2 | - |
| `p2_0_2_checks.json:regress` | D | tier2 | - |
| `p2_0_2_checks.json:tamper` | D | tier2 | - |
| `p2_0_3_checks.json:H-P2-0.3a` | Z | mac | Z-DA |
| `p2_0_3_checks.json:H-P2-0.3a′` | Z | mac | Z-DA |
| `p2_0_3_checks.json:H-P2-0.3b` | O | mac | O-DB4 |
| `p2_0_3_checks.json:H-P2-0.3b-yük` | O | mac | O-DB4 |
| `p2_0_3_checks.json:H-P2-0.3b′` | O | mac | O-DB4 |
| `p2_0_3_checks.json:H-P2-0.3c` | O | mac | O-DB4 |
| `p2_0_3_checks.json:H-P2-0.3d` | O | mac | O-DC′ |
| `p2_0_3_checks.json:H-P2-0.3e` | O | mac | O-DA′ |
| `p2_0_3_checks.json:H-P2-0.3f` | Z | mac | Z-DB2 |
| `p2_0_3_checks.json:H-P2-0.3h` | D | tier2 | - |
| `p2_0_3_checks.json:H-P2-0.3j` | D | tier2 | - |
| `p2_0_3_checks.json:logs_vs_p2a` | Z | mac | Z-DA |
| `p2_0_3_checks.json:rules` | D | tier2 | - |
| `p2_0_3_checks.json:seed` | D | tier2 | - |
| `p2a_checks.json:A-1` | D | tier2 | - |
| `p2a_checks.json:A-2` | D | tier2 | - |
| `p2a_checks.json:A-3` | D | tier2 | - |
| `p2a_checks.json:A-4` | D | tier2 | - |
| `p2a_checks.json:A-5` | D | tier2 | - |
| `p2a_checks.json:A-6` | D | tier2 | - |
| `p2a_checks.json:A-7` | D | tier2 | - |
| `p2a_checks.json:A-8` | D | tier2 | - |
| `p2a_checks.json:fail_summary` | D | tier2 | - |
| `p2a_checks.json:regress` | D | tier2 | - |
| `p2a_checks.json:tamper` | D | tier2 | - |
| `p2b_checks.json:B-1` | D | tier2 | - |
| `p2b_checks.json:B-2` | D | tier2 | - |
| `p2b_checks.json:B-5/B-6` | D | tier2 | - |
| `p2l_checks.json:L-1` | D | tier2 | - |
| `p2l_checks.json:L-10` | D | tier2 | - |
| `p2l_checks.json:L-7` | D | tier2 | - |
| `p2l_checks.json:L-8` | D | tier2 | - |
| `p2l_checks.json:L0` | D | tier2 | - |
| `p2l_checks.json:L2` | D | tier2 | - |
| `p2p_checks.json:P-2/P-3` | D | tier2 | - |
| `p2p_checks.json:P-3/P-6` | D | tier2 | - |
| `p2p_checks.json:P-4/P-5` | D | tier2 | - |
| `p2p_checks.json:P-7` | D | tier2 | - |
| `p2p_checks.json:P0` | D | tier2 | - |
| `p2r_checks.json:R-1` | D | tier2 | - |
| `p2r_checks.json:R-2/R-4` | D | tier2 | - |
| `p2r_checks.json:R-5/R-8` | D | tier2 | - |

## Gözlenen farklar (sınıflandırmanın sınaması; sınıflandırma mekanizmadan)
- V0 (`verify.yml`, `352f790`, `ubuntu-latest`, koşu 40582139980): farklar yalnız D-A kümesinde (`p2_0_2/d_a.json` ve onu özetleyen `p2_0_2_checks.json`, `p2a_checks.json` alanları, D-A `events.jsonl`) ve R17 `detail`'inde (H1) — K-P2R.5 teşhisi. Her zincirde sayılan FAIL 16: Z (H-P2-0.2a, H-P2-0.3a, logs_vs_p2a), O-Obsidian (E1-3, E1-9, H-P2-0.3b, b-yük, c, d, e, p1e1 tamper), O-platform (C1-8; D-9 ve D1-12 yalnız C1-8'in alanlarında farklı; E-7; E1-4).
- Geliştirme koşusu, kademe 2 `macos-14` (karalama dalı `p2-v/scratch`, koşu 40590891313; V1-4): zincirler 2 sa 31 dk ve 2 sa 55 dk (ubuntu'da V0: 1 sa 44 dk ve 1 sa 52 dk). Sayılan FAIL (taslak envanterle) her zincirde aynı beş: C1-8 (C-10 `min_cosine` 0.9999999999991904, yayım Mac'inde 0.9999999999992171; vektör dosyalarının özetleri ve sıralama skorları), D-9 ve D1-12 (yalnız C1-8'in alanları), E1-4 (R4, R15), p1e1 tamper (`real_cases_run: false`). Sonuç: C1-8 ailesi her yayım-Mac'i-olmayan donanımda farklıdır; kademe 2 `ubuntu-latest`'e döndü (V1-1), platforma bağlı kriterler O.
- Geliştirme koşusu, kademe 1 (`ubuntu-latest`, koşu 40590894752): hızlı çift 18 550 alan eşit (yayım Mac'indeki son FAST ile aynı sayı); zincirler 26 ve 31 dk; `fast/conformance.json` `fixtures_root` host yolu (H4) yakalandı.
- Geliştirme koşusu, Mac kümesi (yerel, 1. tekrar): P2-0.2 kümeleri 11,5 dk; P2-0.3'ün D-C′'sü `roasting.md was not written by Obsidian within 2 s of the save` ile durdu (dondurulmuş yeniden deneme de) — durmuş harness başarısız tekrar sayılır (V2-2).
- **P2-R 3. deneme (Mac)**: 204172 alan, 2712 farklı; dosyalar `p2_0_2/d_a.json`, `p2_0_2/d_a2.json`, `p2_0_2/d_c.json`, `p2_0_2_checks.json`, `p2a_checks.json`; bayt `p2_0_2/d_a/events.jsonl`, `p2_0_2/d_a2/events.jsonl`, `p2_0_2/d_c/attestations.jsonl`, `p2_0_2/d_c/events.jsonl`
- **P2-R 1. deneme (Mac)**: 203776 alan, 33 farklı; dosyalar `p2_0_2/d_a.json`, `p2_0_2_checks.json`, `p2a_checks.json`; bayt `p2_0_2/d_a/events.jsonl`
- **CI 40265181721 pair_compare**: 786981 alan, 40 farklı; dosyalar `p1e1/release/release_check.json`, `p2_0_2/d_a.json`, `p2_0_2_checks.json`, `p2a_checks.json`; bayt `p2_0_2/d_a/events.jsonl`
- **V0 40582139980 pair_compare**: 788033 alan, 31 farklı; dosyalar `p1e1/release/release_check.json`, `p2_0_2/d_a.json`, `p2_0_2_checks.json`, `p2a_checks.json`; bayt `p2_0_2/d_a/events.jsonl`
