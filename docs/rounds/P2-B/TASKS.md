# P2-B — Görevler

Genel kurallar:
- Sıra: B0 → B1 → B2 → B3 → B4 → B5.
- Progress `docs/rounds/P2-B/progress/`. Commit öneki `P2-B/B<n>:`.
- Dal `p2-b/round`, yalnız birleşmiş `main`'den (K-P2.2). `ROUND_START` = Blok 1 kapanış commit'inin birleşmiş `main` HEAD'i.
- Geliştirme turu dozu (K-P2.4): gözden geçirici yalnız B5'te, tek tur. Güvenlik incelemesi B1 (bayat dosyada yanlış ofset, yol dışı okuma), B3 (`events` filtresiyle vault dışı yol, ağır sorguyla kaynak tüketimi) ve B4 (b) (VS Code iskeletinin token saklaması) için B5'in gözden geçiricisinde ayrı başlık.
- `make verify FAST=1` görev sonlarında; tam `make verify` B5'te bir kez.

## B0 — Açılış, Blok 1 kalıntıları
**Yapar:**
- `ROUND_START`; DECISIONS'ta tur açılışı.
- GOAL 1 (a): CLIENT_CONTRACT arama `path_prefix` NFC katlama cümlesi; iki test (arama katlar, kural reddeder).
- GOAL 1 (b): `migrate --dry-run` planı (`server` göç çerçevesi) ve CHANGELOG "Unreleased" geri dönüş satırı.
- GOAL 1 (c):
  - (5) `core` okuma kuralı ve `vault.rs` replay denetimi; testler.
  - (8) `status.migrations.artifacts.policy`: okunamayan tarif hatası; test.
  - (11) Alınmamış dış değişiklikten sonra eşleşen `propose` testi.

**Kabul:** B-1; mevcut testler aynen; `check_frozen`, `check_deps`; `make verify FAST=1`.

## B1 — Blok konumu
**Yapar:**
- `core`:
  - `Block`/`BlockDraft`'ın satır aralığına bayt aralığı eklenir. Ayrıştırıcının `into_offset_iter` aralıkları kullanılır; ön-bilgi dahil, dosya baytlarına göre.
  - Konum olaylara ve kimliklere **girmez**; serileştirilen olay baytları aynen.
- `server`:
  - `blame`, `replay` ve `search` isabetlerinde `loc`; `loc_stale` (diskteki dosyanın hash'i son alınan sürümle karşılaştırılır).
  - `tools.rs` araç açıklamaları.
- CLIENT_CONTRACT §2.5 (v2, `loc`, `loc_stale`, v1 uyumluluğu) ve §6 sürüm; `scripts/gen_client_types.sh`.
- Spec'e gerekiyorsa yalnız §9 bilgilendirici notu (`SPEC_EDITS.md` satırı, CRITERION).

**Kabul:** B-2; `make conformance` 39/39, C3-06 aynen; P2-A'nın altı eski vault'unda `events_hash` aynen.

## B2 — Hizalama ölçümü
**Yapar:**
- `research/data/d_notes/`:
  - Manifest: kaynak depolar, revizyon, lisans, dosya başına sha256.
  - Seçim betiği, tohum, sınıf sayımı.
- Ön-kayıt commit'i **ölçümden önce**: manifest, tohum, ölçüm betiği `research/eval/p2b_align.py`. Betik `bindings/python`'a ya da `rootmark` CLI JSON çıktısına bakar.
- Kenar vakası üreteci: sabit tohum, sınıf başına ≥ 200.
- Ölçüm:
  - Dilim ve örtü denetimi; sınıf başına hata.
  - Eski sıra eşlemesinin eşleyebildiği belge oranı.
  - Sonuç `research/reports/runs/<RUN>/p2b/align.json`.
- Lisans koşulunu sağlayan korpus bulunamazsa STOP ve PARK; korpus koşulu gevşetilmez.

**Kabul:** B-3, B-4 (değerler raporlanır; kalırsa karar kuralı).

## B3 — Olay akışı ve zaman filtresi
**Yapar:**
- `server`:
  - `events` aracı (HTTP `POST /v1/events`, MCP `events`).
  - Seq sırası, `limit` üst sınırı ve `next_seq`.
  - `kinds` ve `path_prefix` filtresi (B0 (a)'nın kuralı).
  - Yazma yok.
- `index`/`server`:
  - `[index] date_from_path` ya da `date_field` (yükleme doğrulaması, alan adlı hata).
  - Arama `date_from`, `date_to`; `trace` filtre kaydı (L3'ün `trace_filters` yolu); dışarıda kalan belge sayısı.
- CLIENT_CONTRACT §2.2, yeni §2.8 `events`; tipler.

**Kabul:** B-5, B-6; filtresiz `query_hash` aynen (P2-L'nin L-7 testleri aynen).

## B4 — İki istemci
**Yapar:**
- `clients/obsidian`:
  - `align.ts` `loc`'u kullanır; yoksa sıra eşlemesi kalır.
  - `loc_stale` paneli; katman işaretleri.
  - vitest; B2 korpusundan ≥ 20 notla e2e (mevcut e2e altyapısı).
- `clients/vscode`:
  - İskelet: `package.json`, `src/extension.ts`, HTTP taşıması; `contract.gen.ts` paylaşılır (tek kaynak, kopya değil).
  - Komutlar: `status`, `search`, `blame` süslemesi, `history`, `promote`/`demote`.
  - Testler (`@vscode/test-electron` ya da saf birim testleri; `make test`'e bağlanır).
  - README (tasdik yok notu).
- Sözleşme denetleyicisi: çağrı kaydı + şema denetimi; iki istemcinin bütün koşularında çalışır.
- `check_dep_age`: yeni npm bağımlılıkları.

**Kabul:** B-7, B-8.

## B5 — Zincir, rapor, hazır
**Yapar:**
- `verify.sh`: P2-B adımı (B-2 `events_hash`/C3-06, B-3/B-4 hizalama, B-5/B-6, B-9 determinizm). Dondurma istisnası kaydıyla, önce/sonra.
- Tam `make verify` bir kez.
- Gözden geçirici: tek tur, bütün kabul ölçütleri, güvenlik başlığı (B1, B3, B4 (b)).
- FINAL: kriter tablosu, karar kuralı dalı, Blok 2 sonu doğrulayıcısı için P2-B özeti.
- READY; tek PR.

**Kabul:** B-9 (tam çift), B-10; izlenebilirlik; karar kuralı.
