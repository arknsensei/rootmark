"""P1-A round report (TASKS A7): `FINAL.md` from exactly one RUN directory.

Reads only `<run_dir>/{manifest, p1a_checks, holdout, split_grid,
regress_vs_v3final, h1s_v3, h1r_v3, h4_v2repro}.json` and, when present,
`<run_dir>/pair_compare.json` (written by `scripts/verify.sh pair`; without it
A-4 is reported as "doğrulamada"). Any other missing file is an error; no
search, no "latest", no fallback. Every number comes from these JSON files and
its source path is printed under its table. Thresholds are coded verbatim from
docs/rounds/P1-A/GOAL.md; the decision rule is applied mechanically.

    python -m eval.report_p1a --run-dir <research/reports/runs/RUN> [--out <FINAL.md>]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets import synthetic_edits_v2 as s2
from eval.datasets.common import ROOT, die

REQUIRED = [
    "manifest.json",
    "p1a_checks.json",
    "holdout.json",
    "split_grid.json",
    "regress_vs_v3final.json",
    "h1s_v3.json",
    "h1r_v3.json",
    "h4_v2repro.json",
]
OPTIONAL = ["pair_compare.json"]
DEFAULT_OUT = ROOT.parent / "reports" / "rounds" / "P1-A" / "FINAL.md"

# docs/rounds/P1-A/GOAL.md §Önceden kayıtlı çıkış kriterleri (verbatim thresholds)
A2 = [("H1-H1", ">=", 0.90), ("H1-H2", "<=", 0.03), ("H1-H3", ">=", 0.70), ("H1-H4", ">=", 0.90)]
A5 = {"v2": "8914f9b418ae360d49c85c951050f432", "v3": "808a41f21a43e121d7d688fee9f5bc24"}


def _fmt(row: dict, key: str) -> str:
    return f"{row[key]:.4f}" if key in row else "—"


def load(run_dir: Path) -> dict:
    missing = [n for n in REQUIRED if not (run_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(f"{run_dir}: missing {missing}")
    out = {n: json.loads((run_dir / n).read_text(encoding="utf-8")) for n in REQUIRED}
    for n in OPTIONAL:
        p = run_dir / n
        out[n] = json.loads(p.read_text(encoding="utf-8")) if p.is_file() else None
    return out


def _ok(v: float, op: str, bound: float) -> bool:
    return v >= bound if op == ">=" else v <= bound


def criteria(d: dict) -> list[dict]:
    c, h, g = d["p1a_checks.json"], d["holdout.json"], d["split_grid.json"]
    reg, pair = d["regress_vs_v3final.json"], d["pair_compare.json"]
    rows = []
    a1 = c["A-1"]
    rows.append(
        {
            "code": "A-1",
            "what": "C1-01..C1-17",
            "threshold": "hepsi geçer",
            "value": f"{a1['passed']}/{a1['total']} (level {a1['level_claimed']}, fixture manifest {a1['fixtures_manifest']})",
            "pass": a1["passed"] == a1["total"] == 17
            and a1["level_claimed"] == 1
            and a1["fixtures_manifest"] == "ok",
            "src": "p1a_checks.json:A-1.{passed,total,level_claimed,fixtures_manifest}",
        }
    )
    a2 = {k: h[k] for k, _, _ in A2}
    rows.append(
        {
            "code": "A-2",
            "what": "Holdout H1-H1 / H1-H2 / H1-H3 / H1-H4",
            "threshold": "≥ 0.90 / ≤ 0.03 / ≥ 0.70 / ≥ 0.90",
            "value": " / ".join(f"{a2[k]:.6f}" for k, _, _ in A2),
            "pass": all(_ok(a2[k], op, b) for k, op, b in A2),
            "src": "holdout.json:{H1-H1,H1-H2,H1-H3,H1-H4}",
        }
    )
    rows.append(
        {
            "code": "A-3",
            "what": "v3 D1-easy, D1-hard, D2 sayıları v3-final ile",
            "threshold": "birebir",
            "value": f"{reg['n_fields']} alan, maks. göreli fark {reg['max_relative_difference']}",
            "pass": reg["H4-2_equal"] is True and reg["max_relative_difference"] == 0.0,
            "src": "regress_vs_v3final.json:{n_fields,max_relative_difference,H4-2_equal}",
        }
    )
    if pair is None:
        rows.append(
            {
                "code": "A-4",
                "what": "make verify çift koşturma, kalite alanları maks. göreli fark",
                "threshold": "0",
                "value": "—",
                "pass": None,
                "src": "doğrulamada (pair_compare.json yok)",
            }
        )
    else:
        rows.append(
            {
                "code": "A-4",
                "what": "make verify çift koşturma, kalite alanları maks. göreli fark",
                "threshold": "0",
                "value": f"{pair['max_relative_difference']} ({pair['n_fields']} alan, {len(pair['files'])} dosya)",
                "pass": pair["A-4_equal"] is True and pair["max_relative_difference"] == 0.0,
                "src": "pair_compare.json:{max_relative_difference,n_fields,files,A-4_equal}",
            }
        )
    a5 = c["A-5"]
    rows.append(
        {
            "code": "A-5",
            "what": "mini_vault events_hash v2 / v3",
            "threshold": f"{A5['v2']} / {A5['v3']}",
            "value": f"{a5['events_hash']['v2']} / {a5['events_hash']['v3']}",
            "pass": a5["events_hash"] == A5,
            "src": "p1a_checks.json:A-5.events_hash",
        }
    )
    a6 = c["A-6"]
    rows.append(
        {
            "code": "A-6",
            "what": "Bağımlılık yönü (cargo tree, import tarayıcı)",
            "threshold": "ters bağımlılık 0",
            "value": f"ihlal {a6['violations']}; check_deps OK={a6['check_deps_ok']}; kasıtlı ters bağımlılık testi OK={a6['deliberate_reverse_dependency_test_ok']}",
            "pass": a6["violations"] == 0
            and a6["check_deps_ok"]
            and a6["deliberate_reverse_dependency_test_ok"],
            "src": "p1a_checks.json:A-6.{violations,check_deps_ok,deliberate_reverse_dependency_test_ok}",
        }
    )
    rows.append(
        {
            "code": "A-7",
            "what": "split_hard: THETA_SPLIT ızgarasında farklı amaç değeri sayısı",
            "threshold": "bilgi (eşik yok)",
            "value": str(g["A-7"]),
            "pass": None,
            "src": "split_grid.json:A-7",
        }
    )
    return rows


def decision(rows: list[dict]) -> list[str]:
    """docs/rounds/P1-A/GOAL.md §Karar kuralı, applied mechanically."""
    st = {r["code"]: r["pass"] for r in rows}
    out = []
    if st["A-4"] is None:
        out.append(
            "A-4 bu RUN'da ölçülmedi (doğrulamada); aşağıdaki çıkarımlar A-4'ün geçmesine bağlıdır."
        )
    a4 = st["A-4"] is not False
    if st["A-3"] is False or st["A-4"] is False or st["A-5"] is False:
        out.append("**A-3/A-4/A-5 kaldı → taşıma hatası; P1-B başlamaz.**")
        return out
    if st["A-2"] is False:
        out.append(
            "**A-2 kaldı → `DECISIONS.md`'ye karar talebi; eşleyici turu açılır, P1-B beklemez.**"
        )
    if st["A-1"] is False or st["A-6"] is False:
        out.append(
            '**A-1 veya A-6 kaldı → "A-1..A-6 geçti" koşulu sağlanmadı; spec §1–6 "frozen" damgası teyit edilmez, P1-B başlamaz (karar kuralında ayrıca tanımlı değil).**'
        )
        return out
    if st["A-2"] and a4:
        out.append('**A-1..A-6 geçti → spec §1–6 "frozen" damgası teyit edilir, P1-B başlar.**')
    return out


def render(run_dir: Path, d: dict) -> str:
    m, h, g = d["manifest.json"], d["holdout.json"], d["split_grid.json"]
    s1, r1 = d["h1s_v3.json"], d["h1r_v3.json"]
    v2r = d["h4_v2repro.json"]
    phases = m["phases"]
    shas = sorted({p["git_sha"] for p in phases.values()})
    clean = all(p["worktree_clean"] for p in phases.values())
    any_p = next(iter(phases.values()))
    rows = criteria(d)
    L = []
    L.append("# P1-A — Nihai Rapor")
    L.append("")
    L.append(
        f"RUN: {m['run_id']} · Commit: {', '.join(x[:12] for x in shas)} "
        f"({'temiz' if clean else 'KİRLİ ağaç'}) · Rust {any_p['versions']['rust']} · "
        f"Python {any_p['versions']['python']} · SEED {m['seed']}"
    )
    L.append("")
    L.append(
        f"> `research/eval/report_p1a.py` tarafından yalnız `{run_dir.name}` RUN dizininden üretildi. "
        "Elle sayı yoktur; her tablonun altında kaynak alan yolları vardır. Eşikler "
        "`docs/rounds/P1-A/GOAL.md`'den aynen kodlanmıştır."
    )
    L.append("")
    L.append("## 1. Çıkış kriterleri")
    L.append("| Kod | Ölçüm | Eşik | Sonuç | Durum |")
    L.append("|---|---|---|---|---|")
    for r in rows:
        status = {
            True: "GEÇTİ",
            False: "KALDI",
            None: "bilgi" if r["code"] == "A-7" else "doğrulamada",
        }[r["pass"]]
        L.append(f"| {r['code']} | {r['what']} | {r['threshold']} | {r['value']} | {status} |")
    L.append("")
    L.append("Kaynak: " + " · ".join(f"{r['code']} ← `{r['src']}`" for r in rows))
    L.append("")
    L.append("## 2. Karar kuralı çıktısı")
    for line in decision(rows):
        L.append(f"- {line}")
    L.append("")
    L.append("## 3. A-3 — v3-final regresyonu (seçili alanlar)")
    L.append("| Alan | Değer |")
    L.append("|---|---|")
    for k in ("H1-S1", "H1-S2", "H1-S3", "H1-S4", "H1-H1", "H1-H2", "H1-H3", "H1-H4", "H1-G"):
        L.append(f"| {k} | {s1[k]} |")
    for k in ("H1-R1", "H1-R2"):
        L.append(f"| {k} | {r1[k]} |")
    L.append(f"| H4-V2 equal | {v2r['H4-V2']['equal']} |")
    L.append("")
    L.append(
        "Kaynak: `h1s_v3.json:{H1-S1..S4,H1-H1..H4,H1-G}` · `h1r_v3.json:{H1-R1,H1-R2}` · "
        "`h4_v2repro.json:H4-V2.equal`; tüm 9 dosyanın alan alan karşılaştırması "
        "`regress_vs_v3final.json` (referans RUN `20261013T182004Z-f82dabd`)."
    )
    L.append("")
    L.append("## 4. A-2 — Holdout")
    ho = h["holdout"]
    L.append(
        f"{ho['n_docs']} doküman, {ho['n_steps']} puanlı adım · eşleyici {h['matcher']} "
        f"(THETA={h['theta']}, THETA_SPLIT={h['theta_split']}, derlenmiş) · uygun aday {h['eligible_docs']}"
    )
    L.append("")
    target = dict(s2.OPS_HARD)
    rates = h["realised_ops"]["rates"]
    L.append(
        "| Op | Hedef oran (D1-hard) | Gerçekleşen oran | Adım | Koruma | Yanlış eşleme | Olay recall |"
    )
    L.append("|---|---|---|---|---|---|---|")
    for op, row in ho["per_op"].items():
        L.append(
            f"| {op} | {target[op]:.2f} | {rates[op]:.3f} | {row['steps']} | {_fmt(row, 'retention')} | "
            f"{_fmt(row, 'false_match')} | {_fmt(row, 'event_recall')} |"
        )
    det = ho["H1-H3_detail"]
    L.append("")
    L.append(
        f"Olay F1 ayrıntısı: P {det['precision']:.4f} · R {det['recall']:.4f} · tp {det['tp']} / "
        f"tahmin {det['n_pred']} / gerçek {det['n_truth']} · payda H1 {ho['n_retention']}, "
        f"H2 {ho['n_false_match']}, H4 {ho['n_dup']}"
    )
    L.append("")
    dj = h["disjoint"]
    L.append(
        f"Kesişimler (programatik): holdout∩easy {dj['holdout_and_easy']}, holdout∩hard "
        f"{dj['holdout_and_hard']}, easy∩hard {dj['easy_and_hard']}, üçü {dj['all_three']}."
    )
    L.append("")
    split_steps = sum(
        ho["per_op"][o]["steps"] for o in ("split_edit", "split_three") if o in ho["per_op"]
    )
    merge_steps = ho["per_op"].get("merge_edit", {}).get("steps", 0)
    L.append(
        "Not: gerçekleşen oranlar yeniden çekilişlerden sonradır ve D1-hard tablosundan sapar "
        "(easy ∪ hard dışında çok cümleli paragraf az; ASSUMPTIONS A5). H1-H3 olay gerçeklerinin "
        f"kaynağı: bölme adımı {split_steps}, birleştirme adımı {merge_steps} — holdout olay F1'i "
        "ağırlıkla birleştirmeleri ölçer."
    )
    L.append("")
    L.append(
        "Kaynak: `holdout.json:holdout.{per_op,H1-H3_detail,n_retention,n_false_match,n_dup}` · "
        "`holdout.json:{disjoint,realised_ops,eligible_docs}` · hedef oranlar "
        "`eval/datasets/synthetic_edits_v2.py:OPS_HARD`"
    )
    L.append("")
    L.append("## 5. A-7 — Bölme eşiği zorluk dilimi (bilgi)")
    L.append(
        f"{g['n_docs']} doküman, {g['n_steps']} puanlı adım · THETA={g['theta']} sabit · "
        f"derlenmiş THETA_SPLIT={g['compiled_theta_split']} değiştirilmedi · alt dizi kontrolü "
        f"{g['substring_check']['split_pieces_checked']} parça, {g['substring_check']['substring_pieces']} alt dizi"
    )
    L.append("")
    variants = sorted({v for r in g["grid"] for v in r["per_variant"]})
    L.append(
        "| THETA_SPLIT | H1-H1 | H1-H2 | H1-H3 | H1-H4 | Amaç | "
        + " | ".join(f"{v} olay recall" for v in variants)
        + " |"
    )
    L.append("|---" * (6 + len(variants)) + "|")
    for r in g["grid"]:
        L.append(
            f"| {r['theta_split']:.2f} | {r['H1-H1']:.4f} | {r['H1-H2']:.4f} | {r['H1-H3']:.4f} | "
            f"{r['H1-H4']:.4f} | {r['objective']:.6f} | "
            + " | ".join(f"{r['per_variant'][v]['event_recall']:.4f}" for v in variants)
            + " |"
        )
    L.append("")
    L.append(f"Farklı amaç değeri sayısı (6 ondalık): **{g['A-7']}**.")
    L.append("")
    L.append("Kaynak: `split_grid.json:{grid[*],A-7,substring_check,n_docs,n_steps}`")
    L.append("")
    L.append("## 6. Koşturma kaydı")
    L.append("| Faz | Commit | Temiz | Çıktılar |")
    L.append("|---|---|---|---|")
    for name, p in sorted(phases.items()):
        L.append(
            f"| {name} | {p['git_sha'][:12]} | {p['worktree_clean']} | {', '.join(p['outputs'])} |"
        )
    L.append("")
    L.append("Kaynak: `manifest.json:phases`")
    L.append("")
    return "\n".join(L)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    a = ap.parse_args(argv)
    run_dir = a.run_dir.resolve()
    d = load(run_dir)
    text = render(run_dir, d)
    (run_dir / "FINAL.md").write_text(text, encoding="utf-8")
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(text, encoding="utf-8")
    print(f"wrote {run_dir / 'FINAL.md'} and {a.out}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
