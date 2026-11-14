# P2-Q.4 — Tasarım: PQ3-9'un kök nedeni, atomik dizin kurulumu, başlangıç denetimi, PQ3-6, düzenekler

R0'da, R1'in ilk kod commit'inden önce commit'lenir (Q4-1). Kod dayanağı `ROUND_START` (`b6f7193`; ürün kodu `v0.3.3` = `8f0bcb8` ile aynı): `server/src/serve.rs` (`start_watcher`, `ingest_removal`, `run`), `server/src/tools.rs` (`start_index`, `watch_removed`, `reconcile_files`, `remove_missing_under`), `index/src/store.rs` (`install`, `Store::open`, `load_hot`, `Store::index_head`, `Store::status`).

## 1. PQ3-9 — kök neden (kanıtla)

### 1.1 Olay sırası (kod okuması)
`a2_sigkill_move_canonical`: `n0.md` → `sub/moved.md` (`rename`), 0,3 s sonra SIGKILL, yeniden başlatma.
1. İzleyicinin toplu işi iki yolu adlandırır. `sub/moved.md` dosya → `ingest_doc` → `Kernel::watch_doc` → `ingest_external` → `Store::watch_plan`: önce `Vault::commit_plan` (olaylar `events.jsonl`'e), sonra `Store::index_head`: gömme, sonra sıcak katmana **dört ayrı ek**, her biri `sync_data`'lı: `hot/records.jsonl` → `hot/embedders.jsonl` → `hot/vectors.f32` → `hot/snapshots.jsonl` (`index/src/store.rs` `index_head`). `n0.md` yok → kaldırma, `held`'de 1,5 s bekler.
2. SIGKILL bu dört ekin arasına (ya da birinin ortasına) düşerse sıcak katman **hizasız** kalır: günlük `sub/moved.md`'nin yeni başını taşır, sıcak katmanda `records.jsonl` bir satır önde, `snapshots.jsonl` eksik.
3. Yeniden başlangıç: `start_index` yalnız `manifest.json`'u okur (`store::staleness`) → `Current`. Uzlaştırma (`reconcile_files`) `sub/moved.md`'yi günlükle aynı bulur (yoklama olay üretmez) → depo **açılmaz**; `n0.md` eksik → `held`'e girer, özet satırı yazılır (`reconciled()` döner).
4. 1,5 s sonra `ingest_removal` → `Kernel::watch_removed` → `Store::open` → `load_hot` → hata: `hot tier invalid (rebuild with \`rootmark index build\`)`. `ingest_removal` hatayı stderr'e yazar (`watch: n0.md: removal: error: …`) ve kaldırmayı **bırakır**: yeniden denenmez. Test 90 s bekler → "timed out: n0.md's removal ingested".

Aynı sınıfın ikinci yüzü (R0'da doğal olarak görüldü, §1.3): SIGKILL `commit_plan` ile ilk ek arasına (gömme süresince — yükte en geniş pencere) düşerse sıcak katman tutarlıdır ama günlüğün başı dizinde yoktur; başlangıç yine `Current` der ve her `search` 500 döner (`snapshot … of sub/moved.md is not indexed`; `index/src/search.rs` `View::new`). Sunucu kapalıyken `ingest-dir`'den sonra da aynı durum vardır (bugün, `index status`'un `stale_docs`'u).

### 1.2 Kanıt
- **Belirlenimli üretim (0.3.3 + tanı yaması, yalnız scratch çalışma ağacında; ürün değil):** `index_head`'in k. ekinden sonra `abort` (k = 1, 2, 3; yalnız `sub/moved.md` için) ve yeniden başlatma: **9/9 tekrar** aynı hatayla kaldı ("timed out: n0.md's removal ingested"); ikinci sunucunun günlüğü her tekrarda `reconcile end changed=0 missing=["n0.md"]` → 1,5 s sonra `removal attempt n0.md` → `watch: n0.md: removal: error: hot tier invalid (rebuild with \`rootmark index build\`): read …/hot/snapshots.jsonl: No such file or directory`. Öldürmeden sonraki sıcak katman: k=1 `records 1, embedders yok, vectors yok, snapshots yok`; k=2 `records 1, embedders 1, …`. Kayıt: `reports/rounds/P2-Q.4/r0/abort_hot_macos/`, yama `reports/rounds/P2-Q.4/r0/diag_v0.3.3.patch`.
- **Doğal SIGKILL, yük altında (0.3.3 + aynı tanı yaması; enjeksiyon kapalı; macOS ve Linux 200'er tekrar):** A2'de öldürmeden sonra sıcak katmanı hizasız kalan **5 tekrarın 5'i** CI'daki iletiyle kaldı (macOS 2, Linux 3) ve beş günlüğün beşi yukarıdaki sırayı gösterir; sıcak katmanı hizalı kalan **793 tekrarın 0'ı** kaldı. Kalan öteki 17 tekrar (A2'de 2, C1'de 15) aynı sınıfın öteki yüzleridir: günlük dizinden önde (`snapshot … is not indexed`, 500) ve kapanış sıkıştırmasının yarım bıraktığı soğuk dizin (PQ3-7). Bütün 22 kalış dizinin günlükle tutarsızlığıdır; port yarışı yok. Ayrıntı: §1.3.

### 1.3 Yük altındaki yeniden üretim
`progress/R0_repro.md`, kayıtlar `reports/rounds/P2-Q.4/r0/`. Koştu/geçti (macOS · Linux): A1 iki politika 200/200 · 200/200; A2 `canonical` 200/198 · 200/197, `unverified` 200/199 · 200/199; C1 `canonical` 200/195 · 200/198, `unverified` 200/191 · 200/199. Kalışların sınıfları: (1) hizasız sıcak katman → kaldırma düşer (A2: 5); (2) günlük dizinden önde → `search` 500 (A2: 2, C1: 2); (3) kapanış sırasındaki SIGKILL → hizasız sıcak katman ya da yarı silinmiş soğuk dizin → `search` 500 (C1: 13). PQ3-4'ün gözlemi de değişti: yük altında Linux'ta da C1'in SIGKILL'i gönderildi (47/198, 47/199).

### 1.4 Neden FSEvents ya da bekleme zamanlayıcısı değil
GOAL'ün sınanacak öteki yolları: (i) FSEvents'in eski olayları yeni izleyiciye iletmesi — tanı kaydında yeniden başlangıçtan sonra gelen her olay `n0.md`'yi yeniden `held`'e koyar ya da hiçbir şey yapmaz; kaldırma yine süre sonunda denenir (bir olay kaldırmayı yok edemez, yalnız erteler; erteleme sınırlıdır, P2-Q.2 DESIGN §4); (ii) taşımada eksik ve yeni dosyanın aynı toplu işte okunması — uzlaştırmada yeni yol değişmemiş, eski yol eksik olarak görülür (kayıtta `missing=["n0.md"]`); (iii) bekleme zamanlayıcısı — kayıtta kaldırma 1,5 s'de denenir ve **hata** ile biter. Üç yolda da kaldırma denenir; düşmesinin tek nedeni denemenin hatasıdır.

## 2. Düzeltme: başlangıçta dizinin günlükle tutarlılığı (R1)

### 2.1 Kural
`rootmark serve`'ün başlangıcında (`Kernel::start_index`), manifest bu uygulamanınki (`Current` olurdu) iken dizin bir de **bütün olarak** denetlenir (`store::incomplete`, yeni, `index/src/store.rs`):
1. `Store::open` (soğuk dosyalar manifest'le, sıcak katman kendi içinde hizalı) başarısızsa → **eksik** (neden: hatanın metni).
2. Açılırsa: günlüğün her dokümanının başı dizinde mi (`Store::status(vault).stale_docs`, bugünkü `index status`'un kuralı) — değilse → **eksik** (neden: "N documents' heads are not in the index (first: <doc>)").
3. İkisi de değilse `Current` (bugünkü gibi, ileti yok).

Eksik dizin `[index] auto_rebuild = true`'da (varsayılan) **yeniden kurulur** (`store::build`; geçerli soğuk vektörler yeniden kullanılır, gömme belirlenimli, baytlar temiz `rootmark index build`'inkiyle aynı) ve stderr'e açık bir satır yazılır: `rootmark serve: index incomplete (<neden>; an interrupted write): rebuilt (<n> records, <m> snapshots, <embedder>)`. `auto_rebuild = false`'ta dizin olduğu gibi kalır ve satır `index incomplete (<neden>); [index] auto_rebuild = false: rebuild it with \`rootmark index build\``; indeks isteyen çağrıların yanıtı bugünkü gibidir (`internal`).

### 2.2 Neden bu yol (ve öteki seçenekler)
- **Sıcak katman ekini tek atomik yazım yapmak** `Store::index_head`'i değiştirir; GOAL'ün ürün sınırı `index/src/store.rs`'te yalnız `install`'a ve bütünlük denetimine izin verir. Üstelik günlük–dizin arasındaki pencereyi (§1.1 ikinci yüz) kapatmaz: günlük ile dizin iki ayrı yerdir, aralarına düşen SIGKILL her durumda bir sonraki başlangıçta onarılmalıdır.
- **Başarısız kaldırmayı yeniden denemek** nedeni kapatmaz: depo açılamadıkça her deneme aynı hatayı verir ve arama 500 kalır.
- **Hizasız sıcak katmanı geri kesmek** (son eksiksiz `snapshots.jsonl` satırına) dizini günlüğün gerisinde bırakır; yine 2. denetim ve yeniden kurma gerekir. Yeniden kurma tek yol olarak daha basit ve baytları belirlenimlidir.
- Dizin türevdir (NORTH_STAR ilke 1): günlükten yeniden kurmak bilgi kaybetmez.

### 2.3 Sözleşme ve önceki kararlar
- `docs/CLIENT_CONTRACT.md` değişmez: yeni uç, alan, kod, araç yok; `status.index.state` yalnız manifest'i okumaya devam eder (K-E1.2: dosyaları manifest'le uyuşmayan dizin `ok` görünür). Sözleşme §3'ün "başlarken indeks yoksa kurar; başka biçimdeyse `auto_rebuild` ile yeniden kurar" kuralına GOAL'ün istediği durum eklenir: eksik dizin `auto_rebuild` kuralıyla yeniden kurulur (sözleşmedeki "eksik" sözcüğünün kapsamı; ASSUMPTIONS A1). Kılavuz (R4) bunu yazar.
- **K-E1.2 aynen:** okunamayan manifest (`damaged`) hiçbir ayarda kendiliğinden kurulmaz; `store::incomplete` yalnız manifest okunur ve bu uygulamanınki iken çağrılır.
- Tek bilet içinde (`ticket().run`) çalışır; MCP/HTTP başlangıç bittikten sonra açılır (bugünkü sıra).

### 2.4 Nedenin doğrudan testi (`server/tests/hot_tear_restart.rs`)
SIGKILL'in düştüğü her nokta, gerçek bir SIGKILL'den sonra disk durumu birebir kurularak sınanır: sunucu `sub/moved.md`'yi sıcak katmana bütünüyle ekledikten sonra (kaldırma beklerken; `window_s` uzun) SIGKILL alır; sonra sıcak katman dosyaları, `index_head`'in sırasıyla, k. ekten sonraki hâline kesilir (k. ekten sonrakiler eklemeden önceki boyutlarına). Böylece ürün koduna enjeksiyon gerekmez ve durum, abort'lu tanı koşusunda gözlenen durumla aynıdır (§1.2). Noktalar: H0 hiçbir ek yok (günlük önde), H1/H2/H3 1./2./3. ekten sonra, H4 `records.jsonl`'in son satırı yarım (bir ekin ortası). Beklenen: yeniden başlangıçta `index incomplete … rebuilt` satırı; `n0.md` tam bir kez kapanır; `search` 200, `zqr0` yalnız `sub/moved.md`; durdurmadan sonra dizin baytları aynı günlükten temiz `rootmark index build`'inkiyle eşit.

## 3. PQ3-7 — atomik kurulum (R2)

### 3.1 Bugünkü sıra ve sorun
`install`: `remove_dir_all(index/)` sonra `rename(index.build → index/)`. `remove_dir_all` dosyaları birer birer siler; SIGKILL arada gelirse yarım dizin kalır (manifest kalabilir, `lexical/<id>.store` gitmiş olabilir); başlangıç onu `Current` sayar, `search` 500 (P2-Q.3 R5: C1 `unverified` 1/20, `r5/make_test_run1_failures.txt`).

### 3.2 Yeni sıra
`index.build` (`write_cold`) bugünkü gibi eksiksiz yazılır, sonra:
1. `.rootmark/index.old` varsa silinir (önceki bir kurulumun yarım temizliği);
2. `index/` varsa `rename(index/ → index.old)` — tek atomik adım;
3. `rename(index.build → index/)` — tek atomik adım;
4. `remove_dir_all(index.old)`.
Her SIGKILL noktasında bir sonraki başlangıç (`store::recover_install`, `start_index`'in ilk adımı) şunu görür:
- 1–2 arası ya da 2'den önce: `index/` eski ve eksiksiz; artık `index.build` silinir (satır `… removed an index build left unfinished at .rootmark/index.build`) → **eski dizin**.
- 2–3 arası: `index/` yok, `index.old` eksiksiz eski dizin → `rename(index.old → index/)` ve satır `rootmark serve: index: an interrupted install left the previous index at .rootmark/index.old: restored` → **eski dizin**.
- 3–4 arası ya da 4'ün ortası: `index/` yeni ve eksiksiz, `index.old` (belki yarım) artık → silinir (satır `… removed the previous index left at .rootmark/index.old`) → **yeni dizin**.
Sonra §2'nin denetimi koşar: eski dizin günlüğün gerisindeyse (ör. kesilen kurulum bir `index build`'in onarımıydı) eksik sayılır ve yeniden kurulur — **sessiz değil**.
- Eski (0.3.4'ten önceki) bir sürümün yarım bıraktığı dizin (manifest var, dosya eksik) §2'nin 1. adımında eksik görülür ve yeniden kurulur (PQ3-7'nin 0.3.2/0.3.3'teki belirtisi).
- `index.old` `.rootmark/` altında geçici bir kardeş dizindir (`index.build` gibi); kesintisiz bir koşudan sonra yoktur. Dizinin kendi dosyaları ve biçimi değişmez (göç yok); kurulmuş dizinin baytları bugünküyle aynıdır.
- Güç kesintisine karşı dayanıklılık (`fsync` ve dizin `fsync`'i) kapsam dışıdır: SIGKILL sayfa önbelleğini kaybettirmez.

### 3.3 Enjeksiyon (yalnız test)
`ROOTMARK_TEST_ABORT_AT` — yalnız `index/src/store.rs` `install`'da okunur; değerler `install:before_swap` (1–2 arası; "silmeden önce"), `install:swapped` (2–3 arası; "silme ile yeniden adlandırma arasında"), `install:installed` (3–4 arası; "yeniden adlandırmadan sonra"). Eşleşen adımda stderr'e `rootmark: test setting ROOTMARK_TEST_ABORT_AT=<v>: aborting (not a user setting)` yazılır ve süreç `std::process::abort()` ile biter (SIGABRT; dosya durumu SIGKILL'inkiyle aynı: temizlik yok). Değişken yoksa ya da boşsa hiçbir şey değişmez (ürün varsayılanı); tanınmayan bir değer `install`'ı hatayla durdurur (sessiz değil). Emsal: `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` (PQ1-4). Kurulumun iki yolu da sınanır: `rootmark index build` (CLI) ve kapanış sıkıştırması (`compact (shutdown)`).

## 4. PQ3-6 — yalnız `NotFound` yokluktur (R3)
Üç yer ve izleyicinin toplu işi:
- `Kernel::watch_removed`: `symlink_metadata(path)` — `Ok` → `None` (bugünkü gibi); `Err(NotFound)` → kaldırma; öteki `Err` → hata (`<doc>: not read (<e>); not taken as a removal`), `ingest_removal` bunu bir not satırı olarak yazar (`watch: <doc>: removal: error: …`); doküman canlı kalır.
- `Kernel::reconcile_files`'ın eksik listesi ve `Kernel::remove_missing_under`: yalnız `NotFound` eksiktir; öteki hatalar not (`<doc>: not read (<e>); not taken as a removal`) — uzlaştırmada özet satırının `errors` sayısına ve `watch: …` satırına, `ingest-dir`'de stderr'e (`rootmark ingest-dir: <doc>: …`).
- `serve.rs` toplu işi (`symlink_metadata(&p).is_err()` → `removed`) ve `targets_of`'un `Gone`'u: yalnız `NotFound`; öteki hata not satırı.
Test (`server/tests/watch_noaccess.rs`): `chmod 000 V/sub` (sunucu kapalıyken, sonra başlatma; ve sunucu çalışırken); bekleme × 2 sonra `sub/k.md`'nin blokları canlı, stderr'de not satırı; `chmod 755` sonra yeniden başlatma: yeni olay 0. `ingest-dir` ile de. Kök (uid 0) izni yok saydığından test kök olarak koşulursa hata verir (yeniden üretilemedi, sessiz geçmez); Linux koşuları kök olmayan kullanıcıyla.

## 5. Düzenekler (R3)
- **PQ3-8 port:** `restartkit` ve `watchkit` `--http 127.0.0.1:0` ile başlatır ve portu sunucunun `rootmark serve: http listening on 127.0.0.1:<port>` satırından okur (`serve_watch.rs`'in yolu). `free_port` kalkar. İddialar değişmez.
- **PQ3-4 C1'in zorunlu SIGKILL yolu:** yeni test ayarı `ROOTMARK_WATCH_TEST_STOP_DELAY_MS` (`server/src/serve.rs`): verilirse izleyici durdurulurken bekleyen kaldırmaları almadan önce o kadar bekler (yük altında yavaş bir durdurmanın yerine geçer); başlangıçta stderr'e bildirilir, sayı değilse hata. Yeni dosya `server/tests/watch_restart_forced.rs`: C1'in adımları aynen, ilk sunucu bu ayarla (2000 ms) başlar → 0,5 s'de süreç her tekrarda yaşar → SIGKILL her tekrarda gönderilir (`sigkill_sent` = tekrar sayısı, iki platformda) → yeniden başlatmadan sonra `n0.md` kapalı, tam bir kez `BlockDeleted`, aramada yok. `restartkit`'e yalnız alt sürece ortam değişkeni verme yolu eklenir (SIGKILL yolu). C1 (`watch_restart.rs`) aynen kalır.

## 6. Ürün sınırı
Değişen: `index/src/store.rs` (`install`, `recover_install`, `incomplete`; enjeksiyon `install`'da), `server/src/tools.rs` (`start_index`, `IndexAtStart`'ın yeni dalları, `watch_removed`, `reconcile_files`, `remove_missing_under`), `server/src/serve.rs` (başlangıç iletileri, toplu işin `NotFound`'u, durdurma test ayarı), `server/src/bin/rootmark.rs` (yalnız `ingest-dir`'in not satırı gerekirse). Yeni testler yeni dosyalarda: `hot_tear_restart.rs`, `index_install_kill.rs`, `watch_noaccess.rs`, `watch_restart_forced.rs`. Düzenekler: `restartkit/mod.rs`, `watchkit/mod.rs` yalnız port ve ortam değişkeni. Değişmeyen: `core`, `bindings`, `clients/*/src`, spec, fixture'lar, `docs/CLIENT_CONTRACT.md`, dizin biçimi, `watch_restart.rs` ve öteki mevcut testler.

## 7. Kademe 2, Mac kümesi ve D-NOTES'ta değişmesi beklenen kayıtlar (önceden, adıyla; K-P2Q1.4)
Karşılaştırma tabanı: kademe 2'de P2-Q.3'ün tetiklemesi `43319749692`; Mac kümesinde `reports/rounds/P2-Q.3/mac_set/`; D-NOTES'ta 0.3.3'ün kayıtları (`reports/rounds/P2-Q.3/pilot/{mac,docker,ci}`). Liste dışındaki her değişiklik **adsız fark**tır.
1. **Sürüm dizgileri (0.3.3 → 0.3.4):** `status.mdk_version`, MCP `serverInfo.version` ve `instructions`, `rootmark --version`, eklenti ve VS Code iskeletinin `version`'ı, `release_check` R1 değerleri; D-NOTES'ta sürüm dizgisi taşıyan alanlar. Tur içinde CHANGELOG başlığı "## Unreleased — 0.3.4".
2. **E1-4 / R14 (etikete bağlı):** `v0.3.4` tur içinde atılınca `release_check` R14 ve Mac kümesinin E1-4 kaydı farklı okunabilir.
3. **CI yapılandırması (`.github/workflows/**`): değişiklik planlanmadı.** Yeni testler `make test` ile macOS ve `ubuntu-latest` işlerinde koşar. İşin süresi gerekli kılarsa CI'daki tekrar sayısı değişiklikten önce TESTS.md'ye ekle yazılır; yapılandırma değişirse bu listeye önce bir DESIGN ekiyle.
4. **Başlangıç denetimi:** zincirin her `serve` başlangıcında dizin günlükle tutarlıdır (vault `ingest-dir` + `index build` ile hazırlanır) → `Current`, yeni satır yok, yeniden kurma yok. D kayıtlarında değişiklik **beklenmez**; görülürse adsız farktır.
5. **Atomik kurulum:** kesintisiz koşuda `index.old` hiç kalmaz; kurulmuş dizinin baytları aynı → D kayıtlarında değişiklik yok.
6. **Z ve O kayıtları (Mac kümesi):** değerler tekrar tekrar değişir; başlangıç denetimi sunucunun ilk saniyesine dizinin okunmasını ekler (küçük vault'ta milisaniyeler). Hüküm P2-V.2 okumasıyla.
7. **Bağımlılık kayıtları:** yeni bağımlılık yok; `check_dep_age`, `check_deps` değişmez.
8. **Kademe 1 (FAST):** yalnız 1. madde.

## 8. Ön-kayıtlı testler
`progress/TESTS.md`.

## Ek (R3, koddan sonra; Linux bulgusu PQ4-3)
Son ağacın (`a0f5125`) Linux koşusunda (kök olmayan kullanıcı) N1 20/20 kaldı: kilitli bir alt dizin varken `serve --watch` hiç başlamadı (`error: watch <vault>: Permission denied`). Neden: `notify`'ın inotify arka ucu özyineli kayıtta her alt dizine ayrı izleyici ekler ve birinde `EACCES` alınca bütün kaydı düşürür (macOS FSEvents kökü tek izler, etkilenmez). 0.3.3'te de aynıdır (kayıt yolu değişmedi); sessiz değil, ama N1'in iki platformdaki beklentisini (başlangıç, not satırı, bloklar canlı) karşılamaz. Düzeltme (`server/src/serve.rs`, §4'ün kapsamı): özyineli kayıt yalnız izin hatasıyla düşerse kökten başlayarak okunabilen her dizin özyinesiz kaydedilir (`watch_readable`), okunamayan dizin için `watch: <dir>/: not read (…); not watched` notu yazılır, başlangıçta bir satır durumu söyler; bu kipte toplu işin yeni dizinleri (`Target::Scan`) de kaydedilir; sonradan okunur olan dizin bir sonraki başlangıçta izlenir. Normal yol (tek özyineli kayıt) aynen kalır; kök okunamıyorsa başlangıç yine hata verir. Kayıt: `reports/rounds/P2-Q.4/r3/final_a0f5125_{linux,macos}.jsonl`.

## Düzeltme eki (R5; gözden geçirici N1)
R0'ın kalış sayısı yanlış toplanmıştı; metin değiştirilmedi, düzeltme burada. Kayıtlar (`reports/rounds/P2-Q.4/r0/repro_0.3.3_{macos,linux}.jsonl`): macOS 1 183/1 200 (17 kalış), Linux 1 193/1 200 (7 kalış) → **24 kalış, 22 değil**. A2'de 7 (sınıf 1: 5, sınıf 2: 2), C1'de 17 (macOS 14, Linux 3). §1.2'deki "kalan öteki 17 tekrar (A2'de 2, C1'de 15)" doğrusu 19 (A2 2, C1 17); §1.3'teki "(C1: 2) … (C1: 13)" doğrusu C1 2 + 15. Sınıflandırma (her kalış dizin–günlük tutarsızlığı) ve kök neden değişmez.
