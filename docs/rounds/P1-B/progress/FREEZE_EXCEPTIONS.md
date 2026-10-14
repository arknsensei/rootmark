# P1-B — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-B/F<n>` anar; `scripts/check_frozen.sh` bunu denetler). Eski satırlar silinmez ve değişmez.

## F1 · B0 · `scripts/check_frozen.sh` — Ö1 (tablo yalnız-ekleme denetimi)
- **Ne:** (1) Her `docs/rounds/*/progress/ROUND_START` sha'sından beri `research/FROZEN.md` tablosundan satır silinmediği/yeniden yazılmadığı `git log -p` (commit'ler) + `git diff HEAD` (çalışma ağacı) ile, ayrıca o sha'daki satırların sırayla ve aynen hâlâ var olduğu uç karşılaştırmasıyla denetlenir; en az bir `ROUND_START` zorunlu. (2) `[append-only]` notlu satır (yalnız `research/FROZEN.md`) bayt eşitliği yerine aynı yalnız-ekleme kuralıyla denetlenir. (3) Yeniden dondurma: aynı kalıplı sonraki satır öncekinin yerine geçer; commit'i öncekinin torunu olmalı, notu bu dosyadaki bir `F<n>` girdisini anmalı.
- **Neden:** GOAL P1-B Ö1 ve TASKS B0. (3) zorunlu: bu turun görevleri dondurulmuş dosyaları açıkça değiştirir (Ö1 `check_frozen.sh`, Ö2 `conformance.rs`, B0 `test_tamper.sh`, B7 `spec/fixtures/MANIFEST.json` + C2 denetleyicisi, B9 `verify.sh`); Ö1'in "satır silinmez/gevşetilmez" kuralıyla bunun tek tutarlı yolu satır **eklemektir** (ASSUMPTIONS B0-1).
- **Diff:** `git diff 983c62d -- scripts/check_frozen.sh` — satır işleme ilk döngüsü korunur (aynı ilk-commit kuralı, aynı glob, aynı `git diff --name-only`); eklenenler: `append_only()`, yeniden dondurma döngüsü, `ROUND_START` bölümü.
- **Önce/sonra bit eşitliği:** önce (`983c62d`'deki denetleyici, `983c62d` ağacında): 25 satırın hepsi "unchanged", `check_frozen: OK`. Sonra (B0 commit'i): aynı 22 yeniden dondurulmamış satır aynı dosya sayılarıyla "unchanged"; yeniden dondurulan 3 satır "superseded", yeni satırlar B0'dan itibaren "unchanged"; `check_frozen: OK`. Kayıt: B0.md.
- **Gözden geçirici onayı:** B0.review.md.

## F2 · B0 · `core/tests/conformance.rs` — Ö2 (fixture kökü çalışma anında)
- **Ne:** `fixtures()` derlenmiş `env!("CARGO_MANIFEST_DIR")` yerine `MDK_FIXTURES` (yoksa `<cwd>/../spec/fixtures`) ile çalışma anında çözülür; kök `MANIFEST.json` içermiyorsa hata. Rapora `fixtures_root` (mutlak, kanonik), `fixtures_root_source`, `fixtures_manifest_hash` (MANIFEST.json blake3, 32 hex) eklenir. `hash()` aynı hesabı `blake32()` üzerinden yapar.
- **Neden:** GOAL P1-B Ö2; taşınmış `target/` ile derlenmiş yol eski ağacın fixture'larını okuyabiliyordu.
- **Önce/sonra bit eşitliği:** `983c62d` ve B0 ağaçlarında `make conformance` → `tests[].{id,result,actual_hash,expected_hash}` JSON'u bayt eşit (blake3 `01e68807a1441ee793d412bbd3803802`), 17/17, `level_claimed 1`.
- **Gözden geçirici onayı:** B0.review.md.

## F3 · B0 · `scripts/test_tamper.sh` — B-7, B-8 kasıtlı vakaları
- **Ne:** iki yeni bölüm: (4) `target/` ile birlikte (mtime korunarak) kopyalanmış ağaçta fixture bozulur → `make conformance` hata ve `conformance.json` `fixtures_root` kopyanın yolu; (5) temiz klonda `check_frozen.sh` 0, sonra FROZEN.md satırı silinir (commit'siz ve commit'li) ve bir satır gevşetilir (commit'li) → her biri hata.
- **Neden:** TASKS B0 kabulü ("`scripts/test_tamper.sh` iki yeni vaka").
- **Önce/sonra bit eşitliği:** mevcut 8 vaka (1–3. bölümler) değişmedi (diff yalnız başlık yorumu ve 4–5. bölümler); B0 koşturmasında 8 eski vaka aynı etiket ve aynı hata kalıplarıyla yakalandı. Kayıt: B0.md.
- **Gözden geçirici onayı:** B0.review.md.

## F4 · B0 · `scripts/check_frozen.sh` — tablo işaretçisi açığı (B0 gözden geçirmesi)
- **Ne:** dosyada tam olarak bir `<!-- frozen-table -->` ve bir `<!-- /frozen-table -->` işaretçisi (bu sırayla) zorunlu; `ROUND_START`/[append-only] geçmiş denetiminde eklenen (`+`) işaretçi satırı da ihlal.
- **Neden:** B0 gözden geçirici: son denetim noktasından sonra eklenen bir satır, üstüne ikinci bir bitiş işaretçisi eklenerek hiçbir satır silinmeden tablodan çıkarılabiliyordu (klonda yeniden üretildi).
- **Diff:** `git diff 3f2be39 -- scripts/check_frozen.sh` — işaretçi sayımı bloğu ve `grep -E` kalıbına `^\+.*<!-- /?frozen-table -->` eklendi; başka değişiklik yok.
- **Önce/sonra bit eşitliği:** temiz ağaçta önce/sonra aynı satır raporları ve `check_frozen: OK`; `test_tamper.sh` 12/12 + yeni vaka (ikinci bitiş işaretçisi) yakalanıyor. Kayıt: B0.md.
- **Gözden geçirici onayı:** B0.review.md (ek).
