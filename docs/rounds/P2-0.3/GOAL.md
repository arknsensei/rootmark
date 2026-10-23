# P2-0.3 — Tasdikli Yazma: Yeniden Ölçüm (Hipotez Turu, Kod Değişikliği Yok)

Başlangıç: P2-A tamamlandı (K-P2A.5). P2-A/A0'ın (A) düzeltmesinden sonraki **bilgi ölçümü** P2-0.2'nin ön-kayıtlı kümesinde yanlış `unverified` 19/2743 = 0.0069 verdi (`reports/rounds/P2-A/info_d_b_test3.json`). Bu tur o sinyali ön-kayıtlı bir hipoteze çevirir: ürün ve ölçüm kodu **değişmez**; yalnız yeni tohumla, iki ortamda ölçüm ve bağımsız teyit. Hedef süre: ölçüm yarım gün, doğrulama 2–3 saat.

## Hipotez
(A) düzeltmesiyle (K-P2-02.5 (2), P2-A/A0), tasdikli istemci kullanan gerçek insan düzenlemeleri yaygın Obsidian bağlantı biçimleri ve yeniden adlandırmalar dahil pratikte `unverified`'a düşmez (P2-0 0b, aynı eşik), ajan yazımı canonical'a sızmaz (P2-0 0a) ve bu ölçüm ortama bağlı bir yarış değildir (yük altında da eşiğin içinde).

## Tasarım
- Ürün: `main` HEAD aynen (P2-A birleşmiş). Hiçbir kaynak, spec, eşik, W (0.5 s), üreteç ya da karşılaştırıcı değişmez. Değişirse tur geçersizdir.
- **D-B test-4:** P2-0.2'nin üreteci ve bağlantı biçimi dağılımı (ASSUMPTIONS P2-0.2 X0-7/X0-15/X0-18 aynen), **yeni tohum** `seed = blake3("P2-0.3/D-B")[:8]` (R0'da ön-kayıtlı), 200 oturum, gerçek Obsidian. P2-0.2 test-3 ve P2-A bilgi ölçümünün tohumu yeniden kullanılmaz.
- **D-A, D-A2, D-C, D-C′, D-A′, D-B2, D-E:** P2-0.2'nin kümeleri aynen (regresyon; yeniden ölçülür, değişmez beklenir).
- **İki ortam:** yüksüz çift (`make verify`) resmi; `VERIFY_LOAD=1` çift ikinci ortam. Dört zincirin D-B değerleri raporlanır.

## Önceden kayıtlı eşikler
| Kod | Ölçüm | Küme | Eşik |
|---|---|---|---|
| H-P2-0.3a | Canonical sızıntısı | D-A, D-A2 | = 0 |
| H-P2-0.3a′ | Canonical içerik korunumu | D-A | 200/200 |
| H-P2-0.3b | Yanlış `unverified`, W sonunda, **yüksüz resmi zincir** | D-B test-4 | ≤ 0.01 |
| H-P2-0.3b-yük | Aynı ölçüm, **yüklü çiftin her iki zinciri** | D-B test-4 | ≤ 0.02 |
| H-P2-0.3b′ | Biçim başına ve oturum türüne göre (bilgi); P2-0.2 ile karşılaştırma | D-B test-4 | raporlanır |
| H-P2-0.3c | Tasdik gecikmesi p95 | D-B test-4 | < 500 ms |
| H-P2-0.3d | Karışık; gerçek Obsidian çakışma | D-C; D-C′ | ≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0 |
| H-P2-0.3e | Aklama (bağlantı güncellemesi, sahte, katman taşıma) | D-A′ | ajan `canonical` = 0; sahte 10/10 |
| H-P2-0.3f | Obsidian dışı insan | D-B2 | ≤ 0.01; p95 < 500 ms |
| H-P2-0.3h | Eski vault'lar | D-E | 6/6 |
| H-P2-0.3i | Determinizm: yüksüz ve yüklü çift (D-A, D-A2, D-C günlükleri bayt eşit) | — | ikisi de eşit |
| H-P2-0.3j | Uyumluluk | — | 39/39 |

## Karar kuralı
- **Hepsi geçti** → hipotez kanıtlandı. Bağımsız doğrulama "güvenilir" (ya da koşulları kapanmış) ise: `unverified` yeniden **varsayılan** olur (`rootmark serve`'ün yazdığı tarif `unattested = "unverified"`; K-P2-02.5 (1) geri alınır; mevcut `policy.toml`'lar değişmez — uygulama bir sonraki turun açılış görevinde, bu turda kod değişmez), spec §7.1/§8 "Validated" (CRITERION, damga kuralı DEV_PROCESS §5), NORTH_STAR'ın "ölçülen koşul" notu yeniden yazılır (insan kararı; öneri FINAL'de), P2-0 dizisi kesin kapanır; **P2-L başlar**.
- **Yalnız 0.3b-yük kaldı** (yüksüz geçti) → hipotez kanıtlandı, **belgelenmiş sınırla**: "yük altında yanlış `unverified` %2'yi aşabilir"; `unverified` varsayılan olur, sınır THREAT_MODEL ve NORTH_STAR notunda; P2-L başlar.
- **0.3b kaldı** → varsayılan `canonical` **kesin** kalır; P2-0 dizisi belgelenmiş sınırla kapanır, başka yama turu açılmaz; NORTH_STAR notu olduğu gibi; P2-L başlar.
- **0.3a, 0.3a′, 0.3e kaldı** → tasarım kusuru; tur kaldı, karar talebi; P2-L başlamaz.
- **0.3d, 0.3f, 0.3h, 0.3i, 0.3j kaldı** → altyapı/regresyon kusuru; tur kaldı, karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
Her türlü ürün, spec, üreteç, eşik, W değişikliği; test-3 ya da bilgi ölçümünün tohumunun yeniden kullanımı; varsayılanın bu turda değiştirilmesi (sonraki turun açılışında); yayım.
