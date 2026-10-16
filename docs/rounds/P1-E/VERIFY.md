# P1-E — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P1-E/VERIFICATION.md.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; sahne vault'ları `events.jsonl`/`traces.jsonl`/md bayt eşit; plugin e2e çıktıları (zaman damgası hariç) eşit; kalite alanları fark 0.
- **V2 Sahne (E-1, E-11):** `scripts/scene.sh`'i kendi temiz vault'unda koştur; 11 adımı tek tek doğrula; özellikle 9. adım (checkpoint'e dönüş) ve 11. adım (ajan canonical değiştirme → `denied`).
- **V3 Plugin e2e (E-2, E-11):** harness'i kendin koştur. Gerçek Obsidian ile koşturulabiliyorsa koştur; koşturulamıyorsa `ASSUMPTIONS.md`'deki sınırlamayı değerlendir ve plugin'in komut katmanının UI katmanından ne kadar ayrık olduğunu kaynaktan oku (UI'da gizli yazma yolu var mı).
- **V4 Yazmama (E-3):** sahne boyunca md sha256 izleyici kaydını incele; `propose` dışında değişen dosya var mı. Statik tarama: `clients/obsidian/src` içinde `vault.modify|vault.create|vault.delete|adapter.write|fs.write` → 0.
- **V5 Sözleşme (E-4):** HTTP erişim günlüğündeki her yol/metot CLIENT_CONTRACT'ta; sözleşme dışı 0. `gen_client_types.sh` çıktısını yeniden üret, `git status` temiz.
- **V6 Terfi (E-5):** plugin akışından Promote → `Promoted` actor `human:<human_id>`; gerekçesiz → 400 ve olay yok.
- **V7 Yayım (E-7):** en az yerel platform ikilisini `cargo build --release` ile kendin derle; `mdk --version`, `--spec-version`; `mdk conformance` 34/34; CI artefaktları varsa 4 ikilinin hash'leri CI günlüğüyle eşleşiyor mu. `dist/spec-<ver>/` içeriği `spec/` ile birebir; `LICENSE` var.
- **V8 Regresyon ve uyumluluk (E-9):** P1-D.1 referansıyla fark 0; 34/34; `check_frozen.sh`, `check_deps.sh` (istemci kuralı dahil).
- **V9 TS kalite (E-10):** `tsc --strict`, `eslint`, `vitest`; bundle boyutu; `package.json` çalışma zamanı bağımlılığı 0.
- **V10 Belgeler:** NORTH_STAR/ARCHITECTURE farkı yalnız kanıt durumu mu (`git diff` oku); `RELEASE.md` kontrol listesi betiklerle doğrulanabilir mi; `THREAT_MODEL.md` K-D1.3 varsayımını açıkça söylüyor mu.
- **V11 Karar kuralı:** GOAL'den türet (varsayılan dal dahil); FINAL ile karşılaştır; **0.1.0 etiketinin atılmamış olduğunu** teyit et (insan adımı).
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri; faz 2 için gördüğün en önemli üç risk (bilgi).
