# D-0 — Görevler

Genel kurallar:
- **Sıra:** R0 → R1 → R2 → R3 → R4 → R5 → R6 → R7. R1, R2, R3 ve R5 birbirinden bağımsızdır; insan adımı beklerken (ör. R2'nin elle sorgulanan kaynakları) bir sonrakine geçilebilir. R4, R1 ve R3'ün sonucuna dayanır. R0'ın `DESIGN.md`'si commit'lenmeden tarama kaydı yazılmaz.
- **Kayıt ve dallar:** progress `docs/rounds/D-0/progress/`. Commit öneki `D-0/R<n>:`. Dal `d-0/round`, yalnız birleşmiş `main`'den (K-P2.2). Raporlar `reports/rounds/D-0/`.
- **Gözden geçirici:** karar turu, araç dozu (K-P2.4): yalnız R7'de, tek tur.
- **Ürün sınırı:** yalnız `docs/`, `reports/rounds/D-0/`, `research/d0/` ve `server/CLAUDE.md`'nin `rmcp` satırı değişir (GOAL İlkeler).
- **Dışarıya yazan eylem yok:** kayıt defterlerine yalnız herkese açık `GET`. Hesap, kurum, depo, ad, paket, imaj, etiket, DNS: hiçbiri. Gerekirse `HUMAN_STEP.md`.
- **İnsan adımı ve sorular:** insan adımı beklemek durma değildir (`HUMAN_STEP.md`). Soru sorulmaz, varsayım `ASSUMPTIONS.md`'ye.
- **Kişisel içerik ve gizli bilgi:** kişisel vault ve qorbit'e dokunulmaz. Raporlarda token, parola, kişisel e-posta adresi yok.

## R0 — Açılış
**Yapar:**
- `ROUND_START`; DECISIONS K-D0.2 (açılış).
- `progress/DESIGN.md`:
  - R1'in ad × kayıt defteri matrisi ve her kayıt defteri için sorgu yöntemi (API uç noktası ya da elle arayüz; "dolu" ve "boş"un nasıl anlaşıldığı, ör. 200/404, ad normalizasyonu: PyPI'da `-`/`_`/büyük harf eşdeğerliği, GitHub'da büyük/küçük harf);
  - R2'nin kaynakları ve sorguları;
  - R3'ün izin listesinin biçimi (desen dili, sınıflar) ve denetim betiğinin ne denetleyeceği;
  - R5'in sınama adımları ve beklenen gözlemler.

**Kabul:** D0-1.

## R1 — İsim ve kayıt defterleri taraması
**Yapar:**
- `research/d0/registry_scan.py`: GOAL'deki ad listesi × kayıt defterleri; her hücre için URL, UTC zaman, yanıt kodu, içeriksiz özet. Çıktı `reports/rounds/D-0/registry_scan.json` ve okunur tablo `registry_scan.md`. Yeniden koşulabilir.
- PyPI'daki `rootmark` projesinin ve yakın alandaki projelerin içeriksiz özeti (ne yaptığı, ilk ve son sürüm tarihi, kaynak deposu).
- Alan adları RDAP ile; `rootmark.ai` ürün sahibinin (GOAL girdisi) — yalnız kayıt durumu ve bitiş tarihi raporlanır.
- Erişilemeyen ya da tarayıcı gerektiren kaynaklar `HUMAN_STEP.md`'ye.

**Kabul:** D0-2.

## R2 — Marka taraması
**Yapar:**
- TÜRKPATENT, EUIPO TMview, USPTO, WIPO Global Brand Database: "rootmark", "root mark", "rootmarks" ve R1'in yedek adları; Nice 9 ve 42.
- Ajanın erişebildiği kaynaklar betikle ya da belgelenmiş elle sorguyla; erişemediği arayüzler `HUMAN_STEP.md`'de adım adım (sayfa, sorgu, sonucun hangi biçimde `progress/R2_trademark_human.md`'ye yazılacağı).
- `reports/rounds/D-0/trademark.md`: kaynak başına kanıtlı satır ve "hukuki görüş değildir" notu.

**Kabul:** D0-3.

## R3 — Depo ayrımı ve aktarım planı
**Yapar:**
- `reports/rounds/D-0/transfer/allowlist.txt` (giden), `stays.txt` (kalan), `transforms.md` (dönüşen ve nasıl dönüştüğü).
- `research/d0/classify.py`: `git ls-files` (bugünkü `main`) → her dosya tam bir sınıfta; sayılar `reports/rounds/D-0/transfer/classification.json` (toplam, giden, kalan, dönüşen, sınıflandırılmamış = 0, çift sınıflı = 0).
- Sızıntı denetimi (aynı betik): giden kümede `research/`, `reports/`, `docs/rounds/`, `docs/archive/`, `.rootmark/`, `server.token` ya da anahtar desenli dosya, kişisel yol dizgisi (`/Users/`) 0.
- `reports/rounds/D-0/TRANSFER_PLAN.md`:
  - aktarım yöntemi (sürüm başına; laboratuvar etiketi → ürün deposu commit'i), geçmiş politikası ve gerekçesi;
  - bayt eşitliği denetimi (ürün deposundaki ağaç = izin listesinin laboratuvar etiketindeki ağacı + yazılı dönüşümler);
  - ürün deposunun kurum ve adı (öneri + alternatif), hangi iş akışlarının ürün deposunda koşacağı, hangilerinin laboratuvarda kalacağı, sürüm etiketlerinin ve GitHub sürümlerinin yeri, bugünkü GitHub Pages bağlantılarının (`arknsensei.github.io/rootmark/spec/`) kaderi;
  - laboratuvar deposunun adı ve görünürlüğü;
  - lisans ve üçüncü taraf: Apache-2.0, `NOTICE`, `models/` (MiniLM ailesi) lisansları; modelin imajda ya da ilk açılışta indirilmesinin lisans sonucu;
  - insan adımları listesi.

**Kabul:** D0-4.

## R4 — Kod adları ve kanallar
**Yapar:**
- `reports/rounds/D-0/CHANNELS.md`:
  - kod adları tablosu (ikili, crate'ler, Python modülü, `.rootmark/`, spec adı ve sürümü, Obsidian ve VS Code kimlikleri): karar önerisi, değişirse göç ve uyumluluk notu;
  - kanallar tablosu: `ghcr.io`, Docker Hub, Homebrew tap, PyPI, crates.io, npm, Obsidian topluluk eklentileri, VS Code Marketplace / Open VSX, belge sitesi. Her biri için: ad, yayımlanıp yayımlanmayacağı ve gerekçesi, yayım yöntemi (trusted publishing mümkünse o), insan adımları, gizli değişken **adları**;
  - imaj tasarımı: taban dağıtım, mimariler (`linux/amd64`, `linux/arm64`), modeller imajda mı ilk açılışta mı (boyut, lisans, ağsız çalışma), çalıştırma kullanıcısı, sağlık denetimi, kılavuzun hızlı başlangıcının `docker run` biçimi;
  - P2-Q.2 doğrulamasının Öneri 1, 2, 3'ünün D-1 planındaki yeri.

**Kabul:** D0-5.

## R5 — PQ2-1 sınaması
**Yapar:**
- `research/d0/pq2_1_probe.py` (ya da `.sh`): yayımlanmış 0.3.2 (`gh release download v0.3.2`, `SHA256SUMS` denetimiyle), D-NOTES kopyası ya da sentetik kasa (scratchpad'de):
  1. `ingest-dir` + `index build`;
  2. `serve --watch` başlat; hazır olmasını bekle;
  3. bir notu sil (ve ayrı bir tekrarda tasdiksiz taşı);
  4. kaldırma beklemesi dolmadan sunucuyu SIGKILL ile öldür;
  5. `serve --watch` ile yeniden başlat, taramanın bitmesini bekle;
  6. gözlem: notun blokları için `BlockDeleted` var mı, varsayılan `search` onları buluyor mu, `events_hash`.
- Karşılaştırma: aynı dizi SIGKILL yerine düzgün durdurmayla (SIGTERM, beklemeden sonra).
- ≥ 10 tekrar macOS'ta ve Linux'ta (Docker, P2-Q'nün imaj reçetesi). Kayıt `reports/rounds/D-0/pq2_1/` (sayılar, sürümler, komutlar; içerik yok).
- Sonuç ve öneri `REPORT.md`'ye: kalıcı mı, düzeliyor mu; D-1'den önce düzeltme gerekli mi.

**Kabul:** D0-6.

## R6 — README iskeleti ve belgeler
**Yapar:**
- `reports/rounds/D-0/README_SKELETON.md` (İngilizce): amaç, kapsam, ne değildir, hızlı başlangıç (ikili, Homebrew, Docker), bağlantılar; bugünkü `README.md`'den yararlanır ama ürün deposunun kitlesine göre.
- `server/CLAUDE.md`: `rmcp =3.5.0` → `=3.6.0` (PQ2-4).

**Kabul:** D0-7'nin README ve PQ2-4 kısmı.

## R7 — Rapor, gözden geçirici, hazır
**Yapar:**
- Kapılar: `make test`, `make lint`, `check_frozen`, `check_deps`, `check_dep_age`; `git diff ROUND_START..HEAD --stat` yalnız izinli yollarda.
- `reports/rounds/D-0/REPORT.md`: R1–R6 özeti; **karar listesi** — en az: ürün adı, PyPI adı, GitHub kurumu ve ürün deposu adı, laboratuvar deposunun adı/görünürlüğü, imaj adları ve modelin yeri, Homebrew tap adı, yayımlanacak/yayımlanmayacak kayıt defterleri, belge sitesinin adresi, kod adlarının kaderi, PQ2-1'in sırası, marka tescili yapılıp yapılmayacağı. Her biri için seçenekler, öneri, gerekçe.
- `FINDINGS.md` (yeni bulgular sınıflı ve hedefli).
- Gözden geçirici tek tur (GOAL başlığı).
- `reports/rounds/D-0/FINAL.md`: D0-1..D0-7, karar kuralının dalı, "koştu/geçti".
- READY; tek PR: `gh pr create --base main --head d-0/round --title "D-0: isim, depo ve aktarım planı" --body-file reports/rounds/D-0/FINAL.md`.

**Kabul:** D0-7; READY'de "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
