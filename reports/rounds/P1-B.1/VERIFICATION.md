# P1-B.1 — Bağımsız Doğrulama Raporu

- **Doğrulanan commit:** `b243206` (READY.md commit'i). Klonun HEAD'i bu commit, çalışma ağacı temiz. READY'nin gösterdiği `de77219` ve ölçülen kod commit'i `226b511` bu commit'in ataları. `226b511..b243206` arasında yalnız `docs/` ve `reports/` değişiyor.
- **Resmi RUN:** `20261014T212243Z-226b511` (`reports/rounds/P1-B.1/runs/…`).
- **Tarih:** 2026-10-14/15 (UTC ~22:35–23:15).
- **Ortam:** macOS (Darwin 27.0.0 arm64) · rustc 1.92.0 (ded5c06cf 2025-12-08) · Python 3.13.5 · git 2.49.0. Python araçlarım `blake3` için A zincirinin `research/.venv` ortamını kullandı.
- **Yöntem:** Depo `git clone` ile scratchpad'e alındı (`…/scratchpad/clone`). Deponun uzak kaynağı (remote) olmadığı için klonun kaynağı yerel deponun nesne deposu. Bütün koşturmalar ayrı kopyalarda yapıldı:
  - `A/`, `B/` — `verify.sh pair` zincirleri
  - `v3/` — `git archive v3-final` ile çıkarılmış ağaç, v3-final'i **kendim derledim**
  - `cur/` — HEAD kopyası, yalnız kendi örnek programlarım için
  - `p1bf/` — `git archive p1b-final core`, ayırt edicilik denemesi için
  - `v5/` — fixture yeniden üretimi, `make test`, `make lint`
  - `v6/` — kasıtlı dondurma bozmaları, her biri geri alındı
  
  Doğrulama sonunda `clone`, `A/repo`, `B/repo`, `v5`, `v6` temiz (`git status --porcelain --untracked-files=all` 0 satır, HEAD `b243206`).
- **Süreç notu (dürüstlük):** Klonu almadan önceki ilk komutla, ana çalışma dizinindeki commit'li `docs/DEV_PROCESS.md` §5 ve `docs/rounds/P1-B.1/GOAL.md` "Doğrulama" bölümünü okudum. Bu, DEV_PROCESS §5'teki "okuma dahil erişim yok" kuralından sapma. Okunan dosyalar HEAD'de commit'li ve değişmemiş (başlangıçtaki durumda yalnız `docs/rounds/P1-C/` izlenmiyordu). Doğrulamanın geri kalanı yalnız klondan yürüdü. Ana dizine yazılan tek dosya bu rapor.
- **Kendi araçlarım** (hiçbiri depoya girmedi):
  - `vgen.rs` — 9 senaryoyu kütüphane API'siyle üreten örnek program. Kendi metinlerimle yazıldı; `scenarios.json` ya da commit'li fixture kullanmıyor. Aynı kaynak hem v3-final'e hem HEAD çekirdeğine karşı derlendi.
  - `v2check.py` — beklenen katmanı yalnız `events.jsonl`'daki yazar dizisinden ve spec §7 + K17-T kuralından türetir.
  - `v3del.py` — `LayerChanged` silme vakaları.
  - `v4ids.py`, `v4open.py` — `trace_id` formülünü blake3 ile yeniden hesaplama ve elle bozulmuş iz satırları.
  - `vtrace.rs` — B1-4/5 vakaları, kütüphane API'siyle.
  - `cmpjson.py` — JSON yapraklarını tam eşitlikle karşılaştırır. Gecikme yapraklarını hariç tutar: `ms`, `*_ms`, `latencies_ms`, `p50`, `p95`, `seconds`, `elapsed*`.

Etiketler: `[OK] [FARK] [ŞÜPHE] [İHLAL]`

---

## V1 — Çift koşturma (B1-7) `[OK]`
**Komut:** `bash clone/scripts/verify.sh pair <scratch>/clone <scratch>/A <scratch>/B`
- Çıkış kodu **0**.
  - A zinciri: RUN `20261014T224135Z-b243206`.
  - B zinciri: RUN `20261014T225124Z-b243206`, 588 s.
- `pair_compare.json` (compare_p1b): `n_fields 2195`, `max_relative_difference 0.0`, `B-4_equal true`, `H4-2_equal true`, `bytes_equal true`.
- Kendi karşılaştırmam (`cmpjson.py`, 20 dosya, 2494 kalite yaprağı; `p1b_checks`, `conformance`, `governance.json` dahil): A↔B ve A↔resmi RUN'da yalnız 1 fark var. O da `conformance.json:fixtures_root`, yani klonun mutlak yolu; ortamdan gelir, kalite alanı değildir. Geri kalan yaprakların hepsi eşit.
- Yönetişim vault'u: `events.jsonl`, `traces.jsonl` ve `checkpoints/` (4 dosya) A↔B ve A↔resmi RUN `cmp`/`diff -r` ile bayt eşit. A↔B için `.mdkernel/` dizininin tamamı da eşit. A↔resmi RUN'da tek fark iki boş `journal/blobs/` alt dizini; git boş dizin tutmaz, dosya farkı yok.
- Her iki RUN'da 14 fazın hepsi `git_sha b243206`, `worktree_clean true`.
- `test_tamper.sh` iki zincirde de 12/12 bozmayı yakaladı.
- A klonunda `make p1b1-checks RUN=<A>` çıkış 0. `p1b1_checks.json` resmi RUN'dakiyle aynı; tek fark `run_b` yolu. Sonuç: `B1-1..B1-6 true`.

## V2 — B1-1, kendi derlediğim v3-final ile `[OK]` 9/9
- **Derleme:** `git archive v3-final` (`71c244e5…`, hafif etiket, doğrudan commit), ardından `cargo build --offline --release -p mdkernel-core --bin mdk --example vgen`.
- **Üretim:** Commit'li fixture'lar ve `scripts/gen_v3final_mixed.sh` **kullanılmadı**. `vgen` v3-final'in herkese açık `Vault::ingest(doc, text, Provenance{author_kind,…}, V3)` API'siyle GOAL'deki 9 senaryoyu kendi metinlerimle üretti:
  - 8 tek dokümanlı senaryo; her birinde ikinci ingest bir paragrafı ve bir liste öğesini düzenliyor, bir paragraf ekliyor.
  - 4 dokümanlı karışık vault: H; A→H; H→A; S→T, iç içe sırayla.
  - Üretilen 9 günlüğün hiçbirinde `layer` alanı ya da `LayerChanged` yok.
- **Mevcut çekirdekle açılış** (HEAD `mdk`), her senaryo ve her doküman için:
  - `replay` → `snapshot_id` ve `blocks` v3-final `mdk replay` ile eşit.
  - `events-hash` → v3-final ile eşit.
  - Açılıştan önce ve sonra `.mdkernel/` ağacının sha256'sı ve `events.jsonl` baytları değişmedi.
  - `blame` her soy için `[author_kind, layer, layer_source]` veriyor. Bu değerler yalnız günlükteki yazar dizisinden ve kuraldan türettiğim beklenen değerle eşit. Kural: H ve S yalnız → `canonical`; A ve T → `proposal`; katman yalnız düşer; düşüş varsa `derived`.
  - Aynı senaryoları HEAD çekirdeğiyle de ürettim (yeni biçim, düşüşler `LayerChanged` ile kayıtlı). Katmanlar soy soy aynı çıktı; `layer_source` yeni biçimde `event`.

| Senaryo | Doküman | Blok | `derived` | Sonuç |
|---|---|---|---|---|
| agent_only | 1 | 7 | 0 | ✓ |
| transcript_only | 1 | 7 | 0 | ✓ |
| synthetic_only | 1 | 7 | 0 (canonical, K-B1.4) | ✓ |
| human_synthetic | 1 | 7 | 0 | ✓ |
| agent_human | 1 | 7 | 0 (proposal kalır) | ✓ |
| human_agent | 1 | 7 | 2 | ✓ |
| human_transcript | 1 | 7 | 2 | ✓ |
| synthetic_agent | 1 | 7 | 2 | ✓ |
| mixed_vault | 4 | 22 | 3 | ✓ |

- **Ayırt edicilik:** aynı 9 vault `p1b-final` çekirdeğiyle açılınca human_agent, human_transcript, synthetic_agent ve mixed_vault (4/4 doküman) P1-B'deki F1 hatasını verdi (`snapshot committed before LayerChanged`). Diğer 5 senaryo açıldı. P1-B F1'i kapandı.
- **Ek:** `v5/` kopyasında `scripts/gen_v3final_mixed.sh` (varsayılan hedef) ile commit'li `core/tests/fixtures/v3_final_mixed/` yeniden üretildi. `git status` 0 satır: 110 dosya bayt eşit. Üretecin kaynağı yalnız v3-final API'sini çağırıyor (`Vault::open`, `ingest`); olay elle kurulmuyor.

## V3 — B1-2, kasıtlı `LayerChanged` silme `[OK]` (bilinen sınırla; bkz. Ş1)
HEAD çekirdeğiyle üretilmiş yeni biçimli günlükler kullanıldı: human_agent ve synthetic_agent'ta 2'şer, mixed_vault'ta 3 `LayerChanged` var. Her `LayerChanged` satırı tek tek silindi; her silme iki biçimde denendi: (i) `seq` boşluğu bırakılarak, (ii) `seq` yeniden numaralanarak.

| Vaka | (i) boşluk | (ii) yeniden numaralı |
|---|---|---|
| human_agent #0 / #1 | `Err` / `Err` | `Err` (`LayerChanged … does not match`) / `Err` (`snapshot committed before LayerChanged`) |
| synthetic_agent #0 / #1 | `Err` / `Err` | `Err` / `Err` |
| mixed_vault #0 (günlüğün ilk katman taşıyan olayı; c.md'nin snapshot'ında tek) | `Err` | **açılıyor:** katman aynı (`proposal`), `layer_source = derived` |
| mixed_vault #1 / #2 | `Err` / `Err` | `Err` / `Err` |

- Satırı silmek (GOAL V3'ün tarif ettiği bozma) **7/7 `Err`** veriyor. Yeniden numaralandırılmış silmede 6/7 `Err`.
- Açılan tek vaka K-B1.1'in DECISIONS'a yazılmış sınır notuyla ve ASSUMPTIONS B11-2 ile birebir: dosya bayt bayt "v3-final ile yazılmış, sonra mevcut çekirdekle sürdürülmüş" bir vault. Katman yükselmiyor.
- Zincir içindeki `compat_v3final` testleri (15) de geçti, B1-2'nin üç adlandırılmış testi dahil.

## V4 — `trace_id` formülü ve B1-4/5 `[OK]`
- **Bağımsız hesap** (`v4ids.py`, blake3, ilk 32 hex): `H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" + at_seq)`.
  - Resmi RUN'ın 4 iz satırı **4/4** eşit, A RUN'ı da 4/4.
  - Her satırda `at_seq` = satır sırası, `checkpoint_id` mevcut bir `checkpoints/<id>.json`, `query_hash` 32 küçük harf hex, `returned` nesne biçiminde.
  - Eski (1.0) formülün değeri hiçbir satırla eşleşmiyor.
- **P1-B ↔ P1-B.1 izleri:** P1-B resmi RUN'ının 4 satırı eski formülle eşleşiyor. İki turun satırları `trace_id` dışında alan alan aynı. `events.jsonl` ve `checkpoints/` iki tur arasında bayt eşit.
- **B1-4** (kütüphane API'si, `vtrace.rs`, `record_trace`):
  - 31 hex, 33 hex, hex olmayan `query_hash` ve `\n` içeren `consumer` → **4/4 `Err`**. Ek olarak büyük harf hex → `Err` (B12-1).
  - Her `Err`'de `traces.jsonl` bayt boyu değişmedi.
  - Geçerli girdi `Ok`; yazılan satırın kimliği benim hesabımla eşit.
- **B1-4, açılış tarafı** (`v4open.py`): `traces.jsonl` elle yazıldı; geçersiz alanlarla, ama `trace_id` doğru hesaplanmış satırlar kullanıldı. `mdk trace list` 31/33/hex olmayan `query_hash` ve `\n`'li `consumer` için **4/4 `Err`** verdi. 1.0 formüllü kimlik de `Err` (`trace_id … != …`, B12-2). Değiştirilmemiş kontrol satırı açılıyor.
- **B1-5:** (`"ab"`, qh) → `Ok`. (`"a"`, `"b"+qh`) → `Err`: 33 karakter, `query_hash` biçimi. Saf fonksiyon olarak da yeni formül iki kimliği ayırıyor, eski formülde ikisi çakışıyor.
- K-B1.2'deki düzeltilmiş örnek (`qh` "1" ile bitiyor; ("ab", qh, 2) ↔ ("a", "b"+qh[0..31], 12), ikisi de geçerli 32 hex) eski formülde gerçekten çakışıyor, yenisinde ayrılıyor. P1-B Ş1 kapandı.

## V5 — C2-07 ve MANIFEST `[OK]`
- `v5/` kopyasında `spec/fixtures/C2-07/expected.json` silindi, ardından `bash scripts/gen_fixtures_c2.sh` çalıştırıldı (çıkış 0).
  - `cmp` yeni üretilen ↔ `p1b-final` beklenen: **farklı** (satır 116).
  - `cmp` yeni üretilen ↔ HEAD beklenen: **aynı**.
  - Üretimden sonra bütün `spec/fixtures` ağacı `git status` ile 0 satır; C1 ve manifest dahil her şey bayt eşit üretildi.
- Eski ve yeni `expected.json` arasındaki fark yalnız iki `trace_id` ve aynı ikisini taşıyan `result` dizgisi:
  - `052ae79a…` ve `8e2df27f…` eski formülle eşit;
  - `17536b4d…` ve `f4ee3c57…` yeni formülle eşit. Kendi hesabım.
- `git diff p1b-final HEAD -- spec/fixtures/MANIFEST.json`: **tek satır** değişmiş, `C2-07/expected.json` `ead96c1e…` → `49b9bf5b…`. `git diff --stat p1b-final HEAD -- spec/fixtures`: yalnız bu 2 dosya (4+/4−).
- C1 fixture'ları `p1a-final`'den beri değişmemiş (`git diff p1a-final HEAD -- spec/fixtures/C1-*` boş).
- A zincirinde `conformance.json`: `level_claimed 2`, `spec_version 1.0.1`, `fixtures_manifest ok`, **25/25**; C2-07 `03f9907946bb4e238ae6937604c33e58` (FINAL ile aynı) → **B1-3 geçer**.

## V6 — F-istisnaları, üç şart `[OK]`
Her istisnada DEV_PROCESS §4'ün üç şartı arandı: (a) girdi: ne, neden, diff; (b) önce/sonra bit eşitliği; (c) gözden geçirici onayı. Ayrıca `research/FROZEN.md`'ye eklenen bir yeniden dondurma satırı arandı. (b)'yi her istisna için kendim yeniden ürettim.

| İstisna | (a) | (b) — kendi denemem | (c) | FROZEN satırı |
|---|---|---|---|---|
| F1 `check_frozen.sh` | ✓ diff tek satır (kalıba `(\.[0-9]+)?` eklenmiş) | Eski denetleyici (p1b-final) HEAD'de çıkış 1 veriyor. İhlallerin **hepsi** yalnız P1-B.1 satırlarında (`docs/rounds//progress` — noktalı kimlik eşleşmiyor). P1-B/F1–F8 atıfları ihlal üretmiyor. Yeni denetleyici çıkış 0. | B1.0.review "F1 onaylandı" | ✓ |
| F2 `spec/fixtures/**` | ✓ | V5: yalnız C2-07 değişmiş ve formüllerle doğrulandı; yeniden üretim bayt eşit | B1.2.review | ✓ |
| F3 `conformance.rs` | ✓ diff tek satır (`spec_version`) | P1-B resmi RUN ↔ P1-B.1 resmi RUN `conformance.json` yaprak karşılaştırması: farklar yalnız `spec_version`, `fixtures_root` (yol), `fixtures_manifest_hash` ve C2-07'nin iki hash'i | B1.2.review | ✓ |
| F4 `check_v3_compat.sh` | ✓ | `v5/` ağacında `p1b-final` betiği ile HEAD betiği çalıştırıldı: stdout ve `<out.json>` **bayt eşit**, ikisi de P1-B resmi RUN'ın `v3_compat.json`'ı ile bayt eşit (12 doküman). Yeni betik ayrıca `mixed-author vaults 9/9` | B1.3.review | ✓ |

- `bash scripts/check_frozen.sh` klonda → `OK`. Tablo `63a4004` ve `983c62d`'den beri yalnız ekleniyor; bu turda 10 satır eklendi, silinen ya da değiştirilen yok.
- `v6/` kopyasında kasıtlı bozmalar:
  - dondurulmuş `research/eval/compare_p1b1.py` değiştirilip commit'lendi → çıkış 1 (`changed since 226b511`);
  - F4 satırının atfı var olmayan `P1-B.1/F9` yapılıp commit'lendi → çıkış 1;
  - geri alındıktan sonra → çıkış 0.
- `research/eval/**` altında değiştirilen dosya yok. Bu turda yalnız 3 yeni dosya eklendi (`compare_p1b1.py`, `run_p1b1_checks.py`, `report_p1b1.py`). Üçü de `226b511`'de (B1.4-pre) resmi RUN'dan önce kendi satırlarıyla donduruldu.
- GOAL.md `63a4004`'ten (ROUND_START) beri değişmemiş; NORTH_STAR ve `spec/CONFORMANCE.md` `p1b-final`'den beri değişmemiş.
- `index`, `server`, `bindings`, `clients` `p1b-final`'den beri değişmemiş. `check_deps.sh` OK.
- Not: `core/tests/traces.rs`'te 2 satır değişmiş (B12-3). Değişen iki şey var:
  - eski formülü elle yazan beklenti;
  - geçersizleşen `"q"` girdisi.
  
  İkisi de K-B1.2'nin değiştirdiği davranışın kendisi; dosya dondurulmuş değil. "Test geçsin diye test değiştirme" sayılmadı.

## V7 — K-B1.3 ile SPEC_EDITS tutarlılığı `[OK]`
- `git diff -U0 p1b-final HEAD -- spec/SPEC.md` tam olarak SPEC_EDITS'teki S1–S6 satırlarından oluşuyor: satır 1, 3, 89 (S3+S4), 98 (S5+S6). Başka satır yok, §1–6 metni değişmemiş.
- Kendi sınıflandırmam, `docs/SPEC_EDITS_CRITERION.md`'deki (a)/(b) tanımıyla:
  - S1 (sürüm) ve S2 (durum/sürüm notu): (a) ✓ (b) ✓ → açıklık / sürüm. Tutarlı.
  - S3 (küçük harf MUST): (a) ✓ (b) ✗ → serileştirme kararı. Tutarlı.
  - S4 (undeclared synthetic → canonical): (a) ✓ (b) ✗ → normatif ek, K-B1.4. Tabloda "anlam değişikliği" olarak sınıflanmış. (b) sağlanmadığı için ölçüt "açıklık değil" der; iki sınıftan hangisi seçilirse seçilsin karar kaydı var. Tutarlı.
  - S5 (`returned` nesne MUST): (a) ✓ (b) ✗ → serileştirme kararı. Tutarlı.
  - S6 (formül ve doğrulama): (a) ✗ (b) ✗ → anlam değişikliği, K-B1.2. C2-07 yalnız bu kararla yeniden üretildi (V5). Tutarlı.
- P1-B düzenlemelerinin yeniden sınıflandırması (CRITERION tablosu):
  - E1 ve E2 serileştirme kararı. P1-B doğrulamasındaki Ş2 bunu istemişti; K15-T'nin ilgili kısmı "kısmen kapandı" notuyla kapatılmış.
  - E3 bilgilendirici not → açıklık.
  - E4 karar kuralına bağlı damga → açıklık.
  - Ölçütle tutarsız bir sınıflandırma bulmadım.
- Sürüm kuralı (CRITERION: Validated/Draft bölümlerini etkileyen değişiklik yama sürümüdür) → 1.0.1 doğru. Küçük metin farkı için bkz. Ş2.

## V8 — Karar kuralı `[OK]`
GOAL'den türettiğim kural: B1-1..B1-7 geçerse P1-B karar kuralının B-5 dalı kapanır ve P1-C başlar.

| Kriter | Benim ölçümüm |
|---|---|
| B1-1 | 9/9: kendi derlediğim v3-final ve kendi senaryolarım (V2); zincirde `check_v3_compat` 9/9 ve `compat_v3final` geçti |
| B1-2 | satır silme 7/7 `Err` (V3) |
| B1-3 | C1 17/17, C2 8/8, `level_claimed 2`, spec 1.0.1 (V5) |
| B1-4 | 4/4 `Err`, API'de ve açılışta (V4) |
| B1-5 | farklı; ikinci vaka `Err` (V4) |
| B1-6 | P1-B resmi RUN ↔ A RUN: 2426 kalite yaprağı, hariç tutulanlar dışında **0 fark** (V1, `cmpjson.py`). Hariç tutulanlar: FINAL'deki 5 alan, ayrıca `conformance.json`'da yalnız C2-07, manifest hash, `spec_version` ve yol. `events.jsonl` ve `checkpoints/` bayt eşit; `traces.jsonl` yalnız `trace_id` alanında farklı (yeni beklenen). |
| B1-7 | fark 0, bayt eşit (V1) |

**B1-1..B1-7 geçti → P1-C başlar.** FINAL.md'nin sonucu bağımsız olarak yeniden üretildi.

Kalite kapıları (`v5/`, HEAD):
- `make test` çıkış 0: cargo 120 test geçti / 0 kaldı; pytest 169 geçti / 9 atlandı; bu turda TS istemci paketi yok.
- `make lint` çıkış 0 (check_frozen ve check_deps dahil).
- `make conformance` ve `make verify` zincir içinde temiz (V1, V5).

---

## Sonuç

### İhlaller
- Yok.

### Farklar
- Yok.

### Şüpheler
- **Ş1 — B11-2'nin koşul metni gerçek davranıştan geniş; ayrıca yeniden numaralı silmenin bir alt sınıfı tespit edilemiyor.**
  - ASSUMPTIONS B11-2 şöyle diyor: "silinen satırdan önce katman taşıyan bir olay yoksa … açılır". Ama human_agent ve synthetic_agent'ta günlüğün **ilk** `LayerChanged`'i silinip yeniden numaralanınca açılış `Err` veriyor. Sebep: aynı snapshot'ta ikinci bir `LayerChanged` kalıyor ve günlüğü "katman taşıyan" yapıyor. Davranış metinden daha güvenli tarafta, yani ihlal değil; yalnız metin yanlış genelliyor.
  - Gerçekten açılan vaka (mixed_vault #0) K-B1.1 sınır notunda kabul edilmiş. Katman yükselmediği için güvenlik açığı değil. Yine de B1-2'nin "eksik → `Err`" özelliği yeniden numaralandırmaya karşı genel olarak tutmuyor.
- **Ş2 (bilgi) — GOAL K-B1.2 "Spec §8 Draft olduğu için" diyor, spec ise §8'i "Validated (P1-B)" olarak gösteriyor.** Sürüm sonucu (1.0.1) CRITERION'un yama kuralıyla yine doğru; yalnız gerekçe cümlesi spec durumuyla çelişiyor. GOAL değiştirilemez olduğu için not olarak bırakıldı.
- **Ş3 (bilgi) — "MUST be rejected" maddesi yalnız başvuru uygulamasının testlerinde sınanıyor.** `spec/CONFORMANCE.md`'de C2-07 yalnız "trace line equals fixture" ölçüyor. 1.0.1'in yeni normatif gereği (`query_hash` 32 küçük harf hex, `consumer`'da `\n` yok, ihlalde ret) için uyumluluk fixture'ı yok. Bu yüzden başka bir uygulama doğrulama yapmadan Seviye 2 alabilir.

### Hüküm
**Güvenilir.**

FINAL.md'deki her ölçüm bağımsız olarak yeniden üretildi:
- çift koşturmada ve resmi RUN ile 0 fark, bayt eşitlik;
- kendi derlediğim v3-final ve kendi senaryolarımla B1-1 9/9 (eski çekirdek aynı vakalarda F1'i veriyor);
- B1-2 satır silmede 7/7 `Err`;
- 25/25, L2, spec 1.0.1;
- `trace_id` formülü 4/4, B1-4 4/4 `Err`, B1-5 farklı;
- C2-07 ve MANIFEST yalnız beklenen yerde değişmiş;
- F1–F4 üç şartı taşıyor;
- K-B1.3 sınıflandırması tutarlı.

Karar kuralının sonucu **P1-C başlar**. Ş1'deki sınır kayıtlı bir kabul olarak okunmalı.

### Öneriler (en fazla 3)
1. **B11-2 metnini gerçek koşula göre düzeltin.** Doğru koşul: silinen `LayerChanged`'den sonra, aynı snapshot'a kadar başka katman taşıyan olay da yoksa günlük türetilmiş önek olarak açılır; aksi hâlde `Err`. Yukarıdaki iki `Err` vakasını (aynı snapshot'ta iki `LayerChanged`, ilki silinip yeniden numaralı) teste sabitleyin. Ayrıca yeniden numaralı silmenin tespit edilemeyen alt sınıfı için açık bir karar alın: ya kabul edilmiş sınır olarak bırakın ya da yeni vault'lar için bir sonraki majör spec sürümünde günlük biçim işareti tanımlayın (§4.2 donmuş).
2. **1.0.1'in ret gereği için uyumluluk vakası ekleyin.** CONFORMANCE'a, karar kaydıyla, Seviye 2 bir negatif fixture eklenmeli: geçersiz `query_hash` (31/33/hex olmayan/büyük harf) ve `\n`'li `consumer` içeren `traces.jsonl` satırı → ret.
3. **Bir sonraki GOAL'de §8 durumunu doğru yazın** (Validated, P1-B). Sürüm kararlarının gerekçesini CRITERION'un sürüm kuralına bağlayın; böylece GOAL ile spec arasındaki Ş2 türü çelişki tekrarlanmaz.
