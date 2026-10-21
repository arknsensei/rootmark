# Resmi olmayan koşular (P2-A/A5)

- `dev_20261020T203735Z-c51159b/` — geliştirme RUN'ı (tek zincir, `c51159b`): listeler ve dışlamalar bundan yazıldı (FREEZE_EXCEPTIONS F7). `p2a_checks.json` — `tests` kümesinin 0 test gördüğü (ayrıştırma hatası) ilk kayıt; `d_a.json` ve `d_a_events.jsonl` — D-A yarışı (`a007`, `a017`): P2-0.2 resmi RUN'ının `p2_0_2/d_a/events.jsonl`'iyle `CheckpointCreated` dışındaki olaylar aynı çoklu küme, yalnız kontrol noktası olaylarının yeri ve sayısı (2 eksik) farklı.
- `attempt1_20261020T212900Z-40cd417/` — resmi çiftin 1. denemesi (`40cd417`): `pair_compare.json` (fark 2: E1-9'un `search_before_rebuild.error`'u, `hint`'teki vault yolu), A zincirinin `p2a_checks.json`'u, `verify_pair.log`. Düzeltme `P2-A/A5-fix`; resmi çift `../20261020T230453Z-1a775d3/`.
