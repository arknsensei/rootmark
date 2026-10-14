# P1-B.1 — Geriye Uyumluluk Yaması

Kısa tur. P1-B doğrulamasının F1 farkını ve üç önerisini kapatır. Kapanmadan P1-C başlamaz (P1-B karar kuralı, B-5 dalı).

## Kararlar (DECISIONS.md'ye bu turun açılışında eklenir)
- **K-B1.1 Katman'sız günlüklerde türetim.** `layer` bilgisi taşımayan (v3-final biçimli) günlüklerde replay, spec §7 varsayılan kuralını ve "katman yalnız düşer" kuralını (K17-T) **olay beklemeden** uygular: Human/canonical soyu Agent veya Transcript düzenlediğinde `proposal`'a düşer, türetilmiş olarak işaretlenir (`layer_source = derived`). `events.jsonl` değişmez; migration yazılmaz. Yeni günlüklerde (`layer` taşıyan) kural değişmez: geçiş olayı zorunludur ve eksikse `Err`.
- **K-B1.2 `trace_id` formülü ayırıcılı.** `trace_id = H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" + at_seq)`; `query_hash` 32 hex zorunlu, `consumer` `\n` içeremez. Spec §8 Draft olduğu için spec sürümü **1.0.1**; C2-07 beklenen değeri yeniden üretilir ve `MANIFEST.json` güncellenir (karar kaydıyla). C1 fixture'ları değişmez.
- **K-B1.3 "Yalnız açıklık" tanımı.** Bir spec düzenlemesi yalnız açıklık sayılır ⇔ (a) hesaplanan hiçbir değeri değiştirmez **ve** (b) daha önce uyumlu bir uygulamayı uyumsuz kılmaz. Bu tanımla E1 (küçük harf katman adları) ve E2 (`returned` nesne biçimi) açıklık değil, **serileştirme kararı**dır: K15-T kapanır, ikisi normatif kabul edilir ve 1.0.1'e yazılır.
- **K-B1.4 Undeclared `synthetic` → `canonical`.** Spec §7 "as declared"e normatif ek: bildirilmemişse `canonical`.

## Çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| B1-1 | v3-final **kütüphane API'siyle** üretilmiş 8 senaryo (yalnız Agent; yalnız Transcript; yalnız Synthetic; Human→Synthetic; Agent→Human; Human→Agent; Human→Transcript; Synthetic→Agent; + karışık 4 dokümanlı vault) mevcut çekirdekle açılır: replay `snapshot_id` eşit, `events_hash` değişmez, `blame` türetilmiş katmanı doğru verir | 9/9 |
| B1-2 | Yeni biçimli günlükte eksik `LayerChanged` | `Err` (davranış korunur) |
| B1-3 | C1 17/17, C2 8/8 (C2-07 yeni beklenen değerle), `level_claimed 2` | geçer |
| B1-4 | `trace_id` doğrulaması: `query_hash` 31 hex / 33 hex / hex olmayan; `consumer` içinde `\n` | 4/4 `Err` |
| B1-5 | Kayma testi: (`"ab"`, qh) ve (`"a"`, `"b"+qh`) farklı `trace_id` | farklı (ikincisi zaten `Err`) |
| B1-6 | Regresyon: P1-B RUN'ı ile kalite alanları (C2-07 ve `traces.jsonl` hariç, onlar yeni beklenen) | fark 0 |
| B1-7 | Çift koşturma | fark 0, bayt eşit |

## Görevler
- **B1.0** `git tag p1b-final`; `ROUND_START`; DECISIONS ekleri; `SPEC_EDITS_CRITERION.md` (K-B1.3 metni) `docs/`'a; `spec/SPEC.md` 1.0.1: §7 "as declared → canonical if undeclared", §8 formül, §7/§8 serileştirme (küçük harf katman; `returned` nesne).
- **B1.1** `core`: replay'de türetim (K-B1.1); `layer_source ∈ {event, derived}`; `blame` çıktısında görünür; `Err` davranışı yeni biçimde korunur. Testler: 9 senaryo `core/tests/compat_v3final.rs` (v3-final çekirdeği `git archive` ile derlenip fixture vault'ları **bu görevde üretilir ve `core/tests/fixtures/v3_final_mixed/` altına commit'lenir**, üretim betiği `scripts/gen_v3final_mixed.sh`).
- **B1.2** `core`: `trace_id` formülü ve doğrulamalar; C2-07 beklenen değeri yeniden üretim (`gen_fixtures_c2.sh`), `MANIFEST.json`; `conformance.json` `spec_version 1.0.1`.
- **B1.3** `scripts/check_v3_compat.sh` genişletme — dondurulmuş; **F-istisnası** ile (FREEZE_EXCEPTIONS girdisi, önce/sonra: eski senaryoda çıktı bit eşit, gözden geçirici onayı).
- **B1.4** `make verify` + rapor (`reports/rounds/P1-B.1/FINAL.md`) + `READY.md`.

## Doğrulama (ek maddeler; DEV_PROCESS §5 temel)
- V1 çift koşturma; V2 B1-1'i **kendi derlediğin** v3-final ile üret (fixture'lara güvenme), 9/9; V3 B1-2 kasıtlı: yeni günlükten bir `LayerChanged` satırı sil → `Err`; V4 `trace_id` formülünü bağımsız hesapla (4 iz), B1-4/5 vakaları; V5 C2-07 eski beklenen ile `cmp` farklı, yeni ile aynı; MANIFEST'te yalnız C2-07 satırı değişmiş; V6 F-istisnası üç şart; V7 K-B1.3 ile SPEC_EDITS yeniden sınıflandırması tutarlı mı; V8 karar kuralı: B1-1..B1-7 geçti → P1-C başlar.

## İstemler
**Ana:** `Aktif tur P1-B.1 (docs/rounds/P1-B.1/GOAL.md; GOAL, görevler ve doğrulama aynı dosyada). CLAUDE.md, NORTH_STAR.md, spec/SPEC.md §7–§8, docs/DEV_PROCESS.md, reports/rounds/P1-B/VERIFICATION.md (F1, Ş1–Ş3) oku. B1.0'dan B1.4'e sırayla; DEV_PROCESS §2 disiplini; commit öneki P1-B.1/B1.<n>. Kırmızı çizgiler: events.jsonl asla değişmez; spec §1–6 ve C1 fixture'ları değişmez; yalnız C2-07 beklenen değeri yeniden üretilir; dondurulmuş betik değişikliği yalnız F-istisnasıyla; v3-final ikilisini kendin derle, fixture'ları ondan üret ve commit'le. Bitince READY.md, dur.`
**Doğrulama:** `Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 ve docs/rounds/P1-B.1/GOAL.md "Doğrulama" bölümünü uygula; v3-final'i kendin derle. Kod değiştirme. Tek çıktı reports/rounds/P1-B.1/VERIFICATION.md.`
