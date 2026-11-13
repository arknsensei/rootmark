# P2-Q.4 — Yama Turu: Öldürülünce Kaçan Kaldırma, Bozulan Dizin ve Test Düzeneği, 0.3.4

Başlangıç: P2-Q.3 karar kuralına göre **kaldı** (Q3-2 ve Q3-10 lafzen; `reports/rounds/P2-Q.3/FINAL.md`). 0.3.3 yayımlı (`v0.3.3` `8f0bcb8`), turun PR'ı #66 birleşti (`ca760f8`). K-P2Q3.2 K-P2Q3-T1'i **(b)** ile kapatır: D-1'den önce bu kısa yama turu (0.3.4). P2-Q.3'ün ayrı doğrulaması koşulmaz; bu turun doğrulayıcısı iki turu birlikte doğrular. Sıra (K-P2Q3.3): P2-Q.4 → D-1 → P2-I → P2-G → P2-O → P2-U → sıra kararı → P2-C / P2-E / P2-H / P2-D.

Sorun:
- **PQ3-9 — taşımadan sonra SIGKILL ve yeniden başlatma: eski yolun kaldırması bazen alınmıyor (ürün hatası olabilir; yeni; önce kök neden).** PR #66'nın son commit'i `e8c462f`'in CI koşusu `43411497589`'un ilk denemesinde (iş `131838876300`, `macos-14`) `server/tests/watch_restart.rs` `a2_sigkill_move_canonical` 19/20 geçti. 13. tekrarda "timed out: n0.md's removal ingested" hatası verdi. Senaryo: not `sub/moved.md`'ye taşınır, kaldırma beklemesi içinde SIGKILL gelir, sunucu yeniden başlatılır. `reconciled()` döndükten sonra `n0.md`'nin blokları 90 s içinde kapanmadı. Bu P2-Q.3'ün kendi düzeltmesinin (D0F-1) yoludur; PQ3-7 (dizin) ya da PQ3-8 (port) değildir. Yeniden koşu (deneme 2) geçti. P2-Q.3'ün FINAL'inde yoktur (FINAL bir önceki commit'in CI'ını anıyor). Olası yollar (sınanacak, varsayılmayacak): macOS FSEvents'in başlatmaya yakın eski olayları yeni izleyiciye iletmesi (D0F-6) ile uzlaştırmanın kaldırması arasındaki etkileşim; taşımada eksik ve yeni dosyanın aynı toplu işte okunması; bekleme zamanlayıcısı.
- **PQ3-7 — kapanış sıkıştırmasının ortasındaki SIGKILL dizini bozabilir (ürün hatası; 0.3.2'de de).** `index/src/store.rs` `install` eski dizini `remove_dir_all` ile siler, sonra yenisini `rename` eder; iki adım atomik değil. Başlangıç (`server/src/tools.rs` `start_index`) yarım dizini `Current` sayar ve `search` 500 `internal` döner, `rootmark index build` elle koşulana kadar. Claude Code'un her oturum sonu dizisi (SIGINT → SIGTERM → ~0,5 s → SIGKILL) bu pencereye düşebilir.
- **PQ3-6 — erişim hatası "dosya yok" sayılıyor (ürün hatası; P2-Q.1'den beri).** Kaldırma sınaması `symlink_metadata(p).is_err()` kullanıyor: `EACCES` gibi `NotFound` dışındaki hatalar da yokluk sayılıyor. İzleyicinin `watch_removed`'ı, `reconcile_files` ve `remove_missing_under` aynı sınamayı kullanır.
- **PQ3-8 — test düzeneğinin port yarışı (test düzeneği).** `restartkit` boş portu `bind("127.0.0.1:0")` ile seçip bırakıyor; yük altında `Address already in use` (PQ2-3'te `watchkit`'te de aynı kalıp).
- **PQ3-4 — gerçek istemci dizisinin zorunlu SIGKILL yolu yalnız macOS'ta sınandı (test düzeneği).** Linux'ta sunucu SIGTERM'den sonra 0,5 s içinde kendisi kapandı (SIGKILL 0/40).

## İlkeler
- **Önce kök neden (PQ3-9).** Ürün kodu PQ3-9'un nedeni belirlenmeden değişmez. Nedeni gösteren kanıt (yük altında yeniden üretim, olay sırasının kaydı, ya da belirlenimli enjeksiyonla aynı sonucun üretimi) `progress/DESIGN.md`'ye yazılır. Neden bulunamazsa düzeltme uydurulmaz: hipotezler ve sonuçları yazılır, PARK + karar talebi.
- **Ürün sınırı.** Değişebilecekler: `server/src/serve.rs`, `server/src/tools.rs`, `server/src/bin/rootmark.rs`, `index/src/store.rs` (yalnız `install` ve dizinin bütünlük denetimi) ve yeni testler; test düzenekleri `server/tests/restartkit/`, `server/tests/watchkit/` (yalnız port seçimi ve SIGKILL yolu; iddialar gevşetilmez). Değişmeyenler: `core`, `bindings`, `clients/*/src`, spec §1–6, uyumluluk fixture'ları, `docs/CLIENT_CONTRACT.md` (yeni uç, alan, araç yok), dizinin dosya biçimi ve baytları (göç yok). Bunlardan biri gerekirse: PARK + karar talebi.
- **Dizin kurulumu ya eski ya yeni.** SIGKILL `install`'ın hangi adımına düşerse düşsün, bir sonraki başlangıç ya eksiksiz eski dizini ya eksiksiz yeni dizini görür, ya da dizini eksik görüp açık bir iletiyle yeniden kurar (`auto_rebuild` kuralı; sessiz değil). Kurulmuş dizinin baytları bugünküyle aynıdır (D-NOTES ve kademe 2).
- **Hata belirlenimli sınanır.** Yarışa bağlı pencereler (PQ3-7) yalnız rastgele zamanlı SIGKILL'le değil, test yapılandırmasıyla belirlenimli enjeksiyonla da sınanır: ör. `ROOTMARK_TEST_ABORT_AT=<adım>` gibi yalnız test ortamında okunan bir değişken, süreci adlı bir adımda `abort` ettirir (PQ1-4'ün `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` emsali). Ürün varsayılanı değişmez; değişkenin adı ve okunduğu yerler DESIGN'da.
- **Yalnız `NotFound` yokluktur (PQ3-6).** Öteki erişim hataları kaldırma sayılmaz; bir not satırı yazılır ve doküman canlı kalır.
- **Testler yük altında da geçer.** P2-Q.3'ün iki kalışı da `make test`'in tam yükünde görüldü. Bu yüzden etkilenen durumlar (`watch_restart.rs` A ve C kümeleri, yeni dizin testleri) yerelde yük altında (eşzamanlı `make test` ya da `stress`) çok tekrarla koşulur. CI'daki tekrar sayısı CI süresini uzatmamak için 20'de kalır (K-D0.3'ün amacı).
- **READY'nin kanıtı PR'ın son commit'idir.** READY ve FINAL, PR'ın o andaki son commit'inin CI koşusunu anar. READY'den sonra itilen her commit'in (HUMAN_STEP dahil) CI'ı okunur; bir kalış varsa HUMAN_STEP'ten önce FINDINGS ve FINAL'e yazılır (PQ3-9 böyle kaçtı).
- **Kayıt farklarının önceden adlandırılması (K-P2Q1.4):** sürüm dizgileri (0.3.4), E1-4/R14 (etiket), değişecekse CI yapılandırması.
- Sessiz başarısızlık yok; markdown'a kimlik/meta veri yazılmaz; kimliklere zaman ya da rastgelelik girmez.

## Kapsam
1. **R0 — Açılış, yeniden üretim, tasarım.**
   - `ROUND_START`; K-P2Q4.1.
   - PQ3-9'un yeniden üretimi 0.3.3 ile (yayımlanmış ikili ya da `v0.3.3`): `a2`/`a1` durumları macOS'ta yük altında ≥ 200 tekrar, Linux'ta ≥ 200. Olay sırası kaydı (izleyicinin aldığı olaylar, uzlaştırmanın kararları; içerik yok).
   - PQ3-7'nin belirlenimli üretimi (enjeksiyonun tasarımı DESIGN'da; 0.3.3'te enjeksiyon yoksa kod okuması ve P2-Q.3'ün `r5` kaydı yeterli kanıt).
   - `DESIGN.md`: PQ3-9'un kök nedeni ve düzeltmesi; `install`'ın yeni sırası ve başlangıç denetimi; enjeksiyon noktaları; PQ3-6; düzeneklerin portu ve SIGKILL yolu; ön-kayıtlı testler.
   - `TESTS.md` ve değişmesi beklenen kayıtların listesi.
2. **R1 — PQ3-9.** Düzeltme ve testler. A ve C kümeleri yerelde yük altında macOS ≥ 200 ve Linux ≥ 200 tekrar, 0 kalış. Yeni test PQ3-9'un nedenini (DESIGN'daki sıra) doğrudan sınar.
3. **R2 — PQ3-7.** Atomik kurulum ve başlangıçta eksik dizinin denetimi. Enjeksiyon her adımda (silmeden önce, silme ile yeniden adlandırma arasında, yeniden adlandırmadan sonra) ≥ 20 tekrar, iki platform: sonraki başlangıçta `search` 200 ve sonuçlar temiz kurulumla aynı; dizin baytları temiz `rootmark index build`'inkiyle eşit.
4. **R3 — PQ3-6, PQ3-8, PQ3-4.** Yalnız `NotFound` (üç yer) ve izinsiz dizin testi (`chmod 000`): bloklar kapanmaz, not satırı yazılır, izin geri gelince yeni olay yok. `restartkit` ve `watchkit` portu sunucunun stderr'inden okur. C1'in SIGKILL'i iki platformda zorunlu yoldan da sınanır (durdurmayı yavaşlatan test ayarı; Linux'ta `sigkill_sent` > 0).
5. **R4 — Belgeler, güvenlik ve 0.3.4.** Kılavuz (PQ3-7'nin belirtisi ve `index build` onarımı yalnız eski sürümler için not; PQ3-6'nın davranışı), `CHANGELOG.md` `## 0.3.4`, uyarılar (`progress/R4_security.md`). Kademe 2 bir tetikleme; Mac kümesi bir kez (izleyiciye dokunuldu; önce `fseventsd` sağlığı). `release/0.3.4`, `release_check` (R1–R20), insan adımları `HUMAN_STEP.md`'de. Yayımlanmış 0.3.4 ile: R1 ve R2'nin durumları iki platformda kısa tekrarla; gerçek Claude Code oturum sonu sınaması (≥ 3 oturum, D-NOTES kopyası); D-NOTES senaryosu Mac, Docker ve CI'da, 0.3.3 kayıtlarıyla bayt eşit.
6. **R5 — Rapor, gözden geçirici, hazır.** Kapılar (`make test` iki kez, ikisi de temiz); FINDINGS; gözden geçirici tek tur (başlık: "PQ3-9'un nedeni kanıtlı mı; kurulum her SIGKILL noktasında eski ya da yeni mi; enjeksiyon yalnız test ortamında mı; `NotFound` dışı hatalar sessiz mi; düzenek değişikliği iddiaları gevşetiyor mu; ürün sınırı"); FINAL; READY; tek PR.

## Önceden kayıtlı çıkış kriterleri
Senaryo ve test adımları için "koştu" ile "geçti" ayrı sayılır. Kriter, koşan **her** adımın geçmesini ister.

| Kod | Ölçüm | Eşik |
|---|---|---|
| Q4-1 | `DESIGN.md` ve `TESTS.md`, R1'in ilk kod commit'inden önce commit'li. PQ3-9'un kök nedeni kanıtla yazılı. PQ3-7 belirlenimli üretilmiş ya da üretim yöntemi gerekçeli. | önce; kanıtlı |
| Q4-2 | **PQ3-9:** A ve C kümeleri yerelde yük altında macOS ≥ 200 ve Linux ≥ 200 tekrar; nedeni hedefleyen yeni test; CI'da 20 tekrar, iki platform. | 0 kalış |
| Q4-3 | **PQ3-7:** her enjeksiyon noktasında ≥ 20 tekrar, iki platform: sonraki başlangıçta `search` 200, sonuçlar temiz kurulumla aynı, dizin baytları temiz kurulumla eşit; eksik dizin açık iletiyle yeniden kuruluyor (sessiz değil). | hepsi |
| Q4-4 | **PQ3-6:** üç yerde yalnız `NotFound` kaldırma; izinsiz dizin testi geçer (bloklar canlı, not satırı var, izin dönünce yeni olay 0). | hepsi |
| Q4-5 | **PQ3-8 ve PQ3-4:** `restartkit` ve `watchkit` portu sunucudan okur; C1'in zorunlu SIGKILL yolu iki platformda sınanmış (`sigkill_sent` > 0, her platformda). İddialar P2-Q.3'tekiyle aynı ya da daha sıkı. | hepsi |
| Q4-6 | **Sözleşme ve geriye uyum:** `docs/CLIENT_CONTRACT.md` değişmedi; MCP `tools/list` 0.3.3 ile bayt eşit; `contract.rs`, `mcp_schemas.rs`, `mcp_compat.rs` ve P2-Q.3'ün testleri aynen geçer; mevcut vault'lar göçsüz açılır; `make conformance` 39/39. | eşit; geçti; 39/39 |
| Q4-7 | Kademe 2 bir tetikleme: zincir çıkışları 0, D farkı 0, sayılan FAIL 0, `portable` 0; değişen kayıtlar yalnız R0'da adlı olanlar. Mac kümesi bir kez, kayıtlı, P2-V.2 okumasıyla. | 0; adsız 0; kayıtlı |
| Q4-8 | **Ürün sınırı:** `git diff ROUND_START..HEAD -- core bindings spec clients/*/src` boş (sürüm dizgileri hariç); `index/` farkı yalnız `store.rs`'in kurulum ve denetimi; `server/` farkı yalnız İlkeler'deki dosyalar ve yeni testler; dondurulmuş dosyalara FREEZE_EXCEPTIONS'sız dokunulmadı. D-NOTES 0.3.4 Mac ↔ Docker ↔ CI ↔ 0.3.3 bayt eşit. | hepsi |
| Q4-9 | Belgeler ve güvenlik: kılavuz ve CHANGELOG 0.3.4; açık Dependabot ve CodeQL uyarısı 0 ya da her biri gerekçeli. | hepsi |
| Q4-10 | 0.3.4: `release_check` 21/21; `release.yml` zorunlu iki hedef `ok`; `v0.3.4` yayımlı (insan); yayımlanmış 0.3.4 ile R4'ün durumları ve ≥ 3 gerçek Claude Code oturum sonu sınaması geçti. | hepsi |
| Q4-11 | Süreç: R5'te `make test` iki kez temiz, `make lint`, `make conformance`, `make verify FAST=1` temiz; `check_frozen`, `check_deps`, `check_dep_age` temiz; **PR'ın son commit'inin** CI'ı yeşil (READY'den sonraki commit'ler dahil); gözden geçirici kabul. | hepsi |

## Karar kuralı
- **Hepsi geçti** → PQ3-9, PQ3-7, PQ3-6, PQ3-8, PQ3-4 kapandı; 0.3.4 yayımlı. Bağımsız doğrulama birleşmeden sonra (P2-Q.3 ve P2-Q.4 birlikte, K-P2Q3.2). Sıradaki D-1.
- **Q4-1 kaldı (PQ3-9'un nedeni bulunamadı) ya da düzeltme spec'in tanımlamadığı bir olay/alan, dizin biçimi değişikliği ya da göç gerektiriyor** → PARK; karar talebi. 0.3.4 yayımlanmaz.
- **Q4-2, Q4-3 ya da Q4-6 kaldı** → hata kapanmadı ya da sözleşme bozuldu; 0.3.4 o düzeltmeyle yayımlanmaz; tur kaldı, karar talebi (D-1 bu hatayla mı açılır).
- **Q4-4, Q4-5, Q4-7, Q4-8, Q4-9, Q4-10 ya da Q4-11 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Yeni MCP aracı, HTTP ucu ya da sözleşme alanı; dizinin dosya biçimi değişikliği ya da göçü.
- `rootmark init`, gösterim, "git ile fark" sayfası (D-1 ve P2-I; K-P2Q3.3); doğrudan yazma kapısı (P2-G).
- Dağıtım işleri (D-1); marka (H2, en sonda).
- PQ3-1 (`check_deps` macOS) ayrı iş değildir; yeniden görülürse tanı çıktısıyla FINDINGS'e yazılır.
- PQ3-3 ve PQ3-5 (belge) D-1'in kılavuzuna gider.
- Kişisel içerikle ölçüm.
