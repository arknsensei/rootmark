# P1-D.1 — FINAL

- RUN: `20261016T124520Z-1527365`; commit `15273654438c3eab7490e1e79078cedde151bdb2`
- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| D1-1 | `append_only` altında canonical bloğa dokunan 6 `replace_section` (tam, kısmi, başlık, karışık, iç içe, insan dokümanının tamamı) | 6/6 `denied`; dosya ve `events.jsonl` değişmedi: True | 6/6 `denied`, dosya ve `events.jsonl` değişmez | geçti | `p1d1/d1_1.json` |
| D1-2 | yalnız proposal bloklardan oluşan bölüme `replace_section` | kabul True, bloklar proposal, proposal | kabul, bloklar `proposal` | geçti | `p1d1/d1_2.json` |
| D1-3 | 500 ajan yazımı (create/append/replace) sonrası varsayılan `search` canonical sonuçları (200 sorgu × 3 kip) | 451/500 yazım kabul (append:accepted 156, append:denied 13, create:accepted 158, replace_section:accepted 137, replace_section:denied 36); 600 karşılaştırmada fark 0 | bayt eşit | geçti | `p1d1/d1_3.json` |
| D1-4 | `unrestricted` ile silme → `BlockDeleted.meta.actor`; `history` silen aktörü gösterir | olay {'id': 'criteria-agent/1.0', 'kind': 'agent'}, history {'id': 'criteria-agent/1.0', 'kind': 'agent'} | 1/1 | geçti | `p1d1/d1_4.json` |
| D1-5 | boru hatlı 100 çağrı (40 `propose`) iki özdeş vault'a | A↔B {'events.jsonl': True, 'md_files': True, 'proposals.jsonl': True, 'traces.jsonl': True}; kilit adımlı ile {'events.jsonl': True, 'md_files': True, 'proposals.jsonl': True, 'traces.jsonl': True} | `events.jsonl`, `traces.jsonl`, md dosyaları bayt eşit | geçti | `p1d1/d1_5.json` |
| D1-6 | boru hatlı 100 salt okunur çağrı | 100/100 doğru; süre oranı 0.27 (< 0.6: True, bilgi) | 100/100; süre < seri %60 (bilgi) | geçti | `p1d1/d1_6.json` |
| D1-7 | HTTP `promote` (`human_id`) → `human:<id>`; MCP `promote` → `denied`; `human_id`'siz HTTP yazma → 403 | 3/3 (MCP denied, HTTP'siz kimlik 403, aktör {'id': 'editor-ada', 'kind': 'human'}) | 3/3 | geçti | `p1d1/d1_7.json` |
| D1-8 | 12 kötü girdi (P1-D V2/V3) × MCP ve HTTP → beklenen kod | 24/24 | 24/24 | geçti | `p1d1/d1_8.json` |
| D1-9 | adsız checkpoint'i adlandırma; ad çakışması → `exists` | 2/2 | 2/2 | geçti | `p1d1/d1_9.json` |
| D1-10 | uyumluluk 31/31 + C2-10, C3-06, C3-07 | 34/34, level 3, spec 1.0.3 (CRITERION: 1.0.3); yeni: {'C2-10': 'pass', 'C3-06': 'pass', 'C3-07': 'pass'} | 34/34, spec sürümü CRITERION kuralıyla | geçti | `conformance_p1d1.json` |
| D1-11 | eski vault'lar (p1d-final sahne ve e2e vault'ları, v3-final karışık) açılır, replay eşit, `events_hash` değişmez | e2e: 15/15 replay, hash eşit True; scene: 13/13 replay, hash eşit True; v3-final True | eşit | geçti | `p1d1/d1_11.json` |
| D1-12 | çift koşturma (boru hatlı vakalar dahil); P1-D kalite alanları (karar gereği hariçler) | pair 100875 alan, fark 0, baytlar eşit True; P1-D'ye göre 99881 alan, hariç dışı fark 0, hariç tutulan fark 29 alt ağaç | fark 0 | geçti | `pair_compare.json, regress_vs_p1d.json` |
| D1-13 | `docs/CLIENT_CONTRACT.md`, `docs/SECURITY.md`; her araç/kod bir testle eşleşir (`scripts/contract_check.sh`) | 10 araç, 10 kod, eksik 0 | eksik 0 | geçti | `p1d1/d1_13.json` |

Kasıtlı bozmalar (`p1d1/tamper_p1d1.json`): 9/9 reddedildi, kontroller 5/5 → geçti.

## Karar kuralı

Kalan kriterler: yok.

D1-1..D1-13 geçti → **K-D4-T kapanır, P1-E başlar.**
