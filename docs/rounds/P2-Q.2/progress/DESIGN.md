# P2-Q.2 — Tasarım notu (R0; R1/R2'nin kodundan önce)

Kapsam: MCP katmanı (`server/src/mcp.rs`, `server/Cargo.toml`'daki `rmcp` sürümü, `Cargo.lock`) ve izleyici (`server/src/serve.rs`). `core`, `index`, `bindings`, istemcilerin kaynakları, spec ve fixture'lar değişmez. MCP araçlarının adları, şemaları ve anlamı değişmez; yeni olay türü, alan ya da biçim yok.

## 1. PQ1-1 — kök neden (kayıtla)

### 1.1 İstemcinin istekleri (kayıt)
Yayımlanmış `v0.3.1` (`rootmark-0.3.1-aarch64-apple-darwin`, `SHA256SUMS` ile denetlendi) + yüklü Claude Code **2.1.319**, başsız (`claude -p … --mcp-config <dosya> --strict-mcp-config --debug-file …`), vault D-NOTES'un depo dışı bir kopyası. Sunucu, stdio'yu bayt bayt kopyalayan ve her iletinin yalnız **biçimini** yazan bir kaydedicinin (`research/p2q2/mcp_tap.py`) arkasında başlatıldı. Kayıt: `progress/r0_requests.json`; koşu özeti: `reports/rounds/P2-Q.2/claude_code/r0_v0.3.1.json`; anlatım: `progress/R0_repro.md`.

| # | Yön | İleti | Biçim |
|---|---|---|---|
| 1 | istemci → sunucu | `server/discover` (id: dize) | `params` yalnız `_meta`: `io.modelcontextprotocol/protocolVersion = "2026-07-28"`, `…/clientInfo = {claude-code, 2.1.319}`, `…/clientCapabilities = {elicitation, extensions, roots}` |
| 2 | sunucu → istemci | hata `-32022` "Unsupported protocol version" | |
| 3 | istemci → sunucu | `initialize` (id: sayı) | `protocolVersion = "2025-11-25"`, `capabilities = {elicitation, roots}`, `clientInfo = {claude-code, 2.1.319}`; `_meta` yok |
| 4 | sunucu → istemci | sonuç, `protocolVersion = "2025-11-25"` | |
| 5 | istemci → sunucu | `notifications/initialized` | |
| 6 | istemci → sunucu | `tools/list` | `params` yok (dolayısıyla `_meta` yok) |
| 7 | sunucu → istemci | hata `-32602` "request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities" | |

Claude Code'un hata ayıklama günlüğü: `protocolEra: "legacy"`, `negotiatedProtocolVersion: "2025-11-25"`, ardından `Failed to fetch tools: …` (yukarıdaki 7). Yani istemci 2026-07-28'i önce `server/discover` ile **deniyor**, reddedilince eski (oturumlu, `initialize`'lı) modele düşüyor ve sonraki isteklerine istek başına meta koymuyor — 2025-11-25 oturum modelinde doğru davranış budur.

### 1.2 rmcp 3.5.0 bu diziyi neden reddediyor
`rmcp-3.5.0/src/service/server.rs`, `serve_server_with_ct_inner`: sunucu ilk isteği okur. İstek `initialize` değilse ve `_meta`'sı 2026-07-28'in zorunlu anahtarlarını (`protocolVersion`, `clientCapabilities`; `model/meta.rs` `missing_required_keys`) **tam** taşıyorsa, oturum **satır içi yaşam döngüsüne** geçer: `peer.require_request_metadata()` bayrağı konur ve istek ana döngüde işlenir (`server/discover` → `handler/server.rs` `handle_request`: istenen sürüm 2026-07-28 `supported_protocol_versions`'ta yok → `-32022`). Bayrak oturum boyunca kalır. Sonra gelen `initialize` ana döngüde sıradan bir istek gibi işlenir; bayrak sıfırlanmaz. `tools/list` geldiğinde `handle_request` `requires_request_metadata = uses_inline_negotiation && (… || context.peer.request_metadata_required() || …)` koşulunu doğru bulur, `_meta` yoktur, 2026-07-28'in anahtarları eksik → `-32602`. Kısaca: **bir keşif yoklaması (`server/discover`) oturumu kalıcı olarak satır içi modele kilitliyor**; istemcinin eski modele düşmesi mümkün olmuyor. Sunucu kodumuz (`mcp.rs`) bu yola girmiyor; yol tamamen rmcp'nin yaşam döngüsü seçimindedir. Elle `initialize` + `tools/list` (keşifsiz) 11 aracı verir (P2-Q.1 R3) — çünkü ilk istek `initialize`'dır ve bayrak hiç konmaz.

### 1.3 Aday sürümler (≥ 14 gün; kaynak ve değişiklik notları okundu)
crates.io tarihleri: 3.5.1 (2026-10-05), **3.6.0 (2026-10-20)**, 3.7.0 (2026-10-22), 3.8.0 (2026-10-26; 2026-11-09'dan sonra uygun), 3.9.0 (2026-11-05; uygun değil).
- **3.6.0**, `service/server.rs`: "Select the lifecycle only after an initialize request or the first valid non-discover request with complete inline metadata. **A discover request is a bootstrap probe: respond to it, but remain open to either lifecycle.**" `server/discover` artık geçici bir eşle yanıtlanır (`continue`), `require_request_metadata` konmaz; eksik meta'lı ilk istek de hata alır ama oturum açık kalır (3.5.0'da sunucu dururdu). 3.7.0 ve 3.8.0 aynı kodu taşır. `handler/server.rs`'in istek meta denetimi (`requires_request_metadata`) 3.5.0–3.9.0 arasında aynıdır (yalnız bir değişken adı); yani sorun meta denetiminde değil, yaşam döngüsü seçimindedir ve 3.6.0'da kapanır.
- **API:** sunucumuz 3.6.0 ile **kod değişikliği olmadan** derlenir (depo dışı bir çalışma ağacında denendi: `cargo build --release -p rootmark-server`, uyarısız). Kullandığımız yüzeyler (`ServerHandler`, `Service`, `Transport`, `ServerConfig`, `ProtocolVersion::known_up_to`, `ListToolsResult`, `CallToolResult`) değişmemiş.
- **Ön sınama (tasarımın dayanağı; Q2-2'nin kanıtı değildir — o, R1'de commit'li kodla):** aynı başsız Claude Code 2.1.319 sınaması, 3.6.0 ile derlenmiş deneme ikilisiyle: `server/discover` yine `-32022`, `initialize` 2025-11-25, `tools/list` **11 araç**, `search` ve `propose` başarılı, `propose`'un iki `BlockCreated`'ı `author_kind = Agent`, `source = claude-code/2.1.319` (`reports/rounds/P2-Q.2/claude_code/pre_rmcp_3.6.0.json`). Genel istemcinin `tools/list` çıktısı (2025-06-18 el sıkışması; sıralı anahtarlarla yeniden yazılmış JSON) 0.3.1'inkiyle bayt eşit.
- **Geçişli bağımlılık:** `rmcp 3.6.0` `rmcp-macros = "3.6.0"` (alt sınır) ister; `cargo` kendiliğinden 3.9.0'ı (2026-11-05, < 14 gün) seçer. `Cargo.lock`'ta `rmcp-macros` **3.6.0**'a sabitlenir (`cargo update -p rmcp-macros --precise 3.6.0`). Başka geçişli değişiklik `cargo tree` farkıyla R1'de yazılır; `check_deps` ve `check_dep_age` temiz olmalı.

## 2. PQ1-1 — seçilen yöntem ve alternatif
**Seçilen:** `rmcp` `=3.5.0` → **`=3.6.0`** (`server/Cargo.toml`, `Cargo.lock`; `rmcp-macros` 3.6.0). Fork ya da yamalı kopya yok. `mcp.rs`'te davranış değişikliği yok; `supported_protocol_versions` aynı kalır (`known_up_to(2025-11-25)`: yalnız `initialize`'lı sürümler; tüketici kimliği `initialize`'dan gelir — `mcp.rs` başlığı). Gerekçe: hata yukarı akışın düzelttiği bir yaşam döngüsü hatası; düzeltme en küçük sürüm adımında (3.5.0 → 3.6.0) ve API'siz; 3.7.0/3.8.0 bu tur için gereksiz değişiklik getirir (kimlik doğrulama ve HTTP istemcisi; bizim kullanmadığımız özellikler), 3.8.0 bugün henüz 14 günlük değil.

**Alternatif (seçilmedi): sunucuda eski oturum modeli.** `mcp.rs`'in `Sequenced` taşıyıcısı ilk iletiyi rmcp'den önce görür; `server/discover`'ı kendisi `-32022` ile yanıtlayıp rmcp'ye hiç iletmeyebilirdi (oturum `initialize` ile açılır, bayrak konmaz). İşe yarardı, ama MCP yaşam döngüsünün bir parçasını kendi kodumuzda yeniden yazmak olurdu (rmcp'nin sonraki sürümleriyle ayrışma, `-32022`'nin `data`'sının biçimi, `ping` sırası gibi ayrıntıların bizde tekrar edilmesi). Yukarı akışın düzeltmesi uygun yaşta olduğu için gerekmiyor. 3.6.0 beklenmedik bir gerilemeyle Q2-3'ü bozarsa geri dönülecek yol budur (o durumda DESIGN eki ve karar talebi).

**İstek meta denetimini gevşetmek** (rmcp'nin açık bir yapılandırması): 3.5.0–3.6.0'da böyle bir yapılandırma yok (`request_metadata_required` `pub(crate)`); seçenek mevcut değil.

## 3. Uyumluluk matrisi (beklenen; R1'in testi `server/tests/mcp_compat.rs`)
Sunucunun desteklediği sürümler değişmez: `2024-11-05`, `2025-03-26`, `2025-06-18`, `2025-11-25` (yalnız `initialize`'lı). "Çalışır" = `tools/list` 11 araç ve 0.3.1'in listesiyle bayt eşit, `search` başarılı, `propose` başarılı ve günlükte `author_kind = Agent`, `source = <clientInfo.name>/<clientInfo.version>`.

| Hücre | İstemci | El sıkışma | İsteklerin `_meta`'sı | 0.3.1 (rmcp 3.5.0) | 0.3.2 (rmcp 3.6.0) |
|---|---|---|---|---|---|
| G1 | genel | `initialize` 2025-06-18 | yok | çalışır | çalışır |
| G2 | genel | `initialize` 2025-11-25 | yok | çalışır | çalışır |
| G3 | genel | `initialize` 2026-07-28 istenir → 2025-11-25 müzakere | yok | çalışır | çalışır |
| G4 | genel | `initialize` 2025-11-25 | tam, `protocolVersion = 2025-11-25` (+ `clientInfo`, `clientCapabilities`) | çalışır | çalışır |
| G5 | genel | `initialize` 2025-06-18 | ilgisiz anahtar (`progressToken`) | çalışır | çalışır |
| G6 | genel | `initialize` 2025-11-25 | tam, `protocolVersion = 2026-07-28` | `-32022` (açık hata) | `-32022` (açık hata; sunucu 2026-07-28'i ilan etmez) — aynı |
| G7 | genel (keşifsiz satır içi deneme) | önce `tools/list`, tam 2026-07-28 meta'sıyla; sonra `initialize` 2025-11-25 | sonra yok | ilk istek `-32022`, sonra her istek `-32602` (kilitli) | ilk istek `-32022`, sonra çalışır |
| C1 | Claude Code (kaydın birebir biçimi) | `server/discover` tam 2026-07-28 meta'sıyla → `initialize` 2025-11-25 → `notifications/initialized` | yok | `tools/list` `-32602` (PQ1-1) | çalışır |
| C2 | Claude Code benzeri, eksik meta'lı keşif | `server/discover` yalnız `protocolVersion` → `initialize` 2025-11-25 | yok | 3.5.0'da sunucu durur (beklenen; kaydedilir) | keşif `-32602`, sonra çalışır |

G6'nın satırı bir uyumsuzluk değil, bilinçli sınırdır: 2026-07-28'in satır içi modeli (oturumsuz, istek başına kimlik) bu turun kapsamı dışında; sunucu onu ilan etmez ve açık hata döner (sessiz başarısızlık yok). Güncel Claude Code bu durumda eski modele düşer (C1).

## 4. PQ1-4 — kaldırma beklemesi

### 4.1 Değişmez
**Kaldırma, eski yolun geç tasdik penceresi dolmadan alınmaz.** Geç tasdik penceresi `W` (`[attestation] window_s`) yeni yolun **gözleminden** başlar: `watch_doc` yazar kuyruğunda bileti aldıktan sonra dosyayı okur ve `attesting.decide`/`now_ms` ile gözlem anını kaydeder (`tools.rs` `external_plan`, `External.observed_ms`). Bu an, toplu işin alımı sırasında, kuyruk beklemesinden **sonradır**.

### 4.2 Bugünkü hata
`serve.rs` `start_watcher`: `let due = Instant::now() + hold;` toplu işin belgeleri alınmadan **önce** hesaplanır; kaldırma `held.insert(d, due)` ile bu ana bağlanır; aynı yinelemenin sonunda `t <= now` olan kaldırmalar alınır. Toplu işin alımı (kuyruk beklemesi + okuma + gömme) `hold`'dan uzun sürerse eski yolun kaldırılması yeni yolun alımının hemen ardından, `W` dolmadan alınır; o anda gelen geç tasdik eski yolun bloksuz başına karşı denetlenir ve reddedilir.

### 4.3 Yöntem
1. **Bekleme toplu işin alımından sonra başlar.** Bir toplu işte kaldırmaya alınan her belge için `due = <toplu işin son alımının bittiği an> + hold` (`hold = W + 500 ms`, değişmez). Yani `due` belge döngüsünden **sonra** hesaplanır.
2. **Sınırlı uzatma (yeni yol sonraki bir toplu işte gelirse).** Bekleyen her kaldırma `(due, orig)` taşır; `orig` ilk hesaplanan `due`'dur. Belge alan bir toplu iş, **başlangıcı** (ilk olayının alındığı an) bir kaldırmanın `orig`'inden önceyse, o kaldırmanın `due`'sunu `max(due, <bu toplu işin son alımının bittiği an> + hold)` yapar. Böylece kaldırmanın penceresi içinde başlamış (yeniden adlandırmanın yeni yolunu taşıyabilecek) her toplu işin alımından sonra en az `hold` beklenir; `orig`'den sonra başlayan toplu işler uzatmaz — sürekli yazım akışı bir kaldırmayı süresiz bekletemez (üst sınır: `orig` + bir toplu işin süresi + `hold`).
3. Kaldırmanın kendisi, durdurmada hemen alım, bayt sırası, `ingest_removal`'ın yeniden yaratılmış dosyayı değişiklik olarak alması **aynı**. Duvar saati yine yalnız *ne zaman* işleneceğine karar verir; günlüğe ya da kimliğe girmez (aynı olay dizisi aynı günlüğü üretir).
4. **Test yapılandırması (ürün varsayılanı değişmez):** ortam değişkeni `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` (yalnız testler kullanır; yoksa ya da boşsa davranış bugünküdür). Verilirse izleyici iş parçacığı bir toplu işte **dosyası var olan** her belgeyi almadan önce o kadar bekler — yazar kuyruğundaki beklemenin (yük) yerine geçer: gözlem anı (`observed_ms`) bekleme bittikten sonradır, tıpkı kuyruk beklemesinde olduğu gibi. Değer izleyici başlarken bir kez okunur ve stderr'e "test setting" satırıyla yazılır (sessiz değil); sayı değilse servis hata ile durur. Sürüm notunda ve kılavuzda yer almaz (kullanıcı ayarı değildir); kodda belgelenir.

### 4.4 Neden yeter
- Aynı toplu iş (yeniden adlandırmanın iki yolu bir arada; macOS FSEvents ve inotify'da tipik): eski yolun `due`'su yeni yolun alımı bittikten sonra `hold` = `W + 500 ms` — yeni yolun gözlemi alımın içinde olduğundan, geç tasdik penceresi (gözlem + `W`) kaldırmadan önce kapanır.
- Yeni yol sonraki bir toplu işte (olaylar 50 ms'lik sessizlikten sonra gelirse): o toplu iş `orig`'den önce başlar (yeniden adlandırmanın iki olayı arasındaki süre `hold`'dan çok kısadır) → uzatma 2'den.
- Tasdikin kendisi de yazar kuyruğundan geçer; kaldırmanın alımı da. Uzatma yalnız kaldırmanın *zamanını* geciktirir, sırasını ya da biçimini değiştirmez.

### 4.5 Sınırlar
- Yeni yolun alımı, kaldırmanın `orig`'inden sonra başlayan bir toplu işe düşerse (yeniden adlandırmanın iki olayı arasında `hold`'dan uzun süre) uzatma olmaz. Bu, yeniden adlandırma olmayan bir dizidir (bir silme ve çok sonra bir yaratma) ve tasdiksiz taşıma gibi alınır.
- Gömme süresi gözlemden sonra, ama tasdik de kuyrukta beklediği için gözlem + `W`'yi aşan bir gömme, tasdiki pencere dışına itebilir; bu PQ1-4 değil, P2-H'nin (gömme hızı) konusudur ve bu tur değiştirmez.

## 5. Ön-kayıtlı testler
`progress/TESTS.md`. Yeni dosyalar: `server/tests/mcp_compat.rs` (R1), `server/tests/watch_hold.rs` (R2), test verisi `server/tests/fixtures/p2q2/` (0.3.1'in `tools/list` çıktısı, R1'in ilk commit'i). Dondurulmuş testler (`mcp_schemas.rs`, `contract.rs`, `serve_watch.rs`) ve `watch_remove.rs`, `watch_newdir.rs` değişmez. CI'da `make test` (`cargo test --workspace --release`) macOS ve `ubuntu-latest` işlerinde koşar (`ci.yml`) ve yeni test dosyalarını kendiliğinden içerir: **CI yapılandırması değişmez.**

## 6. Kademe 2 ve Mac kümesinde değişmesi beklenen kayıtlar (önceden, adıyla; K-P2Q1.4)
Kademe 2'nin hükmü A↔B'dir (aynı commit'in iki zinciri). Aşağıdakiler **önceki koşulara göre** (P2-Q.1'in tetiklemesi `41944358184` ve doğrulayıcının `42471635466`'sı) değeri değişebilecek kayıtlardır; R4'te turun tetiklemesinin kayıtları bunlarla karşılaştırılır. Liste dışında bir değişiklik **adsız fark**tır.
1. **Sürüm dizgileri (0.3.1 → 0.3.2).** Yayım dalı birleştikten sonra tur dalı `main`'in üstüne alınır ve crate'ler 0.3.2 olur: rootmark'ın kendi sürümünü yazan her alan — `status`'un `mdk_version`'ı, MCP `serverInfo.version` ve `instructions` metni, `rootmark --version`, eklenti ve VS Code iskeletinin `version`'ı, `release_check`'in R1 değerleri (crate ve CHANGELOG sürümleri). Alan adı değişmez, yalnız `0.3.1` dizgisi `0.3.2` olur.
2. **E1-4 / R14 (etikete bağlı; K-P2Q1.4 (2)).** `v0.3.2` tur içinde atılınca bir sonraki `ROUND_START`'a kadar `release_check` R14 farklı okunur ("etiket yok ya da etiketin commit'i en yeni `ROUND_START`'ın atası" artık sağlanmaz); Mac kümesinin E1-4 kaydı bu yüzden farklı olabilir. Turun içinde CHANGELOG başlığı "## Unreleased — 0.3.2" olur (R1 crate sürümüyle çelişmez; P2-Q.1 dersi).
3. **Bağımlılık kayıtları.** `rmcp` 3.5.0 → 3.6.0 ve `rmcp-macros` 3.5.1 → 3.6.0: `check_dep_age`'in kaydında `rmcp` satırı (sürüm ve yayım tarihi); `check_deps` günlüğü bağımlılık yönünü denetler, değişmez beklenir.
4. **CI yapılandırması (`.github/workflows/**`): değişiklik planlanmadı.** Yeni testler `make test`'le koşar. Bir değişiklik gerekirse bu listeye o değişiklikten **önce** adıyla eklenir (DESIGN eki, commit'li); eklenmemiş her iş akışı değişikliği adsız farktır.
5. **İzleyici (PQ1-4) ve MCP (PQ1-1) için değişmez beklenenler:** kademe 2'nin D kayıtları — `p1d/inspector` (Inspector 2.9.0'ın `tools_list.json`'ı ve çağrıları: araç listesi aynı), `p2_0_2/tamper_p2_0_2.json` (izleyici altında üç yeniden adlandırma; pencere içinde geldikleri ve yük olmadığı için sonuçlar aynı), öteki MCP ve izleyici kayıtları. Mac kümesi: `Z-D10`, `Z-DA` değişmez beklenir; O kümelerinin yeniden adlandırma oturumları (0.3b/0.3b-yük/0.3c/0.3d/0.3e, `p2_0_3_checks.json`; D-B4) **değerleri** değişebilir (PQ1-4 düzeltmesi eski yolların kaldırılmasını yük altında geciktirir: `reported_only` 0.3b'nin yanlış `unverified` oranı düşebilir) — değerler adıyla raporlanır; hüküm P2-V.2 okumasıyla (deterministik O kriterleri geçmeli, `reported_only` hükme girmez).
6. **Kademe 1 (FAST):** yalnız sürüm dizgileri (1).

## 7. Determinizm ve geriye uyum
Yeni olay türü, alan, biçim ya da sürüm yok; eski günlükler aynen okunur, göç gerekmez; `make conformance` 39/39 beklenir. MCP araç listesi ve şemaları 0.3.1'inkiyle bayt eşit (Q2-3; `mcp_compat.rs` sabit dosyaya karşı denetler). D-NOTES senaryosu (`p2q.driver`, MCP `2025-06-18`) 0.3.2 ile 0.3.1'in kayıtlarıyla bayt eşit beklenir (`events_hash` `173c13df79d2d98dbff1df45453e193c`, vektörler, manifest).
