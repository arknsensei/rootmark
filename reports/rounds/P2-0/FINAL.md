# P2-0 — FINAL (tasdikli yazma)

- RUN: `20261018T161850Z-e378be2`; commit `e378be204e0536c69f3ae908358ce5ab6aa05aef`
- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).

## Hipotezler

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| H-P2-0a | Canonical sızıntısı: ajan yazımından sonra `canonical` olan yeni/değişen blok | 0 / 525 yeni blok, 200 senaryo | = 0 | geçti | `p2_0/d_a.json` |
| H-P2-0a′ | Canonical içerik korunumu: eski sürüm checkpoint'ten okunur, `history` silen/değiştiren aktörü (`unattested_fs`) gösterir | 200/200 senaryo; canonical soyu değiştiren senaryo 16 (diğerlerinde koşul boş), değişen canonical soy 52 | 200/200 | geçti | `p2_0/d_a.json` |
| H-P2-0b | Yanlış `unverified`: gerçek insan düzenlemesinde W sonunda `unverified` kalan blok oranı | 931/1890 = 0.4926 (200 oturum, 286 kayıt) | ≤ 0.01 | kaldı | `p2_0/d_b.json` |
| H-P2-0b′ | Geç tasdikle düzelen kayıt (bilgi) | 0/286 kayıt | raporlanır | raporlandı (bilgi) | `p2_0/d_b.json` |
| H-P2-0c | Tasdik gecikmesi: kayıt → `canonical` p95 | 270 ms (208 ölçüm; yazımsız kayıt 78) | < 500 ms | geçti | `p2_0/d_b.json` |
| H-P2-0d | Karışık (D-C): katman doğruluğu | 333/340 = 0.9794 (160 tik) | ≥ 0.99 | kaldı | `p2_0/d_c.json` |
| H-P2-0e | Anahtar zinciri: terminal sürecinin gizi okuyamadığı deneme oranı (alt-hipotez) | ölçülmedi: the probe reads a person's keychain item (Z6); scripts/keychain_probe.sh is run by a person | 20/20 | ölçülmedi | `p2_0_checks.json (Z6)` |
| H-P2-0f | Eski vault'lar (`v0.1.0`): açılır, replay eşit, `events_hash` değişmez, katmanlar korunur | 3/3 | 3/3 | geçti | `p2_0/d_e.json (ASSUMPTIONS Z5-3)` |
| H-P2-0g | Determinizm: çift koşturma, D-A ve D-C `events.jsonl` + `attestations.jsonl` bayt eşit; bütün çift kümesi | günlükler eşit True; 106701 alan, fark 0; yapıtlar bayt eşit True | eşit | geçti | `pair_compare.json` |
| H-P2-0h | Uyumluluk: C2-11, C2-12, C3-08 + mevcut 34 | 37/37, seviye 3, spec 2.0.0 | 37/37 | geçti | `conformance_p1d1.json` |

## Pencere (W)

Seçilen `W` = **0.5 s** (choose_window (Z5.md); ayar seti BLAKE3 `77f2aacda0581a96e87f657374b7977bcf0d130081dbb7717bafeb9d977822fb`; kayıt `docs/rounds/P2-0/progress/Z5_tuning.json`).

| W (s) | kayıt | yeni blok | W sonunda `unverified` | oran | geç yükselme | p95 (ms) |
|---|---|---|---|---|---|---|
| 0.5 | 140 | 542 | 63 | 0.1162 | 0 | 257 |
| 1 | 140 | 542 | 63 | 0.1162 | 0 | 260 |
| 2 | 140 | 542 | 63 | 0.1162 | 0 | 260 |
| 3 | 140 | 542 | 63 | 0.1162 | 0 | 266 |
| 5 | 140 | 542 | 63 | 0.1162 | 0 | 261 |

## D-B oturum türlerine göre (test)

| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |
|---|---|---|---|
| char | 29 | 29 | 4 |
| focus | 28 | 28 | 0 |
| multi | 65 | 65 | 0 |
| paragraph | 29 | 29 | 0 |
| paste | 30 | 384 | 0 |
| rapid | 27 | 92 | 0 |
| rename | 78 | 1263 | 927 |

## Regresyon ve kasıtlı vakalar

- P1-E.1 resmi RUN'ına karşı (`regress_vs_p1e1.json`, referans `20261017T140538Z-7ea12dc`): 102370 alan, hariç dışı fark 1, karar gereği 98 alt ağaç, kullanılmayan hariç 0; uyumluluk eski testler eşit True, yeni testler {'C2-11': True, 'C2-12': True, 'C3-08': True} → kaldı.
- Kasıtlı vakalar (`p2_0/tamper_p2_0.json`): 9/9 reddedildi, kontroller 2/2 → geçti. "Token'sız attest" `attest_session`'la denenir (token'sız oturum yok → MAC'li `attest` üretilemez); `attest`'in kendisi de token ister (sözleşme §2.6, `contract_check.sh`). Sahte satır vakası ilk satırın kimliği bozulmuş kopyasını sona ekler: yalnızca-ekleme sırası da onu reddeder, kimlik formülü denetimini tek başına kanıtlamaz (onu C2-12 `attestation_tamper id` kanıtlar).

## Önceki turların kaydedicileri

`p2_0/historical.json` (`eval.historical_p2_0`, referans: `reports/rounds/P1-E.1/runs/20261017T140538Z-7ea12dc`): 14 tarihsel FAIL sayılmadı; sayılan FAIL 0. (`verify.sh`'in FAIL özeti P1-E.1'in `eval.historical`'ını, P1-E referansıyla, bütün `*_checks.json` üzerinde koşar: P1-E.1 kaydedicilerini referansta bulamaz ve bu turun `p2_0_checks.json` satırlarını da sayar; o sayım bu sınıflandırmanın yerine geçmez.)

- `p1b1_checks.json` B1-3: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED) (P1-E.1'de de tarihsel: K-C1.3, K-C1.5, K-D1.2, K-D1.4-T, K-E0.1)
- `p1c1_checks.json` C1-7: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED) (P1-E.1'de de tarihsel: K-D1.2, K-D1.4-T, K-E0.1)
- `p1c_checks.json` C-11: tarihsel, karar K-D1.2, K-D1.4-T, K-E0.1 ile kapandı — P1-C's regression against P1-B.1 sees B1-3's spec-version difference (historical in P1-E.1 too: research/eval/historical_fails.json) (P1-E.1'de de tarihsel: K-D1.2, K-D1.4-T, K-E0.1)
- `p1c_checks.json` C-3: tarihsel, karar K-C8-T, K-C1.1, K-C1.2 ile kapandı — P1-C's hybrid multi-question nDCG@10 was -0.0136 off v2; closed in P1-C.1 by the declared lexical variant and the Rust quality reference (historical in P1-E.1 too: research/eval/historical_fails.json) (P1-E.1'de de tarihsel: K-C8-T, K-C1.1, K-C1.2)
- `p1d1_checks.json` D1-10: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)
- `p1d1_checks.json` D1-12: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)
- `p1d_checks.json` D-2: tarihsel, karar K-D1.1, K-D1.5, K-E0.4, K-D1.4-T ile kapandı — P1-D's MCP e2e expects agent edits of human blocks to pass (append_only since K-D1.1), the old error codes (K-D1.5, K-E0.4) and spec 1.0.2 in status (historical in P1-E.1 too: research/eval/historical_fails.json) (P1-E.1'de de tarihsel: K-D1.1, K-D1.5, K-E0.4, K-D1.4-T)
- `p1d_checks.json` D-7: tarihsel, karar K-D1.4-T, K-D1.5 ile kapandı — P1-D's HTTP e2e expects spec 1.0.2 (D7-3) and the old status codes of two cases (historical in P1-E.1 too: research/eval/historical_fails.json) (P1-E.1'de de tarihsel: K-D1.4-T, K-D1.5)
- `p1d_checks.json` D-9: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED) (P1-E.1'de de tarihsel: K-D1.7, K-D1.4-T)
- `p1e1_checks.json` E1-6: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)
- `p1e1_checks.json` E1-8: tarihsel, karar K-P2-0.5 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)
- `p1e1_checks.json` tamper: tarihsel, karar K-P2-0.5 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)
- `p1e_checks.json` E-7: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED) (P1-E.1'de de tarihsel: K-E5-T, K-E1.1)
- `p1e_checks.json` E-9: tarihsel, karar K-P2-0.4 ile kapandı — a P2-0 decision (compare_p2_0.EXCLUDED)

## Karar kuralı

Kalan hipotezler: H-P2-0b, H-P2-0d; bağımsız doğrulama açık.

**Tanımlanmamış kalış → tur kaldı, karar talebi**: kalan hipotezler ['H-P2-0b', 'H-P2-0d']; regress/tamper ['regress']; sayılan önceki-tur FAIL'leri yok.

## Tehdit modeli (taslak)

- **Aşama 1 (etkin, `status.attestation.stage = 1`):** tasdik, token'la açılan bir oturum anahtarının MAC'idir (HTTP-only; MCP'de tasdik aracı yoktur — kasıtlı vakalar). İnsan kanalı Obsidian sürecinden geçen yazımlardır (eklentinin `adapter.write/append/rename` gözlemi; aynı süreçteki başka eklentiler de bu kanaldadır). **`.rootmark/server.token`'ı okuyabilen bir ajan oturum açıp tasdik üretebilir**; aşama 1 dikkatsiz ajana (dosyaya doğrudan yazan) karşı korur, kasıtlı ajana karşı korumaz.
- **Tasdiksiz yazım** (`unverified`, aktör `unattested_fs`): aramada varsayılan görünmez, `include_proposal` ile etiketli; insan "Mine"/`promote` ile yükseltir. Toplu tasdik (`rootmark attest --all`) yalnız günlüğe yazar, hiçbir alımı yükseltmez.
- **Bilinen boşluklar:** Obsidian'ın bağlantı güncellemeleri `adapter.write` dışından yazılır ve tasdik edilmez (ASSUMPTIONS Z5-5; yeniden adlandırmada yanlış `unverified`); ajanın insanın kaydedilmemiş değişikliği olan nota yazması ve Obsidian'ın birleştirip kaydetmesi ajan bloklarını tasdikli yapabilir (Z3 bulgu 12); insanın kaydı ile ajanın yazımı izleyicinin tek gözlemine düşerse (D-C `collide`) insan içeriği de `unverified` olur (güvenli taraf, Z5-6); birkaç kayıt W + 1.5 s içinde `canonical` olmadı (D-B `latency_unsettled`). Bu taslak tasarımı doğrulanmış saymaz: hipotez bu turda kanıtlanmadı (karar kuralı).
- **Aşama 2 (ölçülmedi, Z6):** eklentinin tek anahtar zinciri ilkeli Electron `safeStorage`; açık sorunlar: sunucu tarafında sırrın ajandan uzak saklanması, açık anahtar kaydının insana özgü bir onaya bağlanması, `.obsidian/plugins/**` yazımıyla eklenti enjeksiyonu, ve terminalin Obsidian'ı `--remote-debugging-port` ile başlatıp CDP üzerinden `safeStorage`'ı çağırması (istem yok; anahtar zinciri ACL'si bunu kapatmaz). Sonda `scripts/keychain_probe.sh` insanın koşturması için hazır.

## NORTH_STAR önerisi

NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına göre:

- Hipotez kanıtlandıysa: vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir (aşama 1: tasdikli istemci ile; token okuyan kasıtlı ajan kapsam dışı)." eklenir.
- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical` kalır, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).
- Tanımlanmamış kalışta (bu RUN'ın sonucu dalda değilse): vaat değişmez; öneri insanın karar talebine verdiği cevaba bağlıdır (DECISIONS K-P2-0.6-T).

## Notlar (el ile, üreteç çıktısından sonra eklendi; kaynakları adlı)

1. **Regresyondaki tek fark** (`regress_vs_p1e1.json`): `p1e1/release/release_check.json:passed` 16 → 17. Nedeni K-P2-0.3'ün R16 maddesidir (kontrol listesi 17 madde, hepsi geçti). Hariç tutma listesini yazdığım geliştirme RUN'ında (`20261018T125648Z-78b5119`, kirli klon) R4a "dirty work tree" ile kaldığı için `passed` yine 16'ydı ve fark görünmedi; liste resmi RUN'dan önce yazıldı ve sonuç görüldükten sonra **değiştirilmedi**. Bu yüzden regress "kaldı" olarak durur (Z7.md, ASSUMPTIONS Z7-1).
2. **Çift koşturma:** `make verify` dal sabitken `e378be2`'de koşturuldu, çıkış 0: A `20261018T161850Z-e378be2`, B `20261018T170104Z-e378be2`, 106 701 alan fark 0, yapıtlar bayt eşit (`pair_compare.json`, `verify_pair.log`). Bundan önceki bir `make verify` (A `20261018T140007Z-fa02519`) B klonu koşu sırasında eklenen commit'leri aldığı için 3 ile bitmişti; o koşu resmi değildir ve commit'lenmedi (ASSUMPTIONS Z8-1). İki koşunun hipotez sonuçları aynı dallara düştü.
3. **0b'nin kaynağı:** W sonunda `unverified` kalan 931 bloğun 927'si yeniden adlandırma oturumlarındandır (yeniden adlandırılan notlar ve Obsidian'ın bağlantılarını güncellediği notlar); kalan 4 blok, daha önce yeniden adlandırılmış notlara yapılan 4 tek-karakter kaydıdır ve bunlar W + 1.5 s içinde hiç `canonical` olmadı (`latency_unsettled` 4; p95'e sonsuz gecikme olarak girerler). Diğer bütün oturum türlerinde 0. Neden ölçümden önce yazıldı (ASSUMPTIONS Z5-5): Obsidian'ın bağlantı güncellemesi `adapter.write`'tan geçmez, tasdik edilmez; ardından o notun yeniden adlandırması da (`canonical` olmayan baş) tasdik edilmez ve yeni yolun bütün blokları `unverified` olur; test setinde yeniden adlandırmalar zincirlendiği için etki ayar setindekinden (63/542) büyüktür. Oran W'den bağımsızdır (`Z5_tuning.json`).
4. **0d'nin kaynağı:** 7 yanlış bloğun hepsi `collide` tiklerindendir (7/14); `collide` hariç 326/326. Etki ölçümden önce yazıldı (ASSUMPTIONS Z5-6): ajan insanın kaydettiği nota izleyici bakmadan eklediğinde iki yazım tek gözlemdir, tasdik bayt eşleşmesi olduğundan insanın paragrafı da `unverified` olur (güvenli taraf).
5. **Kalite kapıları** (`fa02519`; sonrası yalnız rapor üreteci ve belgeler): `make test` (cargo 256, pytest 232 + 1 atlanan, vitest 139), `make lint`, `make conformance` (37/37) temiz; `make verify` (`e378be2`) temiz, çift eşit.
6. **Karar talebi:** `docs/DECISIONS.md` K-P2-0.6-T (0b ve 0d için iki ayrı eksen). 0e ölçülmedi (Z6, PARK; `STOP.md`).
