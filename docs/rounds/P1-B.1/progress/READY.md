# P1-B.1 — Hazır

doğrulama için hazır

- Commit (FINAL.md, resmi RUN ve tüm görev kayıtları + incelemeler): `de77219b9a96d45bcd503cd1ad6c5e468053939e` (`P1-B.1/B1.4: review (TAMAMLANDI)`); FINAL.md ve resmi RUN ilk kez `472f691` (`P1-B.1/B1.4: FINAL.md …`).
- Ölçülen kod commit'i (RUN'ın `git_sha`, 15 fazın hepsi, `worktree_clean true`): `226b511bda34c1e09a9f8e43890df8ac85b9dfec` (`P1-B.1/B1.4-pre`); sonraki commit'ler yalnız `docs/` ve `reports/`.
- RUN: **`20261014T212243Z-226b511`** (`make verify` pair'inin A zinciri; B: `20261014T213229Z-226b511`; `compare_p1b` 2195 alan fark 0, `events.jsonl`/`traces.jsonl`/`checkpoints/` A↔B bayt eşit). `p1b1-checks` ve rapor aynı RUN'a A klonunda.
- Rapor: `reports/rounds/P1-B.1/FINAL.md`; RUN dizini: `reports/rounds/P1-B.1/runs/20261014T212243Z-226b511/`.
- Tur başlangıcı: `ROUND_START` = `63a4004`; `p1b-final` = `2df32a7`.
- Sonuç (FINAL, GOAL V8): B1-1..B1-7 geçti → P1-B karar kuralının B-5 dalı kapanır; P1-C başlayabilir.
- Okuma notları: `regress_vs_p1b.json` içindeki `H4-2_equal: false` hariç tutulan alanları da sayan ham bayraktır; B1-6 kararı `B1-6_equal: true`. `fixtures_manifest_hash` farkı yalnız C2-07'den doğar (MANIFEST'te tek değişen satır `C2-07/expected.json`).
- Kayıtlı sınırlar: K-B1.1 türetim sınırı (ASSUMPTIONS B11-2; DECISIONS K-B1.1 notu) — günlüğün ilk katman taşıyan olayı olan `LayerChanged` silinip yeniden numaralanırsa vault açılır, katman aynı, `layer_source = derived`. 1.0 formüllü `traces.jsonl` 1.0.1 çekirdekte açılmaz (B12-2, `Err`). `query_hash` küçük harf hex (B12-1). `traces.rs`'te karar gereği 2 satır (B12-3).
- Dondurma istisnaları: F1 (`check_frozen.sh` noktalı tur kimliği), F2 (`spec/fixtures` C2-07), F3 (`conformance.rs` spec_version), F4 (`check_v3_compat.sh`).

Doğrulamayı ana ajan yapmadı (GOAL "Doğrulama", ayrı oturum).
