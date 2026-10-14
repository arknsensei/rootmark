# P1-B — Bağımsız Doğrulama Raporu

- **Doğrulanan commit:** `2df32a7563b850d446f3c493f503f67a357a9783` (READY.md commit'i; klon HEAD'i bununla eşit, çalışma ağacı temiz)
- **Ölçülen kod commit'i (resmi RUN):** `7735a07` · resmi RUN `20261014T135400Z-7735a07`. `7735a07..2df32a7` arasında kod dışında değişen tek yol `spec/SPEC.md` (E4 durum damgası); geri kalan her şey `docs/` ve `reports/` altında.
- **Tarih:** 2026-10-14 (UTC ~14:20–14:45)
- **Ortam:** macOS 27.0.1 arm64 · rustc 1.92.0 (ded5c06cf 2025-12-08) · cargo 1.92.0 · Python 3.13.5 (sistem), `research/.venv` (uv 0.11.8) · node v24.3.0
- **Yöntem:** Ana çalışma dizininden yalnız `git clone` yapıldı. Temiz klon `…/scratchpad/clone`; zincirler `A/`, `B/` altında. Kasıtlı testler ayrı kopyalarda yapıldı: `v5/` (yönetişim), `v7/` (dondurma), `v8/` (taşınmış `target/`), `v2gen/` (fixture'ları yeniden üretme), `v3/` (v3-final ikilisi). Doğrulama sonunda `clone`, `A/repo` ve `B/repo` temiz (`git status --porcelain` 0 satır).
- **Kendi araçlarım** (depoya girmedi): `cmpjson.py` (JSON yapraklarını tam eşitlikle karşılaştırır; yalnız gecikme yapraklarını hariç tutar: `ms`, `*_ms`, `latencies_ms`, `p50`, `p95`), `ids.py` (`checkpoint_id` ve `trace_id` değerlerini spec formülünden blake3 ile yeniden hesaplar), `v9.py` (kesişimler, gerçekleşen oranlar, uygunluk), `v5g.sh` (21 politika/CLI vakası), `zz_verifier.rs` (8 Rust testi, yalnız `v5/` kopyasında), `vcase.rs`/`vmix.rs` (v3-final API'siyle vault üreten örnek programlar, yalnız `v3/` arşiv kopyasında).

Etiketler: `[OK] [FARK] [ŞÜPHE] [İHLAL]`

---

## V1 — Çift koşturma `[OK]`
**Komut:** `bash clone/scripts/verify.sh pair <scratch>/clone <scratch>/A <scratch>/B`
- Çıkış kodu **0**. A zinciri RUN `20261014T142440Z-2df32a7`, B zinciri RUN `20261014T143408Z-2df32a7` (B: 578 s).
- `pair_compare.json` (compare_p1b): `max_relative_difference = 0.0`, `n_fields = 2195`, 14 dosya, `B-4_equal = true`, `bytes_equal = true`.
- Kendi karşılaştırmam A↔B: 17 dosyada (v3 9 dosya, holdout, split_grid, holdout_split, fuzz×2, v3_compat, policy_bounds, governance.json) **1989 kalite yaprağı, 0 fark**; 409 gecikme yaprağı hariç tutuldu.
- `governance/vault/.mdkernel/traces.jsonl` ve `events.jsonl` A↔B için `cmp` ile eşit. `checkpoints/` (4 dosya) `diff -r` ile eşit. Ayrıca `.mdkernel/` dizininin tamamı `diff -r` ile eşit.
- A↔resmi RUN: `verify.sh compare-p1b` çıkış 0 (2195 alan, fark 0.0, bayt eşit). Kendi karşılaştırmam aynı 17 dosyada 1989 yaprak, **0 fark**. traces/events/checkpoints resmi RUN ile de bayt eşit.
- Her iki RUN'da 14 fazın hepsinde `worktree_clean = true`. Her iki zincirde `test_tamper.sh` 12/12 bozmayı yakaladı.

## V2 — Uyumluluk `[OK]`
- A zinciri ve `v2gen/` kopyası: `reports/conformance.json` → `level_claimed 2`, **25/25** (seviye 1: 17 geçti, seviye 2: 8 geçti), kimlikler C1-01..C1-17 ve C2-01..C2-08 (`spec/CONFORMANCE.md` tablosuyla birebir).
- `fixtures_root` mutlak ve kanonik (`…/v2gen/spec/fixtures`), `fixtures_root_source = "MDK_FIXTURES"`, `fixtures_manifest_hash = 7c7148b55500342862878b815e4dcbfb`, `fixtures_manifest = "ok"`.
- **Yeniden üretim:** `v2gen/` kopyasında 17 C1 ve 8 C2 `expected.json` dosyasıyla `MANIFEST.json` silindi. Ardından `bash scripts/gen_fixtures_c2.sh` çalıştırıldı: C2 değerleri mevcut çekirdekten, C1 değerleri ve manifest dondurulmuş v3-final üretecinden (`71c244e5`) geldi. Sonuç: `git status --porcelain --untracked-files=all` **0 satır**, yani 26 dosya bayt bayt aynı üretildi.
- **Formüller bağımsız hesaplandı (`ids.py`):** yönetişim senaryosundaki 4 checkpoint'in `checkpoint_id` değeri spec §3 formülüyle (`H(sorted "doc_path\tsnapshot_id" satırları "\n" ile + "\n" + event_seq)`) ve dosya adıyla eşit. 4 iz satırında `trace_id = H(checkpoint_id + consumer + query_hash + at_seq)` eşit, `at_seq` = satır sırası, `returned` girdileri `{doc_path, lineage_id, version_id}` nesnesi. Her `CheckpointCreated` için `event_seq` = son checkpoint olmayan olayın `seq + 1` değeri (K18-T). Sonuç: 12/12.
- Sınır: C2 beklenen değerleri mevcut çekirdekten üretiliyor. Yeniden üretim kendi içinde tutarlılığı gösterir, spec yorumunun doğruluğunu göstermez. Formülle bağımsız denetlenebilenler yalnız kimlik formülleri; onlar tuttu.

## V3 — Geriye uyumluluk (B-5) `[OK]` öngörülen yöntemle · `[FARK]` genel iddia için (bkz. **F1**)
**Komutlar:** `git archive v3-final Cargo.lock rust-toolchain.toml .cargo core` (sha `71c244e5…`, yalnız `core` üyeli çalışma alanı) → `cargo build --offline --release --bin mdk`. Mevcut `mdk`, `v5/` kopyasında HEAD'den derlendi.
- **mini_vault (VERIFY'da öngörülen):** v3-final `mdk ingest-dir VA spec/fixtures/mini_vault` + `core/tests/fixtures/v3_final/edit` → 118 olay, hiçbirinde `layer` yok. Mevcut `mdk` ile 12/12 doküman için `replay` `snapshot_id` eşit. `events_hash` eski = yeni = `f3cb01242e8cb7123e417a72a196250f` (FINAL ile aynı). `events.jsonl` sha256 önce ve sonra aynı. `blame` 12 dokümanda çalışıyor: 98 blok, hepsi `author_kind Human` → `layer canonical` (türetilmiş; olaylarda `layer` yok). `history` ve `checkpoint list` de çalışıyor.
- **Genişletilmiş (kendi senaryolarım, v3-final kütüphane API'siyle `Vault::ingest(…, Provenance{author_kind}, V3)`):**

  | Senaryo (v3-final ile yazıldı) | Mevcut çekirdekle açılış | Türetilmiş katman |
  |---|---|---|
  | yalnız Agent / yalnız Transcript | açılır, replay eşit, events_hash eşit | proposal ✓ |
  | yalnız Synthetic | açılır | canonical (ASSUMPTIONS B1-2) |
  | Human yazdı, Synthetic düzenledi | açılır | canonical |
  | Agent yazdı, Human düzenledi | açılır | proposal (K17-T: katman yalnız düşer) |
  | **Human yazdı, Agent düzenledi** | **`Err`: `event seq 4: snapshot committed before LayerChanged of 1581f7d3…`** | — |
  | **Human yazdı, Transcript düzenledi** | **aynı `Err`** | — |
  | **Synthetic yazdı, Agent düzenledi** | **aynı `Err`** | — |
  | karışık 4 dokümanlı vault (son adım: Agent bir Human dokümanını düzenler) | **tüm vault açılmıyor**; `replay`, `blame`, `events-hash` 4/4 dokümanda hata | — |

  Hata sessiz değil ve dosya baytları değişmedi (açılış salt-okur). Ama bu vault'lar mevcut çekirdekle hiç açılamıyor. → **F1**.

## V4 — Regresyon (B-3) `[OK]`
**Komut:** zincir içinde `make regress RUN=…` (çıkış 0). `regress_vs_v3final.json`: 1577 alan, fark 0.0, `H4-2_equal true`. `regress_vs_p1a.json`: 1790 alan, fark 0.0, `A-4_equal true`.
- Kendi karşılaştırmam, A RUN ↔ v3-final referansı `research/reports/runs/20261013T182004Z-f82dabd`: 9 dosyada (h1s_tuning, h1s, h1r, h1s_v3_tuning, h1s_v3, h1r_v3, h3_v3, h4_v3, h4_v2repro) **1570 kalite yaprağı, 0 fark** (P1-A doğrulamasındaki sayıyla aynı).
- A RUN ↔ P1-A resmi RUN `reports/rounds/P1-A/runs/20261013T221822Z-27d8ae8`: 11 dosyada (holdout 66 ve split_grid 145 yaprak dahil) **1781 yaprak, 0 fark**.

## V5 — Yönetişim davranışı (kasıtlı, `v5/` kopyasında) `[OK]`
- **(a) Hook'suz terfi:** `v.promote(&req)` (politikasız çağrı) **derlenmiyor**: `error[E0061]: this method takes 2 arguments but 1 argument was supplied`. `Deny` ile `Err` (`denied by policy deny v1`). CLI'da `policy.toml` yokken yerleşik tarif `deny`: `Err`, varsayılan uyarısı yok. **Atlatma denemeleri:** herkese açık `Vault::commit` elle kurulmuş `Promoted` olayını reddediyor (`written only through the policy hook`), elle kurulmuş yükseltici `LayerChanged` olayını da reddediyor (`does not match the replayed layer rule`). (Bilgi: `events.jsonl`'a elle eklenen, meta'sı tam bir `Promoted` satırı açılışta kabul ediliyor. Kimlik doğrulama yok, ASSUMPTIONS B6-4 ile kabul edilmiş sınır.)
- **(b)** `HumanOnly` + actor `agent:bot` → `Err` (`actor "bot" is not human`). Actor `policy:auto` → `Err`. Katman proposal'da kaldı, günlükte `Promoted` yok. CLI ile de aynı sonuç.
- **(c)** API'de `reason` `""` veya `"   "` → `Err` (`a layer transition requires a reason`). Geçerli terfiden sonra günlükteki `Promoted` olayından `meta.reason`, `meta.actor` ya da `meta.policy_version` silinince `Vault::open` → `Err` (`Promoted requires meta.reason` vb.). CLI'da `--reason ""` → `Err`, `--reason` verilmezse clap hatası.
- **(d)** `prune_inactive` → `Demoted` + `meta.prune: true`, katman değişmiyor (`from = to = proposal`). `objects/` dosya sayısı 2 → 2. Yeniden açılışta soy hâlâ pasif.
- **(e)** Checkpoint `v1` oluşturuldu, ardından `y.md` yeniden yazıldı ve yeni bir `z.md` ingest edildi. `snapshot_at("y.md","v1")` = ilk `snapshot_id` (`cb6ae7b4…`, yeni `61747f4c…` değil). `snapshot_at("z.md","v1")` = `None`. `blocks_at` eski metni veriyor. Yeniden açılıştan sonra sonuç aynı.
- **(f)** `checkpoints/<cid>.json` içeriğine üç farklı değişiklik denendi (başka ad, başka `event_seq`, sona satır sonu): her birinde `Vault::open` → `Err` (`does not match its CheckpointCreated event`). Dosya silinince de `Err`. Henüz oluşturulmamış bir checkpoint'in yoluna önceden sahte dosya yazılınca `checkpoint()` → `Err` (`object conflict … exists with different content`) ve sahte dosya değişmedi. Aynı durumu farklı adla yeniden adlandırma → `Err`.
- **(g)** Kendi 21 vakam (`v5g.sh`, B-6 betiğinden ayrı değerler). retention 0 / 23 / −5; idle_ms −1000 / −1; min_sim_change −0.01 / 1.0001 / `nan` / `inf`; promotion `"auto"` / `"Human_Only"` / `""`; bilinmeyen tablo; yazım hatalı anahtar `retention_hour`; eksik `promotion` anahtarı; `policy.toml` yok (hook'suz, deny); actor `human:` (boş id) ya da hiç verilmemiş; reason boş ya da hiç verilmemiş; `agent:bot`. Sonuç: **21/21 çıkış ≠ 0**, stdout boş, stdout ve stderr'de **"default" kelimesi 0 kez** geçiyor. Vakalardan sonra günlükte `Promoted` sayısı 0. Geçerli tarif terfi ediyor. Zincirdeki `test_policy_bounds.sh` (`policy_bounds.json`) de A ve B'de 6/6.

## V6 — Determinizm `[OK]`
- `core/tests/layers_fuzz.rs` (B-10) kendi seçtiğim tohumlarla koşturuldu: `MDK_FUZZ_SEED=1` (1000 adım), `424242` (1000 adım), `99` (3000 adım). Hepsinde `replay_equal true` ve test geçti. Örnek: seed 1 → 68 promote_ok, 76 demote_ok, 57 prune_ok, 125 hook_refused.
- Aynı tohum (1) iki kez koşturuldu: özet JSON'ları (`events_hash` `1f6c9a5f…` dahil) bayt eşit.
- `traces.jsonl` A↔B bayt eşit (V1).
- Sınır (B1 gözden geçirmesindeki not 4 ile aynı): fuzz dizisi Human→Agent düzenlemesi sonrası katman düşüşünü kurulu bir v3-final günlüğü üzerinde denemiyor. F1 bu boşluktan çıktı.

## V7 — Dondurma `[OK]` (bir şüphe notuyla, bkz. Ş2)
- `bash scripts/check_frozen.sh` temiz klonda → çıkış 0 (`table append-only since 983c62d`, `check_frozen: OK`).
- `v7/` kopyasında kasıtlı denemeler (her birinden sonra geri alındı):
  - P1-A satırı silme, commit'siz → exit 1
  - aynı silme commit'li → exit 1
  - P1-B'de eklenen bir satırı silme, commit'li → exit 1 (`rows removed or rewritten since 983c62d`)
  - bir satırın kalıbını gevşetme, commit'li → exit 1
  - dondurulmuş bir dosyayı değiştirme (`run_holdout_split.py`) → exit 1
  - geri alındıktan sonra → exit 0
- `ROUND_START` = `983c62d` = "P1-B: round directives (GOAL, TASKS, VERIFY, PROMPT)" commit'i (yalnız bu 4 dosyayı ekliyor). GOAL/TASKS/VERIFY/PROMPT, NORTH_STAR ve `spec/CONFORMANCE.md` bu commit'ten sonra değişmemiş.
- Tur içindeki dondurulmuş dosya değişiklikleri F1–F8 (`FREEZE_EXCEPTIONS.md`) ile kayıtlı. Her biri DEV_PROCESS §4'ün üç öğesini taşıyor (girdi/diff, önce/sonra bit eşitliği, gözden geçirici onayı) ve FROZEN.md'ye eklenen bir yeniden dondurma satırıyla eşleşiyor. `research/eval/**` altında P1-A dondurmasından beri değişen dosya yok (check_frozen OK).
- **Spec:** `git diff 983c62d HEAD -- spec/SPEC.md` tam olarak E1–E4'ten oluşuyor (SPEC_EDITS tablosuyla birebir, başka satır yok). §1–6 metni değişmemiş; 3. satırdaki durum cümlesi §1–6 için aynı kalmış. Değerlendirmem:
  - E1: katman değerlerinin yazımı, spec'te yazıldığı biçimde.
  - E2: `returned` nesne biçimi; `+` düz birleştirme; `at_seq` ondalık. Formülün harfi harfine okunması ve §3'ün diğer formülleriyle tutarlı.
  - E3: bilgilendirici not.
  - E4: karar kuralına bağlı durum damgası.

  Hesaplanan değerleri değiştiren bir anlam değişikliği görmedim. Bu yüzden `[İHLAL]` yok. E1 ve E2 serileştirmeyi sabitliyor; bu konudaki not için bkz. Ş2.

## V8 — Ö2 (B-7) `[OK]`
- `v2gen/` (derlenmiş `target/` ile) `cp -Rp` ile `v8/` dizinine kopyalandı. `make conformance` hiçbir şeyi yeniden derlemeden (`Compiling` 0) taşınmış ikiliyi kullandı.
  - Fixture'da yalnız boşluk değişikliği (`C2-05/expected.json`) → çıkış 2, `fixtures_manifest = "mismatch"`, `level_claimed 0`.
  - Anlamsal bozma (`C2-07/agent.md` içeriği değişti, manifest satırı yeni hash'e güncellendi) → çıkış 2, C2-07 `fail`, `level_claimed 0`.
  - Her iki durumda `fixtures_root = …/scratchpad/v8/spec/fixtures` (kopyanın yolu, kaynak ağacın değil).
- Taşınmış test ikilisi `MDK_FIXTURES` olmadan `v8/core` içinden çalıştırıldı → `fixtures_root_source = "cwd"`, kök kopyanın yolu. İlgisiz bir dizinden çalıştırılınca → hata. Derlenmiş yola bağımlılık kalmamış.
- Zincirdeki `test_tamper.sh` B-7 vakası A ve B'de yakalandı.

## V9 — Holdout-split (B-9) `[OK]` (K20-T çekincesiyle)
- Kendi hesabım (`v9.py`, A'nın yeniden üretilmiş verisi):
  - Kaynak kümeleri: easy (d1) 200, hard (d1h) 200, d1ho 100, d1hs 60. Altı ikili kesişimin hepsi, kaynak yolu ve `v00.md` sha256'sı bazında **0**.
  - d1hs tabanlarından hiçbiri D3'teki 1012 md metninden biriyle aynı değil; boşluk normalleştirilmiş karşılaştırmada da 0.
- Gerçekleşen oranlar (600 puanlanan adım, truth dosyalarından): split_edit **0.295** ≥ 0.25 · split_three **0.205** ≥ 0.10. Diğer oplar FINAL ile birebir.
- Uygunluk: 60/60 tabanda en az bir ≥ 3 cümleli paragraf var (kaba cümle bölücüyle). Doküman sayısı 60 ≥ 60.
- SEED `20261013:holdout-split` (üreteç ve `MANIFEST.d1hs.json`). Eşikler `run_holdout_split.py` içinde ve GOAL ile aynı (`>= 0.90`, `<= 0.03`, `>= 0.70`). Üreteç, manifest ve koşturucu tek commit'te (`df19545`, B8-pre) ve herhangi bir d1hs sonucundan önce girmiş.
- A, B ve resmi RUN'da aynı değerler: H1-H1 **0.992874** ≥ 0.90 · H1-H2 **0.000000** ≤ 0.03 · H1-H3 **0.997222** ≥ 0.70 → geçer. Çekince FINAL ve K20-T'de açıkça yazılı: tabanlar D2 korpusu.

## V10 — Bağımlılık yönü `[OK]`
- `bash scripts/check_deps.sh` → `OK (core ← index ← server ← clients; research → bindings/python)`. Zincirdeki `check_deps.log` dosyaları da temiz.
- `core/Cargo.toml` bağımlılıkları yalnız crates.io paketleri (anyhow, blake3, clap, pathfinding, pulldown-cmark, regex, serde, serde_json, strsim, tiktoken-rs, toml, unicode-normalization). `cargo tree -p mdkernel-core` içinde başka bir mdkernel crate'i yok. `core/` kaynağında `index`/`server` referansı yok.

## V11 — Karar kuralı `[OK]` (ölçüm düzeyinde) · F1 ile birlikte okunmalı
GOAL'den türetilen kural: B-1..B-8 ∧ B-10 geçti → §7/§8/§11 "Validated", P1-C başlar. B-9 kaldı → karar talebi, P1-C beklemez. B-5 kaldı → P1-C başlamaz. B-3/B-4 kaldı → P1-C başlamaz.

Benim ölçümlerim:

| Kriter | Sonuç |
|---|---|
| B-1 | 8/8, L2 |
| B-2 | 17/17 |
| B-3 | 0 fark |
| B-4 | 0 fark, bayt eşit |
| B-5 | mini_vault ile eşit; genel iddia için bkz. F1 |
| B-6 | 6/6 ve kendi 21 vakam |
| B-7 | ✓ |
| B-8 | ✓ |
| B-9 | ✓ |
| B-10 | ✓, kendi tohumlarımla da |

GOAL'deki ölçüm tanımıyla FINAL'in sonucu ("B-1..B-10 geçti → Validated; P1-C başlayabilir") **yeniden üretildi**. Ancak B-5'in koruduğu özellik ("v3-final ile üretilmiş vault mevcut çekirdekle açılır") v3-final'in kendi herkese açık API'siyle üretilebilen bir vault sınıfında tutmuyor (F1). Karar kuralında "B-5 kaldı → P1-C başlamaz" dalı var. Bu yüzden F1'in B-5 başarısızlığı sayılıp sayılmayacağı karar sahibinindir; doğrulayıcı bunu hükmetmez.

---

## Sonuç

### İhlaller
- Yok.

### Farklar
- **F1 — v3-final vault'ları için geriye uyumluluk genel olarak tutmuyor (B-5 kapsamı).** v3-final çekirdeği ve Python bağlaması (`bindings/src/lib.rs`, `ingest(doc_path, text, author_kind="Human", …)`, `"Agent"`/`"Transcript"`/`"Synthetic"` kabul ediyor) Human dışı yazarlarla ingest yapabiliyor. Human (ya da undeclared Synthetic) yazarın oluşturduğu bir soyu Agent ya da Transcript düzenlediğinde mevcut replay kuralı (ASSUMPTIONS B1-3: "kuralın gerektirdiği her `LayerChanged`'i … bekler") günlükte bulunmayan bir `LayerChanged` bekliyor. Vault'un tamamı `Err` ile açılmıyor. ASSUMPTIONS B1-3'teki "v3-final günlükleri (yalnız `Human`)" önermesi v3-final CLI'si için doğru, kütüphane ve bağlama için doğru değil.
  - En küçük örnek: v3-final API'siyle `ingest("d.md", t1, Human)`, ardından `ingest("d.md", t2, Agent)` → mevcut `mdk replay` / `blame` / `events-hash` → `event seq 4: snapshot committed before LayerChanged of …`.
  - Resmi B-5 ölçümü (`check_v3_compat.sh`) yalnız Human içerikli mini_vault kullandığı için bunu göremiyor.

### Şüpheler
- **Ş1 — K22-T'deki çakışma örneği metinde yanlış.** Formülde `query_hash` `consumer` ile `at_seq` arasında duruyor. Bu yüzden ("a1", qh, 2) ile ("a", qh, 12) aynı `query_hash` ile çakışmaz (`"a1"+qh+"2"` ≠ `"a"+qh+"12"`). Çakışma yine de mümkün: `record_trace` `query_hash` uzunluğunu doğrulamıyor, bu yüzden `consumer`/`query_hash` sınırında kayma olabiliyor ("ab"+"c…" = "a"+"bc…"). K22-T'nin talebi (ayırıcı) geçerli; yalnız gerekçe örneği düzeltilmeli.
- **Ş2 — "Yalnız açıklık" ölçütü tutarsız uygulanmış.** E3'ün normatif cümlesi "kırpan bir uygulamayı uyumsuz kılardı" gerekçesiyle geri alındı. E1 (küçük harf serileştirme) ve E2 (`returned` girdisinin nesne olarak serileştirilmesi) de daha önce uyumlu sayılabilecek bir yazımı (ör. PascalCase katman, dizi biçiminde demet) uyumsuz kılıyor. Hesaplanan kimlikler değişmediği ve C2 fixture'ları zaten bu yazımı kayıt altına aldığı için bunu ihlal saymadım. K15-T'nin "`returned` demet ↔ nesne: P1-B'de karar" maddesi ise hâlâ açık talep olarak duruyor; karar spec metnine düzeltme olarak girmiş.
- **Ş3 (bilgi) — B-5 ve B-10 kapsamı.** Ne B-5 betiği ne fuzz testi kurulu (layer'sız) bir günlük üzerinde katman düşüren düzenlemeyi deniyor. F1 bu kapsam boşluğundan çıktı. Undeclared `synthetic` → canonical davranışı (ASSUMPTIONS B1-2) spec'teki "as declared" ifadesinin bir yorumu; kayıtlı, ama spec'te normatif karşılığı yok.

### Hüküm
FINAL.md'deki ölçümler bağımsız olarak yeniden üretildi:
- çift koşturma ve resmi RUN ile 0 fark, bayt eşitlik;
- 25/25, `level_claimed 2`;
- regresyon 0 fark;
- yönetişim davranışı, politika sınırları, dondurma, Ö2, d1hs, bağımlılık yönü.

GOAL'deki ölçüm tanımıyla B-1..B-10 geçiyor; **FINAL'in karar kuralı sonucu ölçüm düzeyinde doğru.** Bir fark var (F1): B-5'in amaçladığı geriye uyumluluk, v3-final'in herkese açık API'siyle üretilebilen karışık yazarlı vault'larda kırık. Karar kuralının "B-5 kaldı → P1-C başlamaz" dalı göz önüne alınınca, P1-C başlamadan önce F1'in B-5 kapsamında sayılıp sayılmayacağına karar verilmesi gerekir.

### Öneriler (en fazla 3)
1. **F1'i P1-C'den önce kapatın.** Replay, `layer` bilgisi taşımayan (v3-final biçimli) günlüklerde kuralın gerektirdiği katman düşüşünü olay beklemeden türetmeli; ya da bu tür günlükler için açık bir yükseltme (migration) adımı olmalı ve `events.jsonl` değişmemeli. Ardından `check_v3_compat.sh`'e v3-final API'siyle üretilmiş Human→Agent, Human→Transcript ve Synthetic→Agent senaryoları eklenmeli. Betik dondurulmuş olduğu için bu, F-istisnası ile yapılmalı.
2. **K22-T'nin gerekçe örneğini düzeltin** ve karar sırasında `query_hash` biçim doğrulamasını (sabit uzunluk, hex) de değerlendirin. Ayırıcı kararı alınırsa C2-07 beklenen değeri spec sürüm artışıyla yeniden üretilmeli.
3. **SPEC_EDITS'te "yalnız açıklık" ölçütünü yazılı hâle getirin.** "Hesaplanan değeri değiştirmez ve daha önce uyumlu bir uygulamayı uyumsuz kılmaz" gibi bir tanım koyun. E1/E2'yi bu ölçütle yeniden sınıflandırın; serileştirme kararını K15-T'de kapatın.
