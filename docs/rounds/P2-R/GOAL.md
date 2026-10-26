# P2-R — Saklama ve Budama (Geliştirme Turu, Blok 2'nin son turu)

Başlangıç: P2-B kapandı (`reports/rounds/P2-B/FINAL.md`: B-3 kaldı 0.00551, geri kalanı geçti; karar kuralının "B-3 kaldı" dalı: `loc` deneysel, K-P2B-T1; K-P2B.5). Bu tur K-P2.4 dozuyla bir **geliştirme turu**dur ve **Blok 2'yi kapatır** (P2-B + P2-R). Tam çift bu turda bir kez; bağımsız doğrulama bu turdan sonra, tek seferde, dar kapsamla (`PROMPT.md`'deki doğrulayıcı istemi: P2-B `VERIFY.md` VB1–VB9 + bu turun `VERIFY.md` VR1–VR7; çıktı `reports/rounds/P2-R/VERIFICATION.md`). Girdi: ROADMAP §2 P2-R satırı, K-Q.1 (5), K19-T, spec §7, §8, §11; K-P2B-T1 kararı; K-P2B.4 (kaydedicilerin 11 araca uyarlanması önerisi); P2-B gözden geçiricisinin bıraktığı bulgular 5, 8 (`search`), 9, 10, 11. Hedef süre 1–2 gün.

Sorun: `.rootmark` sınırsız büyür. Her `search` ve `read_block` bir iz satırı yazar (`traces.jsonl`), her değişiklikten sonraki ilk sorgu bir checkpoint nesnesi bırakır (`checkpoints/`), her alım belge ve sürüm başına bir konum kaydı yazar (`loc/`, P2-B; gözden geçirici bulgusu 10), ham günlük (`journal/`) birikir (budaması var, 24 saat alt sınır). Proposal soyları süresiz `proposal` kalır: sönümleme yolu çekirdekte var (`prune_inactive`, `Demoted` + `meta.prune`, K19-T) ama tarifi, sunucu adımı, sözleşmesi ve ölçümü yok. Blok 1 ve 2'nin bütün vaatleri (`events_hash`, checkpoint'te aynı yanıt, iz kimliği) bu dosyaların üstüne kuruludur; budama bu vaatleri bozmadan hacmi sınırlamalıdır.

## İlkeler (bu turun değişmezleri)
- **rootmark yönetişim kaydı silmez.** Proposal budaması bir **olaydır** (`Demoted`, `meta.prune: true`, katman aynen; K19-T). İz ve checkpoint budaması **arşive taşımadır** (bayt bayt, aynı sıra); arşivi yalnız insan siler. Tek silme istisnası türetilmiş konum kayıtlarıdır (`loc/`; olaylara ve kimliklere girmez, P2-B).
- `events.jsonl`, `objects/`, `snapshots/`, `attestations.jsonl` **asla** budanmaz, taşınmaz, yeniden yazılmaz. `events_hash` yalnız proposal budamasının eklediği `Demoted` olaylarıyla değişir; kuralsız tarifle aynen.
- **Determinizm:** budama kararı yalnız `seq`, checkpoint sırası ve tarif parametrelerinden türer; duvar saatine bakılmaz (mevcut `journal prune`'un açık `now_ms` argümanı aynen kalır, değişmez). Aynı vault + aynı tarif + aynı çağrı → bayt eşit canlı dosyalar, arşiv dosyaları ve çıktı.
- **Budama yalnız açık çağrıyla koşar:** CLI `rootmark prune` ve HTTP `POST /v1/prune` (`[identity] human_id`; başsız insan kanalı). MCP'de budama aracı yoktur: ajan budamaz. `rootmark serve` kendiliğinden budamaz (kapanışta, izleyicide, sorguda yok).
- **Çekirdek karar vermez:** sönümleme kuralı tarifte, eşleyici `server`'da, geçiş çekirdeğin kancasından (P2-P'nin deseni; `canonical` hiçbir kuralla budanmaz).
- Sessiz başarısızlık yok: arşivlenmiş bir şeye başvuran çağrı açık bir kodla yanıtlanır.
- NORTH_STAR 8 (ölçek sınırı) ve spec §1–6 değişmez.

## Kapsam
1. **R0 — Açılış ve P2-B kalıntıları.**
   - (a) **K-P2B-T1 (a)** — *bu madde insan kararına bağlıdır; tur başlamadan kesinleşir ve sonra değişmez. Karar (a) değilse R0 (a) yapılmaz, R-3 "ölçülmedi" yazılır ve kriter sayılmaz; FINAL bunu yazar.* `core/src/parse.rs`: girintili kod bloğunda konumun `byte_start`'ı satır başından başlar. Değişiklik yalnız konum hesabındadır: `normalized_text`, kimlikler, olay baytları, `events_hash` aynen (39/39, C3-06, P2-A'nın altı eski vault'u). D-NOTES **aynı korpus, aynı manifest, aynı tohum ve aynı dondurulmuş ölçücüyle** (`research/eval/p2b_align.py` değişmez) yeniden ölçülür; iki koşu bayt eşit; sonuç R-3.
   - (b) **K-P2B.4'ün önerisi:** dondurulmuş `scripts/contract_check.sh` ve `scripts/mcp_e2e.sh` (D-2) istisna prosedürüyle 11 araca uyarlanır (önce/sonra; ölçülen şey gevşemez: her araç ve uç sözleşmede, kurcalamaların reddi aynen). K-P2B.4'ün `historical_fails_p2b.json` ve `migration_required_fails_p2b.json` girdileri (D1-13, iki tamper, E-16, D-2, D1-12'nin yolları, E1-8, E1-9'un `contract_check_pass` yolu) listeden çıkar; sınıflandırıcı kodu değişmez; sayılan FAIL 0.
   - (c) P2-B gözden geçiricisinin bulguları:
     - (5) Zincir hizalama ölçütünü denetler: `p2b-checks`'in `align.json`'u referans değerle (R0 (a) uygulandıysa R-3'ün resmi değeri, değilse `reports/rounds/P2-B/b2/align.json`) karşılaştırılır; gerileme (gerçek notlarda ya da bir sınıfta artış) zinciri kırmızı yapar.
     - (8) `search`'te konum izden önce hesaplanır (`read_block`'ta yapıldı).
     - (9) `events`: okuma `since_seq`'ten başlar (seq→ofset indeksi ya da satır sayacı); seçici filtrede okuma kilidi altında bütün günlük ayrıştırılmaz; `limit` dolunca durur. Yanıt baytları aynen (B-5 testleri geçer).
     - (11) `commit_plan`'ın snapshot kimliği denetimi olaydan önce.
2. **R1 — Proposal sönümleme (K19-T, K-Q.1 (5)).**
   - `policy.toml` `[[prune.rules]]`: `id`, `layer` (`proposal` ya da `unverified`), `idle_checkpoints` (tam sayı, çekirdek sınırı ≥ 1; sınır dışı → yükleme hatası, kırpma yok), isteğe bağlı `path_prefix` (liste; P2-B'nin kural kuralı: NFC, NFD ret), `[prune] version`.
   - Kural: soyun son olayından sonra en az `idle_checkpoints` checkpoint oluşmuşsa ve soy hâlâ o katmanda ve etkinse → `Demoted` `meta.prune: true`, katman aynen, `meta.actor` `policy:<id>`, `reason` `rule:<id>`, `policy_version`, `from = to = layer`. `canonical` asla. Eşleyici `server`'da, yalnız `prune` çağrısında; geçiş `Vault::prune_inactive` (kanca) ile.
   - Okuma kuralı (K-P2B.2 (c5)'in deseni): `policy` aktörlü, `meta.prune` taşıyan `Demoted`'ın `reason`'ı `rule:<meta.actor.id>`; aksi okuma hatası. Spec metni değişmez.
   - Budanmış soy aramada görünmez (etkin değil; mevcut davranış), `read_block` `inactive: true` ile okur, budamadan önceki bir checkpoint'te `at` ile aynen okunur ("budanan içerik checkpoint'ten okunur").
   - Kuralsız tarif: hiçbir olay, `events_hash` aynen.
3. **R2 — İz, checkpoint ve konum kaydı saklaması; `prune` komutu.**
   - `policy.toml` `[retention] keep_checkpoints = K` (çekirdek sınırı ≥ 1; tablosuz tarif: saklama sınırsız, `prune` iz/checkpoint için no-op ve bunu söyler).
   - **Sınır:** oluşturma sırasıyla son K checkpoint'in en eskisi; onun `event_seq`'inden önceki durumda yazılmış izler (ilk canlı izin `at_seq`'i = `base`) sınırın gerisindedir. Adlı checkpoint'ler sayıma girer ama **asla arşivlenmez**.
   - **İz arşivi:** sınırın gerisindeki `traces.jsonl` satırları bayt bayt, aynı sırayla `.rootmark/traces/archive/<ilk at_seq>-<son at_seq>.jsonl`'e taşınır. Kalan satırların `at_seq`'i mutlak kalır (`base` kaydı: `.rootmark/traces/base` ya da eşdeğeri; okuyucu `at_seq = base + satır`), `trace_id`'leri değişmez, yeni izler eskisi gibi eklenir (K18-T aynen). `trace` aracı `since_seq < base`'de yanıtta `archived_before: <base>` taşır (sessiz değil); CLI `rootmark trace --archived` arşivi okur. Canlı + arşiv birleştirilince eski dosya bayt bayt geri gelir (R-5).
   - **Checkpoint arşivi:** adsız, hiçbir canlı iz tarafından başvurulmayan ve sınırın gerisindeki checkpoint nesneleri `.rootmark/checkpoints/archive/`'e taşınır. `checkpoints` listesi onları `archived: true` ile gösterir; `at` ile arşivli checkpoint'e sorgu `archived` koduyla reddedilir ve arşiv dosyasını adlandırır. Checkpoint kimliği değişmez.
   - **Konum kaydı:** belge başına son alınan sürümün kaydı ve canlı checkpoint'lerde başvurulan sürümlerin kayıtları kalır; gerisi silinir (türetilmiş; P2-B: kaydı olmayan sürümde `loc` hiç yok, sessiz yanlış ofset yok).
   - **Ham günlük:** mevcut `journal prune` aynen; `rootmark prune` onu da çağırır (`--now-ms` açık argüman; zincirde sabit).
   - `rootmark prune` (CLI): `--dry-run` planı (taşınacak satır ve nesne sayıları, budanacak soylar, silinecek konum kayıtları; hiçbir dosya değişmez) ve uygulama; çıktı JSON, bayt bayt belirlenimli. HTTP `POST /v1/prune {dry_run}`: `human_id` ister, yanıt aynı JSON, `X-Consumer` kanal etiketi (`HEADLESS_CHANNEL.md`: dry-run → onay → prune). MCP'de yok. Tek yazar kuyruğundan geçer (CLIENT_CONTRACT §5).
   - Spec: §8'in "append-only"ına yalnız bilgilendirici bir not (CRITERION ile, 2.0.4): başvuru uygulaması iz satırlarını saklama sınırının gerisinde arşive taşıyabilir, `at_seq` mutlaktır. §1–6 değişmez.
4. **R3 — Hacim ölçümü (ön-kayıtlı, küçük ve deterministik; K-P2.4 (1)).**
   - Senaryo D-VOL: `research/eval/p2r_volume.py`, sabit tohum, sentetik vault (CLI ve HTTP ile, gerçek sunucu): 200 belge; 40 tur, her turda 25 belgede düzenleme + 10 `propose`; her turdan sonra 50 `search` + 10 `read_block` ve bir `checkpoint`; 10 soy `canonical`'a terfi eder (asla budanmamalı). Tarif: `keep_checkpoints = 5`, `[[prune.rules]]` `proposal` `idle_checkpoints = 10`. Betik ve tarif ölçümden önce commit'lenir.
   - Hacim `V` = `traces.jsonl` + `checkpoints/` (canlı) + `loc/` + `journal/` baytları. `events.jsonl`, `objects/`, `snapshots/`, `attestations.jsonl` ve arşiv ayrıca raporlanır (budanmaz).
   - Ölçümler: `V_before` (40 tur sonu), `V_after` (`prune`), ikinci evre: 10 tur daha + `prune` → `V_after2`; budanan soy sayısı; arşive taşınan satır ve nesne sayısı; canlı + arşiv iz baytları = `V_before`'un iz baytları.
   - Değerler `research/reports/runs/<RUN>/p2r/volume.json`; FAST çiftinde iki koşturma bayt eşit.
5. **R4 — Sözleşme, belgeler, güvenlik.**
   - CLIENT_CONTRACT: yeni §2.10 (`prune`, saklama, arşiv, `archived_before`, `archived` kodu), `status.retention {keep_checkpoints, traces_base_at_seq, archived_traces, archived_checkpoints, rules[]}` (isteğe bağlı), `checkpoints[].archived`; §3 yeni kod; sürüm 2'nin eki (yalnız ek alan ve yeni uç; v2 istemcisi bozulmaz). `scripts/gen_client_types.sh`; iki istemcinin sözleşme denetleyicisi yeni ucu ve alanları tanır (istemciler özellik eklemez).
   - THREAT_MODEL: budama operatör eylemidir (CLI ya da insan kanalı); arşiv dizini vault'un içindedir ve `.rootmark` yazma yetkisiyle aynı güven düzeyindedir; `dry_run` yan etkisizdir. HEADLESS_CHANNEL: `prune` akışı. ARCHITECTURE: `on_prune` kancası gerçekleşti. CHANGELOG.
6. **R5 — Zincir, rapor, hazır.**
   - `make p2r-checks` (kriter testleri, D-VOL, determinizm kaydı); `verify.sh` P2-R adımı ve `compare_p2r` (dondurma istisnası, önce/sonra); R0 (b)'nin kaydedici uyarlaması.
   - `make verify FAST=1` temiz; tam `make verify` bu turda **bir kez**.
   - Gözden geçirici tek tur (R5), güvenlik başlığı: R1 (kuralın yanlış soyu budaması, `canonical`), R2 (arşiv yolu, `base` kurcalaması, `at_seq` tutarlılığı, HTTP `prune` yetkisi).
   - FINAL; READY; tek PR. Blok 2 sonu doğrulayıcısı istemi `PROMPT.md`'de hazırdır.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| R-1 | R0: (a) [yalnız K-P2B-T1 (a) ise] konum düzeltmesiyle kimlikler, olaylar, `normalized_text`, `events_hash` aynen (39/39, C3-06, altı eski vault); (b) kaydediciler 11 araçla, K-P2B.4 girdileri listeden çıktı, sınıflandırıcı kodu aynen, sayılan FAIL 0; (c) bulgular 5, 8, 9, 11 kodlandı ve testli | hepsi |
| R-2 | R1: kurallı tarifle sönümleme `Demoted` + `meta.prune` (katman aynen, `policy:<id>`, `rule:<id>`, `policy_version`); `canonical` asla; `unverified` yalnız kuralı varsa; kuralsız tarifle hiçbir olay ve `events_hash` aynen; sınır dışı parametre ve NFD önek yüklemede ret; okuma kuralı testli | hepsi |
| R-3 | R0 (a): hizalama hatası D-NOTES (aynı korpus, tohum, dondurulmuş ölçücü), gerçek notlar | ≤ 0.005 (K-P2B-T1 (a) değilse ölçülmez, kriter sayılmaz) |
| R-4 | Budanmış proposal: aramada yok; `read_block` `inactive: true`; budamadan önceki checkpoint'te `at` ile aynen; `history` olayı gösterir | hepsi |
| R-5 | İz ve checkpoint arşivi: kalan izlerin `at_seq` ve `trace_id` aynen, yeni iz aynı kuralla eklenir; canlı + arşiv = eski dosya bayt bayt; `since_seq < base` → `archived_before`; arşivli checkpoint `at`'te `archived` hatası; adlı checkpoint asla arşivlenmez; canlı izin başvurduğu checkpoint asla arşivlenmez | hepsi |
| R-6 | Budama determinizmi: aynı vault + tarif + çağrı → canlı dosyalar, arşiv ve `prune` çıktısı bayt eşit (FAST çifti, `p2r/volume.json` ve `p2r/det.json`); tam çift bir kez fark 0 | eşit |
| R-7 | D-VOL: (a) `V_after / V_before`; (b) `V_after2 / V_after` | (a) ≤ 0.35; (b) ≤ 1.10 |
| R-8 | Budama yalnız açık çağrıyla: `serve` kendiliğinden budamaz (uzun oturum testi: dosyalar aynen); MCP `tools/list`'te `prune` yok; HTTP `prune` `human_id` ister (yoksa 403); `dry_run` hiçbir dosyayı değiştirmez (hash'ler aynen) | hepsi |
| R-9 | CLIENT_CONTRACT §2.10 ve `status.retention` yazılı, tipler üretilmiş (`--check`), THREAT_MODEL/HEADLESS_CHANNEL/ARCHITECTURE notları; iki istemcinin sözleşme denetleyicisinde sözleşme dışı çağrı 0 (P2-B B-8 kaydı yeniden koşar) | hepsi; 0 |
| R-10 | `make verify FAST=1` ≤ 20 dk; tam `make verify` bir kez temiz; `check_frozen`, `check_deps`, `check_dep_age` | temiz |

## Karar kuralı
- **Hepsi geçti** → Blok 2 kapanır; bağımsız doğrulama başlar (`PROMPT.md`'deki istem; `reports/rounds/P2-R/VERIFICATION.md`). P2-C doğrulamadan sonra açılır.
- **R-3 kaldı** (geri kalan geçti) → konum düzeltmesi geri alınmaz (kimlikler değişmedi); `loc` deneysel kalır, ölçülen yeni değer CLIENT_CONTRACT §2.5'e yazılır; K-P2B-T1 (b)/(c) için karar talebi; Blok 2 kapanır, doğrulama başlar.
- **R-7 kaldı** (geri kalan geçti) → saklama "deneysel" işaretiyle kalır, ölçülen oranlar CLIENT_CONTRACT §2.10'a yazılır; optimize edilmez; karar talebi; Blok 2 kapanır, doğrulama başlar.
- **R-3 ve R-7 birlikte kaldı** → iki dal birden uygulanır; doğrulama başlar.
- **R-2, R-4, R-5, R-6 ya da R-8 kaldı** → tur kaldı; karar talebi; doğrulama başlamaz (P2-B'nin VB maddeleri bekler).
- **R-1, R-9 ya da R-10 kaldı** → tur kaldı; karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
- Spec §1–6; §8'e yalnız bilgilendirici not.
- `events.jsonl`, `objects/`, `snapshots/`, `attestations.jsonl`'in silinmesi, taşınması ya da yeniden yazılması.
- Arşivlenen iz ya da checkpoint satırlarının değiştirilmesi; arşivin rootmark tarafından silinmesi.
- Duvar saatine dayalı budama kararı (journal'ın mevcut açık `now_ms`'i hariç).
- `serve`'de kendiliğinden budama; MCP'de `prune`.
- `canonical` budaması; kuralsız tarifle herhangi bir olay.
- Kural listesinin genişlemesi (joker, regex, zaman koşulu).
- İstemcilere özellik (Obsidian, VS Code); yalnız üretilmiş tipler ve denetleyici.
- D-set, gerçek Obsidian; ölçüm yalnız D-VOL ve R-3'ün yeniden ölçümü.
- Ölçek sınırı (NORTH_STAR 8) değişikliği.
- Dondurulmuş ölçüm kodunun istisnasız değişmesi; `p2b_align.py` ve D-NOTES manifesti hiç değişmez.
- Yayım; secret; NORTH_STAR.
