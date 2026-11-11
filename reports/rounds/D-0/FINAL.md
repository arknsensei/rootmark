# D-0 — Sonuç (FINAL)

**Tur:** D-0 — Dağıtım: isim, depo ve aktarım planı (karar + rapor turu; K-D0.1, K-P2Q2.2, K-D0.2). `ROUND_START` = `b6f5b7b`. Dal `d-0/round`.

**Özet:** "rootmark" ve ilgili sekiz ad 24 kayıt defteri sütununda tarandı (216/216 hücre kanıtlı); marka taraması dört kaynakta insan adımıyla; bugünkü `main`'in 16 584 dosyası izin listesiyle sınıflandı (sınıflandırılmamış 0, sızıntı 0, giden küme tek başına derleniyor) ve aktarım planı yazıldı; kod adları ve D-1'in kanalları (gizli değişkenlerin yalnız adları) planlandı; **PQ2-1 kalıcı** (macOS ve Linux'ta 40/40) ve kök nedeni daha geniş bir hata (D0F-5: başlangıçta uzlaştırma yok); ürün README iskeleti; PQ2-4 düzeltildi; ürün sahibi için 18 maddelik karar listesi (`REPORT.md` §7).

## Kriterler ("koştu" ve "geçti" ayrı)

| Kod | Ölçüm | Eşik | Koştu | Sonuç | Kanıt |
|---|---|---|---|---|---|
| D0-1 | `DESIGN.md` R1'in ilk tarama kaydından önce commit'li | önce | evet | **geçti** — DESIGN `7306155` (R0), ilk tarama kaydı `1fc1123` (R1) | `git log` |
| D0-2 | her ad × kayıt defteri hücresi kanıtlı; erişilemeyenler nedeniyle; betikler depoda, yeniden koşulabilir | eksik 0; kanıtsız 0 | evet (iki koşu) | **geçti** — 216/216, eksik 0, kanıtsız 0; erişilemedi 4 (npm kullanıcı sayfası 403, nedeniyle; H1) | `registry_scan.json`, `research/d0/registry_scan.py` |
| D0-3 | dört marka kaynağı için kanıtlı satır; "hukuki görüş değildir" notu | 4/4; not var | [H2] | [H2] | `trademark.md`, `trademark_probe.json` |
| D0-4 | izin listesi her dosyayı sınıflandırıyor; giden kümede laboratuvar yolu, kişisel içerik, `.rootmark/`, anahtar yok (betikle); yöntem, geçmiş politikası, bayt eşitliği, insan adımları yazılı | 0; 0; yazılı | evet | **geçti** — 16 584 → 589 / 15 351 / 644, sınıflandırılmamış 0, çift 0; sızıntı 0; kırık bağlantı 0; giden küme `cargo build --release` + iki istemci ok; TRANSFER_PLAN §1–6 | `transfer/classification.json`, `export_check.json`, `TRANSFER_PLAN.md` |
| D0-5 | her kod adı için öneri ve göç notu; her kanal için ad, insan adımları, gizli değişken adları, yayım yöntemi; P2-Q.2'nin üç önerisi D-1 planında | hepsi yazılı | evet | **geçti** | `CHANNELS.md` §1, §2, §5, §6 |
| D0-6 | PQ2-1 macOS ve Linux'ta ≥ 10 tekrar; sonuç sayılarla; D-1'den önce düzeltme önerisi gerekçeli | koştu; sayılarla | evet | **geçti** — her platformda {sil, taşı} × {SIGKILL, SIGTERM} × 10 = 40; SIGKILL 20/20 kalıcı, SIGTERM 20/20 alındı (iki platform); öneri: 0.3.3, D-1'den önce (D0-K10) | `pq2_1/*.json`, `REPORT.md` §5 |
| D0-7 | karar listesi; README iskeleti; PQ2-4; ürün sınırı; dışarıya yazan eylem 0; `make test`, `make lint` temiz; PR'da kademe 1 yeşil; gözden geçirici kabul | hepsi | [R7] | [R7] | `REPORT.md` §7, §9; `README_SKELETON.md`; `server/CLAUDE.md` |

## Karar kuralının dalı
[R7]

## Bulgular
`FINDINGS.md`: D0F-1 (PQ2-1 kalıcı), **D0F-5 (sunucu kapalıyken yapılan değişiklikler başlangıçta alınmıyor; `ingest-dir` silmeyi almıyor)**, D0F-2 (arşivlerde üçüncü taraf lisans bildirimi yok), D0F-3 (HTTP yalnız loopback; konteyner), D0F-4 (`mdk_version`), D0F-6 (macOS FSEvents sınama notu), D0F-7 (kılavuzun Known issues metni), D0F-8 (test başvuru verisinde MIT atfı).

## Sonraki adım
Ürün sahibi `REPORT.md` §7'den kararlarını verir; orkestratör K-D0.x'i yazar. Öneri: önce 0.3.3 (PQ2-1 + D0F-5), sonra D-1. Kısa bağımsız doğrulama birleşmeden sonra (`docs/rounds/D-0/VERIFY.md`).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
