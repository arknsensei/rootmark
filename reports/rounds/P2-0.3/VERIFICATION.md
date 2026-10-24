# P2-0.3 — Bağımsız Doğrulama (dar kapsam)

- **Doğrulanan commit:** `5686196391e9bb5b8fcf733410aa851a7d05945e` (`p2-0.3/round`, PR #17'nin başı, `OPEN`). FINAL'in ölçtüğü commit `5632b3b`; `ROUND_START` `c50ee2f`.
- **Yöntem:** DEV_PROCESS §5, VERIFY.md'deki V0, V1, V3, V7, V9 ve V10. Ana dizinden yalnız `docs/rounds/P2-0.3/VERIFY.md` ve DEV_PROCESS §5 okundu; geri kalan her şey scratchpad'deki temiz klondan (`5686196`) okundu ve her koşturma orada yapıldı. Ana dizine yalnız bu dosya yazıldı; kod değiştirilmedi, commit yapılmadı.
- **Kendi araçlarım** (scratchpad'de kaldı, depoya girmedi):
  - deterministik vault ve düzenleme üreteci;
  - kendi CDP sürücüm: gerçek giriş olayları, kaydetme yolları (`editor:save-file` komutu, otomatik kaydetme, odak kaybı) ve `app.fileManager.renameFile`;
  - biçim başına rapor betiği;
  - `d_b.json` kayıt sayacı.
- **Ortam:** macOS 27.0.1 arm64, 16 mantıksal çekirdek, Obsidian 1.13.4, Node 24.3.0.
- **Obsidian ayrımı:** kullanıcının açık Obsidian'ına (PID 24764) dokunulmadı. V3 koşumlarım kendi `--user-data-dir`'i ve CDP portu olan ayrı bir örnekte yapıldı ve her koşumdan sonra kapatıldı. Hiçbiri bir çiftle aynı anda koşmadı: V3 yüksüz çift bittikten (11:07Z) sonra, yüklü çift başlamadan (11:24Z) önce koştu.

## Hüküm

**Şu koşullarla güvenilir.**

FINAL'in **karar dalı** bağımsız olarak doğrulandı: **"0.3b kaldı"**. Varsayılan `canonical` kesin kalır, P2-0 dizisi belgelenmiş sınırla kapanır, NORTH_STAR notu olduğu gibi kalır, P2-L başlar.

Hipotezin kendisi kanıtlanmadı:
- **Kendi dört zincirim:** H-P2-0.3b ve b-yük kaldı (0.0678 / 0.0112 yüksüz; 0.0655 / 0.0660 yüklü).
- **Kendi vault'um (V3):** "W sonunda `unverified` ≤ 1" ölçütü kaldı: yüksüzde 10 blok, yükte 11 blok.

Dolayısıyla `unverified` varsayılanını geri getirecek bir "güvenilir" hükmü **yok**.

Koşul: FINAL'in kalışı betimleyen notları (2, 3, 8, 9) ölçümle uyumlu hale getirilmeden kapanış kaydına (DECISIONS, THREAT_MODEL) taşınmamalı. İki nokta düzeltilmeli:
- **"Büyük ölçüde sistematik, 140/151'i dört zincirde aynı bloklar" doğru değil.**
  - 140 blok tek bir olaydır: t133'te `espresso`'nun yeniden adlandırılmasında 35 blok `unverified` kalır. Yeniden adlandırma katmanı taşıdığı için aynı 35 blok t172, t173 ve t198'de yeniden sayılır.
  - Bu olay altı zincirin beşinde oldu (FINAL'in dördü ve benim iki yüklü zincirim). Benim yüksüz A zincirimde de oldu; yüksüz B zincirimde olmadı.
  - Yani olay yüksek olasılıklı bir yarıştır, deterministik değildir.
- **Olay olmasa da 0.3b kalıyor.**
  - Benim B zincirim 25 tekil blokla 0.0112 verdi.
  - V3'te kalışın ana sınıfı, büyük bir not yeniden adlandırıldığında Obsidian'ın gelen bağlantılarını yeniden yazdığı **yan notlardır**.

## Hipotez başına

Bağımsız ölçümüm iki çiftten, dört zincirden ve V3'ten oluşuyor. Sütun sırası: yüksüz A, yüksüz B, yüklü A, yüklü B.

| Kod | Eşik | FINAL | Bağımsız ölçüm | Sonuç |
|---|---|---|---|---|
| H-P2-0.3a | = 0 | D-A 0/525, D-A2 0/210 | dört zincirde D-A 0/525, D-A2 0/210 | **geçti** |
| H-P2-0.3a′ | 200/200 | 200/200 | dört zincirde 200/200 | **geçti** |
| H-P2-0.3b | ≤ 0.01 (yüksüz resmi zincir) | 151/2228 = 0.0678 | 151/2228 = 0.0678 ve 25/2228 = 0.0112 | **kaldı** |
| H-P2-0.3b-yük | ≤ 0.02 (yüklü çiftin iki zinciri) | 152 → 0.0682; 146 → 0.0655 | 146/2228 = 0.0655 ve 147/2228 = 0.0660 | **kaldı** |
| H-P2-0.3b′ | raporlanır | tablolar | aşağıdaki tablolar; V3 biçim tablosu | raporlandı (bilgi) |
| H-P2-0.3c | p95 < 500 ms | 285 ms | 283, 296, 271, 315 ms; V3'te yazılı kaydetmeler 125 ms (yükte 137 ms) | **geçti** |
| H-P2-0.3d | D-C ≥ 0.99; D-C′ ≥ 0.95 ve ajan `canonical` = 0 | 340/340; 40/40, 0 | dört zincirde 340/340 = 1.0; D-C′ 40/40, ajan `canonical` 0 | **geçti** |
| H-P2-0.3e | ajan `canonical` = 0; sahte 10/10 | 0/50; 10/10 | dört zincirde 0/50, sahte ret 10/10, taşıma 0/10 | **geçti** |
| H-P2-0.3f | ≤ 0.01; p95 < 500 ms | 0/630; 136.9 ms | dört zincirde 0/630; p95 137.7, 140.4, 138.5, 132.4 ms | **geçti** |
| H-P2-0.3h | 6/6 | 6/6 | dört zincirde 6/6 | **geçti** |
| H-P2-0.3i | iki çift de eşit | eşit | yüksüz çift: çıkış 0, 111 756 alan, fark 0, `bytes_equal` true. Yüklü çift: aynı sonuçlar. Dört zincirde D-A, D-A2, D-C günlükleri P2-A resmi RUN'ıyla eşit | **geçti** |
| H-P2-0.3j | 39/39 | 39/39 | dört zincirde 39/39, seviye 3. Kendi koşumum da 39/39, seviye 3, spec 2.0.2 | **geçti** |
| V3 (VERIFY) | W sonunda `unverified` ≤ 1 | — | 10 (yüksüz), 11 (yük altında) | **kaldı** |

## V0 — Değişmezlik

- **Ürün kodu değişmedi.** `git diff c50ee2f 5686196 -- core index server clients spec NORTH_STAR.md research/eval/datasets research/eval/run_p2_0_2.py bindings` boş çıktı.
- **Dalın ürün dışı değişiklikleri ve yeni dosyaları:**
  - `Makefile`: `p2-0-3-checks` hedefi; `.PHONY`'de 1 satır.
  - `scripts/verify.sh`: +7 satır (dondurma istisnası F1).
  - `research/FROZEN.md`: +2 satır.
  - Yeni dosyalar: `research/eval/run_p2_0_3.py`, `report_p2_0_3.py` ve testi.
  - Belgeler ve RUN arşivi.
- **Sarmalayıcı yalnız tohum ve RUN adı geçiriyor.** `_guard` başka tohumları reddediyor; P2-0.2 test-3'ün ve P2-A bilgi ölçümünün tohumu olan `1793744460` reddedilenler arasında.
- **Tohum veriden önce kaydedilmiş.** Tohum `2671050228 = int(blake3("P2-0.3/D-B")[:8], 16)`; kendim yeniden hesapladım. R0 commit'i `5632b3b`'de (02:58+03) kayıtlı, veri sonra geldi (`c3f7d78`, 07:06+03).
- **Sonuç: geçti.**

## V1 — Çift koşturma, iki ortam

Klon `5686196` üzerinde iki koşum yaptım:

| Çift | Zincirler (RUN) | Çıkış | `pair_compare` |
|---|---|---|---|
| Yüksüz: `verify.sh pair` | A `20261024T091903Z-5686196`, B `20261024T101453Z-5686196`; 09:15Z–11:07Z | 0 | `equal` true, 111 756 alan, `bytes_equal` true |
| Yüklü: `VERIFY_LOAD=1`, 16 × `yes` | A `20261024T112902Z-5686196`, B `20261024T123155Z-5686196`; 11:24Z–13:30Z | 0 | aynı |

Her iki çiftte de `p1e_e0` kaydedilmiş bir çıkış 101 gösteriyor; FINAL'in zincirlerinde de aynısı var ve kriter dışı. P2-A'nın sınıflandırıcısıyla sayılan FAIL dört zincirde de 0.

**D-B test-4** (çift karşılaştırmasının dışında; `d_b.json`'dan kendim saydım):

| | yüksüz A | yüksüz B | yüklü A | yüklü B | FINAL (A / B / yüklü A / yüklü B) |
|---|---|---|---|---|---|
| yanlış `unverified` | 151/2228 = 0.0678 | **25/2228 = 0.0112** | 146/2228 = 0.0655 | 147/2228 = 0.0660 | 151 / 151 / 152 / 146 |
| p95 (ms) | 283 | 296 | 271 | 315 | 285 / 297 / 281 / 291 |
| t133 `espresso` olayı (35 blok, ×4 sayım) | var | **yok** | var | var | dördünde var |
| olay dışı bloklar | 11 | 25 | 6 | 7 | 11 / 11 / 12 / 6 |

**Yön ve dal.** Dört zincirin dördünde de 0.3b kalıyor (en düşük değer 0.0112 > 0.01), iki yüklü zincirde de b-yük kalıyor. Eşik dalı FINAL'inkiyle aynı.

**Oturum türüne göre** (W sonunda `unverified` / yeni blok):

| Tür | yüksüz A | yüksüz B | yüklü A | yüklü B |
|---|---|---|---|---|
| char, focus, multi, paragraph, paste, rapid | 0 | 0 | 0 | 0 |
| move_folder | 36/350 | 2/350 | 35/350 | 35/350 |
| rename | 112/1038 | 18/1038 | 110/1038 | 111/1038 |
| rename_again | 3/372 | 5/372 | 1/372 | 1/372 |

**H-P2-0.3b′ — yeniden adlandırma oturumları, biçime göre:**

| Biçim | yüksüz A | yüksüz B | yüklü A | yüklü B |
|---|---|---|---|---|
| embed | 3/35 | 8/35 | 0/35 | 0/35 |
| md_label | 2/26 | 0/26 | 2/26 | 2/26 |
| md_name | 0/44 | 8/44 | 0/44 | 1/44 |
| none | 138/1541 | 8/1541 | 136/1541 | 136/1541 |
| self | 4/35 | 0/35 | 4/35 | 4/35 |
| wiki | 4/42 | 0/42 | 4/42 | 4/42 |
| wiki_alias | 0/24 | 1/24 | 0/24 | 0/24 |
| wiki_heading | 4/50 | 0/50 | 4/50 | 4/50 |

**FINAL ile fark (şüphe değil, betimleme hatası).**
- **140 bloğun kaynağı tek bir olaydır.** FINAL not 3 "151 bloğun 140'ı dört yeniden adlandırmadan" der; kayıtlar ise şunu gösteriyor:
  - t133'te notun 35 bloğunun tamamı W sonunda `unverified` kalıyor.
  - Yeniden adlandırma katmanı taşıdığı için aynı 35 blok t172, t173 ve t198'de yeni blok olarak bir daha sayılıyor (35 × 4 = 140).
  - Yani bağımsız 140 kalış değil, bir olayın dört sayımı var. Ölçüm kuralı (`unverified_after_window`'un kayıt başına sayılması) dondurulmuş koddur ve kendi başına yanlış değildir; yorumdaki "sistematik" nitelemesi yanlıştır.
- **Olay yarıştır, dört zincirde ortak değildir.** Olay altı zincirin beşinde oldu, benim yüksüz B zincirimde olmadı. t129'daki tasdik gecikmesiyle bir bağıntı görmedim.
- **Olay dışında da eşik aşılıyor.** Olay dışı tekil kalışlar zincire göre 0–25 arasında değişiyor ve yalnız tekil bloklarla da eşik aşılıyor (yüksüz B: 25).
- **Not 2'nin "kayıt kayıt aynı" ifadesi.** FINAL'in iki yüksüz zinciri için bu ifade doğru olabilir. Ancak genelleme ("dört zincirde aynı bloklar") benim zincirlerimde tutmuyor.

## V3 — İnsan düzenleme, kendi vault'um

**Vault** (deterministik üreteç, 24 not):
- **Büyük notlar:** üç büyük ve kendine bağlantılı not: `atlas.md` 66 blok, `gunluk.md` 65 blok, klasördeki `proje/plan.md` 65 blok. Bunlar P2-0.2 doğrulamasının (A) senaryosudur.
- **Orta not:** kendine bağlantılı `tarif.md`, 22 blok.
- **Küçük notlar:** 20 not. İçerdikleri bağlantı biçimleri: wiki, wiki alias, wiki heading (alias'lı dahil), embed (başlık ve not), Markdown adla, Markdown etiketle, Markdown yolla ve komşu ya da kendine bağlantılar.

**Düzenlemeler:** 50 düzenleme, bunların **23'ü yeniden adlandırma**:
- yeniden adlandırma türleri: kökte, klasöre, klasörden çıkış, klasörler arası ve geri dönüş;
- kaydetme yolları: komut, otomatik kaydetme ve odak kaybı;
- metin konumu: notun sonu ve ortası; çok paragraflı yapıştırma.

**Ayarlar:**
- Obsidian bağlantı ayarları: `alwaysUpdateLinks: true`, `newLinkFormat: shortest`, `useMarkdownLinks: false`.
- Politika: `unattested = "unverified"`, `window_s = 0.5`.
- Sunucu ve eklenti: `rootmark serve --http --watch` ve PR başının eklentisi.

**Ölçüm adımları:**
- Her düzenlemeden önce bütün notların `blame`'i alındı.
- Düzenlemeden sonra son yazımın üzerine W + 1 s beklendi.
- Ardından değişen notların `blame`'i yeniden alındı.

| Koşum | W sonunda `unverified` (ayrık blok) | kayıt başına | rol başına | yazılı kaydetme p95 |
|---|---|---|---|---|
| **yüksüz (resmi)** | **10** | 36/1047 = 0.0344 | düzenlenen 0/29; yeniden adlandırılan 4/915; yan not 32/103 | 125 ms (n = 27) |
| **yük altında** (16 × `yes`) | **11** | 44/1047 = 0.0420 | yan not 39/103 | 137 ms |
| bilgi: `window_s = 2.0` (spec'in başvuru değeri) | 3 | 10/1047 = 0.0096 | — | 123 ms |

Ölçütün değerlendirmesi: VERIFY V3'ün "W sonunda `unverified` ≤ 1" ölçütü **kaldı**: yüksüzde 10, yükte 11 blok.

**Biçim başına** (yeniden adlandırma kayıtlarının yeni blokları; bir blok birden çok biçimde sayılabilir):

| Biçim | yüksüz | yük altında |
|---|---|---|
| embed | 3/32 | 9/32 |
| md_label | 8/35 | 5/35 |
| md_name | 6/27 | 7/27 |
| md_path | 0/1 | 1/1 |
| none | 2/852 | 2/852 |
| self | **0/50** | **0/50** |
| wiki | 1/68 | 4/68 |
| wiki_alias | 7/39 | 0/39 |
| wiki_heading | 9/63 | 16/63 |

**Gözlemler:**
1. **(A) senaryosu geçti.** Büyük, kendine bağlantılı notların kendi blokları, yeniden adlandırma ve sonrasında yapılan düzenlemeler dahil, iki koşumda da 0 kez `unverified` kaldı (`self` satırı ve "yeniden adlandırılan" rolü). Yeniden adlandırılan rolündeki 4 blok küçük notlardan geldi.
2. **Kalış yan notlarda.** Büyük bir not yeniden adlandırılınca Obsidian'ın gelen bağlantılarını yeniden yazdığı yan notlarda kalış görülüyor.
   - **Sıraya bağlı:** düzenleme 1'de (`atlas` yeniden adlandırması) `not00`–`not10` geçti, `not13`–`not19` kaldı.
   - **Biçime bağlı değil:** aynı biçimler bir notta geçip başka notta kalıyor; örneğin `not00` ve `not16`'nın biçimleri aynı.
   - Bu imza FINAL'deki "yan notlar `canonical`" betimlemesiyle çelişiyor; test-4'te de olay dışı tekil kalışların çoğu yan notlarda (FINAL not 3: 11 bloğun 9'u).
3. **Büyük not geç tasdik ediliyor.** Büyük notun kendi kaydı pencerenin kıyısında tasdik ediliyor: `gunce.md`'nin 70 bloğu ve `arsiv/atlas.md`'nin 68 bloğu önce `unverified` olarak alındı, sonra geç tasdikle yükseldi.
4. **W = 2.0'da kalış 10'dan 3'e iniyor** (bilgi). Kalan 3 blok `plan`'ın klasör taşımasında Markdown yol bağlantıları yeniden yazılan `not10` ve `not11` ile "Son plan notu." bloğu.
5. **Açıklanmamış düşüş.** Düzenleme 30'da yazılan ve `canonical` olan "Son plan notu." bloğu, düzenleme 38'deki yeniden adlandırmadan sonra `unverified` göründü. Bu, §7.1'in "yeniden adlandırma katman yükseltmez, düşürmez" ilkesiyle çelişebilir. Kök nedeni araştırılmadı; harness ve sürücüm `--attest-log` açmıyor. Gözlem olarak kaydediyorum, ihlal hükmü vermiyorum.

## V7 — Uyumluluk

- **Kendi koşumum:** PR başının ikilisiyle `rootmark conformance`, modeller `ROOTMARK_MODELS` ile verildiğinde **39/39, seviye 3, spec 2.0.2**. Modelsiz koşumda embedder testleri değerlendirme hatası veriyor (31/39, seviye 2). Bu ortam kaynaklı; ürün kusuru değil.
- **Zincirler:** dört zincirin `conformance_p1d1.json`'u 39/39.
- **Sonuç: geçti.**

## V9 — Ön-kayıt, dondurma, karar kuralı

- **Denetleyiciler:** `scripts/check_frozen.sh` OK, `scripts/check_deps.sh` OK.
- **Değişmezlik:** NORTH_STAR ve spec `ROUND_START`'tan beri değişmedi.
- **Damga:** §7.1 ve §8 hâlâ "Draft"; damga konmamış.
- **Ön-kayıt:** tohum R0'da, veriden önce kaydedilmiş (V0).
- **Karar kuralının bağımsız türetimi.** GOAL'e göre "hepsi geçti" ve "yalnız 0.3b-yük kaldı" dalları 0.3b'nin geçmesini gerektirir; 0.3b kaldı. Tasarım (0.3a, a′, e) ve altyapı (d, f, h, i, j) hipotezlerinin hepsi geçti. Dolayısıyla geçerli dal **"0.3b kaldı"**, yani FINAL'inkiyle aynı.
- **Ufak bulgu:** spec §7.1'in Draft notu hâlâ "P2-0.2'nin bağımsız doğrulamasına kadar" diyor. Bu atıf eskimiş.
- **Sonuç: geçti.**

## V10 — Vaat sorusu

**Vaat ne zaman tutuyor?** Vaat şu: dosya erişimli bir ajanın tasdiksiz yazımı insan içeriği gibi görünmez ve insan düzenlemesi `canonical` kalır. P2-0.3'ten sonra durum şöyle:

- **Ajan tarafı koşulsuz tutuyor.** Tasdiksiz doğrudan yazım `unattested = "unverified"` ile hiçbir kümede `canonical`'a sızmadı. Bu dört zincirde, aklama ve sahte tasdik denemeleri dahil geçerli (a, a′, d, e).
  - Ancak bu yalnız `unverified` politikası seçildiğinde geçerli. Varsayılan `canonical`'da tasdiksiz yazım tanım gereği `canonical` olur. Yani vaatin ajan yarısı **isteğe bağlı bir ayardır**, varsayılan değil.
- **İnsan tarafı bazı koşullarda tutuyor.**
  - **Koşulsuz tutan durumlar:**
    - tasdikli istemci ve tek not düzenlemesi; yazı, yapıştırma ve odak dahil her kaydetme yolu (test-4'te dört zincirde 0; V3'te düzenlenen 0/29);
    - Obsidian dışı insan düzenlemesi (0/630);
    - büyük, kendine bağlantılı notun kendi blokları (V3'te 0/50).
  - **Tutmayan durum:** yeniden adlandırma ve klasör taşıma oturumları. Obsidian'ın gelen bağlantıları yeniden yazdığı yan notlar ile ara sıra yeniden adlandırılan notun kendisi, W = 0.5 s'de yüzde birkaç oranında `unverified` kalır:
    - test-4: tekil 6–25 blok / 2228; olaylı zincirlerde 0.066–0.068;
    - V3: 10–11 blok.
  - **Kalışın niteliği:**
    - Yük oranı belirgin biçimde değiştirmiyor.
    - Olay deterministik değil.
    - Kalış yalnız güvenli yönde: insan içeriği `unverified` görünür, ajan içeriği `canonical` görünmez.

**NORTH_STAR "Ölçülen koşul" notu için önerdiğim metin** (insan kararı; karar kuralının dalı notun olduğu gibi kalmasını söylüyor, bu yalnız pekiştirme önerisi):

> **Ölçülen koşul (P2-0 dizisi, 2026-10-20; P2-0.3 ile yeniden ölçüldü, 2026-10-24; K-P2-02.5):** dosya erişimli bir ajanın tasdiksiz doğrudan yazımı, `unattested = "unverified"` seçildiğinde hiçbir ölçülen yolda `canonical` sayılmaz (sızıntı 0, sahte tasdik 10/10 ret). Tasdikli istemciyle yapılan tek not insan düzenlemeleri `canonical` kalır. Obsidian'ın yeniden adlandırma ve klasör taşımada gelen bağlantıları yeniden yazdığı notlarda insan içeriğinin bir kısmı güvenli tarafta `unverified` kalabilir: yeni tohumda ve iki ortamda 0.011–0.068, bağımsız vault'ta 10/1047 blok; yarışa bağlı, yükle belirgin değişmiyor. Bu yüzden varsayılan `canonical`'dır ve `unverified` isteğe bağlıdır.

**Damga görüşü: "Validated" konamaz.** Nedenleri:
1. Damga kuralı (DEV_PROCESS §5) "güvenilir" hüküm ister. Bu hüküm "şu koşullarla güvenilir"dir ve hipotez kanıtlanmadı.
2. §7.1'in "default `unverified`" cümlesi ürünün kesinleşen varsayılanıyla (`canonical`) çelişiyor. Bu metin değişmeden §7.1 doğrulanmış sayılamaz.
3. V3'te ölçüt kaldı. Ayrıca yeniden adlandırma sonrasında `canonical`'dan `unverified`'a açıklanmamış bir düşüş gözlendi (V3 gözlem 5). §7.1'in yeniden adlandırma ilkesi bu ölçümle teyit edilmiş değil.

§7.1 ve §8 Draft kalmalı. Varsayılan cümlesinin ve eskimiş P2-0.2 atfının nasıl düzeltileceği insan kararıdır.

## İhlal / fark / şüphe

- **İhlal:** yok. Ürün, spec, eşik, W, üreteç ve karşılaştırıcı değişmemiş. Tohum ön-kayıtlı. İki çift de eşit.
- **Fark:**
  - FINAL not 2 ve 3'teki "büyük ölçüde sistematik; 140/151 dört zincirde aynı bloklar" betimlemesi ölçümle uyuşmuyor (V1). Not 8 ve 9'daki "140/151'i sistematik" ifadesi bu betimlemeyi THREAT_MODEL ve NORTH_STAR önerisine taşıyor.
  - FINAL "yan notlar `canonical`" diyor; V3'te kalışın ana sınıfı yan notlar.
- **Şüphe:** yeniden adlandırma sonrasında `canonical`'dan `unverified`'a düşüş (V3 gözlem 5), kök neden araştırılmadı.

## Öneriler

1. **Kapanış kaydını düzelt.** DECISIONS kapanış kararına, THREAT_MODEL sınır metnine ve NORTH_STAR önerisine "sistematik" yerine şu betimlemeyi yaz: "yarışa bağlı; bir olayın katman taşımasıyla tekrar sayılması (35 × 4) artı yan not bağlantı güncellemeleri; olaysız zincirde de 0.0112". FINAL not 2, 3, 8 ve 9 için bu dosyaya atıf yeterli.
2. **Pencereyi ölç ve kök nedeni açıkla** (P2-L ya da sonrasında bir bilgi ölçümü olarak; varsayılan kararını yeniden açmaz). Spec'in başvuru değeri W = 2.0 ile P2-0 ayarı 0.5'i karşılaştır; V3'te 10 bloğa karşı 3 blok kalıyor. Harness'te `--attest-log`'u açarak yan not ve `espresso` kalışlarının (A)'nın kapsamadığı bir yoldan mı, P2-0.2'nin (B)'sinden mi geldiğini belirle.
3. **Düşüşü araştır ve spec'i hizala.** Yeniden adlandırma sonrasındaki `canonical` → `unverified` düşüşünü (V3 gözlem 5) `--attest-log` ile yeniden üret ve §7.1'in yeniden adlandırma ilkesine karşı sına. Ardından §7.1'in varsayılan cümlesini ve eskimiş "P2-0.2 doğrulaması" atfını ürünün kesin varsayılanına göre düzelt (insan kararı; damga ondan sonra değerlendirilir).
