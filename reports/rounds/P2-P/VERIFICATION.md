# Blok 1 (P2-L + P2-P) — Bağımsız Doğrulama (dar kapsam; K-P2.4)

Doğrulanan commit `main` `df50673`. Bu commit P2-P PR #25'in rebase-birleşik başıdır; ağacı (`3ab3d34`) PR başı `5cfef2a` ile aynıdır.

- P2-L PR #23 `fd9cbc2` olarak birleşti. Bu, FINAL'de ölçülen `ced61f9`'un rebase karşılığıdır.
- P2-P'nin ölçülen commit'i `42b4494`'tür; rebase karşılığı `d60df27`'dir. Ondan sonraki commit'ler yalnız kayıt içerir.

Yönergeler: `docs/rounds/P2-P/VERIFY.md` (VP1–VP7), `docs/rounds/P2-L/VERIFY.md` (VL1–VL8), DEV_PROCESS §5.

Çalışma ortamı:
- GitHub'dan temiz klon aldım ve her koşturmayı kendi scratch dizinimde yaptım. Yeniden kullandığım hiçbir ürün çıktısı yok.
- Ölçtüğüm ikili `df50673`'ten kendi derlememdir (`rootmark 0.2.0`).
- Eski ikililer v0.1.0 ve v0.2.0 release varlıklarıdır (aarch64-apple-darwin).
- Senaryo betikleri benim yazdığım geçici Python betikleridir (CLI, HTTP ve MCP stdio kullanır) ve depoya girmedi.
- Gerçek Obsidian kullanmadım; VERIFY.md bunu gerektirmiyor.

## Hüküm

**Güvenilir.**

Gerekçe:
- İki turun bütün kriterlerini kendi koşturmamla yeniden ürettim. Tam çift fark 0, sayılan FAIL iki zincirde de 0.
- 12 D-TR hücresi FINAL tablosuyla dört ondalıkta aynı.
- Karar kurallarının iki dalını da bağımsız türettim; iki FINAL ile aynı çıktı.
- Güvenlik örneklerimin hiçbiri tasdiksiz ya da `unverified` içeriği kuralla terfi ettiremedi.
- Eski vault'larda `events_hash` değişmedi.

Bulduğum farklar (aşağıda) ihlal değildir: biri yönerge metniyle ürün arasında bir ifade farkı, ikisi gözden geçiricinin zaten bıraktığı düşük bulgular, biri de süreçten sapmadır. Süreç sapması, doğrulamanın ayrı bir oturumda koşmamasıdır (§ "İhlal / fark / şüphe", madde 1). Hükmü ölçümler belirler; bu sapmanın kabulü insan kararıdır.

## Tam çift (VL1, P-6, P-8; bir kez)

`bash scripts/verify.sh pair` (`make verify`'ın yaptığı şey), `df50673`'te:

| | RUN | Süre |
|---|---|---|
| A | `20261025T104107Z-df50673` | 10:37 – 11:46 |
| B | `20261025T114629Z-df50673` | 11:46 – 12:47 |

Sonuçlar:
- **Çıkış 0.**
- `pair_compare.json`: `n_fields` **189 372**, `differing_fields` boş, `bytes_equal` true. Bu, FINAL'deki alan sayısıyla aynı.
- Karşılaştırılan dosyalar arasında `p2l/d_tr.json`, `p2l/english.json`, `p2l/migrate.json`, `p2l_checks.json`, `p2l_quality.json` ve `p2p_checks.json` var.
- İndeks baytları D-TR'nin dört kurulumunun manifest `records_blake3`, `snapshots_blake3` ve `embedders_blake3` alanlarıyla karşılaştırılıyor. Yinelenen hücre çiftlerinin dördü de eşit (`duplicates_equal`).
- Sınıflandırıcı `eval.historical_p2p` (P2-L resmi RUN `20261024T184609Z-ced61f9`'a karşı):
  - A: 29 tarihsel + 2 göç gerekli, **sayılan FAIL 0**.
  - B: 30 + 2, **sayılan FAIL 0**.
  - Fark tek kayıttır: `H-P2-0.3b-yük`. Yarışa bağlıdır (K-P2-03.1); A'da 0.0189 geçti, B'de 0.0732 kaldı (eşik 0.02). Karşılaştırmaya girmez.
  - FINAL'de sayılar tersti (A 30, B 29). Aynı sınıf.
- İki zincirde de `p2l_checks.json` (L-1, L-7, L-8, L-10, L0, L2) ve `p2p_checks.json` (P0, P-2/P-3, P-3/P-6, P-4/P-5, P-7) `pass: true`.

İlk çift denemem çift karşılaştırmasına ulaşmadı. A zincirinin P2-L adımında `make models-multilingual` `curl: (56) Recv failure: Operation timed out` ile düştü (Hugging Face, altyapı) ve B hiç başlamadı. Kayıtları sakladım. HF'nin yanıt verdiğini (HEAD 200) doğruladım, iki çalışma dizinini silip çifti baştan bir kez koşturdum. Yukarıdaki sonuç bu ikinci, tam koşturmadır; ürün tarafında bir hata yok.

## P2-L

### VL1–VL8

- **VL1 — geçti.** Yukarıdaki tam çift: iki tokenizer ve iki embedder ile indeks manifest blake3'leri ve 12 hücre fark 0.
- **VL2 — geçti.** Kaynak, tohum ve 12 hücre:
  - D-TR manifestinin revizyonu `b04c8d1…`. İki parçanın sha256'sını HF `paths-info` ile ayrıca teyit ettim (tr `6994014607…`, en `382e7f6f…`). Üreteç sha256'sı `788a3fc3…` ile eşleşiyor.
  - Ön-kayıt commit'i `94376ed` (17:00:11Z), ölçüm RUN'u `20261024T170157Z`'den önce; tohum `3775637042`. `d_tr.py`, manifest ve `run_p2l` ön-kayıttan sonra değişmemiş.
  - 12 hücreyi kendi çiftim üretti; FINAL ile aynı:

    | Kip | `mdk_lower_alnum` / minilm | `uni_fold_tr` / minilm | `mdk_lower_alnum` / multilingual | `uni_fold_tr` / multilingual |
    |---|---|---|---|---|
    | lexical | 0.9949 | 0.9952 | 0.9949 | 0.9952 |
    | flat | 0.5715 | 0.5715 | 0.6475 | 0.6475 |
    | hybrid | 0.8029 | 0.8097 | 0.8676 | 0.8801 |

  - Eşik dalları:
    - L-2 = **+0.00029** (< +0.10, kaldı).
    - L-5 = **+0.07592** (≥ +0.05, geçti).
    - L-6 = 0.8801 − 0.8029 = **+0.07725** (< +0.10, kaldı). "Eski en iyi"yi eski tokenizer'ın en iyi hybrid hücresi (0.8676) diye okusam bile +0.0125 olur ve dal değişmez.
- **VL3 — geçti.** `english.json`:
  - `mdk_lower_alnum` 618 değerde fark 0, `differing` boş; dört hücrede Δ 0.
  - `uni_fold_tr` Δ: hybrid single −0.00249, hybrid multi −0.00056, flat 0 / 0. |Δ|max 0.0025 > 0.001 olduğundan GOAL'e göre opt-in.
  - Varsayılan `mdk_lower_alnum` kaldı. Bunu kendi denememde de gördüm: varsayılan tarifle kurulan vault'un manifesti `mdk_lower_alnum`.
- **VL4 — geçti.** Kendi vakalarım (geçici bir crate'te `rootmark-index` tokenizer'ı):
  - Belgelenen Türkçe katlama doğru çalışıyor: `İSTANBUL` → `istanbul`, `ILIK` → `ılık`, "değerlendirme" ve "Şengönül" tek token, NFC önce.
  - Belgelenmiş ödünler:
    - Türkçe kural dil bağımsız uygulanıyor: `ISPARTA` → `ısparta` (`isparta`'ya eşit değil), `COVID` → `covıd`, "I am IBM" → `ı`, `ıbm`.
    - `ΟΔΟΣ` → `οδοσ`: son sigma katlanmıyor.
    - `STRASSE`, `straße`'ye eşit değil.
  - Emoji ve rakam-harf karışık girdiler belgelendiği gibi.
  - `mdk_lower_alnum` ASCII dışını düşürüyor (değişmemiş eski davranış).

  Bu ödünler, `uni_fold_tr`'nin Türkçe dışı Büyük-I içeren metinde varsayılan olmaması için ayrı bir nedendir.
- **VL5 — geçti; bir ifade farkı var.**
  - Vault dışına çıkma denemelerinin hepsi HTTP 400 `invalid_arguments` aldı ve hiçbir şey kaydedilmedi: `../`, `notlar/../../etc`, `/etc`, `/`, boş, `./notlar`, `C:/x`, NUL, `[]`, 65 öğe, `notlar//genel.md`.
  - Filtresiz `query_hash` 0.2.0 ikilisininkiyle aynı (`54fccb80…`), aynı isabetler.
  - `trace` `filter` alanını taşıyor; `trace_filters.jsonl` yazılıyor.
  - **Fark:** NFD bir arama öneki reddedilmiyor, NFC'ye katlanıp kabul ediliyor (200, doğru belge). VL5 metni "NFD varyantı reddediliyor" diyor.
    - Ürün bunu bilerek yapıyor: commit `a37bf94` "the search filter keeps folding it" der. GOAL P2-P'nin NFD reddi yalnız kural önekine bağlı ve orada ret var (VP2 d).
    - Katlama vault dışına çıkış açmıyor; bu bir ifade farkıdır, ihlal değil (öneri 1).
- **VL6 — geçti.**
  - 0.1.0'ın ve 0.2.0'ın kurduğu indekslerle (`mdkernel-index/3`, `mdk_lower_alnum`), tarifte `tokenizer = "uni_fold_tr"` iken matris indeks için `migration_required` gösteriyor.
  - `rootmark migrate` `index_rebuild` uyguluyor: manifest `uni_fold_tr`, `events_hash` eşit, Türkçe lexical arama belgeyi buluyor.
  - Resmi `p2l/migrate.json` 3/3: rollback bayt eşit, blame ve replay eşit.
  - Not: `serve` `auto_rebuild` ile açılışta indeksi kendisi yeniden kuruyor. Matrisi bu yüzden `serve`'den önce okudum.
- **VL7 — geçti.**
  - `v0.2.0` etiketi `fc645ed`'de; bu `main`'in atası.
  - `release.yml` koşusu 40113904549 üç hedefte başarılı.
  - `SHA256SUMS` 5/5, `gh attestation verify` 5/5.
  - Linux tarball'ı Docker `debian:trixie-slim` amd64'te `rootmark --version` → `rootmark 0.2.0` verdi; `conformance` seviye 3, 39/39 (spec 2.0.2, model indirildikten sonra).
- **VL8 — geçti.**
  - `check_frozen` OK (F1–F3), `check_deps` OK, `check_dep_age` OK (102 bağımlılık).
  - FAST kipi tam zinciri değiştirmemiş: `verify.sh`'ten silinen satırlar `d6ac64c`, `b38ccd7` ve `c7cc6ab`'de yalnız F3, F6 ve P2-P F2 kayıtlarının tarif ettikleri. `research/eval` `fc645ed`'den bu yana 2465 ekleme, 0 silme.
  - Karar kuralı GOAL'den (aşağıda).
  - NORTH_STAR notu K-P2-03.1 ile uyumlu: yarışa bağlı `unverified` sınıfı ve kesin varsayılan `canonical`.

### P2-L kriterleri

| Kod | Benim ölçümüm | Sonuç |
|---|---|---|
| L-1 | `p2_l_tokenize` 6/6 (iki zincir); VL4 vakalarım belgelenen davranış | geçti |
| L-2 | +0.00029 | **kaldı** |
| L-3 | 618 değer, fark 0 | geçti |
| L-4 | \|Δ\|max 0.0025 > 0.001 → `uni_fold_tr` opt-in, varsayılan `mdk_lower_alnum` | raporlandı (opt-in) |
| L-5 | +0.07592 | geçti |
| L-6 | +0.07725 | **kaldı** |
| L-7 | `p2_l_l3` 6/6; VL5 kaçış denemeleri ret; filtresiz `query_hash` 0.2.0 ile eşit; `trace` filtreli | geçti |
| L-8 | `migrate.json` 3/3; VL6 iki eski ikiliyle | geçti |
| L-9 | Tam çift fark 0 (indeks manifest blake3'leri, 12 hücre) | geçti |
| L-10 | `p2_l_conformance`; kendi derlemem 39/39, C3-06 `10f03eb0e5f58b0829041bb9e2d716c4` beklenenle aynı | geçti |
| L-11 | VL7; `p2_l_l0` 6/6 (Ö3 testleri); FAST süresini yeniden ölçmedim (FINAL: 735 s) | geçti |

**Karar dalı (bağımsız türetim):** L-2 ve L-6 kaldı, ötekiler geçti. Bu, "L-2, L-5 veya L-6 kaldı (diğerleri geçti)" dalıdır: yeni varyantlar opt-in kalır, kazanç "ölçüldü, eşik altı", karar talebi K-P2L-T2, P2-P başlar. P2-L FINAL ile **aynı**.

## P2-P

### VP1–VP7

- **VP1 — geçti.** Kendi tarifimi kurdum:
  - Kurallar: örtüşen `r_team` (`agent:a`, `notes/team/`) ve `r_all` (`agent:a`, `notes/`); `r_bsrc` (`agent:b`, source `collector`); `r_bmisc` (`agent:b`).
  - Aktörler: CLI, MCP `clientInfo` ve HTTP insan.

  Dokuz tahmini koşturmadan önce yazdım; dokuzu da tuttu:
  1. `notes/team/` için `r_team`, `r_all`'a karşı kazanıyor.
  2. `notes/teamx/` `r_all`'a düşüyor; önek yol bileşeni bazında eşleşiyor.
  3. `r_bsrc` CLI source `b`'ye uymuyor.
  4. `r_bmisc` `agent:b`'yi terfi ettiriyor.
  5. HTTP insan `propose` için `promoted: null`.
  6. MCP `a/1.0` `agent:a` olarak eşleşiyor.
  7. MCP `ab/1.0` eşleşmiyor.
  8. `Promoted.meta` dört alanı taşıyor: `actor {kind: policy, id: <kural>}`, `reason "rule:<id>"`, `policy "rules"`, `policy_version`, ayrıca `from proposal` ve `to canonical`.
  9. `history`'deki olay `propose` yanıtının `promoted` olayıyla bayt bayt aynı; toplam 10 `Promoted`.
- **VP2 — geçti.**
  - (a) Kuralın önekine ve source'una uyan tasdiksiz dosya yazımı (`--watch` ile), `unverified` ve `canonical` tarifleriyle: 0 `Promoted`. Yalnız sonradan gelen ajan `propose`'unun kendi belgesi terfi etti.
  - (b) `unverified` soy ve geç `rootmark attest`:
    - Tasdik kaydedildi (`at_seq` 3), bloklar `unverified` kaldı; kural onları terfi ettirmedi.
    - Eşleşen bir `append` yalnız çağrının kendi iki yeni soyunu terfi ettirdi.
    - Not: benim geç tasdikim pencere ya da eşleşme koşulu yüzünden `proposal`'a da çıkmadı. Ölçütün güvenli yönü bu.
  - (c) `actor = "unattested_fs"`, `"human:x"`, `"policy:p"`, `"agent:"` ve `"agent"` içeren tarifler `policy lint` çıkış 1 veriyor ve `serve` başlamıyor; hata metni alanı adlandırıyor.
  - Ayrıca `promote --actor policy:r_team` / `agent:a`, `human_only` altında reddediliyor.
  - (d) Şu `path_prefix` değerleri yüklemede reddediliyor: `../`, `notes/../../x/`, `/etc/`, boş, `./`, NFD, `C:/x/`. NFD için mesaj NFC'yi adlandırıyor.
  - (e) Başlıksız `append` ve `replace_section` `append_only` altında reddediliyor: olay yok, dosya ve `events.jsonl` bayt eşit.
- **VP3 — geçti.**
  - P2-A'nın altı eski vault'u: `events_hash` `migrate`'ten önce ve sonra beklenenle eşit (e2e `f4100b64…`, mixed `952cf483…`, scene `10972fd9…`, attested `ce178f13…`, bulk `973c3a19…`, partial `bcab134c…`). Replay ve katmanlar eşit.
  - `rootmark conformance` 39/39 (spec 2.0.3); C3-06 aynen.
- **VP4 — geçti.** Vault'u v0.1.0 release ikilisi kurdu ve yazdı (tarifte `[attestation]` yok). Yeni ikilide:
  - `status`: `attestation.implicit: true`, `unattested: "unverified"`.
  - `migrations`: `required: []`, `recommended: [policy_attestation_1]`, `artifacts.policy` `recipe/implicit`. **`migration_required` değil.**
  - `serve` uyarı satırı var.
  - `migrate --dry-run` hiçbir şeyi değiştirmedi.
  - `migrate` yalnız `.rootmark/policy.toml`'u değiştirdi. Eklenen `\n[attestation]\nunattested = "canonical"\n`, eski baytlar önek, `events_hash` eşit; sonra `implicit: false`.
  - `--rollback` dosyayı bayt bayt geri getirdi; `implicit: true` yeniden.
  - İki not:
    - `--dry-run` planında `policy_attestation_1` hem `"recommended": true` hem `"required": true` görünüyor. Bu gözden geçiricinin bıraktığı bulgu (7); `status` doğru.
    - 0.1.0 vault'unda yeni ikilinin ilk `serve`'ü türetilmiş indeks dosyalarını yeniden yazıyor; günlük değişmiyor.
- **VP5 — geçti.**
  - HTTP `promote` ve `demote`, `X-Consumer: telegram-bot` ile `meta.source` taşıyor.
  - `X-Consumer`'sız HTTP isteği 400 "X-Consumer header required" alıyor ve 0 olay yazılıyor. Bu 0.2.0'dan beri böyle (`fc645ed` CLIENT_CONTRACT: `X-Consumer` zorunlu), yani "önceki gibi". Alan taşımayan terfi CLI yolu.
  - CLI `promote` `--source` olmadan alan taşımıyor; `history` source'u gösteriyor.
  - Patch uyumluluğu:
    - 0.2.0 ikilisi yeni ikilinin yazdığı günlüğü (`meta.source`'lu `Promoted`/`Demoted`, `policy` aktörlü `Promoted`) okuyor: `events-hash`, `replay`, `blame`, `history` çıktıları bayt eşit.
    - Ters yön: 0.2.0'ın yazdığı vault'u yeni ikili bayt eşit okuyor.
    - 0.2.0 **kurallı bir tarifi** `[promotion]: unknown field rules` hatasıyla reddediyor. Bu yüksek sesli bir ret; sessiz yanlış okuma değil ve tarif uyumluluğu spec'in kapsamında değil.
  - Spec: `fc645ed..HEAD` farkı yalnız başlık, §7 ve §8 satırlarını değiştiriyor; §1–6 boş, `spec/fixtures` değişmemiş (yalnız `spec/fixtures_variants` eklendi).
  - §7.1 damgası "Draft (… K-P2-03.1 …)", damga konmamış.
- **VP6 — geçti.**
  - `check_frozen`, `check_deps`, `check_dep_age` OK.
  - `ROUND_START`: P2-L `fc645ed` (PR #22'nin birleşmesi), P2-P `cfa31d6` (PR #24'ün birleşmesi); ikisi de `main`'de.
  - Dondurma istisnaları önce/sonra kanıtlı: silinen `verify.sh` satırları yalnız F3, F6 ve P2-P F2 kayıtlarının tarif ettikleri; `research/eval` yalnız ekleme.
  - FAST kipi tam zinciri değiştirmemiş: tam zincir kendi çiftimde 189 372 alanı üretti.
  - CI `df50673`'te yeşil.
  - Karar kuralı aşağıda.
- **VP7 — Vaat sorusu.** Aşağıda ayrı başlıkta.

### P2-P kriterleri

| Kod | Benim ölçümüm | Sonuç |
|---|---|---|
| P-1 | VP4 (0.1.0 release ikilisiyle); resmi `migrate.json`'da üç eski vault'un ilk göçünde `policy_attestation_1` | geçti |
| P-2 | VP2 c/d: kendi geçersiz tariflerim (aktör türleri, `..`, mutlak, NFD, boş, Windows) lint 1 / `serve` başlamaz, alan adlandırılır; zincirde `P-2/P-3` pass | geçti |
| P-3 | VP1: ilk eşleşen kazanır, dört alan, yanıt = `history` olayı | geçti |
| P-4 | VP2 a/b/e: 0 terfi; zincirde `P-4/P-5` pass | geçti |
| P-5 | VP3: kuralsız tarifle `events_hash`, 39/39, C3-06 aynen; S6b sıralamasını kendim ölçmedim, iki zincirde `P-4/P-5` pass | geçti |
| P-6 | Tam çift fark 0 (P4 sahnesinin `events_hash` ve `trace`'i `p2p_checks.json`'da) | geçti |
| P-7 | VP5; belgeler (`HEADLESS_CHANNEL.md`, THREAT_MODEL, CLIENT_CONTRACT, `SPEC_EDITS.md`) var | geçti |
| P-8 | Tam `make verify` bir kez, çıkış 0, sayılan FAIL 0; üç denetleyici temiz; FAST süresini yeniden ölçmedim (FINAL 779 s) | geçti |
| P-9 | Zincirde `P0` pass (a1–a4 testleri) | geçti |

**Karar dalı (bağımsız türetim):** hepsi geçti. Bu, "Blok 1 kapanır; dar doğrulama; 'güvenilir' ise P2-B başlar; §7.1 damgası doğrulayıcının görüşüne göre" dalıdır. P2-P FINAL ile **aynı**.

## VP7 — Vaat sorusu

NORTH_STAR'ın vaadi: "onaylanmamış hiçbir şey bilgi sayılmaz". Kural listesi bu vaadi kuralın kapsamında şuna çeviriyor: **"bu öneke, bu adla gelen ajan yazısını kuralı yazan insan önceden onayladı."**

Ölçtüğüm sınırlar:
- Onay hâlâ bir insanın elinden çıkıyor. Kural yalnız `policy.toml`'da, insanın yazdığı tarifle geliyor; `policy_version` ve kural kimliği her `Promoted` olayında kayıtlı.
- Kural yalnız `propose` çağrısının kendi soylarına uygulanıyor. Tasdiksiz dosya yazımı, `unverified` soy, geç tasdik ve reddedilen `propose` kural yolundan geçmiyor (VP2).
- Zayıf nokta onayın konusu, yani **ajanın adı**. Ad bir beyandır: MCP `clientInfo.name`, CLI `--source`. Ajan kanalında kimlik doğrulaması yok. MCP'ye konuşabilen ya da `rootmark propose`'u çalıştırabilen her süreç bir kuralın adını beyan edip o öneke `canonical` yazabilir. Bunu THREAT_MODEL ve `HEADLESS_CHANNEL.md` açıkça yazıyor, P2-P FINAL not 1 de.

Sonuç olarak "ajan yazdığı için canonical olan" içerik artık var, ama yalnız insanın açtığı bir önekte ve adı beyan edilen ajan için. Vaat "her içerik tek tek onaylanır" değil, "önceden ve kayıtlı onaylanır" hâline geliyor. Bu daha zayıf ama izlenebilir bir vaat.

**NORTH_STAR notuna ek gerekir.** Önerdiğim metin (insan kararı):

> **Kural listesi (P2-P, 2026-10-25; K-P2P):** tarifte `[[promotion.rules]]` tanımlayan insan, belirli bir öneke belirli bir ajan adıyla gelen `propose` yazılarını önceden onaylamış olur; bu yazılar `canonical` olur ve her biri kural kimliği ile tarif sürümünü taşıyan bir `Promoted` olayıyla kayıtlıdır. Ajanın adı bir beyandır (kimlik doğrulaması yoktur, P2-E'ye kadar): ajan sürecini çalıştırabilen herkes o adı kullanabilir. Bu yüzden kuralın kapsadığı önek, "kuralı yazan insanın önceden onayladığı" içerik demektir; tasdiksiz dosya yazımı ve `unverified` içerik hiçbir kuralla terfi etmez.

## §7.1 damga görüşü

**Görüşüm: "Validated" konabilir.** §8'in tasdik günlüğü için de aynısı geçerli. Tek koşul: aşağıdaki süreç sapmasının (madde 1) insan tarafından kabul edilmesi. Kabul edilmezse damga ayrı bir oturumun teyidini bekler.

P2-0.3 doğrulamasının "konamaz" gerekçeleri ve bugünkü durumları:
1. *Hüküm koşulluydu.* Bu doğrulamanın hükmü "güvenilir".
2. *"Default `unverified`" cümlesi ürünle çelişiyordu.* 2.0.3'te düzeltildi: §7.1 "default policy is `canonical` — the one an implementation gives a vault it sets up — and `unverified` an option".
   - Ürün yeni vault'a `canonical` yazıyor.
   - Tablosuz eski tarif `implicit: true` ile `unverified` okunuyor; önerilen göç `canonical`'ı açıkça yazıyor (VP4).
   - Bu, spec cümlesiyle çelişmiyor: cümle uygulamanın kurduğu vault'tan söz ediyor ve eski davranış K-P2L-T1 ile kayıtlı ve görünür.
3. *Açıklanmamış yeniden adlandırma düşüşü (Ö3).* P2-L'de hata olarak açıklandı ve düzeltildi (2.0.3 §7.1 E2); `p2_l_l0` 6/6, iki zincirde.

Kalan yarışa bağlı `unverified` sınıfı (`H-P2-0.3b-yük`, benim çiftimde 0.0189 / 0.0732) K-P2-03.1 ile kayıtlı. Yalnız `unverified` seçeneğinde görülüyor ve yalnız güvenli yönde. Bu, metnin tarif ettiği mekanizmanın doğrulanmasını engellemiyor.

Kapsam sınırı: Blok 1'de insan kanalını gerçek Obsidian'la yeniden ölçmedim (VERIFY.md gereği). Damga, P2-0.3'ün ölçtüğü insan kanalı davranışının Blok 1'de değişmediği varsayımına da dayanıyor. Spec farkı yalnız E2 ve varsayılan cümlesi; ikisi testli.

## İhlal / fark / şüphe

**İhlal:** yok. Ürün kodu, spec §1–6, fixture'lar, eşikler ve dondurulmuş ölçüm kodu kayıtsız değişmemiş.

1. **Süreç sapması (bağımsızlık).**
   - DEV_PROCESS §5 "ayrı oturum, ana dizine okuma dahil erişim yok" der. Bu doğrulama, ayrı bir arka plan oturumu başlatma izni verilmediği için orkestratörün oturumunda koştu.
   - Ana dizinden yalnız VERIFY yönergelerini okudum. Oturumun kalıcı notları (önceki turların doğrulama notları) bağlamımdaydı.
   - Bütün ölçümler temiz klonda ve kendi betiklerimle yapıldı; hiçbir sonuç ana dizinden alınmadı.
   - Hükmü ölçümler belirliyor. Sapmanın kabulü insan kararıdır ve damga görüşü buna bağlı.
2. **VL5 ifade farkı.** NFD arama öneki reddedilmiyor, NFC'ye katlanıyor. Kural öneki ise reddediliyor. Güvenlik açığı yok; iki yol arasında tutarsızlık var (öneri 1).
3. **`--dry-run` planında `required: true`.** `policy_attestation_1` için, `recommended: true` yanında. `status` doğru; plan çıktısı yanıltıcı (gözden geçirici bulgusu 7, bırakılmış).
4. **0.2.0 kurallı tarifi reddediyor.** Bu yüksek sesli bir ret, ama geri sürüme dönen kullanıcı için bir belge satırı eksik (öneri 3).
5. **İlk çift altyapı yüzünden düştü** (HF zaman aşımı); bir kez yeniden koşturuldu (yukarıda).
6. **`uni_fold_tr`'nin Türkçe dışı ödünleri** (`ISPARTA`, `COVID`, son sigma, `ß`). Belgeli; opt-in kararını destekliyor.
7. **`H-P2-0.3b-yük` A ile B arasında farklı.** Beklenen ve kayıtlı (K-P2-03.1); FINAL'deki A/B sayılarının ters olması aynı nedenden.

## Öneriler (≤ 3)

1. **NFD önek davranışını tek kurala bağla.** Arama filtresi NFD'yi katlıyor, kural yüklemesi reddediyor. Ya arama filtresi de reddetsin, ya da katlama CLIENT_CONTRACT'ta ve P2-L VERIFY metninde açıkça yazılsın. Kural tarafındaki ret korunmalı.
2. **Ajan kimliğini doğrulamadan kuralları "önerilen" yapma.** Kural listesinin vaadi ajan adının doğruluğuna bağlı. NORTH_STAR ekini (yukarıda) yaz; istemci başına kimlik doğrulamalı ajan kimliğini (P2-E) kural listesinin varsayılan önerilmesinin önkoşulu say.
3. **Göç çıktısını ve geri dönüş yolunu düzelt/belgele.** `--dry-run` planında `policy_attestation_1` için `required: true` olmamalı (bulgu 7). CHANGELOG'a ya da `HEADLESS_CHANNEL.md`'ye "kurallı bir tarif 0.2.0'da açılmaz; geri dönmeden önce `[[promotion.rules]]` kaldırılmalı" satırı eklenmeli.
