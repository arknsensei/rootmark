# P2-A — İnsana bırakılanlar (durma koşulu değil; tur bunlar açıkken devam eder)

1. **Linux CI işini zorunlu yapmak (K-GH.6, GOAL A-6).** `ci.yml`'in Linux işi artık bilgi işi değildir (`continue-on-error` kaldırıldı, adı `test + lint + conformance (ubuntu-latest)`). Onu `main`'in zorunlu kontrollerine eklemek ruleset ayarıdır (Settings → Rules → Rulesets → `25320154` → Require status checks → `test + lint + conformance (ubuntu-latest)`; ya da `gh api -X PUT repos/arknsensei/rootmark/rulesets/25320154 …`). Ajan ruleset'e dokunmaz.
2. **Intel macOS (K-P2A.4-T).** Karar talebi açık: hedef olmaz (öneri) / ONNX Runtime'ı kaynaktan derleyen bilgi işi / `load-dynamic`.
3. **Kod imzalama ve notarization (RELEASE.md "İmzalama").** Bu turda yapılmadı (GOAL); sertifikalar insanındır.
4. **Yayım etiketi ve GitHub release.** Bu tur etiket atmadı; `release.yml` yalnız dalda tetiklendi, artefaktları ve köken tasdikleri kayıtlı (`reports/rounds/P2-A/release_dispatch.json`). Bir yayım, insan adımlarını (RELEASE.md) izler.
