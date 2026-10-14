# P1-C — Ek (tur başlamadan önce `docs/rounds/P1-C/ADDENDUM.md` olarak eklenir; GOAL ve TASKS'in parçasıdır)

P1-B.1 doğrulamasının üç önerisi bu turda, C0 görevi içinde kapatılır. Çıkış kriterlerine iki satır eklenir.

## Kararlar (C0'da DECISIONS.md'ye eklenir)
- **K-C0.1 Yeniden numaralı silme kabul edilmiş sınırdır.** `layer` taşıyan bir günlükte bir `LayerChanged` satırı silinip `seq` yeniden numaralanırsa ve aynı snapshot'a kadar başka katman taşıyan olay kalmazsa, günlük "türetilmiş önek" olarak açılır; bu tespit edilemez ve katman yalnız düşebildiği için güvenlik açığı değildir. Yeni vault'lar için günlük biçim işareti (ilk satırda `{"format":"mdkernel-events/2"}`) **spec 2.0'a** ertelenir; §4.2 donmuş olduğu için 1.x'te eklenmez.
- **K-C0.2 Negatif uyumluluk vakası.** `spec/CONFORMANCE.md`'ye Seviye 2 **C2-09**: `traces.jsonl` içinde geçersiz `query_hash` (31 hex, 33 hex, hex olmayan, büyük harf) ve `\n` içeren `consumer` satırları → açılışta ret. Spec sürümü değişmez (1.0.1'in zaten normatif gereği); fixture ve MANIFEST satırı eklenir, mevcut fixture'lar değişmez.
- **K-C0.3 Sürüm gerekçesi kuralı.** Bundan sonra her GOAL'de spec sürüm kararının gerekçesi `docs/SPEC_EDITS_CRITERION.md`'deki sürüm kuralına atıfla yazılır; bölüm durumu spec'teki güncel damgadan alınır.

## C0'a eklenen işler
- `docs/rounds/P1-B.1/progress/ASSUMPTIONS.md` B11-2 metni doğrulayıcının koşuluyla düzeltilir (tarihli ek olarak; eski metin silinmez) ve iki `Err` vakası (aynı snapshot'ta iki `LayerChanged`, ilki silinip yeniden numaralı) `core/tests/compat_v3final.rs`'e sabitlenir.
- C2-09 fixture'ı ve `gen_fixtures_c2.sh` güncellemesi; `conformance.json` C2 9/9.

## Çıkış kriterlerine ek
| Kod | Ölçüm | Eşik |
|---|---|---|
| C-12 | C2-09 | geçer; `level_claimed 2` için zorunlu |
| C-13 | Yeniden numaralı silme iki `Err` vakası + bir "türetilmiş önek" vakası testi | 3/3 beklenen davranış |

C-2 satırı "C1 17/17, C2 **9/9**" olarak okunur. Karar kuralı değişmez; C-12/C-13 C-2'nin parçasıdır.

## Doğrulamaya ek (VERIFY.md V3'e)
- C2-09 negatif vakaları kendi ürettiğin bozuk `traces.jsonl` satırlarıyla dene (5 vaka), her biri ret.
- K-C0.1 sınır vakasını kendin kur (iki `LayerChanged`, ilkini sil, yeniden numarala) → `Err`; tek `LayerChanged` sil → açılır, `layer_source derived`.

## İstem değişikliği
Ana istemin ikinci satırına ekle: `docs/rounds/P1-C/ADDENDUM.md'yi de oku; C0 görevi ADDENDUM'daki işleri içerir.`
