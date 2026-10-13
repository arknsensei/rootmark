# P1-A — Yeniden Yapılandırma ve Dondurma

## Amaç
Prototip v2/v3 deposunu `docs/PACKAGES.md` monorepo yapısına taşımak; `spec/SPEC.md` §1–6'yı yürütülebilir uyumluluk testlerine (C1-01..C1-17) bağlamak; prototip ölçüm altyapısını `research/` altında kalıcı regresyon kapısına dönüştürmek; doğrulayıcının v3 önerilerini kapatmak. Yeni ürün özelliği yok.

## Kapsam
- Monorepo iskeleti: `core/`, `index/` (boş crate, trait tanımları), `server/` (boş crate), `bindings/python/`, `clients/` (boş), `research/`, `scripts/`, `spec/fixtures/`.
- `core/`: prototip crate taşınır; `MatcherVersion::V2` yolu korunur (bayrak); public API `spec` terimleriyle hizalanır (`Vault`, `Block`, `Snapshot`, `Event`, `Checkpoint` tipi henüz sadece tanım).
- `spec/fixtures/`: C1 testlerinin her biri için girdi + beklenen; `make conformance` → `reports/conformance.json`.
- `research/`: prototip `eval/` + `mdkernel/` Python kodu; `make verify` uyarlanır; **holdout seti** (yeni SEED `20261013:holdout`, D3'ten easy/hard ile kesişmeyen 100 doküman, D1-hard op dağılımı) üretilir ve v3 eşikleri (H1-H*) üzerinde koşturulur; **bölme eşiği zorluk bölgesi** (her iki parçanın düzenlendiği bölme, ≤ 8 kelimelik parçalar, ≥ %15 düzenlemeli birleştirme) ayrı bir sentetik dilim olarak eklenir ve THETA_SPLIT ızgarası `{0.45..0.85 adım 0.10}` ile raporlanır (yalnız bilgi; sabit değişmez).
- Dondurma politikası: `research/eval/freeze_check*.py` dondurma listesine alınır; denetleyiciler derlenmiş sabitlere bağlı olamaz (fixture kendi parametrelerini taşır).

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| A-1 | C1-01..C1-17 | hepsi geçer |
| A-2 | Holdout sette H1-H1 / H1-H2 / H1-H3 / H1-H4 | ≥ 0.90 / ≤ 0.03 / ≥ 0.70 / ≥ 0.90 |
| A-3 | v3 D1-easy, D1-hard, D2 sayıları `v3-final` ile | birebir |
| A-4 | `make verify` çift koşturma, kalite alanları maks. göreli fark | 0 |
| A-5 | mini_vault `events_hash`: v2 bayrağı `8914f9b418ae360d49c85c951050f432`, v3 `808a41f21a43e121d7d688fee9f5bc24` | eşit |
| A-6 | Bağımlılık yönü CI kontrolü (`cargo tree`, import tarayıcı) | ters bağımlılık 0 |
| A-7 | Bölme eşiği zorluk dilimi: THETA_SPLIT ızgarasında farklı amaç değeri sayısı | bilgi (eşik yok); raporlanır |

## Karar kuralı
A-1..A-6 geçti → spec §1–6 "frozen" damgası teyit edilir, P1-B başlar. A-2 kaldı → `DECISIONS.md`'ye karar talebi; eşleyici turu açılır, P1-B beklemez (yönetişim primitifleri eşleyiciden bağımsız). A-3/A-4/A-5 kaldı → taşıma hatası; P1-B başlamaz.

## Yapılmayacaklar
Katman/terfi/kontrol noktası/iz kodu (P1-B); tantivy, ONNX (P1-C); MCP (P1-D); istemci; H2 koşturması; API çağrısı.
