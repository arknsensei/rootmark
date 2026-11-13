# P2-Q.3 — Final raporu: kapalıyken ve öldürülünce kaybolan değişiklikler, 0.3.3

**Karar kuralının dalı: tur kaldı (Q3-2 ve Q3-10 lafzen) → karar talebi K-P2Q3-T1.** D0F-1/PQ2-1 ve D0F-5'in düzeltmesi yayımlanmış 0.3.3 ile iki platformda geçti; 0.3.3 yayımlı. Kalış, R5'in kapı koşusunda yeni testin açığa çıkardığı, bu turdan önce de var olan bir dizin hatasından (PQ3-7) ve bir test düzeneği yarışından (PQ3-8) geldi.

- `ROUND_START` `325d8b9` (PR #64'ün birleşmiş `main`'i).
- Dal `p2-q.3/round`.
- Yayım PR'ı #65 → `main` `8f0bcb8`, `v0.3.3`.
- Kademe 2 `43319749692`.
- Mac kümesi RUN `20261112T222913Z-be3bdd5`.
- `pilot.yml` `43393345943`.
- `release.yml` `43392096415`.
- Turun PR'ının CI koşusu: `READY.md`'de.

## Ne yapıldı
- **Başlangıç uzlaştırması** (`server/src/serve.rs`, `server/src/tools.rs`; DESIGN §2–§3):
  - `serve --watch` önce izleyiciyi kaydeder, sonra vault'un `*.md` dosyalarını günlüğün canlı dokümanlarıyla uzlaştırır. Bunu izleyicinin ilk toplu işi olarak, bugünkü `ingest_external`/`watch_removed` yoluyla yapar.
  - Değişen ve yeni dosyalar yolun bayt sırasıyla alınır. Dosyası olmayan dokümanlar kaldırma beklemesinden sonra, yine bayt sırasıyla kaldırılır.
  - Değişiklik yokken olay yazılmaz ve sıcak katmana dokunulmaz.
  - Spec §7.1'in dış değişiklik kuralı aynen uygulanır: başlangıçta önceki bir tasdik eşleşmez; geç tasdik §7.1'in kuralıdır.
  - SIGKILL'le kaybolan bekleyen kaldırma bir sonraki başlangıçta eksik dosya olarak alınır.
- **`ingest-dir` silmeleri** (`server/src/bin/rootmark.rs`, `Kernel::remove_missing_under`; DESIGN §4):
  - Kök vault'un içindeyse, kökün altında dosyası olmayan canlı dokümanlar aynı kuralla kapatılır (kaynak `rootmark-ingest-dir`).
  - Kökün dışına ve vault dışındaki köklere dokunulmaz.
- **`check_deps`** (R3): sahte "not an endpoint" macOS CI'da üç kez görülmüş (bir değil). Yeniden üretilmedi; boru hattının yanıtı borusuz yeniden denetlenir, ayrışma NOTE satırıyla ve çıkış durumlarıyla yazılır. Yeni sınama `test_check_deps_membership.sh`.
- **Belgeler:** kılavuz §2.1, §3 ve Known issues (D0F-7; gözden geçiricinin B2'si dahil); CHANGELOG `## 0.3.3`.
- **0.3.3 yayımı** (A5: kullanıcı bu oturumda yayımın insan adımlarını ajana verdi).
- **Yeni testler:**
  - `server/tests/watch_restart.rs`: 16 durum, her biri 20 tekrar.
  - `server/tests/ingest_dir_remove.rs`: 9 test.
  - `server/tests/restartkit/` (yardımcı modül).

## Kriterler ("koştu" / "geçti" ayrı)
| Kod | Sonuç | Kanıt |
|---|---|---|
| Q3-1 | **geçti** | `DESIGN.md` ve `TESTS.md` R0 commit'inde (`main`'de `caadd0c`; dalda `d9b12ca`), R1'in kodundan (`a324712`) önce. 0.3.2'de iki platformda yeniden üretim (her durum 3 tekrar): SIGKILL kalıcı 3/3; kapalıyken düzenle/sil/ekle 0/3; `ingest-dir` silme 0/3 (`r0/`, `progress/R0_repro.md`). |
| Q3-2 | **kaldı (lafzen)** | Ayrıntı tablonun altında. |
| Q3-3 | **geçti** | I1–I9 macOS ve Linux 9/9 (silme alınır; alt dizin kökünde dışarısı aynen; değişiklik yokken yeni olay 0; vault dışı kök kaldırmaz). Yayımlanmış 0.3.3'te `ingest-dir` silme 3/3, iki platform. Eski ağaçta kaldırma testleri kaldı (`r2/legacy_macos.txt`). |
| Q3-4 | **geçti** | `docs/CLIENT_CONTRACT.md` değişmedi. `contract.rs`, `mcp_schemas.rs` ve `mcp_compat.rs` aynen geçti; `mcp_compat.rs` MCP `tools/list`'i 0.3.1/0.3.2'nin sabit dosyasıyla bayt eşit denetler. Yeni olay türü, alan, uç ya da araç yok; mevcut vault'lar göçsüz açılır (D-NOTES `events_hash` aynı). `make conformance` 39/39. |
| Q3-5 | **geçti** | Ayrıntı tablonun altında. |
| Q3-6 | **geçti** | `git diff 325d8b9..HEAD -- core index bindings spec clients/*/src`: yalnız sürüm dizgileri (dört `Cargo.toml`). `server/` farkı yalnız `serve.rs`, `tools.rs`, `bin/rootmark.rs` (A1: GOAL'ün `main.rs`'i) ve yeni testler. Dondurulmuş dosyalara dokunulmadı (`check_frozen` OK). D-NOTES 0.3.3 Mac ↔ Docker ↔ CI ↔ 0.3.2 kayıtları bayt eşit. |
| Q3-7 | **geçti** | Kök neden bulunamadı. Hipotezler, sonuçları ve güçlendirilmiş tanı çıktısı yazılı (`progress/R3_check_deps.md`). `test_check_deps.sh` ve yeni sınama geçer (yerel, CI macOS ve Linux). |
| Q3-8 | **geçti** | Kılavuz güncel: Known issues'tan PQ2-1 çıktı; uzlaştırma ve `ingest-dir` yazılı. CHANGELOG 0.3.3. Açık Dependabot 0, CodeQL 0 (`progress/R4_security.md`). |
| Q3-9 | **geçti** | `release_check` 21/21 (R15 gerçek Obsidian 11/11). `release.yml` `43392096415`: zorunlu iki hedef `ok`, 39/39. `v0.3.3` yayımlı (12 dosya; `SHA256SUMS.local` yalnız yerel dosyalarla). Yayımlanmış 0.3.3 ile R1 (a)/(c)/(d) durumları iki platformda geçti; gerçek Claude Code 2.1.323 oturum sonu sınaması geçti (`progress/R4_claude_code.md`). |
| Q3-10 | **kaldı (lafzen)** | Ayrıntı tablonun altında. |

**Q3-2 ayrıntısı.**
- Koşan ve geçen:
  - `watch_restart.rs` 16 durum × 20 tekrar: macOS 20/20, Linux (Docker `aarch64`) 20/20 (`r1/watch_restart_{macos,linux}.jsonl`).
  - PR #65'in CI'ı: macOS 593 s ve `ubuntu-latest` x86_64 443 s, 16/16.
  - E1/E2'de yeni olay 0 ve günlük bayt eşit; F1/F2'de katman ve aktör §7.1'deki gibi.
  - `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs` ve `serve_watch.rs` aynen geçti.
- Kalış: R5'in ilk `make test` koşusunda (HEAD `765f89d`, macOS, tam paket yükü) C1 `canonical` 19/20 ve C1 `unverified` 19/20 (`r5/make_test_run1_failures.txt`).
  - `canonical`: port yarışı (PQ3-8).
  - `unverified`: yeniden başlatmadan sonra `search` 500, dizin dosyası eksik (PQ3-7). Bu turun değiştirmediği `index` yolu.
  - İkinci koşu temiz.
- Kriter koşan her adımın geçmesini ister; bu yüzden **kaldı**.

**Q3-5 ayrıntısı.**
- Kademe 2 tek tetikleme `43319749692`: iki zincir ve karşılaştırma işi başarılı; D farkı 0 (788 890 alan), sayılan FAIL 0/0, `portable` bulgu 0.
- Commit'ler arası (`42982215036` ↔ `43319749692`, A zincirleri): bayt farkı 0. 22 alan farkının hepsi DESIGN §8.2'deki E1-4/R14 ile commit'e bağlı iki alan. **Adsız fark 0.**
- Mac kümesi bir kez: hüküm `pass` (P2-V.2 okuması); değişen değerler adıyla (`progress/R4.md` (d)).

**Q3-10 ayrıntısı.**
- `make lint`, `make conformance` (39/39), `make verify FAST=1` (hızlı çift eşit, 18 550 alan, 1290 s), `check_frozen`, `check_deps` ve `check_dep_age` temiz.
- PR #65'te kademe 1 yeşil; turun PR'ının CI'ı `READY.md`'de.
- Gözden geçirici: KOŞULLU KABUL → koşullar karşılandı → KABUL.
- **`make test`: koştu 2, geçti 1** (1. koşu yukarıdaki iki kalış, 2. koşu temiz).

## Karar kuralı (GOAL'den türetilmiş)
- "Hepsi geçti" dalı **tutmuyor**: Q3-2 ve Q3-10 lafzen kaldı.
- Q3-2'nin dalı: "hata kapanmadı ya da sözleşme bozuldu; 0.3.3 o düzeltmeyle yayımlanmaz; tur kaldı, karar talebi (D-1 bu hatayla mı açılır)".
- Q3-10'un dalı: "tur kaldı; karar talebi".
- Sözleşme bozulmadı (Q3-4 geçti). Turun hedeflediği hatalar (D0F-1, D0F-5) yayımlanmış ikiliyle kapandı. Kalışların nedeni, yeni testin açığa çıkardığı ve 0.3.2'de de var olan PQ3-7 ile düzenek yarışı PQ3-8.
- **0.3.3 kalıştan önce yayımlandı.** GOAL'ün sırası R4 (e) yayımı R5'in kapılarından önceye koyuyor. 0.3.3 0.3.2'den kesin olarak daha iyi; PQ3-7 iki sürümde aynı.
- Karar talebi **K-P2Q3-T1** (`docs/DECISIONS.md`). Seçenekler:
  - (a) D-1 açılır, PQ3-7 ve PQ3-8 P2-C'ye;
  - (b) D-1'den önce yama turu P2-Q.4 (0.3.4: dizinin atomik kurulumu, düzenek portu, C1'in zorunlu SIGKILL yolu);
  - (c) (a) ile birlikte kılavuza geçici not.
  - Önerim (b).
- Spec damgası yok: spec değişmedi.

## Bulgular
`reports/rounds/P2-Q.3/FINDINGS.md`:
- **Kapanan ve ele alınanlar:** D0F-1 ve D0F-5 kapandı; D0F-6 tasarımda ele alındı; D0F-7 kapandı. `check_deps` düşüşü açık (PQ3-1).
- **Yeni bulgular:**
  - PQ3-2: 0.3c'nin bir tekrarı eşik üstünde (`reported_only`).
  - PQ3-3: `ingest-dir`'in klasöre göre adlandırması ve sonraki uzlaştırma (B2).
  - PQ3-4: C1'in SIGKILL yolu Linux'ta sınanmadı.
  - PQ3-5: uzlaştırma yalnız `--watch`'ta.
  - PQ3-6: erişim hatası "dosya yok" sayılıyor (B1; izleyicide P2-Q.1'den beri).
  - **PQ3-7:** kapanış sıkıştırmasının ortasındaki SIGKILL dizini bozabiliyor.
  - PQ3-8: düzenek port yarışı.

## Bilinen sınırlar (gözden geçiricinin B3–B5'i)
- Uzlaştırma toptan hata verirse yalnız hata satırı yazılır; özet satırı yazılmaz.
- E1 sıcak katmanın baytlarını değil, günlüğü karşılaştırır.
- UTF-8 olmayan dosya adları not satırı yazılmadan atlanır.
- Windows yolları sınanmadı.

## Süreç notları
- PR #64 (açılış) tur başında birleşmemişti; kullanıcı `--admin` birleştirmeyi onayladı (A4).
- Yayımın insan adımları ve tur dalının yayım PR'ından sonra `--force-with-lease` ile itilmesi kullanıcı onayıyla ajan tarafından yapıldı (A5). Eski uç yerel `backup/p2-q.3-round-pre-rebase` dalında.
- Gözden geçirici alt-ajan çağrısını bir kanca (context-mode) yeniden yazıyor; auto modun denetleyicisi önce reddetti, kancanın ürettiği girdiyle yeniden gönderilince koştu (P2-Q'deki gotcha'nın aynısı).
