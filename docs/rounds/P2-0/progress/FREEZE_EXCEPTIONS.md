# P2-0 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir. Satırın notu `P2-0/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · Z0 · `scripts/contract_check.sh` — `status` §2 satırının bütün alanları (P1-E.1 doğrulaması öneri 2)
- **Ne:** sözleşmenin §2 `status` satırının yanıt sütunu okunur; her üst düzey ad ve her `{…}` grubunun adları (`…` alan değildir) bir yol olur (`status_paths`: `vault`, `spec_version`, `mdk_version`, `level_claimed`, `embedder`, `index.{state,format,expected_format,auto_rebuild}`, `last_checkpoint`, `policy.{promotion,edit}`, `server_policy.{server,index,identity}`). `index_state_check` dört indeks durumunun her birinde GET ve POST `status` yanıtında her yolun bulunmasını ister (değer `null` olabilir: `last_checkpoint`, `embedder`); eksik yol `missing`'e `http-status-fields:<durum> [<yol>…]` olarak girer ve o durumun `ok`'unu düşürür. Satırda alan listesi yoksa `contract:status fields`. Çıktıya `http.status_fields`. Başlık yorumu (+5 satır). Diğer adımlar, `index {…}` denetimi, kod istekleri, aktarım kuralları aynen.
- **Neden:** P1-E.1 doğrulaması V6 / öneri 2: üst düzey `last_checkpoint` kaldırılınca denetleyici çıkış 0 verdi; yalnız CI'da atlanan gerçek Obsidian e2e yakaladı. TASKS Z0: "`contract_check.sh` `status` §2 tüm üst düzey alanlar".
- **Önce/sonra bit eşitliği** (güncel ağaç, güncel sunucu): eski denetleyici (`HEAD:scripts/contract_check.sh`) çıkış 0; yeni denetleyici çıkış 0; iki çıktı, yeni `http.status_fields` alanı silindikten sonra `jq -S` ile bayt eşit.
- **Kasıtlı vaka:** `last_checkpoint`'i `status`'tan kaldıran derleme (`ROOTMARK_BIN`): eski denetleyici çıkış 0 (yakalamadı), yeni denetleyici çıkış 1, `missing`: dört durumun her birinde iki kez `http-status-fields:<durum> ["last_checkpoint"]` ve `http-status-index:<durum>`. Z7'nin tamper betiği bu vakayı zincire alır.
- **Gözden geçirici onayı:** Z0.review.md.

## F2 · Z0 · `scripts/release_check.sh` — R16 ve R14 (K-P2-0.3; P1-E.1 doğrulaması öneri 1)
- **Ne:** yeni madde **R16**: `docs/RELEASE.md` "İnsan adımları" bölümünün 1. maddesinin adını verdiği `reports/rounds/<tur>/VERIFICATION.md` var ve son `**Hüküm: …**` satırı "Güvenilir". **R14** "etiket yok"tan şuna döner: `v<ver>` etiketi yok (geçer), ya da var ve R16'nın hükmü "Güvenilir" ve etiketin commit'i en yeni `ROUND_START`'ın kendisi ya da atası (etiketi bir tur atmaz). Başlık yorumu R1–R16. Diğer maddeler, sıraları, `release_check.json` biçimi aynen; madde sayısı 16 → 17.
- **Neden:** TASKS Z0 "`release_check.sh` 17/17 (yeni madde)" ve "`RELEASE.md` insan adımı 1 → VERIFICATION bağı". `v0.1.0` GH-0'da insan tarafından atıldı (K-GH.4); eski R14 ("etiket yok") bundan sonra her koşuda FAIL'di, oysa madde "etiketi tur atmaz" kuralını denetler.
- **Önce/sonra bit eşitliği** (aynı `dist/`, `be923d6` ağacı, Z1'in commit'lenmemiş spec düzenlemeleri kenara alınmış): eski betik (`be923d6~1:scripts/release_check.sh`) 15/16, yeni 17/17. İki `release_check.json`'un R14 ve R16 dışındaki 15 maddesi (`id`, `item`, `ok`, `detail`) `jq -S '[.items[] | select(.id != "R14" and .id != "R16")]'` ile **bayt eşit**; yalnız R14 değişti (eski FAIL "etiket yok", yeni ok) ve R16 eklendi. Kasıtlı vaka: Z0.md "Ölçüm".
- **Gözden geçirici onayı:** Z0.review.md.

## F3 · Z2 · `index/tests/conformance/eval_c3.rs` — `unverified` ikinci geçişte (spec 2.0.0 §9)
- **Ne:** C3 değerlendiricisinin arama gözleminde iki satır: `n_p` (`proposal` hit sayısı) artık `proposal` ve `unverified` hitleri birlikte sayar; sıralama denetimindeki `match h.layer` kolu `Layer::Proposal | Layer::Unverified => seen_prop = true`. Başka hiçbir gözlem, alan adı ya da işlem değişmedi (+6/−2 satır).
- **Neden:** `Layer`'a dördüncü değer (`Unverified`, K-P2-0.4) eklenince eşleşme tam olmadığından değerlendirici derlenmez; `rootmark conformance` bu dosyayı ikiliye gömer. Spec 2.0.0 §9: `proposal` ve `unverified` aynı ayrı geçişte, `k` sınırına birlikte.
- **Önce/sonra bit eşitliği:** `make conformance` (yeni değerlendirici, yeni çekirdek): Seviye 3, 34/34; 34 testin her birinin `id`, `result`, `actual_hash`, `expected_hash`'i P1-E.1 resmi RUN'ının `conformance_p1d1.json`'uyla `jq -S` sonrası **bayt eşit** (var olan fixture'larda `unverified` yoktur; gözlem aynı). Eski değerlendirici yeni çekirdekle derlenmez (önce = P1-E.1 kaydı).
- **Gözden geçirici onayı:** Z2.review.md.

## F4 · Z2 · `server/tests/serve_watch.rs` — D-10'un politikası açık (K-P2-0.4)
- **Ne:** `d10_watched_human_edit_is_indexed_within_a_second` sunucuyu başlatmadan önce vault'a `default_recipe()` + `[attestation] unattested = "canonical"` yazar (+11 satır, yorum dahil). Ölçüm, eşikler ve SIGTERM testi aynen.
- **Neden:** D-10 izleyicinin aldığı insan düzenlemesini ölçer (P1-D: dış düzenleme insan içeriği, `canonical`). Spec 2.0.0 §7.1 ile varsayılan `unverified` oldu; P1-D davranışı artık açık `unattested = "canonical"` politikasıdır. Test aynı davranışı ölçmeye devam eder; varsayılan politika `server/tests/p2_0_attest.rs`'te ölçülür. Testi geçirmek için değil, kararın değiştirdiği varsayılanı testin ölçtüğü politikaya sabitlemek için (emsal P1-E.1 E1.1-3).
- **Önce/sonra:** yeni sunucu, eski test: 1/2 (D-10: arama hiti `canonical` değil — `unverified`); yeni test: 2/2. Bayt eşitliği uygulanamaz (test bir kayıt yazmaz, koşar); ölçüm kodu, eşikler ve doğrulamalar aynen, yalnız vault kurulumuna politika satırı eklendi.
- **Gözden geçirici onayı:** Z2.review.md.

## F5 · Z2 · `scripts/http_e2e.sh` — W-1'in politikası açık (K-P2-0.4)
- **Ne:** "watch" adımından önce vault'un `policy.toml`'una `[attestation] unattested = "canonical"` eklenir (+4 satır, yorum dahil). Adımlar, kayıtlar, eşikler aynen.
- **Neden:** F4 ile aynı: W-1 ("dış insan düzenlemesi: canonical, Human, rootmark-serve-watch") ve onun düzenlemesini soğuk indekste arayan S-1, P1-D'nin izleyici kuralını ölçer; bu kural 2.0.0'da `unattested = "canonical"` politikasıdır.
- **Önce/sonra:** yeni sunucu ile eski betik: 10/15 (W-1 ve S-1 FAIL: düzenleme `unverified`, varsayılan aramada yok; D7-3, H-1, W-3 FAIL — P1-E.1 resmi RUN'ında da aynı üç FAIL, tarihsel D-7); yeni betik: 12/15 (W-1, S-1 ok; aynı üç tarihsel FAIL). W-1 ve S-1 dışındaki 13 kayıt (`id`, `pass`, `detail`) iki koşuda `jq -S` ile bayt eşit.
- **Gözden geçirici onayı:** Z2.review.md.

## F6 · Z2 · `scripts/contract_check.sh` — §2.6 HTTP'ye özel uçlar (K-P2-0.5)
- **Ne:** (1) araç listesi §2 tablosundan (`## 2.` → `### 2.1`) okunur; §2.6'nın satırları araç sayılmaz. (2) §2.6 satırları (`http_only`): hiçbiri MCP `tools/list`'te olmamalı; `attest_session` 200 ve `session_id`, 64 hex `key`, `ttl_s`, `stage`; o anahtarla (`openssl dgst -sha256 -mac HMAC`) yapılmış MAC'li `attest` 200 ve satırdaki her alan; yanlış MAC 403 `denied`. Eksik: `http-only:<ad>`; satır yoksa `contract:§2.6 …`. Çıktıya `http.http_only`. Başlık yorumu. Diğer adımlar aynen.
- **Neden:** sözleşme her ucu gerçek bir HTTP çağrısıyla eşler (§ başlığı); K-P2-0.5'in iki yeni ucu §2 araç tablosuna giremez (MCP'de yoktur, `tools.equal` bozulurdu), §2.6'da ayrı tablodur.
- **Önce/sonra:** yeni sözleşme ve sunucu ile Z0 denetleyicisi çıkış 1 (`tools.equal: false`, `missing`: `tool:attest`, `tool:attest_session` ve iki "no request" ucu — §2.6 satırlarını araç sanıyor); yeni denetleyici çıkış 0. Z0 çıktısından bu iki adın ve onların `missing` satırlarının çıkarılmasıyla, yeni çıktıdan `http.http_only` silinince iki çıktı `jq -S` ile bayt eşit (araçlar, kodlar, uçlar, aktarım, `status_index`, `status_fields` — artık `attestation.*` dahil 19 yol).
- **Gözden geçirici onayı:** Z2.review.md.

## F7 · Z4 · uyumluluk değerlendiricileri: `core/tests/conformance/eval_c2.rs`, `core/tests/conformance.rs`, `core/tests/conformance_c2_gen.rs`, `index/tests/conformance_c3.rs`, `index/tests/conformance_c3_gen.rs` — C2-11, C2-12, C3-08 (spec 2.0.0, K-P2-0.4)
- **Ne:** `eval_c2.rs`'e yeni işlemler (yalnız ekleme): `restore_store`, `fingerprint`, `unattested_ingest`, `external`, `attest`, `attestations`, `meta_tamper`, `attestation_tamper`; vakanın `window_ms`'i (açık zamanlı pencere); yardımcılar `copy_dir`, `attributed`, `Run::unattested`, `Run::open_err`. Var olan işlemler ve gözlemleri aynen. `conformance.rs`: `LEVEL2` 12 test, rapordaki `spec_version` `"2.0.0"`. `conformance_c2_gen.rs`: dizin sayısı 12. `conformance_c3.rs`: `LEVEL3` 8 test, `LOWER_TOTAL` 29. `conformance_c3_gen.rs`: dizin sayısı 8. (`eval_c3.rs`'in `unverified` kolu Z2'de, F3.)
- **Neden:** TASKS Z4 "C2-11, C2-12, C3-08 fixture'ları; `gen_fixtures_*`; `make conformance` 37/37"; GOAL H-P2-0h. C2-12'nin eşleşmesi zamana bağlıdır: değerlendirici çekirdeğin saf penceresini (`attest::Window`) vakanın açık zamanlarıyla çalıştırır, determinizm korunur.
- **Önce/sonra bit eşitliği:** `make conformance`: Seviye 3, 37/37. Eski 34 testin her birinin `id`, `result`, `actual_hash`, `expected_hash`'i P1-E.1 resmi RUN'ının `conformance_p1d1.json`'uyla `jq -S` sonrası **bayt eşit**; yeni üçü geçer. `scripts/gen_fixtures_c2.sh` ve `gen_fixtures_c3.sh` var olan 34 fixture'ın `expected.json`'unu bayt bayt aynı bıraktı (`git status`: yalnız `MANIFEST.json` ve üç yeni dizin). Gömülü `rootmark conformance`: 37/37, spec 2.0.0.
- **Gözden geçirici onayı:** Z4.review.md.

## F8 · Z4 · `spec/fixtures/**` — C2-11, C2-12, C3-08 fixture'ları ve MANIFEST
- **Ne:** yeni dizinler `spec/fixtures/C2-11/` (vaka, belgeler, `pre2_store/` — `v0.1.0` ikilisinin, yani spec 1.0.3 uygulamasının yazdığı bir depo: `ingest-dir` + ajan `propose`), `C2-12/`, `C3-08/` (vaka; belgeleri `../C3-07`'den okur); `MANIFEST.json`'a yalnız yeni dosyaların satırları eklendi (27 satır; silinen ya da değişen satır yok). Var olan fixture dosyaları aynen.
- **Neden:** F7 ile aynı. `pre2_store` C2-11'in "2.0.0 öncesi bir günlük katmanlarını ve parmak izini korur" vakasının girdisidir: açılıştan sonraki `events_hash` (`bec30291346b6d640f6bc725c9b5f943`) `v0.1.0`'ın aynı depo için yazdığı `rootmark events-hash` ile eşit.
- **Önce/sonra:** F7 kaydı; `git diff --stat spec/fixtures`: yalnız `MANIFEST.json` +27.
- **Gözden geçirici onayı:** Z4.review.md.

## F9 · Z4 · uyumluluk sayısı 34 → 37: `scripts/test_tamper_p1c.sh`, `scripts/test_tamper_p1d1.sh`, `scripts/release_local.sh`, `scripts/release_target.sh`, `scripts/release_check.sh`
- **Ne:** uyumluluk raporunun toplamını `34` diye sabitleyen ifadeler `37`'ye (ve ilgili yorum/etiket metinleri); başka değişiklik yok.
- **Neden:** spec 2.0.0 ile Seviye 3 = 37 test (K-P2-0.4). Bu betiklerin denediği şey "Seviye 3'ün bütün testleri geçti"dir; sayı spec sürümünün özelliğidir. Değiştirilmezse doğrulama zinciri (`verify.sh`, P1-C/P1-D.1 tamper adımları) ve yayım denetimi 37/37'lik doğru bir raporu reddederdi.
- **Önce/sonra:** `bash -n` temiz; `git diff` yalnız sayı ifadeleri ve yorumlar. Zincirde (Z7) ve yayım denetiminde yeni değerle koşar; eski değerle 37/37'lik rapor her birinde FAIL'dir (sayı eşitsizliği).
- **Gözden geçirici onayı:** Z4.review.md.

## F10 · Z4 gözden geçirmesi · `core/tests/conformance/eval_c2.rs` (biçim), `spec/fixtures/**` (C2-11, C2-12 vakaları)
- **Ne:** (1) `eval_c2.rs`: `rustfmt`'in istediği satır kırmaları (3 yer; F7 commit'inde biçimlendirilmemişti, `make lint` kalıyordu); belirteçler aynen, rustfmt'in eklediği iki sondaki virgül dışında (boşluk ve bu iki virgül çıkarılınca d8c90db ile özdeş). (2) `C2-12/case.c2.json`: W'nin kenarları — tam W önce (60 000 → 62 000: `attested`), W+1 önce (62 000 → 64 001: `unverified`), tam W sonra (70 000 → 72 000: yükselir); `C2-11/case.c2.json`: `Promoted` olayında `unattested_fs` aktörü reddi (spec §7.1 "Reading"); ikisinin `expected.json`'u `gen_fixtures_c2.sh` ile; `MANIFEST.json`'da yalnız bu dört dosyanın satırları değişti.
- **Neden:** Z4 gözden geçirmesi B1 (biçim, `make lint`), B3 (sınırların ikisi de sınanmalı: `<`/`<=` hatası yakalanır), B4 (Reading maddesinin bir vakası eksikti). B2 için spec §8'e cümle (SPEC_EDITS S13): çift kimlik reddi zaten değerlendiricinin vakasıydı.
- **Önce/sonra:** `make conformance` 37/37; diğer 35 fixture'ın `expected.json`'u bayt bayt aynı (`git status`); eski 34 testin hash'leri P1-E.1 kaydıyla eşit kalır (C2-01..C2-10, C1, C3 dosyaları değişmedi). `cargo fmt --check` temiz.
- **Gözden geçirici onayı:** Z4.review.md.

## F11 · Z5 · `clients/obsidian/e2e/p2_0/sessions.ts` (yeniden adlandırma penceresi)
- **Ne:** harness'ın `__p2.watch()`'ı Obsidian'ın `alwaysUpdateLinks` ayarını açar ve açıldığını doğrular (açılmazsa hata). Başka değişiklik yok.
- **Neden:** ön-kayıt commit'inden sonraki ilk ayar koşusu (RUN `20261018T120620Z-12fa90b`) başka notların bağlandığı `cupping.md`'nin yeniden adlandırılmasında Obsidian'ın "bağlantıları güncelle?" penceresinde takıldı (`CDP evaluate timed out: __p2.rename(...)`); koşu fail-fast durdu, hiçbir `W` için sonuç yazılmadı, `tuning.json` yok. ASSUMPTIONS Z5-5.
- **Önce/sonra:** yeniden adlandırma dışı oturumlarda adımlar ve ölçüm aynı (ayar yalnız `renameFile`'ın pencere sormasını kaldırır; duman denemesinde aynı 7 türün kayıtları aynı biçimde, gecikmeler aynı aralıkta). Yeniden adlandırma oturumları önce hiç tamamlanmıyordu, sonra 12/12 tamamlanıyor. Durmuş RUN kullanılmaz; ayar yeni bir RUN'da baştan koşar.
- **Gözden geçirici onayı:** Z5.review.md.
