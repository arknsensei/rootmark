# P2-R — Görevler

Genel kurallar:
- Sıra: R0 → R1 → R2 → R3 → R4 → R5.
- Progress `docs/rounds/P2-R/progress/`. Commit öneki `P2-R/R<n>:`.
- Dal `p2-r/round`, yalnız birleşmiş `main`'den (K-P2.2). `ROUND_START` = P2-B kapanış commit'inin birleşmiş `main` HEAD'i.
- Geliştirme turu dozu (K-P2.4): gözden geçirici yalnız R5'te, tek tur. Güvenlik incelemesi R1 (kuralın yanlış soyu ya da `canonical`'ı budaması, `path_prefix` ile vault dışı adresleme) ve R2 (arşiv yolu ve `base` kurcalaması, `at_seq` tutarlılığı, HTTP `prune` yetkisi, `dry_run` yan etkisi) için R5'in gözden geçiricisinde ayrı başlık.
- `make verify FAST=1` görev sonlarında; tam `make verify` R5'te bir kez.
- Dondurulmuş kodda her değişiklik `FREEZE_EXCEPTIONS.md`'ye önce/sonra kanıtıyla (DEV_PROCESS §4); `research/eval/p2b_align.py`, `research/eval/datasets/d_notes.py` ve D-NOTES manifesti bu turda **hiç** değişmez.

## R0 — Açılış, P2-B kalıntıları
**Yapar:**
- `ROUND_START`; DECISIONS'ta tur açılışı (K-P2R.2: K-P2B-T1'in kesinleşmiş kararı buraya yazılır; GOAL R0 (a)'nın uygulanıp uygulanmayacağı bu kayıtla belli olur).
- GOAL R0 (a) [yalnız K-P2B-T1 (a)]: `core/src/parse.rs` konum hesabı; `core`/`server` testleri (kimlikler, olaylar, `normalized_text` aynen; `p2_b_b1` geçer); D-NOTES yeniden ölçümü **değiştirilmemiş** ölçücüyle, iki koşu, `reports/rounds/P2-R/r0/align.json`; `notes_by_class.json`.
- GOAL R0 (b): `scripts/contract_check.sh`, `scripts/mcp_e2e.sh` istisna prosedürüyle 11 araca; listelerden K-P2B.4 girdileri; `eval.historical_p2b` aynı geliştirme RUN'ına karşı önce/sonra.
- GOAL R0 (c): (5) `p2b-checks` hizalama denetimi (referans değer dosyada; gerileme → çıkış ≠ 0); (8) `search` iz sırası; (9) `events` okuma yolu + test (büyük günlükte `since_seq` sonu, kilit altında süre kaydı); (11) `commit_plan`.

**Kabul:** R-1 (ve R-3 ölçüldüyse değeri raporlanır); mevcut testler aynen; `make conformance` 39/39, C3-06 aynen; `check_frozen`, `check_deps`; `make verify FAST=1`.

## R1 — Proposal sönümleme
**Yapar:**
- `core/src/policy.rs`: `[[prune.rules]]` ve `[prune] version` ayrıştırma ve doğrulama (sınırlar, NFC/NFD, `layer` kümesi, `canonical` ret); `policy lint`.
- `server`: eşleyici (`rules.rs` deseni) yalnız `prune` adımında; soy başına karar: son olay `seq`'i, sonraki checkpoint sayısı, katman, etkinlik, önek. Geçiş `Vault::prune_inactive` ile; `meta` alanları GOAL R1.
- `core` okuma kuralı (replay): `policy` aktörlü `meta.prune` `Demoted`'da `reason = rule:<actor.id>`; hata yolu testi.
- Testler: kurallı/kuralsız tarif, `events_hash`, `canonical` asla, `unverified` yalnız kuralla, budanmış soy `search`/`read_block`/`history`/`at`.

**Kabul:** R-2, R-4 (budama henüz CLI'den; HTTP R2'de).

## R2 — Saklama, arşiv, `prune` komutu
**Yapar:**
- `core`: `[retention] keep_checkpoints`; sınır hesabı (`base`); iz arşivi (taşıma, `base` kaydı, mutlak `at_seq`, `trace_id` aynen); checkpoint arşivi (adsız + başvurulmayan + sınırın gerisi); konum kaydı silme kuralı; `journal prune` çağrısı; hepsi tek `prune` planı + uygulaması; `dry_run`.
- `server`: CLI `rootmark prune [--dry-run] [--now-ms]`, `rootmark trace --archived`; HTTP `POST /v1/prune` (`human_id`, 403; `X-Consumer`; kuyruk); `trace` yanıtı `archived_before`; `checkpoints[].archived`; `at` arşivli → `archived` kodu; `status.retention`.
- Testler: birleştirme eşitliği (canlı + arşiv = eski dosya), `at_seq`/`trace_id` sabitliği ve yeni iz ekleme, adlı checkpoint, canlı izin checkpoint'i, `dry_run` hash'ler aynen, `serve` uzun oturumda budamaz, MCP listesinde yok, HTTP yetki, tablosuz tarif no-op mesajı, `base` kurcalaması (bozuk `base` → açık hata).
- Spec §8 bilgilendirici not (CRITERION, 2.0.4; `SPEC_EDITS.md`).

**Kabul:** R-5, R-8; R-4'ün HTTP yolu; `make verify FAST=1`.

## R3 — Hacim ölçümü D-VOL
**Yapar:**
- Ön-kayıt commit'i **ölçümden önce**: `research/eval/p2r_volume.py` (senaryo, tohum, tarif, `V` tanımı, iki evre), `research/eval/datasets/d_vol.py` (sentetik belge üreteci).
- Ölçüm: gerçek sunucuya HTTP ile (propose/search/read_block/checkpoint/promote), `prune` CLI ile; `p2r/volume.json` (`V_before`, `V_after`, `V_after2`, bileşen başına baytlar, budanan soy, taşınan satır/nesne, birleştirme eşitliği); iki koşu bayt eşit.
- Kalırsa optimize etme; raporla; karar kuralı.

**Kabul:** R-7 (değerler raporlanır; kalırsa karar kuralı); R-6'nın FAST kaydı.

## R4 — Sözleşme, belgeler, güvenlik
**Yapar:**
- CLIENT_CONTRACT §2.10, `status.retention`, `checkpoints[].archived`, §3 `archived` kodu, §2.9 şema; `scripts/gen_client_types.sh`; iki istemcinin denetleyicisi (`contract.gen.ts` yeniden üretilir; istemcilere özellik yok); P2-B'nin istemci kayıt koşusu (`p2b/client_checks.json`) yeniden koşar.
- THREAT_MODEL, HEADLESS_CHANNEL (`prune` akışı), ARCHITECTURE (`on_prune`), CHANGELOG (Unreleased).

**Kabul:** R-9.

## R5 — Zincir, rapor, hazır
**Yapar:**
- `make p2r-checks` (kriter testleri → `p2r_checks.json`; D-VOL → `p2r/volume.json`; `p2r/det.json`: `prune` çıktıları, canlı ve arşiv dosya hash'leri); `verify.sh` P2-R adımı, `compare_p2r`, `historical_p2r` (dondurma istisnası, önce/sonra); `ROUND_CHECKS` += `p2r-checks`.
- Tam `make verify` bir kez.
- Gözden geçirici: tek tur, bütün kabul ölçütleri, güvenlik başlığı (R1, R2).
- FINAL: kriter tablosu, karar kuralı dalı, Blok 2 sonu doğrulayıcısı için P2-R özeti (P2-B özetine ek), damga görüşü yok (spec damgası doğrulamadan sonra, DEV_PROCESS §5).
- READY; tek PR (`gh pr create --base main --head p2-r/round --title "P2-R: saklama ve budama" --body-file reports/rounds/P2-R/FINAL.md`).

**Kabul:** R-6 (tam çift), R-10; izlenebilirlik; karar kuralı.
