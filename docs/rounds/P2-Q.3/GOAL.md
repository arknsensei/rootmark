# P2-Q.3 — Yama Turu: Kapalıyken ve Öldürülünce Kaybolan Değişiklikler, 0.3.3

Başlangıç: D-0 karar kuralına göre D0-3'te kaldı (marka taraması ürün sahibinin kararıyla ertelendi); öteki kriterler geçti. Bağımsız doğrulamanın hükmü **"güvenilir"** (`reports/rounds/D-0/VERIFICATION.md`, PR #62). K-D0.4 kararları verir: D0-K10 (a) gereği D-1'den önce bu kısa yama turu (0.3.3) açılır. Sıra: P2-Q.3 → D-1 → P2-O, P2-Q.S, P2-C/P2-H.

Sorun:
- **D0F-1 / PQ2-1 — SIGKILL bekleyen kaldırmaları kaybettiriyor ve bu kalıcı (ürün hatası, izleyici).** `serve --watch` bir silmeyi ya da tasdiksiz taşımayı `window_s` + 0,5 s bekletir (PQ1-4'ün kaldırma beklemesi). Sunucu bu sürede SIGKILL ile ölürse kaldırma hiç alınmaz. Notun blokları yeniden başlatmada da, `ingest-dir` ile de kapanmaz; arama onları döndürmeyi sürdürür. Ölçüm: SIGKILL 20/20 kalıcı (macOS ve Linux), SIGTERM kontrolü 20/20 alındı; doğrulayıcı 0.3.2'de SIGKILL 12/12 kalıcı, SIGTERM 6/6 alındı. Claude Code oturum sonunda sunucusunu SIGINT → SIGTERM → ~0,5 s → SIGKILL ile kapatıyor; imajın varsayılan kipi (MCP stdio) bu hatayı her oturumda üretebilir. Kanıt: `reports/rounds/D-0/FINDINGS.md` D0F-1, `reports/rounds/D-0/pq2_1/`.
- **D0F-5 — sunucu kapalıyken yapılan değişiklikler başlangıçta alınmıyor (ürün hatası, yeni).** `serve --watch` başlarken vault'u günlükle uzlaştırmıyor: `run` → `start_watcher` yalnız yeni olayları alıyor, başlangıç taraması yok. Sunucu dururken düzenlenen, silinen ya da eklenen not yeniden başlatmada günlüğe girmiyor (0/3, her platformda). `ingest-dir` düzenleme ve eklemeyi alıyor (3/3), silmeyi almıyor (0/3). Dosyası olmayan dokümanın canlı bloklarını kapatan tek yol izleyicinin `watch_removed`'ı (`server/src/tools.rs`). D0F-1 bunun özel hali. Sonucu: NORTH_STAR'ın "dosyalar gerçek kaynaktır" sözü sunucu kapalıyken yapılan değişikliklerde tutmuyor. Kanıt: D0F-5, `reports/rounds/D-0/pq2_1/offline_{macos,linux}.json`.
- **D0F-7 — kılavuz eksik (belge).** Known issues PQ2-1'i "to be fixed, P2-C" diye anıyor; kalıcı olduğu ve sunucu kapalıyken yapılan değişikliklerin alınmadığı yazılı değil.
- **macOS'ta `check_deps` bir kez düştü (süreç, nedeni bilinmiyor).** K-D0.3 PR'ında (#61, koşu `43241761070`) macOS `test + lint + conformance`'ın `make lint` adımında: "clients/ names /v1/attest_session, not an endpoint of docs/CLIENT_CONTRACT.md". Aynı ağaç yerelde, Linux'ta ve yeniden koşuda geçti; `CLIENT_CONTRACT.md` ve istemciler değişmemişti. Sessiz başarısızlığın tersi (yanlış kırmızı), ama nedeni bilinmeyen bir kapı güvenilmez kapıdır.

## İlkeler
- **Ürün değişikliği yalnız sunucuda: başlangıç uzlaştırması, izleyici, `ingest-dir`.** Değişebilecekler: `server/src/serve.rs`, `server/src/tools.rs` (izleme alımı, `ingest-dir`, `watch_removed`), gerekirse `server/src/main.rs`'in CLI bayrakları ve yeni testler. Değişmeyenler: `core`, `index`, `bindings`, `clients/*/src`, spec §1–6 ve uyumluluk fixture'ları; MCP ve HTTP sözleşmesi (`docs/CLIENT_CONTRACT.md`) — yeni uç, alan ya da araç yok. Düzeltme spec'in tanımlamadığı bir olay türü ya da alan gerektirirse: PARK + karar talebi (uydurma yok).
- **Uzlaştırmanın anlamı spec §7.1'in dış değişiklik kuralıdır.** Sunucu kapalıyken yapılan bir değişiklik, izleyicinin gördüğü tasdiksiz dış değişiklikle aynı sonucu verir: `unattested = "unverified"` altında `unverified` / aktör `unattested_fs`, `canonical` politikasında `canonical`. Başlangıçta tasdik penceresi yoktur (geçmişe dönük tasdik yok); bu `DESIGN.md`'de açıkça yazılır. Eksik dosya → kaldırma (`BlockDeleted`), farklı bayt → dış değişiklik, yeni dosya → ekleme. Taşımanın (eksik + yeni, aynı bayt) nasıl okunacağı izleyicinin bugünkü kuralıyla aynı olur ve `DESIGN.md`'de yazılır.
- **Determinizm.** Uzlaştırma belirlenimli sırayla (ör. yolun bayt sırası) yürür; aynı disk durumu ve aynı günlük her makinede aynı olayları üretir; kimliklere zaman damgası ya da rastgelelik girmez. Değişiklik yoksa uzlaştırma hiçbir olay yazmaz (boş açılış bayt bayt aynı günlük).
- **Kapsam sınırı.** `ingest-dir` yalnız verilen kökün altındaki dokümanları kaldırabilir; kökün dışındaki dokümanlara dokunmaz. `.rootmark/` ve izleyicinin bugün yok saydığı yollar uzlaştırmada da yok sayılır.
- **Tasarım notu koddan önce.** R0'da `docs/rounds/P2-Q.3/progress/DESIGN.md`: uzlaştırmanın algoritması ve izleyiciyle sırası (başlangıç taraması ile ilk olayların yarışı dahil: macOS FSEvents'in başlatmaya çok yakın olayı yeni izleyiciye bazen iletmesi, D0F-6), PQ2-1'in bu düzeltmeyle nasıl kapandığı (bekleyen kaldırma kalıcı değil; bir sonraki başlangıç onu eksik dosya olarak alır — ya da gerekçeli başka yöntem), `ingest-dir`'in silme kuralı ve kapsamı, ön-kayıtlı testler. Bunlar commit'lenir, sonra kod yazılır.
- **Testler ürüne.** D-0'ın sondaları (`research/d0/pq2_1_probe.py`, `offline_probe.py`) Rust entegrasyon testlerine (`server/tests/` altında **yeni** dosyalar) çevrilir; ürün deposuna aktarımla gider (D-0 doğrulaması, Öneri 3). Dondurulmuş testler değişmez.
- **Kayıt farklarının önceden adlandırılması (K-P2Q1.4).** R0'ın "değişmesi beklenen kayıtlar" listesine sürüm dizgisi alanları (0.3.3), E1-4/R14 (etiket) ve `.github/workflows/**`'ta yapılacak her değişiklik o değişiklikten önce adıyla yazılır. D-NOTES senaryosunda değişmesi beklenen bir kayıt varsa (uzlaştırmanın yeni olayları) o da adıyla ve gerekçesiyle.
- Sessiz başarısızlık yok; markdown'a kimlik/meta veri yazılmaz. Ön-kayıt anahtarları `~/.config/rootmark-lab/` altında (DEV_PROCESS).

## Kapsam
1. **R0 — Açılış ve tasarım.**
   - `ROUND_START`; DECISIONS K-P2Q3.1 (açılış).
   - `DESIGN.md` (yukarıda) ve `TESTS.md` (ön-kayıtlı testler, beklenen sonuçlar).
   - Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi.
   - **Hataların 0.3.2'de yeniden üretimi:** yayımlanmış `v0.3.2` ile D-0'ın sondaları (SIGKILL sil/taşı; sunucu kapalıyken düzenle/sil/ekle; `ingest-dir` silme), macOS ve Linux (Docker), kısa tekrarla; kayıt `reports/rounds/P2-Q.3/r0/`.
2. **R1 — Başlangıç uzlaştırması (D0F-5, D0F-1).** `serve --watch` başlarken, izleyici olayları alınmadan önce ya da onlarla tutarlı biçimde, vault'taki `*.md` dosyaları ile günlüğün canlı dokümanları uzlaştırılır (İlkeler). Yeni entegrasyon testleri, macOS ve Linux'ta, her durum ≥ 20 tekrar:
   - (a) SIGKILL, kaldırma beklemesi içinde: silme ve taşıma → yeniden başlatmadan sonra eski yolun blokları kapalı, aramada yok;
   - (b) SIGTERM ~0,5 s sonra (bugün geçiyor; düzeltme bozmamalı);
   - (c) gerçek istemci dizisi SIGINT → SIGTERM → ~0,5 s → SIGKILL;
   - (d) sunucu kapalıyken düzenle, sil, ekle (ayrı ayrı ve birlikte) → yeniden başlatmadan sonra günlük ve arama diskle aynı;
   - (e) değişiklik yokken yeniden başlatma → yeni olay 0, günlük bayt eşit;
   - (f) `unverified` ve `canonical` politikalarında katman ve aktör §7.1'deki gibi.
   `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs` aynen geçer.
3. **R2 — `ingest-dir` silmeleri.** Sunucu kapalıyken `ingest-dir <kök>`, kökün altında dosyası olmayan canlı dokümanların bloklarını kapatır (aynı §7.1 kuralı); kökün dışına dokunmaz. Yeni test: silme alınır; alt dizin kökü verildiğinde dışarıdaki dokümanlar aynen kalır; değişiklik yokken yeni olay 0.
4. **R3 — `check_deps`'in macOS düşüşü.** Kök neden aranır: betiğin `clients/` ve `CLIENT_CONTRACT.md` okuması, `make lint`'in öteki hedeflerle sırası (ör. `make test`'in bıraktığı ya da değiştirdiği dosyalar, `npm ci` sırasında yazılan dosyalar), BSD/GNU araç farkları. Yeniden üretilirse düzeltilir (`scripts/check_deps.sh` dondurulmuş değildir; `test_check_deps.sh` aynen geçer, gerekirse yeni bir sınama eklenir). Yeniden üretilemezse: denenen hipotezler ve sonuçları `progress/R3_check_deps.md`'ye yazılır ve betik, düştüğünde nedeni ayırt edecek tanı çıktısı verecek biçimde güçlendirilir (hangi dosya, hangi uç listesi); düşüş açık bulgu olarak kalır.
5. **R4 — Belgeler, güvenlik ve 0.3.3.**
   - (a) Kılavuz: Known issues'tan PQ2-1 çıkar; "dosyalar gerçek kaynaktır" sözünün sunucu kapalıyken yapılan değişikliklerde de tuttuğu (uzlaştırmanın davranışı, tasdik penceresinin başlangıçta olmadığı, `unverified` altındaki sonucu) ve `ingest-dir`'in silmeleri aldığı yazılır (D0F-7).
   - (b) `CHANGELOG.md` `## 0.3.3`.
   - (c) Açık Dependabot ve CodeQL uyarıları 0 ya da her biri yazılı gerekçeli (`progress/R4_security.md`).
   - (d) **Kademe 2** bir tetikleme; **Mac kümesi** bir kez (izleyiciye dokunuldu, DEV_PROCESS §3; önce `fseventsd` sağlığı).
   - (e) **0.3.3 yayımı:** `release/0.3.3` dalı, R1–R20 (`release_check`), PR; insan adımları `HUMAN_STEP.md`'de (birleşme, `release.yml`, `v0.3.3` etiketi, GitHub sürümü). `SHA256SUMS.local` P2-Q.2 doğrulamasının F1'i bilinerek: yerel derlemenin sağlamaları yayımlanmış arşivlerinkiyle karışmayacak biçimde adlandırılır ya da hiç eklenmez (gerekçe `HUMAN_STEP.md`'de).
   - (f) **Yayımlanmış 0.3.3 ile:** R1'in (a), (c), (d) durumları macOS ve Linux'ta kısa tekrarla; gerçek Claude Code başsız oturumunun son saniyelerinde silinen bir not (yalnız D-NOTES kopyasında) → bir sonraki oturumda aramada yok; P2-Q'nün D-NOTES senaryosu Mac, Docker ve CI'da (`pilot.yml`, bir koşu): `events_hash`, vektör dosyaları ve indeks manifest'i 0.3.2'nin kayıtlarıyla bayt eşit (ya da yalnız R0'da adıyla beklenen farklar). Kayıtlar `reports/rounds/P2-Q.3/` altına.
6. **R5 — Rapor, gözden geçirici, hazır.**
   - FINAL; `FINDINGS.md` (D0F-1, D0F-5, D0F-7 kapandı mı; `check_deps` düşüşü; yeni bulgular sınıflı ve hedefli).
   - Gözden geçirici tek tur. Başlık: "Uzlaştırma spec §7.1'e uyuyor mu (tasdik, katman, aktör); belirlenimli mi; değişiklik yokken sessiz mi; `ingest-dir` kökün dışına dokunuyor mu; testler gerçek SIGKILL'i mi sınıyor; ürün sınırı".
   - READY; tek PR; birleşmeden sonra temizlik (K-P2V2.2).

## Önceden kayıtlı çıkış kriterleri
Senaryo ve test adımları için "koştu" ile "geçti" ayrı sayılır. Kriter, koşan **her** adımın geçmesini ister.

| Kod | Ölçüm | Eşik |
|---|---|---|
| Q3-1 | `DESIGN.md` ve `TESTS.md`, R1'in ilk kod commit'inden önce commit'li. Hatalar 0.3.2'de yeniden üretilmiş (SIGKILL kalıcı; kapalıyken düzenle/sil/ekle alınmıyor; `ingest-dir` silmeyi almıyor), iki platformda kayıtlı. | önce; kayıtlı |
| Q3-2 | **D0F-1 ve D0F-5:** R1'in (a)–(f) testleri macOS ve Linux'ta her durum ≥ 20 tekrar geçti; (e)'de yeni olay 0 ve günlük bayt eşit; (f)'de katman ve aktör §7.1'deki gibi. `watch_remove.rs`, `watch_newdir.rs`, `watch_hold.rs`, `serve_watch.rs` aynen geçer. | hepsi |
| Q3-3 | **`ingest-dir`:** silme alınır; alt dizin kökünde dışarıdaki dokümanlar aynen; değişiklik yokken yeni olay 0. | hepsi |
| Q3-4 | **Sözleşme ve geriye uyum:** `docs/CLIENT_CONTRACT.md` değişmedi; MCP `tools/list` 0.3.2 ile bayt eşit; `contract.rs`, `mcp_schemas.rs`, `mcp_compat.rs` aynen geçer; mevcut vault'lar göçsüz açılır; `make conformance` 39/39. | eşit; geçti; 39/39 |
| Q3-5 | Kademe 2 bir tetikleme: zincir çıkışları 0, D farkı 0, sayılan FAIL 0, `portable` bulgu 0; değişen kayıtlar yalnız R0'da adı geçenler (adsız fark 0). Mac kümesi bir kez: kayıtlı, hüküm P2-V.2 okumasıyla, değişen değerler adıyla. | 0; 0; 0; adsız 0; kayıtlı |
| Q3-6 | **Ürün sınırı:** `git diff ROUND_START..HEAD -- core index bindings spec clients/*/src` boş (sürüm dizgileri hariç). `server/` farkı yalnız `serve.rs`, `tools.rs`, gerekirse `main.rs` ve yeni testler. Dondurulmuş dosyalarda değişiklik yalnız FREEZE_EXCEPTIONS ile. D-NOTES 0.3.3 Mac ↔ Docker ↔ CI ve 0.3.2 kayıtlarıyla bayt eşit (ya da yalnız R0'da adlı farklar). | boş; yalnız bunlar; aynen; eşit |
| Q3-7 | **`check_deps`:** kök neden bulundu ve düzeltildi, ya da yeniden üretilemedi ve hipotezler, sonuçları ve güçlendirilmiş tanı çıktısı yazılı. `test_check_deps.sh` geçer. | biri; geçer |
| Q3-8 | **Belgeler ve güvenlik:** kılavuz güncel (Known issues'tan PQ2-1 çıktı; uzlaştırma ve `ingest-dir` yazılı, D0F-7); CHANGELOG 0.3.3; açık Dependabot ve CodeQL uyarısı 0 ya da her biri yazılı gerekçeli. | hepsi |
| Q3-9 | 0.3.3: `release_check` 21/21 (R1–R20, 0.3.2 ile aynı küme); `release.yml` zorunlu iki hedef `ok`; `v0.3.3` yayımlı (insan); yayımlanmış 0.3.3 ile R4 (f)'nin durumları ve gerçek Claude Code oturum sonu sınaması geçti. | hepsi |
| Q3-10 | Süreç: `make test`, `make lint`, `make verify FAST=1` temiz; turun PR'ında kademe 1 yeşil; `check_frozen`, `check_deps`, `check_dep_age` temiz; gözden geçirici kabul. | hepsi |

## Karar kuralı
- **Hepsi geçti** → D0F-1, D0F-5, D0F-7 kapandı, 0.3.3 yayımlı. Sıradaki D-1 (K-D0.4). Kısa bağımsız doğrulama birleşmeden sonra.
- **Q3-1 kaldı ya da düzeltme spec'in tanımlamadığı bir olay/alan gerektiriyor** → PARK; karar talebi. 0.3.3 yayımlanmaz.
- **Q3-2 ya da Q3-4 kaldı** → hata kapanmadı ya da sözleşme bozuldu; 0.3.3 o düzeltmeyle yayımlanmaz; tur kaldı, karar talebi (D-1 bu hatayla mı açılır).
- **Q3-3 kaldı** → tur kaldı; karar talebi (0.3.3 yalnız başlangıç uzlaştırmasıyla yayımlanabilir mi).
- **Q3-5 kaldı** (adsız fark, sayılan FAIL, bulgu) → tur kaldı; karar talebi.
- **Q3-6 kaldı** → tur kaldı; ürün sınırı ihlali geri alınır, karar talebi.
- **Q3-7, Q3-8, Q3-9 ya da Q3-10 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Yeni MCP aracı, HTTP ucu ya da sözleşme alanı; konteynerde HTTP (D0F-3, D0-K8); `mdk_version` adı (D0F-4, sonraki sözleşme sürümü).
- Dağıtım işleri: ürün deposu, imaj, tap, belge sitesi, aktarım (D-1); lisans bildirimleri (D0F-2, D0F-8 → D-1).
- Marka taraması ve ad kararları (H2, ürün sahibinde, en sonda).
- P2-Q.2'nin öteki bulguları: PQ2-2, PQ2-3, PQ2-7 (→ P2-C/P2-H).
- Kişisel içerikle yeni ölçüm. Obsidian sahnesi koşulmaz; Mac kümesi gerçek Obsidian O kriterlerini zaten koşar.
