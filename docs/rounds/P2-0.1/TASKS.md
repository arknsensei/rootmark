# P2-0.1 — Görevler

Sıra: Y0 → Y1 → Y2 → Y3 → Y4 → Y5 → Y6 → Y7. Progress `docs/rounds/P2-0.1/progress/`. Commit öneki `P2-0.1/Y<n>:`. Dal `p2-0.1/round` (K-P2.1).

## Y0 — Tur açılışı
**Yapar:** `ROUND_START` (= PR #11'in birleştiği `main` commit'i); DECISIONS: tur açılışı, P2-0 doğrulamasının üç önerisinin bu turdaki karşılığı (Ö1 → Y1/Y5, Ö2 → Y4, Ö3 → Y4), F5 (eşleşmeyen hash) → Y4; D-B test-2 tohumu ve D-B2/D-A2/D-A′/D-C′ üreteç tanımları **ölçümden önce** `ASSUMPTIONS.md`'ye; regresyon referansı P2-0 resmi RUN'ı `20261018T161850Z-e378be2`.
**Kabul:** `check_frozen.sh`, `check_deps.sh`; tohum ve üreteç tanımları commit'li, veri yok.

## Y1 — Bağlantı güncellemesi ve yeniden adlandırma tasdiki (T1)
**Yapar:** `clients/obsidian`: yeniden adlandırma sırasında Obsidian'ın yan notlara yazdığı güncellemeleri yakalar (`vault.on('rename')` + `metadataCache`/`fileManager` gözlemi ya da eşdeğeri; **yalnız Obsidian sürecinin kendi yazımları**, dış değişiklik değil) ve `attest {link_update_of}` gönderir; `server`: fark denetimi (canonical baş ↔ yeni baytlar: yalnız bağlantı hedef metni; parser ile blok bazında), aksi 403 `denied`; `status.attestation.link_updates` sayacı. Spec §7.1 metni; SPEC_EDITS.
**Kabul:** birim testler (yalnız bağlantı farkı → kabul; başka fark → ret; ajan içerikli notta bağlantı güncellemesi → ajan blokları `unverified` kalır); gerçek Obsidian duman denemesi 5 yeniden adlandırma (ölçüm değil); gözden geçirici + güvenlik incelemesi (aklama senaryoları).

## Y2 — Gözlem başına blok eşleşmesi (T2)
**Yapar:** `attest.content` (isteğe bağlı, bayt); sunucu: gözlem blokları ⊇ tasdik blokları → kısmi eşleşme; günlük satırı `partial: true`; olaylar katmanı taşır; `content` saklanmaz. Spec §7.1/§8; CONFORMANCE C2-13 tanımı + fixture. Eklenti ve CLI `content` gönderir.
**Kabul:** birim testler (tam eşleşme değişmez; kısmi: insan blokları `canonical`, ajan `unverified`; ajanın insan bloğuyla aynı metni yazması sızıntı sayılmaz); `make conformance` 38/38; determinizm.

## Y3 — Obsidian dışı kanallar (T3)
**Yapar:** CLI `rootmark attest --renamed-from <eski> <yeni>` ve `content` gönderimi; `research/eval/datasets/p2_0_1_sets.py`: D-B2 düzenleyici öykünmesi (atomik kayıt = geçici dosya + `rename(2)`), D-A2 ajan yolları, D-A′ ve D-C′ gerçek Obsidian senaryoları; `scripts/agent_fs_writer.py` D-A2 kipleri.
**Kabul:** üreteçler deterministik (aynı tohum → aynı dizi); fail-fast; D-B2'nin her oturumu tek denetim fonksiyonuyla ölçülür (kod gösterilir).

## Y4 — Tehdit modeli, özet, eşleşmeyen hash (T4)
**Yapar:** `docs/THREAT_MODEL.md` + `docs/SECURITY.md` + spec §7.1 bilgilendirici not (CDP yolu, insan kanalı tanımı, aşama 2'nin sınırı, kapsam dışı); `scripts/verify.sh` FAIL özeti turun sınıflandırıcısına bağlanır (FREEZE_EXCEPTIONS girdisi, önce/sonra kanıt, gözden geçirici onayı); eşleşmeyen hash: `matched: false`, `status.attestation.unmatched`.
**Kabul:** `make verify` özetinde sayılan FAIL = `historical` sınıflandırması; metinler gözden geçirici onaylı.

## Y5 — Ölçüm
**Yapar:** `run_p2_0_1.py --phase test` (ayar yok; W = 0.5 `Z5_tuning.json`'dan okunur, yoksa durur); D-A, D-A′, D-B test-2 (gerçek Obsidian), D-B2, D-A2, D-C, D-C′, D-E; `p2_0_1_checks.json`; her hipotez kaynağıyla.
**Kabul:** H-P2-0.1a..k ölçüldü; test-2 tohumu Y0'daki; P2-0 test seti okunmadı (kod ve git sırası).

## Y6 — Doğrulama zinciri
**Yapar:** `verify.sh`: conformance → regress (P2-0 referansı) → D-E → D-A/D-A2 → D-C → D-B/D-B2/D-A′/D-C′ (gerçek Obsidian yoksa ilgili kümeler "atlandı") → tamper (sahte bağlantı güncellemesi, yanlış `content`, `partial` sahteciliği, MCP attest) → pair compare.
**Kabul:** H-P2-0.1i, 0.1k; kasıtlı vakalar.

## Y7 — Rapor ve hazır
**Yapar:** `report_p2_0_1.py`; `reports/rounds/P2-0.1/FINAL.md` (hipotez tablosu, karar kuralı dalı, tehdit modeli metni, NORTH_STAR önerisi); `READY.md`; tek PR.
**Kabul:** izlenebilirlik; karar kuralı; NORTH_STAR değiştirilmez.
