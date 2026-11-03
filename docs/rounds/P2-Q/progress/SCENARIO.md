# P2-Q — Senaryo (ön-kayıt; GOAL Kapsam 1, Q-1)

Makine okunur ön-kayıt: `research/p2q/scenario.json` (üreticisi `research/p2q/make_scenario.py`, tohum `20261103`; `--check` dosyanın anlık görüntülerden bayt bayt yeniden üretildiğini denetler). Bu belge onun okunur özetidir; çelişkide `scenario.json` kazanır. İkisi de Q0'da, Q2'nin ilk ölçümünden önce commit'lenir ve sonra değişmez. Kılavuzun (`docs/GUIDE.md`) metni ön-kayıt değildir; senaryonun adımları ve beklenen sonuçları öyledir.

## Korpuslar
| Ad | Tür | Kaynak | Dosya | Bayt | Özet blake2b | Tarif | Sahne |
|---|---|---|---|---|---|---|---|
| `qorbit` | kişisel | `~/Works/qorbit`'in `node_modules/` ve `.git/` dışındaki bütün `*.md`'leri | 111 | 1 565 500 | `f7bcd2c3709a6abef45e2483f1efea08` | `tr` | başsız |
| `obsidian` | kişisel | `~/Works/Obs` (`.trash/` hariç bütün vault; notlar `.obsidian/` dışındaki `*.md`) | 658 | 4 513 708 | `b0b17dfc45a5a966c0a1beea9ede7000` | `obsidian` | gerçek Obsidian |
| `dnotes` | açık | `research/data/d_notes/notes` (`NOTICE.md`) | 400 | 4 926 763 | `99282c64eec75fc90625fe7ba1eefd61` | `default` | başsız |

Kişisel korpuslar Q0'da bir kez salt okunur anlık görüntüye kopyalandı (depo dışında, turun çalışma dizini); her koşu o görüntünün taze kopyasında yürür (Mac, Docker: görüntü salt okunur bağlanır, konteyner içinde kopyalanır). Özet = `list_md` sırasıyla `"b2(yol) b2(bayt)\n"` satırlarının blake2b'si (`research/p2q/common.py`); içerik değil. Kişisel bir yol kayda yalnız `b2:<blake2b>` olarak girer.

## Tarifler (`scenario.json` `recipes`)
Üçü de: `[promotion] policy = "human_only"`, `version = "p2q-1"`, kural `toplayici` (`actor = "agent:toplayici"`, `path_prefix = "toplayici/"`, `to = "canonical"`); `[edit] append_only`; `[identity] human_id = "pilot"`; `[attestation] unattested = "unverified"`; `[server]`/`[index]` `serve`'ün yazdığı varsayılanlar. `tr`: ayrıca `[index] tokenizer = "uni_fold_tr"`, `embedder = "multilingual-minilm-l12-v2"`. `obsidian`: `tr` + `[server] http_port = 27357`. `default`: indeks varsayılanları (`mdk_lower_alnum`, `minilm-l6-v2`).

## Başsız sahne (qorbit ve dnotes; Q2 (b), (d); Q3)
Ajan MCP istemcisi `toplayici/1.0`; insan HTTP (token, `human:pilot`). Aynı adımlar Mac'te iki taze kopyada (determinizm), Docker'da (`linux/amd64`) ve — yalnız dnotes — CI'da.

| Adım | Eylem | Beklenen (makinece denetlenir) |
|---|---|---|
| H01 | kopya | `corpus_summary` kayıtlıya eşit |
| H02 | `ingest-dir` | çıkış 0; alınan dosya = korpus dosyası |
| H03 | tarif | `policy lint` çıkış 0, 1 kural |
| H04 | `index build` | çıkış 0; `index status` indekslenmemiş belge yok |
| H05 | `serve --mcp stdio --http 127.0.0.1:0 --watch` | HTTP `status`: `unverified`, `human_id` `pilot`, indeks `ok` |
| H06 | checkpoint `baslangic` | var |
| H07 | 100 MCP `propose` (50 `toplayici/` altında: toplayıcı notu, resmi kaynak özeti; 50 `yorum/` altında: LLM yorumu, brif; TR/EN dönüşümlü) | hepsi başarılı; `blame`: kural önekindekilerin her bloğu `canonical`, yorumların her bloğu `proposal` |
| H08 | her kural yazımından hemen sonra kodunun sözcüksel araması | bulunur, `canonical` (gecikme kaydı) |
| H09 | 20 doğrudan dosya yazımı (14 yeni `ajan-dogrudan/`, 6 tohumla seçilmiş korpus belgesine başlıklı ek) | izleyiciden sonra her yeni blok `unverified`; bu yazımlardan `canonical` **0** |
| H10 | checkpoint `ajan-yazdi` | var; evrenin sorgu sonuçları kaydedilir |
| H11 | 48 sorgu, evre `ajan-yazdi` | her sorgunun beklenen kümesi; varsayılan aramada `proposal`/`unverified` **0** |
| H12 | 12 yorum belgesinin bütün soylarına HTTP `promote` | hepsi `promoted` |
| H13 | 3 kural belgesinin bütün soylarına HTTP `demote` | hepsi `demoted` |
| H14 | checkpoint `insan-onayladi` | var |
| H15 | 48 sorgu, evre `son` | beklenen kümeler; varsayılan aramada `proposal`/`unverified` **0** |
| H16 | her sentetik belgenin `blame`'i | katmanlar evre `son`'un beklenenine eşit |
| H17 | 5 brif araması (MCP) ve `trace` | her iz kaydı var; dönen bloklar aramanın hit'lerine eşit |
| H18 | `at: ajan-yazdi` ile arama ve `blame` | H10/H11'de kaydedilenle eşit |
| H19 | `status` HTTP ve MCP | indeks, politika, tasdik, son checkpoint aynı |
| H20 | HTTP `prune {dry_run: true}` | başarılı; `events_hash` ve `.rootmark/` dosyaları değişmedi |
| H21 | `SIGTERM` | 60 s içinde çıkış 0; yeniden açılış: `status` ok, `events_hash` ve bir sorgunun sonucu aynı |
| H22 | sunucu durur, `rootmark index compact` | `events_hash`, indeks dosyaları, sorgu sonuçları (blake2b) kaydedilir |
| H23 | `observe.py` | salt okunur (öncesi/sonrası blake2b eşit); sayıları sürücününkiyle birebir |

**Sorgular (korpus başına 48; `scenario.json` `queries`):** 38 sözcüksel tam küme sorgusu (16 kural yazımı — üçü sonra indirilen —, 16 yorum — sekizi sonra terfi eden —, 6 doğrudan yazım; her üç sorgudan biri büyük harfle, harf katlama), 6 hibrit karışık dil sorgusu ("içerir": hedef varsayılan aramada yalnız `canonical` ise), 4 "beklenen yok" sözcüksel sorgu (0 hit). Her sorgu için iki evrede (`ajan-yazdi`, `son`) varsayılan ve `include_proposal` beklenen belge kümeleri ve hedefin katmanı yazılıdır. Sözcüksel tam küme = dönen belgelerin kümesi beklenene eşit; varsayılan aramanın her hit'i `canonical`.

**Ajan yazım şablonları:** `make_scenario.py` `TEMPLATES` (toplayıcı notu, resmi kaynak özeti, LLM yorumu, brif, doğrudan not, doğrudan ek; Türkçe ve İngilizce); konu/kaynak/sayı tohumdan; metin `scenario.json`'da açık (sentetiktir, kişisel değil). Determinizm karşılaştırması: iki taze kopyada `events_hash`, indeks dosyaları (vektörler, manifest) ve sorgu sonuçları bayt eşit; Mac ↔ Docker ↔ CI aynı ölçütlerle, sözcüksel skorların son ULP'si adıyla (K-P2V1.2).

## Obsidian sahnesi (Q2 (c))
Gerçek Obsidian kendi `--user-data-dir`'i ve CDP ile (P1-E.1/K-E1.3 yöntemi); eklenti 0.3.0 sürümünün zip'inden kılavuzun elle kurulum adımlarıyla. `serve --mcp stdio --http 127.0.0.1 --watch --attest-log <vault dışı>`; ajan MCP istemcisi `yardimci/1.0`.

| Adım | Eylem | Beklenen |
|---|---|---|
| O01–O04 | kopya, `ingest-dir` (süre), tarif `obsidian`, `serve` | kayıtlı özet; çıkış 0; lint 0 |
| O05 | Obsidian + eklenti (kılavuzla) | durum çubuğu `rootmark 0.3.0 · human:pilot` |
| O06 | 22 düzenleme Obsidian'ın düzenleyici API'siyle: 8 yeni not (`pilot/`), tohumla seçilmiş 8 nota ek, 2 değiştirme, 4 silme (ikisi yeni, ikisi tohumla seçilmiş) | her düzenlemenin bloklarının katmanı kaydedilir (beklenen `canonical`); yanlış `unverified`, `attest` hataları (403 dahil, gerekçeyle), izleyici gecikmesi **raporlanır** (eşik yok) |
| O07 | 2 yeniden adlandırma, bağlantı güncellemeli (gelen bağlantısı olan, tohumla seçilmiş notlar → `pilot-adlandirma-<n>`) | yeniden adlandırılan notun ve bağlantısı değişen notların katmanları raporlanır |
| O08 | 4 MCP önerisi (`ajan/oneri-<n>.md`) | `proposal` |
| O09 | Obsidian'dan (eklentinin komutları) 4 terfi, 2 geri alma | son katmanlar: `oneri-1`, `-2` `proposal`; `-3`, `-4` `canonical` |
| O10 | 42 sorgu (HTTP): 18 sözcüksel kod sorgusu, 4 öneri kodu, 16 hibrit, 4 "beklenen yok" | varsayılan hit'ler beklenen küme; `proposal`/`unverified` 0 |
| O11 | dokunulan her belgenin `blame`'i | kayıtlı |
| O12 | md bayt denetimi | her md baytı Obsidian'ın (ya da kopyanın, ya da ajanın `propose`'unun) yazdığıyla aynı: rootmark'ın md yazımı **0** |
| O13 | `SIGTERM` | çıkış 0; stderr'de panik yok; tasdik günlüğü özeti (retler gerekçeleriyle) |
| O14 | `observe.py` | salt okunur; sayıları sürücününkiyle birebir |

Yarıda kalan adım, o adımın kayıtlı başarısızlığıdır (Q-7).

## Ölçülecek sayılar (eşiksiz; `scenario.json` `measures`)
Kurulum süresi, kurulu ikilinin ve arşivin blake2b'si, conformance; `ingest-dir` süresi, olay ve belge sayısı; `index build` süresi, parça ve parça/s, indeks baytı; arama gecikmesi p50/p95 (istemci tarafı, HTTP); MCP `propose` gecikmesi ve `propose`→aranabilir p50/p95; doğrudan yazım → alındı ve Obsidian kaydı → katman p50/p95; `serve`'ün en yüksek RSS'i (`ps`, 0,2 s); sondaki `.rootmark/` baytı; determinizm (`events_hash`, indeks dosyaları, sorgu sonuçları, sıralamalar, adlı ULP farkları).

## Linux
Docker: `debian:trixie-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4` (amd64 manifest `sha256:1e1a22175fbf1b32157ddaf0219e1db4e5401fab2b94ed285add64d18587cd2d`), `--platform linux/amd64` (emülasyon: süreler bilgi). CI: `.github/workflows/pilot.yml`, `ubuntu-latest`, yalnız `dnotes`.
