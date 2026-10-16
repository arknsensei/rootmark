# P1-E — Görevler

Sıra: E0 → E1 → E2 → E3 → E4 → E5 → E6 → E7 → E8. Progress `docs/rounds/P1-E/progress/`. Commit öneki `P1-E/E<n>:`.

## E0 — Tur açılışı
**Yapar:** `git tag p1d1-final`; `ROUND_START`; `clients/obsidian/CLAUDE.md`; TypeScript iskeleti (`esbuild`, `tsc --strict`, `eslint`, `vitest`; `obsidian` tipi dev-dependency, çalışma zamanı bağımlılığı yok); `scripts/check_deps.sh` istemci kuralı (yalnız `docs/CLIENT_CONTRACT.md`'deki yollar; kaynakta `core/`, `index/`, `server/` içe aktarımı yasak — zaten var, doğrula).
**Kabul:** `npm run build` boş plugin; `check_deps.sh` temiz.

## E1 — Bağlantı ve API istemcisi
**Yapar:** `src/api.ts`: CLIENT_CONTRACT'tan **üretilmiş** tipler (`scripts/gen_client_types.sh`, sözleşmedeki JSON Schema'lardan); token/port okuma; hata kodu eşlemesi; sunucu yoksa pasif durum.
**Kabul:** `vitest` sözleşme testleri: her araç için sahte sunucuya karşı istek biçimi sözleşmeyle birebir; sözleşme dışı uç çağrısı derleme hatası.

## E2 — Blame paneli ve proposal görünümü
**Yapar:** kenar işaretleri (CodeMirror 6 gutter), `history` açılır penceresi; proposal listesi, Promote/Demote (gerekçe zorunlu), sonuç bildirimi.
**Kabul:** `vitest` bileşen testleri; E-5 sahte sunucuyla; plugin dosya yazma API'lerini (`vault.modify`, `vault.create`, `adapter.write`) hiç çağırmıyor (statik tarama testi).

## E3 — Arama, checkpoint, trace panelleri
**Yapar:** arama paneli (mode, k, include_proposal), proposal hit'ler ayrı stil; checkpoint listesi/adlandırma/`blame --at` salt okunur görünümü; trace listesi ve atlama.
**Kabul:** bileşen testleri; `include_proposal` açıkken canonical sıra değişmiyor (sahte sunucu sözleşme davranışı).

## E4 — Sahne ve plugin e2e
**Yapar:** `scripts/scene.sh` (11 adım; ajan adımları MCP, insan adımları HTTP); `clients/obsidian/e2e/` harness: Obsidian başsız çalıştırılamıyorsa plugin'in komut katmanı (`src/commands.ts`) gerçek `mdk serve`'e karşı Node'da koşturulur ve bu sınırlama `ASSUMPTIONS.md`'ye yazılır; md sha256 izleyicisi (E-3); HTTP erişim günlüğü ↔ sözleşme karşılaştırması (E-4).
**Kabul:** E-1, E-2, E-3, E-4, E-6.

## E5 — Yayım paketi
**Yapar:** CI çapraz derleme (4 hedef), `mdk --version/--spec-version`, `mdk conformance` alt komutu (fixture'lar ikiliye gömülü); wheel; plugin paketi; `spec/` yayım dizini (`dist/spec-<ver>/`); `CHANGELOG.md`; `docs/RELEASE.md`; `docs/THREAT_MODEL.md`.
**Kabul:** E-7; `RELEASE.md` kontrol listesinin her maddesi betikle doğrulanabilir.

## E6 — NORTH_STAR ve ARCHITECTURE güncellemesi
**Yapar:** "Kanıtlananlar" bölümü faz 1 sonuçlarıyla; ARCHITECTURE §9 tablosu; §4.1 primitifler "Kanıtlı"; faz 2 girdileri için `docs/rounds/P2-A/` boş iskelet (yalnız dizin ve `TODO.md`). Değişiklikler yalnız kanıt durumu; vaat ve ilkeler değişmez.
**Kabul:** gözden geçirici: "yalnız kanıt durumu değişti".

## E7 — Doğrulama zinciri ve regresyon
**Yapar:** `verify.sh`: conformance → regress (P1-D.1) → scene → plugin e2e → release build (en az yerel hedef) → pair compare.
**Kabul:** E-8, E-9, E-10.

## E8 — Rapor ve hazır
**Yapar:** `report_p1e.py`; `FINAL.md`; `READY.md`. **0.1.0 etiketi atılmaz**; doğrulama "Güvenilir" dedikten sonra insan atar (`docs/RELEASE.md` adımı).
**Kabul:** izlenebilirlik; karar kuralı.
