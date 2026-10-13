
---

# Aktif tur: docs/v3 (v2-final etiketinden sonra)

Bu bölüm `CLAUDE.md`'nin üstündeki kuralları **değiştirmez**, daraltır ve üstüne ekler.

## Okuma sırası (v3)
1. `docs/v3/00_GOAL.md`
2. `docs/ARCHITECTURE.md` §4.1, §10 (K4 kararı)
3. `docs/v3/01_SPEC_DELTA.md` (üzerine yazdığı yerler dışında `docs/01_SPEC.md` geçerli)
4. `docs/v3/02_TASKS.md`
5. `docs/v3/03_EVAL_DELTA.md` (üzerine yazdığı yerler dışında `docs/03_EVAL.md` geçerli)
6. `docs/progress/v3/`

## v3'e özgü kurallar
- Görevler T20'den başlar; progress kayıtları `docs/progress/v3/T<nn>.md`.
- **v2 dondurması aynen geçerli:** `eval/metrics.py`, `eval/truth.py`, `eval/datasets/*.py` (T9'daki dosyalar), `eval/freeze_check.py`, `tests/fixtures/freeze_fixture/**` değişmez. v3 yalnız yeni dosya ekler.
- **İkinci dondurma:** `T23: freeze eval v3` sonrası `eval/metrics_v3.py`, `eval/datasets/synthetic_edits_v2.py`, `eval/run_*_v3.py`, `eval/run_h4_v2repro.py`, `tests/fixtures/freeze_fixture_v3/**` değişmez. İstisna prosedürü aynı (`FREEZE_EXCEPTIONS.md`, önce/sonra `freeze-check-v3`, gözden geçirici).
- **v2 davranışı kutsal:** `MatcherVersion::V2` yolu v2-final ile bayt bayt aynı çıktıyı vermeli. Bir v3 değişikliği bunu bozuyorsa değişiklik yanlıştır, test değil.
- Eşleyici sabitleri: `V2` sabitleri (0.60, 0.70) dokunulmaz; `V3` sabitleri yalnız T24 ayar fazında yazılır.
- `docs/v3/00_GOAL.md` eşikleri değiştirilemez. K4 ile değişen S2 tanımı zaten orada kayıtlıdır; başka tanım değişikliği yapılmaz.
- H2 koşturulmaz; `graph`, `hier`, varlık, akış kodu bu turda yazılmaz.
- `ANTHROPIC_API_KEY` bu turda **gerekmez**; API çağrısı yapan kod eklenmez.
- Varsayımlar `docs/progress/v3/ASSUMPTIONS.md`'ye etiketli (`[H1] [H3] [H4] [-]`) yazılır.
- T27 bittiğinde `docs/progress/v3/T27.md` sonuna "T28 için hazır" ve RUN kimliği yazılır; dur. T28'i sen yapma.
