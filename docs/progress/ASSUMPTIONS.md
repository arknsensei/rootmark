# Varsayımlar

Her satır etkileyebileceği hipotezle etiketlenir: `[H1]`, `[H2]`, `[H3]`, `[H4]`, `[-]`.

- [-] T0: `ASSUMPTIONS.md`, `FREEZE_EXCEPTIONS.md`, `STOP.md`, `EVAL_FREEZE_ISSUES.md` dosyaları `docs/progress/` altında tutulur (CLAUDE.md yol vermiyor).
- [-] T0: Python sürümü 3.12'ye sabitlendi (`requires-python = ">=3.12,<3.13"`); `networkx==3.7` 3.11'i desteklemiyor. CLAUDE.md "3.11+" koşulu sağlanır.
- [-] T0: Rust çekirdek crate'inin kütüphane adı `mdkcore`; Python modül adı `mdkernel_core` bağlama crate'ine ayrıldığı için çakışmayı önler.
- [-] T0: `uv` projesi `package = false`; uzantı modülü yalnız `maturin develop --release` ile kurulur (`make build`). `uv sync` uzantıyı kaldırabildiği için `make setup` sırayla ikisini koşturur.
- [H1][H4] T2: Nesne deposu yalnız anahtarının belirlediği alanları saklar. `objects/<vid>.json` = `{version_id, block_type, normalized_text, heading_level}`; `snapshots/<sid>.json` = `{snapshot_id, version_ids}`. Gerekçe: SPEC §4 "aynı kimlik farklı içerik → Err" kuralı, ham metin (yalnız boşluk/satır kırılımı farklı, aynı `version_id`; rust-lang/book'ta paragraf yeniden kırma commit'leri) veya snapshot'a özgü alanlar (doc_path, soy, ebeveyn; aynı içerikli iki doküman veya geri alma commit'i aynı `snapshot_id`) saklansaydı meşru girdilerde hata üretirdi. Oluşum alanları (soy, ebeveynler, köken, frontmatter, ebeveyn snapshot) olay günlüğündedir; `heading_path` blok sırasından yeniden hesaplanır.
- [H2] T2: `Vault::block(version_id)` ve chunk metni ham markdown yerine `normalized_text` kullanır (ham metin içerik-adresli değildir, bkz. üstteki madde). Kod bloklarında normalize metin satır yapısını korur.
- [H4] T2: `SnapshotCommitted` olayı tam sıralı `(lineage_id, version_id)` listesini, `frontmatter` ve `parent_snapshot` alanlarını taşır. `replay` blok olaylarını uygulayıp her commit'te canlı soy kümesinin bu listeyle eşit olduğunu ve `snapshot_id`'nin yeniden hesaplandığını doğrular.
- [H4] T2: `events_hash` = tüm `events.jsonl` baytlarının blake3 hex ilk 32 karakteri.
