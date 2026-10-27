# Tam çift — 2. deneme (5b235b0), geçmedi: gerçek Obsidian ortamı

- `caffeinate -i -s make verify`, toplam 7448 s, çıkış 2.
- Zincir A (RUN `20261026T201903Z-5b235b0`, 4813 s): sayılan FAIL 3, üçü de P2-0.3'ün gerçek
  Obsidian kümesi D-B test-4'ten: H-P2-0.3b, H-P2-0.3b′, H-P2-0.3c. D-B'de
  `false_unverified_rate` 0.3149 ve `latency_unsettled` 30; P2-B'nin resmi RUN'ında ve 1. denemede
  0.0732 ve 0 (aynı tohum, aynı üreteç). Kaydedici Obsidian'ın yazılarını geç/eksik gördü.
- Zincir B (RUN `20261026T213922Z-5b235b0`): P2-0.3 adımında öldü —
  `session t016: typed notes not written by Obsidian within 6 s: origins.md` (sessions.mjs).
- P2-R'nin kodu eklenti, dosya izleme ve alım yoluna dokunmuyor; 1. denemede aynı commit'te
  iki zincirde de bu küme referansla aynıydı. İki zincirde birden Obsidian'ın yazmaması ortam
  kaynaklı (gerçek UI: ekran/oturum durumu, kullanıcı etkinliği). Ölçüm kodu değişmedi, liste
  genişletilmedi; çift bir kez daha (`caffeinate -d -i -s -u`, ekran da uyanık) koşuldu.
- Burada: `pair.out`, A'nın `*_checks.json`'ları ve `p2_0_3/d_b.json`.
