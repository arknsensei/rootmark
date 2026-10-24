# P2-0.3 — FINAL (tasdikli yazma: yeniden ölçüm, hipotez turu)

- RUN: `20261024T000230Z-5632b3b` (resmi, yüksüz çiftin A zinciri); commit `5632b3b970586d5263fc1a3a03db09ad79f5119b`
- Zincirler: yüksüz B `chain_B`; yüklü A `chain_A`, yüklü B `chain_B` (dördü aynı commit).
- Eşikler GOAL P2-0.3'ünkiler (P2-0'ınkiler aynen); `W` = 0.5 s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı). Spec §7.1/§8 durumu: Draft (damga bağımsız doğrulamadan sonra; DEV_PROCESS §5).
- Tohum (D-B test-4): `2671050228` = int(blake3("P2-0.3/D-B")[:8], 16) (R0'da ön-kayıtlı); kayıtlı 2671050228; yeniden kullanılmayan [1793744460] → geçti.
- Ürün değişmedi: D-A, D-A2, D-C günlükleri P2-A resmi RUN'ıyla (`reports/rounds/P2-A/runs/20261020T230453Z-1a775d3`) bayt eşit → geçti (dört zincirde: A True, B True, load A True, load B True).

## Hipotezler

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| H-P2-0.3a | Canonical sızıntısı (D-A, D-A2) | D-A 0/525 (200 senaryo); D-A2 0/210 (120 senaryo) | = 0 | geçti | `p2_0_2/d_a.json, p2_0_2/d_a2.json` |
| H-P2-0.3a′ | Canonical içerik korunumu (D-A) | 200/200 senaryo; değişen canonical soy 52 | 200/200 | geçti | `p2_0_2/d_a.json` |
| H-P2-0.3b | Yanlış `unverified`, W sonunda, yüksüz resmi zincir (D-B test-4, gerçek Obsidian) | 151/2228 = 0.0678 (200 oturum, 413 kayıt, yan not 179; tohum 2671050228) | ≤ 0.01 | kaldı | `p2_0_3/d_b.json` |
| H-P2-0.3b-yük | Aynı ölçüm, yüklü çiftin her iki zinciri | yüklü A 152/2228 = 0.0682; yüklü B 146/2228 = 0.0655 | ≤ 0.02 (ikisi de) | kaldı | `load A, load B: p2_0_3/d_b.json` |
| H-P2-0.3b′ | Biçim başına ve oturum türüne göre (bilgi); P2-0.2 ile karşılaştırma | yeniden adlandırma oturumları 151/1760 (249 kayıt); tablolar aşağıda | raporlanır | raporlandı (bilgi) | `p2_0_3/d_b.json (renames)` |
| H-P2-0.3c | Tasdik gecikmesi p95 (D-B test-4) | 285 ms (164 ölçüm; `canonical` olmayan 0; yazımsız kayıt 249) | < 500 ms | geçti | `p2_0_3/d_b.json` |
| H-P2-0.3d | Karışık katman doğruluğu (D-C); gerçek Obsidian çakışma (D-C′) | D-C 340/340 = 1.0000 (160 tik; ajan `canonical` 0); D-C′ 40/40 = 1.0000, ajan `canonical` 0 (40 tik) | ≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0 | geçti | `p2_0_2/d_c.json, p2_0_3/d_c_prime.json` |
| H-P2-0.3e | Aklama: bağlantı güncellemesi, sahte, katman taşıma (D-A′) | ajan `canonical` 0/50 (50 senaryo); sahte ret 10/10; taşıma: ajan `canonical` 0/10 (10 senaryo) | ajan `canonical` = 0; sahte 10/10 | geçti | `p2_0_3/d_a_prime.json` |
| H-P2-0.3f | Obsidian dışı insan (D-B2) | 0/630 = 0.0000; p95 136.9 ms (150 oturum) | ≤ 0.01; p95 < 500 ms | geçti | `p2_0_2/d_b2.json` |
| H-P2-0.3h | Eski vault'lar (D-E) | 6/6 | 6/6 | geçti | `p2_0_2/d_e.json` |
| H-P2-0.3i | Determinizm: yüksüz ve yüklü çift (D-A, D-A2, D-C günlükleri bayt eşit; bütün çift kümesi) | yüksüz: günlükler eşit True; 111756 alan, fark 0; yapıtlar bayt eşit True; yüklü: günlükler eşit True; 111756 alan, fark 0; yapıtlar bayt eşit True | ikisi de eşit | geçti | `pair_compare.json; yüklü çiftin pair_compare.json'u` |
| H-P2-0.3j | Uyumluluk | 39/39, seviye 3, spec 2.0.2 | 39/39 | geçti | `conformance_p1d1.json` |

## D-B test-4

**D-B test-4, dört zincir** (her zincirin `p2_0_3_checks.json`'u; karşılaştırma: P2-0.2 resmi RUN `p2_0_2_checks.json`, P2-A `info_d_b_test3.json`):

| | A (resmi) | B | yüklü A | yüklü B | P2-0.2 resmi (test-3) | P2-A bilgi (test-3, (A) sonrası) |
|---|---|---|---|---|---|---|
| yanlış `unverified` | 151/2228 = 0.0678 | 151/2228 = 0.0678 | 152/2228 = 0.0682 | 146/2228 = 0.0655 | 116/2743 = 0.0423 | 19/2743 = 0.0069 |
| p95 (ms) | 285 | 297 | 281 | 291 | 299 | 293 |
| yeniden adlandırma oturumları | 151/1760 | 151/1760 | 152/1760 | 146/1760 | 116/2287 | 19/2287 |
| tohum | 2671050228 | 2671050228 | 2671050228 | 2671050228 | 1793744460 | 1793744460 |

**Oturum türüne göre** (W sonunda `unverified` / yeni blok; test-4 ile test-3'ün oturumları farklıdır, sütunlar yalnız yan yana konur):

| Oturum türü | A (resmi) | B | yüklü A | yüklü B | P2-0.2 resmi | P2-A bilgi |
|---|---|---|---|---|---|---|
| char | 0/22 | 0/22 | 0/22 | 0/22 | 0/22 | 0/22 |
| focus | 0/21 | 0/21 | 0/21 | 0/21 | 0/21 | 0/21 |
| move_folder | 36/350 | 36/350 | 36/350 | 35/350 | 1/416 | 1/416 |
| multi | 0/56 | 0/56 | 0/56 | 0/56 | 0/57 | 0/57 |
| paragraph | 0/22 | 0/22 | 0/22 | 0/22 | 0/22 | 0/22 |
| paste | 0/270 | 0/270 | 0/270 | 0/270 | 0/255 | 0/255 |
| rapid | 0/77 | 0/77 | 0/77 | 0/77 | 0/79 | 0/79 |
| rename | 112/1038 | 112/1038 | 115/1038 | 110/1038 | 108/1238 | 10/1238 |
| rename_again | 3/372 | 3/372 | 1/372 | 1/372 | 7/633 | 8/633 |

**H-P2-0.3b′ — yeniden adlandırma oturumlarının yeni blokları, bağlantı biçimine göre** (`renames.by_class`; sınıflar P2-0.2 ASSUMPTIONS X0-7, X0-18; bir blok birden çok sınıfta sayılabilir):

| Biçim | A (resmi) | B | yüklü A | yüklü B | P2-0.2 resmi | P2-A bilgi |
|---|---|---|---|---|---|---|
| embed | 3/35 | 3/35 | 0/35 | 0/35 | 3/39 | 0/39 |
| md_label | 2/26 | 2/26 | 2/26 | 2/26 | 3/41 | 2/41 |
| md_name | 0/44 | 0/44 | 2/44 | 0/44 | 0/38 | 1/38 |
| none | 138/1541 | 138/1541 | 139/1541 | 136/1541 | 100/2058 | 7/2058 |
| self | 4/35 | 4/35 | 4/35 | 4/35 | 2/31 | 0/31 |
| wiki | 4/42 | 4/42 | 5/42 | 4/42 | 1/43 | 0/43 |
| wiki_alias | 0/24 | 0/24 | 0/24 | 0/24 | 2/32 | 2/32 |
| wiki_heading | 4/50 | 4/50 | 4/50 | 4/50 | 7/41 | 7/41 |

Bağlantı ayarları (X0-15, resmi zincir): `{"alwaysUpdateLinks": true, "newLinkFormat": "shortest", "useMarkdownLinks": false}`.

## Aynı zincirin P2-A kaydedicileri (bilgi)

- FAIL özeti (`p2a/historical.json`, referans `reports/rounds/P2-0.2/runs/20261019T212951Z-72fd78f`): sayılan FAIL 0, tarihsel 21, göç gerekli 4.
- Regress (`regress_vs_p2_0_2.json`): 108847 alan, dışlama dışı fark 4 → kaldı.
- Kasıtlı vakalar (`p2a/tamper.json`): geçti.

## Karar kuralı

Kalan hipotezler: H-P2-0.3b, H-P2-0.3b-yük; bağımsız doğrulama açık.

**0.3b kaldı** → varsayılan `canonical` **kesin** kalır; P2-0 dizisi belgelenmiş sınırla kapanır, başka yama turu açılmaz; NORTH_STAR notu olduğu gibi; **P2-L başlar**.

## Notlar (el ile, üreteç çıktısından sonra eklendi; kaynakları adlı)

1. **Zincirler (üretecin başlığındaki adlar arşiv dizinleridir).** Yüksüz çift (`make verify`, `5632b3b`, çıkış 0): A `20261024T000230Z-5632b3b` (resmi RUN, 3372 s), B `20261024T005840Z-5632b3b` (3376 s; arşivde `chain_B/`). Yüklü çift (`VERIFY_LOAD=1 make verify`, 16 × `yes`, çıkış 0): A `20261024T015716Z-5632b3b` (3818 s; `load_pair/chain_A/`), B `20261024T030056Z-5632b3b` (3801 s; `load_pair/chain_B/`). Dördünün manifest'i aynı commit'i ve temiz çalışma ağacını gösterir. Hiçbir gerçek-Obsidian kümesinde yeniden deneme olmadı (`retries` boş). Gerçek Obsidian ikilisi `/Applications/Obsidian.app` (R0 gözden geçirme O5): hiçbir küme `skipped` değil.
2. **H-P2-0.3b'nin kalışı büyük ölçüde sistematiktir.** Dört zincirde 151, 151, 152, 146 / 2228 (0.0655–0.0682); yüksüz A ile B `unverified` sayılarında kayıt kayıt aynı (zamanlama alanları farklı). Kalışın 140/151'i (`espresso`, not 3) dört zincirde aynı bloklardır; kalan tek bloklar zincire göre değişir (yüklü A'da 11, yüklü B'de 5 kayıt yüksüz A'dan farklı; R2 gözden geçirme O1). Oran yükle pek değişmedi (P2-0.2'de 116 → 135/131 idi). H-P2-0.3b-yük'ün kaynağı yalnız yüklü çiftin iki zinciridir; yüksüz zincirlerin dosyalarındaki `H-P2-0.3b-yük` satırı aynı değerin ≤ 0.02'ye karşı kaydıdır, karara girmez (R1 gözden geçirme O5).
3. **Betimleme (resmi zincir, `p2_0_3/d_b.json` `records`; kök neden araştırılmadı — GOAL: hipotez düşerse optimize etme, raporla).** 151 bloğun **140'ı** kendine bağlanan tek bir notun — `espresso.md` (test-4 vault'unda `Bu not: [[espresso]]`; önceki oturumlarla 35–37 bloğa büyümüş) — dört yeniden adlandırmasından: t133 (35/35), t172 (35/37), t173 (35/37), t198 (`move_folder`, 35/37); notun eski bloklarının tamamı W sonunda `unverified`, gelen bağlantıları güncellenen yan notlar `canonical`. Kalan 11 blok birer blok: 9'u yan notlarda, 2'si kendine bağlanmayan `safety` notunun yeniden adlandırmasında (t152 1/43, t180 1/62). Test-4'te kendine bağlantılı 35 yeniden adlandırma var; imzayı yalnız `espresso` gösterdi. İmza, NORTH_STAR'ın P2-0 dizisinden kalan **ölçülen sınırıyla** aynıdır ("kendine bağlantılı büyük notların yeniden adlandırılmasında insan içeriği güvenli tarafta `unverified` kalabilir"). (A) düzeltmesi P2-0.2'nin test-3'ündeki iki notu (t172/t185) kapatmıştı (P2-A bilgi ölçümü 19/2743); yeni tohumun vault'unda aynı sınıfın başka bir örneği kalıyor. Harness sunucunun tanı günlüğünü (`--attest-log`, P2-A/A0 c) açmadığı için nedenin (A)'nın tam olarak kapsamadığı bir yol mu, P2-0.2'nin (B)'si (kabul edilmiş ama eşleşmemiş tasdik) mi olduğu **açıktır**; bu tur ürünü ve ölçümü değiştirmez.
4. **Güvenli taraf korundu.** Kalış yalnız yanlış `unverified` yönündedir: ajan yazımı hiçbir kümede `canonical`'a sızmadı (H-P2-0.3a, e; D-C/D-C′ ajan `canonical` 0), yeniden adlandırma dışı insan oturumlarında dört zincirde 0, Obsidian dışı insan 0/630.
5. **Ürün değişmedi (GOAL "Tasarım").** `git diff main -- core index server clients spec bindings` boş; `git diff main --stat -- research/eval` yalnız iki yeni dosya (`run_p2_0_3.py`, `report_p2_0_3.py`); üreteçler, harness'ler, `run_p2_0_2.py`, eşikler ve `W` (`Z5_tuning.json`) değişmedi (ASSUMPTIONS R0-3'ün dosya hash'leri). Dalın bütün değişiklikleri: `Makefile` (hedef `p2-0-3-checks`), `scripts/verify.sh` (+7, dondurma istisnası P2-0.3/F1), `research/FROZEN.md` (+2 satır), iki sarmalayıcı ve testi, belgeler ve RUN arşivi. Dört zincirde D-A, D-A2, D-C günlükleri P2-A resmi RUN'ıyla bayt eşit.
6. **Kaydediciler.** Zincirin yazdırdığı FAIL özeti (`eval.historical_p2a`, P2-A'nın sınıflandırıcısı) "sayılan 3" gösterir: bunlar bu turun kendi `p2_0_3_checks.json` satırlarıdır (H-P2-0.3b, b-yük kaldı; b′ bilgi, `pass: null`) — P2-A'nın sınıflandırıcısı referans RUN'da olmayan dosyayı tanımaz; regresyon değildir. `p2a_checks.json`'un kendi özeti dört zincirde sayılan 0, tarihsel 21, göç gerekli 4 (P2-A'daki gibi). `p2a_recorders.regress.pass: false` dört zincirde, P2-A resmi RUN'ıyla aynı 4 `check_compare_fields` sayaç alanı (P2-A FINAL not 2; R1 gözden geçirme O4).
7. **Karar kuralı.** Kalan: H-P2-0.3b, H-P2-0.3b-yük; geri kalan bütün hipotezler geçti → GOAL P2-0.3'ün **"0.3b kaldı"** dalı: varsayılan politika `canonical` **kesin** kalır (K-P2-02.5 (1) aynen; `rootmark serve`'ün tarifi `unattested = "canonical"`; `unverified` isteğe bağlı), P2-0 dizisi belgelenmiş sınırla kapanır, başka yama turu açılmaz, NORTH_STAR notu olduğu gibi, **P2-L başlar**. Karar talebi yok (kural adlı dalı verir).
8. **Varsayılan değişikliğinin uygulama planı.** Bu dalda varsayılan değişmez; sonraki turun (P2-L) açılış görevinde **uygulanacak bir varsayılan değişikliği yoktur**. Yapılacak olan yalnız kayıt: DECISIONS'a kapanış kararı (orkestratör, bağımsız doğrulamadan sonra), isteğe bağlı olarak THREAT_MODEL'de sınırın P2-0.3 ölçümüyle güncellenmesi (dalın gerektirdiği değil; R2 gözden geçirme O2) (yeni tohumda da 0.0678; 140/151'i sistematik, oran yükle pek değişmedi), ROADMAP'te P2-0.3 satırının durumu. "Hepsi geçti" dalının planı (tarifin `unattested = "unverified"` yazması, `status.attestation.unattested`, mevcut `policy.toml`'ların değişmemesi, C2-11 ve spec §7.1'in zaten `unverified` demesi) bu sonuçla uygulanmaz.
9. **NORTH_STAR önerisi** (NORTH_STAR değiştirilmedi; insan kararı): karar kuralının dalı gereği "Ölçülen koşul" notu olduğu gibi kalır. İsteğe bağlı, dalın gerektirmediği bir öneri (R2 gözden geçirme O2): ölçümün tarihi ve kaynağıyla sınır cümlesinin pekiştirilmesidir: "… `unverified` kalabilir (P2-0.2; P2-0.3'te yeni tohum ve iki ortamda yeniden ölçüldü: 0.0678; kalışın büyük kısmı sistematik, oran yükle pek değişmedi — varsayılan `canonical` kesin, K-P2-02.5 (1))."
10. **Damga.** Spec §7.1/§8 Draft kalır; ana ajan damga koymaz (DEV_PROCESS §5). Bu dalda "Validated" önerilmez: §7.1'in varsayılan cümlesi ("default `unverified`") ile ürünün fiilî varsayılanı (`canonical`, K-P2-02.5 (1) seçenek b) arasındaki ayrım kalıcı hale gelir; damga ve §7.1 metni bağımsız doğrulamadan sonra insan kararına bırakılır.
11. **Kalite kapıları** (bu dalın ucunda, el ile gözlendi): `make test` çıkış 0 (cargo hepsi; pytest 258 + 1 atlanan; vitest 142), `make lint` çıkış 0, `make conformance` çıkış 0 (39/39, seviye 3); `make verify` yüksüz ve yüklü çıkış 0, iki çift eşit (not 1). `check_frozen` OK, `check_deps` OK.
12. **Bağımsız doğrulama açık** (`docs/rounds/P2-0.3/VERIFY.md`). PR'ın gövdesi bu dosyadır. Ölçülen `5632b3b` dal commit'idir; PR rebase ile birleşirse `main`'de yeniden yazılır (K-P2.2) — doğrulama için dal korunmalıdır.
