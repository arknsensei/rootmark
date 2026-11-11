# D-0/R3 — Dönüşerek gidenler

Biçim: `docs/rounds/D-0/progress/DESIGN.md` §3. Bir dosya üç listeden onu eşleyen **en uzun** desenin sınıfını alır; bu tablodaki tam yollar, `allowlist.txt`'deki dizin öneklerinden (ör. `server/`) ve `stays.txt`'deki `research/`'ten uzun olduğu için önce gelir. Üçüncü sütun ürün deposundaki yoldur (`=`: aynı yol; dizin → dizin: yolun geri kalanı korunur).

Dönüşüm türleri (D-1'in aktarım aracı bunları uygular; TRANSFER_PLAN §2):
- **M (mekanik):** belirlenimli bir yeniden yazım — depo URL'si (`arknsensei/rootmark` → ürün deposu), laboratuvar göreli bağlantısı → laboratuvarın etiketteki mutlak URL'si, yol değişimi (`../research/data` → `../testdata`), varsayılan çıktı dizini. Girdi laboratuvar etiketindeki dosya, çıktı her koşuda aynı baytlar.
- **K (konum):** baytlar aynen, yol değişir.
- **Y (yerine yazılan):** ürün deposundaki dosya laboratuvarda tutulan bir **ürün kaplaması**ndan (`product/overlay/<yol>`, D-1'de açılır; İngilizce) bayt eşit gelir; laboratuvardaki kaynak dosya o kaplamanın kaynağıdır ve kaplama sürüm başına onunla birlikte güncellenir.

## Dönüşümler

| desen | dönüşüm | ürün yolu |
|---|---|---|
| `README.md` | Y — ürün README'si (`reports/rounds/D-0/README_SKELETON.md`'den, D-1) | `=` |
| `CLAUDE.md` | Y — İngilizce ajan kuralları: dil, bağımlılık yönü, md'ye kimlik/meta veri yok, determinizm, testleri geçmek için test değiştirilmez; tur/karar/doğrulama süreci çıkar | `=` |
| `index/CLAUDE.md` | Y — İngilizce paket notu (sorumluluk, araçlar, sabitleme kuralı); tur atıfları çıkar | `=` |
| `server/CLAUDE.md` | Y — aynı | `=` |
| `bindings/python/CLAUDE.md` | Y — aynı | `=` |
| `clients/CLAUDE.md` | Y — aynı | `=` |
| `clients/obsidian/CLAUDE.md` | Y — aynı | `=` |
| `CONTRIBUTING.md` | Y — ürün deposuna katkı akışı (PR'lar laboratuvara yönlendirilir ya da ürün deposunda kabul edilip laboratuvara taşınır: TRANSFER_PLAN §1.3) | `=` |
| `CHANGELOG.md` | M — `docs/rounds/…`, `reports/…`, `docs/DECISIONS.md` göreli bağlantıları → laboratuvar etiketinin mutlak URL'si; depo URL'si | `=` |
| `Makefile` | Y — yalnız ürün hedefleri (`build`, `test`, `lint`, `conformance`, `models`, `release`, istemciler); `verify*`, `regress`, tur hedefleri çıkar | `=` |
| `docs/GUIDE.md` | M — depo URL'si (`arknsensei/rootmark` → ürün deposu; kurulum, `gh attestation verify -R`, BRAT), laboratuvar bağlantıları → mutlak URL | `=` |
| `docs/CLIENT_CONTRACT.md` | Y — İngilizce çeviri (normatif içerik aynen; çeviri D-1) | `=` |
| `docs/THREAT_MODEL.md` | Y — İngilizce çeviri (D-1) | `=` |
| `packaging/` | M — depo URL'si (`install.sh`, `install.ps1`, `homebrew/rootmark.rb.in`) | `=` |
| `server/Cargo.toml` | M — `repository` alanı → ürün deposu (binstall `{ repo }`) | `=` |
| `.github/workflows/ci.yml` | Y — ürün CI'ı: `cargo test`/`clippy`/`fmt`, `make conformance`, istemcilerin `tsc`/`eslint`/`vitest`, `check_deps`, `check_dep_age`; kademe 1 (`verify.sh chain-fast`, `eval.*`) çıkar | `=` |
| `.github/workflows/release.yml` | M — depo adı (`gh attestation verify -R`), yorumlardaki `docs/RELEASE.md` → ürünün `RELEASING.md`'si | `=` |
| `.github/workflows/pages.yml` | M — yayımlanan spec sayfasının adresi (TRANSFER_PLAN §3.4) | `=` |
| `.github/dependabot.yml` | M — `research/` ekosistemi (`pip`) çıkar | `=` |
| `.github/PULL_REQUEST_TEMPLATE.md` | Y — ürün PR şablonu (tur/kriter alanları çıkar) | `=` |
| `scripts/check_deps.sh` | M — `research/` kuralı (3. denetim) çıkar; Rust ve istemci yönü aynen | `=` |
| `scripts/check_dep_age.sh` | M — `research/pyproject.toml` kısmı çıkar; npm kilitlerinin dolaylı bağımlılıkları eklenir (P2-Q.2 doğrulaması Öneri 1; CHANNELS §5) | `=` |
| `scripts/release.sh` | M — gerçek Obsidian e2e adımı ve `docs/RELEASE.md` atıfları (laboratuvarın yayım kapısı) çıkar | `=` |
| `scripts/scene.sh` | M — varsayılan çıktı `reports/rounds/P1-E/scene` → `out/scene` | `=` |
| `scripts/scene_agent.sh` | M — varsayılan çıktı `reports/rounds/P1-D` → `out/scene_agent` | `=` |
| `scripts/plugin_e2e.sh` | M — varsayılan çıktı `reports/rounds/P1-E/plugin_e2e` → `out/plugin_e2e` | `=` |
| `scripts/contract_check.sh` | M — sözleşmenin İngilizce sürümünün tablo başlıklarına göre | `=` |
| `index/tests/lexical.rs` | M — `../research/data/` → `../testdata/` | `=` |
| `index/tests/embed.rs` | M — aynı | `=` |
| `index/tests/p2_l_tokenize.rs` | M — aynı | `=` |
| `server/tests/p2_b_b0.rs` | M — aynı | `=` |
| `server/tests/p2_b_b1.rs` | M — aynı | `=` |
| `server/tests/p2_p_channel.rs` | M — aynı | `=` |
| `server/tests/p2_p_p0.rs` | M — aynı | `=` |
| `server/tests/promotion_rules.rs` | M — aynı | `=` |
| `SECURITY.md` | M — `docs/SECURITY.md` (Türkçe, kalır) bağlantısı → laboratuvar etiketinin mutlak URL'si; D-1'de İngilizce özet | `=` |
| `server/tests/fixtures/index_ort/vault/.rootmark/` | K — eski dizin biçiminin sentetik test kasasının deposu; ürün deposunda nokta dizini olarak durmaz (sızıntı kuralı: giden kümede `.rootmark/` yok), test onu çalışırken geçici kopyada `.rootmark/` adıyla kurar | `server/tests/fixtures/index_ort/vault/dot-rootmark/` |
| `server/tests/p2v1_migrate.rs` | M — `index_ort` fixture'ı geçici dizine kopyalanırken `dot-rootmark` → `.rootmark` (testin gözlemi aynen) | `=` |
| `research/data/tok_ref.json` | K — ürün testlerinin başvuru verisi (sözcüksel dizgeleyici) | `testdata/tok_ref.json` |
| `research/data/embed_ref.json` | K — gömme başvuru vektörleri (işlemciden bağımsızlık testi) | `testdata/embed_ref.json` |
| `research/data/p2_0_old_vaults/` | K — eski sürümlerin yazdığı kasalar (göç ve `events_hash` uyumluluk testleri; sentetik) | `testdata/p2_0_old_vaults/` |
| `research/data/p2_0_1_old_vaults/` | K — aynı | `testdata/p2_0_1_old_vaults/` |
| `research/data/p2_0_2_old_vaults/` | K — aynı | `testdata/p2_0_2_old_vaults/` |

## İstisnalar

Giden dosyalarda bulunan `/Users/<x>/` ya da `/home/<x>/` dizgileri; her biri kurgu ya da genel bir örnektir (kişisel yol değil). Gerekçesiz dizgi `classify.py`'yi düşürür.

| dosya | dizgi | gerekçe |
|---|---|---|
| `docs/GUIDE.md` | `ada` | kılavuzun kurgu kullanıcısı (`/Users/ada/notes`: örnek vault, örnek çıktı) |
| `docs/GUIDE.md` | `rmhuman` | kılavuzun §9 başsız kurulumunda önerilen ayrı OS kullanıcısının adı (`/home/rmhuman/…`), örnek |
