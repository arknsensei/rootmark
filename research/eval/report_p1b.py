"""P1-B round report (TASKS B9): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
B-1..B-10 table, the realised d1hs rates, and the decision rule of
docs/rounds/P1-B/GOAL.md derived from the pass/fail column. Writes
`<RUN_DIR>/FINAL.md` and `reports/rounds/P1-B/FINAL.md`.

    python -m eval.report_p1b --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from eval.datasets.common import ROOT, die

OUT = ROOT.parent / "reports" / "rounds" / "P1-B" / "FINAL.md"


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def commit_of(m: dict) -> str:
    """The one git sha every phase of the RUN ran at (an error otherwise)."""
    shas = {p["git_sha"] for p in m.get("phases", {}).values()}
    if len(shas) != 1:
        raise RuntimeError(f"RUN phases ran at {len(shas)} commits: {sorted(shas)}")
    return shas.pop()


def tamper(d: Path) -> dict:
    p = d / "tamper.log"
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    lines = p.read_text(encoding="utf-8").splitlines()
    ok = [ln for ln in lines if ln.startswith("ok ")]
    total = re.search(r"test_tamper: OK \((\d+) corruptions detected\)", lines[-1] if lines else "")
    return {
        "B-7": [ln for ln in ok if "(B-7)" in ln],
        "B-8": [ln for ln in ok if "(B-8)" in ln],
        "fixtures_root_line": any("fixtures_root is the copy's" in ln for ln in ok),
        "clean_clone_ok": any("check_frozen clean clone -> exit 0" in ln for ln in ok),
        "total": int(total.group(1)) if total else None,
    }


def evaluate(d: Path) -> tuple[list[dict], dict]:
    c = _load(d, "p1b_checks.json")
    hs = _load(d, "holdout_split.json")
    pair = _load(d, "pair_compare.json")
    t = tamper(d)
    rows = []

    def row(code, what, value, threshold, ok, src):
        rows.append(
            {
                "code": code,
                "what": what,
                "value": value,
                "threshold": threshold,
                "pass": ok,
                "src": src,
            }
        )

    row(
        "B-1",
        "C2-01..C2-08",
        f"{c['B-1']['passed']}/{c['B-1']['total']}, level_claimed {c['level_claimed']}",
        "8/8; level_claimed 2",
        c["B-1"]["passed"] == 8 and c["level_claimed"] == 2,
        "p1b_checks.json B-1, level_claimed",
    )
    row(
        "B-2",
        "C1-01..C1-17",
        f"{c['B-2']['passed']}/{c['B-2']['total']}",
        "17/17",
        c["B-2"]["passed"] == 17,
        "p1b_checks.json B-2",
    )
    r3 = c["B-3"]
    row(
        "B-3",
        "regression: v3-final 9 files + P1-A holdout/split_grid",
        f"max rel. diff {r3['v3_final']['max_relative_difference']} ({r3['v3_final']['n_fields']} fields) / "
        f"{r3['p1a_official']['max_relative_difference']} ({r3['p1a_official']['n_fields']} fields)",
        "0 / 0",
        r3["equal"],
        "p1b_checks.json B-3 (regress_vs_v3final.json, regress_vs_p1a.json)",
    )
    b = pair["bytes"]
    row(
        "B-4",
        "make verify pair: quality fields + traces.jsonl, checkpoints/",
        f"max rel. diff {pair['max_relative_difference']} ({pair['n_fields']} fields); "
        + ", ".join(f"{k} {'=' if v['equal'] else '≠'}" for k, v in b.items()),
        "0, byte-equal",
        bool(pair["B-4_equal"]),
        "pair_compare.json",
    )
    b5 = c["B-5"]
    row(
        "B-5",
        "v3-final vault opens: replay snapshot_id, events_hash",
        f"{b5['docs']} docs, replay equal {b5['replay_equal']}, events_hash "
        f"{b5['events_hash_v3_final']} → {b5['events_hash_current']}",
        "equal",
        bool(b5["pass"]) and b5["events_hash_v3_final"] == b5["events_hash_current"],
        "p1b_checks.json B-5 (v3_compat.json)",
    )
    b6 = c["B-6"]
    row(
        "B-6",
        "policy bound violations (6 cases)",
        f"{len(b6['cases'])} cases, all Err {b6['all_err']}, no fallback {b6['no_fallback']}",
        "6 Err, none silent",
        len(b6["cases"]) == 6 and b6["all_err"] and b6["no_fallback"],
        "p1b_checks.json B-6 (policy_bounds.json)",
    )
    row(
        "B-7",
        "Ö2: moved target/ + broken fixture",
        f"{len(t['B-7'])} case(s) detected; fixtures_root of the copy {t['fixtures_root_line']}",
        "make conformance error",
        len(t["B-7"]) >= 1 and t["fixtures_root_line"],
        "tamper.log",
    )
    row(
        "B-8",
        "Ö1: FROZEN.md row deleted / relaxed",
        f"{len(t['B-8'])} case(s) detected; clean clone exit 0 {t['clean_clone_ok']}",
        "check_frozen.sh error",
        len(t["B-8"]) >= 1 and t["clean_clone_ok"],
        "tamper.log",
    )
    v9 = hs["b9"]
    row(
        "B-9",
        "d1hs H1-H1 / H1-H2 / H1-H3",
        " / ".join(f"{v9[k]['value']:.6f}" for k in ("H1-H1", "H1-H2", "H1-H3")),
        "≥ 0.90 / ≤ 0.03 / ≥ 0.70",
        all(v9[k]["pass"] for k in v9),
        "holdout_split.json b9",
    )
    f = c["B-10"]
    row(
        "B-10",
        "layer transitions = replay (seeded fuzz)",
        ", ".join(
            f"seed {s}: {v['steps']} steps, {v['live_lineages']} lineages, replay equal {v['replay_equal']}"
            for s, v in f["seeds"].items()
        ),
        "equal",
        f["replay_equal"],
        "p1b_checks.json B-10 (fuzz_<seed>.json)",
    )
    ok = {r["code"]: r["pass"] for r in rows}
    core = all(ok[k] for k in ("B-1", "B-2", "B-3", "B-4", "B-5", "B-6", "B-7", "B-8", "B-10"))
    rule = {
        "validated": core,
        "lines": [],
    }
    if core:
        rule["lines"].append('B-1..B-8 ∧ B-10 geçti → spec §7, §8, §11 "Validated"; P1-C başlar.')
    if not ok["B-9"]:
        rule["lines"].append("B-9 kaldı → eşleyici karar talebi DECISIONS.md'ye; P1-C beklemez.")
    if not ok["B-5"]:
        rule["lines"].append("B-5 kaldı → geriye uyumluluk kırılmış; P1-C başlamaz.")
    if not (ok["B-3"] and ok["B-4"]):
        rule["lines"].append("B-3/B-4 kaldı → regresyon; P1-C başlamaz.")
    if not core and ok["B-3"] and ok["B-4"] and ok["B-5"]:
        rule["lines"].append(
            "B-1/B-2/B-6/B-7/B-8/B-10'dan en az biri kaldı → spec §7, §8, §11 Draft kalır."
        )
    return rows, {"rule": rule, "hs": hs, "checks": c, "tamper": t, "pair": pair}


def render(d: Path) -> str:
    rows, x = evaluate(d)
    m = _load(d, "manifest.json")
    hs = x["hs"]
    ho = _load(d, "holdout.json")
    lines = [
        "# P1-B — FINAL",
        "",
        (
            f"RUN `{d.name}` · commit `{commit_of(m)}` · "
            "üretildi: `python -m eval.report_p1b` (yalnız bu RUN dizini)."
        ),
        "",
        "## Çıkış kriterleri (GOAL, önceden kayıtlı)",
        "",
        "| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak (RUN içinde) |",
        "|---|---|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f"| {r['code']} | {r['what']} | {r['value']} | {r['threshold']} | "
            f"{'geçti' if r['pass'] else 'KALDI'} | `{r['src']}` |"
        )
    real = hs["realised_ops"]["rates"]
    lines += [
        "",
        "## d1hs (Ö3)",
        "",
        (
            f"- Kaynak: `{hs['source']['repo']}` @ `{hs['source']['rev'][:12]}` `src/*.md` "
            f"({hs['eligible_docs']} uygun, {hs['holdout_split']['n_docs']} doküman, "
            f"{hs['holdout_split']['n_steps']} puanlanan adım) — ASSUMPTIONS B8-1, DECISIONS K20-T."
        ),
        (
            f"- Gerçekleşen oranlar: split_edit {real.get('split_edit', 0):.3f} (≥ 0.25), "
            f"split_three {real.get('split_three', 0):.3f} (≥ 0.10). `holdout_split.json realised_ops`"
        ),
        (
            f"- Kesişimler: {json.dumps({k: v for k, v in hs['disjoint'].items() if k != 'n'})} "
            "`holdout_split.json disjoint`"
        ),
        "- H1 kodları: "
        + ", ".join(f"{k} {hs[k]:.6f}" for k in sorted(hs) if re.fullmatch(r"H1-H\d", k)),
        "- Yan yana (aynı RUN, aynı ölçüt): d1ho "
        + ", ".join(f"{k} {ho[k]:.6f}" for k in sorted(ho) if re.fullmatch(r"H1-H\d", k))
        + " `holdout.json`. Çekince: d1hs tabanları D2 korpusu (v3-final H1-R'de değerlendirme için"
        " kullanıldı); bölümler dilimi kolaylaştırıyor olabilir (K20-T).",
        "",
        "## Karar kuralı (GOAL'den türetildi)",
        "",
    ]
    lines += [f"- {ln}" for ln in x["rule"]["lines"]]
    g = x["checks"]["governance"]
    lines += [
        "",
        "## Yönetişim senaryosu (B-4 bayt karşılaştırmasının konusu)",
        "",
        (
            f"- `events_hash` {g['events_hash']} ({g['events']} olay), `traces.jsonl` {g['traces']} satır "
            f"(sha256 {g['traces_sha256'][:16]}…), {len(g['checkpoints'])} checkpoint "
            f"({', '.join(cp['name'] for cp in g['checkpoints'])}). `governance/governance.json`"
        ),
        f"- Kasıtlı bozmalar: {x['tamper']['total']} yakalandı. `tamper.log`",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    text = render(a.run_dir)
    (a.run_dir / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(text)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
