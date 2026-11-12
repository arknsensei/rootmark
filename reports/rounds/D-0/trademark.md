# D-0 — Marka taraması

> **Bu tarama hukuki görüş değildir.** Herkese açık veri tabanlarında ad eşleşmelerinin bir dökümüdür; tescil edilebilirlik, karıştırılma olasılığı ya da kullanım hakkı hakkında bir yargı içermez. Tescil kararı öncesinde bir marka vekili ya da avukat görüşü alınmalıdır.

**Sorgular:** `rootmark`, `root mark`, `rootmarks`, `rhizomark` (R1 yedeği). **Sınıflar:** bütün sınıflar listelenir; Nice 9 (yazılım) ve 42 (yazılım hizmetleri, SaaS) ilgili sınıflardır.

## Ajanın erişim denemesi (yalnız `GET`)
`research/d0/trademark_probe.py` → `trademark_probe.json` (yanıt kodu, içerik türü, boyut, gövde sha256; gövde yazılmaz):

| kaynak | URL | UTC | kod | sonuç |
|---|---|---|---|---|
| TÜRKPATENT | https://www.turkpatent.gov.tr/arastirma-yap?form=trademark | 2026-11-11T20:27:38Z | 200 | text/html; charset=utf-8, 133696 B; sorgu sonucu yanıtta yok (JavaScript arayüzü) → insan adımı |
| EUIPO TMview | https://www.tmdn.org/tmview/#/tmview/results?page=1&pageSize=30&criteria=C&basicSearch=rootmark | 2026-11-11T20:27:39Z | 200 | text/html, 531 B; sorgu sonucu yanıtta yok (JavaScript arayüzü) → insan adımı |
| EUIPO TMview (API, GET) | https://www.tmdn.org/tmview/api/search/results?basicSearch=rootmark&page=1&pageSize=30 | 2026-11-11T20:27:39Z | 200 | text/html, 244 B; sorgu sonucu yanıtta yok (JavaScript arayüzü) → insan adımı |
| USPTO Trademark Search | https://tmsearch.uspto.gov/search/search-results?query=rootmark&section=default | 2026-11-11T20:27:39Z | 200 | text/html, 125603 B; sorgu sonucu yanıtta yok (JavaScript arayüzü) → insan adımı |
| WIPO Global Brand Database | https://branddb.wipo.int/en/quicksearch/brand?by=brandName&v=rootmark | 2026-11-11T20:27:39Z | 200 | None, 1717 B; sorgu sonucu yanıtta yok (JavaScript arayüzü) → insan adımı |

## Kaynak başına sonuç

| kaynak | sorgulayan | arayüz | UTC | `rootmark` | `root mark` | `rootmarks` | `rhizomark` | Nice 9/42'de eşleşme |
|---|---|---|---|---|---|---|---|---|
| TÜRKPATENT | insan (HUMAN_STEP H2) | Marka Araştırma | — | ertelendi | ertelendi | ertelendi | ertelendi | ertelendi |
| EUIPO TMview | insan (HUMAN_STEP H2) | tmdn.org/tmview | — | ertelendi | ertelendi | ertelendi | ertelendi | ertelendi |
| USPTO | insan (HUMAN_STEP H2) | tmsearch.uspto.gov | — | ertelendi | ertelendi | ertelendi | ertelendi | ertelendi |
| WIPO Global Brand Database | insan (HUMAN_STEP H2) | branddb.wipo.int | — | ertelendi | ertelendi | ertelendi | ertelendi | ertelendi |

**Durum (2026-11-12): H2 ertelendi.** Ürün sahibi marka taramasını en sona (geliştirmelerden de sonra) bıraktı; sorguları kendisi yapacak (`docs/rounds/D-0/progress/R2_trademark_human.md`). Bu turda dört kaynağın kanıtlı satırı **yok** → D0-3 karar kuralına göre **kaldı (ertelendi)**. İsim kararları (REPORT §7: D0-K1, K2, K3, K15) tarama tamamlanana kadar açık.

## Doğrulanmamış risk (satır değildir)
Ürün sahibinin gözlemi: uluslararası kayıtlarda Nice **35** ve **42**'de **"RootMark Inc"** adlı bir şirkete ait bir kayıt görünüyor. Kaynak, sorgu zamanı, durum (başvuru / tescil / geçersiz), ülke ve numara yazılı değil; **doğrulanmadı**. Doğrulanırsa ve kayıt etkinse Nice 42 (yazılım hizmetleri) bu ürünün sınıfıyla örtüşür: K1'in (a) seçeneği için ciddi bir engel olabilir ve (b) `rhizomark` ya da (c) yeniden değerlendirilir. Tarama yapılırken bu kayıt ilk bakılacak satırdır (kaynak, durum, ülke, numara, sınıflar, sahip). Bu not hukuki görüş değildir.
