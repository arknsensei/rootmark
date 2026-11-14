# D-1 — Dönüşerek gidenler (research/transfer; D-0/R3'ün tablosundan)

Biçim: `docs/rounds/D-0/progress/DESIGN.md` §3; araç `research/transfer/` (`docs/rounds/D-1/progress/DESIGN.md` §1). Bir dosya üç listeden onu eşleyen **en uzun** desenin sınıfını alır. Üçüncü sütun ürün deposundaki yoldur (`=`: aynı yol; dizin → dizin: yolun geri kalanı korunur). İkinci sütunun ilk harfi dönüşümün türüdür ve araç onu okur:
- **M (mekanik):** `research/transfer/rewrite.py`'de yol başına kurallar; her kural beklenen sayıda eşleşmezse araç durur.
- **K (konum):** baytlar ve kip aynen, yol değişir.
- **Y (kaplama):** ürün dosyası `product/overlay/<ürün yolu>`'dan bayt eşit gelir; `product/overlay/SOURCES.json` kaynağın (bu satırın deseni) blob kimliğini taşır, eşleşmezse araç durur.

Yalnız ürüne giren kaplamalar (laboratuvar kaynağı yok: `RELEASING.md`, `Dockerfile`, iş akışları, site…) bu tabloda değildir; `SOURCES.json`'da `"source": null` ile listelenir.

## Dönüşümler

| desen | dönüşüm | ürün yolu |
|---|---|---|
| `README.md` | Y — ürün README'si (README_SKELETON'dan) | `=` |
| `CLAUDE.md` | Y — İngilizce ajan kuralları (süreç atıfsız) | `=` |
| `index/CLAUDE.md` | Y — İngilizce paket notu | `=` |
| `server/CLAUDE.md` | Y — aynı | `=` |
| `bindings/python/CLAUDE.md` | Y — aynı | `=` |
| `clients/CLAUDE.md` | Y — aynı | `=` |
| `clients/obsidian/CLAUDE.md` | Y — aynı | `=` |
| `CONTRIBUTING.md` | Y — ürün deposuna katkı akışı (D0-K16) | `=` |
| `SECURITY.md` | Y — İngilizce güvenlik politikası (`security@rootmark.ai`) | `=` |
| `NOTICE` | Y — laboratuvarınki + dendron-site MIT bildirimi (K18) + imajdaki model | `=` |
| `Makefile` | Y — yalnız ürün hedefleri | `=` |
| `docs/GUIDE.md` | Y — ürün kılavuzu (depo URL'si, PQ3-3, PQ3-5, PQ4-9, Docker, Homebrew) | `=` |
| `docs/CLIENT_CONTRACT.md` | Y — İngilizce çeviri (numaralı başlıklar, tablo satırları ve şema bloğu aynen) | `=` |
| `docs/THREAT_MODEL.md` | Y — İngilizce çeviri | `=` |
| `.github/workflows/ci.yml` | Y — ürün CI'ı (`deps`, `test`, `test-linux`) | `=` |
| `.github/workflows/release.yml` | Y — ürün yayımı (tek `SHA256SUMS`, wheel ve eklenti CI'da, tasdik, imaj, tap) | `=` |
| `.github/workflows/pages.yml` | Y — belge sitesi (mdBook) | `=` |
| `.github/PULL_REQUEST_TEMPLATE.md` | Y — ürün PR şablonu | `=` |
| `packaging/homebrew/rootmark.rb.in` | Y — macOS arm64 ve Linux x86_64 blokları, ürün deposu | `=` |
| `packaging/` | M — depo adı `arknsensei/rootmark` → `rootmark-ai/rootmark` (`install.sh`, `install.ps1`) | `=` |
| `server/Cargo.toml` | M — `repository` alanı → ürün deposu | `=` |
| `.github/dependabot.yml` | M — `uv` (`/research`) ekosistemi çıkar | `=` |
| `scripts/scene.sh` | M — varsayılan çıktı → `out/scene` | `=` |
| `scripts/scene_agent.sh` | M — varsayılan çıktı → `out/scene_agent` | `=` |
| `scripts/plugin_e2e.sh` | M — varsayılan çıktı → `out/plugin_e2e` | `=` |
| `index/tests/lexical.rs` | M — `research/data/` → `testdata/` | `=` |
| `index/tests/embed.rs` | M — aynı | `=` |
| `index/tests/p2_l_tokenize.rs` | M — aynı | `=` |
| `server/tests/p2_b_b0.rs` | M — aynı | `=` |
| `server/tests/p2_b_b1.rs` | M — aynı | `=` |
| `server/tests/p2_p_channel.rs` | M — aynı | `=` |
| `server/tests/p2_p_p0.rs` | M — aynı | `=` |
| `server/tests/promotion_rules.rs` | M — aynı | `=` |
| `server/tests/p2v1_migrate.rs` | M — `index_ort` kopyasında `dot-rootmark` → `.rootmark` | `=` |
| `server/tests/fixtures/index_ort/vault/.rootmark/` | K — ürün ağacında nokta dizini yok (D0-K14) | `server/tests/fixtures/index_ort/vault/dot-rootmark/` |
| `research/data/tok_ref.json` | K — sözcüksel dizgeleyici başvurusu | `testdata/tok_ref.json` |
| `research/data/embed_ref.json` | K — gömme başvuru vektörleri | `testdata/embed_ref.json` |
| `research/data/p2_0_old_vaults/` | K — eski sürümlerin sentetik kasaları | `testdata/p2_0_old_vaults/` |
| `research/data/p2_0_1_old_vaults/` | K — aynı | `testdata/p2_0_1_old_vaults/` |
| `research/data/p2_0_2_old_vaults/` | K — aynı | `testdata/p2_0_2_old_vaults/` |

## İstisnalar

Giden dosyalarda bulunan `/Users/<x>/` ya da `/home/<x>/` dizgileri; her biri kurgu ya da genel bir örnektir (kişisel yol değil). Gerekçesiz dizgi `classify.py`'yi düşürür.

| dosya | dizgi | gerekçe |
|---|---|---|
| `docs/GUIDE.md` | `ada` | kılavuzun kurgu kullanıcısı (`/Users/ada/notes`: örnek vault, örnek çıktı) |
| `docs/GUIDE.md` | `rmhuman` | kılavuzun §9 başsız kurulumunda önerilen ayrı OS kullanıcısının adı (`/home/rmhuman/…`), örnek |

## E-posta istisnaları

Ürün ağacında rol adresleri (`@rootmark.ai`), `example.com/org/net` ve `git@github.com` dışındaki her e-posta adresi burada dosya ve adresle, gerekçeli olmalıdır; gerekçesiz adres aktarımı durdurur (`research/transfer/leaks.py`).

| dosya | adres | gerekçe |
|---|---|---|
| `testdata/tok_ref.json` | `support@dendron.so` | dendron-site belgesinin (MIT, K18; `testdata/THIRD_PARTY.md`) metnindeki kamusal destek adresi; başvuru verisi aynen (D0-K18 (a)) |
