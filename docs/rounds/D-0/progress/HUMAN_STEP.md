# D-0 — İnsan adımları

> **Durum (R7, 2026-11-11): tur H2'de bekliyor.** R0–R6 tamamlandı, R7'nin kapıları temiz ve gözden geçirici KABUL verdi. Kalan tek şey D0-3: aşağıdaki **H2**'nin dört marka sorgusu. Sonuçlar `docs/rounds/D-0/progress/R2_trademark_human.md`'ye yazılınca ajan `trademark.md`'yi doldurur, `FINAL.md`'yi kapatır, READY'yi yazar ve PR'ı açar. H1 (npm kullanıcı sayfaları) isteğe bağlıdır; beklenmez.

Ajan bu adımları yapmaz (GOAL İlkeler: tarayıcı isteyen sorgular, hesaplar). Sonuçlar belirtilen dosyaya yazılır; ajan okuyup raporlara işler. Hiçbir adımda parola, token ya da kişisel e-posta adresi yazılmaz.

## H1 — npm kullanıcı adları (R1; `npm_user` sütununun 4 erişilemeyen hücresi)
`www.npmjs.com` ajanın isteklerine 403 verdi. Bir tarayıcıda şu sayfaları açın ve her biri için "sayfa var (kullanıcı/kurum)" ya da "404" yazın:
- https://www.npmjs.com/~rootmark-core
- https://www.npmjs.com/~rootmark-index
- https://www.npmjs.com/~rootmark-server
- https://www.npmjs.com/~rhizomark

(Son tarama koşusunun 403 aldığı dört ad; ilk koşuda 403 alan `rootmark-ai`, `getrootmark`, `rootmarkhq` ikinci koşuda 404 = boş verdi.)

Yazılacak yer: `docs/rounds/D-0/progress/R1_npm_human.md`, satır başına `| ad | URL | tarih (UTC) | sonuç |`. Karar etkisi düşük (R1.md): bu adım beklenirken tur sürer.

## H2 — Marka taraması (R2; D0-3'ün dört kaynağı)
Ajanın `GET` denemesi (`research/d0/trademark_probe.py` → `reports/rounds/D-0/trademark_probe.json`): dört kaynak da yalnız JavaScript arayüzü döndürüyor, sorgu sonucu yanıtta yok. Sorguyu bir tarayıcıda siz yapın. **Sorgular (her kaynakta dördü):** `rootmark`, `root mark`, `rootmarks`, `rhizomark`. Benzerlik/bulanık arama seçeneği varsa açık; bütün sınıflar; sonuçta Nice 9 ve 42'yi işaretleyin.

1. **TÜRKPATENT** — https://www.turkpatent.gov.tr/arastirma-yap?form=trademark → "Marka Araştırma"; "Marka Adı" alanına sorgu; (gerekirse e-Devlet girişi; kimlik bilgisi dosyaya yazılmaz).
2. **EUIPO TMview** — https://www.tmdn.org/tmview/ → arama kutusuna sorgu; ofis filtresi yok (bütün ofisler); sonuç listesinde "Nice class" sütunu.
3. **USPTO** — https://tmsearch.uspto.gov/ → "Wordmark" araması; canlı (LIVE) ve ölü (DEAD) kayıtlar ikisi de yazılır.
4. **WIPO Global Brand Database** — https://branddb.wipo.int/ → "Brand name" araması (Fuzzy/benzer açık).

**Yazılacak yer:** `docs/rounds/D-0/progress/R2_trademark_human.md`, kaynak ve sorgu başına bir bölüm:

```
### <kaynak> · <sorgu> · <tarih saat UTC>
Sonuç sayısı: <n>
| marka | Nice sınıfları | durum (başvuru/tescil/geçersiz) | sahip ülkesi | başvuru/tescil no |
|---|---|---|---|---|
```

Sonuç yoksa "Sonuç sayısı: 0". Sahibin kişi adı yazılmaz (şirket adı olabilir); ekran görüntüsü isteğe bağlıdır ve depoya girmez. Bu adım bitene kadar D0-3 açıktır; tur öteki görevlerle sürer.
