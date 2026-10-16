# P1-D.1 — Yönetişim ve Sözleşme Kapanışı

Kısa tur. P1-D doğrulamasının iki koşulunu (Ş1, Ş2), üç önerisini ve P1-C.1'in kalan önerilerini kapatır. Bitmeden P1-E başlamaz.

## Kararlar (D1.0'da DECISIONS.md'ye; K-D4-T kapanır)

- **K-D1.1 Ajan yazımı canonical içeriği değiştiremez (varsayılan).** `propose`'un `create` ve `append` dışındaki her biçimi (`replace_section`, ileride `edit`) bir `EditPolicy` hook'undan geçer. Dahili politikalar: `AppendOnly` (varsayılan; canonical bloğa dokunan her istek `denied`), `Unrestricted` (yalnız açıkça `policy.toml`'da). `replace_section` yalnız hedef bölümdeki **tüm** bloklar `proposal` ise (ajanın kendi önerisini düzeltmesi) `AppendOnly` altında kabul edilir. Gerekçe: dosya tek gerçek kaynaktır; canonical'ın dosyadan silinmesi geri alınabilir ama görünürdür, bu yüzden izin yönetişim kararıdır, uygulama kararı değil.
- **K-D1.2 Silme ve değiştirme olayları aktör taşır.** `BlockDeleted`, `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged` olaylarının `meta`'sına `actor {kind, id}` ve `source` eklenir (ingest'i tetikleyen kim). Mevcut vault'lar açılır (alan opsiyonel okunur, zorunlu yazılır). Spec §4.2 bilgilendirici not + §4.3 "provenance of events" cümlesi; sürüm CRITERION kuralıyla.
- **K-D1.3 Kanal aktör türünü belirler.** MCP (stdio) = ajan kanalı; aktör daima `agent`, istemci adı `id`. HTTP (token) = insan kanalı; token sahibi insandır, aktör `human`, `id` `policy.toml [identity] human_id` (zorunlu; yoksa HTTP yazma uçları 403). HTTP'den `promote` `HumanOnly` ile **izinli**, MCP'den **denied** kalır. `X-Consumer` yalnız iz etiketi, aktör türünü değiştiremez. Tehdit modeli notu (`docs/SECURITY.md`): token dosyasını okuyabilen yerel süreç insan sayılır; OS anahtar zinciri faz 2.
- **K-D1.4 Vault başına tek yazar kuyruğu.** Yazan ve iz yazan tüm çağrılar (MCP ve HTTP birlikte) vault başına tek FIFO kuyruğunda, geliş sırasıyla seri işlenir; salt okunur çağrılar (`status`, `history`, `blame`, `checkpoints`, `trace`) paralel. Boru hatlı istemci aynı istek dizisiyle aynı `events.jsonl`/`traces.jsonl` üretir.
- **K-D1.5 Araç hataları kararlı kod taşır.** `structuredContent.error.code ∈ {invalid_arguments, empty_query, path_outside_vault, exists, not_found, too_large, heading_not_found, denied, policy_violation, internal}`; HTTP eşlemesi 400/401/403/404/409/413/422/500. Sözleşme `docs/CLIENT_CONTRACT.md` (P1-E istemcisinin tek bağımlılığı).
- **K-D1.6 Adsız checkpoint adlandırılabilir.** `checkpoint {name}` aynı durumda var olan adsız checkpoint'e ad ekler (`CheckpointNamed` olayı); kimlik değişmez.
- **K-D1.7 Sözcüksel bildirim tamdır.** Manifest `lexical.dl = "exact token count, f64, no fieldnorm quantization"` alanı; spec §9 bilgilendirici not. (P1-C.1 Ö1)
- **K-D1.8 `include_proposal` yanıt uzunluğu normatif.** Katman başına ≤ k, toplam ≤ 2k; spec §9. (P1-C.1 Ö3)
- **K-D1.9 Süreç.** DEV_PROCESS §1'e: GOAL'ler spec sürüm numarası öngörmez, yalnız "CRITERION kuralıyla" der; her karar kuralına varsayılan dal eklenir: "tanımlanmamış kalış → tur kaldı, karar talebi". (P1-C.1 Ö2, P1-D Ö3)

## Çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| D1-1 | `AppendOnly` altında canonical bloğa dokunan 6 `replace_section` vakası (tam bölüm, kısmi, başlık, karışık canonical+proposal, iç içe başlık, insan dokümanının tamamı) | 6/6 `denied`, dosya ve `events.jsonl` değişmez |
| D1-2 | Yalnız proposal bloklardan oluşan bölüme `replace_section` | kabul, bloklar `proposal` |
| D1-3 | **Değişmezlik:** 500 rastgele ajan yazımı (create/append/replace) sonrası varsayılan `search`'ün canonical sonuçları (200 sorgu × 3 mod) yazım öncesiyle | bayt eşit |
| D1-4 | `Unrestricted` ile silme → `BlockDeleted.meta.actor` = ajan kimliği; `history` silen aktörü gösterir | 1/1 |
| D1-5 | Boru hatlı istemci: aynı 100 çağrı (40 propose dahil) iki özdeş vault'a yanıt beklemeden → `events.jsonl`, `traces.jsonl`, md dosyaları | bayt eşit |
| D1-6 | Boru hatlı 100 salt okunur çağrı → tek yazar kuyruğu nedeniyle sıralanmıyor (paralel), yanıtlar doğru | 100/100, süre < seri sürenin %60'ı (bilgi) |
| D1-7 | HTTP `promote` (token, `human_id` ayarlı) → `Promoted` actor `human:<id>`; MCP `promote` → `denied`; `human_id` yokken HTTP yazma → 403 | 3/3 |
| D1-8 | Hata kodları: 12 kötü girdi vakası (P1-D V2/V3 listesi) MCP ve HTTP'de beklenen kod | 24/24 |
| D1-9 | Adsız checkpoint'i adlandırma; ad çakışması → `exists` | 2/2 |
| D1-10 | Uyumluluk 31/31 + yeni Seviye 2/3 vakaları: C2-10 (olay aktörü), C3-06 (canonical değişmezliği), C3-07 (yanıt uzunluğu) | 34/34, spec sürümü CRITERION kuralıyla |
| D1-11 | Eski vault'lar (P1-D sahne vault'u, v3-final karışık vault'lar) açılır, replay eşit, `events_hash` değişmez | eşit |
| D1-12 | Çift koşturma (boru hatlı vakalar dahil); P1-D kalite alanları (karar gereği hariçler) | fark 0 |
| D1-13 | `docs/CLIENT_CONTRACT.md` ve `docs/SECURITY.md` var; sözleşmedeki her araç/kod bir testle eşleşiyor (`scripts/contract_check.sh`) | eksik 0 |

## Karar kuralı
D1-1..D1-13 geçti → K-D4-T kapanır, P1-E başlar. D1-1/D1-3/D1-4 kaldı → yönetişim; P1-E başlamaz. D1-5 kaldı → determinizm; P1-E başlamaz. **Tanımlanmamış kalış → tur kaldı, karar talebi** (K-D1.9).

## Görevler
- **D1.0** `git tag p1d-final`; `ROUND_START`; DECISIONS ekleri (K-D1.1..9, K-D4-T kapanış); DEV_PROCESS §1 ve CRITERION güncellemesi (K-D1.9); `docs/SECURITY.md` taslağı.
- **D1.1** `core`: olay `meta.actor/source` (K-D1.2); `EditPolicy` trait, `AppendOnly`, `Unrestricted`; `CheckpointNamed`; eski vault uyumu.
- **D1.2** `server`: kanal→aktör (K-D1.3), `[identity] human_id`; tek yazar kuyruğu (K-D1.4); hata kodları (K-D1.5); `checkpoint` adlandırma; `propose` `EditPolicy` çağrısı.
- **D1.3** `index`: `lexical.dl` alanı; yanıt uzunluğu kuralı; C3-06/07 fixture'ları.
- **D1.4** Spec: §4.2/4.3, §7 (EditPolicy, bilgilendirici), §9 (K-D1.7/8); sürüm CRITERION ile; C2-10 fixture; `SPEC_EDITS.md`.
- **D1.5** `docs/CLIENT_CONTRACT.md` (araçlar, HTTP uçları, kodlar, kanal/aktör kuralı, `include_proposal` anlamı ve uzunluğu, checkpoint akışı), `scripts/contract_check.sh`.
- **D1.6** Testler: D1-1..D1-9 betikleri (`scripts/test_p1d1.sh`, boru hatlı istemci `scripts/mcp_pipeline_client.py` ya da Rust test), eski vault uyumu.
- **D1.7** `verify.sh` genişletme, `report_p1d1.py`, `FINAL.md`, `READY.md`.

## Yapılmayacaklar
İstemci kodu; örtük onay; OS anahtar zinciri; çok vault; uzak erişim; `Unrestricted`'ın varsayılan olması.
