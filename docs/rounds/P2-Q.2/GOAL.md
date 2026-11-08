# P2-Q.2 — Yama Turu: MCP Uyumluluğu (Güncel Claude Code), Kaldırma Beklemesi ve 0.3.2

Başlangıç: P2-Q.1 karar kuralına göre Q1-4'te kaldı; K-P2Q1-T3 (a) ile kapandı. PQ-22 ve PQ-18 kapandı, 0.3.1 yayımlı. Bağımsız doğrulamanın hükmü **"güvenilir"** (`reports/rounds/P2-Q.1/VERIFICATION.md`, PR #53). K-P2Q1.2'ye göre P2-Q.1'den hemen sonra, D-0'dan önce bu yama turu (0.3.2) açılır. K-P2Q1.5 turu açar ve doğrulamanın önerilerini dağıtır.

Sorun:
- **PQ1-1 — güncel Claude Code rootmark'ın MCP araçlarını listeleyemiyor (ürün hatası).** Claude Code 2.1.318 (bugün 2.1.319) `rootmark serve --mcp stdio`'ya bağlanıyor ama `tools/list` şu hatayla dönüyor: `request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities`. Hata `rmcp 3.5.0`'ın istek meta doğrulamasından geliyor (`src/handler/server.rs`). Bir isteğin `_meta`'sı 2026-07-28 ya da sonrası bir protokol sürümü bildirince sunucu o sürümün bütün istek meta alanlarını istiyor. Elle `initialize` + `tools/list` 11 aracı veriyor, yani sunucu çalışıyor; uyumsuzluk istemcinin yeni istek meta'sıyla. 0.3.0 ve 0.3.1'de aynı. Sonucu: NORTH_STAR'ın ilk ürün sahnesinin ajan yolu (MCP `propose`) güncel Claude Code ile çalışmıyor. Kılavuz §4.1'in ajan ayrımı ajanın tek yazma yolunu MCP'ye bıraktığı için bugün ajan hiç yazamıyor (P2-Q.1 doğrulaması, VR4 (ii)). Kanıt: `docs/rounds/P2-Q.1/progress/R3_agent_isolation.md` "Yan bulgu", `reports/rounds/P2-Q.1/FINDINGS.md` PQ1-1.
- **PQ1-4 — kaldırma beklemesi alımdan önce hesaplanıyor (ürün hatası, izleyici).** `serve.rs` kaldırmanın bitiş anını (`due = Instant::now() + hold`) toplu işin belgeleri **alınmadan önce** hesaplıyor. Geç tasdik penceresi ise yeni yolun gözleminden başlıyor. Yeni yolun alımı yük altında 500 ms'den uzun gecikirse eski yolun kaldırılması `W` dolmadan alınabiliyor. O anda gelen geç yeniden adlandırma tasdiki bloksuz eski yola karşı denetlenip reddediliyor ve yeni yol `unverified` kalıyor (P2-Q.1 gözden geçiricisi B1; DESIGN §2.4'ün "her zaman önce olur" iddiası yük altında tutmuyor). Kanıt: `reports/rounds/P2-Q.1/FINDINGS.md` PQ1-4.

## İlkeler
- **Ürün değişikliği yalnız MCP katmanında ve izleyicide.** Değişebilecekler:
  - `server/src/mcp.rs`;
  - `server/Cargo.toml`'daki `rmcp` sürümü ve `Cargo.lock`;
  - PQ1-4 için `server/src/serve.rs` (gerekirse `tools.rs`'in izleme alımı).

  Değişmeyenler: `core`, `index`, `bindings`, `clients/*/src`, spec §1–6 ve uyumluluk fixture'ları. MCP araçlarının adları, giriş/çıkış şemaları ve anlamı da değişmez: araç listesi 0.3.1'inkiyle aynı, `mcp_schemas.rs` ve sözleşme testleri aynen geçer.
- **Bağımlılık kuralı.** `rmcp` yükseltilecekse sürüm `scripts/check_dep_age.sh`'e göre ≥ 14 günlük olur. Bugün uygun adaylar 3.6.0 (2026-10-20) ve 3.7.0 (2026-10-22); 3.8.0 (2026-10-26) 2026-11-09'dan sonra uygun. Yükseltmenin geçişli bağımlılık değişiklikleri `check_deps` ve `check_dep_age` ile denetlenir. Hiçbir uygun sürüm sorunu çözmüyorsa sunucu tarafında eski oturum modeli (2026-07-28 öncesi protokol sürümünü müzakere etmek, ya da istek meta doğrulamasını rmcp'nin açık bir yapılandırmasıyla gevşetmek) seçilir. Seçim ve gerekçesi `DESIGN.md`'de yazılır. **Fork ya da yama edilmiş rmcp kopyası yok.**
- **Tasarım notu koddan önce.** R0'da `docs/rounds/P2-Q.2/progress/DESIGN.md` şunları yazar:
  - PQ1-1'in kök nedeni: rmcp'nin hangi koşulda hangi alanları istediği, Claude Code'un hangi protokol sürümünü ve `_meta` alanlarını gönderdiği. İstemcinin istekleri bir **kayıtla** gösterilir; kayıtta içerik ve host değeri yok.
  - Seçilen yöntem ve alternatifinin neden seçilmediği.
  - Uyumluluk matrisi: protokol sürümleri `2025-06-18`, `2025-11-25`, `2026-07-28`+ × `_meta`'lı/`_meta`'sız istek × Claude Code ve genel istemci.
  - PQ1-4'ün yöntemi ve "kaldırma, eski yolun geç tasdik penceresi dolmadan alınmaz" değişmezinin nasıl sağlandığı.
  - Ön-kayıtlı testlerin listesi.

  Bunlar commit'lenir, sonra kod yazılır.
- **Geriye uyum ve determinizm.** Eski istemciler (2025 protokol sürümleri, `_meta`'sız istekler) aynen çalışır. Aynı dosya sistemi olay dizisi her makinede aynı olay günlüğünü üretir. Mevcut vault'lar göçsüz açılır, `make conformance` 39/39.
- **Gerçek istemciyle sınama.** Uyumluluk yalnız elle yazılmış JSON-RPC ile değil, **gerçek Claude Code** ile başsız sınanır: `claude -p` ile `--mcp-config` ve `--strict-mcp-config`, D-NOTES kopyası bir vault, kişisel içerik yok. Kullanılan Claude Code sürümü kayda yazılır. CI'da gerçek Claude Code koşulmaz (kimlik bilgisi CI'a konmaz). CI'ın kanıtı, gerçek istemcinin kaydedilmiş istek biçimlerini (protokol sürümü, `_meta` alanları; içerik yok) yeniden oynatan yeni bir Rust entegrasyon testidir.
- **Kayıt farklarının önceden adlandırılması (P2-Q.1 doğrulaması, Öneri 2).** R0'ın "değişmesi beklenen kayıtlar" listesine sürüm dizgisi alanları (0.3.2) ve E1-4/R14 adıyla yazılır. R14 etikete bağlıdır: `v0.3.2` tur içinde atılınca bir sonraki `ROUND_START`'a kadar R14 farklı okunur. **CI yapılandırmasındaki (`.github/workflows/**`) her değişiklik** de bu listeye adıyla girer; P2-Q.1'in Q1-4 kalışının nedeni `release.yml`'nin adsız değişikliğiydi.
- Sessiz başarısızlık yok; markdown'a kimlik/meta veri yazılmaz; kimliklere zaman damgası ya da rastgelelik girmez.
- **Ön-kayıt anahtarları kalıcı yerde.** Bu turda ya da sonrasında kişisel içerikli bir ön-kayıt anahtarı üretilirse `~/.config/rootmark-lab/` altında, `600` kipinde ve bir `README.txt` satırıyla tutulur; scratchpad'de tutulmaz. P2-Q'nün anahtarı scratchpad temizliğinde kayboldu (K-P2Q1-T2). Kural `docs/DEV_PROCESS.md`'ye yazılır (R3).

## Kapsam
1. **R0 — Açılış ve tasarım.**
   - `ROUND_START`; DECISIONS K-P2Q2.1 (açılış).
   - `DESIGN.md` (yukarıda) ve `TESTS.md` (ön-kayıtlı testler, beklenen sonuçlar).
   - Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtların adıyla listesi.
   - **PQ1-1'in 0.3.1'de yeniden üretimi:** yüklü güncel Claude Code + yayımlanmış `v0.3.1`, başsız; araç listesi alınamıyor; istemcinin istek biçimi kaydı.
2. **R1 — PQ1-1: MCP uyumluluğu.**
   - Yöntem `DESIGN.md`'deki gibi (`rmcp` yükseltmesi ya da eski oturum modeli).
   - Yeni Rust entegrasyon testi: `server/tests/mcp_compat.rs`, ya da benzer adlı **yeni** bir dosya; dondurulmuş testler değişmez. Uyumluluk matrisinin her hücresi için `initialize` → `tools/list` (11 araç, adlar ve şemalar 0.3.1 ile aynı) → bir `search` ve bir `propose` çağrısı. `propose` olay günlüğünde ajan aktörüyle görünür; `events_hash` belirlenimli.
   - Test CI'da `ubuntu-latest` ve macOS'ta koşar.
   - **Gerçek Claude Code ile başsız sınama** (yerel derlenmiş ikiliyle): araçlar listelenir, bir `search` ve bir `propose` başarılı olur, `propose` vault'ta insan onayı bekleyen öneri olarak görünür. Kayıt `progress/R1_claude_code.md`'ye yazılır: Claude Code sürümü, komut, çıktının içeriksiz özeti; token ve içerik yazılmaz.
3. **R2 — PQ1-4: kaldırma beklemesi.** Kaldırmanın alınma anı, eski yolun geç tasdik penceresinin (`W`) yeni yolun **alımından** sonra dolmasına bağlanır; yük altında da geçerli olmalı. Yeni entegrasyon testi, macOS ve Linux'ta: yeni yolun alımı yapay olarak `W`'den uzun geciktirilir (yalnız test yapılandırmasıyla; ürün varsayılanı değişmez), geç gelen tasdikli yeniden adlandırma yeni yolu yükseltir ve eski yol bir kez kapanır. ≥ 20 tekrar. Silme ve tasdiksiz taşıma testleri (`watch_remove.rs`) aynen geçer.
4. **R3 — Belgeler ve güvenlik.**
   - (a) Kılavuz §4.1: güncel Claude Code ile bağlanma ve doğrulama adımları (`/mcp`, araç listesi), sınanmış biçimde; "Known issues"tan PQ1-1 çıkar.
   - (b) P2-Q.1 doğrulamasının Öneri 3a'sı: kılavuzun "Removals are taken a little late" maddesine, `unverified` altında Obsidian'ın tasdikli yeniden adlandırmasının eski yolunun da `unattested_fs` ile kapandığı (kaldırma tasdik edilmez) eklenir.
   - (c) `CHANGELOG.md` `## 0.3.2` (düzeltmeler, Known issues).
   - (d) `docs/DEV_PROCESS.md`'ye anahtar saklama kuralı (İlkeler).
   - (e) **`minimatch`** (Dependabot #80, #81; düzeltme 11.0.1, 14 gün kuralı 2026-11-10'dan sonra): iki istemcide `overrides` ile; `make lint-ts` ve `make test-ts` temiz olmalı. Değilse gerekçeyle açık bırakılır.
   - (f) Açık Dependabot ve CodeQL uyarıları 0 ya da her biri yazılı gerekçeli; son durum `progress/R3_security.md`'de.
5. **R4 — Doğrulama koşuları ve 0.3.2.**
   - (a) **Kademe 2** bir tetikleme. D farkı 0, sayılan FAIL 0, bulgu 0; değişen kayıtlar R0'ın listesiyle karşılaştırılır.
   - (b) **Mac kümesi** bir kez (`make verify-mac`; izleyiciye dokunuldu, DEV_PROCESS §3). Koşudan önce `fseventsd` sağlığı `ps` ile kaydedilir; sağlıksızsa `HUMAN_STEP.md`.
   - (c) **0.3.2 yayımı:** `release/0.3.2` dalı, R1–R20, PR; insan adımları `HUMAN_STEP.md`'de (birleşme, `release.yml`, `v0.3.2` etiketi, GitHub sürümü).
   - (d) **Yayımlanmış 0.3.2 ile:**
     - gerçek Claude Code başsız sınaması yeniden (R1'in adımları, yüklü 0.3.2 ikilisiyle);
     - P2-Q'nün D-NOTES senaryosu Mac, Docker ve CI'da (`pilot.yml`, bir koşu). `events_hash`, vektör dosyaları ve indeks manifest'i **0.3.1'in kayıtlarıyla bayt eşit** olmalı; ürün çıktısı değişmemeli.

     Kayıtlar `reports/rounds/P2-Q.2/` altına.
6. **R5 — Rapor, gözden geçirici, hazır.**
   - FINAL; `FINDINGS.md` (PQ1-1 ve PQ1-4 kapandı; yeni bulgular sınıflı ve hedefli).
   - Gözden geçirici tek tur. Başlık: "MCP araç sözleşmesi değişti mi; eski istemciler çalışıyor mu; gerçek Claude Code sınaması gerçekten gerçek istemciyle mi; PQ1-4 yük altında kapandı mı; ürün sınırı".
   - READY; tek PR; birleşmeden sonra temizlik (K-P2V2.2).

## Önceden kayıtlı çıkış kriterleri
Senaryo ve test adımları için "koştu" ile "geçti" ayrı sayılır (P2-Q.1 doğrulaması, Öneri 3b). Kriter, koşan **her** adımın geçmesini ister.

| Kod | Ölçüm | Eşik |
|---|---|---|
| Q2-1 | `DESIGN.md` ve `TESTS.md`, R1'in ilk kod commit'inden önce commit'li. PQ1-1'in kök nedeni istemcinin kaydedilmiş istek biçimiyle gösterilmiş. Yöntem seçimi gerekçeli. Fork/yamalı rmcp yok. | önce; kayıtlı; fork 0 |
| Q2-2 | **PQ1-1:** (i) 0.3.1 ile yeniden üretildi (araç listesi alınamıyor). (ii) Yeni uyumluluk testi matrisin her hücresinde `ubuntu-latest` ve macOS'ta geçti. (iii) Gerçek Claude Code (sürümü kayıtlı, ≥ 2.1.318) başsız: R1'de yerel ikiliyle ve R4'te yayımlanmış 0.3.2 ile araç listesi 11/11, `search` ve `propose` başarılı, `propose` günlükte ajan aktörüyle. | hepsi |
| Q2-3 | **Sözleşme ve geriye uyum:** araç adları ve şemaları 0.3.1 ile bayt eşit (`tools/list` çıktısı karşılaştırması). `mcp_schemas.rs`, `contract.rs` ve öteki mevcut testler aynen geçer. 2025 protokol sürümleriyle `_meta`'sız istemci çalışır. `make conformance` 39/39. | eşit; geçti; 39/39 |
| Q2-4 | **PQ1-4:** yeni test macOS ve Linux'ta ≥ 20 tekrar geçti (yeni yolun alımı `W`'den uzun gecikse de geç tasdikli yeniden adlandırma yeni yolu yükseltir; eski yol bir kez kapanır). `watch_remove.rs` ve `watch_newdir.rs` aynen geçer. | hepsi |
| Q2-5 | Kademe 2 bir tetikleme: zincir çıkışları 0, D farkı 0, sayılan FAIL 0, `portable` bulgu 0. Değişen kayıtlar yalnız R0'da adı geçenler (adsız fark 0). Mac kümesi bir kez: kayıtlı, hüküm P2-V.2 okumasıyla, değişen değerler adıyla. | 0; 0; 0; adsız 0; kayıtlı |
| Q2-6 | **Ürün sınırı:** `git diff ROUND_START..HEAD -- core index bindings spec clients/*/src` boş (sürüm dizgileri hariç). `server/` farkı yalnız `mcp.rs`, izleyici, `Cargo.toml`/`Cargo.lock` ve yeni testler. Dondurulmuş dosyalarda değişiklik yalnız FREEZE_EXCEPTIONS ile. Eski vault'lar göçsüz açılır. D-NOTES 0.3.2 Mac ↔ Docker ↔ CI ve 0.3.1 kayıtlarıyla bayt eşit. | boş; yalnız bunlar; aynen; eşit |
| Q2-7 | **Belgeler ve güvenlik:** kılavuz §4.1 güncel Claude Code ile sınanmış; Known issues'tan PQ1-1 çıkmış; Öneri 3a yazılı; anahtar kuralı DEV_PROCESS'te; CHANGELOG 0.3.2. `rmcp` (yükseldiyse) ≥ 14 gün. Açık Dependabot ve CodeQL uyarısı 0 ya da her biri yazılı gerekçeli. | hepsi |
| Q2-8 | 0.3.2: R1–R20 21/21; `release.yml` zorunlu iki hedef `ok`; `v0.3.2` yayımlı (insan). | hepsi |
| Q2-9 | Süreç: `make test`, `make lint`, `make verify FAST=1` temiz; turun PR'ında kademe 1 yeşil, FAST ≤ 40 dk; `check_frozen`, `check_deps`, `check_dep_age` temiz; gözden geçirici kabul. | hepsi |

## Karar kuralı
- **Hepsi geçti** → PQ1-1 ve PQ1-4 kapandı, 0.3.2 yayımlı, ajan yolu güncel Claude Code ile çalışıyor. Sıradaki D-0 (isim ve depo), sonra D-1 (K-D0.1'deki sıra). Kısa bağımsız doğrulama birleşmeden sonra.
- **Q2-1 kaldı ya da hiçbir uygun yöntem fork'suz çalışmıyor** → PQ1-1 park; karar talebi (ör. 14 gün kuralına tek seferlik istisna, rmcp'den ayrılmak, istemci sürümünü sabitlemek). PQ1-4 sürer, 0.3.2 yalnız onunla yayımlanmaz (karar talebiyle).
- **Q2-2 ya da Q2-3 kaldı** → PQ1-1 kapanmadı ya da sözleşme bozuldu; 0.3.2 o düzeltmeyle yayımlanmaz; tur kaldı, karar talebi.
- **Q2-4 kaldı** → PQ1-4 kapanmadı; tur kaldı, karar talebi (0.3.2 yalnız PQ1-1 ile yayımlanabilir mi).
- **Q2-5 kaldı** (adsız fark, sayılan FAIL, bulgu) → tur kaldı; karar talebi (değişen kaydın adı ve sınıfı için; yeni ad kararla).
- **Q2-6 kaldı** → tur kaldı; ürün sınırı ihlali geri alınır, karar talebi.
- **Q2-7, Q2-8 ya da Q2-9 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- MCP araçlarına yeni araç, alan ya da davranış eklemek; ajan kimliği (P2-E).
- P2-Q ve P2-Q.1'in öteki bulguları: PQ1-2 (0.3b ölçümü, rapor), PQ1-3 (sahnenin sabit sürümü → P2-Q.S), PQ1-5 (→ P2-C), PQ-13/14/15/19/20 (→ P2-C).
- Gömme hızı (P2-H), telemetri (P2-O), sync (P2-C).
- İsim değişikliği ve depo bölünmesi (D-0, D-1; bu tur yalnız `rootmark` adıyla 0.3.2).
- Kişisel içerikle yeni ölçüm. Obsidian sahnesi bu turda koşulmaz; değişiklik Obsidian eklentisinin yolunu etkilemez, Mac kümesi gerçek Obsidian O kriterlerini zaten koşar.
