# P2-Q.1 — Tasarım notu (R0; R1/R2'nin kodundan önce)

Kapsam: yalnız `rootmark serve --watch`'ın izleyicisi (`server/src/serve.rs`) ve onun alım yolu (`server/src/tools.rs`: `Kernel::watch_doc`'un yanına kaldırma alımı ve izleyicinin belge listesi). `core`, `index`, `bindings`, istemcilerin kaynakları, spec ve fixture'lar değişmez. Yeni olay türü, yeni alan, yeni biçim yok.

## 0. Ön sınama (tasarımın dayanağı, kod yazılmadan)
0.3.0 ikilisiyle (`target/release/rootmark`, `ROUND_START`) bir karalama vault'unda: izleyici açıkken bir not yazıldı, sonra **boşaltıldı**. Sonuç: izleyici boş içeriği bugün de alıyor — günlükte notun her soyu için `BlockDeleted` (`meta.actor = {kind: human, id: unidentified}`, `meta.source = "rootmark-serve-watch"`), ardından blok içermeyen bir `SnapshotCommitted`; `search` 0 sonuç, `blame` boş. Yani "belgenin hiç bloğu kalmadı" durumu çekirdekte ve indekste spec'in mevcut biçimiyle zaten var; eksik olan yalnız izleyicinin **kaldırılan** bir dosyayı bu duruma eşlemesi. Bu yüzden çekirdeğe ve indekse dokunmadan çözülür.

## 1. Olay biçimi (spec atıflarıyla)
| Durum | Üretilen olaylar | Dayanak |
|---|---|---|
| Kaldırılan md dosyası (silme, vault dışına taşıma) | Belgenin gözlenen yeni hali **blok içermeyen** belgedir: eşleştirmenin kalanı (eski blokların hepsi) `BlockDeleted`, sonra `SnapshotCommitted` (bloksuz anlık görüntü). Yeni olay türü yok. | §5 adım 7 ("unmatched old → `BlockDeleted`"); §7.1 "External change: a change of a document's file … observed on disk" — dosyanın kaldırılması dosyanın bir değişikliğidir ve gözlenen içeriği boştur. |
| Aktör ve kaynak, `unattested = "canonical"` ya da tasdikli | `BlockDeleted` `meta.actor = {kind: human, id: <vault insanı>}`, `meta.source = "rootmark-serve-watch"` (izleyicinin bugünkü değişiklik olaylarıyla aynı) | §7 "Provenance of events (1.0.3)": `BlockDeleted` yönetilen bir yazma yolundan aktör + kaynak taşır; §7.1 "Under `canonical` every external change is ingested as in 1.0.3". |
| Aktör, `unattested = "unverified"` ve tasdiksiz | `meta.actor = {kind: unattested_fs, id: fs}`, `meta.source = "unattested_fs"` | §7.1 "its change and delete events carrying `meta.actor = {kind: unattested_fs, id: fs}` …; a lineage it deletes is deleted with its actor". Katman veren olay yoktur (yalnız silme), bu yüzden `meta.layer` yazılmaz (§7.1 Reading). |
| Vault içinde tasdiksiz taşıma (`a.md → b.md`) | Eski yol: yukarıdaki kaldırma; yeni yol: bugünkü dış değişiklik alımı (`BlockCreated`, politika gereği `canonical` ya da `unverified`). Belgeler arası soy yoktur. | §5 belge başınadır (girdi: aynı belgenin önceki anlık görüntüsü); §7.1 "External change". Soy belgeler arasında taşınmaz: bugün de taşınmıyor, uydurulmaz. |
| Tasdikli yeniden adlandırma (§7.1 "Rename", `renamed_from`) | Yeni yol bugünkü gibi taşınan katmanlarla bir kez alınır (attested ingest, caps); eski yolun kaldırılması **ayrıca ve bir kez** yukarıdaki kaldırma olarak alınır. Tasdikin kendisi eski yol için olay yazmaz (bugünkü davranış: `check_rename` yalnız denetler). | §7.1 "Rename": kontrol "the head snapshot of the old path" üzerinde yapılır — bu yüzden eski yolun kaldırılması tasdik penceresi kapanmadan alınmaz (§2). |
| Boş dosya ile kaldırılmış dosya | Günlükte aynı görünür (bloksuz anlık görüntü). | §7.1 (2.0.3) "a removal is not an event" — bu tasarımda da kaldırmanın kendine ait bir olayı yoktur; yalnız bloklarının silinmesi kaydedilir. |

**Spec'in tanımlamadığı bir durum bulunmadı:** kaldırma için yeni bir olay ya da alan gerekmiyor; §5'in kalan kuralı ve §7/§7.1'in aktör kuralları yetiyor. Bu yüzden karar talebi yazılmadı (Q1-1: uydurma 0). GOAL'ün örneklediği "kaldırılan bir belgenin alımının olay biçimi" sorusunun cevabı yukarıdaki ilk satırdır: spec'in mevcut biçimi.

## 2. PQ-18: silme ve taşıma — yöntem
1. **Olayların hedefe çevrilmesi.** Her olay yolu (erişim olayları hariç) vault'a göreli `/` yola çevrilir; kaldırılmış bir yolun `canonicalize`'ı başarısız olduğundan, var olan en uzun ata `canonicalize` edilir ve geri kalan bileşenler eklenir (macOS `/var` → `/private/var`). Gizli bileşen (`.` ile başlayan: `.rootmark/`, `.obsidian/`) ya da vault dışı → yok sayılır (bugünkü kural, `watched_doc`). Sonra:
   - `.md` yolu → **belge** hedefi (var olsa da olmasa da);
   - var olan bir dizin, olay türü `Create` ya da `Modify(Name)` → **tarama** hedefi (PQ-22, §3);
   - var olmayan, `.md` olmayan yol (kaldırılan ya da dışarı taşınan bir dizin olabilir) → **önek** hedefi.
   Bu üç hedef de toplu işi (50 ms sessizlik, en çok 250 ms) uzatır; hiçbir şey adlandırmayan olaylar bugünkü gibi düşer.
2. **Toplu işin işlenmesi** (bayt sırasıyla): belge yolunda düzenli dosya varsa `watch_doc` (bugünkü gibi); yolda **hiçbir şey yoksa** (`symlink_metadata` hata) kaldırma **beklemeye** alınır; yolda dosya olmayan başka bir şey (dizin, sembolik bağ) varsa bugünkü gibi günlüğe yazılır, alınmaz. Önek hedefi: günlüğün bildiği ve bloğu olan belgelerden yolu `önek/` ile başlayanlar (izleyicinin kuralıyla gizli olmayanlar) beklemeye alınır.
3. **Bekleme ve kaldırma alımı.** Beklemedeki kaldırma, gözleminden `W + 500 ms` sonra (`W` = vault politikasının `[attestation]` penceresi, varsayılan 2 s) işlenir: yolda yine hiçbir şey yoksa `Kernel::watch_removed` belgeyi **boş içerikle** dış değişiklik olarak alır (aynı `external_plan`: tasdik kararı, aktör, katman kuralları, sıcak katman — §1); bu arada yol yeniden düzenli dosya olduysa `watch_doc` (değişiklik olarak; içerik eşitse olay yok). Aynı anda dolan kaldırmalar bayt sırasıyla. Servis dururken (SIGTERM/SIGINT, MCP akışının sonu) bekleyen kaldırmalar beklemeden işlenir, son sıkıştırmadan önce (izleyici iş parçacığı birleştirilir) — sessiz kayıp yok.
4. **Beklemenin nedeni (çift işlem yok).** §7.1'in "Rename" ve "Link update" kontrolleri eski yolun **başındaki** anlık görüntüye bakar; tasdik değişiklikten en çok `W` sonra gelebilir (late attestation). Eski yolun kaldırılması hemen alınsaydı, geç gelen tasdikli bir yeniden adlandırma boş başla karşılaştırılır ve reddedilirdi (Obsidian'da yeniden adlandırma tasdiki dosya işleminden sonra gelir). Bekleme `W`'yi aştığı için yeni yolun alımı ve (zamanında ya da geç) tasdik kararı her zaman eski yolun kaldırılmasından önce olur; eski yol bir kez (kaldırma), yeni yol bir kez (tasdikli alım) işlenir. Pencereden (`W + 500 ms`) sonra gelen bir yeniden adlandırma tasdiki artık eski yolun boş başıyla karşılaşır ve §7.1 gereği reddedilir (403, gerekçeli) — bu pencere dışı bir tasdiktir ve zaten hiçbir değişikliği eşleyemez ya da yükseltemezdi (§7.1 Window); sınır olarak belgelenir.
5. **Duvar saati** yalnız *ne zaman* işleneceğine karar verir; hiçbir günlüğe ya da kimliğe girmez (§7.1 Window kuralı).

## 3. PQ-22: Linux'ta yeni dizin — yöntem
`notify` 8.2.0'ın inotify arka ucu özyinelemeli izlemede yeni dizine izlemeyi ancak dizinin `IN_CREATE` olayını okuduktan sonra ekler; bu arada dizine yazılan dosyanın olayı hiç üretilmez (P2-Q CI'da iki işlemcide aynı dosya). Düzeltme: **tarama** hedefi (yeni oluşturulan ya da vault'a taşınan dizin) toplu iş işlenirken dizin özyinelemeli taranır ve bulunan her md dosyası belge hedefi olur. Kural `rootmarkcore::files::markdown_files` ve `watched_doc` ile aynıdır (gizli bileşen atlanır); sembolik bağlar izlenmez ve atlandığı günlüğe yazılır (döngü yok; `paths::doc_file` zaten reddeder). Bir dosya hem taramayla hem kendi olayıyla gelirse ikinci alım no-op'tur: `external_plan` önce içeriği başla karşılaştırır, eşitse olay yazmaz ve tasdik kararı vermez (tasdik tüketilmez). macOS (FSEvents) bugün de dosya olaylarını veriyor; tarama orada aynı dosyaları bulur, sonuç değişmez.

## 4. Sınır durumlar
- **Silinip hemen yeniden yaratılan dosya** (bekleme içinde): değişiklik olarak alınır (§5 eski başa karşı eşleşir; içerik eşitse olay yok) — soy korunur. Bekleme dolduktan sonra yeniden yaratılırsa: önce kaldırma (`BlockDeleted`), sonra yeni alım (`BlockCreated`).
- **Atomik kaydetme** (geçici dosya + üstüne `rename`): hedef yol hep var; kaldırma doğmaz.
- **Dizin silme** (`rm -r`): dosya başına kaldırma olayları ve/veya dizin olayı (önek) → altındaki her belge bir kez (küme).
- **Dizin taşıma** vault içinde: eski önek → kaldırmalar (beklemeli), yeni dizin → tarama → yeni yolların alımı. Vault dışına: kaldırmalar. Dışarıdan içeri: tarama.
- **`.rootmark/` ve gizli dizinler:** hem taramada hem önekte dışarıda (PQ-13 davranışı değişmez, yalnız belgelenir).
- **Sembolik bağlar:** taramada izlenmez (günlükte not); bir belge yolunun yerine sembolik bağ gelirse "kaldırma" sayılmaz (yolda bir şey var), bugünkü hata satırıyla günlüğe yazılır.
- **Vault kökünün kendisi** hiçbir hedef üretmez (tarama da önek de yok).
- **Bilinmeyen belgenin kaldırılması** (günlükte yok ya da zaten bloksuz): kaldırma alımı no-op'tur (olay yok).

## 5. Politikanın rolü
- `append_only` ajan kanalının (MCP `propose`) düzenleme kancasıdır; izleyici bir dış değişikliktir ve ondan geçmez (bugün de dosyayı boşaltmak `canonical` blokları siler). Kaldırma da aynı yoldadır.
- `unattested = "canonical"` (varsayılan): kaldırma insan aktörüyle (`human`) alınır — bugün bir dosyayı boşaltmakla aynı.
- `unattested = "unverified"`: tasdiksiz kaldırma `unattested_fs` aktörüyle silinir (§7.1, yukarıda). Sonuç: dosya erişimli bir ajan `canonical` blokları bir dosyayı silerek kaldırabilir — bunu bugün dosyayı boşaltarak da yapabiliyordu; silme günlükte aktörüyle kayıtlıdır ve içerik kontrol noktasından okunur (§8). Bu tur bunu değiştirmez; kılavuzda yazılır. (Bir kaldırmanın tasdiki, boş baytların `content_hash`'iyle, spec'in genel kuralıyla mümkündür; Obsidian eklentisi silmeyi tasdik etmez — ürün davranışı aynı kalır.)
- `[promotion.rules]` kaldırmayla ilgisizdir (terfi yok).

## 6. Determinizm ve geriye uyum
- Yeni olay türü, alan, biçim ya da sürüm yok; eski günlükler aynen okunur, `events_hash`'leri değişmez, göç gerekmez; `make conformance` 39/39 beklenir.
- Aynı dosya sistemi olay dizisi aynı günlüğü üretir: toplu işler ve beklemedeki kaldırmalar bayt sırasıyla; kaldırma alımı bugünkü dış değişiklik alımının aynısıdır (boş içerikle). Zamanlama yalnız toplu işin sınırını ve kaldırmanın ne zaman alınacağını belirler (bugünkü toplu işleme gibi).
- PQ-22'nin düzeltmesiyle Linux'ta günlük macOS'takiyle aynı belgeleri içerir (P2-Q'de fark yalnız kaçan dosyadandı).

## 7. Testler
Ön-kayıtlı liste ve beklenen olaylar: `TESTS.md`. Yeni dosyalar `server/tests/watch_newdir.rs` (R1) ve `server/tests/watch_remove.rs` (R2); dondurulmuş `serve_watch.rs` değişmez. CI'da `make test` macOS ve `ubuntu-latest` işlerinde koşar (`ci.yml`), yani ikisinde de.

## 8. Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtlar (önceden, adıyla)
Kademe 2'nin hükmü A↔B'dir (aynı commit'in iki zinciri); aşağıdakiler **önceki koşulara göre** değeri değişebilecek kayıtlardır (izleyici açıkken dosya taşıyan ölçüm adımları; dosya silen dondurulmuş bir adım yok). Liste dışında bir değişiklik adsız farktır.
- **Kademe 2 (D):** `p2_0_2/tamper_p2_0_2.json` — `serve --watch` altında üç yeniden adlandırma (`roasting→kiln`, `selfnote→self2`, `fake→fake2`); vakaların sonucu (`pass`, kabul/ret, durum kodları, yeni yolların katmanları) **aynı** beklenir (tasdikler pencere içinde gelir); eski yolların kaldırılması kayda girmez (kayıtta günlük baytı yok). Bir alanı değişirse adıyla raporlanır.
- **Kademe 2 (Z, karşılaştırma dışı):** yok beklenir — D-A, D-A2, D-C kümelerinin adımları yalnız yazımdır (taşıma yok).
- **Mac kümesi (O, gerçek Obsidian):** D-A′ (`H-P2-0.2e`, O10) ve D-B4 (`H-P2-0.2b`, `b′`, `c`; O7–O9), P2-0.3'ün 0.3b/0.3b-yük/0.3c/0.3d/0.3e (`p2_0_3_checks.json`) — yeniden adlandırma oturumları var: eski yolların blokları artık `BlockDeleted` alır (günlük uzar), pencere dışı yeniden adlandırma tasdikleri 403 olabilir (`attest` hata sayıları). Yeni yolların katman ölçümleri (yanlış `unverified` oranı) değişmez beklenir; değerler adıyla raporlanır. `p1e1` gerçek Obsidian sahnesi (E1-3, E1-9, tamper e2e): yeniden adlandırma yok, değişmez beklenir.
- **Mac kümesi (Z):** `Z-D10` (`serve_watch` D-10, SIGTERM): kaldırma yok, değişmez beklenir; `Z-DA` (D-A, D-A2, `logs_vs_p2a`): taşıma yok, değişmez beklenir.
- **Kademe 1 (FAST):** izleyici koşmaz; değişmez.
