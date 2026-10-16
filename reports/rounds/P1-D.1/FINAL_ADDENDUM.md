# P1-D.1 — FINAL eki (P1-E/E0, ADDENDUM Ş0)

`FINAL.md` değişmez. Bu ek, P1-D.1 doğrulamasının (VERIFICATION.md, "Şu koşullarla güvenilir") iki koşulunun kapanışını kaydeder.

## Koşul 1 · D1-12 tekrarlanabilirliği (Ş0, Ö1) — kapandı
- Düzeltme (dondurma istisnaları P1-E/F1, F2; K-E0.2): D1-5'in `responses_a_vs_b_equal` ve `responses_pipelined_vs_lockstep_equal` alanları bilgi alanı (sözleşme §5: boru hattında salt okunur `blame` paraleldir); yazan ve iz yazan çağrıların yanıtları yeni `write_responses_a_vs_b_equal` / `write_responses_pipelined_vs_lockstep_equal` alanlarıyla karşılaştırılır. Bildirim: `research/eval/DETERMINISTIC_FIELDS.md`, denetim `scripts/check_compare_fields.sh`.
- Önce/sonra (resmi RUN `20261016T124520Z-1527365`): `docs/rounds/P1-E/progress/E0_compare_p1d1_before_after.json` — karşılaştırılan yapraklar 878 → 876 (yalnız iki bilgi alanı), doğrulayıcının durumu fark 1 → 0.
- **Yeniden koşturma (iki temiz klon, commit `06cd370`, `scripts/verify.sh pair`):** A `20261016T230508Z-06cd370`, B `20261016T233030Z-06cd370`; zincirler 1 532 s ve 1 515 s. **Çıkış 0.** `pair_compare`: 100 982 alan, fark 0, P1-D ve P1-D.1 yapıtları bayt eşit. Her iki zincirde `p1d1_checks.json` D1-1..D1-13 geçti (D1-12'nin `regress` yarısı dahil). D1-5: `write_responses_a_vs_b_equal: true`, `responses_a_vs_b_equal: false` (bilgi). Kayıt: `runs/20261016T230508Z-06cd370/pair_recheck.json`, `verify_pair.log`, `p1d1_checks.json`, `d1_5.json`.
- Bildirimin gerçek koşturmalarda teyidi: `check_compare_fields --runs A B` → 100 982 karşılaştırılan yaprak eşit, bildirilen her belirlenimsiz alan mevcut (`check_compare_fields_runs.json`).
- Not: E0 commit'i P1-D.1 davranışına ADDENDUM'un iki ekini (K-E0.3 `index_stale`, K-E0.4 `policy_violation` + `hint`) de taşır; P1-D.1 zincirinin bütün kriterleri bu kodla geçti. D1-5'in `propose_denied` sayısı K-E0.4 gereği 8 → 0'dır (o çağrılar `policy_violation`).

## Koşul 2 · spec sürüm düzeyi (Ş1, Ö2; K-D1.4-T) — kapandı
K-E0.1 (DECISIONS): CRITERION'daki "veri" = bir uygulamanın **yazdığı** veri; 1.0.3 yama olarak doğrudur. CRITERION metni güncellendi; P1-D.1 `SPEC_EDITS.md`'ye 1.0.2 okuyucularının artık okuyamadığı biçimlerin listesiyle tarihli not düşüldü. D1-10'un sürüm yarısı bu kararla geçer.

## Öneri Ö3 (Ş2–Ş4)
K-E0.3 (`index_stale`), K-E0.4 (başlıklı ekleme, `hint`), `contract_check.sh` HTTP adımı (P1-E/F3) — P1-E E-14..E-16, `docs/rounds/P1-E/progress/E0.md`.
