# P1-D.1 — Bağımsız Doğrulama

- Yönerge: `docs/rounds/P1-D.1/VERIFY.md`, `docs/DEV_PROCESS.md` §5.
- Doğrulanan commit: `c3b6497` (READY.md). READY'nin andığı sha `f75d557`, ölçülen commit `1527365`. `1527365..c3b6497` farkı yalnız `reports/rounds/P1-D.1/`, `docs/rounds/P1-D.1/progress/` ve `docs/DECISIONS.md`'de.
- Ortam:
  - temiz klon (oturum scratchpad'i, ana depodan `git clone`);
  - macOS (Darwin 27.0.0), rustc 1.92.0, Node 24.3.0, Python 3.13.5;
  - model `minilm-l6-v2/e73168f9` (`make models`: sha256 + blake3 manifest ok).
- Kendi RUN'larım: A `20261016T135522Z-c3b6497`, B `20261016T142215Z-c3b6497` (`scripts/verify.sh pair <klon> A B`; zincir A 1 661 s, B 1 592 s).
- Süreç notu:
  - **Bağımsızlık sapması (açıkça kayıt):** VERIFY "ana dizine erişme" diyor. Klonu kurmadan önce ana dizinde şunlar yapıldı:
    - `cat docs/rounds/P1-D.1/VERIFY.md`, `cat docs/rounds/P1-D.1/progress/READY.md`, `DEV_PROCESS.md` §5 (`awk`);
    - `ls docs/rounds/P1-D.1/ docs/rounds/P1-D.1/progress/ reports/rounds/ reports/rounds/P1-D`;
    - `git log --oneline -8`, `git status --short` (ağaç temiz).

    READY'deki commit'in HEAD'de olup olmadığını görmek ve klonun kaynağını belirlemek içindi. Ana dizinden başka belge, kod ya da RUN okunmadı. `GOAL.md`, FINAL ve bütün diğer belgeler klondan okundu. Bütün koşturmalar klonda ve kendi çalışma dizinlerimde yapıldı. P1-D doğrulaması da aynı ön okumayı yapmıştı, ama bu yine de §5'in harfinden bir sapmadır. Hükmü etkilediğini düşünmüyorum, çünkü READY'nin iddiaları klondaki dosyalarla ve kendi RUN'larımla yeniden sınandı.
  - Deneyler için kendi istemcimi yazdım: Python, MCP stdio JSON-RPC + HTTP, depo kodu kullanılmadı.
  - Kendi vault'um: `spec/fixtures/mini_vault` (12 belge) + `NORTH_STAR.md`, `docs/ARCHITECTURE.md`, `docs/PACKAGES.md`, `docs/DEV_PROCESS.md`. Toplam 16 insan belgesi, `mdk ingest-dir`.
  - Her deney bu vault'un ayrı bir kopyasında yapıldı.
  - Ana dizine yazılan tek dosya bu rapor.

## V1 · Çift koşturma — **kaldı (çıkış 3)**; nedeni bir ölçüm kodu kusuru, ürün determinizmi tuttu (Ş0)
- **`verify.sh pair` çıkış 3.** `compare_p1d1 pair`: 100 875 alan, **1 fark: `p1d1/d1_5.json:responses_a_vs_b_equal`** (A `true`, B `false`). `bytes_equal: true`: P1-D ve P1-D.1 bayt kümesinin hepsi eşit.
  - Bayt kümesi: e2e transkriptleri; e2e vault'unun `events/traces/proposals.jsonl`'u, `espresso.md`'si ve `agent/plan.md`'si; `scene.json`; `tools_list.json`; yönetişim vault'u; `d1_3`, `d1_8`, `d1_13`.
- İki RUN'da da `p1d1_checks.json` D1-1..D1-13 `true`. `tamper_p1d1` 14/14 beklendiği gibi (9 bozma reddedildi, 5 kontrol kabul).
- **Kök neden.** `d1_5_calls()` (`server/tests/p1d1_criteria.rs`) 100 boru hatlı çağrının içine salt okunur `blame espresso.md` koyuyor.
  - Sözleşme §5'e göre salt okunur çağrılar paraleldir. Kendinden önce gönderilmiş ama işlenmemiş bir `propose`'u görmeyebilir.
  - Bu yüzden iki boru hatlı koşunun **yanıtları** zamanlamaya bağlı. `responses_a_vs_b_equal` belirlenimsiz bir alan.
  - `compare_p1d1` bu alanı kalite alanı gibi karşılaştırıyor. F16 yalnız iki süre alanını çıkarmıştı (`P1D1_TIMING`), bu alan listede yok.
- **Kanıt (resmi çiftin şansı):** resmi RUN `20261016T124520Z-1527365`'in `d1_5.json`'unda da `responses_a_vs_b_equal: **false**`. Resmi çift çıkış 0 verdi, çünkü iki koşusu da `false` çıkmıştı. Benimkinde biri `true`, biri `false` çıktı.
- **D1-5'in ölçtüğü şey tuttu.** Her iki RUN'da `pipelined_a_vs_b` ve `pipelined_vs_lockstep` için `events.jsonl`, `traces.jsonl`, `proposals.jsonl` ve md dosyaları `true`. `events_hash32` ve `traces_hash32` iki RUN'da ve resmi RUN'da aynı.
- **Resmi RUN ile karşılaştırma.** Resmi RUN'a e2e çalışma vault'ları konmadığı için `compare-p1d1` onunla koşamıyor (`transcript.jsonl missing`; READY'de yazılı). Bunun yerine `p1d1/*.json` + `p1d1_checks.json` + `conformance_p1d1.json` (17 dosya, 853 alan; süre ve yol alanları hariç) alan alan karşılaştırıldı:
  - B ↔ resmi: fark 0 (yalnız `fixtures_root` yolu);
  - A ↔ resmi: tek fark aynı `responses_a_vs_b_equal`.
- **Sahne vault'ları** (zincir yalnız `scene.json`'u karşılaştırıyor): `MDK_SCENE_KEEP=1 scripts/scene_agent.sh` güncel kodla iki kez koşturuldu.
  - `events.jsonl` (116 satır), `traces.jsonl` (3) ve `proposals.jsonl` (1) bayt eşit.
  - Vault `diff -r` eşit, `.mdkernel/` dahil.
  - 7 MCP/CLI transkripti eşit.
  - `scene.json`, resmi RUN'ınkiyle bayt eşit.
- **Boru hattı vault'ları:** V4'te kendi istemcimle 2 boru hatlı + 1 kilit adımlı vault bayt eşit. Zincirde de 2 RUN × (A↔B, A↔kilit adımlı) eşit.
- **Kasıtlı bozma (benim):** A ↔ A'nın kopyası → çıkış 0. Kopyada `mcp_e2e` vault `traces.jsonl`'inde 1 bayt → çıkış 3. `d1_1.json`'da 1 boole alanı → çıkış 3. Geri yükleme → çıkış 0.

## V2 · Canonical korunumu (D1-1/2/3) — geçti
Hazırlık: kendi vault'umun kopyası, `append_only`, `human_id` ayarlı. MCP (`verifier-v2/0.1`) üzerinden 3 proposal:
- `agent/plan.md` (`create`);
- `espresso.md`'ye `## Öneri: Grind` (`append`);
- `maintenance.md`'nin son `## Checklist` bölümüne `### Agent tip` (`append`). Bu, karışık canonical + proposal bir bölüm üretir.

**Canonical bloklara dokunan 8 `replace_section` (MCP): 8/8 `isError: true`, `code: denied`.** Her vakadan önce ve sonra bütün md dosyalarının, `events.jsonl`'in ve `proposals.jsonl`'in sha256'sı **aynı**.

GOAL'ün 6 vakası:
1. tam bölüm: `espresso.md` "Recipe";
2. kısmi: "Equipment", yalnız kod bloğundaki yorum değişti, gövdenin geri kalanı aynı;
3. başlık: `grinders.md` "Calibration", içerik yeni bir `##` başlığıyla başlıyor;
4. karışık canonical + proposal: `maintenance.md` "Checklist";
5. iç içe başlık: `ARCHITECTURE.md` "4. Katmanlar", altında 5 `###`;
6. insan dokümanının tamamı: `cupping.md` H1 "Cupping".

Benim 2 vakam:
7. boş gövde, yani bölümü silme: `safety.md` "Burns";
8. birebir aynı gövdeyi yeniden yazma: `origins.md` "Brazil". İçerik değişmese de reddedildi. Kural, canonical blok içeren bölümü reddediyor; içeriği karşılaştırmıyor. Sözleşme §2.1 ile tutarlı.

Ek olarak:
- aynı tür yazım HTTP insan kanalından (`safety.md` "Fire") → 403 `denied`, dosyalar değişmedi. Politika kanaldan bağımsız.
- başlıksız `append` (`staff.md`) → `denied`, değişiklik yok. İnsan bölümünün chunk'ına katılıyor; sözleşmedeki "istemci kuralı".

**Yalnız proposal bölümde kabul:** `agent/plan.md` "Draft" ve `espresso.md` "Öneri: Grind" üzerinde `replace_section` → `isError: false`, `edit_policy: append_only`. `blame`:
- `agent/plan.md`: 4/4 blok `proposal`/`Agent`/`verifier-v2/0.1`;
- `espresso.md`: 10 insan bloğu `canonical`/`Human`, yeni 2 blok `proposal`/`Agent`.

**Değişmezlik:**
- 50 sorgu × 3 kip (`lexical`, `flat`, `hybrid`, k=10). Sorgular vault'un kendi sözcük dağarcığından, tohumlu rastgele 1–3 sözcük.
- 150 varsayılan `search`'ün `hits` dizisi kaydedildi; 150'sinin de sonucu boş değil.
- Ardından tohumlu 100 rastgele ajan yazımı: 27 `create`, 36 `append` (insan belgelerine ve ajan belgelerine, kendi başlığıyla), 37 `replace_section` (ajanın kendi bölümlerinde). 100/100 kabul.
- Sonrasında aynı 150 `search`: **150/150 `hits` bayt eşit** (`json.dumps(sort_keys)`).
- Yalnız ajan sözcükleriyle sorgu: varsayılan 0 hit, `include_proposal: true` ile yalnız `proposal` hit'i.
- Yanıt uzunluğu (K-D1.8): bu vault'ta 72 `include_proposal` araması (6 sorgu × 3 kip × k ∈ {1, 3, 5, 10}).
  - İhlal **0**: katman başına ≤ k, toplam ≤ 2k.
  - canonical önek, bayrak kapalı yanıtla bayt eşit.
  - 51 aramada sınıra tam olarak ulaşıldı (k + k).

## V3 · Aktör izi (D1-4) — geçti
- Vault kopyası, `policy.toml` `[edit] policy = "unrestricted"`. `status.policy.edit` = `unrestricted`.
- MCP (`verifier-v3/0.1`) `replace_section "Burns"` boş içerikle → kabul. Yanıtta `edit_policy: unrestricted`, `actor {id: verifier-v3/0.1, kind: agent}`.
- `events.jsonl`'e 4 `BlockDeleted` (seq 284–287) + 1 `SnapshotCommitted` eklendi. Her `BlockDeleted.meta` = `{actor: {id: "verifier-v3/0.1", kind: "agent"}, source: "verifier-v3/0.1", matcher_version: "v3"}`. `SnapshotCommitted` aktör taşımıyor (spec §7: başka türler taşıyamaz).
- `id` = `<clientInfo.name>/<clientInfo.version>`. VERIFY "istemci adı" diyor; sözleşme ve SECURITY.md bu biçimi açıkça tanımlıyor (P1-D D1-1 biçimi).
- `history` (MCP ve CLI): `BlockCreated` (insan, aktörsüz, `provenance.author_kind: Human`), ardından silen aktörlü `BlockDeleted`.
- `append_only`'ye dönüş: `status` `append_only`. Aynı vault'ta `replace_section "Fire"` → `denied`.

## V4 · Boru hattı (D1-5/6) — geçti
Kendi istemcim (`pipe/1`) 100 çağrıyı yanıt beklemeden tek seferde stdin'e yazdı, sonra yanıtları `id` ile topladı.
- Çağrı karışımı (tohumlu): 40 `propose`, 10 `search` (3 kip, %30 `include_proposal`), 5 `read_block`, 7 `checkpoint` (adlı ve adsız), 6 `status`, 10 `blame`, 12 `history`, 10 `trace`.
- `propose`'lar: `create`, insan ve ajan belgelerine kendi başlığıyla `append`, kendi bölümünde `replace_section`.
- Üç kopya: A ve B boru hatlı, L kilit adımlı (her yanıtı bekleyerek). Hata 0.
- A ↔ B ve A ↔ L, `cmp` ile **bayt eşit**:
  - `events.jsonl` (422 satır);
  - `traces.jsonl` (15);
  - `proposals.jsonl` (40);
  - md ağacı (`diff -r`, 35 dosya).
- Ayırt edicilik kontrolü: iki `propose`'un sırası değiştirildiğinde `events.jsonl` farklı (`cmp` 1). Ölçüm sırayı gerçekten görüyor.
- **Salt okunur paralellik:** 100 salt okunur çağrı (`status`, `blame ARCHITECTURE.md`, `history`, `checkpoints`, `trace`; 20'şer). İki tekrar:

  | Tekrar | Seri | Boru hatlı | Oran |
  |---|---|---|---|
  | 1 | 0.199 s | 0.045 s | **0.22** |
  | 2 | 0.200 s | 0.049 s | **0.25** |

  100/100 yanıt seri yanıtlarla eşit, hata 0. Resmi RUN 0.27 diyor; eşik < 0.6 (bilgi).

## V5 · Kanal/aktör (D1-7) — geçti
Vault kopyası, `human_id = "arkin"`.
- MCP `promote` → `decision: denied`, `actor {id: verifier-v5/0.1, kind: agent}`, `events.jsonl` değişmedi. Sözleşmeye göre bu bir yanıttır, hata değildir (`isError: false`).
- HTTP `promote` (`X-Consumer: human:x`) → 200 `promoted`. `Promoted.meta.actor = {id: "arkin", kind: "human"}`, `from: proposal`, `to: canonical`, `policy: human_only`.
- HTTP `promote` (`X-Consumer: agent:bot`) → yine `promoted`, aktör `human:arkin`. Aktör türünü başlık değil kanal belirliyor.
- MCP'de "insan" kimliği deneme: `clientInfo.name = "human:x"`. `promote` → `denied`, `actor {id: "human:x/1", kind: agent}`. `propose` aktörü de `agent`. MCP'de aktör türü değiştirilemiyor.
- `human_id` kaldırıldı: HTTP `promote`, `propose` ve `checkpoint` → **403 `denied`**, mesaj `[identity] human_id`'yi gösteriyor. Dosyalar ve olaylar değişmedi. HTTP `search` 200 (okuma, sözleşme §1).

## V6 · Hata kodları (D1-8) — geçti
20 vaka hem MCP'de (`structuredContent.error.code`) hem HTTP'de (`POST /v1/<araç>`, durum + gövde) koşturuldu. **20/20'de iki kanalın kodu da beklenen kodla aynı. HTTP durumları sözleşme §3 tablosuyla aynı.**

| Kod | Vakalarım | HTTP |
|---|---|---|
| `invalid_arguments` | `k: 0`, `mode: bogus`, bilinmeyen alan, `.txt` yolu, boş `reason` | 400 |
| `empty_query` | `"   "` | 400 |
| `path_outside_vault` | `../escape.md`, `/tmp/abs.md`, `.mdkernel/evil.md`, vault dışını gösteren sembolik bağ | 400 |
| `denied` | canonical bölüme `replace_section` | 403 |
| `not_found` | bilinmeyen soy (`read_block`, `promote`), olmayan belgeye `append`, bilinmeyen `at` | 404 |
| `exists` | var olan belgeye `create`, harf farkıyla `Espresso.md` | 409 |
| `too_large` | 1 MiB + 1 bayt `content` | 413 |
| `heading_not_found` | olmayan başlık | 422 |

Ek olarak:
- `checkpoint {name: "taken"}` aynı durumda tekrar → hata değil, aynı checkpoint (sözleşme §2.3 c).
- **`policy_violation`:** sunucu çalışırken `policy.toml` bozuldu (yinelenen `[edit]`) → MCP `policy_violation`, HTTP 422. Geçersiz politikayla başlatma → `mdk serve` çıkış 1 (`unknown edit policy "bogus"`).
- **`internal`:** `events.jsonl` salt okunur yapıldı → MCP `internal`, HTTP 500, mesaj `ingest failed; the file was restored`. Vault'ta öksüz dosya kalmadı.
- Yalnız HTTP:
  - token yok → 401 `denied`; yanlış token → 401 `denied`;
  - `X-Consumer` yok → 400 `invalid_arguments`;
  - gövde dizi → 400 `invalid_arguments`;
  - `GET /v1/search` → 405 `invalid_arguments`;
  - bilinmeyen yol → 404 `not_found`;
  - 2 MiB gövde → 413 `too_large`.
- Yalnız MCP: bilinmeyen araç → JSON-RPC `-32602`. Boş `clientInfo` ile `search` → `invalid_arguments`.
- Sözleşmedeki 10 kodun **10'u da iki kanalda tetiklendi**.

## V7 · Checkpoint adlandırma (D1-9) — geçti
- `at`'siz `search` → adsız checkpoint `eec94c60…`. `checkpoints`'te `name: null`.
- `checkpoint {name: "yesterday"}` → **aynı kimlik** `eec94c60…` ve aynı `event_seq` (284). `events.jsonl`'e tek olay eklendi: `CheckpointNamed`, `meta {checkpoint_id: eec94c60…, name: "yesterday"}`.
- Aynı ad, aynı durumda → aynı checkpoint (hata değil, sözleşme §2.3 c).
- **Aynı ad tekrar, başka bir durumda** (araya bir `propose`) → **`exists`** (`name "yesterday" is taken by eec94c60…`).
  - VERIFY'ın "aynı ad tekrar → `exists`" maddesini böyle okudum. Aynı durumda aynı ad sözleşmeye göre idempotent.
- Adlı duruma başka ad → `exists`. 32 hex ad → `invalid_arguments`. `search {at: "yesterday"}` → aynı kimlik.
- Adlı bir durumda `at`'siz `search`/`read_block` çalışıyor ve adlı checkpoint'i kullanıyor.
- Gözlem: adlandırmadan sonra aynı durumda `checkpoint {}` (adsız) → `exists`. Sözleşmede yazılı (§2.3: "O checkpoint adlıysa → `exists`"). Ancak istemci için ilk bakışta şaşırtıcı.

## V8 · Uyumluluk ve eski vault'lar (D1-10/11) — geçti, sürüm türetiminde bir şüpheyle (Ş1)
- `make conformance` (klonda): **level 3, 34/34**, `spec_version` 1.0.3. C2-10, C3-06 ve C3-07 `pass`, beklenen hash = gerçek hash.
- **Yeniden üretim:**
  - C2-10, C3-06 ve C3-07'nin `expected.json`'larını sildim.
  - `scripts/gen_fixtures_c2.sh` + `scripts/gen_fixtures_c3.sh` → üçü de yeniden yazıldı (`MANIFEST.json` dahil).
  - `git status` **temiz**.
- **Spec sürümünü kendim türettim (CRITERION "Sürüm numarası (K-D1.9)"):**
  - `git diff e8827f8 1527365 -- spec/SPEC.md`: §1–6'da yalnız iki `Informative (1.0.3)` paragrafı var (§4.2, §4.3), yani açıklık.
  - Normatif yeni cümleler: §7 "Provenance of events", §8 "Naming", §9 "Response length".
  - Üçü de Validated bölümde ve (b)'yi sağlamıyor (önceki metne uyan bir uygulama artık uyumsuz olabilir) → açıklık değil.
  - Benim sınıflandırmam SPEC_EDITS E1–E8 ile satır satır aynı.
  - Kural: Validated'da açıklık olmayan + kırıcı değil → yama, yani **1.0.3**. SPEC_EDITS ile aynı.
  - **Ama** "kırıcı değil" yargısı yoruma bağlı. Ayrıntı Ş1'de; takım bunu K-D1.4-T olarak açık bırakmış.
- **P1-D sahne vault'u:**
  - `p1d-final` ağacını `git archive` ile ayrı dizine çıkardım, derledim, kendi `scene_agent.sh`'ını `MDK_SCENE_KEEP=1` ile koşturdum: 9/9.
  - Ürettiği `scene.json`, commit'lenmiş P1-D `reports/rounds/P1-D/scene.json` ile bayt eşit.
  - Eski `mdk` ve yeni `mdk`:
    - `events-hash` **eşit**: `f1944f63…`;
    - 13/13 belgede `replay` çıktısı **eşit**;
    - yeni `blame` 13/13 çalışıyor; `agent/scene.md` `Agent`/`scene-agent/1.0`;
    - `events.jsonl` (116 satır) yeni `mdk` tarafından değiştirilmedi.
- **v3-final karışık vault'lar:**
  - `scripts/gen_v3final_mixed.sh <kendi dizinim>` v3-final çekirdeğiyle yeniden üretildi. Commit'lenmiş fixture'larla `diff -r` eşit.
  - 9 senaryonun her biri yeni `mdk` ile açıldı: `events_hash` **9/9 eşit**, `replay_snapshot_ids` **12/12 eşit**, `blame` hatasız.
- **Bulgu (Ş3):**
  - Eski (P1-D) vault, yeni `mdk serve` ile açılıyor; ama `search` **`internal`** (HTTP 500) dönüyor: `index format "mdkernel-index/2" != "mdkernel-index/3" (rebuild it with mdk index build)`. Kapanışta sıkıştırma da aynı hatayı yazıyor.
  - `mdk index build` sonrası arama çalışıyor, `events.jsonl` değişmiyor.
  - Sessiz değil ve SPEC_EDITS'te not edilmiş. D1-11'in ölçtüğü (replay, `events_hash`) bundan etkilenmiyor.

## V9 · Sözleşme (D1-13) — geçti (kapsam gözlemiyle)
- `scripts/contract_check.sh` → **çıkış 0**: 10 araç `tools/list` ile eşit, 10 kod `error.rs` ile eşit (HTTP durumları dahil), her araç ve kod `server/tests/contract.rs`'te gerçek bir çağrıyla eşleşiyor, `missing: []`.
- Sözleşmede **K-D1.3 kanal kuralı** yazılı: §1 tablo (MCP → `agent:<tüketici>`, HTTP → `human:<human_id>`), "X-Consumer … aktör türünü değiştiremez", `human_id` yoksa yazma 403.
- Sözleşmede **K-D1.8 uzunluk kuralı** yazılı: §2.2 "katman başına en çok `k` hit, toplam en çok `2k`" (normatif).
- `SECURITY.md` **token/insan varsayımını açıkça söylüyor**: "Kabul edilen varsayım: token dosyasını … okuyabilen her yerel süreç insan sayılır …; faz 2 OS anahtar zinciri".
- Bütün HTTP uçlarını kendim çağırdım: 10 `POST /v1/<araç>` + `GET /v1/status`. Hepsi sözleşmedeki ana alanlarla yanıt verdi.
  - `promote` canonical bir soy için 400 `invalid_arguments` döndü. Sözleşme §2.4'e uygun.
- **Kapsam gözlemi (Ş4):** `contract_check.sh` araçları ve kodları `contract.rs`'teki **süreç içi** çağrılarla (`Kernel` üzerinden dispatch) eşliyor. VERIFY'ın "her araç, **uç** ve kod" maddesindeki HTTP uçları ve gerçek HTTP durum eşlemesi bu denetimde yok.
  - HTTP tarafı yalnız `Code::http_status()` birim eşlemesi ve başka testler (`p1d1_server.rs`, D1-8'in HTTP yarısı) ile sınanıyor.
  - Kendi V6/V9 çağrılarım HTTP tarafını tam olarak doğruladı. Ama D1-13 kapısı bir uç kaybolsa bile yeşil kalır.

## V10 · Dondurma, bağımlılık, karar kuralı — dondurma ve bağımlılık geçti; karar kuralı koşullu (D1-12, Ş0)
- `scripts/check_frozen.sh` → `OK` (FROZEN tablosu P1-C, P1-D ve P1-D.1 `ROUND_START`'larından beri yalnız ekleme). `scripts/check_deps.sh` → `OK` (core ← index ← server ← clients; research → bindings/python).
- Bağımsız dondurma denetimi: `git diff --name-only e8827f8 1527365` içinde dondurulmuş yollara (`research/eval/**`, `scripts/verify.sh`, uyumluluk testleri, `spec/fixtures/**`, `test_tamper_*`) dokunan her dosya F1–F16'dan birine karşılık geliyor.
  - Turda yeni yaratılanlar: `compare_p1d1.py`, `report_p1d1.py`, `run_p1d1_checks.py`, `test_tamper_p1d1.sh`.
  - F16 (`compare_p1d1.py`, ilk resmi çiftin çıkış 3'ünden sonra) yalnız `d1_5.json`'u bayt listesinden çıkarıyor. Dosya alan alan karşılaştırılmaya devam ediyor, iki duvar saati alanı hariç.
  - Gerekçe dosya içeriğiyle tutarlı. Ölçülen hiçbir kalite alanını gevşetmiyor.
- D1-12'nin 29 hariç tutması (`compare_p1d1.EXCLUDED`) `2f56267`'de, resmi RUN'dan (`c337bf6`) önce yazılmış. Her biri bir karara (K-D1.1/2/5/6/7/8) bağlı.
  - Gözlem: P1-D'nin `D-2`/`D-7`/`D-9` kayıtları bütün alt ağaçlarıyla hariç. Altlarındaki dosya alanları ayrı ayrı sınandığı için regresyon gizlemiyor, ama kaba bir taneli hariç tutma.
- **GOAL'den türettiğim kural:**
  - D1-1..D1-13 geçti → K-D4-T kapanır, P1-E başlar;
  - D1-1, D1-3 ya da D1-4 kaldı → yönetişim, P1-E başlamaz;
  - D1-5 kaldı → determinizm, P1-E başlamaz;
  - **varsayılan dal:** tanımlanmamış kalış (D1-2, D1-6..D1-13'ten biri) → tur kaldı, karar talebi.
- Kendi ölçümlerimde D1-1..D1-11 ve D1-13'ün karşılığı geçti.
- **D1-12 harfiyen geçmedi:** benim çiftimde fark 1 alan (V1, Ş0). Harfi harfine uygulanırsa bu bir "tanımlanmamış kalış"tır, yani tur kaldı, karar talebi.
- Benim yargım: fark, sözleşmenin (§5) zaten belirlenimsiz ilan ettiği, hiçbir eşiğe girmeyen bir yanıt gözlemi. Ölçülen kalite alanı ya da yapıt farkı değil. Aynı sınıftaki kusur F16'yla bu turda düzeltilmişti.
- Bu nedenle FINAL'ın sonucunu (**K-D4-T kapanır, P1-E başlar**) **koşullu** olarak doğru buluyorum. Koşul: karşılaştırıcı düzeltilip çift yeniden koşturulmalı (Ö1).
- FINAL'ın "D1-12 fark 0" satırı resmi çift için doğru; ama tekrarlanabilir değil.
- K-D4-T'nin üç alt maddesi ayrıca:
  - (a) canonical silme/değiştirme ve aktör: V2, V3;
  - (b) CLI `mdk propose` artık aynı politikadan geçiyor: CLI `replace_section "Fire"` → çıkış 1, `denied by edit policy append_only`, dosya ve olaylar değişmedi;
  - (c) TOCTOU: `SECURITY.md` "Bilinen açık sınırlar"da faz 2'ye bırakılmış.

## Sonuç

**İhlal:** GOAL ölçütlerine ve değişmez kurallara göre yok.

**Fark:**
- Kendi çiftimde 1 alan: `p1d1/d1_5.json:responses_a_vs_b_equal`, belirlenimsiz bir gözlem (Ş0).
- Bütün baytlar eşit.
- Resmi RUN'la 853 P1-D.1 alanında yalnız bu alan farklı.
- Sahne ve boru hattı vault'ları bayt eşit.

**Şüphe / gözlem:**
- **Ş0 · `make verify` (pair) tekrarlanabilir değil.** `compare_p1d1`, D1-5'in belirlenimsiz `responses_a_vs_b_equal` alanını (salt okunur `blame` boru hattında paralel; sözleşme §5) kalite alanı sayıyor.
  - Resmi çift ortak şansla (`false`/`false`) çıkış 0 verdi, benimki 3.
  - Ürün davranışı sözleşmeye uygun, kusur ölçüm kodunda. Ama turun kalite kapısı (`make verify` temiz) şu an bir yazı-tura.
  - Aynı sorun `responses_pipelined_vs_lockstep_equal`'da da var (her yerde `false`, bu yüzden şimdilik sabit).
- **Ş1 · Spec sürüm düzeyi yoruma bağlı (K-D1.4-T, açık).**
  - CRITERION'ın majör maddesi "daha önce uyumlu bir uygulamanın verisini (log, indeks) okunamaz kılan kırıcı değişiklik" diyor.
  - 1.0.2'de §4.2 `meta`'ya şema vermiyordu (§5 yalnız `matcher_version` ister). 1.0.2'ye uyan bir uygulama `BlockUpdated.meta.actor = "bob"` ya da `SnapshotCommitted.meta.source` yazabilirdi.
  - 1.0.3'ün §7'si okuyucuya bunları **reddetmeyi** buyuruyor (C2-10 `actor_tamper` bunu sınıyor). Maddenin harfi bu durumu kapsıyor; böyle okunursa sürüm 2.0.0 olur.
  - Takımın "gerçekte yazılmış veri" okuması (yama, 1.0.3) savunulabilir ve etiketli varsayım olarak kayıtlı. Hiçbir davranış değişmiyor, yalnız numara.
  - D1-10'un "spec sürümü CRITERION kuralıyla" eşiği bu kararın sonucuna bağlı, bu yüzden "koşullu geçti" sayıyorum.
- **Ş2 · Başlıksız `append` artık reddediliyor (K-D1.1'in yan etkisi).** P1-D'de kabul edilen "insan belgesinin sonuna paragraf ekle" (`mcp_e2e` check 41) şimdi `denied`. Sözleşmede "istemci kuralı" olarak yazılı ve EXCLUDED'da kararla gerekçeli. Ama P1-E istemcisinin ajan yönergesine konmazsa ilk kullanıcı deneyiminde en sık görülecek ret bu olacak.
- **Ş3 · Eski indeksli vault'ta `search` `internal`/500 dönüyor.** Durum açıkça kurtarılabilir (`mdk index build`) ve beklenen bir durum. Sözleşme §3 `internal`'ı ise "raporlanması gereken hata" diye tanımlıyor. P1-E istemcisi bunu ayırt edemez. `mdk serve` indeks yokken kendiliğinden kuruyor, biçim eskiyken kurmuyor.
- **Ş4 · D1-13 kapısı HTTP uçlarını kapsamıyor** (V9). Şu an doğru, ama bir uç düşerse kapı yeşil kalır.
- Diğer:
  - `checkpoint {}` adlı bir durumda `exists` dönüyor (V7, sözleşmede yazılı).
  - D1-12'nin P1-D `D-2/D-7/D-9` hariçleri bütün alt ağaçlar (V10).

**Hüküm: Şu koşullarla güvenilir.**
- D1-1..D1-11 ve D1-13 kendi istemcim ve kendi vault'larımla bağımsız olarak yeniden üretildi.
- P1-D doğrulamasının Ş1'i (ajan canonical'ı silebiliyordu) ve Ş2'si (boru hatlı determinizm) gerçekten kapanmış. Dosya, olay ve iz düzeyinde boru hatlı determinizmi 6 bağımsız vault çiftinde gördüm.
- Koşullar:
  - (1) Ö1 yapılmadan, yani `compare_p1d1`'in belirlenimsiz alanları düzeltilip `verify.sh pair` yeniden çıkış 0 vermeden, D1-12 "geçti" sayılmamalı. P1-E başlangıcı buna bağlanmalı.
  - (2) K-D1.4-T (spec sürüm düzeyi) P1-E'nin ilk görevinden önce bir kararla kapatılmalı; D1-10'un sürüm yarısı ona bağlı.

**Öneriler (≤ 3):**
1. **Ö1 (Ş0):** Bir dondurma istisnasıyla (F16 emsali) iki yoldan birini seçin.
   - `d1_5.json`'un `responses_*_equal` alanlarını `compare_p1d1`'in karşılaştırmasından çıkarın (bilgi alanı olarak kalsın).
   - Ya da D1-5'in yanıt karşılaştırmasını yalnız yazan ve iz yazan çağrıların yanıtlarıyla sınırlayın; bunlar sözleşme §5'e göre belirlenimlidir.

   Ardından `verify.sh pair`'i temiz klonlardan yeniden koşturun. Ölçüm kodunun her alanını "sözleşmeye göre belirlenimli mi?" sorusuyla tarayan bir denetim satırı ekleyin.
2. **Ö2 (Ş1):** K-D1.4-T'yi kapatın ve CRITERION'ın majör maddesine "veri" sözcüğünün kapsamını yazın: "bir uygulamanın yazdığı veri" mi, "önceki spec'in izin verdiği her veri" mi. Yama seçilirse SPEC_EDITS'e 1.0.2 okuyucularının hangi `meta` biçimlerini artık okuyamayacağını açıkça not edin.
3. **Ö3 (Ş2–Ş4, P1-E istemcisi öncesi sözleşme):**
   - Eski ya da eksik indeks için kararlı bir kod (ör. `index_stale`) koyun ya da `mdk serve` biçim farkında indeksi kendiliğinden yeniden kursun;
   - "insan belgesine ekleme kendi başlığıyla" kuralını istemci tarafında uygulayın;
   - `contract_check.sh`'e §1 uçlarını ve §3 durum tablosunu gerçek HTTP çağrılarıyla eşleyen bir adım ekleyin.
