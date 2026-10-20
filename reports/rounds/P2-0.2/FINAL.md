# P2-0.2 — FINAL (bağlantı biçimleri ve katman taşıma, yama turu)

- RUN: `20261019T212951Z-72fd78f`; commit `72fd78f96a593949303ea9eedde61bbd15cf223d`
- Kaynak: bu RUN dizinindeki dosyalar ve yük altındaki çiftin karşılaştırması (her satırda adı geçer). Eşikler GOAL P2-0.2'ninkiler (P2-0'ınkiler aynen); `W` = 0.5 s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı). Spec §7.1/§8 durumu: Draft (damga bağımsız doğrulamadan sonra; DEV_PROCESS §5).

## Hipotezler

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| H-P2-0.2a | Canonical sızıntısı (D-A, D-A2) | D-A 0/525 (200 senaryo); D-A2 0/210 (120 senaryo) | = 0 | geçti | `p2_0_2/d_a.json, p2_0_2/d_a2.json` |
| H-P2-0.2a′ | Canonical içerik korunumu (D-A) | 200/200 senaryo; canonical soyu değiştiren senaryo 16, değişen canonical soy 52 | 200/200 | geçti | `p2_0_2/d_a.json` |
| H-P2-0.2b | Yanlış `unverified`, W sonunda (D-B test-3, gerçek Obsidian, karışık bağlantı biçimleri) | 116/2743 = 0.0423 (200 oturum, 409 kayıt, yan not 174; tohum 1793744460) | ≤ 0.01 | kaldı | `p2_0_2/d_b.json` |
| H-P2-0.2b′ | Yalnız yeniden adlandırma oturumları; biçim başına (bilgi) | 116/2287 (70 oturum, 244 kayıt, yan notlar dahil); biçim tablosu aşağıda | raporlanır | raporlandı (bilgi) | `p2_0_2/d_b.json` |
| H-P2-0.2c | Tasdik gecikmesi p95 (D-B test-3) | 299 ms (165 ölçüm; `canonical` olmayan 0; yazımsız kayıt 244) | < 500 ms | geçti | `p2_0_2/d_b.json` |
| H-P2-0.2d | Karışık katman doğruluğu (D-C, T3); gerçek Obsidian çakışma (D-C′) | D-C 340/340 = 1.0000 (160 tik; ajan `canonical` 0; `partial` satırı 7); D-C′ 40/40 = 1.0000, ajan `canonical` 0 (40 tik) | ≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0 | geçti | `p2_0_2/d_c.json, p2_0_2/d_c_prime.json` |
| H-P2-0.2e | Aklama: ajan bloğu `canonical`; sahte ret; katman taşımada ajan bloğu `canonical` (D-A′) | ajan `canonical` 0/50 (50 senaryo; bağlantı yeniden yazıldı 48); sahte ret 10/10; taşıma: ajan `canonical` 0/10 (10 senaryo; insan blokları `canonical` 39 → 39) | = 0; 10/10; = 0 | geçti | `p2_0_2/d_a_prime.json` |
| H-P2-0.2f | Obsidian dışı insan: yanlış `unverified`; gecikme p95 (D-B2) | 0/630 = 0.0000; p95 137.4 ms (150 oturum; `canonical` olmayan 0) | ≤ 0.01; < 500 ms | geçti | `p2_0_2/d_b2.json` |
| H-P2-0.2h | Eski vault'lar (3 `v0.1.0` + 2 `ce4cf04` + 1 `21b16ea`): açılır, replay, `events_hash`, `attestations.jsonl` değişmez; 2.0.1 günlüğü yeni okuyucuda | 6/6 | 6/6 | geçti | `p2_0_2/d_e.json` |
| H-P2-0.2i | Determinizm: çift koşturma yüksüz ve yük altında (D-A, D-A2, D-C günlükleri bayt eşit; bütün çift kümesi) | yüksüz: günlükler eşit True; 110600 alan, fark 0; yapıtlar bayt eşit True; yüklü: günlükler eşit True; 110600 alan, fark 0; yapıtlar bayt eşit True | ikisi de eşit | geçti | `pair_compare.json; --load-pair` |
| H-P2-0.2j | Uyumluluk: mevcut 38 + C2-14 (T1/T2) | 39/39, seviye 3, spec 2.0.2; C2-14 pass | hepsi | geçti | `conformance_p1d1.json` |
| H-P2-0.2k | Damga kuralı (DEV_PROCESS §5), CRITERION netleştirmesi, §8 okuyucu kuralı yazılı; başvuru okuyucusu: bilinmeyen alanlı satır okunur, bilinmeyen değer reddedilir | yazılı 4/4; ileri uyumluluk 4/4 | yazılı ve testli | geçti | `p2_0_2/rules.json` |

## Kümeler

**D-B test-3 (gerçek Obsidian), oturum türlerine göre** (`p2_0_2/d_b.json`):

| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |
|---|---|---|---|
| char | 22 | 22 | 0 |
| focus | 21 | 21 | 0 |
| move_folder | 15 | 416 | 1 |
| multi | 57 | 57 | 0 |
| paragraph | 22 | 22 | 0 |
| paste | 21 | 255 | 0 |
| rapid | 22 | 79 | 0 |
| rename | 166 | 1238 | 108 |
| rename_again | 63 | 633 | 7 |

**H-P2-0.2b′ — yeniden adlandırma oturumlarının yeni blokları, bağlantı biçimine göre** (`p2_0_2/d_b.json` `renames.by_class`; sınıflar ASSUMPTIONS X0-7, X0-18; bir blok birden çok sınıfta sayılabilir):

| Biçim | yeni blok | W sonunda `unverified` |
|---|---|---|
| embed | 39 | 3 |
| md_label | 41 | 3 |
| md_name | 38 | 0 |
| none | 2058 | 100 |
| self | 31 | 2 |
| wiki | 43 | 1 |
| wiki_alias | 32 | 2 |
| wiki_heading | 41 | 7 |

Bağlantı ayarları (X0-15): `{"alwaysUpdateLinks": true, "newLinkFormat": "shortest", "useMarkdownLinks": false}`.

**D-B2 (Obsidian dışı insan), oturum türlerine göre** (`p2_0_2/d_b2.json`):

| Tür | oturum | yeni blok | W sonunda `unverified` |
|---|---|---|---|
| append | 20 | 20 | 0 |
| atomic | 50 | 50 | 0 |
| inplace | 30 | 30 | 0 |
| mv | 30 | 459 | 0 |
| rapid | 20 | 71 | 0 |

**D-A2 (Obsidian dışı ajan), yollara göre** (`p2_0_2/d_a2.json`):

| Yol | senaryo | yeni blok | `canonical` sızıntısı |
|---|---|---|---|
| atomic_rename | 20 | 35 | 0 |
| cat | 20 | 35 | 0 |
| cp_rsync | 20 | 35 | 0 |
| git_apply | 20 | 35 | 0 |
| python | 20 | 35 | 0 |
| sed | 20 | 35 | 0 |

**D-C′ (gerçek Obsidian çakışma), gecikmeye göre** (`p2_0_2/d_c_prime.json`):

| Gecikme (ms) | tik | insan bloğu | insan `canonical` | ajan `canonical` |
|---|---|---|---|---|
| 0 | 10 | 10 | 10 | 0 |
| 100 | 10 | 10 | 10 | 0 |
| 20 | 10 | 10 | 10 | 0 |
| 50 | 10 | 10 | 10 | 0 |

**D-A′ (aklama, gerçek Obsidian), türlere göre** (`p2_0_2/d_a_prime.json`):

| Tür | senaryo | ajan `canonical` |
|---|---|---|
| carry | 10 | 0 |
| in_agent_block | 25 | 0 |
| in_human_block | 15 | 0 |
| unobserved | 10 | 0 |

## Regresyon ve kasıtlı vakalar

- P2-0.1 resmi RUN'ına karşı (`regress_vs_p2_0_1.json`, referans `20261019T001516Z-21b16ea`): 102496 alan, hariç dışı fark 0, karar gereği 66 alt ağaç, kullanılmayan hariç 0; uyumluluk eski testler eşit True, yeni test {'C2-14': True}; P2-0.1'in yeniden koşan kümeleri (D-A, D-A2, D-E beş satır, D-C T3/X2-1 eşdeğerliği) eşit True → geçti.
- Kasıtlı vakalar (`p2_0_2/tamper_p2_0_2.json`): 12/12 reddedildi, kontroller 4/4 → geçti.

## Önceki turların kaydedicileri

`p2_0_2/historical.json` (`eval.historical_p2_0_2`, referans `reports/rounds/P2-0.1/runs/20261019T001516Z-21b16ea`; `make verify`'ın FAIL özeti bu sınıflandırıcının çıktısıdır): 14 tarihsel FAIL sayılmadı; sayılan FAIL 0 → geçti.

- `p1b1_checks.json` B1-3: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1c1_checks.json` C1-7: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1c_checks.json` C-11: tarihsel, karar K-D1.2, K-D1.4-T, K-E0.1 ile kapandı — P1-C's regression against P1-B.1 sees B1-3's spec-version difference (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1c_checks.json` C-3: tarihsel, karar K-C8-T, K-C1.1, K-C1.2 ile kapandı — P1-C's hybrid multi-question nDCG@10 was -0.0136 off v2; closed in P1-C.1 by the declared lexical variant and the Rust quality reference (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d1_checks.json` D1-10: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1d1_checks.json` D1-12: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1d_checks.json` D-2: tarihsel, karar K-D1.1, K-D1.5, K-E0.4, K-D1.4-T ile kapandı — P1-D's MCP e2e expects agent edits of human blocks to pass (append_only since K-D1.1), the old error codes (K-D1.5, K-E0.4) and spec 1.0.2 in status (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d_checks.json` D-7: tarihsel, karar K-D1.4-T, K-D1.5 ile kapandı — P1-D's HTTP e2e expects spec 1.0.2 (D7-3) and the old status codes of two cases (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d_checks.json` D-9: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1e1_checks.json` E1-6: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1e1_checks.json` E1-8: tarihsel, karar K-P2-0.5 ile kapandı — the p1e-final server case of P1-E.1's contract tamper expects only status gaps; it also lacks §2.6 and status.attestation since P2-0 (P2-0/F14); historical in P2-0 too, its record equal to the P2-0 official RUN's
- `p1e1_checks.json` tamper: tarihsel, karar K-P2-0.5 ile kapandı — the p1e-final server case of P1-E.1's contract tamper expects only status gaps; it also lacks §2.6 and status.attestation since P2-0 (P2-0/F14); historical in P2-0 too, its record equal to the P2-0 official RUN's
- `p1e_checks.json` E-7: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)
- `p1e_checks.json` E-9: tarihsel, karar K-P2-01.3, K-P2-02.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED) — historical in the P2-0.1 official RUN (its record there is the reference)

## Karar kuralı

Kalan hipotezler: H-P2-0.2b; bağımsız doğrulama açık.

**['H-P2-0.2b'] kaldı → GOAL P2-0'ın 0b dalı**: `unverified` varsayılan **olmaz**, politika `canonical` ile yayımlanır, vaat "tasdikli istemci ile" ibaresiyle daraltılır (karar kaydıyla; öneri aşağıda); **P2-A başlar**.

## NORTH_STAR önerisi

NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına ve bağımsız doğrulamanın hükmüne göre:

- Hepsi geçti ve doğrulama "güvenilir" (ya da koşulları kapandı): vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir; insanın yaygın Obsidian bağlantı biçimleri ve yeniden adlandırmaları bunu bozmaz (tasdikli istemciyle; insan kanalına giremeyen ajana karşı — token'ı okuyan, insan istemcisini başlatıp süren ya da eklenti dosyalarını yazan süreç kapsam dışı)." eklenir.
- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical`, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).
- Kaldı dallarında: vaat değişmez; öneri insanın karar talebine verdiği cevaba bağlıdır.

## Notlar (el ile, üreteç çıktısından sonra eklendi; kaynakları adlı)

1. **Çiftler (H-P2-0.2i).** Yüksüz: `make verify` (`scripts/verify.sh pair`) `72fd78f`'te, çıkış 0: A `20261019T212951Z-72fd78f` (resmi RUN, 3262 s), B `20261019T222411Z-72fd78f` (3276 s); `pair_compare.json`: 110 600 alan, fark 0, yapıtlar bayt eşit; D-A, D-A2, D-C `events.jsonl` ve `attestations.jsonl` bayt eşit (`verify_pair.log`). Yüklü: `VERIFY_LOAD=1 make verify` aynı commit'te, çıkış 0, iki zincir boyunca 16 `yes` (mantıksal CPU sayısı): A `20261019T232111Z-72fd78f` (3710 s), B `20261020T002304Z-72fd78f` (3721 s); `load_pair/pair_compare.json`: 110 600 alan, fark 0, yapıtlar ve üç kümenin günlükleri bayt eşit (`load_pair/verify_pair.log`). Dört zincirin `p2_0_2_checks.json`'unda hipotezler aynı dalda (yalnız 0.2b kaldı; `chain_B/`, `load_pair/chain_A/`, `load_pair/chain_B/`).
2. **H-P2-0.2b'nin dört zincirdeki değeri** (`d_b.json`): yüksüz A 116/2743 = 0.0423, yüksüz B 116/2743 = 0.0423 (kayıt düzeyinde A ile aynı; p95 287 ms); yüklü A 135/2743 = 0.0492 (p95 330 ms, `latency_unsettled` 3), yüklü B 131/2743 = 0.0478 (p95 326 ms). Geliştirme ölçümü (X4, resmi değil) 113/2743 = 0.041. Zamanlamaya bağlı değerler pair karşılaştırmasının dışındadır (`P2_0_2_TIMING`, DETERMINISTIC_FIELDS.md).
3. **0.2b'nin kök nedeni (resmi A; X4'ün dökümüyle aynı yöntem, ham vault harness'in geçici dizininden, RUN'ın parçası değil).** 116 bloğun 113'ü geliştirmedeki gruplarla birebir aynı:
   - **(A) 104 blok — kendine bağlanan iki notun yeniden adlandırması** (staff: t172 16 + yan not 2; origins: t185 83 + yan not 3). Yeni yolun ilk gözlemi eski başın blokları ve notun kendi yeniden yazılmış bağlantılarıyla tek gözleme düştü (t172 `BlockCreated` `seq 2814–2829` ve anlık görüntü `2830`, t185 `3080–3162` ve `3163`; anlık görüntüler `parent_snapshot: null`); yeni yollar için pencerede tasdik satırı yok. Kendine bağlanan 31 yeniden adlandırmanın 29'u tasdikli; tasdiksiz ikisi her koşuda t172 ve t185 (sistematik). Neden: geç bağlantı güncellemesi boş bekleyen tabana karşı denetlendi — **çıkarım** (sunucu reddi günlüğe yazılmaz; resmi koşu çıkarımı destekler, kanıt düzeyini yükseltmez).
   - **(B) 12 blok — kabul edilmiş ama eşleşmemiş tasdik.** safety (t083 ve 8 taşınma, 9 blok; `att at_seq 1150` kabul, gözlem `seq 1198` `unattested_fs`) — üç koşuda aynı; **yeni:** roast-log (t185 yan not 1 + taşınma t190, t198; `at_seq 3066` kabul, gözlem origins'in 83 bloğundan sonra `seq 3164`, eşleşmedi). Geliştirmede aynı tasdik `at_seq 3072`'de kaydedilip eşleşmişti (aynı tasdik hash'i ve blok sürümü); fark tasdikin daha erken kaydedilmesi — W penceresinin gözlemden önce dolması olası (**çıkarım**, günlükte zaman yok). 113 → 116 farkının tamamı bu üç bloktur. (B)'nin nedeni **açık**.
4. **Düzeltme önerisi (bu turda uygulanmadı; X4-1: ölçümden sonra ürün değişmez).** (A): yeniden adlandırılan notun kendi geç bağlantı güncellemesinde, yol log için yeniyse ve `doc == link_update_of` ise eski yolun başına karşı denetim (`link_update_from`'un var olan yolunun genişlemesi) — t172/t185'i kapsar: 104/116 blok, **(A)'nın nedeni üstüne çıkarım doğruysa; ölçülmedi**. (B): nedeni bir sonraki turda sunucu tarafı tanı günlüğüyle (tasdik kaydı ve gözlem zamanları) belirlenmeli; W ayarı bu turun yasağıdır ve önerilmez. Ölçüm, eşik ve küme bu sonuca bakılarak değiştirilmedi.
5. **Regress ve sınıflandırıcı** (`regress_vs_p2_0_1.json`, `p2_0_2/historical.json`): yüksüz ve yüklü A'da hariç dışı fark 0, kullanılmayan hariç 0, sayılan FAIL 0 (14 tarihsel). P1-E.1'in kasıtlı vaka betiği (`test_tamper_p1e1.sh`) her zincirde çıkış 1 kaydeder — `p1e1_checks.json` E1-8/tamper'ın tarihsel kaydı (K-P2-0.5), sayılmaz.
6. **Kalite kapıları** (el ile gözlendi, bu RUN dizininin dosyası değil): ölçülen commit `72fd78f`'in ağacında (yalnız belgeler ve bu RUN kaydı commit'lenmemişken) `make test` çıkış 0 (cargo hepsi; pytest 231 + 9 atlanan; vitest 141), `make lint` çıkış 0 (`scripts/check_frozen.sh` OK — hotfix PR #13 `main`'e girdi ve dala birleştirildi, `b029a33`, ASSUMPTIONS X6-1), `make conformance` 39/39 seviye 3; `make verify` yüksüz ve yüklü çıkış 0, iki çift eşit (not 1).
7. **Karar (DECISIONS K-P2-02.4):** karar kuralının 0b dalı — `unverified` varsayılan olmaz, politika `canonical` ile yayımlanır, vaat "tasdikli istemci ile" daraltılır; P2-A başlar. Varsayılan bu turda kodda değiştirilmedi: GOAL "Yapılmayacaklar" (yayım yok, paket sürümü değişmez), ASSUMPTIONS X4-1 (ölçümden sonra ürün değişmez) ve C2-11 fixture'ı (başlığı "`unverified` by default"; değiştirilemez) — spec §7.1 ("default `unverified`") Draft'tır ve CRITERION ile değişebilir. Yolu karar talebi K-P2-02.4-T'dedir. Spec §7.1/§8 Draft kalır (damga yok; DEV_PROCESS §5). NORTH_STAR değiştirilmedi (öneri yukarıda, 0b maddesi).
8. **Bağımsız doğrulama açık** (VERIFY.md). PR'ın gövdesi bu dosyadır. Ölçülen `72fd78f` dal commit'idir (`p2-0.2/round`); PR yeniden tabanlanarak birleşirse `main`'de bulunmaz (K-P2.2) — doğrulama için dal ya da bir etiket korunmalıdır.
9. **Ham vault eşleştirmesi (not 3):** D-B harness'i vault'u `$TMPDIR/rootmark-p2-0-2-sessions-*` altında bırakır, RUN'a yalnız `d_b.json` girer. Zincirlere eşleştirme `d_b.json` `attestation_lines` ile vault'un tasdik satırı sayısının eşitliğine dayanır (A 453, B 454, geliştirme 456) ve zamana; kimliği belirlenemeyen bir dördüncü tam vault (20:02Z) yalnız destekleyici gözlem olarak anıldı, hiçbir `d_b.json` ile eşleşmedi.
