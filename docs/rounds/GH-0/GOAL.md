# GitHub Kurulumu ve Yeniden Adlandırma — `/goal` yönergesi

Proje adı: **rootmark** (crate/ikili/npm/modül: `rootmark`). Bu dosya `docs/rounds/GH-0/GOAL.md` olarak konur; istem en altta.

## Kararlar (DECISIONS.md'ye)
- **K-GH.1 Proje adı:** `rootmark` ("root" = köken ve tek gerçek kaynak, "mark" = markdown). Yetki çağrışımına karşı README ilk cümlesi anlamı kurar. `mdkernel` yalnız tarihsel referans olarak `docs/archive/` ve git geçmişinde kalır; kod, belge, spec, paket adlarında kalmaz. Spec başlığı "`rootmark` Specification", sürüm numarası **değişmez** (ad değişikliği anlam değişikliği değildir; CRITERION notu).
- **K-GH.2 Lisanslar:** spec Apache-2.0 (zaten); kod Apache-2.0; istemci Apache-2.0 (Obsidian community gereği MIT/Apache uygun). `LICENSE`, `NOTICE`, `spec/LICENSE`.
- **K-GH.3 Depo düzeni:** tek monorepo `github.com/<org>/rootmark`; `spec/` aynı depoda (ayrı depo faz 2'de düşünülür). Varsayılan dal `main`, korumalı.
- **K-GH.4 Tarihçe:** prototip subtree geçmişi korunur; `v0.1.0` etiketi ve tüm `p*-final` etiketleri push edilir.

## Görevler
- **G0 Yeniden adlandırma (tek commit):** `mdkernel` → `rootmark` ve `mdk` → `rootmark` her yerde: Cargo crate adları (`rootmark-core`, `rootmark-index`, `rootmark-server`), ikili adı, `bindings/python` modül adı (`rootmark_core`), Obsidian plugin `manifest.json` (`id`, `name`), `CLAUDE.md`'ler, `docs/**`, `spec/**` başlıkları ve metin, `scripts/**`, Makefile hedefleri, `.mdkernel/` dizin adı → `.rootmark/` (**göç:** `rootmark migrate --rename-store` eski dizini yeniden adlandırır; `events_hash` değişmez; K-GH.1 gereği biçim sürümü aynı), `policy.toml` örnekleri, `CLIENT_CONTRACT.md` uç adları **değişmez** (`/v1/...`), `X-Consumer` değişmez. Tarihsel raporlar (`reports/**`) ve `docs/rounds/**` **değiştirilmez** (kayıt), yalnız `docs/HANDOFF.md`, `NORTH_STAR.md`, `ROADMAP.md`, `ARCHITECTURE.md`, `PACKAGES.md`, `DEV_PROCESS.md`, `DECISIONS.md`, `README.md` güncellenir.
  **Kabul:** `rg -i mdkernel` yalnız `reports/`, `docs/rounds/`, `docs/archive/`, `CHANGELOG.md` (tarihsel satır) içinde; `make test`, `make conformance` 37/37 (ya da güncel sayı), `make verify` çift koşturma fark 0; eski `.mdkernel/` vault `rootmark migrate --rename-store` ile açılır, `events_hash` eşit.
- **G1 Depo dosyaları:** `README.md` (tek paragraf vaat, 11 adımlık sahne gif/komut dökümü, kurulum: `cargo binstall rootmark` / Homebrew taslağı, Obsidian BRAT adımı, spec bağlantısı, lisans), `CONTRIBUTING.md` (DEV_PROCESS'in dış katkıcı özeti: tur disiplini, eşik kuralı, doğrulama), `CODE_OF_CONDUCT.md`, `SECURITY.md` (zaten; açıklama ekle: raporlama kanalı), `.github/ISSUE_TEMPLATE/{bug,hypothesis,decision-request}.md` (karar talebi şablonu `-T` biçiminde), `.github/PULL_REQUEST_TEMPLATE.md` (hangi tur, hangi kabul ölçütü, dondurma istisnası var mı).
- **G2 CI/CD:** `ci.yml` (test, lint, conformance, `check_deps`, `check_frozen`, `check_compare_fields`; PR'da zorunlu), `verify.yml` (haftalık ve `workflow_dispatch`: `verify.sh pair`, çıkış kodu kırmızı), `release.yml` yalnız `workflow_dispatch` (aarch64-apple-darwin zorunlu, diğerleri bilgi), Dependabot yalnız güvenlik güncellemeleri (sürüm sabitleme ilkesi; ≥14 gün kuralı CI'da denetlenir: `scripts/check_dep_age.sh`).
- **G3 Dal koruması ve ayarlar (gh CLI):** `main` korumalı: PR zorunlu, CI yeşil zorunlu, en az 1 onay (sen), doğrusal geçmiş, zorla push yok; etiket koruması `v*`; Actions izinleri salt okunur varsayılan; secrets: yok (API anahtarı CI'da kullanılmaz; research turları yerel). Discussions açık; Wiki kapalı (belgeler depoda).
- **G4 Yayım ve spec yayını:** `v0.1.0` release notu (`CHANGELOG.md`'den), `dist/release.json` ve ikili artefakt; `spec/` için GitHub Pages (`docs/`'tan değil, `spec/`'ten; mdBook ya da düz md) — `https://<org>.github.io/rootmark/spec/`; `conformance.json` Seviye 3 rozeti README'de.
- **G5 Push ve doğrulama:** uzak ekle, tüm dallar ve etiketler push; temiz klonda `make verify` ve `rootmark conformance`; `docs/rounds/GH-0/progress/READY.md`.

## Çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| GH-1 | `rg -i mdkernel` kalıntı (izinli dizinler hariç) | 0 |
| GH-2 | Yeniden adlandırma sonrası `make verify` çift koşturma | fark 0 |
| GH-3 | Eski `.mdkernel/` vault göçü | `events_hash` eşit, replay eşit |
| GH-4 | CI üç iş akışı PR'da yeşil; `verify.yml` elle tetiklenince yeşil | geçer |
| GH-5 | Dal koruması, etiket koruması, Actions izinleri (`gh api` ile okunur) | ayarlı |
| GH-6 | Temiz klondan `cargo build --release` + `rootmark conformance` | 37/37 |
| GH-7 | Spec sayfası yayında, README rozeti doğru | 1/1 |

## İstem (`/goal @docs/rounds/GH-0/GOAL.md`)
```
Bu tur bir altyapı turudur: proje adı rootmark olarak değişir ve depo GitHub'a taşınır. Yönerge docs/rounds/GH-0/GOAL.md. Önce CLAUDE.md, NORTH_STAR.md, docs/DEV_PROCESS.md, docs/PACKAGES.md, docs/RELEASE.md oku.
G0'dan G5'e sırayla; DEV_PROCESS §2 disiplini; commit öneki GH-0/G<n>. G0 tek commit'tir ve "rename: mdkernel -> rootmark" iletisini taşır.
Kırmızı çizgiler: reports/ ve docs/rounds/ içindeki tarihsel kayıtlar değiştirilmez; spec sürümü değişmez; CLIENT_CONTRACT uç adları değişmez; events_hash'ler değişmez; eşleyici, indeks, sunucu davranışı değişmez (make verify fark 0); hiçbir secret depoya girmez; GitHub org/kullanıcı adı ve uzak adres: <buraya yaz>; gh CLI ile yapılamayan ayarı STOP.md'ye yaz ve insana bırak.
G5 bitince READY.md, dur.
```
