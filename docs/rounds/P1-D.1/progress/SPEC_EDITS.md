# P1-D.1 — Spec düzenlemeleri

Ölçüt: `docs/SPEC_EDITS_CRITERION.md`. (a) hesaplanan hiçbir değeri değiştirmez; (b) daha önce uyumlu bir uygulamayı uyumsuz kılmaz. Sürüm, CRITERION'ın "Sürüm numarası (K-D1.9)" kuralıyla seçilir; GOAL sürüm öngörmez.

| # | Bölüm (durum) | Düzenleme | (a) | (b) | Sınıf | Karar |
|---|---|---|---|---|---|---|
| E1 | §4.2 (Frozen) | Bilgilendirici not: §7 değişiklik/silme olaylarına `meta.actor`/`meta.source` ekler, §8 `CheckpointNamed` türünü ekler. Burada tanımlı alanlar, türler, anlamları ve kimlikler değişmez. (D1.4 gözden geçirmesi: önceki ifade "hiçbir olay değişmez" E3 ile çelişiyordu.) | sağlar | sağlar | açıklık (bilgilendirici not) | K-D1.2, K-D1.6 |
| E2 | §4.3 (Frozen) | Bilgilendirici "provenance of events" notu. Değişikliği ya da silmeyi kimin yaptığını §7 olayların `meta`'sına yazar. Sürümlerin kökeni değişmez. | sağlar | sağlar | açıklık (bilgilendirici not) | K-D1.2 |
| E3 | §7 (Validated) | Normatif: yönetilen bir yazma yolunun yazdığı `BlockUpdated/Moved/Split/Merged/Deleted` olayları `meta.actor {kind, id}` ve `meta.source` taşımalıdır (ikisi birlikte ya da hiçbiri). `Promoted` ve `Demoted` dışındaki başka türler bunları taşıyamaz. Bu ikisinin `meta.actor`'ı geçişin aktörüdür ve kendi kuralına tabidir. Okuyucu yokluğu kabul etmeli; bozuk, eşsiz ya da başka türdeki değeri reddetmeli; aktör uydurmamalıdır. | **sağlamaz** (başvuru uygulamasının sunucu yolundan yazdığı olay baytları değişir) | **sağlamaz** (aktör yazmayan bir sunucu önceki metinle uyumluydu; bozuk `meta.actor`'ı yok sayan bir okuyucu da) | anlam değişikliği | K-D1.2 |
| E4 | §7 (Validated) | Bilgilendirici not: düzenleme hook'u; başvuru varsayılanı `append_only`; C3-06. | sağlar | sağlar | açıklık | K-D1.1 |
| E5 | §8 (Validated) | Normatif: adsız bir checkpoint sonradan bir kez `CheckpointNamed` ile adlandırılabilir. Kimlik, `event_seq`, `docs` ve nesne değişmez. Ad tektir ve kimlik biçiminde olamaz. Kurala aykırı adlandırma reddedilir (replay dahil). İçerik olayı değildir. | sağlar (var olan değerlerin hiçbiri değişmez) | **sağlamaz** (önceki metne göre checkpoint ancak oluşturulurken adlandırılabiliyordu. Artık kurala aykırı bir `CheckpointNamed`'i kabul eden ya da adlandırmayı `event_seq`'e sayan bir uygulama uyumsuz) | anlam değişikliği | K-D1.6 |
| E6 | §9 (Validated) | Normatif: yanıt uzunluğu — katman başına ≤ k, toplam ≤ 2k. | sağlar (başvuru uygulaması zaten böyle döndürüyordu) | **sağlamaz** (önceki metin yalnız bilgilendirici notta "up to k of each" diyordu; daha uzun yanıt veren bir uygulama uyumluydu) | anlam değişikliği | K-D1.8 |
| E7 | §9 (Validated) | Bilgilendirici not: `lexical.dl = "exact token count, f64, no fieldnorm quantization"`. | sağlar | sağlar | açıklık | K-D1.7 |
| E8 | başlık/durum satırı | `v1.0.3` ve 1.0.3 değişiklik özeti. Bölüm durumları değişmez. | sağlar | sağlar | açıklık | — |

## Sürüm
- Açıklık olmayan satırlar: E3, E5 ve E6. Hepsi Validated bölümlerde (§7, §8, §9). Bilinen ya da başvuru uygulamasının yazdığı hiçbir verinin okunması bozulmuyor: aktörsüz günlükler okunur (E3, D1-11: P1-D ve v3-final vault'ları), `CheckpointNamed`'siz günlükler değişmez (E5). Bu yüzden CRITERION kuralı: **yama**, `1.0.2` → **`1.0.3`**.
- **Açık nokta (D1.4 gözden geçirmesi, karar talebi K-D1.4-T):** 1.0.2'de §4.2 `meta`'ya şema vermiyordu. Varsayımsal bir 1.0.2 uygulaması bir değişiklik olayına biçimsiz bir `meta.actor`/`meta.source` (örneğin dize) ya da başka bir türe bu anahtarları yazmış olabilirdi. Böyle bir günlük 1.0.3 okuyucusunda reddedilir. CRITERION'ın majör maddesi ("daha önce uyumlu bir uygulamanın verisini okunamaz kılan") bu durumda yorum ister: madde, olası her veriyi mi kapsar, yoksa bir uygulamanın gerçekte yazdığı veriyi mi? Bu tur **yama**yı seçti. Gerekçeler:
  - (i) başvuru uygulaması 1.0.3'ten önce bu anahtarları hiç yazmadı; C1/C2 fixture'ları ve D1-11 bunu doğruluyor;
  - (ii) bilinen başka bir uygulama yok;
  - (iii) reddetme sessiz başarısızlığı önler (bozuk bir aktör okunup uydurulmaz).

  Seçim etiketli varsayımdır (ASSUMPTIONS D1.4-1), kesinleşmesi DECISIONS K-D1.4-T'dedir. Majör seçilirse sürüm 2.0.0 olur ve hiçbir davranış değişmez.

  **Not (2026-10-17, P1-E/E0, K-E0.1):** K-D1.4-T yama lehine kapandı. CRITERION'daki "veri" bir uygulamanın **yazdığı** veridir. İki yönde de okunabilirlik korunur: 1.0.2 okuyucuları yeni `meta` alanlarını (`actor`, `source`) yok sayar, çünkü §4.2 `meta`'yı açık bir JSON nesnesi olarak tanımlar; hiçbir biçim okunamaz hale gelmez. 1.0.3 okuyucusu yalnız biçimsiz bir `meta.actor`'ı reddeder; böyle bir veriyi yazmış bilinen bir 1.0.2 uygulaması yoktur. Sürüm 1.0.3 kalır.
- Frozen bölümlerde (§4.2, §4.3) yalnız bilgilendirici not var (E1, E2), yani açıklık. CRITERION'a göre majör gerekmez. §1–3 ve §5–6'da düzenleme yok (PROMPT kırmızı çizgisi).
- Uyumluluk: Seviye 2'ye C2-10 (E3), Seviye 3'e C3-06 (E4'ün başvuru davranışı, §9 değişmezliği) ve C3-07 (E6) eklendi (`spec/CONFORMANCE.md`). Var olan fixture'ların hiçbiri değişmedi.
- Not (indeks biçimi): K-D1.7 indeks manifest biçimini `mdkernel-index/3`'e çıkarır. Bu bir spec düzenlemesi değil, başvuru uygulamasının türev indeks biçimidir. Eski indeks açıkça reddedilir ve `mdk index build` ile yeniden kurulur. Vault ve günlük etkilenmez.
