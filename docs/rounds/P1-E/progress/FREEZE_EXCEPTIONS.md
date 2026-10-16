# P1-E — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir. Satırın notu `P1-E/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · E0 · `server/tests/p1d1_criteria.rs` — D1-5 yazan ve iz yazan çağrıların yanıt eşitliği (K-E0.2)
- **Ne:** `d1_5_pipelined_calls_give_identical_logs`'a `writes` kapanışı ve üç kayıt alanı: `write_responses` (sayı, 80), `write_responses_a_vs_b_equal`, `write_responses_pipelined_vs_lockstep_equal` (yalnız `propose` ve `search` yanıtları). `responses_*_equal` alanları aynen kalır, yorum onları bilgi alanı olarak adlandırır. `pass`'in tanımı, çağrı dizisi ve diğer alanlar değişmez (+15 satır).
- **Neden:** ADDENDUM Ş0 / K-E0.2. P1-D.1 doğrulaması Ş0: boru hattındaki salt okunur `blame` paraleldir (sözleşme §5), bu yüzden `responses_a_vs_b_equal` belirlenimsizdir ve `verify.sh pair`'i yazı-turaya bağlıyordu. Ölçümün belirlenimli kısmı (yazan ve iz yazan çağrıların yanıtları) ayrı bir alanla karşılaştırılmalı.
- **Önce/sonra bit eşitliği:** D1-5 üç kez koşturuldu: üçünde de `write_responses_a_vs_b_equal: true`, `write_responses_pipelined_vs_lockstep_equal: true`, `responses_*_equal: false`, `pass: true` (`E0_F1_d1_5_runs.json`: üç koşturma duvar saati alanları dışında eşit; `events_hash32` `96100fc0…`, `traces_hash32` `7f24e518…`, P1-D.1 resmi RUN'ınkiyle aynı). Diğer D1 testleri (`cargo test -p mdkernel-server --test p1d1_criteria`) 8/8 geçer. `propose_denied` 8 → 0, bu istisnanın değil K-E0.4'ün sonucudur: o 8 başlıksız `append` artık `policy_violation` döner (DECISIONS K-E0.4).
- **Gözden geçirici onayı:** E0.review.md.

## F2 · E0 · `research/eval/compare_p1d1.py` — `P1D1_INFO`: sözleşmenin belirlenimsiz bıraktığı iki D1-5 alanı (K-E0.2)
- **Ne:** `P1D1_INFO = {"p1d1/d1_5.json": [("responses_a_vs_b_equal",), ("responses_pipelined_vs_lockstep_equal",)]}`; `_timing` bu kalıpları da hariç tutar; `pair` sonucu `informational_fields` listesini yazar; modül belgesi. `regress`, `EXCLUDED`, `P1D1_FILES`, `P1D1_BYTES` aynı.
- **Neden:** ADDENDUM Ş0, P1-D.1 doğrulaması Ö1 (ilk yol): iki alan "bilgi alanı olarak kalsın". Bir `pair` karşılaştırması aynı commit'in iki koşturmasının belirlenimli her alanını karşılaştırır; belirlenimsiz bir alanı karşılaştırmak ölçüm kodu kusurudur (F16 emsali).
- **Önce/sonra bit eşitliği** (`E0_compare_p1d1_before_after.json`, P1-D.1 resmi RUN `20261016T124520Z-1527365` üzerinde, `P1D1_FILES` alan karşılaştırması; RUN'da e2e çalışma vault'ları olmadığı için `compare_p1d.pair`'in bayt kısmı koşamaz, o kısım bu istisnayla değişmedi):
  - resmi RUN ↔ kendisi: eski 878 alan, fark 0; yeni 876 alan, fark 0;
  - resmi RUN ↔ `responses_a_vs_b_equal`'ı çevrilmiş kopya (doğrulayıcının durumu): eski fark 1 (bu alan, çıkış 3), yeni fark 0;
  - karşılaştırılan yaprak kümesi: yeni = eski − {`responses_a_vs_b_equal`, `responses_pipelined_vs_lockstep_equal`}; başka yaprak eklenmedi ya da çıkmadı. Belirlenimli alanların karşılaştırması bit eşit.
  - Sonrası: iki temiz klondan `verify.sh pair` (E0 kaydı, `reports/rounds/P1-D.1/runs/<RUN>/pair_recheck.json`).
- **Gözden geçirici onayı:** E0.review.md.

## F3 · E0 · `scripts/contract_check.sh` — §1 uçları ve §3 durum tablosu gerçek HTTP ile (Ş4)
- **Ne:** 4. adım: canlı `mdk serve --http 127.0.0.1:0` (bir insan belgeli vault, `[server] max_propose_bytes = 64`, `[index] auto_rebuild = false`, `[identity] human_id`); §2'nin her aracı için `POST /v1/<araç>` geçerli gövdeyle 200 + hatasız JSON nesnesi, `GET /v1/status` 200; §3'ün her kodu gerçek bir istekle tetiklenir ve satırındaki durumla ve o `error.code` ile dönmeli; aktarım kuralları (token yok → 401 `denied`, bilinmeyen yol → 404 `not_found`, araca GET → 405 `invalid_arguments`). Tetikleyicisi olmayan bir sözleşme kodu `missing`'e (`http-code:<kod>`) düşer. Çıktıya `http` alanı; `pass` HTTP adımını da ister. `MDK_BIN` ile başka bir `mdk` ikilisi (kasıtlı test). 1.–3. adımlar aynen.
- **Neden:** ADDENDUM Ş4, P1-D.1 doğrulaması Ş4/Ö3: D1-13 kapısı süreç içi çağrılarla eşliyordu; bir HTTP ucu düşse kapı yeşil kalırdı.
- **Önce/sonra bit eşitliği:** 1.–3. adımların çıktısı (`tools`, `codes`, `missing`) aynı mantıkla üretilir; sözleşme ve `error.rs` K-E0.3 ile `index_stale` satırını kazandığı için `codes` 10 → 11 (her iki tarafta). Güncel ağaçta çıkış 0: 10 uç + `GET /v1/status`, 11 kod, 3 aktarım kuralı, `missing: []`. P1-D.1'in kasıtlı vakaları (`test_tamper_p1d1.sh` contract grubu: satır silme, araç adı, durum değişikliği) yine reddedilir, dokunulmamış kontrol kabul edilir. Yeni kasıtlı vaka (E-16): HTTP'de `checkpoints` yolu kaldırılmış ikili → yalnız HTTP adımı kırmızı (`tools.equal`, `codes.equal` true; tek başarısız uç `POST /v1/checkpoints`); sözleşmede `index_stale` durumu 500 → kırmızı (`scripts/test_tamper_p1e.sh`).
- **Gözden geçirici onayı:** E0.review.md.

## F4 · E0 · `server/tests/contract.rs` — `index_stale` ve yeni `policy_violation` tetikleyicisi (K-E0.3, K-E0.4)
- **Ne:** `http_statuses_of_the_codes`'a `(Code::IndexStale, 503)`; `every_code_from_a_real_call`'a iki gerçek çağrı: başlıksız `append` → `PolicyViolation` (K-E0.4) ve biçimi `mdkernel-index/2`'ye çevrilmiş indeksle `search` → `IndexStale` (K-E0.3). Var olan çağrılar ve sıraları aynı; bozuk manifest → `Internal` vakası en sonda kalır.
- **Neden:** K-E0.3 sözleşme §3'e kod ekler; `contract_check.sh` her sözleşme kodunu bu dosyada gerçek bir çağrıyla arar (D1-13). `want.len() == Code::ALL.len()` denetimi yeni kodu zorunlu kılar.
- **Önce/sonra bit eşitliği:** `cargo test -p mdkernel-server --release --test contract` 4/4 geçer; değişmeden önceki dosya yeni kodla `http_statuses_of_the_codes`'ta (10 ≠ 11) düşüyordu, bu yüzden istisna zorunlu. Diğer üç test birebir aynı.
- **Gözden geçirici onayı:** E0.review.md.
