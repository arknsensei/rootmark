# P1-E — İstemci, Ürün Sahnesi ve 0.1.0

Başlangıç: P1-D.1 doğrulandı ("Güvenilir"; koşullu ise koşullar yalnız istemciyi ilgilendirmeyen gözlemler), `p1d1-final` etiketi.

## Amaç
Faz 1'i kapatmak: minimal Obsidian istemcisi (`clients/obsidian`), ürün sahnesinin **insan tarafı**, sahnenin uçtan uca betiği ve bağımsız doğrulayıcı tarafından koşturulması, `mdk 0.1.0` yayım paketi ve spec'in yayımlanabilir hali. İstemci çekirdeğe yalnız `docs/CLIENT_CONTRACT.md` üzerinden (HTTP, token) bağlanır; başka hiçbir iç yüzeye dokunmaz.

## Kapsam
### `clients/obsidian` (TypeScript, Obsidian plugin API)
- **Bağlantı:** vault kökündeki `.mdkernel/server.token` ve `policy.toml [server] http_port` okunur; `mdk serve` yoksa plugin pasif ve durumu bildirir (başlatmaz; faz 2).
- **Blame paneli:** aktif notta blok başına kenar işareti: katman (canonical/proposal), yazar türü, kaynak, son olay; tıklayınca `history`.
- **Proposal görünümü:** vault genelinde `proposal` bloklar listesi (doküman, bölüm, kaynak, zaman); seçili blok(lar) için **Promote** (HTTP `promote`, K-D1.3 insan kanalı) ve **Demote**; gerekçe zorunlu.
- **Arama paneli:** `search` (mode, k, include_proposal anahtarı); proposal hit'ler görsel olarak ayrı; sonuçtan nota atlama.
- **Checkpoint:** liste, adlandırma, `blame --at` ile geçmiş görünümü (salt okunur).
- **Trace görüntüleyici:** son izler; bir izin döndürdüğü bloklara atlama.
- Plugin md dosyalarına **asla yazmaz**; tek yazma yolu HTTP araçlarıdır. Ayar: yalnız `human_id` görüntüsü (düzenleme `policy.toml`'dan).

### Ürün sahnesi (insan tarafı + ajan tarafı birlikte)
`scripts/scene.sh` (P1-D `scene_agent.sh` + insan adımları HTTP ile): status → ajan `propose` → kullanıcı blame'de `proposal` görür → varsayılan arama bulmaz → kullanıcı Promote (HTTP, gerekçeli) → arama bulur → trace cevabın dayandığı blokları gösterir → checkpoint'e dönüp eski halin okunması → ajan canonical'ı değiştirmeye çalışır → `denied`. Plugin'in UI akışı aynı adımları `clients/obsidian/e2e/` içinde başsız Obsidian ile (ya da Obsidian'ın test harness'i yoksa plugin'in komut katmanını doğrudan çağıran bir harness ile) koşturur.

### 0.1.0 yayım paketi
- `mdk` ikilileri: Linux x86_64, macOS arm64/x86_64, Windows x86_64 (CI çapraz derleme); `cargo-binstall` meta; Homebrew formül taslağı.
- Python wheel (`mdkernel_core`); Obsidian plugin paketi (`main.js`, `manifest.json`, `styles.css`; community store başvurusu **yapılmaz**, "yayıma hazır" durumu).
- `spec/` yayım: `SPEC.md` + `CONFORMANCE.md` + fixture'lar, sürüm CRITERION'un güncel değeri, `conformance.json` Seviye 3 raporu, `LICENSE` (Apache-2.0).
- `CHANGELOG.md`, `docs/RELEASE.md` (yayım kontrol listesi), `docs/THREAT_MODEL.md` (SECURITY.md'den).

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| E-1 | `scripts/scene.sh` temiz vault'ta 11 adım | 11/11 |
| E-2 | Plugin e2e harness aynı 11 adımı UI/komut katmanından | 11/11 |
| E-3 | Plugin, md dosyasına hiçbir yazma yapmaz: sahne boyunca vault md dosyalarının sha256'ları yalnız `propose` adımlarında değişir (izleyici kaydı) | doğru |
| E-4 | Plugin yalnız CLIENT_CONTRACT uçlarını çağırır (HTTP erişim günlüğü ↔ sözleşme) | sözleşme dışı 0 |
| E-5 | Promote plugin'den → `Promoted` actor `human:<human_id>`; gerekçesiz → 400 | 2/2 |
| E-6 | Ajan canonical değiştirme denemesi sahnede → `denied`; blame ve arama değişmez | 1/1 |
| E-7 | Yayım paketi: 4 platform ikilisi derlenir, `mdk --version` ve `mdk --spec-version` doğru; her ikilide `make conformance` eşdeğeri (`mdk conformance`) 34/34 | 4/4 |
| E-8 | Çift koşturma: sahne vault'u `events.jsonl`/`traces.jsonl` bayt eşit; plugin e2e çıktıları (zaman damgası hariç) eşit | eşit |
| E-9 | Regresyon: P1-D.1 kalite alanları; uyumluluk 34/34 | fark 0 |
| E-10 | `tsc --strict`, `eslint`, `vitest` temiz; plugin bundle boyutu < 500 KB; Obsidian API dışında çalışma zamanı bağımlılığı 0 | geçer |
| E-11 | Bağımsız doğrulayıcı sahneyi (E-1) ve plugin e2e'yi (E-2) kendi ortamında koşturur | geçer |

## Karar kuralı
E-1..E-11 geçti → **Faz 1 tamamlandı**: `mdk 0.1.0` etiketi, spec §7–§11 "Validated" teyit, NORTH_STAR "Kanıtlananlar" güncellenir, faz 2 planlaması başlar. E-3/E-4/E-5/E-6 kaldı → istemci yönetişimi; yayım yok. E-7 kaldı → yayım ertelenir, faz 2 planlaması başlar. E-2 kaldı, E-1 geçti → sahne çekirdekte doğru, UI eksik; yayım "ikili + spec" olarak yapılır, plugin "hazır değil" etiketiyle. **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
`mdk serve`'i plugin'den başlatma; çok vault; uzak sunucu; community store başvurusu; VS Code; örtük onay; plugin'den md yazma; yeni spec bölümü.
