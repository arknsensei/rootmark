# P2-B — İstemci Sözleşmesi v2: Blok Konumu, Olay Akışı, Zaman Filtresi, İkinci İstemci (Geliştirme Turu, Blok 2)

Başlangıç: Blok 1 (P2-L + P2-P) kapandı, bağımsız doğrulama "güvenilir" (`reports/rounds/P2-P/VERIFICATION.md`, K-P2P.6). Bu tur K-P2.4 dozuyla bir **geliştirme turu**dur ve **Blok 2'yi açar** (P2-B + P2-R). Tam çift ve bağımsız doğrulama Blok 2'nin sonunda, P2-R bitince yapılır. Girdi: ROADMAP §2 P2-B satırı, K-Q.1 (3b) ve (4), Blok 1 doğrulamasının önerileri 1 ve 3, P2-P gözden geçiricisinin bıraktığı düşük bulgular. Hedef süre 1–2 gün.

Sorun: `blame` blokları belge sırasıyla verir ama **konum taşımaz** (CLIENT_CONTRACT §2.5). Obsidian eklentisi blokları editörün kendi bölüm haritasıyla yalnız sayılar tuttuğunda eşler (`clients/obsidian/src/align.ts`). HTML, dipnot, callout ve gömme içeren gerçek notlarda sayılar tutmaz ve panel "hizalama yok" der. Çekirdek konumu zaten bilir (`BlockDraft.line_start/line_end`), ama sözleşmeye çıkmaz.

## Kapsam
1. **B0 — Açılış ve Blok 1 kalıntıları.**
   - (a) Doğrulama önerisi 1: arama filtresinin NFD öneki NFC'ye katlaması **korunur** ve CLIENT_CONTRACT'ta (arama `path_prefix`) açıkça yazılır. Kural öneki yüklemede NFD'yi reddetmeye devam eder. Gerekçe: kural bir yetkidir, arama bir okumadır (K-P2B.1). İki davranış birer testle sabitlenir.
   - (b) Doğrulama önerisi 3:
     - `migrate --dry-run` planında `policy_attestation_1` `required: true` görünmez; yalnız `recommended: true` (gözden geçirici bulgusu 7).
     - CHANGELOG'a 0.2.0'a geri dönüş satırı: kurallı bir tarif 0.2.0'da açılmaz, `[[promotion.rules]]` önce kaldırılır.
   - (c) P2-P gözden geçiricisinin bıraktığı bulgular:
     - (5) Okuma kuralı, `policy` aktörlü `Promoted`'da `reason` içindeki `rule:<id>` ile `meta.actor.id`'nin eşitliğini ve `meta.from = "proposal"` olmasını denetler; aksi okuma hatasıdır. Spec metni değişmez; başvuru uygulamasının kuralıdır (ASSUMPTIONS P1-4 emsali).
     - (8) `status.migrations.artifacts.policy` `unknown` durumu ya ulaşılabilir olur ya da tanımdan çıkar. Okunamayan tarif hatadır; sessiz `unknown` yoktur.
     - (11) Alınmamış dış değişiklikten sonra eşleşen `propose` yolu için test.
2. **B1 — Blok konumu sözleşmede.**
   - `blame`, `replay` ve `search` isabetleri her blok için `loc` taşır:
     - `line_start`, `line_end`: 1 tabanlı, kapsayıcı, ön-bilgi dahil bütün belgede sayılır; çekirdeğin mevcut tanımı.
     - `byte_start`, `byte_end`: dosya baytlarında UTF-8 bayt ofseti, yarı açık aralık.
     - `content_hash`: konumun ait olduğu belge sürümünün (dosya baytlarının) hash'i.
   - Konum **türetilmiş** bilgidir:
     - Olaylara, kimliklere ya da `events_hash`'e girmez.
     - Spec §1–6 değişmez. Spec'e en çok §9'a bilgilendirici bir not girer (CRITERION ile, gerekiyorsa).
     - md'ye bir şey yazılmaz.
   - **Bayat dosya:**
     - Dosyanın diskteki baytları son alınan sürümden farklıysa yanıt `loc_stale: true` taşır ve konumlar alınan sürüme aittir. İstemci `content_hash` ile karşılaştırır.
     - Sessiz yanlış ofset yasaktır.
   - CLIENT_CONTRACT sürüm 2 olur. Ek alanlar isteğe bağlıdır; v1 istemcisi bozulmaz. `scripts/gen_client_types.sh` tipleri üretir.
3. **B2 — Hizalama ölçümü (ön-kayıtlı, küçük ve deterministik; K-P2.4 (1)).**
   - Korpus `research/data/d_notes`:
     - Açık lisanslı (CC0, CC-BY ya da MIT), kamuya açık gerçek not depolarından seçilir; kaynak, revizyon, lisans ve sha256 manifestte.
     - ≥ 300 not. HTML bloğu, dipnot, callout (`> [!note]`), gömme (`![[…]]`), tablo, iç içe liste, kod çiti, ön-bilgi ve CRLF sınıflarının her biri ≥ 20 örnek.
     - ≥ 30 Türkçe not.
     - Ayrıca sabit tohumla üretilen ≥ 200 kenar vakası (sınıf başına).
   - Seçim betiği, tohum ve manifest ölçümden önce commit'lenir.
   - Ölçüt (her blok için):
     - **(i) Dilim:** `file[byte_start..byte_end]` yeniden ayrıştırılınca aynı `normalized_text`'i verir. Satır aralığı bayt aralığıyla tutarlıdır.
     - **(ii) Örtü:** bloklar sıralı ve örtüşmez. Bloklar arası boşluklar yalnız boş satır, ön-bilgi ya da spec §2.2'nin blok saymadığı içeriktir; bu sınıflandırma ayrıca raporlanır.
     - **Hizalama hatası** = (i) ya da (ii)'yi bozan blok / bütün bloklar.
   - Karşılaştırma için eski yöntemin (sıra eşleme) eşleyebildiği belge oranı da raporlanır; eşiği yoktur.
4. **B3 — Olay akışı ve zaman filtresi (K-Q.1 (4), (3b)).**
   - (a) `events {since_seq, kinds?, path_prefix?, limit}`:
     - Salt okunur, HTTP ve MCP.
     - `since_seq`'ten sonraki olayları seq sırasıyla, `events.jsonl` satırlarıyla bayt eşit nesneler olarak döner. `limit` üst sınırlıdır; yanıt `next_seq` taşır.
     - Filtre yalnız süzer, sırayı değiştirmez. `path_prefix` B0 (a)'nın arama kuralıyla aynıdır.
     - Hiçbir şey yazmaz: `events_hash`, `traces.jsonl` dışındaki dosyalar ve iz satırı aynen. İz yazılıp yazılmayacağı CLIENT_CONTRACT'ta tek cümle.
     - İtme (abonelik, websocket) yok; çekme yeterli.
   - (b) Arama zaman filtresi `date_from`, `date_to` (kapsayıcı, `YYYY-MM-DD`):
     - Belgenin tarihi yalnız iki kaynaktan türetilir: `[index] date_from_path` (yol deseni, ör. `"{yyyy}/{mm}/{dd}"`) ya da `[index] date_field` (ön-bilgi anahtarı).
     - Tarih kimliklere girmez, md'ye yazılmaz, dosya zamanına (mtime) **bakılmaz**.
     - Tarihi olmayan belge filtre varken dışarıda kalır; yanıt bu sayıyı taşır.
     - Filtre `trace`'e işlenir. Filtresiz `query_hash` değişmez.
     - Geçersiz tarif (`date_from_path` deseni bozuk, iki kaynak birden) yüklemede hatadır.
5. **B4 — İki istemci aynı sözleşmeyle.**
   - (a) Obsidian eklentisi:
     - `loc` varsa hizalamayı ondan alır (`align.ts`). `loc` yoksa (v1 sunucu) eski sıra eşlemesi kalır. `loc_stale` iken panel "bayat" der ve satır tahmin etmez.
     - `proposal` ve `unverified` blokları editörde ayrı işaretlenir (bugünkü blame oluğunun konumlu hâli).
   - (b) `clients/vscode`: VS Code eklenti iskeleti, TypeScript, HTTP taşıması.
     - Yapar: `status`; `search` (filtrelerle); açık belgenin `blame`'i, `loc` ile satır süslemesi (katman başına); `history`; insan `promote` ve `demote` komutları (`X-Consumer: vscode`).
     - **Kaydetmeleri tasdik etmez.** VS Code kaydı tasdiksiz dış değişikliktir; bu `HEADLESS_CHANNEL.md` benzeri bir notla `clients/vscode/README.md`'de yazılır. Tasdik ayrı bir turun işidir.
     - Pazar yerine yayım yok.
   - (c) Sözleşme denetimi:
     - İki istemcinin test ve e2e koşularındaki **bütün** HTTP ve MCP çağrıları kaydedilir ve CLIENT_CONTRACT v2'den üretilen şemaya karşı denetlenir: bilinmeyen uç ya da araç, bilinmeyen ya da eksik alan, yanlış tip, eksik başlık.
     - Ortak tipler tek kaynaktan gelir (`contract.gen.ts`, iki istemci aynı dosyayı kullanır).
6. **B5 — Zincir, rapor, hazır.**
   - `make verify FAST=1` temiz; tam `make verify` bu turda **bir kez**.
   - FINAL; READY; tek PR.
   - Blok 2 sonu doğrulama yönergesinin P2-B maddeleri `VERIFY.md`'de (doğrulama P2-R'den sonra).

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| B-1 | B0: (a) iki davranış testli ve belgeli; (b) `--dry-run` planı ve CHANGELOG satırı; (c) bulgular 5, 8, 11 kodlandı ve testli (5 ve 8'in hata yolları testle) | hepsi |
| B-2 | B1: `blame`/`replay`/`search` `loc` taşır; `loc_stale` testli (dosya alınmadan değişince); v1 alanları aynen; kuralsız ve varsayılan tarifle `events_hash`, C3-06 ve 39/39 aynen; CLIENT_CONTRACT v2 ve üretilmiş tipler | hepsi; aynen |
| B-3 | B2: hizalama hatası D-NOTES'ta (gerçek notlar) | ≤ 0.005 |
| B-4 | B2: hizalama hatası kenar vakalarında, sınıf başına | ≤ 0.02 her sınıfta; raporlanır |
| B-5 | B3 (a): `events` yanıtı `events.jsonl` ile bayt eşit; filtre yalnız süzer; `since_seq` sınırları (0, baş, baştan büyük) testli; koşu sonrası `events_hash` ve olay dosyası aynen | hepsi |
| B-6 | B3 (b): yol ve ön-bilgi kaynaklı tarih testli; mtime'a bakılmadığı testli; `trace` filtreyi taşır; filtresiz `query_hash` aynen; geçersiz tarif hatası alanı adlandırır | hepsi |
| B-7 | B4: iki istemcinin testleri `make test`'te; Obsidian eklentisi `loc` ile hizalar (B2 korpusundan ≥ 20 not ile e2e); VS Code iskeleti derlenir ve testleri geçer | hepsi |
| B-8 | B4 (c): sözleşme dışı çağrı, iki istemcinin bütün koşularında | 0 |
| B-9 | Determinizm: `loc`, `events` ve zaman filtresi iki koşturmada bayt eşit (FAST çifti); tam çift bir kez fark 0 | eşit |
| B-10 | `make verify FAST=1` ≤ 20 dk; tam `make verify` bir kez temiz; `check_frozen`, `check_deps`, `check_dep_age` (VS Code iskeletinin yeni bağımlılıkları dahil) | temiz |

## Karar kuralı
- **Hepsi geçti** → P2-R başlar (Blok 2 sürer). Bağımsız doğrulama Blok 2 sonunda yapılır.
- **B-3 ya da B-4 kaldı** (geri kalan geçti) → `loc` sözleşmede "deneysel" işaretiyle kalır, ölçülen hata sınıf başına CLIENT_CONTRACT'a yazılır; Obsidian eklentisi `loc`'u yalnız dilim denetimi (`content_hash` + yeniden ayrıştırma) geçerse kullanır. Karar talebi açılır; P2-R başlar.
- **B-6 kaldı** (geri kalan geçti) → zaman filtresi P2-R'ye taşınır; tur kapanır; karar talebi.
- **B-7'nin VS Code kısmı kaldı** (geri kalan geçti) → iskelet sonraki tura taşınır; B-8 yalnız Obsidian eklentisiyle ölçülmüş sayılır ve FINAL bunu yazar; karar talebi; P2-R başlar.
- **B-2, B-5, B-8 ya da B-9 kaldı** → tur kaldı; karar talebi; P2-R başlamaz.
- **B-1 ya da B-10 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Spec §1–6.
- Konumun olaylara, kimliklere ya da `events_hash`'e girmesi.
- md'ye yazan kod.
- Tarihin dosya zamanından türetilmesi.
- Olay akışında itme, abonelik ya da yazma.
- Kayıtlı sorgular (R-2 ile).
- VS Code kaydının tasdiki.
- Pazar yeri yayımı.
- Varsayılan tokenizer ya da embedder değişikliği.
- Kural listesinin genişlemesi (joker, regex, zaman koşulu).
- Uzak erişim, çoklu `human_id`, ajan kimlik doğrulaması (P2-E).
- Budama (P2-R).
- Dondurulmuş ölçüm kodunun istisnasız değişmesi.
- Yayım.
