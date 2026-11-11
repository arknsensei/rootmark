# D-0 — Laboratuvar ↔ ürün deposu: ayrım ve aktarım planı (R3)

Bu belge D-1'in uygulayacağı planı yazar; D-0 hiçbir depo, kurum ya da iş akışı açmaz. Sayılar `ROUND_START` = `b6f5b7b` üzerindedir ve betikle yeniden üretilir.

## 0. Özet
- **İzin listesi** üç dosyadır: `transfer/allowlist.txt` (aynen giden), `transfer/stays.txt` (laboratuvarda kalan), `transfer/transforms.md` (dönüşerek giden; dönüşüm türü ve ürün yolu). `research/d0/classify.py` bugünkü `main`'in her izlenen dosyasına tek sınıf verir.
- **`b6f5b7b`'de:** toplam **16 584** dosya → giden **589**, kalan **15 351**, dönüşen **644**; **sınıflandırılmamış 0**, çift sınıflı 0, ürün yolu çakışması 0, ölü desen 0 (`transfer/classification.json`).
- **Sızıntı denetimi (aynı betik, giden + dönüşenlerin ürün yolları):** `research/`, `reports/`, `docs/rounds/`, `docs/archive/` öneki 0; `.rootmark` yol bileşeni 0; `server.token` 0; anahtar desenli dosya 0; çalışan kullanıcının adı ya da ev dizini içerikte 0; `/Users/<x>/`, `/home/<x>/` dizgisi 2 (ikisi de kılavuzun kurgu örneği: `ada`, `rmhuman`; `transforms.md` "İstisnalar"da gerekçeli; gerekçesiz 0); aynen giden Markdown'da kalan bir dosyaya kırık göreli bağlantı 0.
- **Giden küme tek başına derleniyor** (dönüşümler uygulanmadan, yalnız ürün yollarına konmuş): `research/d0/export_check.sh` → 1 233 dosya ayrı bir dizine, `cargo build --release --locked --workspace` **ok**, `clients/obsidian` ve `clients/vscode` `npm ci && npm run build` **ok** (`transfer/export_check.json`).
- **Yöntem:** sürüm başına aktarım (laboratuvar etiketi `vX.Y.Z` → ürün deposunda tek commit), **temiz geçmiş** (ürün deposunun geçmişi ilk aktarılan sürümle başlar), bayt eşitliği **ağaç kimliğiyle** denetlenir (`git write-tree` eşit), aktarımı bir insan tetikler.

## 1. Sınıflar ve gerekçeleri

### 1.1 Giden (aynen, 589)
Çalışma zamanı kodu (`core/`, `index/`, `server/`, `bindings/` — `CLAUDE.md`'leri ve laboratuvar verisi okuyan 8 test dosyası hariç), çalışma alanı dosyaları (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`), istemciler (`clients/` — hipotez turlarının gerçek Obsidian ve D-NOTES harness'leri `e2e/p2_0*`, `e2e/p2_b` hariç), spec ve uyumluluk fixture'ları (`spec/`, 217), model manifestleri, `LICENSE`, `NOTICE`, `CODE_OF_CONDUCT.md`, `.gitattributes`, `.gitignore`, hata şablonu, laboratuvar yolu taşımayan yapım/yayım/ürün betikleri (`fetch_model.sh`, `linux_cross_build.sh`, `release_target.sh`, `conformance_equiv.sh`, `gen_client_types.sh`, `client_checks.sh`, `mcp_e2e.sh`, `http_e2e.sh`, `mcp_inspector.sh`, `dep_age_exceptions.txt`).

Kod yorumlarındaki tur ve karar atıfları (ör. "GOAL P1-D", "K-P2B.1") **aynen kalır**: bayt eşitliği kodun dokunulmamasını ister; atıflar herkese açık laboratuvar deposunda çözülür (ürün README'si laboratuvarı "geliştirme geçmişi" olarak bağlar, README_SKELETON).

### 1.2 Kalan (15 351)
`research/` (1 193 dosyanın 719'u; 474'ü — ürün testlerinin başvuru verisi — dönüşerek gider, §1.3), `reports/` (13 977), `docs/rounds/`, `docs/archive/`, Türkçe süreç belgeleri (`NORTH_STAR.md`, `docs/DECISIONS.md`, `DEV_PROCESS.md`, `ROADMAP.md`, `HANDOFF.md`, `PACKAGES.md`, `RELEASE.md`, `ARCHITECTURE.md`, `SPEC_EDITS_CRITERION.md`, `SECURITY.md`, `HEADLESS_CHANNEL.md`), dondurulmuş ölçüm ve doğrulama zinciri (`verify.sh`, `verify_mac.sh`, `check_frozen.sh`, `check_compare_fields.sh`, `check_records_portable.sh`, `embed_cpu.sh`), turların kasıtlı bozma testleri (`test_*.sh`), laboratuvarın yayım kapısı (`release_check.sh` — R16 doğrulama raporuna bakar), eski sürüm ikilileri ya da prototip isteyen üreteçler (`gen_fixtures*.sh`, `fixture-gen/`, `gen_*old_vaults.sh`, `gen_old_index.sh`, `gen_v3final_mixed.sh`, `v3final-mixed-gen/`, `check_v3_compat.sh`), `verify.yml`, `pilot.yml`, hipotez ve karar talebi şablonları.

### 1.3 Dönüşen (644)
`transfer/transforms.md` tablosu; üç tür:
- **K — konum (baytlar aynen, yol değişir):** ürün testlerinin okuduğu laboratuvar verisi `research/data/{tok_ref.json, embed_ref.json, p2_0_old_vaults/, p2_0_1_old_vaults/, p2_0_2_old_vaults/}` → `testdata/…` (eski sürümlerin yazdığı sentetik kasalar; göç ve `events_hash` uyumluluğu ürünün sözüdür, testleri ürün deposunda koşmalı); `server/tests/fixtures/index_ort/vault/.rootmark/` (131 dosya, sentetik eski indeks) → `…/vault/dot-rootmark/` (giden kümede `.rootmark/` olmaması kuralı; test geçici kopyada adı geri verir).
- **M — mekanik yeniden yazım:** depo URL'si (`arknsensei/rootmark` → ürün deposu: `docs/GUIDE.md`, `packaging/`, `server/Cargo.toml`, `release.yml`, `pages.yml`, `CHANGELOG.md`), laboratuvar göreli bağlantıları → laboratuvar etiketinin mutlak URL'si (`CHANGELOG.md`, `SECURITY.md`), 8 test dosyasında `../research/data/` → `../testdata/`, `p2v1_migrate.rs`'de `dot-rootmark` → `.rootmark` kopyası, üç betikte varsayılan çıktı dizini `reports/rounds/…` → `out/…`, `check_deps.sh`'ten `research/` kuralı, `check_dep_age.sh`'ten Python kısmı (ve npm dolaylı bağımlılıkları eklenir, CHANNELS §5), `release.sh`'ten gerçek Obsidian adımı, `dependabot.yml`'den `pip`.
- **Y — yerine yazılan (ürün kaplaması):** `README.md` (README_SKELETON'dan), `CLAUDE.md` ve beş paket `CLAUDE.md`'si (İngilizce, süreç atıfsız), `CONTRIBUTING.md`, `Makefile` (yalnız ürün hedefleri), `docs/CLIENT_CONTRACT.md` ve `docs/THREAT_MODEL.md` (İngilizce çeviri), `ci.yml` (ürün CI'ı), PR şablonu. Kaplamalar laboratuvarda `product/overlay/<ürün yolu>` altında tutulur (D-1 açar); aktarım onları bayt eşit kopyalar. Laboratuvardaki kaynak dosya değişip kaplama güncellenmezse aktarım aracı **durur** (kaplama dosyası kaynağın hangi blob'undan yazıldığını `product/overlay/SOURCES.json`'da taşır; eşleşmezse hata — sessiz başarısızlık yok).

### 1.4 Ürüne yeni giren (laboratuvar kaynağı olmayan)
`product/overlay/` yalnızca-ürün dosyaları da taşıyabilir: `RELEASING.md` (İngilizce yayım adımları; `docs/RELEASE.md`'nin ürün yüzü), `Dockerfile` ve imaj iş akışı (CHANNELS §3), `.github/workflows/transfer-verify.yml` (§2.4), `THIRD_PARTY_LICENSES` (§5). Bunlar sınıflandırmada laboratuvar dosyası değildir; aktarım manifestinde `origin: overlay` diye listelenir.

## 2. Aktarım yöntemi

### 2.1 Sürüm başına
1. Laboratuvarda tur biter, bağımsız doğrulama "güvenilir", yayım kapısı (`release_check.sh`, R1–R20) geçer; insan `vX.Y.Z` etiketini laboratuvarın birleşmiş `main`'ine atar (bugünkü insan adımı 4).
2. İnsan laboratuvarda `transfer.yml`'i elle tetikler (`workflow_dispatch`, girdi: etiket). İş, aktarım aracını (laboratuvarda `research/transfer/`, Python — çalışma zamanı değildir; `classify.py`'nin desen mantığını paylaşır) etiketin ağacına uygular: giden dosyalar + K + M + Y → ürün ağacı; `TRANSFER.json` (her ürün dosyası: yol, sha256, kaynak — `lab:<yol>@<sha>` ya da `overlay:<yol>`, dönüşüm türü) ağaca eklenir.
3. Araç ürün deposunda `transfer/vX.Y.Z` dalına **tek commit** yazar (ebeveyni ürün `main`'i; mesaj `rootmark X.Y.Z` + `Lab-Commit: <sha>`, `Lab-Tag: vX.Y.Z`, `Transfer-Tree: <ağaç kimliği>`) ve PR açar. Yetki: ürün deposuna yalnız bu dala yazabilen bir GitHub App ya da dağıtım anahtarı (gizli değişken adı `PRODUCT_REPO_DEPLOY_KEY`; değer laboratuvarın Actions gizlisinde).
4. Ürün deposunda CI (§3.2) ve `transfer-verify` (§2.4) yeşil olunca insan PR'ı **hızlı ileri / rebase** ile birleştirir (ağaç değişmez); ürün deposunda `vX.Y.Z` etiketini atar ve ürünün `release.yml`'ini tetikler (ikililer, köken tasdiki, GitHub sürümü, imaj; CHANNELS).

### 2.2 Geçmiş politikası — **temiz geçmiş** (öneri)
Ürün deposunun ilk commit'i ilk aktarılan sürümdür (öneri: 0.3.3 ya da D-1'in sürümü); her sürüm bir commit ekler. **Gerekçe:** (a) laboratuvar geçmişi her commit'te `reports/` ve `research/` taşır; süzülmüş geçmiş (`git filter-repo --paths-from-file allowlist`) bugünkü izin listesini geçmişin her commit'ine uygular, oysa liste zamanla değişti (ör. `mdkernel` → `rootmark` yeniden adlandırması, K-P2Q; prototip alt ağacı, P1-A) — geçmişte bugün kalan sınıftaki bir yolun giden bir yolla aynı adı taşıması sızıntı denetimini her commit için yeniden yapmayı gerektirir; (b) bayt eşitliği tek bir ağaç karşılaştırmasına iner; (c) köken kaybolmaz: her ürün commit'i `Lab-Commit`/`Lab-Tag` taşır, laboratuvar herkese açıktır. **Alternatif:** süzülmüş geçmiş (`git filter-repo`), yalnız her commit için sızıntı denetimi ve yazar e-postalarının da süzülmesiyle; önerilmez (karar listesi D0-K5).

### 2.3 Ürün deposuna doğrudan katkı
Gerçek kaynak laboratuvardır. Ürün deposuna gelen bir PR insan tarafından laboratuvara taşınır (yazar bilgisiyle, `Co-authored-by`), bir sonraki aktarımla ürün deposuna döner; ürün PR'ı bağlantıyla kapatılır. Ürün deposunun `main`'i yalnız aktarım PR'larıyla değişir (dal koruması: aktarım dalından başka PR birleştirilmez). Alternatif (karar listesi): ürün deposunda yalnız issue açık, PR'lar laboratuvara.

### 2.4 Bayt eşitliği denetimi
**Değişmez:** `ürün commit'inin ağacı` = `aktarım aracı(laboratuvar etiketi)`'nin ağacı. Denetim `git write-tree` (ya da `git rev-parse <commit>^{tree}`) kimliklerinin eşitliğidir; eşit değilse ilk farklı 20 yol ve sha256'ları yazılır. Üç yerde koşar: (1) laboratuvarın `transfer.yml`'i commit'i yazdıktan sonra kendi çıktısını yeniden üretir; (2) ürün deposunun `transfer-verify.yml`'i (PR ve etiket): laboratuvarı `Lab-Commit`'te klonlar (herkese açık), aracı koşar, ağaç kimliğini karşılaştırır; (3) bağımsız doğrulayıcı (D-1 VERIFY). Ek denetim: `TRANSFER.json`'daki her `lab:` satırının sha256'sı laboratuvar etiketindeki blob'a eşit (K ve aynen gidenler), her `overlay:` satırı kaplamaya eşit; ürün ağacında `TRANSFER.json`'da olmayan dosya 0. `classify.py`'nin sızıntı denetimi aktarım aracının ilk adımıdır (sınıflandırılmamış ya da sızıntı > 0 → aktarım durur).

### 2.5 İnsan adımları (aktarım)
Etiket atmak (laboratuvar ve ürün), `transfer.yml`'i tetiklemek, aktarım PR'ını birleştirmek, ürün `release.yml`'ini tetiklemek, dağıtım anahtarını ya da GitHub App'i kurmak (anahtarın değeri yalnız gizli değişkende).

## 3. Ürün deposunun yeri ve CI'ı

### 3.1 Kurum ve depo adı
**Öneri:** GitHub kurumu **`rootmark-ai`**, ürün deposu **`rootmark-ai/rootmark`**. Gerekçe: `rootmark` GitHub adı 2018'den beri boş bir kullanıcı hesabında (R1, `github` × `rootmark`: dolu, 0 depo) — devralma GitHub'ın elindedir ve öngörülemez; `rootmark-ai` boş (R1) ve alan adı `rootmark.ai` ile aynı; `ghcr.io/rootmark-ai/rootmark` bu kurumla gelir. **Alternatif 1:** `rootmarkhq` (boş, tiresiz; Docker Hub kurum adıyla aynı yazılabilir — Docker kimliği tire almaz). **Alternatif 2:** GitHub'dan `rootmark` adının devri için başvuru (GitHub'ın "name squatting" politikası; sonuç ve süre belirsiz, D-1'i bekletmemeli). Kurum açmak insan adımıdır.

### 3.2 Hangi iş akışı nerede
| İş akışı / iş | Ürün deposu | Laboratuvar | Not |
|---|---|---|---|
| `ci.yml` `deps`, `test`, `test-linux` (cargo test/clippy/fmt, conformance, istemciler, `check_deps`, `check_dep_age`) | **evet** (Y: ürün `ci.yml`) | evet | ürün PR'ları ve aktarım PR'ı |
| `ci.yml` `fast`, `fast-compare` (kademe 1: `verify.sh chain-fast`, `eval.compare_p2r`, `eval.portable`) | hayır | **evet** | dondurulmuş ölçüm |
| `ci.yml` `embed-cpu`, `embed-cpu-compare` (gömme işlemciden bağımsızlık) | hayır (ürün testi `index/tests/embed.rs` `testdata/embed_ref.json` ile koşar) | **evet** | |
| `verify.yml` (kademe 2, haftalık), Mac kümesi (`make verify-mac`, yerel) | hayır | **evet** | |
| `pilot.yml` (D-NOTES) | hayır | **evet** | kişisel olmayan ama laboratuvar korpusu |
| `release.yml` (ikililer, kurulum, `SHA256SUMS`, köken tasdiki) | **evet** (M: depo adı) | hayır (D-1'den sonra laboratuvar yalnız `release.sh`'i yerel yayım kapısı için koşar) | `gh attestation verify -R rootmark-ai/rootmark` |
| `pages.yml` (spec sitesi + rozet) | **evet** → belge sitesi (CHANNELS §6) | hayır | |
| imaj (`ghcr.io`, Docker Hub) | **evet** (yeni, D-1) | hayır | CHANNELS §3 |
| `transfer.yml` | hayır | **evet** (yeni, D-1) | §2.1 |
| `transfer-verify.yml` | **evet** (yeni, D-1) | hayır | §2.4 |
| CodeQL, Dependabot | **evet** | evet | |

### 3.3 Sürüm etiketleri ve GitHub sürümleri
0.4.0'dan (ya da D-1'in ilk sürümünden) itibaren **kullanıcıya dönük sürüm ürün deposundadır**: `vX.Y.Z` etiketi, GitHub sürümü (arşivler, kurulum betikleri, `SHA256SUMS`, köken tasdiki, wheel, eklenti dosyaları), kurulum URL'leri `https://github.com/rootmark-ai/rootmark/releases/download/v…`. Laboratuvar aynı adlı etiketi aktarımın kaynağı olarak tutar (R14), GitHub sürümü yayımlamaz. Eski sürümler (0.1.0–0.3.2) laboratuvarda kalır; laboratuvar yeniden adlandırılırsa GitHub eski URL'leri yeni ada yönlendirir (bir depo eski adla yeniden açılmadıkça) — bu yüzden **eski ad (`arknsensei/rootmark`) bir daha depo adı olarak kullanılmaz**. 0.3.2'nin `install.sh`'i ve kılavuzu `arknsensei/rootmark`'a gider; yönlendirme onları çalışır tutar.

### 3.4 GitHub Pages bağlantıları (`arknsensei.github.io/rootmark/spec/`)
Bugün README'nin uyumluluk rozeti ve spec bağlantısı laboratuvarın Pages sitesine gider (`pages.yml`). Proje Pages adresi depo adına bağlıdır ve yeniden adlandırmada **yönlendirilmez**. **Öneri:** spec ve rozet ürün deposunun belge sitesine taşınır (`https://docs.rootmark.ai/spec/` — CHANNELS §6); laboratuvar yeniden adlandırılmadan önce son bir `pages.yml` koşusu eski adreste yeni adrese yönlendiren statik bir sayfa yayımlar (`<meta http-equiv="refresh">` + bağlantı); yeniden adlandırmadan sonra eski adres 404 olur ve 0.3.2'nin README'sindeki rozet kırılır (bilinen sınır; CHANGELOG notu). **Alternatif:** laboratuvar yeniden adlandırılmaz (§4), Pages olduğu yerde kalır ve yönlendirme sayfası orada sürer.

## 4. Laboratuvar deposunun kaderi
**Öneri:** `arknsensei/rootmark` → **`arknsensei/rootmark-lab`**, **herkese açık** kalır (köken, doğrulama kayıtları, kod yorumlarının atıfları ona çözülür; "kişisel içerik makineden çıkmadı" kuralları zaten kayıtları içeriksiz tutuyor). Gerekçe: tireli ad GitHub ve ürün adıyla uyumlu; `rootmark_dev` (K-D0.1'in örneği) alt çizgili, alan adı ve kayıt defteri kurallarında taşınmaz. GitHub eski adı (git, sürüm varlıkları, issue/PR) yönlendirir. **Alternatif 1:** laboratuvarı `rootmark-ai` kurumuna taşıyıp `rootmark-ai/rootmark-lab` yapmak (tek kurum altında; yönlendirme aynı). **Alternatif 2:** yeniden adlandırmamak — ürün deposu `rootmark-ai/rootmark`'ta olduğu için ad çakışması yok; bedeli iki "rootmark" deposunun karışması. Görünürlük: özel yapmak eski sürümlerin indirme URL'lerini ve köken tasdiklerinin doğrulanmasını bozar → önerilmez.

## 5. Lisans ve üçüncü taraf
- **Kod, istemciler, spec:** Apache-2.0 (`LICENSE`, `spec/LICENSE`); `NOTICE` aynen gider. Ürün deposunun `NOTICE`'u laboratuvarınkiyle aynıdır (kaynak aynı).
- **Modeller** (`transfer/model_licenses.json`, Hugging Face API, sabit revizyonlarda): `sentence-transformers/all-MiniLM-L6-v2` @ `1110a24…` — kart lisansı **apache-2.0**, taban model `nreimers/MiniLM-L6-H384-uncased`; `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` @ `e8f8c21…` — kart lisansı **apache-2.0**. Sabitlenmiş dosyaların boyutu 90,9 MB ve 479,4 MB. Bugün model hiçbir arşivde dağıtılmaz; kurulum indirir (`NOTICE`).
- **Model imajda ise** (CHANNELS §3) imaj modelin **yeniden dağıtımıdır**: Apache-2.0 §4 gereği lisans metni ve atıf imajda (`/usr/share/rootmark/models/<ad>/LICENSE` ve `NOTICE` satırı: kaynak, revizyon, lisans) bulunmalı; model kartının ve taban modelin lisansı D-1'de bir kez daha okunup kayda geçer. **İlk açılışta indirilirse** dağıtım yoktur (bugünkü durum), ama imaj ağsız çalışamaz.
- **İkilinin üçüncü taraf lisansları:** ikili Rust bağımlılıklarını ve `ort`'un bağladığı ONNX Runtime'ı (MIT) içerir; bugün arşivlerde bunların bildirimi yok. D-1: `THIRD_PARTY_LICENSES` (ör. `cargo about generate`, sürüm başına), arşivlere ve imaja eklenir (ürüne yeni giren dosya, §1.4). Bu, mevcut 0.3.2 arşivleri için de geçerli bir eksiktir (bulgu D0F-2).

## 6. İnsan adımları (D-1'e devredilen; ajan yapmaz)
1. GitHub kurumu `rootmark-ai`'ı açmak (karar D0-K3'e göre), kurumun güvenlik ayarları (2FA zorunlu, varsayılan izin okuma).
2. Ürün deposunu açmak (boş), dal koruması (`main`: PR + CI + doğrusal geçmiş; yalnız aktarım dalları).
3. Laboratuvara aktarım yetkisi: GitHub App ya da dağıtım anahtarı → laboratuvar Actions gizli değişkeni `PRODUCT_REPO_DEPLOY_KEY`.
4. Laboratuvarın yeniden adlandırılması (karar D0-K4), yeniden adlandırmadan önce Pages yönlendirme sayfası.
5. Her sürümde: laboratuvar etiketi, `transfer.yml` tetikleme, aktarım PR'ını birleştirme, ürün etiketi, ürün `release.yml`.
