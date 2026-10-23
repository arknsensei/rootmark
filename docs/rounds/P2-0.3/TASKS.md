# P2-0.3 — Görevler

Sıra: R0 → R1 → R2. Progress `docs/rounds/P2-0.3/progress/`. Commit öneki `P2-0.3/R<n>:`. Dal `p2-0.3/round` (yalnız birleşmiş `main`'den, K-P2.2). Ürün ve ölçüm kodu değişmez; değişen tek şeyler tohum kaydı, rapor üreteci sarmalayıcısı ve belgeler.

## R0 — Ön-kayıt
**Yapar:** `ROUND_START` (= `main` HEAD, P2-A birleşmiş); DECISIONS: tur açılışı; `ASSUMPTIONS.md`: test-4 tohumu (`blake3("P2-0.3/D-B")[:8]` → sayı), üreteç ve dağılımın P2-0.2 X0 kayıtlarıyla birebir olduğu (dosya hash'leri), `run_p2_0_3.py` yalnız `run_p2_0_2`'yi yeni tohum ve RUN adıyla çağıran ince sarmalayıcı (kod gösterilir); `report_p2_0_3.py` aynı biçimde. `make verify`'a P2-0.3 adımı (dondurma istisnası: yalnız sarmalayıcı eklenir, mevcut adımlar değişmez; önce/sonra kanıt).
**Kabul:** `git diff main -- core index server clients spec` boş; tohum ve sarmalayıcı commit'li, veri yok; `check_frozen`, `check_deps`.

## R1 — Ölçüm
**Yapar:** `make verify` (yüksüz çift) → resmi RUN = A; `VERIFY_LOAD=1 make verify` (yüklü çift); D-B test-4 dört zincirde; diğer kümeler regresyon; `p2_0_3_checks.json`; dört zincirin D-B değerleri ve biçim/oturum türü tabloları.
**Kabul:** H-P2-0.3a..j ölçüldü; çiftler eşit; tohum R0'daki; önceki tohumlar okunmadı.

## R2 — Rapor ve hazır
**Yapar:** `reports/rounds/P2-0.3/FINAL.md` (hipotez tablosu, dört zincir, P2-0.2 ve P2-A bilgi ölçümüyle karşılaştırma, karar kuralı dalı, NORTH_STAR notu için öneri metni, varsayılan değişikliğinin uygulama planı — bu turda uygulanmaz); `READY.md`; tek PR.
**Kabul:** izlenebilirlik; karar kuralı; ürün değişmedi (`git diff` kanıtı FINAL'de).
