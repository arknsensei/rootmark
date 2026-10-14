# P1-B — Yönetişim Primitifleri

## Amaç
`spec/SPEC.md` §7 (katman ve terfi), §8 (kontrol noktası ve tüketim izi), §11 (ham günlük ve sürüm tetikleyici) taslak bölümlerini `core/` içinde uygulamak, C2-01..C2-08 uyumluluk testlerine bağlamak ve P1-A doğrulamasının üç önerisini kapatmak. Arama, MCP, istemci yok.

## Kapsam
### Çekirdek (`core/`)
- **Katman:** `layer ∈ {canonical, proposal, derived}`; ingest'te `author_kind`'a göre varsayılan (spec §7). Her katman geçişi olay: `LayerChanged`, `Promoted`, `Demoted`. Replay katman durumunu kurar.
- **Terfi hook'u:** `trait PromotionPolicy { fn decide(&self, req: &PromotionRequest) -> Decision }`. Çekirdek hook olmadan terfi etmez (C2-03). Dahili politikalar: `Deny` (varsayılan), `HumanOnly` (actor insan ise izin; K12). `Promoted` olayı `actor`, `reason`, `policy_version` zorunlu (C2-02). Budama: `prune_inactive(lineage, reason)` nesne silmez, soyu pasif işaretler (C2-04).
- **Kontrol noktası:** `Checkpoint { checkpoint_id, name?, event_seq, docs }`, spec §3 formülü; yazılınca değişmez; `snapshot_at(doc, checkpoint)`, `block_at(lineage, checkpoint)`; sonraki olaylar görünmez (C2-05, C2-06).
- **Tüketim izi:** `.mdkernel/traces.jsonl`; `record_trace(checkpoint, consumer, query_hash, returned)`; `trace_id` formülü; `read_blocks` türü her okuma API'si iz yazar (C2-07). P1-C'de arama da aynı API'yi kullanacak.
- **Ham günlük ve sürüm tetikleyici:** `.mdkernel/journal/<doc_hash>.jsonl` her kayıt için `(seq, content_hash, bytes_len)`; `VersionTrigger` trait + dahili `IdleOrDistance { idle_ms, min_sim_change }`; çekirdek sınırları: `min_journal_retention ≥ 24h`, sürümsüz içerik indekslenmez, sürüm asla günlükten az bilgi taşımaz. Sınır dışı parametre → `Err`.
- **Politika tarifi:** `.mdkernel/policy.toml` yükleyici; alanlar: `promotion = "deny"|"human_only"`, `version_trigger.idle_ms`, `version_trigger.min_sim_change`, `journal.retention_hours`. Sınır ihlali → yükleme hatası, varsayılana sessiz düşme yok.
- **CLI (`core/src/bin/mdk.rs`, P1-D'de taşınacak):** `propose <doc> <file>` (ajan yazımı, `proposal`), `promote <lineage> --actor --reason`, `demote`, `checkpoint create [--name] | list | show`, `history <lineage>`, `blame <doc> [--at <checkpoint>]`, `trace list [--since]`, `journal append <doc>`. JSON çıktı.

### Spec ve uyumluluk
- `spec/fixtures/C2-xx/` (8 fixture), `core/tests/conformance/` C2 değerlendiricisi, `make conformance` → `level_claimed 2`.
- Spec §7, §8, §11 metni uygulamayla hizalanırken **yalnız açıklık** için değişebilir; anlam değişikliği karar kaydı ister. Tur sonunda bu bölümler "Draft → Validated" damgası alır.

### P1-A önerileri (bu turda kapatılır)
- **Ö1:** `research/FROZEN.md` kendi listesine girer; `check_frozen.sh` tur başlangıç commit'inden itibaren tablo satırlarının silinmediğini/gevşetilmediğini git geçmişinden denetler.
- **Ö2:** Uyumluluk testi fixture kökünü çalışma anında çözer (`MDK_FIXTURES` ya da `$(CURDIR)/spec/fixtures`); `reports/conformance.json` mutlak yol ve manifest hash'i içerir; taşınmış `target/` ile kasıtlı test hata vermelidir.
- **Ö3:** Bölme-ağırlıklı ek holdout dilimi `d1hs` (SEED `20261013:holdout-split`, ≥ 60 doküman, split_edit ≥ %25, split_three ≥ %10 gerçekleşen oran, uygunluk: ≥ 3 cümleli paragraf); mevcut `d1ho` değişmez.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| B-1 | C2-01..C2-08 | hepsi geçer; `level_claimed 2` |
| B-2 | C1-01..C1-17 | hâlâ 17/17 |
| B-3 | Regresyon: v3-final 9 dosya + P1-A holdout/split_grid, kalite alanları | fark 0 |
| B-4 | `make verify` çift koşturma, kalite alanları + `traces.jsonl` ve `checkpoints/` içerikleri | fark 0, bayt eşit |
| B-5 | v3-final ile üretilmiş vault (`layer`'sız olaylar) mevcut çekirdekle açılır, replay `snapshot_id` eşit, `events_hash` değişmez | eşit |
| B-6 | Politika sınır ihlali (6 vaka: retention < 24h, idle_ms < 0, min_sim_change ∉ [0,1], bilinmeyen promotion, hook'suz terfi, actor'sız Promoted) | hepsi `Err`, hiçbiri sessiz varsayılan |
| B-7 | Ö2 kasıtlı test: taşınmış `target/` + bozuk fixture | `make conformance` hata |
| B-8 | Ö1 kasıtlı test: FROZEN.md'den satır silme | `check_frozen.sh` hata |
| B-9 | `d1hs` H1-H1 / H1-H2 / H1-H3 | ≥ 0.90 / ≤ 0.03 / ≥ 0.70 |
| B-10 | Katman geçişleri replay ile kurulur: 1000 rastgele (tohumlu) propose/promote/demote dizisi sonrası `layer` durumu = replay durumu | eşit |

## Karar kuralı
B-1..B-8 ∧ B-10 geçti → spec §7, §8, §11 "Validated"; P1-C başlar. B-9 kaldı → eşleyici karar talebi `DECISIONS.md`'ye; P1-C beklemez. B-5 kaldı → geriye uyumluluk kırılmış; P1-C başlamaz. B-3/B-4 kaldı → regresyon; P1-C başlamaz.

## Yapılmayacaklar
tantivy, vektör, ONNX (P1-C); MCP/HTTP (P1-D); istemci; örtük onay politikası (faz 2); WASM; H2; API çağrısı; eşleyici sabitlerinde değişiklik.
