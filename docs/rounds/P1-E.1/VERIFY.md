# P1-E.1 — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P1-E.1/VERIFICATION.md. Bu doğrulama "Güvenilir" derse insan `v0.1.0` etiketini atacak; hükmü buna göre ver.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; kalite alanları fark 0; yapıtlar resmi RUN'la bayt eşit.
- **V2 `status` (E1-1/2):** `p1d-final`'den **kendin** derlediğin `mdk` ile `/2` indeks kur (fixture'a güvenme); güncel `mdk serve` ile `auto_rebuild=false`: `status` GET/POST/MCP 200 + `index.state=stale`, `search` 503 `index_stale`, salt okunurlar 200; `true` ile hepsi 200. Bozuk manifest → `damaged`. Plugin bağlantı ekranı durumu ve yolu gösteriyor mu (gerçek Obsidian ile).
- **V3 Gerçek Obsidian e2e (E1-3):** `clients/obsidian/e2e/real/` harness'ini kendin koştur; 11/11; P1-E'deki kendi CDP yönteminle farkları not et.
- **V4 Yayım (E1-4/5):** `release_check.sh` tüm maddeler; `release.yml` yalnız `workflow_dispatch`; yerelde `cargo build --release` (aarch64) → `--version`, `--spec-version`, `mdk conformance` 34/34; `dist/release.json` biçimi; Linux/Windows için "atlandı/bilgi" raporlaması.
- **V5 Regresyon ve uyumluluk (E1-6):** P1-E referansıyla fark 0; 34/34; `check_frozen.sh`, `check_deps.sh`.
- **V6 Sözleşme (E1-8/9):** `contract_check.sh` HTTP adımı `status` alanlarını denetliyor; §3 metni ile davranış birebir; bir `status` alanı kaldırılınca kapı kırmızı.
- **V7 Belgeler:** NORTH_STAR/ARCHITECTURE farkı yalnız ifade düzeltmesi; `RELEASE.md` K-E1.1 ile tutarlı; tarihsel FAIL etiketleri doğru kararlara bağlı.
- **V8 Karar kuralı:** GOAL'den türet; FINAL ile karşılaştır; `v0.1.0` etiketinin atılmamış olduğunu teyit et.
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri; 0.1.0 için "yayımlanabilir / yayımlanamaz" tek satır.
