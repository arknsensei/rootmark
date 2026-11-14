# R0 — PQ3-9'un yük altında yeniden üretimi (0.3.3)

**Ağaç:** `v0.3.3` (`8f0bcb8`; ürün kodu `ROUND_START` ile aynı) + tanı yaması (`reports/rounds/P2-Q.4/r0/diag_v0.3.3.patch`; yalnız scratch çalışma ağacında, ürün değil): `ROOTMARK_DIAG_EVENTS=1` ile izleyicinin aldığı her ham olay (tür, dosya adı; içerik yok), toplu işlerin hedefleri, uzlaştırmanın kararları (`changed`, `missing`), bekleme süresi dolan kaldırmalar ve her alım/kaldırma denemesi stderr'e zaman damgasıyla (yalnız süreç içi göreli ms); düzenek her tekrarda öldürmeden sonra sıcak katmanın hizasını (`records`/`embedders`/`snapshots` satır sayısı, `vectors.f32` baytı) kaydeder ve kalan tekrarda iki sunucunun bütün stderr'ini yazar. `ROOTMARK_DIAG_ABORT_HOT=<k>` (yalnız belirlenimli koşu): `index_head`'in k. ekinden sonra `abort`.

**Makine ve yük:** Apple M4 Max (16 çekirdek, 128 GB), macOS 27.0.1. Yük: eşzamanlı `cargo test --workspace --release` döngüsü (turun ağacı, ayrı hedef dizini) **ve** aynı makinede eşzamanlı Linux koşusu (Docker, kendi `cargo test --workspace --release` döngüsüyle). Linux: Docker `rust:1.92.0-trixie`, `aarch64`, 16 CPU, kök kullanıcı (bu testlerde izin sınaması yok). Tekrar 200 (`ROOTMARK_RESTART_REPS=200`), `cargo test --release -p rootmark-server --test watch_restart -- a1_ a2_ c1_`. Başlangıç 2026-11-13 20:34 (+03).

## Sonuç (koştu / geçti)

| Durum | macOS | Linux |
|---|---|---|
| A1 `canonical` (silme + SIGKILL) | 200 / 200 | 200 / 200 |
| A1 `unverified` | 200 / 200 | 200 / 200 |
| A2 `canonical` (taşıma + SIGKILL) | 200 / **198** | 200 / **197** |
| A2 `unverified` | 200 / **199** | 200 / **199** |
| C1 `canonical` (SIGINT → SIGTERM → 0,5 s → SIGKILL) | 200 / **195** (SIGKILL 41) | 200 / **198** (SIGKILL 47) |
| C1 `unverified` | 200 / **191** (SIGKILL 35) | 200 / **199** (SIGKILL 47) |

Kayıt: `reports/rounds/P2-Q.4/r0/repro_0.3.3_{macos,linux}.jsonl` (tekrar başına sonuç; A'da `hot_after_kill`, C1'de `sigkill_sent`); A2'nin kalan tekrarlarının iki sunucu günlüğü `r0/fail_logs_{macos,linux}/`.

## Kalışların sınıflandırması (22 kalışın hepsi)
1. **Hizasız sıcak katman → kaldırma düşer** (PQ3-9'un CI'daki iletisi "timed out: n0.md's removal ingested"): A2 macOS 2 (`canonical` 34, `unverified` 179), Linux 3 (`canonical` 6 ve 190, `unverified` 2). Beşinin de öldürmeden sonraki sıcak katmanı hizasız: `snapshots.jsonl` yok, `records.jsonl` 1 satır (`embedders` 1/yok, `vectors.f32` 1536 B/yok). Beş günlüğün beşinde ikinci sunucu: `reconcile end changed=0 missing=["n0.md"]` → 1,5 s sonra `removal attempt n0.md` → `watch: n0.md: removal: error: hot tier invalid (rebuild with \`rootmark index build\`): read …/hot/snapshots.jsonl: No such file or directory`; sonra hiçbir deneme yok. **Karşı sınama:** A2'nin geçen 793 tekrarının hepsinde öldürmeden sonraki sıcak katman hizalıdır (`records 1, embedders 1, vectors 1536, snapshots 1`); hizasız 5 tekrarın 5'i kaldı.
2. **Günlük dizinden önde → `search` 500** (`snapshot … of sub/moved.md is not indexed`): A2 `canonical` macOS 152, Linux 70 — SIGKILL `commit_plan` ile ilk ek arasına (gömme süresince) düştü.
3. **C1'de kapanış sırasında SIGKILL → `search` 500:** macOS 14, Linux 3 — iletiler: `hot tier invalid` (macOS 1, Linux 2), `snapshot … of n0.md is not indexed` (macOS 1, Linux 1; günlük önde), `Failed to open file for read … lexical/…store … does not exist` ve `open lexical index … Directory does not exist` (macOS 9; **PQ3-7**: kapanış sıkıştırmasının `install`'ı `remove_dir_all`'ın ortasında öldü), `read … No such file or directory` (macOS 3; yarı silinmiş dizinin bir dosyası).

Port yarışı (PQ3-8) bu koşuda görülmedi (0/2 400).

## Belirlenimli üretim
`ROOTMARK_DIAG_ABORT_HOT=1,2,3` (A2 adımları, `canonical`, 3'er tekrar): **9/9 kaldı**, her biri sınıf 1'in iletisiyle; ikinci sunucunun günlükleri `r0/abort_hot_macos/`. Bu, doğal SIGKILL'de görülen olay sırasının aynısıdır.

## Sınanan öteki yollar (GOAL)
- **FSEvents'in eski olayları (D0F-6):** kalan günlüklerde ikinci sunucu ilk sunucunun `vectors.f32`/`records.jsonl` olaylarını alıyor (macOS), ama hepsi `.rootmark/` altında: hedef üretmiyor (`batch targets={}`). Kaldırmayı ne siliyor ne erteliyor.
- **Taşımanın iki yolunun aynı toplu işte okunması:** ilk sunucuda `batch targets={Doc("n0.md"), Doc("sub/moved.md")}` — beklenen; ikinci sunucuda uzlaştırma eski yolu eksik bulur (`missing=["n0.md"]`).
- **Bekleme zamanlayıcısı:** kaldırma her günlükte zamanında (≈1 500 ms) denenir; düşmesinin nedeni denemenin hatasıdır.

**Kök neden:** SIGKILL'in dizin yazımının ortasına (sıcak katmanın dört eki arasına, günlük ile ek arasına ya da kurulumun `remove_dir_all`'ına) düşmesi dizini günlükle tutarsız bırakır; 0.3.3'ün başlangıcı bunu görmez (`start_index` yalnız manifest'i okur), uzlaştırma değişmemiş dosyalar için dizini açmaz, ilk dizin açılışı (kaldırmanın `Store::open`'ı ya da `search`) hata verir ve kaldırma bir daha denenmez. PQ3-9 ve PQ3-7 aynı sınıfın iki yüzüdür. Düzeltme: DESIGN §2–§3.

## Düzeltme eki (R5; gözden geçirici N1)
"22 kalış" yanlış toplam: kayıtlarda macOS 17 + Linux 7 = **24** (A2 7, C1 17). Sınıflar: (1) 5, (2) A2 2 + C1 2, (3) C1 15. Sonuç değişmez; ayrıntı DESIGN'ın düzeltme ekinde. R1.md'deki "0.3.3'te aynı koşu 22 kalış" da 24 okunur.
