# mdkernel-v2 — Yönerge Paketi

Markdown-doğal, blok düzeyinde versiyonlu, yapı-farkında vektör arama çekirdeği için ikinci hipotez doğrulama turu. Çekirdek Rust, arama ve ölçüm Python. Kod içermez; ajan kodu bu yönergelerden üretir.

## Dosyalar
| Dosya | Ne için |
|---|---|
| `CLAUDE.md` | Ajan kuralları (otomatik yüklenir) |
| `docs/00_GOAL.md` | 4 hipotez, önceden kayıtlı eşikler, karar kuralı |
| `docs/01_SPEC.md` | Veri modeli, eşleyici v2, Rust API/CLI, arama stratejileri |
| `docs/02_TASKS.md` | 20 görev (T0–T19), kabul ölçütleri |
| `docs/03_EVAL.md` | Veri manifest'leri, içerik-referanslı gerçekler, metrikler |
| `docs/04_VERIFY.md` | Bağımsız doğrulama (çift koşturma, kasıtlı bozma) |
| `docs/05_REPORT_TEMPLATE.md` | Rapor şablonu |
| `docs/06_LESSONS.md` | v1 dersleri → v2 kuralları (gerekçe) |

## Ortam
Rust stable + cargo, Python 3.11+, `uv`, `maturin`, git, internet (crates.io, PyPI, GitHub, HuggingFace), `ANTHROPIC_API_KEY` (yalnız T12; yoksa T12 park edilir, geri kalan ilerler). Tahmini süre v1'in 2–3 katı (Rust + bağlama).

## Kullanım
1. Boş git deposu köküne kopyala.
2. Claude Code'da ana istemi ver.
3. Ajan `docs/progress/T18.md` sonunda "T19 için hazır" yazıp durunca, **ayrı oturum + temiz klon** ile doğrulama istemini ver. Doğrulayıcıya ana depo yolunu **verme**; yalnız git uzak adresi veya klonlanabilir yol.
4. `reports/FINAL.md` + `reports/VERIFICATION.md` ile dön.

## Ana ajan istemi
```
Bu depo bir hipotez doğrulama prototipidir (v2). Önce CLAUDE.md, sonra docs/00_GOAL.md ve docs/06_LESSONS.md'yi oku.
docs/02_TASKS.md görevlerini T0'dan sırayla yürüt; park kuralı dışında atlama yok.
Her görevde CLAUDE.md görev disiplinini uygula: progress kaydı, cargo+pytest testleri, gözden geçirici alt-ajan, commit.
docs/00_GOAL.md eşiklerini ve T9 sonrası dondurmayı ihlal etme; dondurma istisnası yalnız CLAUDE.md'deki üç şartla.
Hiçbir koşturma sessizce sıfır/boş sonuç üretmesin; çözülemeyen gerçek, eksik dosya, manifest uyuşmazlığı hata olsun.
Hipotez başarısızsa optimize etme; raporla, devam et.
Durma koşulunda STOP.md yaz, görevi PARK et, bağımsız görevlere devam et; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz.
T18 bittiğinde docs/progress/T18.md sonuna "T19 için hazır" ve RUN kimliğini yaz, dur. T19'u sen yapma.
```

## Doğrulama ajanı istemi (ayrı oturum, temiz klon)
```
Sen bağımsız doğrulama ajanısın; geliştirme oturumunun bağlamına ve ana depo dizinine erişimin yok, varsa bile kullanma.
docs/04_VERIFY.md'yi oku, V1'den V10'a sırayla uygula; scripts/verify.sh'i iki ayrı dizinde koştur. Kod değiştirme.
Tek çıktın reports/VERIFICATION.md.
```

## Sonraki aşama
Tüm hipotezler geçerse `01_SPEC.md` §1–7 dondurulur ve çekirdek aşaması (sync, plugin yüzeyi, sunucu) bu Rust crate'in üzerine kurulur; prototip kodu bu kez atılmaz.
