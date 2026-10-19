# P2-0.2 — Bağlantı Biçimleri, Yeniden Adlandırmada Katman Taşıma, Ölçüm Determinizmi (Yama Turu)

Başlangıç: P2-0.1 ön-kayıtlı kümelerde "hepsi geçti" (K-P2-01.5); bağımsız doğrulama "şu koşullarla güvenilir" (`reports/rounds/P2-0.1/VERIFICATION.md`). Bu tur iki koşulu kapatır ve P2-0 hipotezini **aynı eşiklerle**, daha geniş bağlantı biçimleriyle yeniden ölçer. "Validated" damgası ve NORTH_STAR önerisi bu turun doğrulamasına bağlıdır (K-P2-01.6). Tur bitmeden P2-A başlamaz.

## Problem (P2-0.1 doğrulamasından)
1. **Bağlantı biçimi boşluğu (F1).** D-B test-2 yalnız `[[wikilink]]` içeriyordu. Etiketi not adına eşit Markdown bağlantısı (`[ad](ad.md)`) ve kendine bağlantı (`[[ad]]`, `[[ad#ad]]`) olan bir vault'ta yeniden adlandırmalarda yanlış `unverified` 111/219 = 0.507. İki neden: **(A)** Obsidian Markdown bağlantısının etiketini de yeni ada çevirir, sunucunun "yalnız hedef değişsin" denetimi bunu reddeder; **(B)** kendine bağlantılı notun yeniden adlandırması ile notun kendi bağlantı güncellemesi tek tasdik olarak ele alınmıyor, yeni yolun bütün blokları `unverified`. **Basamaklanma:** tek bir `unverified` blok, notun sonraki yeniden adlandırmasını (`renamed_from` "tümü canonical" denetimi) düşürür ve bütün notu `unverified` yapar.
2. **Ölçüm determinizmi (F2).** Doğrulayıcının çift koşturmasında D-C `events.jsonl` bayt eşit çıkmadı: `apart` tikte iki ayrı dosyaya 2 ms arayla yazılan insan ve ajan alımlarının sırası ters düştü (kalite alanları ve katmanlar eşit). Ürün değil, kümenin zamanlaması; H-P2-0.1i bağımsız tekrarda sağlanmadı.
3. **Süreç (Ö3, F4).** FINAL "Validated" damgasını bağımsız doğrulamadan önce koydu. CRITERION, başvuru okuyucusunun bilinmeyen alanı reddettiği durumu (2.0.0 okuyucusu 2.0.1'in `partial`/`matched` satırlarını açamıyor) açıkça çözmüyor.

## Hipotez
Bağlantı güncellemesi denetimi Obsidian'ın gerçek yeniden yazımlarını (etiket dönüşümü dahil) kapsar ve yeniden adlandırma katmanları blok blok taşırsa (hiçbir bloğu yükseltmeden ya da düşürmeden), gerçek insan düzenlemeleri yaygın bağlantı biçimleriyle de pratikte `unverified`'a düşmez **ve** ajan yazımı yine canonical'a sızamaz.

## Tasarım (ölçülecek olan)
- **T1 Bağlantı denetimi.** `link_update_of` tasdikinde kabul edilen fark: bağlantı hedefinin eski addan yeni ada dönüşmesi **ve** etiketi eski notun adına (ya da `%20` kodlu yoluna) eşit olan Markdown bağlantısının etiketinin yeni ada dönüşmesi; takma adlı/başlıklı wikilink, gömme, kodlu yol dahil. Başka her fark → 403 `denied`. Spec §7.1 "same link labels" buna göre; CLIENT_CONTRACT §2.6.
- **T2 Yeniden adlandırmada katman taşıma.** `renamed_from` tasdiki artık "tümü canonical baş" istemez: yeni yolun blokları eski yolun aynı sürümlerinin katmanını **aynen** devralır (`canonical` canonical, `unverified` unverified, `proposal` proposal); yeniden adlandırma sırasında notun kendi içindeki bağlantı yeniden yazımı (kendine bağlantı) T1 kuralıyla aynı tasdikte denetlenir; bu kuralla değişen bloklar canonical'dan geliyorsa canonical kalır. Aklama koruması: yeniden adlandırma hiçbir bloğu yükseltmez. Spec §7.1; sürüm CRITERION ile.
- **T3 D-C determinizmi.** `apart` ve `same_doc_after` tiklerde harness insanın alımının günlüğe girmesini bekler (`quiesce`), sonra ajanı yazdırır; `collide` tanımı değişmez. Karşılaştırıcı gevşetilmez. Dondurma istisnası.
- **T4 Süreç ve okuyucu.** DEV_PROCESS §5'e "spec durum damgası 'Validated' bağımsız doğrulamanın 'güvenilir' hükmünden (ya da koşullarının kapanmasından) sonra konur"; CRITERION'a "başvuru okuyucusu bilinmeyen alanı reddediyorsa ekleme kırıcıdır; kırıcı olmaması için okuyucu kuralı 'bilinmeyen alan yok sayılır' spec'te yazılı olmalıdır". Spec §8 tasdik günlüğü okuyucu kuralı: bilinmeyen alan yok sayılır, bilinen alanın bilinmeyen değeri reddedilir; başvuru okuyucusu buna uyar. Spec §7.1/§8 durumu "Draft" (bu turun doğrulamasına kadar).
- **W** = 0.5 s sabit (P2-0 ayarı); ayar yok.

## Veri setleri
- **D-A** P2-0 üreteci, 200 senaryo (değişmez). **D-A2** P2-0.1 üreteci, 120 (değişmez).
- **D-B test-3 (gerçek Obsidian):** yeni tohum (`seed = blake3("P2-0.2/D-B")[:8]`, ön-kayıtlı), 200 oturum, P2-0.1'in tür dağılımı; vault üreteci bağlantı biçimlerini karıştırır: `[[ad]]`, `[[ad|takma]]`, `[[ad#başlık]]`, `![[ad]]`, `[ad](ad.md)`, `[etiket](ad.md)`, `[ad](dizin%20adı/ad.md)`, kendine bağlantı (`[[ad]]`, `[[ad#ad]]`), aynı notun ikinci yeniden adlandırması, klasöre taşıma; yeniden adlandırma oturumu payı ≥ %30. Biçim dağılımı Y0'da ön-kayıtlı.
- **D-A′ (aklama, gerçek Obsidian):** P2-0.1 üreteci, bağlantı biçimleri test-3'teki gibi; 50 senaryo + 10 sahte; ek 10: ajanın `unverified` içerik yazdığı notu insanın yeniden adlandırması (katman taşıma: ajan blokları `unverified` kalmalı).
- **D-C** P2-0 üreteci, T3 ile; 160 tik. **D-C′** P2-0.1 gibi 40 tik.
- **D-E** 3 `v0.1.0` + 2 `ce4cf04` + 1 `21b16ea` (2.0.1 günlüğü, `partial`/`matched` satırlı).
- **D-B2** P2-0.1 üreteci, 150 oturum (CLI; `--renamed-from` katman taşıma ile).

## Önceden kayıtlı eşikler
| Kod | Ölçüm | Küme | Eşik |
|---|---|---|---|
| H-P2-0.2a | Canonical sızıntısı | D-A, D-A2 | = 0 |
| H-P2-0.2a′ | Canonical içerik korunumu | D-A | 200/200 |
| H-P2-0.2b | Yanlış `unverified`, W sonunda | D-B test-3 | ≤ 0.01 |
| H-P2-0.2b′ | Yalnız yeniden adlandırma oturumları; biçim başına (bilgi) | D-B test-3 | raporlanır |
| H-P2-0.2c | Tasdik gecikmesi p95 | D-B test-3 | < 500 ms |
| H-P2-0.2d | Karışık katman doğruluğu; gerçek Obsidian çakışma | D-C; D-C′ | ≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0 |
| H-P2-0.2e | Aklama: ajan bloğu `canonical`; sahte ret; katman taşımada ajan bloğu `canonical` | D-A′ | = 0; 10/10; = 0 |
| H-P2-0.2f | Obsidian dışı insan: yanlış `unverified`; p95 | D-B2 | ≤ 0.01; < 500 ms |
| H-P2-0.2h | Eski vault'lar: açılır, replay, `events_hash`, `attestations.jsonl` değişmez; 2.0.1 günlüğü yeni okuyucuda | D-E | 6/6 |
| H-P2-0.2i | Determinizm: çift koşturma yüksüz **ve** yapay CPU yükü altında; D-C günlükleri bayt eşit | — | ikisi de eşit |
| H-P2-0.2j | Uyumluluk: mevcut 38 + yeni fixture'lar (T1/T2 için en az biri) | — | hepsi |
| H-P2-0.2k | Damga kuralı DEV_PROCESS'te, CRITERION netleştirmesi, §8 okuyucu kuralı ve başvuru okuyucusu uyumlu (ileri uyumluluk testi: bilinmeyen alanlı satır okunur, bilinmeyen değer reddedilir) | — | yazılı ve testli |

## Karar kuralı
- **Hepsi geçti** → bağımsız doğrulama "güvenilir" (ya da koşulları kapanmış) ise spec §7.1/§8 "Validated"; `unverified` varsayılan kalır; NORTH_STAR önerisi FINAL'de (insan kararı); **P2-A başlar**. Doğrulama "şu koşullarla" derse damga beklenir, koşullar P2-A'nın açılış görevinde (E0 emsali) kapatılır, P2-A başlar.
- **0.2b veya 0.2f kaldı** → GOAL P2-0'ın 0b dalı: `unverified` varsayılan **olmaz**, politika `canonical` ile yayımlanır, vaat "tasdikli istemci ile" daraltılır (karar kaydıyla); P2-A başlar.
- **0.2a, 0.2a′, 0.2e kaldı** → tasarım kusuru; tur kaldı, karar talebi; P2-A başlamaz.
- **0.2d, 0.2h, 0.2i, 0.2j, 0.2k kaldı** → altyapı kusuru; tur kaldı, karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
W ayarı; eşik değişikliği; P2-0/P2-0.1 test setlerinin yeniden okunması; MCP'de `attest`; eklentinin hata ayıklayıcı altında tasdik etmemesi; OS kullanıcı ayrımı; yayım (paket sürümü değişmez); spec §1–6.
