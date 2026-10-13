# Dondurma İstisnaları

(Boş. Girdi şartları CLAUDE.md §Ölçüm dürüstlüğü 2.)

Not (T12): `eval/datasets/questions.py` SDK 1.x'teki `temperature` argüman değişikliğine dosya dondurulmadan **önce** uyarlandı (`extra_body`); bu bir dondurma istisnası değildir (ASSUMPTIONS T12).

## v3 / T24 — `eval/freeze_check_v3.py` parametreleri sabitlendi

**Ne:** `freeze-check-v3` denetleyicisi, dondurma dışı ingest'lerde THETA/THETA_SPLIT'i derlenmiş V3 sabitlerinden (`matcher_defaults("v3")`) alıyordu. T24 ayar fazı tasarım gereği `THETA_V3`'ü 0.60 → 0.50 yaptı (commit "T24: V3 constants from D1-hard tuning"); bunun üzerine `make freeze-check-v3` çıktısı değişti (`9e8389668db180218e2e0ce7a254c6d7` ≠ `expected.json`). Düzeltme: denetleyici dondurma anındaki sabitleri (0.60, 0.65) açıkça kullanır.

**Neden / sınıf:** Bu bir bağımlılık API değişikliği **değildir**; T23'te denetleyicinin T24'te değişecek derlenmiş sabitlere bağlı yazılmasından doğan bir tasarım kusurudur. `eval/freeze_check_v3.py` CLAUDE.md v3 dondurma listesinde yer almıyor (T23.md'de gönüllü olarak donmuş ilan edildi); dondurulmuş `eval/metrics_v3.py`, `eval/datasets/synthetic_edits_v2.py`, `tests/fixtures/freeze_fixture_v3/**` (`expected.json` dahil) **değişmedi**. Şeffaflık için istisna prosedürü tam uygulanır.

**Diff:**
```diff
diff --git a/eval/freeze_check_v3.py b/eval/freeze_check_v3.py
index 8870476..a1f61f1 100644
--- a/eval/freeze_check_v3.py
+++ b/eval/freeze_check_v3.py
@@ -36,6 +36,10 @@ EDIT_SOURCES = ("roasting.md", "maintenance.md", "espresso.md", "cupping.md", "s
 EDIT_STEPS = 3
 MAX_EDIT_RUNS = 80
 GRID = ((0.55, 0.55), (0.60, 0.65), (0.70, 0.75))
+# THETA / THETA_SPLIT of every non-grid ingest: the compiled V3 constants at
+# freeze time (T23), pinned so that tuning (T24 writes THETA_V3/THETA_SPLIT_V3)
+# cannot change this output (FREEZE_EXCEPTIONS.md, T24)
+PARAMS = (0.60, 0.65)
 
 
 def _rec_lists(rec: dict) -> dict:
@@ -63,7 +67,7 @@ def _dominant_index(step: dict) -> int | None:
     return i
 
 
-def _records(truth: dict, out_dir: Path, vault_dir: Path, params=(None, None)) -> list[dict]:
+def _records(truth: dict, out_dir: Path, vault_dir: Path, params=PARAMS) -> list[dict]:
     v = mdkernel_core.Vault(str(vault_dir), params[0], params[1], matcher=MATCHER)
     doc = truth["doc_path"]
     v.ingest(doc, (out_dir / truth["base_file"]).read_text(encoding="utf-8"), "Synthetic", "d1")
@@ -165,7 +169,7 @@ def check_h1r(tmp: Path) -> dict:
     old = (FIX / pair["old"]).read_bytes().decode("utf-8")
     new = (FIX / pair["new"]).read_bytes().decode("utf-8")
     drafts, _ = mdkernel_core.parse_document(old, pair["path"])
-    v = mdkernel_core.Vault(str(tmp / "h1r"), matcher=MATCHER)
+    v = mdkernel_core.Vault(str(tmp / "h1r"), PARAMS[0], PARAMS[1], matcher=MATCHER)
     v.ingest(pair["path"], old)
     old_blocks = v.snapshot_blocks(pair["path"])
     for b, d in zip(old_blocks, drafts, strict=True):
@@ -263,7 +267,9 @@ def run() -> dict:
         tmp = Path(t)
         return {
             "matcher": MATCHER,
-            "matcher_defaults": list(mdkernel_core.matcher_defaults(MATCHER)),
+            # key kept for bit-identity with expected.json: the pinned PARAMS
+            # (equal to the compiled V3 defaults when the fixture was frozen)
+            "matcher_defaults": list(PARAMS),
             "easy": check_easy(tmp),
             "hard": check_hard(tmp),
             "h1r": check_h1r(tmp),
```

**Bit düzeyinde eşitlik kaydı (`tests/fixtures/freeze_fixture_v3/`):**
- Önce (dondurma anı, commit aa3a2e2, V3 sabitleri 0.60/0.65): `freeze-check-v3 OK (d138cddcde1d0978a020a6fe8df2f610)` (T23.md, T23.review.md).
- Eski denetleyici + T24 sabitleri: `freeze-check-v3 FAILED: output 9e8389668db180218e2e0ce7a254c6d7`.
- Sonra (yeni denetleyici + T24 sabitleri): `freeze-check-v3 OK (d138cddcde1d0978a020a6fe8df2f610)` — `expected.json` ile bayt bayt aynı; `MDK_MATCHER=v2` altında da aynı.
- `make freeze-check` (v2): `d489a0ed44b15d3aeb272e16cdd4a5af` (değişmedi).

**Gözden geçirici onayı:** ONAYLANDI — `docs/progress/v3/T24.review.md` ("FREEZE_EXCEPTIONS v3/T24: ONAYLANDI"; eski denetleyici `9e838966…` FAILED, yeni denetleyici `d138cddc…` OK bağımsız olarak yeniden üretildi).
