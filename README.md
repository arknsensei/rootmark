# rootmark — Ana Belge Seti

Bu paket, prototip döneminin (v1–v3) yerine geçen **tek gerçek kaynaktır**. Bundan sonra her Claude Code turu ve her yeni sohbet yalnız bu belgelerden beslenir; bu belgeleri üreten sohbet arşivdir (K13).

## İçerik
| Dosya | Rol |
|---|---|
| `NORTH_STAR.md` | Tek sayfa sabit nokta: amaç, vaat, kim için, ilkeler, kanıtlananlar |
| `spec/SPEC.md` | Yayımlanabilir spesifikasyon v1.0 (İngilizce, Apache-2.0); §1–6 donmuş |
| `spec/CONFORMANCE.md` | Uyumluluk testleri (3 seviye) |
| `docs/ARCHITECTURE.md` | Mimari v1.1: katmanlar, kanıt tablosu, kararlar |
| `docs/PACKAGES.md` | Monorepo, yedi paket, dil kararları, dağıtım |
| `docs/ROADMAP.md` | Faz 1 (5 tur), faz 2, araştırma kolları |
| `docs/DEV_PROCESS.md` | `/goal` tur anayasası, kalite kapıları, dondurma, doğrulama |
| `docs/DECISIONS.md` | K1–K13 karar kayıtları |
| `CLAUDE.md` | Kök ajan kuralları |
| `docs/rounds/P1-A/` | İlk turun GOAL, TASKS, VERIFY, PROMPT dosyaları |

## Kurulum
1. Yeni boş depo `rootmark/` aç; bu paketi köke kopyala.
2. Prototip deposunda `git tag v3-final` at (son commit v3 raporu ve doğrulamayı içermeli); uzak adresini ya da yolunu `docs/rounds/P1-A/progress/SOURCE.md` için hazır tut (A0 görevi subtree ile içe alır; yol/adresi ajan istemine ekle).
3. `git add . && git commit -m "rootmark: master docs, spec v1.0, round P1-A directives"`.
4. Claude Code'u depo kökünde aç; `docs/rounds/P1-A/PROMPT.md` ana istemini `/goal` ile ver (prototip depo yolunu istemin sonuna ekle: "Prototip deposu: <yol veya URL>").
5. READY.md gelince ayrı oturumda doğrulama istemini ver.
6. `reports/rounds/P1-A/FINAL.md` + `VERIFICATION.md` ile dön; P1-B yönergeleri ona göre yazılır.

## Sonraki turların yönergeleri
Her tur için `docs/rounds/<id>/` aynı dört dosyayla üretilir; şablon DEV_PROCESS §1. P1-B..P1-E içerikleri ROADMAP'te özetli; her tur başlamadan önce GOAL/TASKS/VERIFY/PROMPT yazılır ve tur başladıktan sonra değişmez.
