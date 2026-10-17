# P1-E.1 — Faz 1 Kapanışı ve 0.1.0

Kısa tur. P1-E doğrulamasının koşulunu (`status` sözleşme ihlali) ve üç önerisini kapatır, K-E5-T'yi karara bağlar, yayımı insan adımına hazırlar. Yeni özellik yok.

## Kararlar (E1.0'da DECISIONS.md'ye)
- **K-E1.1 K-E5-T kapanır: 0.1.0 hedefi yalnız `aarch64-apple-darwin`.** Gerekçe: Apple Silicon kullanıcı tabanının büyük çoğunluğu; Intel macOS için `ort` ön-derlenmiş kütüphane yok ve `load-dynamic` "tek ikili" sözünü bozar. Linux x86_64 ve Windows x86_64 CI'da derlenmeye devam eder, `conformance` koşturulur, artefakt saklanır ama **yayım kriteri değildir**; kurulum paketleri (installer, imzalama, dağıtım kanalları) faz 2 "Setup" turunun konusudur. `RELEASE.md` ve `PACKAGES.md` buna göre güncellenir; E-7 kriteri bu kararla "1/1 + CI bilgi" olarak yeniden okunur (karar kaydıyla, GOAL değişmez).
- **K-E1.2 `status` indeksten bağımsızdır.** Sözleşme §3: `status` her durumda 200 döner; indeks durumu `index: {state: ok|stale|missing|damaged, format, expected_format, auto_rebuild}` alanında. `index_stale` yalnız indeks gerektiren çağrılarda. E-14 testi gerçek eski biçimle (`p1d-final` ikilisinin yazdığı `/2` indeksi fixture olarak commit'lenir) kurulur ve `status`'u da ölçer.
- **K-E1.3 Gerçek Obsidian e2e resmi kapıdır.** Doğrulayıcının yöntemi (ayrı `--user-data-dir`, CDP, kısıtlı kipten API ile çıkış) `clients/obsidian/e2e/real/` olarak betiklenir; E-2 bu harness'le ölçülür; komut katmanı harness'i birim test olarak kalır. Obsidian ikilisi CI'da yoksa bu adım yerel doğrulama ve yayım kontrol listesinde zorunlu, CI'da atlanır ve atlandığı raporlanır. NORTH_STAR/ARCHITECTURE "plugin e2e" ifadesi buna göre düzeltilir (ASSUMPTIONS E4-1 kapanır).
- **K-E1.4 Bilinen FAIL'ler temizlenir.** Eski kaydedicilerin (`C-3`, `C-11`, `D-2`, `D-7`, `D-9`) birikmiş FAIL'leri rapor üreticilerde "tarihsel, karar K-… ile kapandı" etiketiyle gösterilir; `verify.sh` özetinde sayılmaz.

## Çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| E1-1 | Gerçek `/2` indeks + `auto_rebuild=false`: `status` GET/POST/MCP 200 ve `index.state=stale`; `search` 503 `index_stale`; `blame/checkpoints/trace` 200; `auto_rebuild=true` → hepsi 200 | 8/8 |
| E1-2 | Okunamayan manifest (bozuk JSON): `status` 200 `damaged`; `search` 503 | 2/2 |
| E1-3 | Gerçek Obsidian e2e harness 11 adım (yerel) | 11/11 |
| E1-4 | `release_check.sh` R7 SIGPIPE düzeltmesi; kontrol listesi tüm maddeler betikle | geçer |
| E1-5 | `release.yml` insan tetiklemeli; `aarch64-apple-darwin` artefaktı + `dist/release.json` (hash, `--version`, `--spec-version`, `mdk conformance` 34/34); Linux/Windows artefaktları varsa aynı kayıt, yoksa "atlandı" | 1/1 zorunlu |
| E1-6 | Uyumluluk 34/34; P1-E kalite alanları regresyon | fark 0 |
| E1-7 | Çift koşturma | fark 0 |
| E1-8 | `contract_check.sh` HTTP adımı `status`'un yeni alanını ve 503 istisnasının `status`'u kapsamadığını denetler | geçer |
| E1-9 | `docs/CLIENT_CONTRACT.md` §3 güncel; plugin bağlantı ekranı `stale/damaged` durumunu ve yeniden kurma yolunu gösteriyor (e2e adımı) | 1/1 |

## Karar kuralı
E1-1..E1-9 geçti ∧ doğrulama "Güvenilir" → **insan adımı:** `git tag v0.1.0`, `release.yml` tetikleme, `RELEASE.md` kontrol listesi. Faz 1 kapanır. Tanımlanmamış kalış → tur kaldı, karar talebi.

## Görevler
- **E1.0** `git tag p1e-final`; `ROUND_START`; DECISIONS (K-E1.1..4); `RELEASE.md`, `PACKAGES.md` hedef güncellemesi; NORTH_STAR/ARCHITECTURE ifade düzeltmesi (yalnız kanıt durumu ve e2e tanımı).
- **E1.1** `server`: `status` indeksten bağımsız; `index.state`; `index_stale` kapsamı; sözleşme §3; fixture `/2` indeks (`p1d-final`'den derlenen `mdk` ile, `scripts/gen_old_index.sh`); E1-1/E1-2 testleri.
- **E1.2** `clients/obsidian`: bağlantı ekranında indeks durumu ve yeniden kurma yönlendirmesi; `e2e/real/` CDP harness'i (doğrulayıcının adımları `reports/rounds/P1-E/VERIFICATION.md` V3'te); E1-3.
- **E1.3** `scripts/release_check.sh` R7; `release.yml` (`workflow_dispatch`, matris: aarch64-apple-darwin zorunlu, diğerleri `continue-on-error`), `dist/release.json` üretimi; `mdk conformance` her artefaktta.
- **E1.4** Rapor üreticilerde tarihsel FAIL etiketi (K-E1.4); `verify.sh` özeti.
- **E1.5** `verify.sh` genişletme; `report_p1e1.py`; `FINAL.md`; `READY.md`.

## Yapılmayacaklar
Yeni özellik; sözleşmede `status` dışında değişiklik; tasdikli yazma (P2-0); Intel macOS; installer.
