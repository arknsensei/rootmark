# P1-E.1 — Bağımsız Doğrulama

- Yönerge: `docs/rounds/P1-E.1/VERIFY.md`, `docs/DEV_PROCESS.md` §5.
- Doğrulanan commit: `104bd8e` (READY.md). READY'nin andığı sha `2842f6e`, resmi RUN'ın ölçtüğü commit `7ea12dc`. `7ea12dc..104bd8e` arası yalnız RUN kaydı, FINAL ve ilerleme kayıtları; `release_check` R4a "inputs of 104bd8e = HEAD" ile paket girdileri aynı.
- Ortam:
  - temiz klon (oturum scratchpad'i, ana depodan `git clone`); `p1d-final` için ayrı ikinci klon;
  - macOS (Darwin 27.0.0, arm64), rustc 1.92.0, Node 24.3.0, Python 3.13.5, uv 0.11.8, Docker 29.8.0;
  - model `minilm-l6-v2/e73168f9` (`make models`: sha256 + blake3 manifest ok);
  - Obsidian 1.13.4, `/Applications/Obsidian.app`.
- Kendi RUN'larım (HEAD `104bd8e`): A `20261017T150959Z-104bd8e`, B `20261017T154011Z-104bd8e` (`scripts/verify.sh pair <klon> A B`; zincirler 1 856 s ve 1 719 s).
- Süreç notu:
  - **Bağımsızlık sapması (açıkça kayıt):** klonu kurmadan önce ana dizinde `VERIFY.md`, `DEV_PROCESS.md` §5 (`awk`), `GOAL.md` ve `progress/READY.md` okundu, `docs/rounds/P1-E.1/` listelendi, `git remote`/`git tag` sorgulandı. Sonunda etiket denetimi ve önceki doğrulama commit'inin biçimi için `git tag -l 'v*'` ve `git log` koşturuldu. Ana dizinden kod ya da RUN okunmadı. FINAL, P1-E doğrulaması, kod ve RUN'lar klonda okundu. Önceki doğrulamalar da aynı ön okumayı yapmıştı. §5'in harfinden bir sapmadır, hükmü etkilediğini düşünmüyorum.
  - Deneyler için kendi istemcilerimi yazdım, depo kodu kullanılmadı: Python (stdlib) MCP stdio + HTTP yoklayıcısı; Node (yerleşik `WebSocket`) gerçek Obsidian'ı CDP ile süren sürücü.
  - Gerçek Obsidian ayrı örneklerde koşturuldu: kendi `--user-data-dir`'leri scratchpad'de, CDP portları 9341/9342. Kullanıcının açık Obsidian'ı ve `~/Library/Application Support/obsidian/obsidian.json` dokunulmadı (değişiklik zamanı önce ve sonra `1792152358`).
  - Docker yalnız `scripts/release.sh`'in kendi adlı `--rm` kaplarıyla kullanıldı.
  - Kasıtlı bozmalar ayrı bir klonda (`vt`) yapıldı ve geri alındı (`git status` temiz).
  - Ana dizine yazılan tek dosya bu rapor.

## V1 · Çift koşturma (E1-7) — **geçti**
- **`verify.sh pair` çıkış 0.** `pair_compare`: **103 189 alan, fark 0** (resmi kayıtla aynı sayı), `bytes_equal: true`.
- İki zincirin de FAIL özeti: "10 historical FAIL(s) not counted; counted FAILs 0". İki RUN'da `p1e1_checks.json` E1-1..E1-9 ve `tamper` geçti.
- **Resmi RUN ile karşılaştırma** (`20261017T140538Z-7ea12dc`, klondaki commit'lenmiş hali):
  - `p1e1_checks.json`, `p1e_checks.json`, `regress_vs_p1e.json` **bayt eşit**;
  - karşılaştırıcıların (`compare_p1d/p1d1/p1e/p1e1`) bayt listelerinde adıyla geçen ve resmi RUN'da commit'lenmiş **16 yapıt bayt eşit**;
  - `p1d/p1d1/p1e/p1e1` altındaki 379 dosyadan 16'sı farklı. Hepsi bildirilmiş belirlenimsiz alan: duvar saati (D-10 gecikmeleri, D1-5/D1-6 süreleri), mutlak yol (Inspector `source`/`command`), `server.token`, günlükler, `release.json` `commit` ve `targets/0/sha256`, `release_check.json` commit satırı.
  - Sınır: resmi RUN'da 8 bayt yapıtı commit'lenmemiş (ör. `p1d/mcp_e2e/run1/transcript.jsonl`), bu yüzden `compare-p1e1 <resmi> <A>` "missing" ile çıkış 1 verir. Bu yapıtlar yalnız A ↔ B'de karşılaştırıldı (eşit).
- **Kasıtlı bozma (benim):** A'nın kopyası ↔ B:
  - bozmasız → çıkış 0;
  - `access_log_real.json`'da bir yol (`/v1/trace` → `/v1/tracx`) → çıkış 3;
  - `p1e1_checks.json` `E1-1.pass` → `false` → çıkış 3;
  - aynı dosyaya JSON'u bozan 1 bayt → çıkış 1 (hata).

## V2 · `status` (E1-1, E1-2) — **geçti; gerçek Obsidian'da `stale` ve `damaged`**
- **Eski indeks, fixture'sız:** `p1d-final` etiketinden kendi derlediğim `mdk` ile kendi vault'umda (mini_vault'un 12 belgesi + kendi notum) `ingest-dir` + `index build` → manifest `mdkernel-index/2`, 35 kayıt.
- Ayrıca `MDK_OLD=<benim ikilim> scripts/gen_old_index.sh --check` → "fixture up to date": commit'lenmiş `server/tests/fixtures/index_v2` benim ikilimin ürettiğiyle bayt eşit.
- Güncel `mdk serve --mcp stdio --http 127.0.0.1:0`, kendi yoklayıcım:

  | Durum | `status` GET / POST / MCP | `index` | `search` HTTP / MCP | `blame`, `checkpoints`, `trace`, `history` |
  |---|---|---|---|---|
  | `/2`, `auto_rebuild=false` | 200 / 200 / ok | `stale`, `format /2`, `expected_format /3`, `auto_rebuild false` | **503 `index_stale`** / `index_stale` | 200 |
  | `/2`, `auto_rebuild=true` | 200 / 200 / ok | `ok` (yeniden kuruldu, 35 kayıt) | 200 (10 hit) / ok | 200 |
  | bozuk JSON manifest, `false` | 200 / — / ok | `damaged`, `format null` | 503 `index_stale` / `index_stale` | 200 |
  | bozuk JSON manifest, `true` | 200 / — / ok | `damaged` (kendiliğinden kurulmadı) | 503 `index_stale` | 200 |
  | indeks yok, `false` | 200 | `ok` (sözleşme §3: "indeks yoksa kurar") | 200 | 200 |

  Sunucu `false` iken `/2` manifest'e dokunmadı (biçim sonra da `/2`). Bozuk manifest `mdk index build` ile kurtarıldı (`/3`).
  E1-1 8/8, E1-2 2/2. §3 metni ("`status` her indeks durumunda 200 … `index.state`") ile davranış birebir.
- **Gerçek Obsidian bağlantı ekranı (kendi CDP sürücüm, harness'ten bağımsız):** ayrı örnek, eklenti paketi kurulu, `mdk serve --http` ve `auto_rebuild=false`.
  - **`damaged`:** durum çubuğu `mdk 0.1.0 · human:editor · index damaged`, `mdk-index-warning` sınıfı. İpucu ve bildirim: "index damaged: its manifest cannot be read; search is unavailable (index_stale) until then; blame, history, checkpoints and traces work; rebuild: run `mdk index build <vault>` (a damaged index is never rebuilt automatically)".
  - **`stale`:** `… · index stale`, ipucunda biçimler (`/2` → `/3`), iki yeniden kurma yolu (`mdk index build`, ya da `auto_rebuild = true` + yeniden başlatma).
  - İkisinde de çalışan sunucuya `mdk index build` ve eklentinin "Reconnect" komutu → `mdk 0.1.0 · human:editor`, uyarı yok. Aynı süreçte `search` 200.
- Gözlem: `auto_rebuild=false` ile `stale` ya da `damaged` indekste `mdk serve` kapanışta `error: compact (shutdown): index format …` yazar ve **çıkış 1** verir (P1-E V8'de de görülmüştü). Sessiz değil, sözleşme çıkış kodunu tanımlamıyor; ama servis yöneticisi altında her kapanış hata gibi görünür.

## V3 · Gerçek Obsidian e2e (E1-3) — **geçti, 11/11**
- `scripts/plugin_e2e_real.sh` (`clients/obsidian/e2e/real/`) klonda **iki kez** ve `release.sh` içinde bir kez daha: **11/11, E1-9 true, E-3 true, E-4 57 istekte sözleşme dışı 0**, her koşu yaklaşık 10 s.
- İki koşunun `plugin_e2e_real.json` ve `access_log.json`'ı birbirleriyle, resmi RUN'la ve zincir A'nınkiyle **bayt eşit**.
- E-3: md değişen tek adım 3 (`agent/scene.md`). Uçlar: `GET /v1/status`, `POST /v1/{blame, checkpoint, checkpoints, promote, search, trace}`.
- **P1-E'deki kendi CDP yöntemimle farklar:**
  - aynı: ayrı `--user-data-dir`, önceden yazılmış `obsidian.json`, kısıtlı kipten API ile çıkış (`setEnable`, `enablePlugin`), kaydedici vekil, ajanın aynı `mdk serve` sürecinden MCP ile konuşması;
  - harness düğmeleri ve girişleri DOM üzerinden sürer (`button(...).click()`, alan doldurma), adım 10/11'de kenar çubuğunu eklentinin `refreshBlame()` API'siyle tazeler. Ben bunu düzenleyici olayına bırakmıştım; ölçüme etkisi yok;
  - istek sayısı 57 (benim P1-E koşumda 68): akış daha kısa ve belirlenimli, uç kümesi aynı;
  - harness'in E1-9 adımı yalnız `stale`'i sınar. `damaged` harness'te yok, V2'de kendi sürücümle sınadım (geçti);
  - harness sonuçları belirlenimli (zaman damgası, port, token, mutlak yol yok), bu yüzden bayt karşılaştırılabiliyor. P1-E'deki benim sürücüm öyle değildi.
- Kullanıcının Obsidian'ı açıktı, etkilenmedi.

## V4 · Yayım (E1-4, E1-5) — **geçti**
- **`release.yml`:** `on:` altında yalnız `workflow_dispatch:`. Matris `aarch64-apple-darwin` (`required: true`), `x86_64-unknown-linux-gnu` ve `x86_64-pc-windows-msvc` (`continue-on-error: ${{ !matrix.required }}`); `x86_64-apple-darwin` yok. Her iş `release_target.sh check` (`--version`, `--spec-version`, `mdk conformance`), son iş `dist/release.json`'u toplar, kaydı olmayan hedefi `skipped` yazar.
- **Yerel derleme** (`cargo build --release --locked --target aarch64-apple-darwin -p mdkernel-server --bin mdk`, CI ile aynı):
  - Mach-O arm64, yalnız sistem kütüphaneleri (`otool -L`);
  - `mdk 0.1.0`, `1.0.3`;
  - `mdk conformance` → **level 3, 34/34**, `fixtures_manifest: ok`.
- **`scripts/release.sh` (kendi `dist/`'im), çıkış 0:**
  - `aarch64-apple-darwin` ok (zorunlu);
  - `x86_64-unknown-linux-gnu` ok (CI bilgi; kapta 34/34);
  - `x86_64-pc-windows-msvc` `skipped`, nedeni yazılı ("no MSVC toolchain or Windows host here; …");
  - `release.json`: `version`, `spec_version`, `commit 104bd8e`, `worktree_dirty false`, `required_targets [aarch64-apple-darwin]`, `required_ok true`; her yapılı hedefte `sha256`, `version`, `spec_version`, `conformance {level_claimed 3, 34/34, manifest ok}`, `conformance_exit 0`. Biçim RELEASE.md R4 ile tutarlı.
  - Not: yayım ikilisinin hash'i klon dizinine bağlı (A ve B'de farklı, `DETERMINISTIC_FIELDS` `ortam` satırı). İkili bayt düzeyinde yeniden üretilebilir değil; RELEASE.md insan adımı 3 bunu söylüyor.
- **`release_check.sh`: art arda 5 koşuda 16/16, çıkış 0; R7 her seferinde ok** (SIGPIPE düzeltmesi tuttu).
- **Kasıtlı bozma (benim):**
  - wheel'den `mdkernel_core` çıkarıldı → R6 ve R7 FAIL (14/16);
  - `plugin_e2e_real.json` `skipped` → R15 FAIL;
  - `release.yml`'a `push: tags` eklendi → R13 FAIL;
  - Linux/Windows `required: true` → R13 FAIL.
- Gözlem: `release.yml` paketleme adımında `cp … 2>/dev/null || true` var. İkili yoksa önceki "check" adımı zaten düşer, yanlış yayım üretmez.
- CI artefaktı yok: `release.yml` tetiklenmemiş (insan adımı). CI hash'lerinin karşılaştırılması doğrulamanın kapsamı dışında.

## V5 · Regresyon ve uyumluluk (E1-6) — **geçti**
- `regress_vs_p1e` (A ve B): **101 929 alan, fark 0**, referans P1-E resmi RUN; uyumluluk 34/34, `differing_tests: []`. Hariç 29, kullanılmayan 0. Resmi RUN'ın dosyasıyla bayt eşit.
- 29 hariç tutmanın hepsi bir karara bağlı:
  - K-E1.2: P1-D MCP e2e'nin eski `index == "present"` beklentisi ve bunu gören D-2, D1-12, E-9 kayıtları;
  - F2 (E1-8): `d1_13` yeni alanlar;
  - F5/F6: alan denetimi;
  - K-E1.2 (E1.2): eklenti paketi boyutu, vitest 97 → 100.

  Geliştirme RUN'ı görüldükten sonra, resmi RUN'dan önce yazıldı. E1.5.md'de ve READY'de açık; K-C1.6 ve P1-E E7 emsali.
- `check_frozen.sh` OK (tablo yalnız ekleme). `check_deps.sh` OK. `test_check_deps.sh` OK.
- Dondurma istisnaları F1–F8'in hepsinde girdi, önce/sonra ve gözden geçirici onayı var. Hepsi `FROZEN.md`'ye eklenen satırlarla eşleşiyor.

## V6 · Sözleşme (E1-8, E1-9) — **geçti; bir kapsam boşluğu**
- `contract_check.sh` temiz: `pass: true`, eksik 0.
- HTTP adımı `status`'u dört indeks durumunda sınar (`ok`, `stale` biçimi yaşlandırılmış, `damaged`, `missing`): GET ve POST 200, `index.state`, §2 satırının `index {…}` alanları, `search` 503.
- §3 metni ile davranış birebir (V2 tablosu).
- **Kasıtlı bozma (benim, sunucu kaynağının kopyasında, `MDK_BIN` ile):**

  | Bozma | `contract_check` |
  |---|---|
  | `index.expected_format` kaldırıldı | çıkış 1, `http-status-index:{ok,stale,damaged,missing}` |
  | `index.auto_rebuild` kaldırıldı | çıkış 1, aynı dört durum |
  | `status` indeks `ok` değilse hata (P1-E ihlalinin geri getirilmesi) | çıkış 1, `http-status-index:{stale,damaged,missing}` |
  | **üst düzey `last_checkpoint` kaldırıldı** | **çıkış 0 (yakalanmadı)** |

  Üst düzey alan kaybını `cargo test -p mdkernel-server` de yakalamıyor (hepsi geçti). Yalnız gerçek Obsidian e2e yakalıyor (0/11): eklentinin `shape.ts`'i `status.last_checkpoint: missing` der. Bu adım CI'da atlanır. E1-8'in istediği ("`status`'un yeni alanı") karşılanıyor. VERIFY'ın "bir `status` alanı kaldırılınca kapı kırmızı" maddesi `index` alanları için tam, üst düzey alanlar için yalnız yerel gerçek e2e ile.
- E1-9: §3 ve §2.5 güncel (`index_stale` satırı "`status` hiçbir indeks durumunda dönmez"; `status.index` tanımı). Bağlantı ekranı `stale` ve `damaged`'i ve yeniden kurma yolunu gösteriyor (V2, V3).
- Gözlem: `status.index` dizeden (`"absent"`/`"present"`) nesneye geçti ve `spec_version` 1.0.3'te kaldı. K-E1.2 bunu "`status` dışında değişiklik değil" diye kaydeder. 0.1.0 öncesi dış istemci yok, ama sözleşme biçim değişikliği sürüm alanına yansımıyor.

## V7 · Belgeler — **geçti; bir ifade yanlış rapora işaret ediyor**
- `git diff p1e-final HEAD -- NORTH_STAR.md docs/ARCHITECTURE.md`: yalnız kanıt durumu (P1-E doğrulamasının hükmü, P1-D.1 koşullarının V0 teyidi), e2e tanımı (K-E1.3: "başsız kip yok; ayrı örnek CDP ile") ve yayım hedefi satırı (K-E1.1). Vaatler, ilkeler ve kararlar değişmedi.
- `RELEASE.md` K-E1.1 ile tutarlı: tek hedef `aarch64-apple-darwin`; Linux/Windows CI bilgi; Intel macOS hedef değil; R4, R13, R15 ve insan adımı 3.
- **Tutarsızlık:** `RELEASE.md` insan adımı 1 hâlâ "P1-E bağımsız doğrulaması (`reports/rounds/P1-E/VERIFICATION.md`) **"Güvenilir"** demeden hiçbiri yapılmaz" diyor. P1-E'nin hükmü "şu koşullarla güvenilir" idi; GOAL P1-E.1'in karar kuralı bu doğrulamayı arar. Harfiyen okunursa insan adımı yanlış rapora bağlı.
- **Tarihsel FAIL etiketleri:** 10 etiketin hepsi kararlara bağlı.
  - 8'i `historical_fails.json`'da, kayıt P1-E resmi RUN'ınkine eşit: B1-3, C1-7, C-3, C-11, D-2, D-7, D-9, E-7.
  - **D1-12 ve E-9 bu turda yeni FAIL'dir.** "Tarihsel" etiketini `compare_p1e1.EXCLUDED`'ın K-E1.2/F2 alt ağaçları üzerinden alırlar. Neden kasıtlı sözleşme değişikliği (`status.index` nesnesi), mekanizma açık (F7, E1.5.md), ama K-E1.4'ün adını verdiği "bilinen FAIL" kümesinde değiller.
  - Kasıtlı bozma (resmi RUN'ın kopyasında, `eval.historical`):
    - E-9'a bildirilmemiş bir yaprak → sayılır (çıkış 3);
    - D-7 kaydına bir alan → sayılır;
    - E1-3 `pass=false` → sayılır;
    - bildirilmiş bir yolun (`E-9 regress_vs_p1d1.equal`) değeri değişti → tarihsel kalır.

    Sınır: bildirilen alt ağaç her değeri kabul eder.

## V8 · Karar kuralı — **FINAL'le aynı sonuç**
- **Benim değerlerim:**
  - E1-1 8/8, E1-2 2/2 (V2, kendi eski indeksim);
  - E1-3 11/11 (V3);
  - E1-4 geçer (R7 5/5, 16/16);
  - E1-5 1/1 zorunlu, Linux ok, Windows atlandı (V4);
  - E1-6 fark 0, 34/34 (V5);
  - E1-7 fark 0 (V1);
  - E1-8 geçer (V6);
  - E1-9 1/1 (V2, V3).

  **E1-1..E1-9 geçti.**
- GOAL'ün kuralı: "E1-1..E1-9 geçti ∧ doğrulama 'Güvenilir' → insan adımı: `git tag v0.1.0`, `release.yml` tetikleme, `RELEASE.md` kontrol listesi. Faz 1 kapanır." Tanımlanmamış kalış yok. FINAL de "E1-1..E1-9 geçti → bağımsız doğrulama 'Güvenilir' derse insan adımı" diyor. **Aynı.**
- **`v0.1.0` etiketi atılmamış:** klonda ve ana depoda `git tag -l 'v*'` yalnız `v2-final`, `v3-final`; `release_check` R14 ok; `release.yml` tetiklenmemiş (CI artefaktı yok).

## Sonuç

**İhlal / fark / şüphe:**
1. **Kapsam boşluğu (V6):** `contract_check.sh` ve Rust testleri `status`'un üst düzey alanlarından birinin (`last_checkpoint`) kaybını yakalamıyor. Yalnız yerel gerçek Obsidian e2e yakalıyor, o da CI'da atlanıyor. `index` alanları ve 503 istisnası tam kapsanıyor.
2. **Belge (V7):** `RELEASE.md` insan adımı 1 P1-E doğrulamasına işaret ediyor, P1-E.1'e değil.
3. **Gözlem (V2):** `auto_rebuild=false` + `stale`/`damaged` indeksle `mdk serve` her kapanışta `compact (shutdown)` hatası yazar ve çıkış 1 verir.
4. **Gözlem (V7, V5):** D1-12 ve E-9 bu turda yeni FAIL'dir; "tarihsel" etiketini geliştirme RUN'ından sonra yazılan hariçlerle alırlar. Kasıtlı ve açıkça kayıtlı; bildirilen alt ağaç içindeki değer değişikliği görünmez.
5. **Gözlem (V1):** resmi RUN dizininde 8 bayt yapıtı commit'lenmemiş. Resmi RUN ile bayt karşılaştırması commit'lenmiş yapıtlarla sınırlı (hepsi eşit).

Hiçbiri bir çıkış kriterini ya da sözleşme metniyle davranış eşitliğini bozmuyor. P1-E doğrulamasının koşulu (`status` ihlali) gerçek eski indeksle, HTTP/MCP'de ve gerçek arayüzde kapandı.

**Hüküm: Güvenilir.**
- FINAL'in her satırı kendi ortamımda yeniden üretildi:
  - E1-7: 103 189 alan fark 0, yapıtlar bayt eşit; P1-E.1 kayıtları resmi RUN'la bayt eşit;
  - E1-6: 101 929 alan fark 0;
  - E1-1/E1-2: kendi derlediğim `p1d-final` indeksiyle;
  - E1-3/E1-9: gerçek Obsidian'da, harness'le ve kendi sürücümle;
  - E1-4/E1-5: kendi yayım paketimle.
- Karar kuralı bağımsız türetildi ve FINAL'le aynı.

**Öneriler (≤ 3):**
1. `RELEASE.md` insan adımı 1'i `reports/rounds/P1-E.1/VERIFICATION.md`'ye bağlayın (etiketten önce, tek satırlık belge düzeltmesi).
2. `contract_check.sh`'in `status` adımı §2 satırındaki **bütün** üst düzey alanları da denetlesin. Böylece CI'da (gerçek e2e atlanırken) alan kaybı kırmızı olur.
3. Bayat ya da bozuk indekste kapanış sıkıştırmasını bir notla atlayın (çıkış 0) ya da çıkış kodunu sözleşmeye yazın. Resmi RUN'ların bayt yapıtlarını commit'leyin ya da hash'lerini kaydedin.

**0.1.0 (`aarch64-apple-darwin`): yayımlanabilir.**
