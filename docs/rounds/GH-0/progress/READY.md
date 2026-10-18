# GH-0 — doğrulama için hazır

- Ölçülen commit: `3447242` (`GH-0/G5-stat:`; son kod). Bu dosyanın commit'i yalnız kayıt ekler.
- Depo: `https://github.com/arknsensei/rootmark` (private); `main` korumalı (rulesets), `v0.1.0` → `85be0a7` (GitHub release "rootmark 0.1.0").
- RUN'lar:
  - macOS, temiz klon `3447242`: A `20261018T075227Z-3447242`, B `20261018T082054Z-3447242` — `verify.sh pair` çıkış 0, 103 195 alan fark 0, bayt eşit;
  - macOS, temiz klon `b152968`: A `20261017T225213Z-b152968`, B `20261017T232043Z-b152968` — aynı sonuç;
  - Linux, `verify.yml` run `39098486795` (`3447242`): 505 535 alan fark 0, bayt eşit;
  - yeniden adlandırmanın kendisi (G0, `d061c28`): A `20261017T192224Z-d061c28`, B `20261017T195112Z-d061c28` — 103 189 alan fark 0.

## Çıkış kriterleri

| Kod | Ölçüm | Değer | Eşik | Sonuç |
|---|---|---|---|---|
| GH-1 | `rg -i mdkernel` kalıntı (izinli dizinler hariç) | harfiyen: 35 dosya (GOAL'ün dört izinli yeri dışında). ASSUMPTIONS G0-1 kümesi ve G0-2 veri kimliği `mdkernel-index/<n>` hariç: **0** (küme içindeki 18 dosya tek tek gerekçeli, G0.md) | 0 | **geçti — K-GH.5 (insan kararı, seçenek (a): veri kimliği ve gerekçeli küme hariç)** |
| GH-2 | Yeniden adlandırma sonrası `make verify` çift koşturma | `d061c28`, `b152968`, `3447242` (macOS) ve `3447242` (Linux): fark 0 | fark 0 | geçti |
| GH-3 | Eski `.mdkernel/` vault göçü | `rootmark migrate --rename-store`: `events_hash` ve replay/blame `p1d-final` ikilisininkiyle eşit; depo ağacı dosya dosya bayt eşit; `p1e-final` `/3` indeksi yeniden kurulmadan okunur (`server/tests/gh0_migrate.rs` 3/3) | `events_hash` eşit, replay eşit | geçti |
| GH-4 | CI üç iş akışı PR'da yeşil; `verify.yml` elle tetiklenince yeşil | `ci.yml` PR'larda yeşil (zorunlu: "dependency direction and age", "test + lint + conformance (macos-14)"; Linux bilgi işi bilinen bir testten kırmızı, K-GH.6-T); `release.yml` zorunlu `aarch64-apple-darwin` yeşil (Windows bilgi işi model indirmede düştü); `verify.yml` dispatch 4 yeşil (1–3 kaldı: `xargs`/`tar` ve `stat` taşınabilirliği, F4/F5 ile düzeltildi) | geçer | geçti (notlarla) |
| GH-5 | Dal koruması, etiket koruması, Actions izinleri (`gh api` ile okunur) | rulesets `25320154` (PR + CI + doğrusal geçmiş, muafiyet yok), `25320155` (1 onay, yönetici muaf), `25320083` (`v*`); Actions varsayılan izni salt okuma; secret 0; Discussions açık, Wiki kapalı (G3.md geri okuma) | ayarlı | geçti |
| GH-6 | Temiz klondan `cargo build --release` + `rootmark conformance` | `rootmark 0.1.0`, spec 1.0.3, level 3, **34/34** (güncel sayı; GOAL "37/37 ya da güncel sayı") | 37/37 | geçti (güncel 34/34) |
| GH-7 | Spec sayfası yayında, README rozeti doğru | `https://arknsensei.github.io/rootmark/spec/` (+ `CONFORMANCE.html`, `conformance.json`); rozet `badge.json`'dan, ölçülen "Level 3 · 34/34" | 1/1 | geçti |

## Kırmızı çizgiler
- `reports/**`, `docs/rounds/**` tarihsel kayıtlar değişmedi (yalnız GH-0'ın kendi `docs/rounds/GH-0/` ve `reports/rounds/GH-0/` kayıtları eklendi).
- Spec sürümü 1.0.3; spec §1–6 yalnız ad (K-GH.1); uyumluluk fixture'ları bayt eşit (v3-final'den yeniden üretildi, fark 0).
- CLIENT_CONTRACT uç adları, `X-Consumer`, `status.mdk_version` değişmedi (G0-2).
- `events_hash`'ler değişmedi (uyumluluk 34/34; göç testleri).
- Eşleyici, indeks ve sunucu davranışı değişmedi: çift koşturma fark 0; P1-E referansına karşı E1-6 fark 0 (macOS); indeks baytları yeniden adlandırma öncesiyle aynı (G0-2).
- Secret yok (geçmiş tarandı; `server.token` dosyaları tarihsel test belirteçleri, G0-5).
- Dondurulmuş ölçüm kodu: istisnalar F1–F5 (FREEZE_EXCEPTIONS.md), yeniden dondurma satırları, gözden geçirici onayları; `check_frozen` OK.

## Kalite kapıları
`make test`, `make lint`, `make conformance`: CI'nin zorunlu macOS işi her PR'da (son: PR #8, `3447242` ile aynı kod) yeşil; yerelde G0'da temiz. `make verify`: yukarıdaki çiftler.

## Açık karar talepleri ve insan adımları
- **K-GH.5** (GH-1, insan kararıyla kapandı; K-GH.5-T), açık: **K-GH.6-T** (Linux'ta dizin yazımı).
- **STOP.md 1–7:** tek hesapla "1 onay"; private depoda güvenlik raporlama kanalı; Actions dakikaları; açık talepler; yayım adımlarını ajanın yapması; private depo ve dağıtım; Windows/Node 20 bakım notları.
- Dependabot PR #2 (`clients/obsidian`: `moment`, `obsidian`; güvenlik) açık; birleştirilmedi (sabitleme politikası ve ≥ 14 gün kuralı insan kararı).
- `docs/rounds/P2-0/` bu turda izlenmedi ve değişmedi (başka bir turun dosyaları).

Doğrulama yönergesi bu tur için yazılmadı (GH-0 yönergeleri GOAL + PROMPT); doğrulama bu dosya ve `progress/` kayıtlarıyla yapılır.
