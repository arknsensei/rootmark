# P1-E — Bağımsız Doğrulama

- Yönerge: `docs/rounds/P1-E/VERIFY.md` (+ `ADDENDUM.md` V0/V5/V8/V9 ekleri), `docs/DEV_PROCESS.md` §5.
- Doğrulanan commit: `60febb2` (READY.md). READY'nin andığı sha `da9d64a`, resmi RUN'ın ölçtüğü commit `632f29e`. `632f29e..60febb2` arası yalnız rapor üreteci, testler ve kayıtlar (READY'de yazılı; R4a paket girdileri HEAD ile aynı).
- Ortam:
  - temiz klon (oturum scratchpad'i, ana depodan `git clone`);
  - macOS (Darwin 27.0.0, arm64), rustc 1.92.0, Node 24.3.0, Python 3.13.5, uv 0.11.8, Docker 29.8.0;
  - model `minilm-l6-v2/e73168f9` (`make models`: sha256 + blake3 manifest ok);
  - Obsidian 1.13.4 (Electron 43.1.1), `/Applications/Obsidian.app`.
- Kendi RUN'larım (HEAD `60febb2`):
  - A `20261017T091317Z-60febb2`, B `20261017T094039Z-60febb2` (`scripts/verify.sh pair <klon> A B`; zincirler 1 673 s ve 1 611 s);
  - C `20261017T100834Z-60febb2` (`verify.sh chain`, üçüncü koşturma; V0'la eşzamanlı, yük altında, 1 900 s);
  - V0 (commit `06cd370`): A0 `20261017T100807Z-06cd370`, B0 `20261017T103843Z-06cd370` (`verify.sh pair`, iki temiz klon, kaynak `06cd370`; zincirler 1 841 s ve 1 563 s).
- Süreç notu:
  - **Bağımsızlık sapması (açıkça kayıt):** VERIFY "ana dizine erişme" diyor. Klonu kurmadan önce ana dizinde şunlar yapıldı:
    - `cat docs/rounds/P1-E/VERIFY.md`, `GOAL.md`, `ADDENDUM.md`, `progress/READY.md`, `DEV_PROCESS.md` §5 (`awk`);
    - `ls docs/rounds/P1-E/ docs/rounds/P1-E/progress/ reports/rounds/…`;
    - P1-D.1 `VERIFICATION.md`'nin ilk 60 satırı (rapor biçimi için) ve `git log` ile önceki doğrulama commit'leri.

    Ana dizinden kod ya da RUN okunmadı. Bütün koşturmalar klonda ve kendi çalışma dizinlerimde yapıldı. Önceki iki doğrulama da aynı ön okumayı yapmıştı. Yine de bu, §5'in harfinden bir sapmadır. Hükmü etkilemediğini düşünüyorum, çünkü READY'nin her iddiası klondaki dosyalarla ve kendi koşturmalarımla yeniden sınandı.
  - Deneyler için kendi istemcilerimi yazdım. Depo kodu kullanılmadı:
    - Python (stdlib): MCP stdio JSON-RPC + HTTP;
    - Node (yerleşik `WebSocket`): gerçek Obsidian'ı Chrome DevTools Protocol ile süren sürücü.
  - Gerçek Obsidian **ayrı bir örnek** olarak koşturuldu:
    - kendi `--user-data-dir`'i scratchpad'de, CDP portu 9334;
    - kullanıcının açık Obsidian'ı ve `~/Library/Application Support/obsidian` dokunulmadı (dosya tarihleri değişmedi);
    - yalnız başlattığım süreç durduruldu.
  - Docker yalnız `scripts/release.sh`'in kendi adlı `--rm` kaplarıyla kullanıldı. Sonunda `mdk-release*` kabı kalmadı.
  - Ana dizine yazılan tek dosya bu rapor.

## V0 · P1-D.1 yeniden koşturma ve belirlenimli alanlar (E-12, E-13) — **geçti**
- **P1-D.1 `pair_recheck`'in yeniden üretimi:** `scripts/verify.sh pair` (commit `06cd370`, iki temiz klon) **çıkış 0**.
  - `pair_compare`: **100 982 alan, fark 0** (E0'ın kaydıyla aynı sayı); `bytes_equal: true`.
  - İki RUN'da da `p1d1_checks.json` D1-1..D1-13 geçti.
  - D1-5: `write_responses_a_vs_b_equal: true`, `responses_a_vs_b_equal: false` (bilgi alanı), `propose_denied: 0` (K-E0.4). E0 kaydıyla aynı.
- Resmi yeniden koşturmanın `p1d1_checks.json`'u ile karşılaştırma: 157 yaprak, tek fark `D1-6/measured/ratio`. Bu alan `DETERMINISTIC_FIELDS.md`'de duvar-saati olarak bildirilmiş (satır 79–81).
- E-12 ve E-13 geçti.
- **`DETERMINISTIC_FIELDS.md`, 3 koşturma:** `check_compare_fields --runs A B C` çıkış 0:
  - 85 bildirilen alan = koddaki 85;
  - eşleşmeyen 0;
  - 9 modülün hepsinin bölümü var;
  - **101 958 karşılaştırılan yaprak üç RUN'da eşit**, bildirilen belirlenimsiz alanların hepsi mevcut.
- `compare-p1e`: A↔C ve B↔C çıkış 0. C yük altında koştu (V0 çifti ve yayım derlemesiyle eşzamanlı), yine de fark 0.

## V1 · Çift koşturma (E-8) — **geçti**
- **`verify.sh pair` çıkış 0.**
  - `pair_compare`: **101 958 alan, fark 0** (resmi RUN'la aynı alan sayısı).
  - `bytes_equal: true`: P1-D, P1-D.1 ve P1-E yapıt kümesinin hepsi eşit. Buna sahne ve plugin e2e vault'larının `events.jsonl`, `traces.jsonl` ve `proposals.jsonl`'ü, `access_log.json`, `scene.json` ve `plugin_e2e.json` dahil.
- **Resmi RUN ile karşılaştırma:** A'nın P1-E yapıtları resmi RUN `20261017T024341Z-632f29e`'nin yapıtlarıyla **bayt eşit** (9 dosya).
  - `p1e/scene/{scene.json, events.jsonl, traces.jsonl, proposals.jsonl}`;
  - `p1e/plugin_e2e/{plugin_e2e.json, access_log.json, events.jsonl, traces.jsonl}`;
  - `p1e_checks.json`.

  `p1c/p1d/p1d1/p1e_checks` sonuçları resmi RUN'la birebir aynı. Önceki turların bilinen "FAIL"ları da aynı: C-3, C-11, D-2, D-7 4/5, D-9; E7.md'de yazılı, dondurulmuş kaydediciler.
- **Kasıtlı bozma (benim):** A'nın kopyası ↔ B:
  - bozmasız → çıkış 0;
  - sahne `traces.jsonl`'ine 1 bayt → çıkış 3;
  - `access_log.json`'da bir yol (`/v1/trace` → `/v1/tracx`) → çıkış 3;
  - `p1e_checks.json` `E-3.pass` → `false` → çıkış 3;
  - geri yükleme → çıkış 0.
- Kalite alanları farkı 0. E-9'un yarısı olan regresyon için V8'e bakın.

## V2 · Sahne (E-1, E-6, E-11) — **geçti**
- **`scripts/scene.sh` iki kez** (`MDK_SCENE_KEEP=1`, temiz vault): **11/11**, E-3 `true`, çıkış 0.
  - İki koşu arasında `scene.json`, `events.jsonl`, `traces.jsonl` ve `proposals.jsonl` bayt eşit.
  - `diff -r` vault'ta tek fark `server.token`. Rastgele olması beklenir, kimliğe girmiyor.
  - Dört dosya resmi RUN'ınkiyle de bayt eşit.
- **Kendi 11 adımlı sahnem** (`scene.sh`'den bağımsız). Kurulum:
  - tek süreç `mdk serve --mcp stdio --http 127.0.0.1:0 --watch` (sözleşme §1'in önerdiği biçim; `scene.sh` ayrı süreçler kullanır);
  - başka vault: mini_vault + `NORTH_STAR.md` + `ARCHITECTURE.md`;
  - başka ajan: `verifier-agent/2.0`;
  - başka insan: `human_id = "verifier"`.

  **11/11, iki koşuda; `events.jsonl` ve `traces.jsonl` sha256'ları iki koşuda eşit.** Adımlar:
  1. status: HTTP ve MCP aynı spec sürümü (1.0.3), `human_only`, `append_only`.
  2. Önce `search`, sonra `checkpoint {name: "yesterday"}` (sözleşme §2.3'ün önerdiği sıra) → aynı checkpoint kimliği.
  3. Ajan yeni belge (`agent/notes.md`) ve **kendi başlığıyla** insan belgesine (`cupping.md`) ekleme yaptı.
     - Dosyaya kimlik ya da meta veri yazılmadı: içerik birebir ajanın metni.
     - md değişen yalnız bu iki dosya.
  4. blame: yeni blokların hepsi `proposal / Agent / verifier-agent/2.0`, `cupping.md`'de 2 proposal blok.
  5. Varsayılan arama 0 hit. `include_proposal` ile 2 proposal hit. Canonical kısmı bayrak kapalıyla eşit (boş).
  6. Promote:
     - gerekçesiz, yalnız boşluk gerekçe ve `reason` alanı yok → 400;
     - yanlış token → 401;
     - `X-Consumer` yok → 400;
     - ajanın MCP `promote`'u → `decision: denied`;
     - bunların hiçbiri olay yazmadı;
     - gerekçeli 3 promote → `Promoted`, `meta.actor {kind: human, id: verifier}`, `reason`, `from/to`.
  7. Arama buluyor, canonical. Ajanın insan belgesine eklediği bölüm terfi etmediği için görünmüyor.
  8. Son iz = 7. adımın izi. `returned` = hit'lerin soyları, terfi edilen soyları kapsar. `read_block` metinleri doğru.
  9. **Checkpoint'e dönüş, gerçekten eski hal.** Checkpoint'ten sonra insan `espresso.md`'yi sunucu dışında düzenledi (`Dose: 18 g` → `19 g`; `--watch` aldı).
     - `search at yesterday`: 0 hit, checkpoint kimliği 2. adımınki;
     - `blame at yesterday`: ajan belgesi `not_found`;
     - `espresso.md` blame'i checkpoint anındakine eşit;
     - `read_block at yesterday`: `Dose: 18 g`, şimdi `Dose: 19 g`.
  10. **Ajanın canonical saldırıları: 10/10 reddedildi.** Her durumda md sha256'ları, `events.jsonl` ve `proposals.jsonl` önce ve sonra aynı.

      | Saldırı | Sonuç |
      |---|---|
      | `replace_section` "Recipe" | `denied` |
      | `replace_section` H1 (bütün belge) | `denied` |
      | `replace_section` boş gövde (silme) | `denied` |
      | **terfi etmiş kendi bölümü** üzerinde `replace_section` | `denied` (artık canonical) |
      | başlıksız `append` (insan belgesine) | `policy_violation` + `hint: "start the appended text with a heading"` |
      | başlıksız `append` (terfi etmiş belgeye) | `policy_violation` + hint |
      | var olan belgeye `create` | `exists` |
      | harf farklı yol | `exists` |
      | `.mdkernel/` içine yazma | `path_outside_vault` |
      | `../` ile vault dışına | `path_outside_vault` |
  11. `espresso.md` blame ve arama 10. adımdan önceyle eşit.
- **Sözleşmeyle tutarlı iki gözlem** (ilk denememde kendi test tasarım hatalarımdan çıktı):
  - Başlıksız `append`, belgenin son bölümü ajanın **kendi proposal bölümü** ise kabul edilir. §4.1'e uygun: chunk canonical değil.
  - `--watch` olmadan sunucu dışı insan düzenlemesi bir sonraki `propose`'da `synced_external_events` olarak alınır (§2.1 son madde). Bu yüzden insan düzenlemesini 9. adımda görmek `--watch` ister.
- **Kasıtlı bozma (benim):** `scene.sh`'in kopyasında politika değiştirildi.
  - `[edit] policy = "unrestricted"` → 8/11 (1, 10 ve 11. adımlar FAIL), E-3 `false`, çıkış 1.
  - `[promotion] policy = "deny"` → 7/11 (1, 6, 7, 8 FAIL), çıkış 1.
  - Sahne denetimi dişli.

## V3 · Plugin e2e (E-2, E-11) — **geçti; gerçek Obsidian'da da 11/11**
- **`scripts/plugin_e2e.sh` iki kez:** 11/11, E-3 `true`, E-4 "30 istekte sözleşme dışı 0". Beş çıktı iki koşu arasında **ve resmi RUN'la** bayt eşit: `plugin_e2e.json`, `access_log.json`, `events.jsonl`, `traces.jsonl`, `proposals.jsonl`.
- **Gerçek Obsidian ile koşturuldu.** ASSUMPTIONS E4-1, Obsidian'ın başsız koşmadığı ve kullanıcının genel yapılandırmasını bozacağı için komut katmanı harness'ini seçmişti.
  - Başsız kip gerçekten yok, ama ayrı bir örnek mümkün: kendi `--user-data-dir`'i ve önceden yazılmış `obsidian.json` (vault kaydı) ile ayrı bir Obsidian süreci açılır.
  - Kullanıcının örneği ve yapılandırması etkilenmez, çünkü Electron tek örnek kilidi bu dizine bağlı.
  - Bu sınırlamanın gerekçesi kısmen geçersiz.

  Sürücüm:
  - derlenmiş `main.js`, `manifest.json` ve `styles.css`'i taze bir vault'un `.obsidian/plugins/mdkernel/`'ine kurdu;
  - Obsidian'ı CDP ile sürdü: panellerin gerçek DOM'u, girişler, düğmeler, gerekçe penceresi;
  - ajanı aynı `mdk serve --mcp stdio --http --watch` sürecinden MCP ile konuşturdu;
  - eklenti ile sunucu arasına kayıt tutan bir vekil koydu.

  **11/11, iki koşuda:**
  1. Durum çubuğu `mdk 0.1.0 · human:editor`.
  2. Checkpoint panelinde "before" yazıp "Name": sunucuda tek checkpoint, adı `before`.
  3. Ajan `agent/scene.md` önerdi.
  4. **CodeMirror kenar çubuğunda iki `P` işareti**, ipucu `proposal · Agent · source: obs-agent/3.0 · …`. Proposal tablosunda iki satır (doküman, bölüm "Grind notes", kaynak, seq).
  5. Arama paneli varsayılan: ajan belgesi yok. "Include proposals" ile ayrı `mdk-hit-proposal` listesinde.
  6. Proposal görünümü: kutular, "Promote selected", gerekçe penceresi.
     - Boş ve yalnız boşluk gerekçede **Promote düğmesi devre dışı**;
     - Demote devre dışı (K-E2-T);
     - gerekçeyle "2 promoted" bildirimi;
     - sunucuda iki `Promoted`, `actor {kind: human, id: editor}`, `reason: "approved in Obsidian"`.
  7. Arama bulur, `canonical`.
  8. İz paneli: son iz `mdkernel-obsidian/0.1.0`, ajan belgesinin iki bloğu. Bağlantıya tıklama notu açtı, imleç bloğun satırında (0).
  9. `at: before` araması boş, checkpoint kimliği 2. adımınki. Checkpoint panelinin salt okunur geçmiş görünümü:
     - ajan notu için "had no snapshot";
     - `espresso.md` için blokları ("read only").
  10. Ajan `replace_section` → `denied`, başlıksız `append` → `policy_violation` + hint; olay yazılmadı.
  11. `espresso.md` kenar çubuğu (10 × `C`) ve arama paneli değişmedi.

  Diğer ölçümler:
  - **E-3:** md değişen tek adım 3 (`agent/scene.md`). `--watch` açıkken Obsidian'ın kendisi hiçbir md yazmadı: olay sayısı iki koşuda da 117.
  - **E-4:** 68 istek, **sözleşme dışı 0**. Uçlar: `GET /v1/status` ve `POST /v1/{blame, checkpoint, checkpoints, promote, search, trace}`; tek `X-Consumer` `mdkernel-obsidian/0.1.0`. Harness `checkpoints`'i hiç çağırmıyordu, gerçek UI çağırıyor ve o da sözleşmede.

  Kurulum sapması:
  - Obsidian'ın "Trust author and enable plugins" düğmesini CDP ile tıklamak bu atılık örnekte **hiçbir pencerenin** açılamamasına yol açtı. Komut paleti dahil; eklentiyle ilgisi yok.
  - Bu yüzden kısıtlı kipten API ile çıkıldı (`app.plugins.setEnable(true)`, `enablePlugin('mdkernel')`).
  - Güven penceresi altta açık kaldı. Eklentinin penceresi başlığıyla seçildi.
- **Komut katmanı ↔ UI katmanı ayrımı (kaynak okuması):**
  - `src/main.ts` ve `src/ui/*` sunucuya yalnız `commands.ts`'in işlevleriyle (`blameOf`, `historyOf`, `listProposals`, `promoteAll`, `runSearch`, `listCheckpoints`, `nameCheckpoint`, `blameAt`, `recentTraces`) ve `api.ts`'in `connect`/`describeError`'ıyla gider.
  - UI'nın dosya sistemine dokunuşu yalnız şunlar:
    - `adapter.read`/`exists`: `policy.toml` ve `server.token`'ı salt okuma;
    - `getAbstractFileByPath` + `openFile`: var olan notu açma; `openDoc` yoksa bildirir, not yaratmaz;
    - `editor.setCursor`/`scrollIntoView`;
    - CodeMirror'a yalnız `effects` (`setMarks`) gönderme; `changes` yok.
  - **UI'da gizli yazma yolu bulmadım.**
  - Harness'te koşmayan ama gerçek Obsidian'da koşan kısımlar kenar çubuğu çizimi, `metadataCache` hizalaması, düğmeler, pencere ve izden atlama. Hepsi yukarıda geçti.

## V4 · Yazmama (E-3) — **geçti**
- **İzleyici kayıtları:** üç kaynağın hepsinde md değişikliği yalnız 3. adımda (propose).
  - Resmi RUN, benim `scene.sh` koşularım ve resmi `plugin_e2e.json`: `[[],[],["agent/scene.md"],[],…]`.
  - Kendi sahnem: 3. adımda `agent/notes.md` + `cupping.md`.
  - Gerçek Obsidian: 3. adımda yalnız `agent/scene.md`.
- **Statik tarama:**
  - `clients/obsidian/src` içinde `vault.modify|vault.create|vault.delete|adapter.write|fs.write` → **0**.
  - Daha geniş desen de 0 verdi: `vault.(append|process|rename|trash|copy|…)`, `adapter.(append|remove|mkdir|rename|…)`, `fileManager`, `replaceRange`, `setValue`, `processFrontMatter`, `fs`, `require(`, `child_process`, `changes:`.
  - Paketlenmiş `main.js`'te de yazma API'si 0. Harness kancası `MDK_E2E_TAMPER` pakette 0.
- **Kasıtlı bozma (benim):** `src/ui/open.ts`'e takma adlı bir yazma eklendi: `const v = app.vault; v["modify"](f, "x")`. vitest'in statik taraması **yakaladı** (1 failed / 97). Geri alınınca 97/97.

## V5 · Sözleşme (E-4) — **geçti; ek: hint gösterimi yapısal**
- Bütün erişim günlüklerinde her yol ve yöntem CLIENT_CONTRACT §1'de; sözleşme dışı 0. Kapsanan günlükler:
  - resmi RUN: 30 istek, 6 uç;
  - iki `plugin_e2e.sh` koşum: bayt eşit;
  - gerçek Obsidian: 68 istek, 7 uç.
- `scripts/gen_client_types.sh` yeniden üretti; **`git status` temiz.**
- **Kasıtlı bozma (benim), istemcinin kopyasında:**
  - `client.call("demote", …)` → `tsc` hatası TS2345: sözleşme dışı araç derlenmez.
  - Ham `fetch(".../v1/demote")` → `check_deps.sh` VIOLATION (`clients/ names /v1/demote`). Kontrolü olan kopya temiz.
  - Sınır: `check_deps` yalnız harfi harfine geçen `/v1/<x>` dizgelerini görür. Parça parça kurulan bir URL'yi tip sistemi yakalar, `check_deps` yakalamaz.
- **ADDENDUM V5 ek:**
  - Başlıksız `append` reddi ve `hint`'i sahnede, kendi sahnemde ve gerçek Obsidian koşusunda MCP'den doğrulandı.
  - İstemci hint'i gösterir: `describeError` her bildirimde `code: message — hint` yazar (`api.ts`; birim testi `describe_error_shows_hint`).
  - Ancak eklenti hiç `propose` çağırmaz ve bugün `hint` taşıyan tek hata `propose`'undur. Bu yüzden **çalışma zamanında eklentinin bir hint göstereceği yol yok**; gösterim yapısal olarak hazır.

## V6 · Terfi (E-5) — **geçti**
- Gerçek Obsidian akışından Promote → iki `Promoted`, `meta.actor {kind: human, id: editor}` ve gerekçe.
- Gerekçesiz:
  - UI'da gönderilemez (düğme devre dışı, boşluklu gerekçe dahil);
  - sunucuda boş gerekçe, yalnız boşluk ya da eksik `reason` → **400 `invalid_arguments`, olay yok** (kendi istemcim; olay sayısı değişmedi);
  - eklentinin komut katmanı boş gerekçeyi iletir ve 400'ü raporlar (resmi `plugin_e2e`, erişim günlüğünde `POST /v1/promote 400`).
- 2/2.

## V7 · Yayım (E-7) — **kaldı, 2/4; FINAL'le aynı**
- **Yerel ikili** (`cargo build --release --workspace`, klon):
  - `mdk --version` → `mdk 0.1.0`, `--spec-version` → `1.0.3`;
  - `mdk conformance` → level 3, **34/34**, `fixtures_manifest: ok`;
  - sha256 `e3ea1d58c753fb81…`.
- **Intel macOS'u kendim denedim:** `cargo build --release --target x86_64-apple-darwin` → `error: ort-sys@2.0.0-rc.13: no prebuilt binaries available for target x86_64-apple-darwin`. K-E5-T'nin nedeni doğru.
- **`scripts/release.sh` (kendi `dist/`'im):**
  - `aarch64-apple-darwin` ok;
  - `x86_64-unknown-linux-gnu` ok: betiğin adlı kaplarında çapraz derlendi, amd64 kabında `mdk 0.1.0` / `1.0.3` / 34/34;
  - Intel macOS derlenemedi;
  - Windows ölçülmedi.
  - **2/4, `E5_release.json` ile aynı.**
- **CI artefaktı yok:** `release.yml` tetiklenmemiş (K-E5-T, insan adımı). Karşılaştırılacak dört hash yok; bu madde koşturulamadı.
- **`dist/spec-1.0.3/` ↔ `spec/`:**
  - `SPEC.md`, `CONFORMANCE.md` ve `fixtures/` birebir (`diff -rq`: yalnız eklenen `LICENSE` ve `conformance.json`);
  - `conformance.json` level 3, 34/34;
  - `LICENSE` kökle bayt eşit, Apache License 2.0.
- **`release_check.sh`: 13/15.**
  - R4 FAIL beklenen (E-7).
  - **R7 FAIL yanlış.** Wheel `mdkernel_core/` içeriyor. Kontrol, `set -o pipefail` altında `unzip -Z1 … | grep -qE …` kullanıyor; `grep -q` erken çıkınca `unzip` SIGPIPE (141) alıyor.
  - Aynı komut 8 denemede 7 kez 141 verdi. `release_check.sh` art arda üç koşuda R7'yi FAIL, ok, ok raporladı.
  - Hata güvenli yönde (yanlış "ok" üretemez), ama R7 belirlenimsiz.

## V8 · Regresyon ve uyumluluk (E-9) — **geçti; ek: bir sözleşme ihlali**
- `regress_vs_p1d1` (A, B ve C): **100 846 alan, fark 0**, referans P1-D.1 resmi RUN `20261016T124520Z-1527365`. Uyumluluk 34/34, `differing_tests: []`.
- Hariç tutmalar: 11 önek, kullanılmayan 0. Her biri bir karara bağlı: K-E0.3 `index_stale`, K-E0.4 `policy_violation`, K-E0.2 yeni D1-5 alanları, F3 HTTP adımı. Gerekçeleri `why` alanında.
  - Gözlem: commit `632f29e`'nin iletisine göre hariçler **geliştirme RUN'ının farkları görülerek** yazıldı; resmi RUN'dan önce, ama ilk ölçümden sonra.
  - K-C1.6 emsaline uyuyor ve her farkın kararın amaçlanan davranışı olduğunu kontrol ettim. Yine de "ölçümden önce" iddiası resmi RUN için doğru, ilk ölçüm için değil.
- `check_frozen.sh` OK (tablo P1-E `ROUND_START` `11e29b0`'dan beri yalnız ekleme). `check_deps.sh` OK. `test_check_deps.sh` OK, 5 vaka: istemci `core`/`server` içe aktarımı ve sözleşme dışı yol kırmızı, sözleşme yolu yeşil.
- **ADDENDUM V8 ek, gerçek eski indeksle.** Resmi E-14 testi (`p1e_e0.rs`) güncel `/3` manifestinin yalnız `format` dizgesini `mdkernel-index/2` yapıyor. Ben **gerçekten eski bir indeks** kurdum: `p1d-final` etiketinden derlenen P1-D `mdk`'siyle `mdk index build`, biçim `mdkernel-index/2`. Sonra güncel `mdk serve` ile:

  | `auto_rebuild` | `search` | `status` (GET ve POST) | `blame`, `checkpoints`, `trace` | MCP |
  |---|---|---|---|---|
  | `false` | **503 `index_stale`** | **500 `internal`**: "manifest.json is not an index manifest: missing field `dl`" | 200 | `status` → `internal`; `propose` → `index_stale`, hiçbir şey yazılmadı; `blame` ok |
  | `true` | 200, 10 hit | 200 | 200 | stderr "index stale … rebuilt (33 records, 12 snapshots)" |

  E-14'ün ölçtüğü iki şey (503 / 200) gerçek eski indeksle de tuttu. Ancak sözleşme §3 istisnası "Salt okunur çağrılar (`status`, `blame`, `history`, …) indekssiz de çalışır (K-E0.3)" der ve **`status` bu durumda 500 döner. Bu bir sözleşme ihlalidir.** Etkileri:
  - eklentinin `connect`'i `GET /v1/status` ister, bu durumda eklenti "offline: internal: …" der; kullanıcı `index_stale`'i ve yeniden kurma yolunu göremez;
  - kapanışta stderr'de `error: compact (shutdown): index format …` görünür.

  Varsayılan (`auto_rebuild = true`) etkilenmez. Hiçbir çıkış kriteri `status`'u bu durumda ölçmüyor. (P1-D.1 biçimi `/3`'tür, onunla bayat değil, 200.)

## V9 · TS kalitesi (E-10) — **geçti**
- Klonda:
  - `tsc --noEmit` (strict, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`) 0;
  - `eslint .` 0;
  - `vitest` **97/97**;
  - `main.js` **40 701 B** (< 500 KB, resmi değerle aynı);
  - `package.json`'da `dependencies`, `peerDependencies` ve `optionalDependencies` yok; dev bağımlılıkların hepsi tam sürüm.
- Paketin dış istekleri `obsidian`, `@codemirror/state` ve `@codemirror/view`. CodeMirror modüllerini Obsidian çalışma zamanında verir. Gerçek Obsidian koşusunda kenar çubuğu çizildi, bu bunu doğrular.
- **ADDENDUM V9 ek:** `contract_check.sh` temiz → `pass: true`, eksik 0, HTTP uçlarının hepsi ok.
  - **Kasıtlı bozma (benim, resmi vakadan farklı uç):** kaynağın kopyasında `/v1/trace` HTTP rotası 404 yapıldı ve `MDK_BIN` ile verildi → çıkış 1, `missing: ["http-endpoint:POST /v1/trace"]`, başarısız uç yalnız `POST /v1/trace`.
  - Resmi `tamper_p1e` (`checkpoints` uç kaldırma) zincirde üç RUN'da geçti.

## V10 · Belgeler — **geçti; bir ifade güncellenmeli**
- `git diff 11e29b0 HEAD -- NORTH_STAR.md docs/ARCHITECTURE.md`:
  - NORTH_STAR'a yalnız "Kanıtlanmış olanlar (faz 1, …)" bölümü eklendi;
  - ARCHITECTURE'da sürüm satırı, §4.1 "kanıt bekliyor" → "kanıtlı" ve §9 tablosunun satırları değişti;
  - vaatler, ilkeler ve kararlar değişmedi. **Yalnız kanıt durumu.**
  - Sayılar raporlara dayanıyor (yokladığım üçü): P1-A 1 790 alan, P1-C.1 618 değer Δ 0, P1-C C-8 p95 32.2 ms.
  - Güncellenmesi gereken ifade: NORTH_STAR "Obsidian'ın kendi arayüzü başsız koşturulamadı" ve ARCHITECTURE "Obsidian arayüzü başsız koşmadı, komut katmanı ölçüldü". Başsız kip yok, bu doğru; ama arayüz bu doğrulamada ayrı bir örnekte 11/11 koşturuldu.
- `RELEASE.md`: R1–R14 ve R4a'nın her biri `release_check.sh`'te bir satır. **Betikle doğrulanabilir**, bir istisnayla: R7 yukarıdaki SIGPIPE yüzünden belirlenimsiz.
- `THREAT_MODEL.md` K-D1.3 varsayımını **açıkça** söylüyor (satır 19): "Kabul edilen varsayım: token dosyasını … okuyabilen her yerel süreç insan sayılır …". Faz 2 çözümü ve önerilen işletim de yazılı. İstemci bölümü (satır 34) eklentiyi aynı güven düzeyine koyuyor.

## V11 · Karar kuralı — **FINAL'le aynı sonuç**
- **Benim değerlerim:**
  - geçti: E-1, E-2, E-3, E-4, E-5, E-6, E-8, E-9, E-10;
  - kaldı: E-7 (2/4);
  - geçti: E-11 (bu belge: sahne ve plugin e2e kendi ortamımda, ayrıca gerçek Obsidian'da);
  - geçti: E-12..E-16 (V0, V8, V9).
- **GOAL'ün dalları:**
  - "E-1..E-11 geçti" değil.
  - "E-3/E-4/E-5/E-6 kaldı" değil.
  - "E-2 kaldı, E-1 geçti" değil.
  - **"E-7 kaldı → yayım ertelenir, faz 2 planlaması başlar"** uyuyor. Kalan tek kriter E-7, dal tanımlı.
  - Varsayılan "tanımlanmamış kalış → tur kaldı" dalı uygulanmaz: E-7 dışında kalan yok, ADDENDUM'un E-12 dalı da geçti.
- FINAL "yalnız E-7 kaldı → yayım ertelenir, faz 2 planlaması başlar (K-E5-T); 0.1.0 etiketi atılmaz" diyor. **Aynı.**
- **0.1.0 etiketi atılmamış:**
  - `git tag -l 'v0*'` boş;
  - klonun `origin`'inde (ana depo) `v0.*` etiketi yok;
  - `release_check` R14 ok.

## Sonuç

**İhlal / fark / şüphe:**
1. **Sözleşme ihlali (V8):** başka biçimdeki gerçek bir indeks ve `[index] auto_rebuild = false` ile `status` (HTTP GET/POST ve MCP) **500 `internal`** döner. Sözleşme §3 salt okunur çağrıların indekssiz çalışacağını söyler.
   - E-14'ün testi manifesti yalnız yeniden etiketlediği için bunu göremiyor.
   - Varsayılan ayarı ve hiçbir çıkış kriterini etkilemiyor.
   - Eklenti bu durumda bağlanamaz.
2. **R7 belirlenimsiz (V7):** `release_check.sh` R7 `pipefail` + `grep -q` yüzünden SIGPIPE ile rastgele FAIL verir. Yanlış "ok" üretemez.
3. **ASSUMPTIONS E4-1'in gerekçesi kısmen geçersiz (V3, V10):** gerçek Obsidian kullanıcının yapılandırmasına dokunmadan ayrı örnekte sürülebiliyor. E-2'nin ölçtüğü şey gerçek arayüzde de tuttu, ama NORTH_STAR ve ARCHITECTURE'daki "arayüz koşturulamadı" ifadesi güncel değil.
4. **Gözlem (V8):** E-9 hariç tutmaları geliştirme RUN'ının farkları görülerek yazılmış. Her biri bir karara bağlı ve kullanılmayan yok, ama "ölçümden önce" yalnız resmi RUN için doğru.
5. **Gözlem (V5):** eklentinin hint gösterimi yapısal. Bugün eklentinin çalışma zamanında hint alabileceği bir çağrı yok.

**Hüküm: Şu koşullarla güvenilir.**
- FINAL'in ölçtüğü her satır kendi ortamımda yeniden üretildi: E-8 101 958 alan fark 0 ve yapıtlar resmi RUN'la bayt eşit; E-9 100 846 alan fark 0; E-1/E-2 11/11, E-11 dahil.
- Karar kuralı bağımsız türetildi ve FINAL'le aynı: E-7 kaldı → yayım ertelenir, faz 2 planlaması başlar.
- Koşul: 1. maddedeki `status` ihlali, `0.1.0`'ın herhangi bir yayımından önce düzeltilir (ve E-14 testi gerçek eski biçimle sınanır) ya da `DECISIONS.md`'de sözleşme değişikliğiyle kapatılır.
- Yayım E-7 yüzünden zaten ertelendiği için koşul bugünkü kararı değiştirmiyor. `RELEASE.md` insan adımı 1 ("Güvenilir" demeden hiçbiri yapılmaz) bu koşul kapanana kadar geçerli kalır.

**Öneriler (≤ 3):**
1. **`status`'u indeksten bağımsız yapın** (bayat ya da okunamayan manifestte `index: "stale"`/`"damaged"` gibi bir alan, 200). E-14'ün testini `p1d-final` ikilisinin yazdığı gerçek `/2` indeksiyle kurun ve `status`'u da ölçün. Aynı turda `release_check.sh` R7'yi SIGPIPE'a dayanıklı yapın (`grep -c` ya da ara dosya).
2. **Gerçek Obsidian e2e'sini kapıya alın:** ayrı `--user-data-dir` + CDP, bu doğrulamadaki gibi. E-2'yi komut katmanı harness'iyle değil gerçek arayüzle ölçün, ASSUMPTIONS E4-1'i kapatın, NORTH_STAR ve ARCHITECTURE ifadesini düzeltin. Not: güven penceresi CDP ile tıklanınca bu örnekte pencereler açılamıyor; kısıtlı kipten API ile çıkmak gerekti.
3. **K-E5-T'yi yayımdan önce kapatın:** Intel macOS ya hedeflerden çıkarılır ya `ort` `load-dynamic` ile çözülür. `release.yml` insan tarafından tetiklenir; dört hedefin hash'leri ve `mdk conformance` sonuçları `dist/release.json` ile karşılaştırılıp kaydedilir. Bu doğrulamada CI artefaktı olmadığı için bu karşılaştırma yapılamadı.

**Faz 2 için gördüğüm en önemli üç risk (bilgi):**
1. **İnsan kanalının sınırı dosya izinleridir.** Claude Code gibi vault'ta dosya erişimi olan bir ajan:
   - `server.token`'ı okuyup HTTP'den insan olarak terfi ettirebilir (K-D1.3, THREAT_MODEL'de kabul edilmiş);
   - md dosyasını doğrudan yazabilir. `--watch` ya da bir sonraki `propose`'un eşitlemesi bunu **insan yazımı** olarak alır (§2.1, SECURITY izleyici satırı).

   Ürün sahnesinin vaadi (ajan canonical'ı değiştiremez) yalnız ajanın dosya sistemine erişimi yokken geçerli. Faz 2'de ajan kum havuzu ya da OS düzeyi ayrım ürünün çekirdeği olur.
2. **Biçim göçleri ve bayat durumlar:** indeks biçimi `/2`→`/3` değişti ve bayat durumun kenarında bir ihlal çıktı (V8). Ayrıca:
   - eski kaydedicilerin bilinen FAIL'leri her zincirde birikiyor (C-3, C-11, D-2, D-7, D-9);
   - onnxruntime sabitlenmiş bir `ort` ön sürümüne (`2.0.0-rc.13`) bağlı ve platform erişimini belirliyor (E-7).

   Çok vault, uzak sunucu ve sync gelince göç ve sürüm uyumu yüzeyi büyüyecek.
3. **İstemcinin blok hizalaması tahmin dışı ama kırılgan:** kenar çubuğu Obsidian'ın `metadataCache` blok haritasını çekirdeğin blok sırasıyla eşliyor (ASSUMPTIONS E2-2). İzin listesi dışı bir tür ya da sayı farkında işaret konmuyor; dengelenen iki farkta kayık işaret mümkün. Satır konumu sözleşmede yok (§2.5). Gerçek kullanıcı notlarında (HTML, dipnot, callout, gömme) "işaret yok" durumu sık olabilir. Sözleşmeye blok konumu eklemek ya da hizalamayı ölçmek faz 2'nin istemci riskidir.
