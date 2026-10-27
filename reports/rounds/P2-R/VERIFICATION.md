# Blok 2 — Bağımsız Doğrulama (P2-B + P2-R; geliştirme turları, dar kapsam; K-P2.4, K-P2R.5)

Doğrulanan commit'ler: P2-R `8cf1c74`, P2-B `7014b6f` (ikisi de `main`'in birinci ebeveyn geçmişinde). Yönerge: `docs/rounds/P2-B/VERIFY.md` VB2–VB9, `docs/rounds/P2-R/VERIFY.md` VR2–VR7 ve aynı dosyanın ekindeki VR1′ (a)–(c) (K-P2R.5; ek önce `p2-v/open` dalından, `d4eb038`, okundu; PR #33 doğrulama sırasında `main`'e `aa1e615` olarak birleşti, `docs/rounds/P2-R`, `docs/rounds/P2-B`, `DECISIONS.md`, `DEV_PROCESS.md` içeriği iki commit'te aynı). Tarih 2026-10-27, süre ≈ 2,5 saat.

**Oturum ve temiz klon (DEV_PROCESS §5).** Ayrı oturum (ne uygulama ajanının ne orkestratörün oturumu). `https://github.com/arknsensei/rootmark.git` scratchpad'e klonlandı, `git checkout 8cf1c74`; bütün koşturmalar orada ve oradan kurulan iki ikilide (`8cf1c74`: FAST zincirinin temiz klonunun `target/release/rootmark`'ı; `7014b6f`: aynı klonda ayrı bir worktree ve ayrı `CARGO_TARGET_DIR`). Kod, ölçüm kodu ve belge değiştirilmedi; bütün betikler doğrulayıcının kendisinindir (`research/eval`'in dondurulmuş karşılaştırıcısı kullanılmadı, yalnız FAST zinciri kendi içinde koştu). Gerçek Obsidian kullanılmadı. **İki sapma, açıkça:** (1) klondan önce `docs/DEV_PROCESS.md`, `docs/rounds/P2-R/{VERIFY,PROMPT}.md`, `docs/rounds/P2-B/VERIFY.md` ve `docs/DECISIONS.md`'nin K-P2.4/K-P2B.5/K-P2R.1–5 satırları ana çalışma dizininden (o an `p2-v/open`, `d4eb038`, temiz) okundu; içerik uzak daldakiyle aynıdır, başka bir şey okunmadı. (2) Bu oturumun kalıcı belleği orkestratörün tur notlarını içeriyor (aynı bellek dizini); hüküm yalnız aşağıdaki koşturmalara dayanır. Ana çalışma dizini yalnız bu dosyayı commit'lemek için kullanıldı.

## Kapsam sınırı

Bu doğrulama K-P2R.5'in dar kapsamıdır. **Tam `make verify` çifti koşulmadı.** Zamanlamaya duyarlı kümeler — D-A (`Served.quiesce`), P2-0.2'nin `serve --watch` izleme kümeleri (D-A2, D-C, D-10'un gecikmesi) ve gerçek Obsidian (P2-0.3 D-B, `H-P2-0.3b/b′/c`) — bu doğrulamanın **dışındadır**: ne yeniden koşuldu ne de onlar hakkında bir hüküm verildi; yalnız farkların bu kümelerle sınırlı olduğu (VR1′ c) denetlendi. Blok 1'in kontrolleri (`p2l-checks`, `p2p-checks`) FAST'ta koşmaz (R5-1); onlar için kanıt 3. denemenin `p2l_checks.json` ve `p2p_checks.json`'unun A↔B bayt eşitliğidir. R-10 bu doğrulamayla kapanmaz: **kaldı, kapanışı P2-V V-4** (CI'da yeşil tam çift). FAST çifti makinede doğrulayıcının öteki koşturmalarıyla aynı anda koştu (süre yine 1032 s).

---

## P2-B

### VR1′ — P2-B yüzeyleri (VB1'in yerine; ayrıntı P2-R başlığında)
FAST çiftinde (`8cf1c74`) `p2b_checks.json`, `p2b/align.json`, `p2b/det.json`, `p2b/align_regress.json`, `p2b/client_checks.json`, `p2b/obsidian_e2e.json`, `p2b/vscode_e2e.json` A↔B **bayt eşit** (kendi karşılaştırıcım). 3. denemede `p2b_checks.json` A↔B bayt eşit; zincir B'nin `p2b/*` dosyaları arşivde yok, dondurulmuş karşılaştırıcının kaydı (`pair_compare.json`) onları karşılaştırılmış ve farksız gösteriyor. P2-B'nin kendi tam çifti (`20261025T211044Z-cad7663`): `pair_compare.json` `equal: true`, 202 738 alan, fark 0.

### VB2 — Hizalama ölçümü
- **D-NOTES:** manifest 6 kaynak (rust-lang/rfcs, microsoft/terminal, obsidian-tasks, kepano-obsidian, obsidian-dataview — MIT; reactjs/tr.react.dev — CC BY 4.0), her biri sabit revizyonla; **400 notun 400'ünün sha256'sı manifestle eşleşiyor** (kendi betiğim); sınıf sayıları hepsi ≥ 20 (en az `footnote` 25), Türkçe 54; tohum `20261025`; `d_notes.py --check` "verified (400 notes, 11 edge classes)".
- **Ön-kayıt sırası:** `f48eb0e` (ön-kayıt, 19:59) → `7da5725` (F1, 20:02; ilk koşu değer üretmeden durmuştu) → `ca752b0` (ölçüm, 20:10). `p2b_align.py` ve `d_notes.py` o ikisinden sonra hiç değişmedi; manifest yalnız `f48eb0e`'de.
- **Kendi betiğimle yeniden hesap** (`blame --loc` → dilim → `rootmark_core.parse_document` ile tek başına yeniden ayrıştırma → tek blok ve aynı `version_id`; satır↔bayt tutarlılığı; örtü: sıralı, örtüşmesiz, aralıklar blok üretmiyor):
  - `7014b6f`: **155 / 28 111 = 0.00551**, hepsi dilim hatası, hepsi girintili kod (dilim girintiden sonra başlıyor, tek başına paragraf olarak ayrışıyor). FINAL'in değeriyle birebir.
  - `8cf1c74`: **0 / 28 111 = 0.00000** (R-3).
  - Kenar vakaları (11 sınıf × 200, tohumdan yeniden üretildi): `7014b6f`'te `bom` 0.00435, `html` 0.00128, ötekiler 0; `8cf1c74`'te `html` 0.00128 (bağlam kaynaklı iki `<img>` bloğu, FINAL not 2), ötekiler 0.
- **Eşik dalları:** B-3 0.00551 > 0.005 → **kaldı**; B-4 en kötü sınıf 0.00435 ≤ 0.02 → **geçti**.

### VB3 — Kendi kenar vakalarım
İç içe üç düzey callout, boş satırlı HTML (`<div>…\n\n…</div>` iki HTML bloğu), belge sonunda iki dipnot tanımı (biri devam satırlı, son satır satır sonusuz), CRLF + BOM (BOM ilk bloğun diliminde, satır 1), ön-bilgiden sonra boş belge (iki biçim; 0 blok, hata yok), Türkçe + emoji + ZWJ + bayrak başlık, tek satırda iki gömme (boşluklu ve bitişik), sekme ve karışık girintili kod: **9 belge, 22 blok, hata 0**; her dilim tek başına aynı bloğu veriyor, `line_*` ile `byte_*` tutarlı.

### VB4 — Bayat dosya
HTTP ile `blame` → `loc_stale: false`, `content_hash` H₀. Dosyanın başına iki blok eklendi, alınmadı (sunucu `--watch`'sız): `blame` ve `read_block` **`loc_stale: true`**, konumlar ve `content_hash` alınan sürümünkiyle aynı (H₀), yeni dosyanın baytlarına uygulanmış bir ofset yok. `search` (`loc: true`) isabetleri de `loc_stale` taşıyor. İstemciler: `align.ts` ve VS Code `core.ts` `loc_stale`'de "stale … no line is guessed" döner; FAST'taki Obsidian e2e'si bayat durumda `marks: 0`.

### VB5 — `events`
Sayfa boyu 3 ile bütün günlük: dönen nesneler `events.jsonl` satırlarıyla aynı ve **her satırın baytları HTTP gövdesinde aynen**; `kinds` ve `path_prefix` süzgeçleri sıralı alt dizi; ret (400 `invalid_arguments`, ileti alanı adlandırır): `../etc`, `/etc`, `limit` 1001 ve 0 (kırpma yok), `since_seq = head+1`, bilinmeyen tür; `since_seq = head` boş sayfa, `next_seq = head`. Çağrılardan sonra `.rootmark`'ın bütün dosyaları (izler dahil) bayt bayt aynen.

### VB6 — Zaman filtresi
`date_from_path = "notes/{yyyy}/{mm}/{dd}"`: `2026-10-01..2026-10-10` yalnız aralıktaki belge, `undated_excluded: 1`; **bütün dosyaların mtime'ı 2000'e çekildi → isabetler ve `query_hash` aynen**. `date_field = "date"` (çıplak ve tırnaklı-saatli değer): yalnız aralıktaki belge. `trace` ve `trace_filters.jsonl` filtreyi taşıyor. Filtresiz `query_hash` üç vault'ta (yol kaynaklı, alan kaynaklı, tarihsiz tarif) aynı. Bozuk tarifler (iki kaynak birden, `{zz}` deseni, boş alan) `policy lint`'te ve `serve`'de alanı adlandırarak ret.

### VB7 — İki istemci ve sözleşme
- Denetleyiciye **kendi 13 sözleşme dışı çağrım** (`since_seq` dize, `prune`'da bilinmeyen alan ve dize `dry_run`, `limit` 5000, `GET /v1/prune`, `X-Consumer`'sız `prune`, sorgu dizgisi, bilinmeyen uç, 127.0.0.1 dışı adres, `next_seq`'siz `events` yanıtı, dize `archived_before`, §3'te olmayan kod, yanlış durum kodlu `archived`): **13/13 yakalandı**; uygun bir `prune` çağrısı geçiyor.
- Ortak tipler tek kaynak: `clients/obsidian/src/contract.gen.ts`; VS Code onu `../../obsidian/src/api` üzerinden içe aktarıyor, kendi kopyası yok.
- VS Code token'ı her bağlanışta `.rootmark/server.token`'dan okuyor, yalnız bellekte tutuyor (`globalState`, `SecretStorage`, ayar yazımı yok; THREAT_MODEL ve README).
- Kaydetmelerin tasdik edilmediği `clients/vscode/README.md` "Saves are not attested"ta yazılı.
- FAST: 151 çağrı, 12 koşu, sözleşme dışı istek **0**, e2e'de sözleşme dışı yanıt 0 (1 kasıtlı negatif çağrı ayrı).

### VB8 — Süreç
- `8cf1c74`'te `check_frozen` OK, `check_deps` OK, `check_dep_age` 111 bağımlılık OK.
- `ROUND_START` `6a5e911`, `main`'de.
- GOAL'den, P2-B ve P2-R boyunca değişmedi.
- Spec P2-B'de değişmedi; NORTH_STAR Blok 2 boyunca değişmedi.
- Dondurma istisnaları F1–F5'in her biri ne/neden/önce-sonra/gözden geçirici onayı taşıyor; `research/eval` ve `scripts`'te silinen satırlar yalnız F-kayıtlı dosyalarda. Değişen `check_deps.sh`, `check_dep_age.sh`, `gen_client_types.sh` FROZEN.md'de değil.
- Blok 1 önerisi 1 kendi vault'umda: `path_prefix` NFD ve NFC aynı isabet, aynı `query_hash`, normalize `filter` NFC; NFD önekli kural "not in Unicode NFC" ile ret.
- Blok 1 önerisi 3: CHANGELOG "Downgrading to 0.2.0"; `--dry-run` planı FAST'ın B-1 testleriyle (`p2b_checks.json` B-1 geçti).
- **Karar kuralının türetimi:** B-3 kaldı, B-1/B-2/B-4…B-10 geçti → "B-3 ya da B-4 kaldı (geri kalan geçti)" dalı: `loc` deneysel, dilim denetimli kullanım, karar talebi, P2-R başlar. FINAL ile **aynı**.

### VB9 — Vaat sorusu (P2-B)
- Konum türetilmiş: olaylara, kimliklere, `events_hash`'e girmiyor (`events_hash` P2-A'nın vault'larında aynen, FAST R-1/B-2 testleri).
- `events` yazmıyor (VB5).
- Benim bütün senaryolarımda md dosyaları bayt bayt aynen kaldı; değiştirenler yalnız benim yazımlarım.
- İstemciler konumla yazmıyor: hizalama ve süsleme yalnız görüntü; terfi ve indirme `promote`/`demote` uçlarından ve politikadan geçiyor.
- **"Her bloğun kim tarafından yazıldığı bilinir"** ve **"dosyalar sıradan markdown olarak kalır"** vaatlerine dokunulmuyor. VS Code kaydı tasdiksiz dış değişikliktir; bu bir vaat kaybı değil, belgeli bir kanal sınırıdır.

### P2-B kriterleri (bağımsız)
| Kod | Sonuç | Dayanak |
|---|---|---|
| B-1 | **geçti** | NFD katlama/ret kendi testim; CHANGELOG; FAST `p2b_checks` B-1 |
| B-2 | **geçti** | `loc` blame/read_block/search, `loc_stale` (VB4); FAST uyumluluk 39/39; B-2 testleri |
| B-3 | **kaldı** | 0.00551 (kendi betiğim, `7014b6f`) > 0.005 |
| B-4 | **geçti** | en kötü `bom` 0.00435 ≤ 0.02 |
| B-5 | **geçti** | VB5 |
| B-6 | **geçti** | VB6 |
| B-7 | **geçti** | FAST: Obsidian vitest 164/164, e2e 25/25 konumla; VS Code tsc/eslint/vitest 12/12/build/e2e |
| B-8 | **geçti** | 0 (151 çağrı, 12 koşu); denetleyici kendi 13 ihlalimi yakaladı |
| B-9 | **geçti** | FAST A↔B bayt eşit; P2-B tam çifti 202 738 alan fark 0 |
| B-10 | **geçti** | FAST 1032 s; P2-B'nin tam çifti `cad7663`'te temiz; denetleyiciler temiz |

---

## P2-R

### VR1′ (a) — FAST çifti
`8cf1c74`'te `make verify FAST=1`, bir kez: **çıkış 0**, RUN `20261027T172806Z-8cf1c74` (A, B), `compare_p2r fast` **18 550 alan eşit**, **1032 s** (≤ 1200).
- Kendi karşılaştırıcımla 26 ortak dosya, 16 911 yaprak: tek fark `fast/conformance.json:fixtures_root` (klon dizini; karşılaştırıcının `environment_fields`'ı).
- `p2r/det.json`, `p2r/volume.json`, `p2r_checks.json` ve bütün `p2b/*` A↔B bayt eşit.
- `p2r/volume.json` R3'ün (`reports/rounds/P2-R/r3/volume.json`) ve son FAST'ın (`fast_final/p2r/volume.json`) dosyasıyla bayt eşit.
- `p2b/align.json` R0'ınkiyle (`r0/align.json`) bayt eşit.
- **Fark 0; P2-B ve P2-R yüzeyleri fark 0.**

### VR1′ (b) — 3. denemenin kayıtları (`runs/20261026T222501Z-5b235b0/` ↔ `chain_B/`)
Kendi betiğimle (yaprak yaprak, dışlama yok) 58 ortak dosya.
- **`p2r_checks.json`, `p2b_checks.json`, `p2l_checks.json`, `p2p_checks.json` bayt eşit.**
- Chain B'nin `p2r/*` ve `p2b/*` alt dosyaları arşivde yok (yalnız üst düzey kayıtlar). Onlar için tek kanıt dondurulmuş karşılaştırıcının kaydı: `pair_compare.json` `p2r/volume.json`, `p2r/det.json`, `p2b/*`'yı karşılaştırdı, 2712 farkın hiçbiri bu dosyalarda değil.
- **`eval.historical_p2r` satırları iki zincirde aynı değil:**
  - A sayılan FAIL 5: D1-12, D-10, H-P2-0.2a, H-P2-0.3a, `logs_vs_p2a`.
  - B sayılan FAIL 9: bunlar + H-P2-0.2d, 0.2f, 0.3d, 0.3f.
  - Fazlalık dört satır P2-0.2/0.3 izleme kümelerinin. D1-12 D-10'un `watch` gecikmesini taşır.
  - Sayılan satırların hepsi zamanlamaya duyarlı kümelerde; P2-B ya da P2-R'nin bir kontrolü sayılan FAIL değil.
- **K-P2B.4 girdileri:** `historical_fails_p2r.json`'da K-P2B.4 altında yalnız D1-12'nin 10 Inspector/D-1 yolu kaldı (K-P2R.3'ün dediği gibi).
- `migration_required_fails_p2r.json`'da K-P2B.4 adı yalnız açıklama metninde geçiyor.
- **Kaydediciler 11 araçla:** D1-13 ve E1-9 iki zincirde de geçiyor. D-2 ve E1-8 P2-0.2'den beri tarihsel; kayıtları referansla aynı.
- Sınıflandırıcı kodu (`historical*.py`) P2-R'de değişmedi; yalnız yeni `historical_p2r.py` eklendi.

### VR1′ (c) — Teşhisin teyidi
`gh run download 40265181721` (`ae7e692`, `schedule`, `failure`). Yapıtta iki zincirin üst düzey kayıtları ve `verify.log` var.
- **Dondurulmuş karşılaştırıcının farkları** (`verify.log`):
  - Biri dışında hepsi `p2_0_2/d_a.json` (satır 0 ve 96: `new_blocks`, `pass_0a`, `unverified`) ve onu özetleyen `p2_0_2_checks.json`/`p2a_checks.json` alanları.
  - Biri `p1e1/release/release_check.json:items›17›detail`: A'da `…/verify/A/repo/target/p1e1-dist/bin`, B'de `…/verify/B/…`, yani klon yolu (R17).
- **Kendi karşılaştırıcım** (15 ortak dosya, 679 851 yaprak, 49 fark):
  - Yukarıdakiler.
  - Bildirilmiş zaman alanları: D1-6 süreleri, D-10 gecikmesi, H-P2-0.2f/0.3f p95.
  - `logs_vs_p2a` D-A günlüğünün blake3'ü.
  - **Başka fark yok.** `p2b_checks`, `p2l_checks`, `p2p_checks` ve P1 kayıtları eşit.
- **3. denemenin 2712 alanı** (kendi karşılaştırıcımda 5269 yaprak; ek yapraklar gecikme dizileri, RUN kimlikleri, `fixtures_root`):
  - Ayrımı: D-A, D-A2, D-C, H-P2-0.2a/a′/d/f, H-P2-0.3a/a′/b/b′/c/d/f, bunların `regress`/`fail_summary` özetleri ve zaman alanları.
  - `H-P2-0.3b/b′/c` gerçek Obsidian'ın D-B oranları; FINAL'in "P2-0.2 izleme kümeleri" ayrımında adı geçmiyor ama aynı zamanlamaya duyarlı sınıftan.
  - **Ürün yüzeyinde fark yok. Teşhis tutuyor.**

### VR2 — Hiçbir şey silinmedi (kendi vault'um, CLI)
- **Senaryo:** 5 belge; 3 ajan önerisi (`agents/` ×2, `other/` ×1); 6 tur (düzenleme + `index watch`, iki `search` — biri `include_proposal` —, bir `read`, `checkpoint`; 2. turda adlı checkpoint); `agents/p2.md`'nin paragrafı `promote`; tarif `keep_checkpoints = 2`, `[[prune.rules]] stale proposal idle_checkpoints = 2 path_prefix ["agents"]`.
- **`prune --dry-run`:** `.rootmark` bayt bayt aynen. Plan uygulamanınkiyle aynı; tek fark `events_hash`, beklenen.
- **`prune` sonrası:**
  - `events.jsonl`'in eski baytları yeni dosyanın öneki. Eklenen 3 satırın hepsi `Demoted`, `meta.prune: true`, `from = to = proposal`, `actor policy:stale`, `reason rule:stale`, `policy prune_rules`, `policy_version`.
  - `objects/`, `snapshots/` aynen. `attestations.jsonl` bu vault'ta yok, madde boş geçer.
  - **Arşiv (`0-11.jsonl`) + canlı `traces.jsonl` = eski dosya bayt bayt.**
  - Arşivlenen 4 checkpoint dosyası eskileriyle bayt eşit.

### VR3 — Kimlikler ve "checkpoint'ten okunur"
- **`agents/p1.md` (budandı):**
  - Aramada yok (`include_proposal` dahil).
  - `read` `inactive: true` ile okuyor.
  - Budamadan önceki checkpoint'te `read` ve `search --include-proposal` öncekiyle aynı.
  - `history`'nin son olayı `Demoted` + `meta.prune`.
- `other/p3.md` (önek dışı) budanmadı; terfi eden paragraf budanmadı.
- Kalan 8 izin `trace_id` ve `at_seq`'i eski dosyadakiyle aynı. Budamadan sonraki 4 iz 20, 21, 22, 23 (eski sayımın devamı, K18-T).
- `trace list` `archived_before: 12`; `--archived` 12 iz.
- Adlı checkpoint arşivlenmedi.
- Arşivli checkpoint'e `--at` ile `search` çıkış 1 veriyor ve arşiv dosyasını adlandırıyor.

### VR4 — Determinizm ve duvar saati
- Budamadan önce üç kopya: A ve B aynı `--now-ms` ile budandı. **Çıktılar ve bütün `.rootmark` dosyaları bayt eşit**, asıl vault'un budama sonrası durumuyla da aynı.
- C farklı bir `--now-ms` ile budandı: `journal/` ve yanıtın `journal` alanı dışında her şey aynı. İz, checkpoint ve konum kararı saate bakmıyor.

### VR5 — Yetki ve açık çağrı
- **Uzun oturum:** `serve --watch --http` 60 arama, bir dosya değişikliği, bir `checkpoint`. Arşiv yok; değişen dosyalar yalnız eklemeli (`events.jsonl` öneki aynen, yeni nesne/snapshot/konum kaydı, `traces.jsonl`); hiçbir dosya silinmedi ya da taşınmadı.
- **MCP:** `tools/list`'te 11 araç var, `prune` yok; `tools/call prune` → JSON-RPC `-32602` "unknown tool". CLIENT_CONTRACT §3'ün kuralı budur; §2.10'un "(`not_found`)" ifadesi HTTP'ninkidir.
- **HTTP yetkisi:**
  - `human_id`'siz `POST /v1/prune` **403 `denied`**, dosyalar aynen.
  - `dry_run: true` 200, dosyalar aynen.
  - `X-Consumer`'sız 400.
- `layer = "canonical"` kural `policy lint`'te ve `prune --dry-run`'da ret.
- **Kurcalama** (budanmış vault'un kopyaları: çöp `traces/base`, bir fazla `base`, arşiv dosyası silinmiş, canlı checkpoint silinmiş, arşivli checkpoint silinmiş): `trace list`, `checkpoint list`, `search`, `prune --dry-run` hepsinde çıkış 1 ve dosyayı ya da checkpoint'i adlandıran hata. **Sessiz yol yok.**

### VR6 — Hacim ve R-3
- **D-VOL'u kendi FAST koşum yeniden üretti:** `V_before` 14 756 069, `V_after` 2 115 988, `V_after2` 2 184 310; `volume.json` R3'ünkiyle bayt eşit; 10 terfi eden soy hiç budanmadı.
- `p2r_volume.py` yalnız ön-kayıt commit'inde (`72b25a1`, branş adı `88c50eb`) var; sonra değişmedi.
- **R-7 GOAL'den:** (a) 0.1434 ≤ 0.35, (b) 1.0323 ≤ 1.10 → geçti.
- R0 (a) uygulandı:
  - D-NOTES'u değişmemiş `p2b_align.py` FAST'ta yeniden ölçtü (0.00000, R0'ınkiyle bayt eşit); kendi betiğim de 0.
  - `git log` ölçücüde ve korpusta P2-B'nin ön-kaydından sonra değişiklik göstermiyor.
  - R-3 geçti.

### VR7 — Süreç ve vaat
- **Süreç:**
  - Denetleyiciler temiz (VB8).
  - `ROUND_START` `ae7e692`, `main`'de.
  - GOAL değişmedi.
  - Spec'te `6a5e911..8cf1c74` tek satır: §8'e bilgilendirici not. §1–6, uyumluluk fixture'ları (`spec/fixtures*`) ve NORTH_STAR aynen.
  - Dondurma istisnaları F1–F8'in her biri önce/sonra kanıtlı.
- **R5-1 (FAST kapsamının Blok 2'ye indirilmesi):** K-P2.4 (3)'ün "yalnız aktif turun kontrolleri" okumasının içinde. Sonuç görüldükten sonra alınmış bir kapsam kararı; eşik ve senaryo değişmedi. Blok 1 kontrolleri 3. denemede A↔B eşit. Kabul edilebilir.
- **Karar kuralının türetimi:**
  - Ölçüldüğü gibi R-1 (b) lafzen, R-6 lafzen ve R-10 kaldı → "tur kaldı; karar talebi". FINAL ile **aynı**.
  - Sonraki kararlar K-P2R-T1 (a) ve K-P2R.5'tir. Bunlarla R-1 ve R-6 karşılanmış sayılır, R-10 kaldı kalır. Bu dal bağımsız doğrulamayı dar kapsamla başlatır; bu doğrulama odur.
- **K-P2R.5'in R-6 okuması (bağımsız değerlendirme):** **yerinde.**
  - Kriterin konusu budama determinizmidir.
  - Budama kayıtları FAST'ta (bu doğrulamanınki ve `866dad9`), 3. denemede (`p2r_checks.json` bayt eşit, dondurulmuş karşılaştırıcıda `p2r/*` farksız) ve kendi VR4 deneyimde bayt eşit.
  - Tam çiftin farklarının hiçbiri P2-R yüzeyinde değil.

**Vaat paragrafı.**
- **"Her bloğun kim tarafından yazıldığı bilinir"** budamayla **koşulsuz korunur.** Yazarlık `events.jsonl`, `objects/`, `snapshots/`'tadır ve onlar hiç budanmaz. Proposal budaması yazarlığı silmez, üstüne kim tarafından (`policy:<id>`, `rule:<id>`, `policy_version`) yapıldığı yazılan bir olay ekler (`history` gösterir). Arşivlenen izler okuma kaydıdır, yazarlık değil, ve bayt bayt arşivde durur.
- **"Checkpoint'te aynı yanıt"** **canlı checkpoint'ler için korunur.** Budamadan önceki canlı checkpoint'te `read` ve `search` öncekiyle aynı.
- **Arşivlenmiş checkpoint'e sorgu reddedilir** (`archived`, arşiv dosyası adlı). Bu **açık bir sınırdır**, sessiz bir sapma değil: yanıt farklı verilmiyor, hiç verilmiyor; veri arşivde bayt bayt duruyor ve insan geri taşıyabilir. Vaat "checkpoint'te aynı yanıt ya da açık ret" biçimini alıyor. Bu CLIENT_CONTRACT §2.10'da yazılı; NORTH_STAR düzeyinde de bir cümleyi hak ediyor (öneri değil, not).
- **Bulgu F1 (aşağıda) vaatlerin dışında kalan bir erişilebilirlik kaybıdır.** Checkpoint vaadi tutar, ama canlı bir `canonical` blok aramayla bulunamaz hâle gelebilir.

**Damga görüşü.**
- §7 ve §8 "Validated (P1-B)" **kalır**:
  - Normatif metin değişmedi.
  - `Demoted` + `meta.prune` spec'in mevcut `Demoted` olayının başvuru uygulamasındaki kullanımıdır (okuma kuralı ASSUMPTIONS P1-4 emsali, spec metni değil).
  - §8'in bilgilendirici notu uygulamayı doğru betimliyor: `at_seq` mutlak, arşiv bayt bayt ve sıralı, rootmark silmiyor, arşivli checkpoint'e sorgu reddediliyor. VR2–VR5'te hepsi gözlendi.
- **Not olarak kalabilir** (bilgilendirici olduğu için damga istemez).
- Hüküm koşullu olduğundan (DEV_PROCESS §5), bu nota ya da P2-R'nin herhangi bir yüzeyine yeni bir "Validated" damgası **şimdi konmamalı**: koşullar (R-10 → P2-V V-4; F1'in kararı) kapanıp bağımsız teyit edilince konabilir.
- `loc` (P2-B) spec'te değil, CLIENT_CONTRACT'ta "deneysel"; R-3 0.00000 ile deneysel işaretinin kaldırılması bir karar konusudur, damga konusu değil.

### Bulgular
- **F1 (orta) — Karışık parça: budama canlı bir `canonical` bloğu aramadan tümüyle düşürüyor.** FINAL not 3'ün açık bıraktığı soru ("parçalama karışık — budanmış + canlı — parça üretiyor mu") için **evet**. Doğal bir senaryoda oldu:
  1. Ajan `agents/p2.md`'yi önerir (başlık + paragraf, tek yapısal parça: `chunk` çıktısında iki soy).
  2. İnsan yalnız paragrafı terfi ettirir.
  3. Kural başlığı budar.

  Budamadan önce o parça `include_proposal` aramasında görünüyordu; canonical aramada görünmüyordu, çünkü parça proposal etiketli (önceden var olan davranış). Budamadan sonra **hiçbir arama kipinde yok**. `blame`/`read` paragrafı `canonical`, `inactive: false` gösteriyor. Checkpoint vaadi tutar: budamadan önceki checkpoint'te yanıt aynen. FINAL'in "varsayılan (`canonical`) sonuçlar değişmez" cümlesi bu örnekte lafzen doğru, ama bunun nedeni paragrafın canonical aramada zaten görünmemesi. Canlı, terfi etmiş içerik aramayla erişilemez kalıyor ve bunu söyleyen bir sözleşme satırı yok.
- **F2 (düşük) — 3. denemenin arşivi B'nin `p2r/*`, `p2b/*` dosyalarını içermiyor.** VR1′ (b)'nin "kendi betiğinle yeniden karşılaştır" maddesi bu dosyalar için yalnız dondurulmuş karşılaştırıcının kaydına dayanabildi. `p2r_checks.json`/`p2b_checks.json` ve FAST'ın tam dosyaları bağımsız karşılaştırıldı.
- **F3 (düşük) — Kayıtlarda host yolları.** R17'nin `detail`'i (CI'yı kıran) dışında `conformance*.json:fixtures_root` ve `models_verify.json:root` da klon yolunu taşıyor. Karşılaştırıcı bunları "ortam alanı" sayıyor, R17'yi saymıyor. Tutarsızlık P2-V'nin konusu.
- **F4 (düşük, belge) — İki not:**
  - §2.10 MCP'deki `prune` çağrısı için "`not_found`" diyor, gerçekleşen §3'ün `-32602`'si.
  - Sunucu çalışırken CLI `prune`'u yalnız belgeyle yasak (gözden geçirici koşulu 4'ün kod kısmı), kodla değil.

### P2-R kriterleri (bağımsız)
| Kod | Sonuç | Dayanak |
|---|---|---|
| R-1 | **geçti** (K-P2R-T1 (a) okumasıyla; lafzen (b) kaldı) | (a) FAST `p2_r_r0` + uyumluluk 39/39; (b) kaydediciler 11 araç, K-P2B.4 altında yalnız D1-12'nin 10 araç kaynaklı yolu, sınıflandırıcı kodu aynen; (c) `p2_r_r0` testleri FAST'ta |
| R-2 | **geçti** | VR2/VR5 (olay biçimi, önek, `canonical` ret) + FAST `p2_r_r1` |
| R-3 | **geçti** | 0.00000 (kendi betiğim ve FAST) |
| R-4 | **geçti** | VR3 (F1 kriterin dışında bir yan etki) |
| R-5 | **geçti** | VR2/VR3/VR5 |
| R-6 | **geçti** (K-P2R.5 okumasıyla; lafzen tam çift temiz değil) | FAST A↔B, 3. deneme `p2r_checks`, VR4 |
| R-7 | **geçti** | 0.1434, 1.0323 (yeniden üretildi, bayt eşit) |
| R-8 | **geçti** | VR5 |
| R-9 | **geçti** | §2.10, `status.retention` (gözlendi), sözleşme dışı istek 0, denetleyici yeni ucu tanıyor |
| R-10 | **kaldı** | FAST 1032 s ≤ 20 dk ve denetleyiciler temiz, ama tam `make verify` temiz değil → **kapanışı P2-V V-4** |

## Hüküm

**Şu koşullarla güvenilir.**

Blok 2'nin ürün yüzeyleri — konum (`loc`, `loc_stale`), `events`, zaman filtresi, istemci sözleşmesi v2 ve denetleyicisi, proposal sönümleme, iz/checkpoint arşivi, `prune` — iddia edildiği gibi davranıyor:
- B-3'ün kalışı ve R-3'ün geçişi kendi betiğimle birebir yeniden üretildi.
- Budama hiçbir yönetişim kaydını silmiyor, determinist ve saatten bağımsız; yetki sınırları ve kurcalama yolları sesli.
- K-P2R.5'in teşhisi (CI ve 3. deneme farkları yalnız zamanlamaya duyarlı kümelerde ve R17'nin host yolunda) tutuyor.

Koşullar:
1. **R-10:** tam çiftin temiz koşusu P2-V V-4'te; o kapanmadan Blok 2'nin tam çifti için "temiz" yazılmaz.
2. **F1:** canlı bir `canonical` bloğun budamayla aramadan düşmesi düzeltilir ya da CLIENT_CONTRACT §2.10'da bilinen sınır olarak kararla yazılır.

Blok 2 bu iki koşulla kapanabilir; P2-C'nin açılışını F1 bloke etmez.

## Öneriler (≤ 3)
1. **F1 için karar ve test.** Dizin görünümü `inactive` bloğu parçanın tamamını değil yalnız o bloğu dışarıda bırakmalı (ya da parça sınırı soy katmanında bölünmeli). En azından "budanmış blokla parça paylaşan canlı `canonical` blok aramada bulunur" testi eklenmeli ya da sınır §2.10'a kararla yazılmalı. Aynı karar, terfi etmiş bir bloğun proposal bloğuyla aynı parçada canonical aramada görünmemesini (önceden var olan davranış) de ele almalı.
2. **P2-V'ye iki ek.**
   - Kayıtlardaki host yolları tek kuralla ele alınmalı: R17 `detail`, `fixtures_root`, `models_verify.root`; ya kayıttan çıkar ya hepsi ortam alanı.
   - Tam çiftin arşivi turun kendi yüzeylerinin (`p2r/*`, `p2b/*` ve sonrakiler) **iki zincirdeki** dosyalarını içermeli; doğrulayıcı dondurulmuş karşılaştırıcıya dayanmadan karşılaştırabilsin.
3. **Sunucu çalışırken CLI `prune`'unun reddi kodlanmalı.** Ör. `serve`'ün tuttuğu bir kilit ya da `server.token`/port dosyasıyla canlılık denetimi. Belge kuralı (§2.10) bir iz satırının sessizce kaybolabileceği tek yolu kapatmıyor. Aynı turda §2.10'un MCP ifadesi §3'le hizalanmalı.
