# Spec düzenlemeleri — "yalnız açıklık" ölçütü (K-B1.3)

`spec/SPEC.md`'de bir düzenleme ancak aşağıdaki iki koşulu **birlikte** sağlıyorsa **yalnız açıklık** sayılır:

- **(a)** Hesaplanan hiçbir değeri değiştirmez (kimlikler, hash'ler, olay baytları, sorgu sonuçları; spec'in harfi harfine okunmasıyla hesaplanan değer aynı kalır).
- **(b)** Daha önce uyumlu olan hiçbir uygulamayı uyumsuz kılmaz (düzenlemeden önceki metne göre geçerli olan bir yazım, serileştirme ya da davranış seçeneği düzenlemeden sonra da geçerlidir).

İkisinden biri sağlanmıyorsa düzenleme **anlam değişikliği** ya da **serileştirme kararı**dır:
- karar kaydı (`docs/DECISIONS.md`, `K…`) gerekir,
- spec sürümü artar (yama: hesaplanan değeri değişmeyen ya da yalnız Validated/Draft bölümleri etkileyen; majör: §1–6 ya da kırıcı değişiklik — docs/DEV_PROCESS.md §8),
- etkilenen uyumluluk fixture'ları yalnız o kararla yeniden üretilir.

Bilgilendirici not ("Informative: …") hiçbir uygulamayı bağlamadığı için (a) ve (b)'yi sağlar.

## Uygulama
Her tur, spec düzenlemelerini `docs/rounds/<tur>/progress/SPEC_EDITS.md` tablosunda satır satır kaydeder ve her satırı bu ölçüte göre sınıflandırır: **açıklık** / **serileştirme kararı** / **anlam değişikliği** (karar numarasıyla).

## Sürüm numarası (K-D1.9, P1-D.1)
Sürüm numarasını yalnız bu ölçüt belirler, GOAL belirlemez (`docs/DEV_PROCESS.md` §1). Bir turun düzenlemeleri sınıflandırıldıktan sonra sürüm şöyle seçilir:
- yalnız **açıklık** satırları → sürüm değişmez;
- en az bir **serileştirme kararı** ya da **anlam değişikliği**, hepsi Validated/Draft bölümlerde ve kırıcı değil → **yama** (`1.0.x` → `1.0.x+1`);
- §1–6'da (Frozen) açıklık olmayan bir düzenleme ya da kırıcı bir değişiklik → **majör**.

**Kırıcı (K-E0.1, P1-E/E0):** önceki sürüme uyumlu bir uygulamanın **yazdığı** veri (log, indeks) yeni sürümün uygulaması tarafından okunamıyorsa, ya da yeni sürümün uygulamasının yazdığı veri önceki sürümün okuma kurallarıyla okunamıyorsa. "Veri", bir uygulamanın gerçekte yazdığı veridir, önceki spec'in yalnız izin verdiği her olası değer değildir. Açık bir yapıya (ör. §4.2 `meta` JSON nesnesi) yeni bir alan eklemek, eski okuyucular onu yok sayabildiği sürece kırıcı değildir; yeni alanı yazmayı zorunlu kılmak yazarın yükümlülüğüdür. Örnek: 1.0.3'ün `meta.actor`/`meta.source`'u yama düzeyindedir (K-E0.1, K-D1.4-T).

**Bilinmeyen alan (K-P2-02.3, P2-0.2; P2-0.1 doğrulaması F4):** "eski okuyucular yok sayabildiği sürece" spec'te **yazılı** bir okuma kuralı demektir: yapı ya açıkça **açık** tanımlıdır ya da okuyucusuna "bilinmeyen alan yok sayılır" der. Böyle bir yapıya alan eklemek kırıcı değildir. **Açık yapı:** §4.2'nin `meta` nesnesi açıktır — §5 uygulamalara kendi parametrelerini `meta`'ya yazdırır ("MUST record `matcher_version` and parameters in `meta`"), dolayısıyla bir okuyucu tanımadığı `meta` anahtarlarıyla karşılaşmaya spec gereği hazırdır; K-E0.1'in 1.0.3 örneği (`meta.actor`/`meta.source`) bu yüzden yama düzeyinde kalır. Alanları spec'te tek tek tanımlı **kapalı** alt nesneler (ör. `meta.actor = {kind, id}`) ve ayrı dosyaların satırları (ör. §8'in tasdik günlüğü, iz satırları) açık değildir: onlar için okuma kuralı yazılı değilse ya da önceki sürümün **başvuru okuyucusu** bilinmeyen alanı reddediyorsa (spec bunu istemese de), alan eklemek kırıcıdır — önceki sürümün başvuru uygulaması yeni sürümün yazdığı veriyi açamaz. Kırıcı olmaması için önce okuma kuralı ("bilinmeyen alan yok sayılır; bilinen alanın tanımı dışındaki değeri reddedilir") spec'e yazılır ve başvuru okuyucusu ona uydurulur; kural yürürlüğe girdikten sonraki alan eklemeleri yama düzeyindedir. **Bilinen bir alana yeni değer** eklemek (bir alanın tanımını genişletmek: yeni bir katman, aktör türü ya da `partial: false`'a anlam vermek) her zaman kırıcıdır, çünkü okuma kuralı tanım dışı değeri reddeder (K-E0.1'in 2.0.0 gerekçesi). **Uyumluluk her adımda yalnız önceki sürüme göredir:** bir yama, iki ya da daha çok sürüm önceki okuyucunun yeni veriyi okuyabileceğini güvenceye almaz (ör. 2.0.1'in okuyucusu bilinmeyen alanı reddeder; 2.0.2'den sonra tasdik günlüğüne eklenen bir alanı yalnız 2.0.2 ve sonrası okur). **Emsal:** P2-0.1'in `partial`/`matched` eklemesi (2.0.1) bu netleştirmeye göre **kırıcıydı** (majör olurdu): tasdik günlüğü için yazılı okuma kuralı yoktu ve 2.0.0'ın başvuru okuyucusu (`ce4cf04`) bu alanları reddeder. 2.0.0 hiç yayımlanmadığı için (paket 0.1.0 = spec 1.0.3) **istisna olarak** yeniden sınıflandırılmaz (K-P2-02.3); kural P2-0.2'den itibaren uygulanır (§8 tasdik günlüğü okuma kuralı, 2.0.2).

Spec sürümünde ara (minör) düzey kullanılmaz (ASSUMPTIONS P1-D.1 D1.0-6): spec yalnız yama ve majör adım atar. Bu kural `rootmark` paket sürümünü bağlamaz; paket DEV_PROCESS §8'deki SemVer'e uyar. Frozen bir bölüme eklenen bilgilendirici not açıklıktır ve sürüm gerektirmez. `SPEC_EDITS.md`, seçilen sürümü bu listedeki maddeye atıfla yazar.

## Yeniden sınıflandırma (P1-B, K-B1.3)
| P1-B düzenlemesi | (a) | (b) | Sınıf |
|---|---|---|---|
| E1 — katman değerleri küçük harf | sağlar | **sağlamaz** (PascalCase katman yazan bir uygulama önceki metinle uyumluydu) | serileştirme kararı → K-B1.3, spec 1.0.1 normatif |
| E2 — `returned` nesne biçimi; `+` düz birleştirme; `at_seq` ondalık | sağlar | **sağlamaz** (dizi biçiminde demet yazan bir uygulama önceki metinle uyumluydu) | serileştirme kararı → K-B1.3, spec 1.0.1 normatif; formülün kendisi K-B1.2 ile değişti |
| E3 — §11 bilgilendirici saklama notu | sağlar | sağlar | açıklık |
| E4 — durum damgası (Draft → Validated) | sağlar | sağlar | açıklık (karar kuralına bağlı damga) |
