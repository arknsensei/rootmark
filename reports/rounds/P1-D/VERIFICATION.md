# P1-D — Bağımsız Doğrulama

- Yönerge: `docs/rounds/P1-D/VERIFY.md`, `docs/DEV_PROCESS.md` §5.
- Doğrulanan commit: `c173bc1` (READY; READY'nin andığı `6854945` + yalnız READY.md). Ölçülen commit `a6b52fe`. `a6b52fe..c173bc1` farkı yalnız `docs/rounds/P1-D/progress/` ve `reports/rounds/P1-D/`'de (kod, betik, fixture farkı yok).
- Ortam: temiz klon (oturum scratchpad'i, ana depodan `git clone`), macOS (Darwin 27.0.0), rustc 1.92.0, Node 24.3.0, uv 0.11.8, model `minilm-l6-v2/e73168f9` (`make models`: sha256 + blake3 manifest ok).
- Kendi RUN'larım: A `20261016T065615Z-c173bc1`, B `20261016T071911Z-c173bc1` (`scripts/verify.sh pair <klon> A B`).
- Süreç notu: klonu kurmadan önce ana dizinden yalnız `VERIFY.md`, `GOAL.md`, `READY.md`, `DEV_PROCESS.md` §5 okundu (ağaç temizdi). Bütün diğer okumalar ve koşturmalar klonda ve kendi çalışma dizinlerimde yapıldı. Kendi vault'larım: `spec/fixtures/mini_vault` + `NORTH_STAR.md`, `docs/ARCHITECTURE.md`, `docs/PACKAGES.md`, `docs/DEV_PROCESS.md` (16 belge, insan içeriği, `mdk ingest-dir`). Her deney bu vault'un ayrı bir kopyasında yapıldı. Ana dizine yazılan tek dosya bu rapor.

## V1 · Çift koşturma — geçti (kapsam gözlemiyle, bkz. Ş2)
- `verify.sh pair` **çıkış 0**: zincir A 1 395 s, zincir B 1 375 s.
- `pair_compare.json` (`compare_p1d pair`): **99 985 alan, `equal: true`, `differing_fields: {}`, `bytes_equal: true`**. P1-D bayt kümesinin 10/10'u eşit: iki MCP e2e transkripti, `cli_promote.jsonl`, e2e vault'unun `events.jsonl`/`traces.jsonl`/`proposals.jsonl`'u, `espresso.md`, `agent/plan.md`, `scene.json`, Inspector `tools_list.json`. Her iki RUN'da `p1d_checks.json` D-1..D-11 `true`.
- Kendi A'm ↔ commit'lenmiş resmi RUN `20261015T210419Z-a6b52fe`: `scene.json`, `tools_list.json`, `mcp_e2e.json`, `http_e2e.json`, `tamper_p1d.json` `cmp` ile eşit. P1-D bayt kümesinin blake3'leri resmi `pair_compare.json`'dakilerle aynı.
- **Sahne vault'ları** (zincir bunları karşılaştırmıyor, yalnız `scene.json`'u): `MDK_SCENE_KEEP=1 scripts/scene_agent.sh` iki kez koşturuldu.
  - `events.jsonl` (116 satır), `traces.jsonl` (3), `proposals.jsonl` (1) bayt eşit.
  - Vault `diff -r` eşit, `.mdkernel/index/` dahil.
  - Üç MCP transkripti (`ta`, `ta2`, `tb`) bayt eşit. Bunlarda zaman damgası ya da token yok, hariç tutulacak alan kalmadı.
- Kasıtlı bozma (benim): B kopyasının `traces.jsonl`'unda 1 bayt → `compare_p1d pair` çıkış 3. `scene.json`'da 1 alan → çıkış 3. Geri yükleme → çıkış 0.
- Zincir içi bozmalar: `tamper_p1d` 18/18 reddedildi, 3/3 kontrol geçti. P1-C 6/6.

## V2 · MCP Inspector (D-1) — geçti
- Resmi Inspector'ı kendim kurdum, iki sürüm: `@modelcontextprotocol/inspector@2.9.0` (zincirin sabitlediği) ve `@2.10.1` (2026-10-08). Her biri ayrı bir `npm install` dizininde.
- `tools/list` iki sürümde de bayt eşit: **10 araç** (`status, search, read_block, history, blame, propose, promote, checkpoint, checkpoints, trace`). VERIFY/GOAL metni "9" diyor, GOAL tablosu 10 araç listeliyor (ASSUMPTIONS D5-1). Girdi alanları GOAL tablosuyla birebir. `propose`'ta ek olarak `mode` (`create|append|replace_section`) ve `heading` var.
- **Bağımsız şema doğrulaması** (`ajv@8.17.1` `Ajv2020`, `strict`, kendi betiğim; sunucunun `mcp_schemas.rs`'i kullanılmadı): **10/10**. Kontroller:
  - her `inputSchema` 2020-12 meta-şemasına göre geçerli, `type: object`, `additionalProperties: false`;
  - her araç için yazdığım geçerli örnek kabul, geçersiz örnek (eksik zorunlu alan, yanlış tip, bilinmeyen alan) red;
  - `description` boş değil;
  - enum'lar: `search.mode` `hybrid|lexical|flat`.
- Inspector 2.10.1 ile her araç bir kez çağrıldı: 10/10 `isError: false`. `promote` → `decision: denied`.
- **Geçersiz parametre → yapılandırılmış hata:**
  - `search mode=bogus` → `isError: true`, `unknown search mode "bogus"`.
  - `read_block` (`lineage_id` yok) → `missing field`.
  - `history extra=1` → `unknown field`.
  - Olmayan araç → Inspector `tool_not_found`.
  - Ham JSON-RPC: `k:"abc"`, `k:-1`, `k:0`, `query:""` → hepsi `isError: true`, iz yazılmadı. Inspector, şemaya uymayan `k=abc`'yi istemci tarafında düşürüyor, bu yüzden sunucunun reddini ham JSON-RPC ile ayrıca gösterdim.
- Hata biçimi MCP'nin araç hatasıdır (`isError` + metin). `structuredContent`'te makine okunur bir hata kodu yok (bkz. Ö3).
- Kendi bozmalarım (`scripts/mcp_inspector.sh <out> <liste>`), zincirin 6 şema bozmasından farklı:
  - `search.mode` enum'una değer eklendi;
  - `propose.description` boşaltıldı;
  - `read_block.required` daraltıldı;
  - `trace` çıkarılıp bir araç çiftlendi.

  Dördü de çıkış 1. Değiştirilmemiş liste çıkış 0.

## V3 · `propose` (D-3, D-11) — geçti
Kendi vault'umda, MCP stdio üzerinden (istemci `verifier/0.1`) 3 `propose`: yeni belge `agent/zeolite.md` (`create`), `espresso.md`'ye `append`, `agent/zeolite.md`'de `replace_section "Care"`.
- `grep -cE "lineage|version_id|mdkernel"`: `agent/zeolite.md` **0**, `espresso.md` **0**.
- Dosya diff'i: `espresso.md` için `diff` yalnız ekleme (`22a23,25`). Eklenen metin ve önüne sunucunun koyduğu tek boş satır. Silinen ya da değişen satır yok. Vault'ta yalnız bu iki dosya değişti.
- `blame`: yeni blokların hepsi `layer: proposal`, `author_kind: Agent`, `source: verifier/0.1` (zeolite 4/4, espresso eklenen 1/1). Eski insan blokları `canonical`.
- `search` varsayılan (lexical, hybrid k=50, flat k=50): proposal hit **0**.
- `include_proposal: true`: 3 hit, hepsi `layer: proposal` (2 zeolite, 1 espresso), canonical'lardan sonra.
- **Kötü girdiler: 10/10 hata.** GOAL'ün 4'ü:
  - `../escape.md`;
  - vault dışı mutlak yol;
  - 10 MB + 1 bayt (`max_propose_bytes` 1 MiB);
  - `replace_section` için olmayan başlık.

  Ek 6'sı:
  - var olan dosyaya `create`;
  - `.mdkernel/evil.md`;
  - vault dışını gösteren sembolik bağ;
  - `.txt`;
  - `a/../../escape.md`;
  - boş `reason`.

  Öncesi/sonrası vault dosyaları + `events.jsonl` sha256 **aynı**. Vault dışında dosya oluşmadı.
- `serve --watch` altında `propose` (HTTP): 3 s sonra bloklar hâlâ `proposal`/`Agent`. İzleyici ajan yazımını insan yazımı olarak yeniden almıyor.

## V4 · Terfi (D-4) — geçti
- MCP `promote` → `isError: false`, `structuredContent` `{decision: denied, policy: human_only, policy_version: "1", actor: {id: verifier/0.1, kind: agent}}`, `events.jsonl` değişmedi.
- CLI `mdk promote --actor human:you --reason …` (belgenin 4 soyu) → `Promoted`, `meta: {actor: {id: you, kind: human}, reason, policy: human_only, policy_version: "1", from: proposal, to: canonical}`. Ardından `search` (varsayılan) `agent/zeolite.md`'yi `canonical` buluyor. `--actor agent:x` → çıkış 1, denied.
- HTTP'de `X-Consumer: human:me`, `human`, `policy:auto` → hepsi `actor.kind: agent`, `denied`. Başlıkla insan kimliğine geçilemiyor.

## V5 · İz (D-5) — geçti (bir bulguyla, Ş2)
- Bir MCP oturumunda (`verifier-v5/0.1`) 50 `search` (3 mod, k 1–7) + 50 `read_block` (6 belge): 100 yanıt, 0 hata, **100 yeni iz**, hepsi `consumer: verifier-v5/0.1`.
- Inspector çağrısının izi `consumer: inspector-cli/2.10.1`. Değer handshake'teki `clientInfo` ad/sürümünden geliyor.
- Her yanıtın `trace_id`'si bir ize karşılık geliyor: 100/100. `checkpoint_id` eşit. `search` için `returned` = hit'lerin `(doc_path, lineage_id)` kümesi, `read_block` için tek blok.
- **5 `trace_id` formülle bağımsız** (`blake3==1.0.5`, kendi betiğim, `H = blake3[:32]`, spec §8): **5/5 eşit** (at_seq 0, 7, 33, 61, 100).
- Zincirde: 200 çağrı → 200 iz (`mcp-e2e/1.0`), A ve B.

## V6 · Checkpoint (D-6) — geçti
`checkpoint` → `adf30ee8…`. Sonra `filter.md`'ye `propose append` ("wombat") + CLI terfi:
- `search --at adf30ee8…` → 0 hit (eski durum);
- `search` → `filter.md` `canonical` (yeni checkpoint `730a4c0a…`).

MCP ve CLI aynı. Not: bir `search`/`read_block` güncel durumun adsız checkpoint'ini zaten oluşturduysa, aynı durumda `checkpoint {name: "v6"}` hata veriyor (`… already exists with name None`). Bu ASSUMPTIONS D5-3'te yazılı (bkz. Ö3).

## V7 · HTTP (D-7) — geçti
- token'sız → 401; yanlış token → 401; doğru → 200 (`GET /v1/status`, `POST /v1/search`).
- `X-Consumer` yok → 400; boş → 400.
- Ek: olmayan araç → 404; eksik alan → 422; izdeki consumer `http-v7`.
- Bağlama reddi (çıkış 1, başlamadan): `0.0.0.0`, `[::]`, `192.168.1.10`, `127.0.0.2`, `[::1]`, `localhost`. `lsof`: yalnız `127.0.0.1:PORT` dinleniyor.
- `.mdkernel/server.token`: `-rw-------`, mod **600**, 64 hex.
- Gözlem: `Host` başlığı denetlenmiyor (`Host: evil.example` → 200). Token zorunlu olduğu için DNS rebinding ile okuma yapılamaz, yalnız not.

## V8 · İzleyici (D-10) — geçti
- `serve --watch --http`, 40 dışarıdan insan düzenlemesi (3 mevcut belge + yeni `notes/new_human.md`), HTTP `search` ile yoklama. Ölçüm, arka planda çift zincir koşarken yapıldı (load ~6–8).
  - 40/40 bulundu, 1 s altında 40/40.
  - **p50 111 ms, p95 128 ms, en çok 258 ms**.
  - Katman `canonical`, `author_kind: Human`, `source: mdk-serve-watch`.
  - SIGTERM → `compact (shutdown)`, çıkış 0.
- Zincirde: p95 130.1 ms (A), 127.3 ms (B).

## V9 · Uyumluluk ve regresyon (D-9) — geçti
- `make conformance` (klonda): **level 3, 31/31**. Zincirde de 31/31.
- `regress_vs_p1c1.json` (P1-C.1 resmi RUN'ı ile): **99 341 alan, `equal: true`, fark 0**.
- `check_frozen.sh` OK, `check_deps.sh` OK, `test_check_deps.sh` OK.
- `p1c1-final..a6b52fe`: `spec/` farkı yok. `research/eval` için yalnız yeni dosyalar (`compare_p1d.py`, `report_p1d.py`, `run_p1d_checks.py`), `verify.sh` +29/−2 (F4).
- Testler: `core/tests/{cli_governance,cli_index}.rs` → `server/tests/` taşındı. `chunk.rs`'in 2 CLI testi → `server/tests/cli_chunk.rs`, 5 kütüphane testi `core/tests/chunk.rs`'te kaldı. Kaybolan test yok. `cli_index` farkı `mdk-index` alt sürecinin kalkmasından (`missing_mdk_index_is_an_error` → `mdk_index_binary_is_not_used`). Zayıflatılmış bir doğrulama görmedim.
- Not: zincirdeki P1-C kaydında C-3 ve C-11 `false`. Resmi RUN'da da öyle: P1-C'den bilinen Δ −0.0136, ve spec sürümünün 1.0.2'ye çıkması. Bu tur için ölçüt değil.

## V10 · Sahne — geçti
`scripts/scene_agent.sh`'i iki kez koşturdum: 9/9 adım, sıra ve sonuçlar beklendiği gibi.
1. status `human_only`
2. propose → 2 blok `proposal`/`Agent`, düz md
3. varsayılan search 0
4. `include_proposal` → `proposal`
5. promote `denied`, 0 olay
6. CLI 2× `Promoted`/`canonical`, `actor human:editor`
7. search → `canonical`
8. trace 3 iz, `[0, 2, 2]`
9. checkpoint = 7. adımın checkpoint'i

`scene.json` iki koşturmada ve commit'lenmiş iki kopyayla (`reports/rounds/P1-D/scene.json`, RUN içindeki) bayt eşit.

## V11 · Karar kuralı — FINAL ile aynı
- GOAL'den türetilen kural:
  - D-3/D-4/D-5 kaldı → yönetişim ihlali, P1-E başlamaz;
  - D-8 kaldı → P1-E başlamaz;
  - D-1/D-7 kaldı → P1-E başlar, 0.1.0 bekler;
  - D-1..D-11 geçti → P1-E başlar.
- Benim RUN'larımda (A, B) ve elle yaptığım bağımsız ölçümlerde D-1..D-11'in hepsi geçti → **P1-E başlar. FINAL ile aynı.**
- Kuraldaki boşluk: D-2, D-6, D-9, D-10, D-11'den biri kalırsa ne olacağı GOAL'de tanımlı değil. Bu turda gerçekleşmedi.

## Sonuç

**İhlal:** GOAL ölçütlerine göre yok.

**Fark:** kendi çift koşturmamda 0 (99 985 alan, bütün baytlar). Kendi RUN'umla resmi RUN arasında P1-D yapıtları bayt eşit. Sahne vault'ları bayt eşit.

**Şüphe / gözlem:**
- **Ş1 · Ajan, insan `canonical` içeriğini onaysız silebiliyor (K-D4-T, açık).** Kendi vault'umda MCP `propose mode=replace_section heading="Fire"` ile `safety.md`'nin insan yazımı güvenlik paragrafı ("class D extinguisher…") terfi adımı olmadan dosyadan ve varsayılan `search`'ten anında kalktı. `BlockDeleted` olayının `meta`'sı yalnız `{matcher_version}`, silen ajan olayda yok. GOAL ölçütleri bunu ölçmüyor ve takım açık karar talebi olarak yazmış, bu yüzden ihlal saymıyorum. Ama NORTH_STAR'ın "kim tarafından … bilinir; onaylanmamış hiçbir şey bilgi sayılmaz" vaadiyle ve GOAL'ün "örtük onay yok" maddesiyle gerilim içinde. P1-E istemcileri bu yolu açacak.
- **Ş2 · Determinizm yalnız kilit adımlı istemciler için.** Sunucu bir oturumdaki istekleri eşzamanlı işliyor (`spawn_blocking`; yanıtlar sırasız döndü). Aynı 100 çağrıyı boru hattıyla (yanıt beklemeden) iki özdeş vault kopyasına gönderdim. Sonuç:
  - `traces.jsonl` farklı. Küme bile değil, çünkü `at_seq` ve dolayısıyla `trace_id` sıraya bağlı.
  - 40 boru hatlı `propose`'ta `events.jsonl` ve `espresso.md` farklı (ekleme sırası rastgele).

  Bozulma yok: 20/20 ek var, log ve dosya tutarlı. D-8 ve sahne, istekleri tek tek bekleyen `mcp_client` ile ölçülüyor, yani ölçüt karşılanıyor. Ama bu sınır hiçbir belgede yazılı değil. Gerçek MCP istemcileri paralel araç çağrısı yapabilir.
- Diğer:
  - "9 araç" ↔ 10 araç (D5-1).
  - Araç hataları makine okunur kod taşımıyor.
  - Örtük adsız checkpoint, aynı durumu sonradan adlandırmayı engelliyor (D5-3).
  - HTTP `Host` denetimi yok.
  - CLI `mdk propose` MCP denetimlerinden geçmiyor (K-D4-T b).

**Hüküm: Şu koşullarla güvenilir.** D-1..D-11 bağımsız olarak yeniden üretildi, karar kuralının sonucu (P1-E başlar) doğru. Koşullar:
- (1) Ş1 (K-D4-T) P1-E istemcileri `replace_section`'ı ya da insan bloklarını değiştiren `propose`'u kullanıcıya açmadan kapatılmalı.
- (2) D-8'in determinizm iddiası "kilit adımlı istemci" kapsamıyla okunmalı.

**Öneriler (≤ 3):**
1. **Ö1 (Ş1):** P1-E'den önce `replace_section` (ve `append` dışındaki her `propose`) insan `canonical` bloklarını silemesin ya da değiştiremesin. Ya varsayılan politikada reddedilsin, ya da silme/değişiklik `proposal` olarak beklesin ve canonical içerik terfiye kadar görünür kalsın. Silmenin aktörü olaylardan izlenebilir olsun (K-D4-T a). Bunu bir D-3 benzeri ölçütle sınayın: "ajan yazımından sonra varsayılan `search`'ün canonical sonuçları değişmez".
2. **Ö2 (Ş2):** Sunucu, yazan çağrıları ve iz yazan çağrıları geliş sırasıyla seri işlesin (oturum başına sıra ya da vault başına tek yazar kuyruğu). Alternatif: sınırı spec/istemci sözleşmesine yazın. D-8'e boru hatlı bir istemci vakası ekleyin (aynı istek dizisi → aynı `traces.jsonl`/`events.jsonl`).
3. **Ö3 (istemci sözleşmesi, P1-E öncesi):**
   - Araç hatalarına `structuredContent` içinde kararlı bir hata kodu koyun (`invalid_arguments`, `path_outside_vault`, `too_large`, `heading_not_found`, …).
   - Var olan adsız checkpoint'in adlandırılmasına izin verin ya da `checkpoint {name}`'i `search`'ten önce zorunlu kılan akışı belgeleyin.
   - GOAL şablonundaki araç sayısını tabloyla hizalayın.
   - Karar kuralına tanımsız kalan D-2/D-6/D-9/D-10/D-11 dallarını ekleyin.
