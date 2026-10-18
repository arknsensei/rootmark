# P2-0 — Bağımsız Doğrulama

- **Doğrulanan commit:** `ce4cf049a35cdb6fa659d0ea84d6deaafe702a1b` (`origin/p2-0/round`, PR #11'in başı). FINAL'in resmi RUN'ı `20261018T161850Z-e378be2` (commit `e378be2`); `e378be2..ce4cf04` arasında ürün ve ölçüm kodu değişmedi (`git diff --quiet e378be2 ce4cf04 -- core index server clients research/eval scripts Makefile`), `fa02519..e378be2` arasında yalnız `research/eval/report_p2_0.py` ve `research/FROZEN.md` değişti.
- **Not:** İstemde "PR #11 rebase ile main'e girdi" deniyordu; doğrulama anında PR #11 **açık** (`gh pr view 11`: `OPEN`, `mergedAt: null`) ve `origin/main` = `f628b90` (P2-0'ı içermiyor). Doğrulama doğrudan `ce4cf04` üzerinde yapıldı; bu dosya ana dizine (dal `main`) yazıldı ve başka hiçbir dosya değiştirilmedi.
- **Yöntem:** DEV_PROCESS §5. Ayrı oturum; scratchpad'de `ce4cf04`'ün temiz klonu (GitHub'dan); her koşturma orada. Ana dizinden yalnız bu dosya yazıldı. Kendi araçlarım (scratchpad, depoya girmez): V2 denetleyicisi, CDP sürücüsü (V3/V4), v0.1.0 karşılaştırıcısı (V6), anahtar zinciri denemeleri (V5). Ortam: macOS (Darwin 27.0.0, arm64), Obsidian `/Applications/Obsidian.app`, Node 24.3.0. Kullanıcının açık Obsidian'ına (PID 24764) dokunulmadı; her gerçek Obsidian koşusu kendi `--user-data-dir`'i ve CDP portuyla ayrı örnekti ve sonunda durduruldu (doğrulama sonunda 24764 hâlâ çalışıyor, scratchpad'den kalan süreç yok).

## Hüküm

**Şu koşullarla güvenilir.**

FINAL'in sayıları ve dalı bağımsız olarak yeniden üretildi: kendi çift koşturmamda D-B 931/1890 (iki zincirde de birebir), D-A 0/525 sızıntı, D-C 333/340, D-E 3/3, 37/37, kasıtlı vakalar 9/9, regress'te tek fark aynı (R16 `passed` 16 → 17); kendi bağımsız deneylerim (gerçek Claude Code ile 30 doğrudan yazım, gerçek Obsidian'da kendi sürücümle 50 düzenleme, kendi derlediğim v0.1.0) aynı hipotez sonuçlarını verdi. Hipotez bu turda kanıtlanmadı ve FINAL bunu dürüstçe söylüyor. Koşullar:

1. **H-P2-0e "ölçülmedi" olarak okunmalı** (benim koşumda da ölçülemedi: öğe bu Mac'te yok) ve tehdit modeli taslağındaki "terminal Obsidian'ı `--remote-debugging-port` ile başlatıp CDP üzerinden …" maddesi **açık soru değil, gösterilmiş bir yol** olarak yazılmalı: bunu V5'te 3/3 gösterdim (token'a ve anahtar zincirine dokunmadan ajan metni `canonical`). Aşama 2'nin (anahtar zinciri ACL'si) bu yolu kapatmadığı artık ölçülmüştür.
2. **`make verify`'ın FAIL özeti ("counted FAILs 15") FINAL'in sayımıyla (`historical_p2_0`: 0) birlikte okunmalı.** FINAL bunu açıklıyor; çıktıyı tek başına okuyan biri 15 sayılan FAIL görür (iki zincirimde de aynı satır). Sınıflandırma doğru, sunum yanıltıcı (öneri 3).
3. **Karar kuralının dalı:** GOAL'ün "0b kaldı, 0a geçti" dalının koşulu da harfiyen sağlanıyor; FINAL 0d'nin de kalması yüzünden "tanımlanmamış kalış"ı seçti (ASSUMPTIONS Z7-1, resmi koşudan önce yazıldı). Bağımsız türetimim bu seçimi destekliyor (aşağıda V9) ama insan kararı (K-P2-0.6-T) iki okumayı da görerek verilmeli.

## Hipotez başına

| Kod | FINAL (resmi RUN) | Benim çift koşturmam (A / B) | Bağımsız deneyim | Sonuç |
|---|---|---|---|---|
| H-P2-0a | 0 / 525 | 0 / 525, 0 / 525 | V2: gerçek Claude Code (bu oturum, Write/Edit/mv/rm) 30 yazım, 90 yeni blok, `canonical` 0 | **geçti** |
| H-P2-0a′ | 200/200 | 200/200 | V2: silinen/değişen 20 `canonical` soyun 20'si `history`'de `unattested_fs`, 20'si checkpoint'ten okunuyor | **geçti** |
| H-P2-0b | 931/1890 = 0.4926 | 931/1890, 931/1890 | V3: 69/159 = 0.434; yeniden adlandırma dışı 0/85 | **kaldı** |
| H-P2-0b′ | 0/286 (bilgi) | 0, 0 | — | bilgi |
| H-P2-0c | p95 270 ms | 298 ms, 286 ms | V3: p95 109 ms (35 tek kayıt) | **geçti** |
| H-P2-0d | 333/340 = 0.979 | 333/340, 333/340 | V4 gerçek Obsidian: 40/40; sıkı çakışma (ajan kayıttan 0–20 ms sonra yazınca) 8 insan bloğu `unverified`, toplam 64/72; ajan bloğu `canonical` 0/36 | **kaldı** |
| H-P2-0e | ölçülmedi | ölçülmedi | V5: öğe yok; geçici anahtar zincirinde ACL 8/8 okunamadı; CDP yolu 3/3 açık | **ölçülmedi** |
| H-P2-0f | 3/3 | 3/3, 3/3 | V6: kendi `git archive v0.1.0` derlemem, kendi 3 vault'um 3/3 | **geçti** |
| H-P2-0g | eşit | çıkış 0, 106 701 alan fark 0, bayt eşit | `cmp`: D-A/D-C `events.jsonl` + `attestations.jsonl` eşit | **geçti** |
| H-P2-0h | 37/37 | 37/37 | V7: 37/37, fixture'lar yeniden üretildi, `git status` temiz | **geçti** |

## V1 — Çift koşturma

`bash scripts/verify.sh pair <klon> <A> <B>` (klon `ce4cf04`, dal sabit): **çıkış 0**. A `20261018T182357Z-ce4cf04` (2596 s), B `20261018T190638Z-ce4cf04` (2555 s). `pair_compare.json`: `equal: true`, `n_fields: 106701`, `bytes_equal: true`. Kendi `cmp`'imle D-A ve D-C'nin `events.jsonl` ve `attestations.jsonl`'ı A ↔ B bayt eşit. D-B (gerçek Obsidian zamanlaması, `P2_0_TIMING`, karşılaştırma dışı): A 931/1890 p95 298 ms, B 931/1890 p95 286 ms, ikisinde de `latency_unsettled` 4 — FINAL'in "bu boyutta sapma beklenir" notuyla uyumlu; `unverified` sayısı üç koşuda da 931.

## V2 — Ajan sızıntısı (0a, 0a′): gerçek Claude Code

Vault: 5 insan notu (`ingest-dir`, `canonical`), checkpoint `before-agent`, `rootmark serve --http --watch` (varsayılan politika `unverified`, W 2.0). Dosya erişimli ajan bu oturumdur: Write/Edit araçlarıyla ve kabukta `mv`/`rm` ile doğrudan dosya sistemine **30 yazım**: yeni dosya (4, alt dizin, ön-bilgi, Unicode NFC/NFD, 41 bloklu tek yazım), insan notuna ekleme, ortaya ekleme, paragraf değiştirme, bölüm gövdesi değiştirme, başlık metni/düzeyi değiştirme, paragraf silme, tam üzerine yazma (3, biri canonical metnin bir kısmını aynen koruyarak), boşaltma, ajanın kendi notunu düzenleme, aynı baytları yeniden yazma, `canonical` paragrafları aynen kopyalayan yeni dosya, 5 hızlı ardışık düzenleme, 2 yeniden adlandırma (`mv`), dosya silme (`rm`), ajanın dosyasını eski insan yolunun üstüne taşıma. Her yazımdan sonra (> W) her belgenin `blame`'i:

- **Yeni/değişen blok 90; hepsi `layer: unverified`, `source: unattested_fs`; `canonical` sızıntısı 0.** `canonical` küme her adımda öncekinin alt kümesi (içerik değişmedi).
- Ajanın değiştirdiği/sildiği **20 `canonical` soyun 20'si**: `history`'de `meta.actor.kind = unattested_fs` olayı var; `read_block … at: before-agent` eski metni döndürüyor (0a′).
- Yeniden adlandırmada eski yolun blokları `canonical` kalıyor (sunucu silme almıyor), yeni yol tamamen `unverified`. İlk denetleyicim yalnız diskteki dosyalara baktığı için bunu kayıp sandı; denetleyici eski yolları da sorgulayacak şekilde düzeltildi ve adım yeniden değerlendirildi (ölçüm değil, benim aracımın hatası).
- Not: ajanın yazdığı dosyada daha önceki bir `canonical` sürümle **aynı metin** varsa (adım 30, eski yolda duran "Day two…") o soy `canonical` kalır: yeni ya da değişen içerik yoktur, sızıntı sayılmaz.

**Token'ı okuyan ajan** (`.rootmark/server.token` → `attest_session` → MAC → `attest`): **başarılı (200), bloklar `canonical`** — aşama 1'in bilinen sınırı (`status.attestation.stage = 1`); aşama 2 etkin değil, o yüzden "başarısız olmalı" maddesi uygulanamadı. Aynı yolla yanlış `content_hash`: tasdik 200 ile kaydediliyor ama eşleşmiyor, bloklar `unverified`; bozuk MAC: 403 `denied`; token'sız `attest_session`: 401.

## V3 — İnsan düzenleme (0b, 0c): gerçek Obsidian, kendi sürücüm

Ayrı Obsidian örneği (kendi `--user-data-dir`, CDP), eklenti kendi derlemem (`npm ci && npm run build`, `ce4cf04`), `[attestation] window_s = 0.5` (resmi W), 9 not (8'i birbirine wikilink'li). Harness'ten farklı olarak metin **CDP giriş olaylarıyla** (`Input.insertText`, Enter tuşu) yazıldı, editör API'siyle değil; yazım zamanı diskten (5 ms yoklama), katman HTTP `blame`'den (25 ms) ölçüldü. **50 düzenleme:** tek karakter 12 (5'i otomatik kayıt), paragraf 13 (5'i otomatik kayıt), odak kaybında kayıt 5, büyük yapıştırma 5 (7 blok), hızlı art arda 3 kayıt 4, iki dosya 4, Obsidian'dan yeniden adlandırma 7 (`alwaysUpdateLinks` açık, 2'si yeni klasöre).

- **Yeni blok 159, W + 1.5 s sonunda `unverified` 69 → 0.434 (eşik ≤ 0.01: kaldı).**
- **Yeniden adlandırma dışı 85 bloğun 0'ı `unverified`.** 69'un tamamı 7 yeniden adlandırmadan: Obsidian'ın bağlantı güncellemesi (yan notlar) tasdik edilmiyor; kendine bağlantı içeren ya da bağlantısı güncellenmiş notun yeniden adlandırılması da tasdik edilmiyor ve yeni yolun bütün blokları `unverified`. Tek istisna ilk yeniden adlandırma (`kitchen.md`): not kendisi `canonical` geçti (tasdikli `renamed_from`), yalnız `index.md`'deki bağlantı satırı `unverified`. FINAL'in 0b açıklaması (ASSUMPTIONS Z5-5) bağımsız olarak doğrulandı.
- **Gecikme p95 109 ms** (medyan 96, en çok 110; 35 tek kayıtlı düzenleme; hiçbir kayıt `canonical`'sız kalmadı). Ölçüm başlangıcım harness'inkinden farklı (kayıt eylemi döndükten sonra diskte görülen değişiklik; harness: sayfadaki `adapter.write` bitişi), bu yüzden mutlak değerler karşılaştırılamaz; ikisi de < 500 ms. Çok kayıtlı türler (hızlı, iki dosya) gecikmeye katılmadı.
- **Harness ile karşılaştırma:** harness'in D-B'sinde yeniden adlandırma dışında yanlış `unverified` yalnız 4 tek-karakter kaydındaydı (daha önce yeniden adlandırılmış notlara; `latency_unsettled` 4). Benim 21 düzenlemem daha önce yeniden adlandırılmış notlara yapıldı, hepsi `canonical` oldu; çünkü benim karakterlerim oturumun yazdığı (`canonical`) son paragrafa düştü, harness'inkiler `unverified` başlı bir paragrafa. Bunu ayrıca sınadım: `unverified` bir paragrafın tasdikli bir insan kaydıyla değiştirilmesi soyu `unverified` bırakıyor (ASSUMPTIONS Z2-1 ile tutarlı, aklama yok). Eklentinin durum çubuğu: `attested 57 · unattested 0`.

## V4 — Karışık (0d)

Aynı kurulum; ben Obsidian'da yazıp kaydederken bir ajan betiği vault'a doğrudan yazar; her yeni bloğun gerçek kaynağı metninden (`read_block`), katmanı `blame`'den.

- **20 tik** (10 başka not, 6 aynı not kayıttan 2.5 s sonra, 4 aynı not oturum başından 250–550 ms sonra): **40/40 doğru** (insan → `canonical`, ajan → `unverified`).
- **Sıkı çakışma, 16 tik** (ajan aynı nota kayıttan 0, 20, 50, 100 ms sonra yazar): 50 ve 100 ms'de 16/16 doğru; **0 ve 20 ms'de 8/16: insanın 8 paragrafı da `unverified`** (ajan blokları doğru). Toplam 64/72 = 0.889.
- **Hiçbir koşuda ajan bloğu `canonical` olmadı (36/36 `unverified`).** FINAL'in 0d kalışı (ASSUMPTIONS Z5-6: iki yazım izleyicinin tek gözlemine düşerse tamamı `unverified`, güvenli taraf) gerçek Obsidian'da da var; D-C'nin insanı ölçüm kodudur, bende gerçek eklentidir. Eşik doğruluğu tik karışımına bağlı olduğu için benim oranım FINAL'inkiyle karşılaştırılamaz; nitel sonuç aynıdır.

## V5 — Anahtar zinciri (0e)

- `scripts/keychain_probe.sh` (`TIMEOUT_S=15`): **"Obsidian Safe Storage" öğesi bu Mac'te yok** (`SecKeychainSearchCopyNext: The specified item could not be found`; `-a Obsidian`, `-s Obsidian` gibi adlarla da yok). Obsidian bu makinede `safeStorage`'ı hiç kullanmamış. Öğeyi oluşturmak (kendi Obsidian örneğimde `safeStorage.encryptString`) kullanıcının giriş anahtar zincirinde kalıcı bir değişiklik olacağı için yapmadım. **H-P2-0e: ölçülmedi.**
- **Yerine (bilgi, D-D değil):** scratchpad'de geçici bir anahtar zinciri (arama listesi değişmedi, sonunda silindi), içinde ACL'si yalnız `/Applications/Obsidian.app` olan bir öğe ve denetim olarak her uygulamaya açık bir öğe. Denetim öğesi terminalden okundu (65 bayt). ACL'li öğe: terminal `security find-generic-password -w` **5/5 okunamadı**, her deneme erişim isteminde 20 s bekleyip zaman aşımına uğradı; kendi Python'um (`Security.framework` `SecKeychainFindGenericPassword`, ctypes) **3/3 okunamadı** (istemde bloke, 20 s'de sonlandırıldı). Giz hiçbir zaman yazdırılmadı.
- **Aşama 2'yi aşan yol (gösterildi):** terminalden Obsidian'ı ayrı bir veri dizini ve `--remote-debugging-port` ile başlatıp CDP üzerinden `app.vault.adapter.write` ile yazılan ajan metni eklenti tarafından tasdik edildi: **3/3 `canonical`**, durum çubuğu `attested 3 · unattested 0`; süreç ne token'ı ne anahtar zincirini okudu. ACL Obsidian.app'e güvendiği için, gizi anahtar zincirine taşımak bu yolu kapatmaz. FINAL bu yolu "açık sorun" diye listeliyor; artık ölçülmüş bir boşluktur.

## V6 — Eski vault'lar (0f)

`git archive v0.1.0` (`85be0a7`) → kendi `cargo build --release` (`rootmark 0.1.0`). Kendi 3 vault'um (FINAL'in commit'lenmiş D-E vault'larından bağımsız): **scene** (insan notları, ajan `propose`, bir terfi, iki adlı checkpoint; 3 belge, 8 blok), **e2e** (üç ardışık insan alımı, reddedilen ajan eklemesi, adsız checkpoint; 3 blok), **mixed** (5 insan notu + ajan önerileri, bir `prune`, bir `demote`; 6 belge, 24 blok). Her biri eski ikiliyle kaydedildi, kopyası yeni ikiliyle açıldı (CLI ve kısa bir `serve --http --watch`): **3/3** — `events_hash` eşit (35df84…, ee07fe…, 202ec1…), `events.jsonl` baytları eşit, her belgenin `replay`'i eşit, `blame` (soy, sürüm, katman, etkinlik) eşit, `unverified` 0.

## V7 — Uyumluluk (0h)

`rootmark conformance`: **level 3, 37/37, spec 2.0.0** (C2-11, C2-12, C3-08 dahil). Ayrı bir klonda `scripts/gen_fixtures_c2.sh` ve `scripts/gen_fixtures_c3.sh` → **`git status` temiz** (beklenenler yeniden üretilebilir). Sürüm: `v0.1.0..ce4cf04` spec farkı §1 ve §4.2'ye yalnız "Informative (2.0.0)" notları, §7/§7.1/§8/§9/§10'a normatif ekler; CRITERION'a göre Frozen bölüme bilgilendirici not açıklıktır, yeni katman/aktör değeri 1.0.x okuma kurallarıyla okunamadığı için kırıcıdır → **majör, 2.0.0 doğru**. §1–6'nın normatif metni değişmedi.

## V8 — Yönetişim sınırları

- MCP `tools/list`: `blame, checkpoint, checkpoints, history, promote, propose, read_block, search, status, trace` — **`attest` yok**; `attest` ve `attest_session` çağrıları `-32602 unknown tool`.
- Sahte tasdik: yanlış hash → kaydediliyor ama eşleşmiyor, bloklar `unverified`; bozuk MAC → 403; token'sız → 401. Günlük bozma: bir satırın `content_hash`'ini değiştirmek → açılışta "id does not match its fields" (çıkış 1); bir satırı ikinci kez eklemek → "recorded twice" (çıkış 1).
- `[attestation] unattested = "canonical"`: `status.attestation` = `{unattested: "canonical", window_s: 2.0, stage: 1, pending: 0}`; doğrudan ajan yazımı `canonical` (eski davranış), `status` bunu açıkça gösteriyor.

## V9 — Ayar/test ayrımı, dondurma, bağımlılık, karar kuralı

- **W yalnız ayar setinde:** `run_p2_0.py` `--phase tune` yalnız `tune` oturumlarını koşar (`test` bellekte üretilir ama kullanılmaz — Z5 gözden geçirmesi N1'de de not edilmiş); `--phase test` W'yi `tuning.json`'dan okur, yoksa durur. `make p2-0-checks` commit'lenmiş `Z5_tuning.json`'u kopyalar.
- **Git sırası:** `choose_window` kuralı ön-kayıt commit'inde `12fa90b` (15:05) → ayar sonucu `bb73eeb` (15:51; `Z5_tuning.json` tek commit, sonra hiç değişmedi) → zincir `dfe73ee` (15:52) → test sonuçları ilk kez `e7058e0`'da. Ayardan önce test sonucu içeren commit yok. Ayar: beş W'nin oranı eşit (0.1162), hepsi p95 < 500 → kural gereği en küçük W = 0.5.
- `scripts/check_frozen.sh`: OK (tablo `85be0a7`'den beri yalnızca-ekleme). `scripts/check_deps.sh`: OK. **NORTH_STAR `v0.1.0`'dan beri değişmedi.**
- **FINAL'in izlenebilirliği:** resmi RUN dosyalarından yeniden hesapladım: D-B 931/1890, p95 270 ms (208 ölçüm, 4 sonsuz, en yakın sıra), tür dağılımı FINAL'deki tabloyla aynı; D-A 0/525, 0a′ 200/200 (52 değişen soy); D-C 333/340; D-E 3/3; kasıtlı 9/9 + 2/2; `artifacts.sha256` 26/26 OK.
- **Karar kuralını GOAL'den türetme:** geçen 0a, 0a′, 0c, 0f, 0g, 0h; kalan 0b, 0d; 0e ölçülmedi; regress'te bir fark. (1) "hepsi geçti" dalı: hayır (0b, 0d). (2) "0a veya 0a′ kaldı": hayır. (3) "0b kaldı, 0a geçti": **koşul harfiyen sağlanıyor**, ama dalın gerekçesi ("W ayarı ya da istemci tasdik akışı") 0d'nin kalışını kapsamıyor; 0d'nin kalışı tasarımdan (bayt eşleşmesi) gelir, W ya da istemci akışı yamasıyla düzelmez. (4) "0e kaldı": 0e ölçülmedi, kalmadı. (5) "Tanımlanmamış kalış". 0b ve 0d birlikte kaldığı için GOAL'de bu birleşime yazılmış bir dal yok; **benim türetimim de (5)'tir**, FINAL ile aynı. İki okumanın ortak sonucu: `unverified` varsayılan olmaz, hipotez kanıtlanmadı, P2-A başlamaz; ayrıldıkları nokta NORTH_STAR'ın şimdi mi daraltılacağıdır — bu insan kararıdır (K-P2-0.6-T).

## V10 — Vaat sorusu

NORTH_STAR'ın vaadi: "Bir ajan bilgi tabanına yazdığında, her bloğun kim tarafından, ne zaman, hangi bloktan türeyerek yazıldığı bilinir; onaylanmamış hiçbir şey bilgi sayılmaz; her cevap hangi bloklara dayandığını söyler; dünkü hale blok düzeyinde dönülebilir — ve dosyalar sıradan markdown olarak kalır." Bu turdan sonra, dosya erişimli bir ajanla vaat **şu koşulda tutuyor:** ajan dosyalara doğrudan yazdığında (Write/Edit, `mv`, `rm` — bu oturumun 30 yazımında 90 blok, 0 sızıntı) ve `.rootmark/server.token`'ı okumadığı, Obsidian'ı kendi başlatıp sürmediği sürece, yazdığı her blok `unverified` olur, aramada varsayılan görünmez, `blame` onu `unattested_fs` diye gösterir ve üzerine yazdığı insan içeriği checkpoint'ten blok düzeyinde geri okunur; yani "onaylanmamış hiçbir şey bilgi sayılmaz" ve "dünkü hale dönülebilir" tutar — ama "kim tarafından" yalnız "tasdiksiz bir dosya yazımı" kadar bilinir, ajanın kimliği bilinmez. **Şu koşullarda tutmuyor:** (a) ajan token'ı okuyup HTTP'den tasdik alırsa (aşama 1, gösterildi: `canonical`); (b) ajan Obsidian'ı `--remote-debugging-port` ile başlatıp eklentinin insan kanalından yazarsa (gösterildi: 3/3 `canonical`; anahtar zinciri ACL'si bunu kapatmaz); (c) ajan `.obsidian/plugins/**`'a yazarak eklentiyi değiştirebilirse (ölçmedim; aynı sınıfta). Ayrıca vaat ters yönde de pratikte zayıflıyor: insan not yeniden adlandırdığında içeriğinin büyük kısmı `unverified`'a düşer (0b: benim koşumda 69/159) ve ajan insanın kaydından ~20 ms içinde aynı nota yazarsa insan paragrafı da `unverified` olur (0d) — bunlar güvenli taraftadır (sızıntı değil) ama varsayılan politika `unverified` olursa insan içeriğinin "bilgi sayılmaması" demektir. Bu yüzden bugün dürüst ifade: vaat, tasdikli istemci ile ve **ne token'a ne de insan istemcisini başlatma/değiştirme yetkisine sahip olmayan** bir ajana karşı tutar; politika varsayılanı ve spec §7.1 "Draft" kaldıkça yayımlanmış bir davranış değildir.

## İhlal / fark / şüphe

- **İhlal:** bulmadım (kırmızı çizgiler: W yalnız ayarda; MCP'de `attest` yok; eski vault'lar `canonical` ve `events_hash` aynı; spec §1–6 normatif değişmedi; NORTH_STAR değişmedi; ürün kodu md yazmıyor — V2/V3'te vault'taki tek md yazıcıları ben ve Obsidian'dım).
- **F1 (bilgi):** PR #11 birleşmemiş; istemdeki "main'e girdi" doğru değil (yukarıda).
- **F2 (beklenen fark):** D-B p95 270 (resmi) / 298 / 286 ms; `unverified` üç koşuda 931. Zamanlama sapması, eşik dalı aynı.
- **F3 (sunum):** `make verify` iki zincirde de "4 historical FAIL(s) not counted; counted FAILs 15" yazıyor; doğru sınıflandırma `p2_0/historical.json` (counted 0). FINAL açıklıyor, ama zincirin kendi özeti yanıltıcı.
- **F4 (gösterilmiş boşluk):** CDP yolu (V5) — FINAL'de açık sorun, burada ölçülmüş.
- **F5 (küçük):** yanlış `content_hash`'li tasdik 200 ile günlüğe yazılıyor (eşleşmediği için zararsız; günlük, dosyada hiç olmamış hash'lerle büyüyebilir).
- **F6 (karar kuralı):** "0b dalı" harfiyen de uyuyor (V9); FINAL'in seçimi savunulabilir ve benimkiyle aynı, ama tek doğru okuma değil.

## Öneriler (en çok 3)

1. **Yeniden adlandırma/bağlantı güncellemesi tasdiki için yama turu, aynı ön-kayıtla yeniden ölçüm.** Harness'te ve kendi bağımsız koşumda 0b'nin yeniden adlandırma dışı payı 0 (bende 0/85); kalışın tamamı Obsidian'ın `adapter.write` dışından yazdığı bağlantı güncellemeleri ve ardından gelen yeniden adlandırmalardan. Aklama korumasıyla (yalnız bağlantı metninin değiştiği bayt farkı) ve güvenlik incelemesiyle; 0d'yi (çakışma) ayrı bir hipotez olarak ele alın ya da belgelenmiş sınır sayın.
2. **Tehdit modeline gösterilmiş yolu yazın ve aşama 2'yi buna göre yeniden tanımlayın:** "İnsan kanalı, Obsidian sürecinden geçen yazımdır; bu süreci başlatabilen ya da hata ayıklayıcıyla sürebilen (`--remote-debugging-port`, CDP) veya eklenti dosyalarını yazabilen her yerel süreç insan kanalındadır. Aşama 1'de token'ı okuyabilen süreç de öyledir. Anahtar zinciri ACL'si (aşama 2) uygulamaya güvenir, sürene değil; bu yolları kapatmaz." Aşama 2'nin hedefi ya OS kullanıcı ayrımı/kum havuzu (GOAL'ün kapsam dışı bıraktığı) olmalı ya da eklentinin hata ayıklayıcı bağlıyken tasdik etmeyi reddetmesi gibi ölçülebilir bir alt-hipotez.
3. **`verify.sh`'in FAIL özetini turun sınıflandırıcısına bağlayın** (`historical_p2_0` ya da onun genel hali; dondurma istisna prosedürüyle, sonraki turda): `make verify` çıktısı "counted FAILs 15" yerine FINAL'in sayımını (0) göstersin, ki bağımsız bir okuyucu iki kaynağı uzlaştırmak zorunda kalmasın.

## Tehdit modeli için önerilen ifade

> **Aşama 1 (etkin).** rootmark, vault dosyalarına doğrudan yazan bir süreci insan saymaz: tasdiksiz her dış değişiklik `unverified` alınır, aktörü `unattested_fs`'dir, aramada varsayılan görünmez ve insan terfi ettirene kadar bilgi sayılmaz. Tasdik, insan kanalından gelen bir bayt beyanıdır; insan kanalı, `.rootmark/server.token`'ı okuyabilen her süreç ile insan istemcisini (Obsidian) başlatabilen, hata ayıklayıcıyla (`--remote-debugging-port`/CDP) sürebilen ya da eklenti dosyalarını değiştirebilen her süreci kapsar. Bu nedenle aşama 1 dikkatsiz ya da talimata uyan bir ajanı canonical'dan ayırır; kendi kullanıcısının yetkileriyle çalışan kasıtlı bir ajanı ayırmaz. Anahtar zinciri ACL'si (aşama 2) bu sınırı değiştirmez; kasıtlı ajana karşı ayrım ancak OS düzeyinde bir ayrımla (ayrı kullanıcı, kum havuzu) sağlanır.
