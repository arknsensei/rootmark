# P2-A — Bağımsız Doğrulama (dar kapsam, geliştirme turu)

Bağımsız doğrulayıcısın; ana dizine erişme (yalnız bu yönergeyi ve DEV_PROCESS §5'i oku). Temiz klon, kendi araçların. Kod değiştirme. Tek çıktı `reports/rounds/P2-A/VERIFICATION.md`. Hedef süre 2–3 saat (K-P2.3): yalnız kararı belirleyen kriterler.

- **V1 Çift koşturma:** `verify.sh pair` tek yüksüz çift, çıkış 0, fark 0.
- **V2 Göç (A-1, A-2):** `v0.1.0`'ı kendin derle (`git archive`), kendi 3 vault'unu üret (sahne, e2e, karışık); `migrate --dry-run` hiçbir baytı değiştirmiyor; göç sonrası `events_hash`, replay, `blame` eşit, yeni ikiliyle `conformance`; `--rollback` sonrası orijinalle bayt eşit; kasıtlı: geri alma kaydındaki bir dosyayı boz → geri alma reddi; `.rootmark/migrations/` dışına yazılmış bir şey var mı.
- **V3 Durum ve sözleşme (A-3):** eski indeksli vault'ta `status.migrations` ve `migration_required`; `contract_check.sh`; `index_stale`'in kalan tek anlamı.
- **V4 Kurulum (A-4):** `release.yml`'in bu turdaki dispatch artefaktlarını indir; Linux artefaktını Docker'da (`ubuntu-24.04`) koşturup `rootmark --version`, `--spec-version`, `conformance`; Windows artefaktı için CI günlüğü ve `SHA256SUMS`; `gh attestation verify <artefakt> --owner arknsensei`.
- **V5 A0 (A-7):** yeni vault → `policy.toml` satırı ve `status.attestation.unattested = "canonical"`; ajan doğrudan yazımı `canonical` (eski davranış, `status` gösteriyor); `unverified` seçilince 5 doğrudan yazım `unverified`; (A) senaryosu: kendine bağlantılı büyük notu gerçek Obsidian'da (ayrı örnek, CDP) yeniden adlandır, `--attest-log` nedenleri; bilgi ölçümünün raporda "karar kriteri değil" diye işaretli olduğu.
- **V6 Linux (A-6):** `main`'in Linux CI işi yeşil; `propose` dizin varyantı testini Docker'da kendin koştur.
- **V7 Süreç:** `check_frozen.sh`, `check_deps.sh`, `check_dep_age.sh`; karar kuralını GOAL'den türet, FINAL ile karşılaştır; NORTH_STAR değişmemiş; spec damgası Draft; `ROUND_START` bir `main` commit'i (K-P2.2).
- **V8 Vaat sorusu (zorunlu):** varsayılan `canonical` ile NORTH_STAR'ın vaadi dosya erişimli ajanla hangi koşulda tutuyor (opt-in `unverified` + tasdikli istemci); daraltma cümlesi önerisi (K-P2-02.5) ölçülen sınırla uyumlu mu; tek paragraf.
- **Sonuç:** ihlal/fark/şüphe; kriter başına geçti/kaldı; hüküm; ≤ 3 öneri.
