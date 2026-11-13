# R3 — `check_deps`'in macOS düşüşü: inceleme

## Olay ve geçmiş
- K-D0.3 PR'ı #61, koşu `43241761070` (deneme 1), iş `test + lint + conformance (macos-14)` (koşucu `macos-15` imajı), `make lint` → `bash scripts/check_deps.sh`: `check_deps: VIOLATION: clients/ names /v1/attest_session, not an endpoint of docs/CLIENT_CONTRACT.md`, ardından `FAILED`. Başka hiçbir ileti yok (bash ya da grep hata satırı yok). 36 ms içinde betik bitti. Yeniden koşu (deneme 2) aynı ağaçta geçti.
- **Geçmiş tarama (bu turda, `gh` ile `ci.yml`'nin son 175 koşusunun her denemesi):** macOS `make lint` adımı 147 başarılı, 7 başarısız. 7'nin 4'ü gerçek hata (`check_frozen` ×2, `lint-py` ×1, Linux'ta test ×1'in macOS izi). **Üçü aynı imza:** tek bir uç için sahte "not an endpoint" — `43241761070` (`attest_session`, `check_deps.sh`), `41706382966` (2026-11-03, `demote`, `test_check_deps.sh`'in "unmodified workspace" kopyasında) ve `40247553758` (2026-10-26, `demote`, aynı yerde). Üçünde de uç `CLIENT_CONTRACT.md` §2'de yazılıydı; aynı ağaç başka koşularda geçti. Linux işlerinde bu imza hiç görülmedi.
- Sıklık: macOS'ta `make lint`'in ~150 koşusunda 3 (`check_deps` her koşuda iki kez çalışır: doğrudan ve `test_check_deps.sh`'in temiz kopyasında) → koşu başına ~%2.

## Betiğin ilgili yolu (`scripts/check_deps.sh` §3b)
`endpoints` bir kez hesaplanır (`sed | grep | sed | sort -u`, 15 satır, 126 bayt); her istemci yolu için `echo "$endpoints" | grep -qx "$u"` (betik `set -euo pipefail`). Yanlış "değil" yanıtı yalnız şu iki yoldan gelir: (i) `endpoints` o ucu içermiyordu, (ii) boru hattının çıkış durumu sıfır değildi — `pipefail` altında `echo`'nun (sol uç) **ya da** `grep`'in sıfır olmayan durumu.

## Hipotezler ve sınamalar
| Kod | Hipotez | Sınama | Sonuç |
|---|---|---|---|
| H1 | `echo` satır satır yazar; `grep -q` ilk eşleşmede çıkar, sonraki yazım SIGPIPE alır, `pipefail` boru hattını başarısız sayar (SIGPIPE'la ölen alt kabuk bash'te ileti yazmaz — imzayla uyumlu) | (a) Linux bash 5.2.37'de `strace -e write`: yerleşik `echo` 3 satırlık değeri **tek** `write` ile yazar (`printf` satır satır yazar); (b) macOS bash 3.2.57 ve Linux bash 5.2: ilk satırdan sonra çıkan okuyucuyla 2000'er deneme → `echo` sıfır olmayan 0/2000; (c) `echo "$endpoints" \| grep -qx` macOS'ta 9000 + yük altında (16 meşgul döngü) 20000 deneme, Linux'ta 8000 → yanlış "değil" 0 | Yerelde yeniden üretilmedi. Tek `write` yapılan 126 baytlık bir değerde okuyucu veriyi okumadan çıkamaz; H1 ancak CI'nin bash'i değeri birden çok `write` ile yazıyorsa mümkündür (CI imajında Homebrew bash 5.x; yerelde yok). **Açık.** |
| H2 | `endpoints` eksik okundu (dosyanın yarım okunması, `sed` aralığı) | Eksik okuma birden çok ucu düşürürdü; üç olayda da tek uç düştü ve listenin farklı yerlerinden (2. ve 6. satır). `CLIENT_CONTRACT.md` o koşularda değişmemişti | Olası değil |
| H3 | `make test`'in bıraktığı dosyalar ya da `npm ci`'nin yazdıkları `used`'ı değiştirdi | `used`'a fazladan yol eklemek yanlış "değil" üretmez: düşen uçlar sözleşmede vardı. Ayrıca iki olay `test_check_deps.sh`'in taze kopyasında (`make test`'ten bağımsız, `--exclude node_modules`) | Elendi |
| H4 | BSD/GNU araç farkı (`grep -x`, `sed -E`, yerel ayar) | Farklar belirlenimlidir: aynı ağaçta her koşuda aynı sonucu verirdi; macOS'ta 147 koşu geçti | Elendi |
| H5 | `grep`/`echo` bir hata ile çıktı (EINTR, kaynak yok) | Bash ve grep bu durumlarda stderr'e yazar; günlükte satır yok | Olası değil |
| H6 | `check_deps` yük altında (aynı işte `make test`'ten hemen sonra) | Yerelde 16 meşgul döngü altında `check_deps` 20/20 geçti | Yeniden üretilmedi |

**Depoda emsal:** `scripts/release_check.sh`'in başlığı aynı sınıfı anar — R7'nin `unzip | grep -q` boru hattı `pipefail` altında SIGPIPE ile rastgele kalıyordu (P1-E doğrulaması V7, P1-E.1 E1-4); orada çözüm listeyi önce yakalayıp sonra aramaktı ("No pipeline ends in an early-exiting reader under pipefail"). `check_deps.sh` §3b bu kuralın dışında kalmıştı. Orada yazan taraf çok yazımlıydı (`unzip`); burada 126 baytlık tek `write` beklenir, bu yüzden H1 sınıfı olası ama koşulu gözlenmedi.

**Kök neden bulunamadı.** Kanıt (tek uç, ileti yok, yalnız macOS) en çok H1 sınıfıyla (üyelik boru hattının bir ucunun sinyalle ölmesi) uyumlu, ama yerel bash'lerde bu yolun koşulu (birden çok `write`) gözlenmedi.

## Yapılan (güçlendirme ve tanı)
- `check_deps.sh` §3b: boru hattı "değil" derse yanıt **boru ve süreç olmadan** (bash `case` ile satır eşleşmesi) yeniden denetlenir.
  - İkisi ayrışırsa (uç listede): `check_deps: NOTE: the pipeline 'echo | grep -qx <u>' failed (exit statuses: <echo> <grep>) … not a violation` — ihlal sayılmaz ama satır görünür kalır; çıkış durumları hangi ucun düştüğünü söyler (`141 1`: `echo` SIGPIPE → H1 kanıtlanır; `0 1`: `grep`'in yanıtı; `0 2`: grep hatası).
  - İkisi de "değil" derse: ihlal (bugünkü gibi) ve iki tanı satırı — boru hattının çıkış durumları, okunan uç listesi (satır sayısı ve uçlar), o yolu anan dosyalar.
- Başarılı çıktı bayt eşit (`check_deps: OK (…)`); ihlal metni aynen.
- Yeni sınama `scripts/test_check_deps_membership.sh` (`make test-check-deps`'e eklendi): (1) `grep -qx attest_session`'ın yanlış "hayır"ı → geçer, NOTE satırı `exit statuses: 0 1` ile, ihlal yok; (2) sözleşme dışı yol → kalır, tanı satırlarıyla; (3) değiştirilmemiş kopya → geçer, NOTE yok. `test_check_deps.sh` aynen geçer.
- **Bulgu açık kalır** (FINDINGS): bir sonraki görülüşte NOTE satırının çıkış durumları kök nedeni ayırt eder. Aynı `echo … | grep -q` kalıbı `check_deps.sh`'in 0. adımında (çalışma alanı üyeleri; bu imzayla hiç düşmedi) ve `scripts/test_p1d1.sh`'te de var; bu turda yalnız düşen yol (§3b) güçlendirildi.

Sınama betikleri: `research/p2q3/` değil, oturumun geçici dizininde (yeniden üretim komutları yukarıda tabloda); sonuçları bu dosyada.
