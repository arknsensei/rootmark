# P1-E — Ek (`docs/rounds/P1-E/ADDENDUM.md`; GOAL ve TASKS'in parçasıdır)

P1-D.1 doğrulamasının iki koşulu ve üç önerisi E0 içinde kapatılır. **Başlangıç koşulu değişir:** P1-E, E0'ın tamamlanması ve E0 sonunda `verify.sh pair`'in temiz klonlardan çıkış 0 vermesiyle başlamış sayılır; E1 bundan önce başlamaz.

## Kararlar (E0'da DECISIONS.md'ye)
- **K-E0.1 K-D1.4-T kapanır: 1.0.3 doğrudur.** CRITERION majör maddesindeki "veri" şöyle tanımlanır: *"önceki sürüme uyumlu bir uygulamanın yazdığı veri yeni uygulama tarafından okunamıyorsa, ya da yeni uygulamanın yazdığı veri önceki sürümün okuma kurallarıyla okunamıyorsa"*. Spec §4.2 `meta`'yı açık JSON nesnesi tanımladığı için `meta.actor/source` eklemek her iki yönde okunabilirliği korur; K-D1.2'nin "yazarken zorunlu" kuralı uygulama yükümlülüğüdür, okuyucu yükümlülüğü değil. `SPEC_EDITS.md`'ye not: 1.0.2 okuyucuları yeni `meta` alanlarını yok sayar, hiçbir biçim okunamaz hale gelmez. CRITERION metni bu tanımla güncellenir.
- **K-E0.2 Belirlenimli alan denetimi.** Her `compare_*` aracı yalnız sözleşmeye göre belirlenimli alanları karşılaştırır; `research/eval/DETERMINISTIC_FIELDS.md` her karşılaştırma dosyası için alan listesini ve gerekçesini tutar; `scripts/check_compare_fields.py` bu listeyle `compare_*` kodunu eşler (CI). D1-5'in `responses_a_vs_b_equal` alanı bilgi alanı olur; yazan ve iz yazan çağrıların yanıtları belirlenimli kalır ve karşılaştırılır.
- **K-E0.3 `index_stale` hata kodu.** Sözleşme §3'e `index_stale` (HTTP 503) eklenir: indeks biçimi eski ya da eksikse `search` bunu döner, `internal` değil. `mdk serve` başlangıçta biçim farkında yeniden kurar (`[index] auto_rebuild = true` varsayılan; `false` ise `index_stale`).
- **K-E0.4 İnsan belgesine ekleme kendi başlığıyla.** Ajanın insan belgesine `append`'i yalnız yeni bir başlık (`## …`) ile başlıyorsa `AppendOnly` altında kabul edilir (K-D1.1'in doğal sonucu; canonical bloğa dokunmaz). Sözleşme §4'e "ajan yönergesi" olarak yazılır; MCP `propose` açıklaması bunu söyler; başlıksız ekleme `policy_violation` + `hint: "start the appended text with a heading"` döner.

## E0'a eklenen işler
1. **Ş0:** `compare_p1d1` düzeltmesi dondurma istisnasıyla (FREEZE_EXCEPTIONS girdisi, önce/sonra: P1-D.1 resmi RUN üzerinde belirlenimli alanların karşılaştırması bit eşit, gözden geçirici onayı). `DETERMINISTIC_FIELDS.md` ve `check_compare_fields.py`. Ardından **iki temiz klondan** `verify.sh pair` → çıkış 0; sonuç `reports/rounds/P1-D.1/runs/<RUN>/pair_recheck.json` olarak P1-D.1 raporuna **ek** (FINAL değiştirilmez, `FINAL_ADDENDUM.md` yazılır).
2. **Ş1:** K-E0.1; CRITERION ve SPEC_EDITS güncellemesi.
3. **Ş3:** `index_stale` (server + index); sözleşme §3; test: eski biçimli indeks → 503/`index_stale`; `auto_rebuild` ile düzeliyor.
4. **Ş2:** K-E0.4; MCP araç açıklaması; `hint`; sözleşme §4.
5. **Ş4:** `contract_check.sh`'e gerçek HTTP çağrılarıyla §1 uçları ve §3 durum tablosu adımı; bir uç kaldırılınca kapı kırmızı (kasıtlı test).

## Çıkış kriterlerine ek
| Kod | Ölçüm | Eşik |
|---|---|---|
| E-12 | P1-D.1 `verify.sh pair` yeniden koşturma (temiz klonlar) | çıkış 0, belirlenimli alanlarda fark 0 |
| E-13 | `check_compare_fields.py` tüm `compare_*` için | eşleşmeyen alan 0 |
| E-14 | `index_stale`: eski indeks → 503; `auto_rebuild` → 200 | 2/2 |
| E-15 | Başlıksız `append` → `policy_violation` + hint; başlıklı → kabul | 2/2 |
| E-16 | `contract_check.sh` HTTP adımı; kasıtlı uç kaldırma → hata | 2/2 |

E-12..E-16 E0'ın kabul ölçütleridir; E-11 (bağımsız doğrulayıcı) bunları da kapsar. Karar kuralı değişmez; E-12 kaldı → **E1 başlamaz**, karar talebi.

## Doğrulamaya ek (VERIFY.md'ye)
- V0: P1-D.1 `pair_recheck.json`'ı kendi iki klonunla yeniden üret, çıkış 0; `DETERMINISTIC_FIELDS.md`'deki her alanın gerçekten belirlenimli olduğunu 3 koşturmada teyit et.
- V8 ek: eski biçimli indeksle `search` → `index_stale`; `auto_rebuild=false` ile 503, `true` ile 200.
- V5 ek: başlıksız `append` reddi ve `hint`; istemci bu hint'i kullanıcıya gösteriyor mu.
- V9 ek: `contract_check.sh` HTTP adımı; bir uç kaldırılınca kırmızı.

## İstem değişikliği
P1-E ana isteminin başlangıç koşulu satırını şununla değiştir: *"Başlangıç koşulu: reports/rounds/P1-D.1/VERIFICATION.md 'Şu koşullarla güvenilir'; koşullar docs/rounds/P1-E/ADDENDUM.md ile E0'da kapatılır. E0 tamamlanıp E-12..E-16 geçmeden E1'e geçme."* İkinci satıra ekle: *"docs/rounds/P1-E/ADDENDUM.md'yi de oku; E0 görevi ADDENDUM'daki işleri içerir."*
