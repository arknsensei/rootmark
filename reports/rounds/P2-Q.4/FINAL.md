# P2-Q.4 — Final raporu: öldürülünce kaçan kaldırma, atomik dizin ve 0.3.4

**Karar kuralının dalı: tur kaldı (Q4-7 lafzen; K-P2Q4.2) → karar talebi K-P2Q4-T2.**

- Bu turun düzeltmeleri kapandı: PQ3-9, PQ3-7, PQ3-6, PQ3-8 ve PQ3-4.
- 0.3.4 yayımlı. Yayımlanmış 0.3.4 iki platformda kalışsız geçti.
- Kalış, kademe 2'nin tek tetiklemesinin çiftinden geldi: DESIGN §7'de adı olmayan 4 alan farkı (PQ4-5).
- Bu farkın kaynağı, CI Linux'ta SIGTERM'in başlangıç uzlaştırmasını beklemesi (PQ4-6; 0.3.3'te de aynı). Bu turun gerilemesi gösterilemedi.
- Karar kuralının Q4-7 dalı yayımı durdurmaz (K-P2Q4.2).

**Koşu kimlikleri**
- Başlangıç ve dal: `ROUND_START` `b6f7193`; dal `p2-q.4/round`.
- Kademe 2: sayılan tetikleme `43485696909` (`b65ef1d`). İlk tetikleme `43480344947` sonucu okunmadan iptal edildi, sayılmadı.
- Mac kümesi: RUN `20261113T191922Z-789c400`.
- Yayım:
  - Yayım PR'ı #69; CI `43511347182` yeşil. Önceki commit'in CI'ı `43500764482` kaldı (PQ4-4).
  - #69 → `main` `81a923d`; `v0.3.4` etiketi orada.
  - `release.yml` `43527847034`.
- `pilot.yml`: `43574022026`.
- Turun PR'ının CI koşusu: `READY.md`'de.

## Ne yapıldı
- **PQ3-9'un kök nedeni (R0; DESIGN §1).** 0.3.3 ile yük altında macOS ve Linux'ta 200'er tekrar yeniden üretildi.
  - Mekanizma: SIGKILL dizin yazımının ortasına düşer: sıcak katmanın dört eki arasına, günlük ile ek arasına ya da kurulumun `remove_dir_all`'ına. Dizin günlükle tutarsız kalır.
  - Başlangıç yalnız manifest'i okuyup dizini `Current` sayar. Bekleyen kaldırmanın `Store::open`'ı düşer ve kaldırma bir daha denenmez.
  - Kanıt: hizasız sıcak katmanlı 5/5 tekrar kaldı, hizalı 793 tekrarın 0'ı; belirlenimli abort 9/9.
  - FSEvents, aynı toplu iş ve zamanlayıcı kayıtla elendi.
- **Başlangıç denetimi (R1; `index/src/store.rs` `incomplete`, `server/src/tools.rs` `start_index`).**
  - Her başlangıçta dizin bütün olarak açılıyor mu, günlüğün her başı dizinde mi diye denetlenir; model yüklenmez.
  - Eksik dizin `[index] auto_rebuild` ile açık bir satırla yeniden kurulur (`index incomplete (…; an interrupted write): rebuilt`).
  - `auto_rebuild = false`'ta satır yazılır, dizine dokunulmaz. Okunamayan manifest K-E1.2'deki gibi kurulmaz.
- **Atomik kurulum (R2; `store::install`, `recover_install`).**
  - Kurulum iki atomik `rename`'le yapılır (`index.old`). Kesilmiş kurulumun artıkları başlangıçta toplanır.
  - Test enjeksiyonu `ROOTMARK_TEST_ABORT_AT` yalnız `install`'ın üç adımında okunur. Tanınmayan değer hatadır.
- **Yalnız `NotFound` yokluktur (R3; PQ3-6).** İzleyicinin toplu işi, `watch_removed`, `reconcile_files` ve `remove_missing_under` dört yerde yalnız `NotFound`'u kaldırma sayar. Öteki hatalar not satırı yazar.
  - Linux'ta okunamayan alt dizinle `serve --watch` başlamıyordu (PQ4-3; 0.3.3'te de). Artık okunabilen dizinler tek tek kaydedilir.
- **Düzenekler (R3).**
  - `restartkit` ve `watchkit` portu sunucudan okur (port 0 + stderr; PQ3-8).
  - C1'in zorunlu SIGKILL yolu iki platformda sınanır: `ROOTMARK_WATCH_TEST_STOP_DELAY_MS` ile C2, PQ3-4.
- **PQ4-4 (R4).** Yayım PR'ının CI'ında yakalanan, bu turun R3'ünün getirdiği gerileme, yayımdan önce düzeltildi: `missing_files` sembolik bağlantılı ya da vault dışı yolda bütün `ingest-dir`'i düşürüyordu.
- **Belgeler:** kılavuz §3, §10 ve Known issues; CHANGELOG `## 0.3.4`; `R4_security.md`.
- **Yeni testler:**
  - `hot_tear_restart.rs` (H0–H4), `index_install_kill.rs` (I1–I6), `watch_noaccess.rs` (N1–N4), `watch_restart_forced.rs` (C2).
  - Her biri 20 tekrar (`ROOTMARK_RESTART_REPS`).

## Kriterler
Senaryo ve test adımlarında "koştu" ile "geçti" ayrı sayılır.

| Kod | Sonuç | Kanıt |
|---|---|---|
| Q4-1 | **geçti** | DESIGN ve TESTS R1'in ilk kod commit'inden önce commit'li (R0). PQ3-9'un nedeni kanıtla yazılı (`R0_repro.md`, `r0/`; R0'ın kalış toplamı 24, düzeltme eki PQ4-8). PQ3-7 0.3.3'te doğal olarak üretildi (C1'de 12 tekrar) ve durum kurulumuyla gösterildi (I4 0.3.3'te 0/2); belirlenimli enjeksiyon (DESIGN §3.3) 0.3.4'ün yeni kurulumunu her adımda sınar. |
| Q4-2 | **geçti** | A ve C kümeleri yük altında koştu 2 400, geçti 2 400: macOS 1 200 ve Linux 1 200, durum başına 200. Nedeni hedefleyen H0–H4 düzeltmeden önce 0/2, sonra 20/20 iki platform. CI'da 20 tekrar iki platform: #69 `43511347182` ve turun PR'ının CI'ı (`READY.md`). |
| Q4-3 | **geçti** | `install:before_swap`, `install:swapped` ve `install:installed`; `index build` ve kapanış sıkıştırması. Her nokta 20/20, macOS ve Linux. `search` 200; sonuçlar ve dizin baytları temiz `rootmark index build`'le eşit. Eksik dizin açık iletiyle yeniden kuruldu (I4); `auto_rebuild = false`'ta ileti verildi, dizine dokunulmadı (I5). |
| Q4-4 | **geçti** | N1–N4 20/20 macOS ve Linux (kök olmayan); 0.3.3'te N1 0/2. Bloklar canlı kaldı, not satırı yazıldı; izin geri gelince yeni olay 0. PQ4-1, ön-kaydın N4 öngörüsünün tutmadığını yazar; sessiz değil. |
| Q4-5 | **geçti** | Düzenekler portu sunucudan okur. C2 `sigkill_sent` 400/400 macOS ve 400/400 Linux. İddialar P2-Q.3'tekiyle aynı; C2 ayrıca "kaldırma SIGKILL'den önce alınmadı"yı ister. |
| Q4-6 | **geçti** | `docs/CLIENT_CONTRACT.md` değişmedi. MCP `tools/list` yayımlanmış 0.3.3 ile bayt eşit (11 araç; `r4f/tools_list.json`). `contract.rs`, `mcp_schemas.rs`, `mcp_compat.rs` ve P2-Q.3'ün testleri `make test`'te aynen geçti. Mevcut vault'lar göçsüz açılır (D-NOTES `events_hash` 0.3.3 ile aynı). `make conformance` 39/39. |
| Q4-7 | **kaldı** | Kademe 2 `43485696909`: zincir çıkışları 0; sayılan FAIL 0/0; `portable` A ve B 0; bayt farkı 0. **Alan farkı 4** (bu dört alan aynı zamanda çiftin D farkıdır: `tier2 pair` `pass: false`; Q4-7'nin "D farkı 0" ve "adsız 0" alt ölçüleri ikisi de kaldı): `p2a_checks.json` ve `p2_0_2_checks.json`, `regress › excluded_differences › … › leaves`. Bunlar DESIGN §7'de adlı değil (PQ4-5); K-P2Q4.2'ye göre lafzen kaldı. Kaynağı Z sınıfındaki `p1d/sigterm.json`: A'da 7/10, B'de 1/10 deneme 15 s'de çıkmadı (PQ4-6). Aynı makinede 0.3.3 ve 0.3.4 aynı davranır (`r4/sigterm_diag.json`). Yeniden tetiklenmedi (GOAL; K-P2Q4.2). Mac kümesi bir kez koştu, kayıtlı, hüküm `pass` (`mac_set/`). |
| Q4-8 | **geçti** | `core`, `bindings`, `spec` ve `clients/*/src` farkı yalnız iki `Cargo.toml` sürüm satırı. `index/` farkı yalnız `store.rs`'in kurulum ve denetimi. `server/` farkı yalnız `serve.rs`, `tools.rs`, `bin/rootmark.rs`, düzenekler ve yeni testler. `ROUND_START..HEAD`'de görünen `.github/workflows/ci.yml` farkı turun değil, `main`'e giren PR #68'in (K-P2Q3.4; yeniden temellendirmeyle geldi). `check_frozen` OK. D-NOTES 0.3.4 Mac ↔ Docker ↔ CI ↔ 0.3.3 bayt eşit; en büyük ULP 0 (`pilot/compare.json`). |
| Q4-9 | **geçti** | Kılavuz ve CHANGELOG 0.3.4. Dependabot 0, CodeQL 0, `npm audit` 0/0 (`R4_security.md`). |
| Q4-10 | **geçti** | `release_check` 21/21. `release.yml` `43527847034`: iki zorunlu hedef `ok`, 39/39. `v0.3.4` yayımlı (insan). Yayımlanmış 0.3.4 ile P1/P2: macOS ve Docker `linux/amd64`, 32 durum × 5 tekrar, kalış 0 (`r4f/`). Gerçek Claude Code 2.1.325 ile 3/3 geçti (`R4_claude_code.md`). |
| Q4-11 | **geçti** | R5 kapıları HEAD `02afe7d`'de koştu: `make test` 2/2 temiz, `make lint` 0, `make conformance` 39/39, `make verify FAST=1` 0 (hızlı çift 18 550 alan eşit). `check_frozen`, `check_deps` ve `check_dep_age` OK. Gözden geçirici tek tur: **KABUL** (`R5.review.md`; engelleyici yok, notlar FINDINGS PQ4-7..PQ4-10). PR'ın son commit'inin CI'ı `READY.md`'de. |

## Bulgular (`FINDINGS.md`)
- **Kapandı (0.3.4):** PQ3-9, PQ3-7, PQ3-6 ve PQ4-3.
- **Kapandı (test düzeneği):** PQ3-8 ve PQ3-4.
- **Kapandı (yayımdan önce):** PQ4-4.
- **Bilgi:** PQ4-1 (ön-kaydın N4 öngörüsü), PQ4-2 (E-14 `make test`'te atlanır; yeni değil), PQ4-8 (R0'ın kalış toplamı 24; düzeltme eki).
- **Gözden geçiriciden (R5.review N1–N9; hüküm KABUL):** PQ4-7 (`watchkit` tarifinin atomik olmayan yeniden yazımı; düzenek, sonraki tur), PQ4-9 (yayımlanmış ikili `ROOTMARK_MODELS` ister; D-1 kılavuzu), PQ4-10 (küçük not satırı ve ileti eksikleri; sonraki tur).
- **Sonraki tura (K-P2Q4.2):**
  - PQ4-5: envanter açığı; DEV_PROCESS §4 istisna prosedürüyle.
  - PQ4-6: ürün; durdurma başlangıç uzlaştırmasını kesebilmeli.

## Karar talebi
**K-P2Q4-T2** (`docs/DECISIONS.md`): sıradaki tur ve PQ4-5/PQ4-6'nın yeri. Öneri (a): D-1 açılır. D-1'in R0'ı PQ4-5'i §4 ile envantere işler; PQ4-6 D-1'in kapsamına sınırlı ürün maddesi olarak girer. P2-Q.3 ve P2-Q.4'ün bağımsız doğrulaması birleşmeden sonra yapılır (K-P2Q3.2).

## Önceden adlandırılan kayıt farkları
DESIGN §7'deki beklentiler: sürüm dizgileri 0.3.4, E1-4/R14 (etiket); CI yapılandırması bu turda değişmez (`ci.yml` farkı PR #68'den, `main`'de; turun commit'lerinde CI dosyası yok); D kayıtlarında fark beklenmez.

Gözlenenler:
- D-NOTES'ta sürüm dizgisi bile fark doğurmadı.
- Kademe 2'nin dört `leaves` alanı adsızdı: Q4-7 kaldı.
