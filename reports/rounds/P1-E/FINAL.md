# P1-E — FINAL

- RUN: `20261017T024341Z-632f29e`; commit `632f29ec77e5c077c674f092fc8b069c5bfc1292`
- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer); E-7'nin dört hedefi `scripts/release.sh` kaydı, E-12 P1-D.1 yeniden koşturma kaydı.

## GOAL çıkış kriterleri

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| E-1 | `scripts/scene.sh` temiz vault'ta 11 adım (ajan MCP, insan HTTP) | 11/11 | 11/11 | geçti | `p1e/scene/scene.json` |
| E-2 | Plugin e2e aynı 11 adım (komut katmanı gerçek `mdk serve`'e karşı; Obsidian başsız koşmadı, ASSUMPTIONS E4-1) | 11/11 | 11/11 | geçti | `p1e/plugin_e2e/plugin_e2e.json` |
| E-3 | md sha256'ları yalnız `propose` adımında değişir (sahne ve e2e, her adım) | sahne True, e2e True | doğru | geçti | `p1e/scene/scene.json, p1e/plugin_e2e/plugin_e2e.json` |
| E-4 | Plugin yalnız sözleşme uçlarını çağırır (kayıt tutan vekil ↔ sözleşme) | 30 istek, sözleşme dışı 0; GET /v1/status, POST /v1/blame, POST /v1/checkpoint, POST /v1/promote, POST /v1/search, POST /v1/trace | sözleşme dışı 0 | geçti | `p1e/plugin_e2e/plugin_e2e.json, access_log.json` |
| E-5 | Plugin'den Promote → `Promoted` actor `human:<human_id>`; gerekçesiz → 400 | aktörler [{'id': 'editor', 'kind': 'human'}, {'id': 'editor', 'kind': 'human'}]; gerekçesiz {'code': 'invalid_arguments', 'status': 400} | 2/2 | geçti | `p1e/plugin_e2e/plugin_e2e.json, p1e/scene/scene.json` |
| E-6 | Ajanın canonical değiştirme denemesi → `denied`; blame ve arama değişmez | e2e denied/policy_violation, değişmedi {'blame_unchanged': True, 'search_unchanged': True}; sahne denied | 1/1 | geçti | `p1e/plugin_e2e/plugin_e2e.json, p1e/scene/scene.json` |
| E-7 | 4 platform ikilisi derlenir, `--version`/`--spec-version` doğru, her birinde `mdk conformance` 34/34 | 2/4 (aarch64-apple-darwin: ok; x86_64-apple-darwin: not built: error: ort-sys@2.0.0-rc.13: no prebuilt binaries available for target x86_64-apple-darwin; x86_64-pc-windows-msvc: not built: no MSVC toolchain or Windows host here; built and checked by .github/workflows/release.yml (windows-latest); x86_64-unknown-linux-gnu: ok); zincirin yerel hedefi aarch64-apple-darwin 34/34, `make conformance`'a eşit True | 4/4 | **KALDI** | `p1e/release/release_local.json, docs/rounds/P1-E/progress/E5_release.json` |
| E-8 | Çift koşturma: sahne ve e2e vault `events.jsonl`/`traces.jsonl` bayt eşit; e2e çıktıları eşit | 101958 alan, fark 0; P1-E yapıtları bayt eşit True (9 dosya) | eşit | geçti | `pair_compare.json` |
| E-9 | Regresyon: P1-D.1 kalite alanları (karar gereği hariçler ölçümden önce); uyumluluk 34/34 | 100846 alan, fark 0, hariç tutulan önek 11, kullanılmayan 0; uyumluluk 34/34 | fark 0 | geçti | `regress_vs_p1d1.json, conformance_p1d1.json` |
| E-10 | `tsc --strict`, `eslint`, `vitest` temiz; bundle < 500 KB; Obsidian API dışında çalışma zamanı bağımlılığı 0 | çıkışlar [0, 0, 0], vitest 97/97, 40701 B, bağımlılık [], paketin istekleri ['@codemirror/state', '@codemirror/view', 'obsidian'] | geçer | geçti | `p1e/client.json` |
| E-11 | Bağımsız doğrulayıcı sahneyi (E-1) ve plugin e2e'yi (E-2) kendi ortamında koşturur | doğrulama bekliyor | geçer | açık (doğrulayıcı) | `reports/rounds/P1-E/VERIFICATION.md (yazılacak)` |

## ADDENDUM (E0) kriterleri

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| E-12 | P1-D.1 `verify.sh pair` yeniden koşturma (temiz klonlar) | çıkış 0, 100982 alan, fark 0 | çıkış 0, fark 0 | geçti | `reports/rounds/P1-D.1/runs/20261016T230508Z-06cd370/pair_recheck.json` |
| E-13 | `check_compare_fields` tüm `compare_*` için | eşleşmeyen 0 | 0 | geçti | `p1e/check_compare_fields.json` |
| E-14 | `index_stale`: eski indeks → 503; `auto_rebuild` → 200 | 2/2 | 2/2 | geçti | `p1e/e0/e14.json` |
| E-15 | Başlıksız `append` → `policy_violation` + hint; başlıklı → kabul | 2/2 | 2/2 | geçti | `p1e/e0/e15.json` |
| E-16 | `contract_check.sh` HTTP adımı; kasıtlı uç kaldırma → hata | HTTP adımı True, uç kaldırma reddedildi True | 2/2 | geçti | `p1d1/d1_13.json, p1e/tamper_p1e.json` |

Kasıtlı bozmalar (`p1e/tamper_p1e.json`): 10/10 reddedildi, kontroller 5/5 → geçti.

## Karar kuralı

Kalan kriterler: E-7; açık: E-11.

Yalnız E-7 kaldı (E-11 açık: sonuç bağımsız doğrulamanın ölçülen satırları teyit etmesine bağlıdır) → **yayım ertelenir, faz 2 planlaması başlar** (K-E5-T). `mdk 0.1.0` etiketi atılmaz.
