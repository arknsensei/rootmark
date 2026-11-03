# P2-Q — Türkçe arama sorgularının ek ön-kaydı (K-P2Q.3)

İnsan kararı (2026-11-03, Q2'den önce): "Türkçe arama sınanmıyor." `scenario.json` ve `SCENARIO.md` aynen kalır (Q-1); bu ek ön-kayıt `research/p2q/scenario_tr.json`'dadır (üreticisi `research/p2q/make_scenario_tr.py`, `--check` bayt bayt yeniden üretir), Q2'nin ilk ölçümünden önce commit'lenir ve sonra değişmez. Çelişkide `scenario_tr.json` kazanır.

**Korpuslar:** Obsidian ve qorbit kopyaları (tarif `tr`: `uni_fold_tr` + çok dilli gömme). **Geçiş** ana sahnelerden ayrı: taze kopya → `ingest-dir` → tarif `tr` → `index build` → yalnız HTTP `serve` → sorgular (varsayılan arama; sözcüksel `k` = 1000, hibrit `k` = 10) → `SIGTERM`. Koşucu `research/p2q/tr_queries.py`.

**Sorgular (korpus başına 36; sözcükler genel Türkçe sözlükten, korpustan değil):**
| Grup | Sayı | Örnek | Ölçüt |
|---|---|---|---|
| katlama (küçük, BÜYÜK, Baş harf) | 18 | `istanbul` / `İSTANBUL` / `İstanbul`; `ışık` / `IŞIK` / `Işık`; `şirket`, `değişiklik`, `güncelleme`, `çalışma` | TR-1, TR-2 |
| ASCII `I` tuzağı | 2 | `ISTANBUL` (→ `ıstanbul`), `ILISKI` | TR-2 |
| aksansız (sözcüksel) | 6 | `sirket`, `degisiklik`, `guncelleme`, `calisma`, `isik`, `ogrenme` | TR-2 + kayıp belge sayısı (rapor) |
| aksansız (hibrit) | 2 | `sirket`, `degisiklik` | aksanlıyla ilk 10 örtüşmesi (rapor) |
| TR↔EN karışık (hibrit) | 4 | `şirket company`, `faiz rate decision` | rapor |
| anlamsal (hibrit) | 4 | `proje planındaki değişiklikler`, `haftalık rapor özeti` | rapor |

**Davranış kriterleri** (`uni_fold_tr`'in belgelenmiş kuralı: NFC; `İ`→`i`, `I`→`ı`; küçük harf; harf/rakam dizileri; **aksan katlaması yok**):
- **TR-1:** bir sözcüğün BÜYÜK ve Baş harfi büyük biçimlerinin sözcüksel sonuçları (belge, parça, sıra) küçük harf biçimininkiyle aynı (6 sözcük × 2 = 12 karşılaştırma).
- **TR-2:** her sözcüksel sorgunun belge kümesi, kopyanın belgelerinden aynı kuralla yerelde çıkarılan "bu token'ı içeren belgeler" kümesine (kâhin; ön-bilgi bloğu hariç) eşit (26 sorgu). Aksansız `sirket` yalnız `sirket` yazan belgeleri bulur; aksanlının bulup aksansızın bulmadığı belge sayısı raporlanır.
- **Raporlanan, eşiksiz:** hibrit aksansız ↔ aksanlı örtüşmesi; karışık ve anlamsal sorguların sonuç sayıları.

**Kayıt:** yalnız sayılar, eşitlikler, anahtarlı yol özetleri (`k2:`). **Okuma:** TR-1/TR-2 kalışı bir bulgudur (`FINDINGS.md`, hedef P2-C); GOAL'ün Q-1..Q-11 eşikleri değişmez.
