# P2-Q — Bağımsız Doğrulama (kısa; `docs/rounds/P2-Q/VERIFY.md` VQ1–VQ4)

**Doğrulanan commit:** `11801d0ed71aa697589e522d399733cfba71cd39` (`main`; PR #47, rebase ile birleşti 2026-11-04T19:01:06Z). Ağacı READY commit'i `f6443f3`'ünkiyle eşit (`8e1767b8…`); kademe 1'in koştuğu `0d6ed54`'ün ağacı `main`'deki karşılığı `6aa164d`'ninkiyle eşit (`00b528e8…`). `6aa164d..11801d0` yalnız dört rapor/ilerleme dosyası.

Ayrı oturum, kendi scratchpad'imde temiz klon (GitHub'dan, `11801d0`; önce `make setup` ve `make models`, ikisi de çıkış 0), ana çalışma dizinine erişim yok, kod değiştirilmedi. Kademe 2 tetiklenmedi, Mac kümesi koşulmadı (VERIFY). Kişisel korpusların (qorbit, Obsidian) içeriği okunmadı; bütün koşular açık korpus D-NOTES (`research/data/d_notes/notes`, 400 not) üzerinde. Ortam: Apple M4 Max, macOS; Docker 29.8.0 — **Rosetta açık** (pilotta kapalıydı, `p2-q` FINAL: QEMU). Karşılaştırmaların hepsi kendi betiklerimle (`mycompare.py`, `mycount.py`, `vq3.py`; turun `compare.py`'si kullanılmadı); scratchpad'te kalır.

## VQ1 — Yayım ve kılavuz

### Yayım
- `v0.3.0` yayımlı, "Latest", 12 varlık (`install.sh`, `install.ps1`, üç arşiv, eklenti zip'i, `main.js`/`manifest.json`/`styles.css`, Python tekerleği, `SHA256SUMS`, `SHA256SUMS.local`). `release.yml` `41717748927` `success`.
- `SHA256SUMS`: `install.sh: OK`, `rootmark-0.3.0-aarch64-apple-darwin.tar.gz: OK`. Köken: `gh attestation verify install.sh` çıkış 0, arşiv için çıkış 0. (`SHA256SUMS`'ın kendisinin tasdiki yok — çıkış 1; kılavuz bunu iddia etmiyor, bilgi.)
- Kurulu ikili = arşivdeki ikili: blake2b-128 `a8f2596562f1115ded534e80001a4fd5` ikisinde de (turun kaydıyla aynı).

### Kılavuzu yalnız okuyarak izledim (`docs/GUIDE.md`, `11801d0`)
Her blok kılavuzdan `awk` ile çıkarıldı ve yazıldığı gibi koşuldu; tek uyarlama: kendi önekim (`install.sh --prefix <scratch>`, §1.1'in belgelediği seçenek; `HOME` scratchpad'te, `PATH`/`ROOTMARK_MODELS` `install.sh`'nin yazdığı önek yollarıyla). Bash araç kancası komut satırında `curl`'u engellediği için bloklar betik dosyalarından `bash` ile koşuldu (içerik aynı).

| Bölüm | Sonuç |
|---|---|
| §1.1 kurulum (iki blok) | çıkış 0; `rootmark 0.3.0`, `2.0.3`, `rootmark conformance: level 3, 39/39 (spec 2.0.3)`; kurulum ≈ 11 s |
| §2.1–2.3 (D-NOTES kopyası) | `ingest-dir` 399 belge (400 değil: `.github/` altındaki not atlandı — PQ-13, kılavuzun Known issues'ında var), `events_hash` `35ba798a…`; `policy lint` `ok: true`, kural `collector`; `index build` 6 063 kayıt, `minilm-l6-v2/e73168f9+fixed1`, `cold_records` 6 063, `stale_docs` []; ≈ 5 dk 12 s |
| §3 `serve` + `rmk status` | çıktı kılavuzdakiyle aynı (`0.3.0`, `2.0.3`, `ok`, `unverified`, `ada`); SIGTERM ile 2 s'de `stopped` |
| §4.1 Claude Code | `claude mcp add … --scope project` → kılavuzdaki `.mcp.json`; `claude mcp list` → `⏸ Pending approval` (kılavuz bunu söylüyor; onaylanmadı) |
| §4.2 başka MCP istemcisi | **kendi yazdığım stdio istemcisi** (`clientInfo.name = collector`): protokol `2025-06-18`, 11 araç kılavuzdaki listeyle aynı; `sources/` altına `propose` → kural `collector` ile `Promoted` (aktör `policy`, `policy_version 2026-11-03`); kural dışı `propose` → `proposal`; varsayılan aramada yok, `include_proposal` ile var; ajanın `promote`'u `denied`; stdin kapanınca sunucu 0,3 s'de çıkış 0 |
| §4.3 ajan adları | `collector`, `collector/0.1`, `helper` |
| §5.1 eklenti (elle) | kabuk bloğu: `zip OK`, üç dosya. Obsidian (1.13.4) **kendi profilimde** (`--user-data-dir`, CDP) açıldı; adımlar arayüzde **fare olaylarıyla tıklandı** (Obsidian API'siyle değil): Ayarlar → Community plugins → düğme → rootmark anahtarı. Durum çubuğu `rootmark 0.3.0 · human:ada`; Obsidian'da klavyeyle yazılan bir paragraf `attested 1` ve `blame`'de `canonical`/`Human`; kabuktan doğrudan yazılan yeni klasördeki dosya `unverified` (`unattested_fs`), varsayılan aramada yok. **İki sapma (VF-1, aşağıda).** |
| §5.3 BRAT | **koşuldu** (PQ-23'ün açık bıraktığı): BRAT 2.2.0 → *Add beta plugin* → `arknsensei/rootmark`, "Latest version" → rootmark 0.3.0 kuruldu ve etkinleşti, durum çubuğu bağlandı. BRAT'ın indirdiği `main.js` ve `styles.css` sürüm zip'indekiyle bayt eşit; `manifest.json` aynı içerik, BRAT'ın tek satırlık yeniden yazımı |
| §5.4 kaldırma | `rm -rf` bloğu; ardından BRAT yolu temiz kuruldu |
| §6.1 CLI `propose` (sunucu yok) | çıktı kılavuzla **birebir** (`{"actor":"collector","promoted":2,…}`, `{"actor":"helper",…}`, dosya tam metin) |
| §6.2–6.6 (HTTP, ajan oturumu açıkken) | `blame` `proposal`/`Agent`/`helper`; arama: varsayılanda öneri yok, `include_proposal` ile etiketli; promote → `promoted`/`canonical`, demote → `demoted`/`proposal`; `trace` son izi; `checkpoint before-review`, `at` ile arama ve `blame` |
| §6.7 | yalnız `prune {"dry_run":true}` → `{"pruned":0}` (tarif tabloları eklenmedi; pilot da kuru koşturdu) |
| §7 veri toplama | `observe.py` etiketten indirildi, `rmk status`, özet, `jq` — çalıştı (aşağıda sınaması) |
| §8 göç | `migrate --dry-run` iki satır `required:false`; `migrate` uygulandı, `matrix.required` [] |
| README | iki blok: kurulum (§1.1 ile aynı) ve `ingest-dir vault vault` + `serve … --mcp stdio --http 127.0.0.1:0 --watch` (tarif yoksa varsayılan yazar, indeksi kurar, MCP akışı kapanınca durur) — çalıştı |

**VF-1 (belge, yeni; §5.1'in Obsidian içindeki adımları — pilotun API ile geçtiği yer, PQ-23):** (a) Kılavuzun sırasıyla (önce zip vault'un `.obsidian/plugins/`'ine açılır, sonra vault Obsidian'da ilk kez açılır) Obsidian **"Do you trust the author of this vault?"** penceresini gösterir ("Browse vault in Restricted Mode" / "Trust author and enable plugins"); kılavuz bunu anmıyor. Kılavuzun adımlarını sınamak için Restricted Mode'u seçtim; "Trust author and enable plugins" da aynı yere götürür. (b) Obsidian 1.13.4'te adım 1'in düğmesi **"Exit Restricted mode"**dur, kılavuzdaki "Turn on community plugins" değil (kılavuz "Restricted mode" adını parantezde anıyor; yeni kullanıcı bulur). İkisi de kılavuz dışı birer adım; ürün hatası değil.

### `Q1_verifications.md` koşullarının kapatan kararları (R16)
Her "şu koşullarla" hükmünün kapatan kararı `docs/DECISIONS.md`'de var ve raporu adıyla anıyor: P2-V → **K-P2V.3** (`reports/rounds/P2-V/VERIFICATION.md`), P2-V.1 → **K-P2V1.2** (`…/P2-V.1/VERIFICATION.md`) + K-P2V2.2, Blok 2 (R-10, F1) → **K-P2Q.2** (`…/P2-R/VERIFICATION.md`; insan kararı, ölçümden önce), P2-V.2 "güvenilir" → K-P2V2.2. Blok 1 "Güvenilir". Kapanmamış koşul yok.

### `observe.py` (kendi D-NOTES vault'umda, §7'nin indirdiği dosya)
- **Salt okunur:** vault'un bütün ağacının (`.rootmark/` dahil, yol + içerik) blake2b'si koşudan önce ve sonra `ba115893be30193cf562cc005a876c69`.
- **Sayılar kendi okumamla birebir** (`mycount.py`: `events.jsonl`'ı kendim yeniden oynatarak): toplam 28 520 olay; tür başına `BlockCreated` 28 106, `SnapshotCommitted` 405, `Promoted` 5, `CheckpointCreated` 3, `Demoted` 1; soy katmanları `canonical` 28 100, `proposal` 4, `unverified` 2; terfi eden aktör `human` 1, `policy` 4; kural başına `collector` 4 — `observe.py`'nin çıktısıyla aynı.
- **Çıktıda içerik yok:** yol, sorgu ve metin parçası aramam (`agent/`, `sources/`, `.md`, sorgu sözcükleri, not başlığı) 0.
- rootmark md yazmadı: vault'un md baytları kaynakla aynı, farklar yalnız benim Obsidian düzenlemem ve `propose`'ların oluşturduğu (tam verilen metinle) beş dosya.

## VQ2 — Karşılaştırmayı kendim yaptım
- **Ön-kayıt önce commit'li:** VQ4'te (`git log`): `scenario.json`/`SCENARIO.md` `88a4412`, `51d11ed` (2026-11-03 22:22), `scenario_tr.json`/`SCENARIO_TR.md` `3803886` (23:08); ilk Q2 kaydı 2026-11-04 00:46; sonra değişmedi.
- **Kendi Mac koşum:** kurulu 0.3.0 (kendi önekim, VQ1) ile `python3 -m p2q.driver --corpus dnotes` (turun sürücüsü ve ön-kaydı; depo derlemesi kullanılmadı). Adımlar **19/23**; kalanlar **H02** (399/400, PQ-13), **H11**, **H15** (hibrit "içerir", PQ-15), **H21** (MCP açıkken SIGTERM, PQ-14) — turun Mac koşularıyla aynı dört adım. İlk indeks 305 s (M4 Max; tur 297 s), arama p50 206 ms; `observe_compare` 8/8 eşit.
- **Kendi Docker koşum:** `research/p2q/docker_stage.sh dnotes` — ön-kayıtlı imaj `debian:trixie-slim@sha256:f6b08b09…3ec4`, `--platform linux/amd64`; konteynerde kılavuzun §1.3 bloğu (`install.sh`, `sha256sum -c`, `rootmark conformance`) `ok` (1/1, başarısız 0); senaryo **19/23**, kalanlar aynı dört adım; `observe_compare` 8/8 eşit. Ortam: Docker Desktop'ta Rosetta **açık** (`VirtualApple @ 2.50GHz`), bu yüzden pilottan çok hızlı: ilk indeks 1 055 s (pilotta QEMU ile bütün sahne 32 772 s). Süreler bilgi.
- **Bayt karşılaştırması (`mycompare.py`; `hashes.json`'ın `events_hash`, 13 indeks dosyası, vektörler, manifest; `queries.json`'ın her aşama/sorgu/kip için sıralı `(belge, parça)` listesi ve skorların ULP farkı):**

| Çift | `events_hash` | vektörler | manifest | indeks dosyaları farklı | sıralama farkı | en büyük ULP | adım sonucu farkı |
|---|---|---|---|---|---|---|---|
| benim Mac ↔ benim Docker | eşit | eşit | eşit | 0/13 | 0 | 0 | yok |
| benim Mac ↔ turun Mac (`mac/dnotes/run1`) | eşit | eşit | eşit | 0/13 | 0 | 0 | yok |
| benim Mac ↔ turun Docker (`docker/dnotes`) | eşit | eşit | eşit | 0/13 | 0 | 0 | yok |
| turun Mac run1 ↔ run2 | eşit | eşit | eşit | 0/13 | 0 | 0 | yok |
| benim Mac ↔ turun CI (`41801236926`; `41803469014`, `41809290667` aynı) | **farklı** | farklı | farklı | 19 | **yalnız q33** (`include_proposal`, üç aşamada) | 0 | H09, H16, H23 (son koşuda H09, H16) |
| CI koşuları birbiriyle | eşit | eşit | eşit | 0/13 | 0 | 0 | yalnız H23 (sürücü değişikliği `bb541a9`, VQ4) |

  `events_hash` dört Mac/Docker koşusunda `173c13df79d2d98dbff1df45453e193c`. Mac ↔ CI farkının kaynağı: CI'da `ajan-dogrudan/pqd000.md` (yeni dizinin ilk dosyası) alınmamış (H09) — PQ-22; ondan bağımsız hiçbir sıralama ya da skor farkı yok (checkpoint kimlikleri durum farklı olduğu için farklı, 144). **FINAL'in Q-8 okumasıyla aynı.** CI'ı yeniden tetiklemedim (VERIFY); CI kayıtları depodaki yapıtlardır.

## VQ3 — Kasıtlı bozmalar (`vq3.py`)
Taban: turun kendi kaydı `mac/dnotes/run1/queries.json` değerlendiriciden (`evaluate.check_phase`) `ajan-yazdi` 3, `son` 7 başarısızlıkla geçer (H11/H15'in kayıtlı kalışları, PQ-15); bozmalar bu tabana göre **yeni** başarısızlık olarak okundu.
- **(a) Doğrudan yazım `canonical`:** (a1) gerçek bir sunucu, tarifinde `unattested = "canonical"` (kılavuzun tarifi, tek satır değişik), yeni klasöre doğrudan yazım → `blame` 2 blok `canonical`; `evaluate.direct_write_canonical` **2** (Q-6 eşiği 0 → **kalır**), sürücünün H09 koşulu da yanlış (kalır). (a2) Kayıtta geçen bir sorgunun (`q04`) varsayılan isabeti `unverified` yapıldı → `q04@son: default hit … is unverified` (sızıntı > 0 → Q-6 kalır). İkisi de yakalandı.
- **(b) İçerik sızıntısı, korpus D-NOTES:** bir kaydın (`numbers.json`) kopyasına bir D-NOTES belgesinden (a) o belgenin **tohumla seçilen parçasını içeren satır** eklendi → `leakcheck` **bulur** (1, `pass: false`); (b) **aynı belgeden başka bir satır** eklendi → **bulmaz** (0, `pass: true`); bozulmamış kayıt 0. Yöntem korpustan bağımsız çalışıyor, ama belge başına **tek** 32 baytlık parça örnekliyor: 360 belgede, bir belgeden rastgele sızan tek bir satırın yakalanma olasılığının ortalaması **%4,5**. Q-9'un "0"ı toplu kopyalamaya karşı güçlü, tek cümlelik sızıntıya karşı zayıf kanıttır (Şüphe Ş1).
- **(c) Beklenen küme değişti:** `q01`'in `son` aşamasındaki beklenen varsayılan kümesine bir belge eklendi → `q01@son: default docs [] != ['no/such/doc.md']`. Fark yazıldı.

## VQ4 — Sınır, bulgular, karar kuralı
- **Ürün sınırı:** `git diff 839bf27..11801d0 -- core index server/src bindings 'clients/*/src' spec` yalnız üç `Cargo.toml` sürüm satırı (`0.2.0` → `0.3.0`). Bu dizinlerin dışında ürün tarafında yalnız `server/Cargo.toml` ve istemcilerin `manifest.json`/`package.json`/kilit dosyalarındaki sürüm dizgileri ve yeni `.github/workflows/pilot.yml` (ürün değil).
- **Ön-kayıt (Q-1):** `scenario.json` ve `SCENARIO.md` iki commit'te (`88a4412` Q0, `51d11ed` anahtarlı yol özeti, 2026-11-03 22:22); `scenario_tr.json`/`SCENARIO_TR.md` tek commit `3803886` (23:08, K-P2Q.3). `v0.3.0` etiketi 23:21; ilk Q2 kaydı `4568ea9` (2026-11-04 00:46). Sonra değişmedi. Q2'den sonra `research/p2q/`'de beş commit: üçü kayıt/araç (işlemci adı, hata iletilerinin temizlenmesi, `numbers` üreticisi), biri Obsidian sahnesi; **`bb541a9`** sürücünün H23'ünü değiştirir (gözlem sayısı artık izleyicinin aldığı doğrudan yazımlarla karşılaştırılır) — CI'daki kaçan yazım (PQ-22) görüldükten sonra. H09 ve H16 PQ-22'yi yine kalış olarak kaydeder; değişiklik yalnız Q-11'in "birebir" karşılaştırmasının tanımını düzeltir ve commit metninde açık. Fark değil, not.
- **`FINDINGS.md`:** PQ-1..PQ-23'ün hepsi sınıflı (ürün hatası, belge, kurulum, UX, ölçüm aracı, ortam, bakım) ve hedefli (bu tur, P2-Q.1, P2-C, P2-H, P2-O, insan adımı, doğrulayıcı). PQ-23'ün hedefi "doğrulayıcı (VQ1)": BRAT ve tıklamalar yukarıda koşuldu; tıklamalarda VF-1 çıktı.
- **`numbers.json`:** 10 koşunun her satırı `record` alanında bir kayıt adlandırıyor; 173 sayıyı kendi betiğimle kayıtlarındaki değerle karşılaştırdım: **173/173 eşit**, eksik kayıt 0.
- **Host değeri:** `reports/rounds/P2-Q`, `docs/rounds/P2-Q`, `docs/GUIDE.md`, `README.md`, `research/p2q` içinde host adı, `/Users/<kullanıcı>`, `/home/runner`, kullanıcı adı yol içinde, `/var/folders`, `/private/tmp` araması: **0**. (`eval.portable --all` doğrulama RUN dizini bekliyor; turun komut satırı kayıtlı değil, onu yeniden koşamadım — bu arama onun yerine.)
- **`GUIDE_RUN.md`:** sapma G1 → PQ-6 (düzeltildi, R1'de yeniden koşuldu). **G2** (§4.3'ün boş çıktısı; "(after their first `propose`)" eklendi) ve aday denemesindeki `migrate --dry-run` alan adları düzeltmesi **ayrı bir PQ numarası almamış** (en küçük tarif → PQ-3, CLI `propose` → PQ-11 almış). Düzeltilmiş adımlar R1'de `ok` ve benim koşumda da çalıştı (§4.3, §8). Küçük süreç farkı (F1).
- **Q-10:** PR #47 CI `41845930633` (`0d6ed54`) `success`, 10 işin hepsi yeşil, 14:16:35 → 14:53:26 (36 dk 51 s ≤ 40 dk). Gözden geçirici kaydı `Q4.review.md` KABUL. `make test/lint/conformance` yerelde yeniden koşulmadı (VERIFY kapsamı dışında; CI'daki iki `test + lint + conformance` işi yeşil).
- **Karar kuralını kendim türettim:** GOAL'ün dalları sırayla: Q-2 geçti (kapalı koşullar, yukarıda), Q-3 geçti (yayımlı), Q-4 geçti (turun kaydı; benim koşumda iki belge sapması VF-1, düzeltilebilir — "düzeltilip yeniden koşulamadı" değil); **Q-6 kaldı** (D-NOTES H02, H11, H15, H21 — benim Mac koşumda da, VQ2) ve **Q-8 kaldı** (Mac ↔ CI eşit değil, yalnız PQ-22'den — turun kayıtlarından kendi karşılaştırmamla, VQ2); bu dal: "bulgu; veri kaybı/sızıntı/determinizm ürün hatası ise yama turu (P2-Q.1) P2-C'den önce, değilse P2-C'nin ön-kayıtlı kriteri; tur kaldı, karar talebi". PQ-22 (sessiz kayıp, determinizm) ve PQ-18 (silinen bilgi canlı kalır) yama turu sınıfında; PQ-13/14/15 değil. **FINAL'in dalıyla aynı** (K-P2Q.4 (a)).
- **Vaat sorusu (kılavuzu izleyen yeni kullanıcının gözünden):** (1) Masaüstü kurulumda (§4.1, Claude Code aynı OS kullanıcısıyla) ajan `.rootmark/server.token`'ı okuyabilir; okuyan süreç `human:<id>`'dir (§3 "an agent must not be able to read it (§9)"). Kılavuz ayrımı yalnız Linux için (§9, ayrı OS kullanıcısı) somutlaştırıyor; masaüstünde ajanın token'a (ve vault'a doğrudan yazmaya) erişimini nasıl kapatacağını söylemiyor. NORTH_STAR'ın "onaylanmamış hiçbir şey bilgi sayılmaz" vaadi masaüstünde bu yüzden ajanın iyi niyetine dayanıyor; THREAT_MODEL'de bilinen sınır, ama kılavuzun ana yolunda açık değil. GOAL bunu ölçmüyor. (2) PQ-18 (silinen notun blokları aramada) yeni kullanıcı için "dosyalar gerçek kaynak" vaadine en görünür aykırılık; Known issues'ta var, P2-Q.1'de.

## Sonuç
### İhlal / fark / şüphe
- **İhlal:** yok (ürün kodu sınırı, ön-kayıt, gizlilik, değişmez kurallar).
- **Fark:** **VF-1** — kılavuzun §5.1'indeki Obsidian içi adımlarda iki belge sapması (güven penceresi anılmıyor; düğmenin adı "Exit Restricted mode"); pilot bu adımları API ile geçtiği için görmedi (PQ-23). **F1** — G2 ve `migrate` belge düzeltmeleri `FINDINGS.md`'de numarasız.
- **Şüphe:** **Ş1** — Q-9'un içerik denetimi belge başına tek parça örnekliyor; tek satırlık sızıntıyı ~%4,5 olasılıkla yakalar (VQ3 (b)). Q-9'un eşiği ve ölçümü GOAL'deki tanımla tutarlı; okunuşu "toplu kopya yok" olmalı.

### Kriter başına
| Kod | Sonuç | Dayanak |
|---|---|---|
| Q-1 | **geçti** | `git log` (VQ4) |
| Q-2 | **geçti** | kapatan kararlar DECISIONS'ta, raporları adıyla (VQ1) |
| Q-3 | **geçti** | `v0.3.0` yayımlı, SHA256SUMS ve köken, `release.yml` success; R1–R20 21/21 turun kaydından (yeniden koşulmadı) |
| Q-4 | **geçti** (VF-1 ile) | kılavuzun ve README'nin koştuğum bütün blokları yazıldığı gibi çalıştı; Obsidian içi iki adımda belge sapması (VF-1), düzeltilebilir |
| Q-5 | **geçti** | kendi kurulumum: SHA256SUMS, köken, 0.3.0, ikili = arşiv, 39/39; eklenti elle ve BRAT ile kuruldu, bağlandı |
| Q-6 | **kaldı** | kendi Mac ve Docker koşumda D-NOTES 19/23 (H02, H11, H15, H21 — PQ-13/15/14); `canonical` doğrudan yazımdan 0, sızıntı 0, dönüş eşit, kopyalar arası bayt eşit — yalnız davranış kısmı kalıyor (FINAL'le aynı) |
| Q-7 | **geçti** | turun kaydı (O01–O14 kayıtlı, md yazımı 0, çökme 0); kendi Obsidian denememde tasdik → `canonical`, md yazımı 0, çökme 0 |
| Q-8 | **kaldı** | Mac ↔ Docker bayt eşit (kendi iki koşum ve turunkiler), Docker'da kurulum ve 39/39; Mac ↔ CI eşit değil, yalnız PQ-22'den (q33), 0 ULP (FINAL'le aynı) |
| Q-9 | **geçti** (Ş1 ile) | bulgular sınıflı/hedefli, `numbers.json` 173/173 kayıttan, host değeri 0, sızıntı denetimi D-NOTES ile çalışıyor |
| Q-10 | **geçti** | ürün farkı yalnız sürüm dizgileri; PR CI yeşil 36 dk 51 s; gözden geçirici KABUL |
| Q-11 | **geçti** | `observe.py` salt okunur, sayıları kendi sayımımla birebir, içerik 0 |

### Hüküm
**Şu koşullarla güvenilir.**

Turun ölçümleri bağımsız olarak yeniden üretildi: kendi kurulumum, kendi Mac ve Docker koşum turun Mac ve Docker kayıtlarıyla bayt eşit; Q-6 ve Q-8'in kalışları ve kaynakları (PQ-13/14/15, PQ-22) aynı; karar kuralının dalı FINAL'inkiyle ve K-P2Q.4'le aynı; `numbers.json`, `observe.py`, ürün sınırı ve ön-kayıt kayıtlarıyla tutarlı. Koşullar: **(1)** VF-1 (kılavuz §5.1'in Obsidian içi iki adımı) `FINDINGS.md`'ye bir belge bulgusu olarak yazılır ve kılavuz P2-Q.1'de düzeltilir — o zamana kadar Q-4'ün "kılavuz dışı adım 0"ı yalnız pilotun API ile geçtiği adımlar dışında geçerlidir; **(2)** Q-9'un içerik denetiminin "0"ı toplu kopyalamanın yokluğu olarak okunur, tek cümlelik sızıntının yokluğu olarak değil (Ş1).

### Öneriler (≤ 3)
1. **Kılavuz §5.1'i (P2-Q.1'in belge işi):** güven penceresini ("Trust author and enable plugins" ya da Restricted Mode'dan sonra adım 1) ve Obsidian'ın güncel düğme adını ("Exit Restricted mode") yaz; G2 ve `migrate` düzeltmelerine PQ numarası ver (F1).
2. **Masaüstü ajan ayrımı:** kılavuzun §4.1'ine ajanın `.rootmark/` ve vault'a erişimini kapatmanın somut bir yolunu (ör. Claude Code izin kuralıyla `.rootmark/**` okumayı ve vault'a doğrudan yazmayı reddetmek) ya da bunun yapılmazsa ajanın insan yerine geçebileceğini açıkça ekle; P2-C/P2-E kapsamında değerlendir.
3. **İçerik denetiminin duyarlılığı (Ş1):** sonraki pilot/canlı turlarda `leakcheck` belge başına birden çok parça (ya da her satırın özeti) örneklesin ve FINAL "0"ı yakalama olasılığıyla birlikte raporlasın.
