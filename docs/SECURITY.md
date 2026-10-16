# Güvenlik ve tehdit modeli (faz 1)

Durum: taslak (P1-D.1/D1.0, K-D1.3). Kapsam: tek makinede, tek kullanıcının vault'u üzerinde çalışan `mdk serve` (MCP stdio + 127.0.0.1 HTTP) ve `mdk` CLI. Çok kullanıcılı sunucu, uzak erişim, dinlenmede şifreleme faz 2'dedir (ROADMAP P2-D).

## Varlıklar
- Markdown dosyaları (tek gerçek kaynak) ve onlardan türetilen `.mdkernel/` (olay günlüğü, nesneler, checkpoint'ler, izler, indeks).
- Köken: her bloğun kim tarafından yazıldığı, her katman geçişinin ve her silme/değiştirmenin aktörü (K-D1.2).
- `canonical` içeriğin bütünlüğü: insanın onaylamadığı hiçbir şey bilgi sayılmaz (NORTH_STAR).

## Güven sınırları ve kanallar (K-D1.3)
| Kanal | Kimlik doğrulama | Aktör | Ne yapabilir |
|---|---|---|---|
| MCP stdio (`mdk serve --mcp stdio`) | süreci başlatan istemci; kimlik `initialize.clientInfo` | daima `agent`, `id` = istemci kimliği `<clientInfo.name>/<clientInfo.version>` (K-D1.3'ün "istemci adı"; P1-D ASSUMPTIONS D1-1 biçimi) | okuma, iz yazan okuma, `propose` (EditPolicy altında), `checkpoint`; `promote` → `denied` |
| HTTP 127.0.0.1 (`--http`) | `Authorization: Bearer <.mdkernel/server.token>` | `human:<policy.toml [identity] human_id>` | MCP'nin yaptığı her şey + `promote` (`HumanOnly` ile izinli); `human_id` yoksa yazma uçları 403 |
| CLI `mdk` | yerel kullanıcının dosya izinleri | komutta açıkça (`--actor human:<id>`); `mdk propose` ajan | her şey; politika hook'ları yine uygulanır |
| Dosya izleyici (`--watch`) | dosya sistemine yazabilen herkes | `human` (dışarıdan düzenleme insan yazımı sayılır) | md değişikliklerini insan içeriği olarak alır |

- `X-Consumer` yalnız iz etiketidir (`traces.jsonl` `consumer`). Aktör türünü değiştiremez: `X-Consumer: human:me` bir MCP/HTTP çağrısını insan yapmaz, insanlığı token yapar.
- **Kabul edilen varsayım:** token dosyasını (`.mdkernel/server.token`, mod 600) okuyabilen her yerel süreç insan sayılır. Aynı kullanıcı hesabıyla çalışan bir ajan süreci token'ı okuyabilirse insan kanalına geçebilir. Bunu kapatmak OS anahtar zinciri ya da ayrı kullanıcı hesabı ister (faz 2). Önerilen işletim: ajanı, vault'un `.mdkernel/` dizinini okuyamayan bir hesapta ya da kum havuzunda, yalnız MCP üzerinden çalıştırmak.
- Aynı şekilde vault dosyalarına doğrudan yazabilen bir süreç, izleyici açıksa insan olarak yazar. MCP dışında ajana dosya sistemi yazma izni vermek, köken garantisini kaldırır.

## Yönetişim garantileri
- **Terfi** (spec §7, K12/K16): çekirdek karar vermez, `policy.toml [promotion]` hook'u karar verir. MCP'den terfi daima `denied`.
- **Ajan yazımı** (K-D1.1): varsayılan `[edit] policy = "append_only"`. Canonical bir bloğa dokunan (silen, değiştiren, taşıyan, bölen, birleştiren, katmanını düşüren) ya da canonical bir arama birimini (chunk) değiştiren her ajan yazımı `denied`. `unrestricted` yalnız açıkça yazılırsa geçerlidir; o zaman da her silme/değiştirme olayı aktörünü taşır (K-D1.2).
- **Sıra** (K-D1.4): yazan ve iz yazan çağrılar vault başına tek FIFO kuyruğundan geçer. Aynı istek dizisi aynı günlüğü üretir.
- **Sessiz başarısızlık yok**: reddedilen her çağrı kararlı bir hata kodu döner (K-D1.5, `docs/CLIENT_CONTRACT.md`). Reddedilen yazım md dosyalarına ve `events.jsonl`'e hiçbir şey yazmaz.

## Ağ
- HTTP yalnız `127.0.0.1`'e bağlanır; başka her adres (IPv6 loopback dahil) başlamadan reddedilir.
- `Host` başlığı denetlenmez. DNS rebinding ile tarayıcıdan gelen bir istek token'ı bilmediği için 401 alır. Token gizli kaldığı sürece okuma da yazma da yapılamaz.
- Token 64 hex (256 bit), sabit zamanlı karşılaştırılır, `[server] token_rotate_days` ile döner.

## Bilinen açık sınırlar
- **TOCTOU (K-D4-T c):** `propose`'un yol denetimi ile yazımı arasında, vault'a yazma izni olan başka bir yerel süreç bir dizini sembolik bağla değiştirebilir. Tam kapatmak `openat`/`O_NOFOLLOW` zinciri ister (faz 2).
- **Büyük/küçük harf duyarsız dosya sistemleri:** dosyası silinmiş bir dokümanın harf farklı adıyla yeni doküman yaratılabilir; köken doğru kalır.
- **CLI'nin aktörü beyan edilir:** `mdk promote --actor human:<id>` komutu çalıştıran kişinin beyanıdır; CLI'yi çalıştırabilen her süreç insan beyan edebilir. Bu, token dosyası varsayımıyla aynı güven düzeyidir.
- **Bağlantı başvuru tanımları:** ajan kendi başlıklı bölümüne bir Markdown bağlantı başvuru tanımı (`[ad]: url`) ekleyebilir. Bu, aynı adı kullanan canonical bir bloğun görüntülenen bağlantı hedefini değiştirebilir. Bloğun metni, kökeni ve arama birimleri değişmez (`append_only` bunu ölçer). İstemciler, proposal bloklarından gelen başvuru tanımlarını canonical bloklara uygulamamalıdır.
- **Sunucu dışı eşzamanlı yazar:** sunucu çalışırken aynı vault'a CLI ile yazmak çekirdekte kilitlenmez (ASSUMPTIONS P1-D D1-3). Tek yazar kuyruğu yalnız sunucunun çağrılarını sıralar.
