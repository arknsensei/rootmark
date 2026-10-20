# P2-A — Spec düzenlemeleri

Ölçüt: `docs/SPEC_EDITS_CRITERION.md` (K-B1.3; sürüm K-D1.9). Kapsam GOAL P2-A 2 ("CLIENT_CONTRACT ve spec bilgilendirici not; sürüm CRITERION ile"). §1–6 değişmez; §7.1 varsayılan cümlesi ve C2-11 değişmez (K-P2-02.5); damga konmaz.

| # | Görev | Bölüm | Düzenleme | (a) hesaplanan değer | (b) önceki uyumlu uygulama | Sınıf |
|---|---|---|---|---|---|---|
| E1 | A2 | §9 (Validated) | "Informative (Phase 2 round P2-A)": indeksler türevdir, biçimleri spec değişmeden değişebilir; başvuru uygulaması manifest'te biçimi kaydeder, bildiği eski biçimi okumaz, "göç gerekli" bildirir (eksik/okunamaz indeksten ayrı bir hata), `rootmark migrate` günlükten yeniden kurar ve eskisini geri alma kaydında saklar; göç günlüğü değiştirmez (`events_hash` eşit) | sağlar (bilgilendirici; hiçbir değer hesaplanmaz) | sağlar (bilgilendirici not hiçbir uygulamayı bağlamaz) | açıklık |

**Sürüm:** yalnız açıklık satırı → sürüm değişmez (CRITERION "yalnız açıklık satırları → sürüm değişmez"); spec **2.0.2** kalır, `SPEC_VERSION` ve uyumluluk raporu aynen.
