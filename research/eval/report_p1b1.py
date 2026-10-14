"""P1-B.1 round report (TASKS B1.4): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
B1-1..B1-7 table and the decision rule of docs/rounds/P1-B.1/GOAL.md
(B1-1..B1-7 pass → P1-C starts; the P1-B decision rule's B-5 branch is
closed). Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P1-B.1/FINAL.md`.

    python -m eval.report_p1b1 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-B.1" / "FINAL.md"


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool) -> str:
    return "geçti" if b else "**KALDI**"


def evaluate(d: Path) -> list[dict]:
    c = _load(d, "p1b1_checks.json")["criteria"]
    pair = _load(d, "pair_compare.json")
    m = c["B1-1"]["v3_compat_mixed"]
    b11_tests = sum(v == "ok" for v in c["B1-1"]["tests"].values())
    rows = [
        {
            "id": "B1-1",
            "what": "v3-final kütüphane API'siyle 9 senaryo mevcut çekirdekle açılır (replay, events_hash, blame)",
            "value": f"check_v3_compat {m['passed']}/{m['total']} senaryo; compat_v3final {b11_tests}/9 test",
            "threshold": "9/9",
            "pass": c["B1-1"]["pass"],
            "source": "v3_compat_mixed.json, p1b1_tests.log",
        },
        {
            "id": "B1-2",
            "what": "yeni biçimli günlükte eksik `LayerChanged`",
            "value": ", ".join(f"{k} {v}" for k, v in c["B1-2"]["tests"].items()),
            "threshold": "`Err`",
            "pass": c["B1-2"]["pass"],
            "source": "p1b1_tests.log",
        },
        {
            "id": "B1-3",
            "what": "C1 17/17, C2 8/8 (C2-07 yeni beklenen), `level_claimed 2`",
            "value": f"C1 {c['B1-3']['C1']}, C2 {c['B1-3']['C2']}, level {c['B1-3']['level_claimed']}, "
            f"spec {c['B1-3']['spec_version']}",
            "threshold": "geçer",
            "pass": c["B1-3"]["pass"],
            "source": "conformance.json",
        },
        {
            "id": "B1-4",
            "what": "`trace_id` doğrulaması: 31 / 33 / hex olmayan `query_hash`; `consumer` içinde `\\n`",
            "value": ", ".join(f"{k} {v}" for k, v in c["B1-4"]["tests"].items()),
            "threshold": "4/4 `Err`",
            "pass": c["B1-4"]["pass"],
            "source": "p1b1_tests.log",
        },
        {
            "id": "B1-5",
            "what": 'kayma: (`"ab"`, qh) ve (`"a"`, `"b"+qh`)',
            "value": ", ".join(f"{k} {v}" for k, v in c["B1-5"]["tests"].items()),
            "threshold": "farklı (ikincisi `Err`)",
            "pass": c["B1-5"]["pass"],
            "source": "p1b1_tests.log",
        },
        {
            "id": "B1-6",
            "what": "P1-B resmi RUN'ı ile kalite alanları (C2-07, `traces.jsonl` hariç)",
            "value": f"{c['B1-6']['n_fields']} alan; hariç tutulan farklar {len(c['B1-6']['differing_excluded'])}, "
            f"dışında {len(c['B1-6']['differing_outside_exclusions'])}; bayt: "
            + ", ".join(
                f"{k.rsplit('/', 2)[-2] if k.endswith('/') else k.rsplit('/', 1)[-1]}"
                f"{'/' if k.endswith('/') else ''} {'=' if v else '≠'}"
                + (" (hariç)" if k.endswith("traces.jsonl") else "")
                for k, v in c["B1-6"]["bytes"].items()
            ),
            "threshold": "fark 0",
            "pass": c["B1-6"]["pass"],
            "source": "regress_vs_p1b.json",
        },
        {
            "id": "B1-7",
            "what": "çift koşturma (`make verify` pair, compare_p1b)",
            "value": f"{pair['n_fields']} alan, max fark {pair['max_relative_difference']}, "
            f"bayt eşit {pair['bytes_equal']}",
            "threshold": "fark 0, bayt eşit",
            "pass": pair["B-4_equal"]
            and pair["bytes_equal"]
            and pair["max_relative_difference"] == 0,
            "source": "pair_compare.json",
        },
    ]
    return rows


def render(d: Path) -> str:
    man = _load(d, "manifest.json")
    sha = commit_of(man)
    rows = evaluate(d)
    c = _load(d, "p1b1_checks.json")["criteria"]
    all_ok = all(r["pass"] for r in rows)
    lines = [
        "# P1-B.1 — Sonuç raporu",
        "",
        f"- RUN: `{d.name}` · ölçülen commit `{sha}`",
        "- Üretim: `python -m eval.report_p1b1 --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).",
        "",
        "## Çıkış kriterleri (GOAL P1-B.1)",
        "",
        "| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |",
        "|---|---|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f"| {r['id']} | {r['what']} | {r['value']} | {r['threshold']} | {_ok(r['pass'])} | `{r['source']}` |"
        )
    lines += ["", "## B1-1 senaryoları (`v3_compat_mixed.json`)", ""]
    lines += ["| Senaryo | Doküman | blame eşit | Sonuç |", "|---|---|---|---|"]
    for s in c["B1-1"]["v3_compat_mixed"]["scenarios"]:
        lines.append(f"| {s['scenario']} | {s['docs']} | {s['blame_equal']} | {_ok(s['pass'])} |")
    lines += [
        "",
        "## B1-6 hariç tutulan alanlar (`regress_vs_p1b.json`)",
        "",
        *([f"- `{k}`" for k in c["B1-6"]["differing_excluded"]] or ["- (fark yok)"]),
        "",
        f"C2-07 yeni beklenen hash: `{c['B1-3']['C2-07_expected_hash']}`.",
        "",
        "## Karar kuralı (GOAL P1-B.1 V8)",
        "",
        "B1-1..B1-7 geçti → P1-B karar kuralının B-5 dalı kapanır, **P1-C başlar**."
        if all_ok
        else "B1-1..B1-7'den en az biri kaldı → **P1-C başlamaz** (P1-B karar kuralı, B-5 dalı).",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    try:
        text = render(a.run_dir)
    except (FileNotFoundError, RuntimeError, KeyError) as e:
        die(str(e))
    (a.run_dir / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {a.run_dir / 'FINAL.md'} and {OUT}")


if __name__ == "__main__":
    main()
