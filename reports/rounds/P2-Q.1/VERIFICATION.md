# P2-Q.1 — Bağımsız Doğrulama (kısa; `docs/rounds/P2-Q.1/VERIFY.md` VR1–VR4)

**Doğrulanan commit:** `08b2fa2ffd8c0a88b3ef41247030d2fdbde56911` (`main`; PR #52, rebase ile birleşti 2026-11-05T11:02:28Z). Ağacı READY commit'i `d225d52`'ninkiyle eşit (`dd4fa672…`). Yayımlanmış `v0.3.1` `e74e1f8`'de; `e74e1f8..08b2fa2` arasında ürün kodu farkı yok (yalnız `docs/`, `reports/`, `research/p2q1/`).

Ayrı oturum, kendi scratchpad'imde GitHub'dan temiz klon (önce `make setup` ve `make models`, ikisi de çıkış 0), ana çalışma dizinine erişim yok, kod değiştirilmedi. Kişisel içerik okunmadı: Obsidian sahnesinin kaydından yalnız sayılar ve `k2:` anahtarlı yollar okundu, sahne yeniden koşulmadı; bütün kendi koşularım açık korpus D-NOTES ya da kendi yazdığım sentetik notlar üzerinde. Mac kümesi koşulmadı (VERIFY; turun kaydı okundu). Ortam: Apple M4 Max, macOS 27.0.1; Docker 29.8.0 (Rosetta açık, `linux/amd64`); Claude Code 2.1.319. Bütün karşılaştırmalar kendi betiklerimle yapıldı (`vr2.py`, `vr3cmp.py`, `myscope.py`, `mycompare.py`; turun `compare.py`'si ve dondurulmuş karşılaştırıcılar kullanılmadı; betikler scratchpad'te kalır). Süre: kademe 2 dahil ~3,5 saat (hedef ≤ 2 saat; fazlası kademe 2'nin 2 sa 54 dk'lık zincirleri).

## VR1 — Tasarım ve spec

- **Sıra (`git log`):** `6b33042` R0 (`DESIGN.md` + `TESTS.md`, 01:09) → `a1dbf61` DESIGN §1.1 eki (01:13) → `9e2336e` R1'in ilk kodu (01:22) → `4fb0dcb` R2 (01:40). Tasarım ve ön-kayıtlı testler koddan önce.
- **Olay biçimi ↔ spec (kendim okudum: §5, §7, §7.1):**
  - Kaldırılan md dosyası = gözlenen içeriği boş dış değişiklik (§7.1 "External change: a change of a document's file … observed on disk") → eşleşmeyen eski bloklar `BlockDeleted` (§5 adım 7) + bloksuz `SnapshotCommitted`. Yeni olay türü, alan ya da biçim yok. Kod da öyle: `Kernel::watch_removed` aynı `ingest_external`'ı `b""` ile çağırıyor (`server/src/tools.rs`).
  - Aktör/kaynak: `canonical` altında `human`/`rootmark-serve-watch` (§7 1.0.3), tasdiksiz `unverified` altında `unattested_fs`/`unattested_fs` (§7.1 "a lineage it deletes is deleted with its actor"). `meta.layer` yok (katman veren olay yok; §7.1 Reading). Kendi günlüklerimde birebir böyle (VR2).
  - Tasdikli yeniden adlandırma: eski yolun kaldırılması `W + 500 ms` bekletiliyor, çünkü §7.1 "Rename" ve "Link update" kontrolleri eski yolun **başına** bakıyor ve tasdik `W` kadar geç gelebiliyor. Bu bir uydurma değil; spec'in zamanlama kuralından türüyor. Pencere dışı tasdikin reddi zaten §7.1 Window'un sonucu.
  - "(2.0.3) … a removal is not an event": DESIGN §1.1'in okuması (parantez betimleyici, kaldırmanın kendi olay türü yok, yalnız bloklarının `BlockDeleted`'ı) bence doğru; spec bir yasak koymuyor. `TESTS.md` D10 bunu sınıyor ve CI'da geçti.
  - **Spec'in tanımlamadığı ve uydurulan durum: bulmadım.** Belgeler arası soy taşınmıyor (tasdiksiz taşıma = eski yolun kaldırılması + yeni yolun alımı). Bugün de taşınmıyordu; spec de tanımlamıyor.
- **Ürün sınırı (`git diff 2a0e3a9..08b2fa2`):**
  - `core`, `index`, `bindings`, `spec`, `clients/*/src`: yalnız üç `Cargo.toml` sürüm satırı.
  - `server/`: `src/serve.rs`, `src/tools.rs` (yalnız `watch_removed`, `watched_docs_under`, `attestation_window`), yeni `tests/watch_newdir.rs`, `tests/watch_remove.rs`, `tests/watchkit/mod.rs`. Dondurulmuş `serve_watch.rs` değişmedi.
  - Öteki ürün dışı dosyalar: `release.yml` (`permissions: contents: read`, `on:`'dan önce), istemci sürüm dizgileri, `CHANGELOG`/`README`/`GUIDE`. Dondurulmuş dosyalarda tek değişiklik `scripts/release_check.sh` (F1, `FREEZE_EXCEPTIONS.md` + `FROZEN.md` satırı).
- **Kod okuması (izleyici):** üç hedef (`Doc`/`Scan`/`Gone`), yeni ya da taşınan dizinin özyinelemeli taranması (gizli bileşen atlanır, sembolik bağ izlenmez), bekleyen kaldırmalar bayt sırasıyla. Durdurmada bekleyenler beklemeden işleniyor (izleyici iş parçacığı birleştiriliyor). Duvar saati yalnız ne zaman işleneceğine karar veriyor; günlüğe girmiyor. PQ1-4 (bekleme süresinin toplu alımdan önce hesaplanması) kodda görülüyor ve turun bulgusu olarak kayıtlı.

## VR2 — Hataları kendim ürettim, kapandığını gördüm

Kendi betiğim `vr2.py`: yayımlanmış arşivlerin ikilileri (`SHA256SUMS` OK, aşağıda), `rootmark serve --http … --watch`, kendi yazdığım vault reçetesi (`window_s = 2.0`). Üç sahne: **yeni dizin** (20 kez `mkdir ndNN` + hemen `ndNN/n.md`, 0,3 s arayla). **Kaldırma** (silme; vault içinde `os.rename` ile taşıma; vault dışına taşıma; iki notlu dizinin `rm -r`'ı; 8 s bekleme). **Tasdikli yeniden adlandırma** (`unattested = "unverified"`; not yazılıp `rootmark attest` ile `canonical`, sonra `mv` + `rootmark attest <yeni> --renamed-from <eski>`, bir kez zamanında, bir kez 1,2 s geç).

| | Mac 0.3.0 | Mac 0.3.1 | Linux 0.3.0 (Docker `linux/amd64`, `python:3.12-slim-trixie`) | Linux 0.3.1 |
|---|---|---|---|---|
| Yeni dizin: alınan | 20/20 | 20/20 | **19/20** (`nd00/n.md` kaçtı) + kaldırma sahnesinde yeni `dd/`'nin **iki notu da** hiç alınmadı (`live_before` 0) — **PQ-22 yeniden üretildi** | **20/20**; `dd/` notları alındı |
| Silinen not: canlı blok / aramada | 6 / evet | **0 / hayır** (`BlockDeleted` 6) | 6 / evet | **0 / hayır** (`BlockDeleted` 6) |
| Vault içinde taşınan not, eski yol | 6 canlı, arama eski + yeni yol | **0**, arama yalnız yeni yol | 6 canlı, arama eski + yeni | **0**, arama yalnız yeni |
| Vault dışına taşınan not | 6 canlı, aramada | **0**, aramada yok | 6 canlı, aramada | **0**, aramada yok |
| `rm -r` edilen dizinin 2 notu | 6 + 6 canlı, aramada | **0 + 0**, aramada yok | (hiç alınmamıştı) | **0 + 0**, aramada yok |
| Tasdikli yeniden adlandırma (zamanında / geç): eski yol | 6 canlı (çift görünür) | `BlockDeleted` **6, bir kez**; canlı 0 | 6 canlı | `BlockDeleted` **6, bir kez**; canlı 0 |
| … yeni yol | `BlockCreated` 6, `canonical` | `BlockCreated` **6, bir kez**, `canonical` (geçte + `LayerChanged` 6, tasdik geç) | aynı | aynı |

- **Aktörler (0.3.1 günlükleri):** `canonical` vault'ta her `BlockDeleted` `{kind: human, id: verifier}` / `rootmark-serve-watch`. `unverified` vault'ta tasdikli yeniden adlandırmanın eski yolu `{kind: unattested_fs, id: fs}` / `unattested_fs`. Kaldırmanın kendisi tasdik edilmediği için bu §7.1'e uygun. Kılavuz bunu silmeler için yazıyor ("every deletion is unattested"); yeniden adlandırmanın eski yolu için açıkça yazmıyor (gözlem, Öneri 3).
- **Çift işlem:** iki platformda, zamanında ve geç tasdikte, yeni yolun `BlockCreated`'ı 6 (blok sayısı), eski yolun `BlockDeleted`'ı 6 (soy başına bir). Tasdik çıkışı 0. **Çift işlem yok.**
- **Not:** Linux koşuları Rosetta altında emüle edildi. 0.3.0'ın kaçırması kesintili: 20'den 1'i + iki yeni dizin dosyası. Turun yerel Linux kanıtı (`ROUND_START`'ta kalan test) ve CI'daki yerel `ubuntu-latest` koşuları bunu tamamlıyor.

## VR3 — Karşılaştırma ve kademe 2

### D-NOTES (P2-Q'nün sürücüsü `research/p2q/driver.py`, yayımlanmış 0.3.1)
- **Sapma:** kılavuzun kurulum blokları (§1.1#1–#2 Mac, §1.3#2 Docker; `guide_run`) iki kez denendi. İkisinde de GitHub sürüm indirme uç noktası `curl`'e 502/503 döndü (`curl: (56) … 503`, "tried 5 times"); aynı varlıklar `gh release download` (API) ile indi. Bu yüzden kurulum bloğu yerine aynı arşivleri doğrudan kullandım: `SHA256SUMS` doğrulandı ve köken tasdiki `gh attestation verify` ile çıkış 0. Model, `make models`'in sabitlenmiş manifestle doğruladığı dosyalar (arşivdeki `MANIFEST.json` ile `cmp` eşit). Docker'da senaryonun ön-kayıtlı imajı (`debian:trixie-slim@sha256:f6b08b09…`). Turun yayımdan önceki aday denemesi de aynı yöntemi kullanmıştı. Ölçülen ikili yayımlanmış olanın aynısı.
- **Sonuç:** Mac ve Docker (`VirtualApple`) ikisi de **19/23**; kalanlar H02, H11, H15, H21, yani P2-Q'dekilerle ve turunkilerle aynı. **H09 geçti**: `not_ingested` boş, `unverified` 54, `canonical` 0.
- **Bayt eşitlik (`vr3cmp.py`; `hashes.json` — `events_hash`, indeks dosyaları, vektörler, manifest, sıralama özetleri — ve `queries.json`, yaprak yaprak):** benim Mac ↔ benim Docker ↔ turun Mac ↔ turun Docker ↔ CI `41996549106` ↔ CI `41996563930` ↔ P2-Q'nün 0.3.0 Mac kaydı: `events_hash` `173c13df79d2d98dbff1df45453e193c` hepsinde. `hashes.json` blake2b-128 `113d0184…` ve `queries.json` `c42e42ca…` yedisinde aynı; yaprak farkı **0**.

### Kademe 2 — kendi tetiklemem **`42471635466`**
`gh workflow run verify.yml --ref main -f sha=08b2fa2ffd8c0a88b3ef41247030d2fdbde56911`, 2026-11-08T10:29:30Z.
- İki zincir `success`, `exit` 0/0, `commit` dosyaları `08b2fa2…`. İşlemciler AMD EPYC 7763 ↔ Intel Xeon Platinum 8370C (farklı). Zincirler 10 103 s, karşılaştırma 217 s.
- **Kendi D karşılaştırmam (`myscope.py`, yapıtlar `gh run download` ile indirildi):**
  - Kapsam olarak dondurulmuş çiftin dosya listesini veri olarak aldım: 97 alan dosyası, 33 bayt yolu. Karşılaştırmayı kendi kodum yapıyor: her JSON yaprağı tam eşitlik, bayt yolları `cmp`.
  - Envanterin adlı dışlamaları ve gömülü fark kuralı (kural 3) uygulandı. Envanter alan adlarındaki `·` = `.` kaçışını ilk sürümüm atlamıştı; düzeltip yeniden koştum.
  - **A ↔ B:** 87 dosya, 759 254 yaprak; bayt yolları 25 (28 dosya), **fark 0**.
  - Zaman dışı yaprak farkı 5. Hepsi dondurulmuş karşılaştırıcıların kaynakta adlı zaman/bilgi alanları: `h3_v3.json` `summary›p50/p95` (`compare_runs.py`), `p1d1/d1_6.json` ve `p1d1_checks.json` `D1-6›measured›ratio` (`compare_p1d1.py`), `p1e1/release/release.json` `targets›*›sha256` (`compare_p1e1.py`).
  - Kalan 413 yaprak gecikme/süre alanı (10 ad).
  - **D farkı 0.** CI'ın `tier2_pair.json`'ı da `equal: true`, 788 892 alan, kenara alınan yalnız Z28.
- **Karşılaştırma dışı liste = envanter:** `not_compared` kimlikleri `reports/rounds/P2-V.2/inventory.json`'ın 48 kimliğiyle birebir; desen (`*?[`) 0.
- **Sayılan FAIL (kendi sayımım, zincirlerin kendi `fail_summary.json`'ından):**
  - Her zincirde 39 FAIL etiketi: 23 tarihsel, 2 `migration_required` (E1-1, E-14; karar K-P2A.2), 14 `fail`.
  - 14 `fail`in hepsi Mac kümesine ayrılan kriterler (O: C1-8, E1-3, E1-4, E1-9, tamper, E-7, 0.3b, 0.3b-yük, 0.3c, 0.3d, 0.3e; Z: logs_vs_p2a; türev: D1-12, D-9 tam listeyle).
  - **Sayılan FAIL 0 / 0.** `tamper·det` tarihsel, `record_equal: true`. CI'ın `tier2_fails_{A,B}` çıktısı da 0.
- **`portable`:** A ve B **bulgu 0** (1 153 dosya, `skipped_by_name` 1).
- **Kasıtlı bozma (karşılaştırıcımın sınaması, kopyada):** bir D kaydına alan eklenmesi ve `governance` günlüğünde tek bitin çevrilmesi yakalandı. Envanterin dışladığı `p2_0_2/d_c/events.jsonl`'e eklenen satır yakalanmadı (dışlama doğru uygulanıyor).
- **Değişen kayıtlar ↔ R0'ın adları (DESIGN §8):**
  - R0'ın adlandırdığı tek D kaydı `p2_0_2/tamper_p2_0_2.json`: benim A ve B'm, tur öncesi `41383038238` (`89c4b28`), turun `41944358184`'ü ve turun yerel önce/sonra kayıtları — blake2b-128 `97cef592…` altısında **bayt eşit** (R0'ın beklentisi "aynı").
  - Z için R0 "yok" demişti; Z alanlarında da D tarafına sızan fark yok.
  - Turun ikinci tetiklemesi `41944358184` (`110d243`, aynı ürün kodu) ↔ benimki ve tur öncesi `89c4b28` ↔ benimki; zaman alanları dışında, aynı kapsamda yalnız şu farklar var:
    - Sürüm dizgileri (`0.3.0`/`0.2.0` → `0.3.1`: E-7, E1-5, `release_local.json`, `conformance_rootmark.json`, MCP transcript'lerinin `serverInfo.version`'ı).
    - `release.json` `commit` (commit'e bağlı).
    - Dondurulmuş listedeki arşiv sha'sı.
    - **E1-4'ün `release_check`'i 18/21 → 17/21 (yeni kalan R14)** ve bunu taşıyan gömülü `regress` alanları.
  - Watcher'a bağlanabilen bir D kaydı değişikliği yok. **Adsız fark: D'de 0.**
  - R14 bir O alanı (E1-4, Mac kümesi). Ama nedeni ürün değil depo durumu; bkz. Fark 1.

### Kademe 1 (yerel)
`make verify FAST=1` (temiz klon, `08b2fa2`): çıkış 0, hızlı çift **eşit (18 550 alan)**, 1 271 s. Turun PR'ında (`42010770901`) kademe 1 yeşil: fast chain A 34 dk 23 s, B 23 dk 14 s (≤ 40 dk).

## VR4 — Belgeler, güvenlik, karar kuralı

- **Kılavuz §5.1 (VF-1):** "Do you trust the author of this vault?" penceresi, iki düğmesi ("Trust author and enable plugins" / "Browse vault in Restricted Mode") ve "Settings → Community plugins → **Exit Restricted mode**" yazılı (eski adı parantezde). P2-Q doğrulamasının koşulu (1) belgede karşılanmış. PQ-24..26 `reports/rounds/P2-Q/FINDINGS.md`'de.
- **Kılavuz §4.1 (ajan ayrımı) — kendi oturumumda sınadım.** Claude Code 2.1.319, `claude -p` başsız, kılavuzun yapılandırması (`permissions.deny` `Read(//<vault>/.rootmark/**)`, `Edit(//<vault>/**)`; `sandbox` `denyRead`/`denyWrite`, `allowUnsandboxedCommands: false`, `failIfUnavailable: true`) `--settings` ile. Rastgele baytlı sahte bir `server.token` ile, turun beş adımından farklı altı deneme yaptım:
  - **Kurallarla 6/6 red:** Grep aracı, `head -c`, `node -e readFileSync`, `cp` ile dışarı kopya, vault'a `mkdir`/`touch`, Edit ile ekleme. Token değeri çıktıda yok, dosya kopyalanmadı ya da yazılmadı.
  - **Kurallar olmadan 5/6 başarı:** token okundu ve kopyalandı, vault'a dosya yazıldı. 6. adımı (Edit) Claude Code'un önce-Read şartı durdurdu, kural değil.
  - **Redlerin kaynağı model değil, kurallar.** İlk koşuda 1–4'ün red gerekçesini model yazmıştı. Ham hata metnini isteyen ikinci koşuda 1–4 kuralın ve işletim sisteminin hatasıyla döndü: "File is in a directory that is denied by your permission settings.", `operation not permitted`, `PermissionError: [Errno 1] Operation not permitted`, `cp: … Operation not permitted`.
- **Dependabot / CodeQL (`gh api`, 2026-11-08):**
  - Açık CodeQL **0**. #1 `fixed` (`release.yml`); #2–#4 `dismissed` "used in tests", gerekçe yorumlu (K-P2Q1.2).
  - Açık Dependabot 2: #80 ve #81, `minimatch`, orta önem, `development` kapsamı, düzeltme 11.0.1. Gerekçe yazılı (14 gün kuralı 2026-11-10'dan sonra) ve bugün (2026-11-08) hâlâ geçerli.
- **0.3.1 yayımı:**
  - `release.yml` `41994456323` `success` (`e74e1f8`; linux-gnu, apple-darwin, windows-msvc, provenance işleri). `v0.3.1` → `e74e1f8`, 12 varlık.
  - `shasum -a 256 -c SHA256SUMS`: üç arşiv, `install.sh`, `install.ps1` **OK**.
  - `gh attestation verify` üç arşivde çıkış 0: imzalayan `release.yml@refs/heads/main`, kaynak `e74e1f8`, koşu `41994456323`.
  - Turun yerel `release_check.json`'ı 21/21.
- **Karar kuralı (GOAL'den kendim türettim):**
  - Q1-1 geçti → park yok.
  - Q1-2 ve Q1-3 geçti → PQ-22 ve PQ-18 kapandı, 0.3.1 iki düzeltmeyle yayımlı.
  - **Q1-4:** turun ön-kayıtlı tek tetiklemesi `41927841456`'da sayılan FAIL 1 ve adsız fark (E1-5 `workflow_r13`) var; kayıtları kendim okudum, `tier2_fails_{A,B}.json` `counted_fails: 1`. → "Q1-4 kaldı → tur kaldı; karar talebi" → **K-P2Q1-T3**.
  - Q1-5..Q1-9 geçti. Hepsi-geçti dalı işlemez.
  - **FINAL ile aynı dal.** Kendi tetiklemem K-P2Q1-T3'ün öneri (a)'sının istediği teyidi veriyor: aynı ürün kodunda sayılan FAIL 0, D farkı 0, `portable` 0.
- **Vaat sorusu (GOAL'ün ölçmediği, NORTH_STAR'ı ilgilendiren):**
  - (i) `unverified` altında tasdikli bir yeniden adlandırmanın eski yolu `unattested_fs` aktörüyle kapanıyor. Spec'e uygun, ama günlüğü okuyan biri bunu "ajan sildi" diye okuyabilir. Kılavuz yalnız silmeyi anlatıyor.
  - (ii) Ajan yolu (MCP `propose`) güncel Claude Code ile hâlâ çalışmıyor (PQ1-1 → P2-Q.2). Kurallı oturumda ajanın vault'a tek yazma yolu MCP olduğundan, §4.1'in önerdiği ayrım bugün ajanı **yazamaz** hale getiriyor. Bu P2-Q.2'nin önceliğini destekliyor.

## Sonuç

### İhlal / fark / şüphe
- **İhlal:** yok. Ürün sınırı, dondurulmuş dosyalar (yalnız F1), "markdown'a kimlik yazılmaz", kimliklerde zaman/rastgelelik yok, sessiz başarısızlık yok: kaldırma günlükte, beklemedekiler durdurmada işleniyor.
- **Fark 1 — `release_check` R14 commit'e değil etikete bağlı.**
  - R14 "etiket yok ya da etiketin commit'i en yeni `ROUND_START`'ın atası". `v0.3.1` turun içinde (`e74e1f8`) atıldığı için, P2-Q.2'nin `ROUND_START`'ına kadar R14 her koşuda kalıyor.
  - Aynı ürün kodunda turun `41944358184`'ünde (etiket henüz yoktu) E1-4 18/21, benimkinde 17/21.
  - E1-4 O sınıfı, kademe 2'nin FAIL sayımından düşülüyor ve iki zincir arasında aynı; hükmü değiştirmiyor. Ama "D: aynı girdi → aynı çıktı" okumasına commit dışı bir girdi (deponun etiketleri) giriyor; Mac kümesinin E1-4 okuması da bundan etkilenir (Öneri 2).
- **Fark 2 — kılavuzun kurulum bloğu bu oturumda koşulamadı.** GitHub `curl` 502/503 döndü; arşivler doğrudan, sağlama ve köken doğrulamasıyla kullanıldı (VR3). Ürünle ilgisi yok; kılavuzun kurulum yolu turun kayıtlarında (`guide_run` ok/ok) ve P2-Q doğrulamasında sınanmıştı.
- **Şüphe (küçük):**
  - Linux'taki kendi PQ-22 yeniden üretimim emülasyon altında ve kesintili (3 kaçış). Düzeltmenin yerel Linux kanıtı turun ve CI'ın.
  - Obsidian sahnesini (O06/O10/O14) kişisel içerik nedeniyle yeniden koşmadım; yalnız kaydı okudum: 6 eski yolda canlı blok 0, `all_blamed: true`. Obsidian dışı silme/taşıma ve CLI ile tasdikli yeniden adlandırma kendi koşumda.

### Kriter başına (benim ölçümüm ve okumam)
| Kod | Sonuç | Dayanak |
|---|---|---|
| Q1-1 | **geçti** | DESIGN/TESTS `6b33042`/`a1dbf61` < ilk kod `9e2336e`; her olay öğesi §5/§7/§7.1'e atıflı ve kodla aynı; uydurma 0 (VR1) |
| Q1-2 | **geçti** | `watch_newdir` N1–N6 (N1–N3 20 tekrar, N4 iç içe/çok dosya) PR CI'ında `macos-14` ve `ubuntu-latest` 3/3 ok (günlükten okudum). PQ-22 0.3.0'da yeniden üretildi, 0.3.1'de 20/20 (VR2). D-NOTES 0.3.1 Mac ↔ Docker (benim) ↔ turun Mac/Docker ↔ CI×2 bayt eşit, H09 20/20. Okuma: "bütün adımları" = 23 adım koştu; 19 geçti, kalan 4 P2-Q'deki kalışlar (GOAL "Yapılmayacaklar": P2-C) |
| Q1-3 | **geçti** | `watch_remove` D1–D10 (9 test) PR CI'ında macOS ve Linux 9/9 ok. Kendi koşumda Mac ve Linux'ta silme/taşıma/dizin silme → canlı 0, aramada 0; tasdikli yeniden adlandırmada çift işlem 0 (VR2). Obsidian kaydı: eski yollarda canlı 0 (okundu) |
| Q1-4 | **kaldı (kuralla)** | ön-kayıtlı tek tetikleme `41927841456` sayılan FAIL 1, adsız (E1-5). Bilgi: kendi tetiklemem `42471635466` D farkı 0, sayılan FAIL 0, portable 0, R0'ın adlandırdığı D kaydı aynı. Mac kümesi kayıtlı (turun `mac_set/mac_checks.json`, okundu: 3 tekrar, 6 534 s, `pass: false` — tek koşuluk E1-4'ün kaydı farklı, FINAL'in yazdığı `release.yml` nedeni) |
| Q1-5 | **geçti** | sınır farkı yalnız sürüm satırları + izleyici + yeni testler; dondurulmuş değişiklik yalnız F1; kendi `make conformance` (temiz klon, `08b2fa2`) **level 3, 39/39**; yeni olay/alan/biçim yok (göçsüz) |
| Q1-6 | **geçti** | VF-1/PQ-24..26 yazılı; §4.1 kendi oturumumda sınandı (kurallarla 6/6 red, işletim sistemi hatasıyla); CHANGELOG `## 0.3.1` ve Known issues güncel |
| Q1-7 | **geçti** | açık CodeQL 0 (#1 fixed, #2–#4 gerekçeli); açık Dependabot 2, gerekçeli ve süresi dolmamış |
| Q1-8 | **geçti** | `release.yml` `41994456323` success (zorunlu iki hedef ok); `v0.3.1` yayımlı; SHA256SUMS OK; köken 3/3; `release_check` 21/21 (turun kaydı) |
| Q1-9 | **geçti** | PR #52 CI yeşil (test+lint+conformance macOS/ubuntu, kademe 1 ≤ 40 dk); kendi `make verify FAST=1` eşit; gözden geçirici KABUL (`R5.review.md`) |

### Hüküm
**Hüküm: Güvenilir.**

Turun iddiaları bağımsız olarak yeniden üretildi:
- İki hata 0.3.0'da benim koşumda görünüyor, 0.3.1'de kapalı (Mac + Linux).
- Olay biçimi spec'in mevcut kurallarında; tasdikli yeniden adlandırma çift işlenmiyor.
- D-NOTES kayıtları Mac ↔ Docker ↔ CI bayt eşit.
- Kendi kademe 2 tetiklemem `42471635466` temiz.

Karar kuralının dalı (Q1-4 kaldı → K-P2Q1-T3) FINAL'inkiyle aynı; FINAL'deki bütün sayılar okuduğum kayıtlarla tutuyor. Fark 1 ve 2 hükmü değiştirmiyor.

### Öneriler (≤ 3)
1. **K-P2Q1-T3'ü (a) ile kapatın.** Q1-4 kayıtlı kalış olarak kalsın. Kendi tetiklemem `42471635466` (`08b2fa2`; D farkı 0, sayılan FAIL 0/0, portable 0/0, R0'ın D kaydı aynı) seçeneğin istediği teyit. Kalışın nedeni (`release.yml`'nin `permissions`'ı R13'ün okuduğu bloğa düştü) turun kendi yapılandırma değişikliği, ürün değil.
2. **R14'ün etikete bağlılığını P2-Q.2'nin açılışında yazın.** E1-4/R14, yayım etiketi turun içinde atıldığında bir sonraki `ROUND_START`'a kadar kalır; aynı commit farklı zamanda farklı kayıt üretir. P2-Q.2'nin R0'ı, kademe 2 ve Mac kümesinde "değişmesi beklenen kayıtlar" listesine E1-4 (R14) ve sürüm dizgisi alanlarını (0.3.2) adıyla yazsın. Tur içinde yayım yapan yama turları için kural DECISIONS'ta bir cümleyle belirtilsin (dondurulmuş `release_check.sh`'e dokunmadan).
3. **Kılavuz ve GOAL ifadesinde iki küçük netlik.** (a) Kılavuzun "Removals are taken a little late" maddesine, `unverified` altında Obsidian'ın **tasdikli** yeniden adlandırmasının eski yolunun da `unattested_fs` ile kapandığını ekleyin (kaldırma tasdik edilmez). (b) Sonraki GOAL'lerde senaryo adımları için "koştu" ile "geçti" ayrı yazılsın (Q1-2'nin "bütün adımları" okuması).
