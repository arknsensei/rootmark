"""P1-C round report (TASKS C9): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
C-1..C-11 table, the ADDENDUM C-12/C-13 rows (part of C-2), and the decision
rule of docs/rounds/P1-C/GOAL.md, derived from the rows:

- C-1..C-11 pass → spec §9 "Validated", `level_claimed 3`; P1-D starts.
- C-3 or C-10 fails → embedder/tokenizer mismatch: decision request; P1-D
  starts (MCP does not depend on index quality), the 0.1.0 release waits.
- C-4, C-5 or C-6 fails → determinism: P1-D does not start.
- C-7 fails → governance violation: P1-D does not start.
- any other failure (C-1, C-2 incl. C-12/C-13, C-8, C-9, C-11) is not named
  by the rule: reported as such, no branch is assumed.

C-8 is machine-dependent (VERIFY V8: its status): read from
`c8_latency.json`, outside the pair comparison. Writes `<RUN_DIR>/FINAL.md`
and `reports/rounds/P1-C/FINAL.md`.

    python -m eval.report_p1c --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-C" / "FINAL.md"


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool) -> str:
    return "geçti" if b else "**KALDI**"


def tamper(d: Path) -> dict:
    p = d / "tamper_p1c.log"
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    lines = p.read_text(encoding="utf-8").splitlines()
    m = re.search(r"tamper P1-C: (\d+)/(\d+) corruptions rejected", lines[-1] if lines else "")
    if not m:
        raise RuntimeError("tamper_p1c.log: no summary line")
    return {
        "rejected": int(m.group(1)),
        "total": int(m.group(2)),
        "cases": [ln.split("rejected: ", 1)[1] for ln in lines if "rejected: " in ln],
    }


def evaluate(d: Path) -> list[dict]:
    c = _load(d, "p1c_checks.json")
    pair = _load(d, "pair_compare.json")
    c8 = _load(d, "c8_latency.json")
    q = c["C-3"]
    rows = [
        {
            "id": "C-1",
            "what": "C3-01..C3-04, `level_claimed 3`",
            "value": f"C3 {c['C-1']['C3']}, level {c['C-1']['level_claimed']}, spec {c['C-1']['spec_version']}",
            "threshold": "4/4, level 3",
            "pass": c["C-1"]["pass"],
            "source": "conformance_full.json",
        },
        {
            "id": "C-2",
            "what": "C1 17/17, C2 9/9 (ADDENDUM)",
            "value": f"C1 {c['C-2']['C1']}, C2 {c['C-2']['C2']}",
            "threshold": "geçer",
            "pass": c["C-2"]["pass"] and c["C-12"]["pass"] and c["C-13"]["pass"],
            "source": "conformance_full.json, c13_tests.log",
        },
        {
            "id": "C-3",
            "what": "D3 + D4 test, `hybrid/structural` nDCG@10 Δ vs v2-final (single, multi)",
            "value": f"single {q['single']['ndcg@10']:.4f} (v2 {q['single']['reference']:.4f}, "
            f"Δ {q['single']['delta']:+.4f}); multi {q['multi']['ndcg@10']:.4f} "
            f"(v2 {q['multi']['reference']:.4f}, Δ {q['multi']['delta']:+.4f})",
            "threshold": f"\\|Δ\\| ≤ {q['threshold']}",
            "pass": q["pass"],
            "source": "c3_quality.json",
        },
        {
            "id": "C-4",
            "what": "aynı sorgu, checkpoint, indeks: 100 tekrar (3 mod, D3)",
            "value": f"{c['C-4']['queries']} sorgu × {len(c['C-4']['modes'])} mod × {c['C-4']['repeats']}",
            "threshold": "birebir",
            "pass": c["C-4"]["pass"],
            "source": "p1c_checks.json",
        },
        {
            "id": "C-5",
            "what": "indeks silinip aynı checkpoint'ten yeniden kurulunca sıralama (200 sorgu × 3 mod)",
            "value": f"sıralama eşit {c['C-5']['rankings_equal']}, indeks dosyaları eşit {c['C-5']['index_files_equal']}",
            "threshold": "birebir",
            "pass": c["C-5"]["pass"],
            "source": "p1c_checks.json",
        },
        {
            "id": "C-6",
            "what": "çift koşturma: `index/` bayt (dosya hash'leri), 200 sorgunun sıralaması, tüm kalite alanları",
            "value": f"{pair['n_fields']} alan, en büyük fark {pair['max_relative_difference']}, "
            f"farklı {len(pair['differing_fields'])}, yönetişim baytları eşit {pair['bytes_equal']}",
            "threshold": "eşit",
            "pass": pair["C-6_equal"] and pair["max_relative_difference"] == 0,
            "source": "pair_compare.json",
        },
        {
            "id": "C-7",
            "what": "`proposal` varsayılanda yok; `include_proposal` ile etiketli ve sonda; `derived` asla",
            "value": f"{c['C-7']['queries']} sorgu, {c['C-7']['proposal_hits_with_include_proposal']} proposal hit "
            f"(chunk katmanları {c['C-7']['chunks_by_layer']})",
            "threshold": "property 1000 sorgu",
            "pass": c["C-7"]["pass"] and c["C-7"]["queries"] >= 1000,
            "source": "c7_property.json",
        },
        {
            "id": "C-8",
            "what": "yazma→sıcakta aranabilir (200 doküman, ONNX CPU, model yüklü)",
            "value": f"p50 {c8['p50_ms']:.1f} ms, p95 {c8['p95_ms']:.1f} ms ({c8['edits']} düzenleme; makineye bağlı)",
            "threshold": "p95 < 100 ms",
            "pass": c8["pass"] and c8["p95_ms"] < 100.0,
            "source": "c8_latency.json",
        },
        {
            "id": "C-9",
            "what": "`compact()` öncesi/sonrası 200 sorgu sıralaması (iki checkpoint, 3 mod)",
            "value": f"{c['C-9']['edits']} sıcak düzenleme, taşınan {c['C-9']['compact']['moved_from_hot']}, "
            f"eşit {c['C-9']['rankings_equal']}",
            "threshold": "birebir",
            "pass": c["C-9"]["pass"],
            "source": "p1c_checks.json",
        },
        {
            "id": "C-10",
            "what": "50 sabit chunk: ONNX ↔ prototip sentence-transformers kosinüs",
            "value": f"en düşük {c['C-10']['min_cosine']:.13f} (n {c['C-10']['n']})",
            "threshold": f"≥ {c['C-10']['threshold']}",
            "pass": c["C-10"]["pass"],
            "source": "c10_embedding.json",
        },
        {
            "id": "C-11",
            "what": "P1-B.1 kalite alanları (P1-B.1 resmi RUN ile)",
            "value": f"karar gereği hariç dışında farklı {len(c['C-11']['differing_outside_exclusions'])}",
            "threshold": "fark 0",
            "pass": c["C-11"]["pass"],
            "source": "regress_vs_p1b1.json",
        },
        {
            "id": "C-12",
            "what": "C2-09 (ADDENDUM; C-2'nin parçası)",
            "value": f"C2-09 {c['C-12']['C2-09']}",
            "threshold": "geçer",
            "pass": c["C-12"]["pass"],
            "source": "conformance_full.json",
        },
        {
            "id": "C-13",
            "what": "yeniden numaralı silme: iki `Err` + bir türetilmiş önek (ADDENDUM; C-2'nin parçası)",
            "value": f"{sum(c['C-13']['tests'].values())}/{len(c['C-13']['tests'])}",
            "threshold": "3/3",
            "pass": c["C-13"]["pass"],
            "source": "c13_tests.log",
        },
    ]
    return rows


def decision(rows: list[dict]) -> list[str]:
    ok = {r["id"]: r["pass"] for r in rows}
    core = [f"C-{i}" for i in range(1, 12)]
    if all(ok[k] for k in core):
        return ['C-1..C-11 geçti → spec §9 **"Validated"**, `level_claimed 3`; **P1-D başlar**.']
    out = []
    det = [k for k in ("C-4", "C-5", "C-6") if not ok[k]]
    if det:
        out.append(f"{', '.join(det)} kaldı → determinizm; **P1-D başlamaz**.")
    if not ok["C-7"]:
        out.append("C-7 kaldı → yönetişim ihlali; **P1-D başlamaz**.")
    emb = [k for k in ("C-3", "C-10") if not ok[k]]
    if emb:
        out.append(
            f"{', '.join(emb)} kaldı → embedder/tokenizer uyumsuzluğu; karar talebi; "
            + ("P1-D **başlamaz** (yukarıdaki dal)" if det or not ok["C-7"] else "**P1-D başlar**")
            + " (MCP indeks kalitesine bağlı değil), **0.1.0 yayımı bekler**."
        )
    other = [k for k in ("C-1", "C-2", "C-8", "C-9", "C-11") if not ok[k]]
    if other:
        out.append(
            f"{', '.join(other)} kaldı → karar kuralı bu kriter(ler)i adlandırmaz; dal varsayılmaz, karar talebi gerekir."
        )
    out.append('Spec §9 "Validated" damgası almaz (C-1..C-11 hepsi geçmedi); §9 "Draft" kalır.')
    return out


def flat_note(q: dict) -> str:
    """What the flat cells say about the embedder (computed, not assumed)."""
    flat = [k for k in q["delta"] if k.startswith("flat/")]
    if flat and all(q["delta"][k][m] == 0 for k in flat for m in ("ndcg@10", "recall@10", "mrr")):
        return (
            "`flat` hücrelerinde nDCG@10, recall@10, MRR Δ 0: embedder ve kesin kosinüs prototipi "
            "üretir; `hybrid` farkı sözcüksel taraftan (BM25 varyantı; DECISIONS K-C8-T)."
        )
    return "`flat` hücrelerinde Δ 0 değil: embedder tarafında da fark var."


def render(d: Path) -> str:
    man = _load(d, "manifest.json")
    sha = commit_of(man)
    rows = evaluate(d)
    c = _load(d, "p1c_checks.json")
    q = _load(d, "c3_quality.json")
    t = tamper(d)
    lines = [
        "# P1-C — Sonuç raporu",
        "",
        f"- RUN: `{d.name}` · ölçülen commit `{sha}`",
        "- Üretim: `python -m eval.report_p1c --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).",
        f"- Embedder: `{c['models']['embedder_id']}` (model manifest OK: {c['models']['ok']}).",
        "",
        "## Çıkış kriterleri (GOAL P1-C + ADDENDUM)",
        "",
        "| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |",
        "|---|---|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f"| {r['id']} | {r['what']} | {r['value']} | {r['threshold']} | {_ok(r['pass'])} | `{r['source']}` |"
        )
    lines += [
        "",
        "## C-3 hücreleri (`c3_quality.json`; v2-final `" + q["reference_run"] + "`)",
        "",
        "| Hücre | n | nDCG@10 | v2 | Δ | recall@10 Δ | MRR Δ |",
        "|---|---|---|---|---|---|---|",
    ]
    for key in sorted(q["cells"]):
        cell, ref, dl = q["cells"][key], q["reference_cells"][key], q["delta"][key]
        lines.append(
            f"| {key} | {cell['n']} | {cell['ndcg@10']:.4f} | {ref['ndcg@10']:.4f} | {dl['ndcg@10']:+.4f} "
            f"| {dl['recall@10']:+.4f} | {dl['mrr']:+.4f} |"
        )
    lines += [
        "",
        flat_note(q),
        "",
        f"## Kasıtlı bozma (`tamper_p1c.log`): {t['rejected']}/{t['total']} ret",
        "",
        *[f"- {x}" for x in t["cases"]],
        "",
        "## C-11 karar gereği hariç tutulan alanlar (`regress_vs_p1b1.json`)",
        "",
        *(
            [f"- `{k}`" for k in _load(d, "regress_vs_p1b1.json")["differing_excluded"]]
            or ["- (fark yok)"]
        ),
        "",
        "## Karar kuralı (GOAL P1-C)",
        "",
        *decision(rows),
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
