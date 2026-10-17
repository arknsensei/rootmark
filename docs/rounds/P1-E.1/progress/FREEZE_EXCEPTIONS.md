# P1-E.1 — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir. Satırın notu `P1-E.1/F<n>`'i anar. Eski satırlar silinmez ve değişmez.

## F1 · E1.1 · `server/tests/contract.rs` — `internal`'ın tetikleyicisi manifest'le uyuşmayan indeks dosyası (K-E1.2)
- **Ne:** `a_damaged_index_is_internal` bozuk JSON manifest yerine `records.jsonl`'e bir bayt ekler (manifest okunur, dosya blake3'ü uyuşmaz → `internal`); belge yorumu buna göre. `every_code_from_a_real_call`'ın son adımı: bozuk JSON manifest artık `IndexStale` beklentisiyle (K-E1.2), ardından taze manifest geri yazılıp `records.jsonl` bozulur ve `Internal` beklenir. Diğer vakalar, sıraları ve `http_statuses_of_the_codes` aynı (+15/−3 satır).
- **Neden:** K-E1.2 / GOAL P1-E.1 E1-2: okunamayan manifest indeks isteyen çağrılarda 503 `index_stale`'dir. Eski dosya bu durumda `internal` bekliyordu; davranış kararla değiştiği için test aynı vakada yeni beklentiyi, `internal` için de hâlâ `internal` olan bir bozulmayı sınar. `contract_check.sh` her sözleşme kodunu bu dosyada gerçek bir `expect_code` ile arar; `Internal` ve `IndexStale` ikisi de gerçek çağrıyla kalır.
- **Önce/sonra:** önce (yeni sunucuyla) `cargo test -p mdkernel-server --release --test contract`: 2/4 (iki vaka `index damaged … rebuild it` iletisiyle `IndexStale` aldı, `Internal` bekliyordu); sonra 4/4. `http_statuses_of_the_codes` ve `every_tool_answers` birebir aynı.
- **Gözden geçirici onayı:** E1.1.review.md.

## F2 · E1.1 · `scripts/contract_check.sh` — `status` her indeks durumunda (E1-8) ve `internal` tetikleyicisi (K-E1.2)
- **Ne:** (1) `index_stale` adımı yaşlandırmadan önce indeksin kopyasını alır; `internal` adımı kopyayı geri koyup `records.jsonl`'e bir bayt ekler (bozuk JSON manifest artık `index_stale`). (2) Yeni alt adım: sözleşmenin §2 `status` satırındaki `index {…}` alan listesi okunur; indeks `ok`, `stale` (biçim `/2`), `damaged` (manifest JSON değil) ve `missing` (indeks silinmiş) iken `GET` ve `POST /v1/status` 200, hatasız, `index` nesne, `index.state` o durum ve listedeki her alan mevcut olmalı; `search` `ok`'ta 200, diğer üçünde 503 `index_stale`. Çıktıya `http.status_index` ve `http.status_index_fields`; `http.pass` dört durumun hepsini ister; başarısız durum `missing`'e `http-status-index:<durum>`; §2 satırında `index {…}` yoksa `missing`'e `contract:status.index fields`. 1.–3. adımlar, uçlar, kod istekleri (`internal` hariç) ve aktarım kuralları aynen.
- **Neden:** GOAL P1-E.1 E1-8 ("`contract_check.sh` HTTP adımı `status`'un yeni alanını ve 503 istisnasının `status`'u kapsamadığını denetler"); K-E1.2 ile `internal`'ın eski tetikleyicisi `index_stale` olur.
- **Önce/sonra bit eşitliği** (güncel ağaç, yeni sunucu): eski denetleyici çıkış 1, tek eksik `http-code:internal (status 503, code index_stale; contract 500)`; yeni denetleyici çıkış 0. İki çıktı arasında `tools`, `codes`, `http.endpoints`, `http.transport` eşit; `http.codes`'ta yalnız `internal` satırı farklı (önce 503, sonra 500); yeni alanlar `status_index` (4/4 ok) ve `status_index_fields` (`state, format, expected_format, auto_rebuild`). Kasıtlı vakalar: `scripts/test_tamper_p1e1.sh` contract-status grubu (E1.1.md).
- **Gözden geçirici onayı:** E1.1.review.md.

## F3 · E1.4 · `scripts/verify.sh` — zincir sonunda FAIL özeti (K-E1.4)
- **Ne:** `chain`'in sonunda, "chain finished" satırından önce: `python -m eval.historical --run-dir reports/runs/$run` çıktısı `[verify summary]` önekiyle stderr'e (+6 satır, yorum dahil). Özet çıkış kodunu değiştirmez (`|| true`; kriterler kaydedilir, ölümcül değildir); sınıflandırıcının hatası stderr'de görünür. Zincir adımları, sırası, `pair`, `compare*` aynen.
- **Neden:** GOAL P1-E.1 K-E1.4, E1.4: "eski kaydedicilerin birikmiş FAIL'leri … `verify.sh` özetinde sayılmaz". Bugüne kadar zincirde FAIL'ler yalnız ayrı kayıtlarda görünüyordu; özet her FAIL'i ya kararıyla "tarihsel" ya da sayılan FAIL olarak yazar.
- **Önce/sonra bit eşitliği:** özet yalnız stderr'e yazar; RUN dizinine dosya eklemez, hiçbir kayıt değişmez — `pair`'in karşılaştırdığı yapıtlar önce ve sonra aynı. `bash -n` temiz. P1-E resmi RUN'ı üzerinde `eval.historical`: 8 tarihsel FAIL (B1-3, C1-7, C-3, C-11, D-2, D-7, D-9, E-7), sayılan 0.
- **Gözden geçirici onayı:** E1.4.review.md.

## F4 · E1.5 · `scripts/verify.sh` — P1-E.1 zinciri ve `pair` → `compare_p1e1`
- **Ne:** P1-E F7 emsaliyle aynı biçim: başlık yorumuna P1-E.1 paragrafı; `chain`'de `make p1e-checks`'ten sonra `make p1e1-checks RUN=$run` (2 satır); `pair`'in son satırı `compare_p1e` → `compare_p1e1` (P1-E kümesi bunun içinde, aynen `compare_p1e.pair`); yeni `compare_p1e1` işlevi, `compare-p1e1` alt komutu, kullanım iletisi. Önceki adımlar, sıraları, F3 özeti ve `compare`…`compare-p1e` aynen.
- **Neden:** GOAL P1-E.1 E1.5 ("`verify.sh` genişletme"); E1-7'nin çift koşturması P1-E.1 ölçümlerini iki bağımsız klonda koşturmalı.
- **Önce/sonra bit eşitliği:** önceki adımlar değişmeden koşar; P1-E çift kümesinin bütün alanları ve baytları `compare_p1e1 pair` içinde `compare_p1e.pair` ile karşılaştırılır (resmi çift kaydı E1.5.md). `bash -n` temiz.
- **Gözden geçirici onayı:** E1.5.review.md.

## F5 · E1.5 · `research/eval/check_compare_fields.py` — `compare_p1e1`'in bildirimi
- **Ne:** `in_code`'un döngüsüne `("compare_p1e1", "P1E1")` (zaman/bilgi tabloları ve `EXCLUDED`), dosya sabitlerine `compare_p1e1: [P1E1_FILES, P1E1_BYTES]`, `check_runs`'ın bayt listesi dışlamasına `P1E1_BYTES`. Denetim kuralları, bildirim çözümleyici, diğer modüller aynen (+7/−2 satır, biçimleyici satır kırması dahil).
- **Neden:** denetleyici `compare_*.py`'yi kendiliğinden bulur ama her modülün sabitlerini adıyla okur; yeni karşılaştırıcı bildirilmeden eşlenemez (K-E0.2: her karşılaştırıcının her hariç tutması bildirilir).
- **Önce/sonra:** `compare_p1e1` varken eski denetleyici + eski bildirim: `unmatched` 1 (`modules_without_section: [compare_p1e1]`); eski denetleyici + yeni bildirim: 3 (bildirilen satır ve iki dosya sabiti kodda görünmüyor); yeni denetleyici + yeni bildirim: **0** (86 alan, 18 dosya sabiti). Önceki modüllerin satırları ve sabitleri iki tarafta aynı.
- **Gözden geçirici onayı:** E1.5.review.md.

## F6 · E1.5 · `research/eval/DETERMINISTIC_FIELDS.md` — `compare_p1e1` bölümü ve satırları
- **Ne:** `### compare_p1e1` bölümü; alan tablosuna bir `ortam` satırı (`p1e1/release/release.json` `targets/*/sha256`: yayım ikilisinin hash'i klon dizinine bağlı olabilir); dosya tablosuna `compare_p1e1.P1E1_FILES`, `P1E1_BYTES`. Var olan satırlar değişmedi (yalnız ekleme). `regress` (`karar`) satırları E1.5-pre'de geliştirme RUN'ından yazılır.
- **Neden:** K-E0.2 — her karşılaştırıcının karşılaştırmadığı her alan sınıfı ve gerekçesiyle bildirilir.
- **Önce/sonra:** F5'in ölçümü; `check_compare_fields` `unmatched` 0.
- **Gözden geçirici onayı:** E1.5.review.md.

## F7 · E1.5 · `research/eval/historical.py` — boşalan kap yalnız bildirilen içerikle
- **Ne:** `differing`: bir tarafta boş kap (`{}`/`[]`, E1.4 tur 2'den beri yaprak), öbür tarafta aynı yolun altında yapraklar varsa ve **hepsi** bildirilen değişikliklerin altındaysa, kabın kendisi fark sayılmaz; içinde bildirilmemiş tek bir yaprak bile varsa fark kalır (+13 satır). Test `test_emptied_container_only_with_declared_content` (referansta boş, RUN'da dolu kap: yalnız bildirilen içerik kabul, fazladan bir yaprak sayılır).
- **Neden:** geliştirme RUN'ında (`20261017T132817Z-b77f291`) D1-12 ve E-9'un referanstaki `differing_fields: {}`'i, bu RUN'da yalnız K-E1.2'nin bildirilen yapraklarını taşıyordu; boş kap kuralı (E1.4 gözden geçirmesi bulgu 5) bunu yeni FAIL sayıyordu. Düzeltme kuralı daraltmaz: bildirilmemiş içerik yine sayılır.
- **Önce/sonra:** P1-E resmi RUN'ı: önce ve sonra 8 tarihsel, sayılan 0 (aynı satırlar). Geliştirme RUN'ı (yeni `EXCLUDED` ile): önce sayılan 2 (D1-12, E-9: `…/differing_fields`), sonra 0, tarihsel 10. pytest 12/12.
- **Gözden geçirici onayı:** E1.5.review.md.

## F8 · E1.5 · `research/eval/DETERMINISTIC_FIELDS.md` — `compare_p1e1` `regress` satırları
- **Ne:** alan tablosuna 29 `karar` satırı (`compare_p1e1.EXCLUDED`'ın birebir kopyası). Var olan satırlar değişmedi.
- **Neden:** K-E0.2; E1-6'nın hariçleri geliştirme RUN'ından resmi RUN'dan önce yazıldı (E1.5.md).
- **Önce/sonra:** `check_compare_fields` `unmatched` 0 (115 alan).
- **Gözden geçirici onayı:** E1.5.review.md.
