# Başsız insan kanalı (P2-P/P3; K-Q.1 (7))

Obsidian'sız, başsız bir sunucuda insanın onayını uzaktan (ör. bir Telegram botuyla) iletmek için kurulum ve sınırlar. Bu belge örnek bot kodu içermez (kapsam dışı, GOAL P2-P 4); botun konuşacağı uçlar `docs/CLIENT_CONTRACT.md`'dedir. Tehdit modeli: `docs/THREAT_MODEL.md` "Başsız insan kanalı".

## İlke
- **Token'ı tutan süreç insandır.** HTTP token'ını okuyabilen her süreç `[identity] human_id` adına terfi ettirir, indirir, tasdik eder (K-D1.3). Bot bu yüzden insan kanalıdır ve ajanlardan bir OS sınırıyla ayrılır.
- **Bot karar vermez, iletir.** Uzaktaki insan bir öneriyi görür ve onaylar; bot bu onayı `promote` (ya da geri almayı `demote`) olarak iletir. Kendi başına onay üreten bir bot, insan kanalını ajana açar.
- **Kanal etiketi.** Bot her isteğe `X-Consumer: telegram-bot` (kendi adı) koyar; terfi ve indirme olayları onu `meta.source` olarak taşır (spec 2.0.3 §7). `history`, bir bloğun hangi kanaldan onaylandığını gösterir. Etiket insanı değil kanalı söyler: aktör yine `human:<human_id>`'dir.

## Topoloji
Bir vault'a tek bir yazar süreç: aynı vault'ta iki `rootmark serve` yazıyorsa `events.jsonl` bozulabilir (kilit yok; THREAT_MODEL P2-0.2 F5). Önerilen düzen:

| Hesap | Çalıştırdığı | Erişimi |
|---|---|---|
| `rmhuman` (insan kanalı, bot) | `rootmark serve --vault /srv/vault --mcp stdio --http 127.0.0.1:7357 --watch`, ve bot süreci | vault ve `.rootmark/` (sahibi); `.rootmark/server.token` (mod 600, yalnız bu hesap) |
| `agent` (ajanlar) | MCP istemcisi (ör. Claude Code, toplayıcı ajan) | vault'a ve `.rootmark/`'a erişimi **yok**; yalnız sunucuyu `rmhuman` olarak başlatan tek bir komut |

- MCP stdio taşımasında sunucuyu ajanın istemcisi başlatır. İstemcinin MCP komutu, sunucuyu `rmhuman` olarak çalıştıran sabit bir komuttur; ör. `sudoers`'ta yalnız bu komut satırına izin: `agent ALL=(rmhuman) NOPASSWD: /usr/local/bin/rootmark serve --vault /srv/vault --mcp stdio --http 127.0.0.1\:7357 --watch`, MCP yapılandırmasında `sudo -n -u rmhuman /usr/local/bin/rootmark serve --vault /srv/vault --mcp stdio --http 127.0.0.1:7357 --watch`. Böylece tek süreç hem ajanın MCP'sini hem botun HTTP'sini sunar; token ve vault dosyaları ajan hesabına kapalıdır. Sunucu MCP akışı kapanınca durur ("end of the MCP stream"): botun HTTP'si de ajanın oturumu sürdükçe açıktır; uzun ömürlü bir ajan oturumu (ya da onu yeniden başlatan bir süreç yöneticisi) gerekir. Bot bağlantı reddini "sunucu kapalı" olarak işler, onayı kuyrukta tutar.
- Ajan hesabının vault dosyalarına yazma izni olmaması, tasdiksiz dosya yazımı yolunu (spec §7.1) da kapatır: ajan yalnız `propose` ile yazar.
- `--http` yalnız `127.0.0.1`'e bağlanır (uzaktan erişim yok, P2-E'ye): bot sunucuyla aynı makinede çalışır ve dışarıyla (Telegram) yalnız kendisi konuşur.
- Bot süreci `rmhuman` hesabında çalışır; token'ı `/srv/vault/.rootmark/server.token`'dan okur (`[server] token_rotate_days` sonunda sunucu yeniler; bot her istekte dosyadan okumalıdır). Token dosyası depoya, bot yapılandırmasına ya da mesaja girmez.

## Kurulum
1. Hesaplar: `rmhuman` ve `agent` (ayrı, `agent` `rmhuman` grubunda **değil**). Vault dizini `rmhuman:rmhuman`, mod `0700`.
2. Tarif `/srv/vault/.rootmark/policy.toml` (ilk `rootmark serve` varsayılanı yazar; sonra düzenleyin):
   ```toml
   [identity]
   human_id = "ada"          # uzaktaki insanın kimliği; olayların aktörü human:ada
   [server]
   http_port = 7357
   ```
   İsteğe bağlı bildirimsel terfi kuralları (`[[promotion.rules]]`, CLIENT_CONTRACT §2.1): toplayıcı ajanın resmi kaynak önekine yazdığı içerik doğrudan `canonical` olur; geri kalan her şey insanın onayını bekler. `rootmark policy lint /srv/vault` tarifi denetler.
3. `sudoers` satırı ve ajanın MCP yapılandırması (yukarıda).
4. Bot: `rmhuman` hesabında, `X-Consumer: telegram-bot` ile `http://127.0.0.1:7357/v1/<araç>`'a istek atar; `Authorization: Bearer <token>`.

## Onay akışı
1. **Ne bekliyor?** Bot önerileri bulur: `POST /v1/search` `{"query": …, "include_proposal": true}` (etiketli `proposal` sonuçlar) ya da `POST /v1/blame` `{"doc_path": …}` (blok blok katman, köken). İnsana belge yolu, blok metni (`read_block`) ve kökeni (`source`: hangi ajan) gönderir.
2. **Kim yazdı, ne değişti?** `POST /v1/history` `{"lineage_id": …}`: soyun bütün olayları ve aktörleri.
3. **Onay:** insan "onayla" derse `POST /v1/promote` `{"lineage_id": …, "reason": "telegram: onaylandı"}`. Yanıt `decision: "promoted"` ve `Promoted` olayı (`meta.actor {kind: human, id: ada}`, `meta.source: "telegram-bot"`). Bir chunk'ı aramada görünür kılmak için chunk'ın bütün soyları terfi ettirilir (§2.4).
4. **Geri alma:** `POST /v1/demote` `{"lineage_id": …, "reason": …}` (CLIENT_CONTRACT §2.7): `canonical → proposal`; kuralla terfi etmiş içerik de böyle indirilir. Olay `meta.source: "telegram-bot"` taşır.
5. **Ret:** hiçbir şey yapmamak (öneri `proposal` kalır, varsayılan aramaya girmez). Budama P2-R'de.

Bot `X-Consumer`'ı her istekte göndermek zorundadır (sözleşme §1; yoksa 400 ve hiçbir şey yazılmaz). Aynı makinede terminalden onay veren insan `rootmark promote <vault> <lineage> --actor human:ada --reason … --source terminal` kullanabilir (etiket isteğe bağlıdır).

## Sahne: toplayıcı ajan → otomatik canonical → `trace` (P2-P/P4)
`server/tests/scene_collector.rs`'in koşturduğu akış (deterministik, iki koşturmada bayt eşit):
1. Tarif bir kural taşır: `id = "official"`, `actor = "agent:collector"`, `source = "collector"`, `path_prefix = "sources/official/"`, `[promotion] version`.
2. Toplayıcı ajan MCP ile beş `propose` yapar: üçü `sources/official/` altında (her biri, kendi soyları için birer `Promoted`, `meta.actor = policy:official`), ikisi önek dışında (`proposal` kalır).
3. Varsayılan `search` önek içi üç belgeyi `canonical` döndürür; önek dışı iki belge yalnız `include_proposal` ile, `proposal` etiketiyle.
4. Bir brif üreten tüketici aradığında `trace` kaydı dönen `canonical` blokları listeler: cevabın hangi resmi bloklara dayandığı.
5. `history` her terfi olayında kural kimliğini ve `policy_version`'ı gösterir.
6. İnsan (bot ya da terminal) bir belgeyi `demote` eder: aramada yeniden yalnız `include_proposal` ile görünür.
7. `checkpoint`: o anın hali adlandırılır; aynı sahne iki kez koşturulunca `events_hash` ve `trace` baytları eşittir.
Aynı sahne CLI ile: `rootmark propose <vault> <doc> <file> --source collector` (CLI'de aktör `agent:<--source>`).

## Sınırlar
- **Tek `human_id`:** sunucu başına bir insan. Birden çok insan aynı bot üzerinden onaylıyorsa olaylar onları ayırmaz; `meta.source` kanalı söyler, insanı değil. Çoklu kimlik P2-E'de.
- **Bot insan kanalıdır:** botun hesabını ele geçiren ya da token'ı okuyabilen her süreç insan adına onaylar. Ajanlar o hesaba ve token'a erişemez olmalıdır (yukarıdaki topoloji); aynı hesapta çalışan bir ajan bu ayrımı kaldırır.
- **Etiket bir beyandır:** `X-Consumer` (ve `--source`) çağıranın beyanıdır; token sahibi istediği etiketi yazabilir. Etiket denetim izi içindir, yetki değildir.
- **Kuralın ajan adı da bir beyandır:** `[[promotion.rules]]`'un `actor`'ü MCP istemcisinin kendi bildirdiği addır (THREAT_MODEL "Bildirimsel terfi kuralları"). Yukarıdaki topolojide sunucuyu başlatabilen tek hesap `agent`'tır; o hesaptaki her ajan bir kuralın adını beyan edebilir. Kuralın önekini dar tutun.
- **Uzaktan erişim yok:** HTTP yalnız `127.0.0.1`; bot ve sunucu aynı makinede.
- **Ölçülmedi:** bu belge bir işletim önerisidir; çok hesaplı kurulum bu turda uçtan uca koşturulmadı (GOAL P2-P 4: belge + küçük sözleşme eki). Kanal etiketi ve `demote` ucu testlidir (`server/tests/p2_p_channel.rs`, `server/tests/promotion_rules.rs`).
