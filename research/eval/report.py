"""`make report RUN=<run_id>`: builds FINAL.md from one run directory only.

Reads exactly `reports/runs/<RUN>/{manifest,h1s_tuning,h1s,h1r,h2_tuning,h2,h3,h4}.json`
(any missing file is an error; nothing is searched for, nothing falls back),
plus the committed data manifests, docs/progress/{ASSUMPTIONS,FREEZE_EXCEPTIONS}.md,
progress records (for PARK lines) and the number-free commentary in
`reports/commentary.md`. Writes `reports/runs/<RUN>/FINAL.md` + `figs/*.png`
and copies them to `reports/FINAL.md` and `reports/figs/`.

    python -m eval.report --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
from pathlib import Path

from eval import runinfo
from eval.datasets.common import DATA, ROOT, die

REQUIRED = [
    "manifest.json",
    "h1s_tuning.json",
    "h1s.json",
    "h1r.json",
    "h2_tuning.json",
    "h2.json",
    "h3.json",
    "h4.json",
]
PROGRESS = ROOT / "docs" / "progress"
COMMENTARY = ROOT / "reports" / "commentary.md"

# Pre-registered thresholds (docs/00_GOAL.md); (code, measurement, threshold text, comparator, bound)
THRESHOLDS = [
    ("H1-S1", "Sentetik soy koruma (rephrase_*, move)", "≥ 0.95", ">=", 0.95),
    ("H1-S2", "Sentetik yanlış eşleme", "≤ 0.02", "<=", 0.02),
    ("H1-S3", "Bölme/birleştirme F1", "≥ 0.80", ">=", 0.80),
    ("H1-S4", "Tekrar-blok soy koruma (dup_delete)", "≥ 0.95", ">=", 0.95),
    ("H1-R1", "Gerçek tarihçe, dokunulmamış blok koruma", "≥ 0.999", ">=", 0.999),
    ("H1-R2", "Gerçek tarihçe, genel koruma", "≥ 0.85", ">=", 0.85),
    ("H2-1", "graph_hybrid − hybrid, multi ΔnDCG@10", "≥ +0.03", ">=", 0.03),
    ("H2-2", "graph_hybrid − hybrid, single ΔnDCG@10", "≥ −0.02", ">=", -0.02),
    ("H2-3", "graph_hybrid p95 / hybrid p95 gecikme", "≤ 3.0", "<=", 3.0),
    ("H2-4", "graph_hybrid − hybrid, multi Δfull_cover@10", "≥ +0.03", ">=", 0.03),
    ("H2-5", "graph_flat − flat, multi ΔnDCG@10", "≥ +0.05", ">=", 0.05),
    ("H3-1", "Yazma→aranabilir p95 (ms)", "< 1000", "<", 1000.0),
    ("H3-2", "Yeniden-chunk soy kaybı (kayıp + kazanç)", "= 0", "==", 0),
    ("H4-1", "Olay günlüğü hash eşitliği (D3 yeniden kurulum)", "= eşit", "is", True),
    ("H4-2", "İki temiz koşturmada kalite metrikleri maks. göreli fark", "= 0", "verify", None),
]


def passes(cmp: str, value, bound) -> bool:
    return {
        ">=": lambda: value >= bound,
        "<=": lambda: value <= bound,
        "<": lambda: value < bound,
        "==": lambda: value == bound,
        "is": lambda: value is bound,
    }[cmp]()


def load_run(run: str) -> dict:
    d = runinfo.run_dir(run, create=False)
    missing = [n for n in REQUIRED if not (d / n).exists()]
    if missing:
        raise FileNotFoundError(f"run {run} is missing {missing}")
    return {n[:-5]: json.loads((d / n).read_text()) for n in REQUIRED}


def values(r: dict) -> dict:
    """Every threshold value with the JSON path it came from."""
    h1s, h1r, h2, h3, h4 = r["h1s"], r["h1r"], r["h2"], r["h3"], r["h4"]
    return {
        "H1-S1": (h1s["H1-S1"], "h1s.json:H1-S1"),
        "H1-S2": (h1s["H1-S2"], "h1s.json:H1-S2"),
        "H1-S3": (h1s["H1-S3"], "h1s.json:H1-S3"),
        "H1-S4": (h1s["H1-S4"], "h1s.json:H1-S4"),
        "H1-R1": (h1r["H1-R1"], "h1r.json:H1-R1"),
        "H1-R2": (h1r["H1-R2"], "h1r.json:H1-R2"),
        "H2-1": (h2["H2-1"], "h2.json:H2-1"),
        "H2-2": (h2["H2-2"], "h2.json:H2-2"),
        "H2-3": (h2["H2-3"], "h2.json:H2-3"),
        "H2-4": (h2["H2-4"], "h2.json:H2-4"),
        "H2-5": (h2["H2-5"], "h2.json:H2-5"),
        "H3-1": (h3["H3-1"]["p95_ms"], "h3.json:H3-1.p95_ms"),
        "H3-2": (h3["H3-2"]["lost"] + h3["H3-2"]["gained"], "h3.json:H3-2.lost+gained"),
        "H4-1": (h4["H4-1"]["equal"], "h4.json:H4-1.equal"),
        "H4-2": (None, "VERIFICATION.md (compare_runs.py)"),
    }


def statuses(vals: dict) -> dict:
    out = {}
    for code, _, _, cmp, bound in THRESHOLDS:
        v = vals[code][0]
        out[code] = (
            "doğrulamada" if cmp == "verify" else ("GEÇTİ" if passes(cmp, v, bound) else "KALDI")
        )
    return out


def decision(st: dict) -> list[str]:
    """docs/00_GOAL.md §Karar kuralı, applied mechanically."""
    ok = lambda codes: all(st[c] == "GEÇTİ" for c in codes)
    h1 = ok(["H1-S1", "H1-S2", "H1-S3", "H1-S4", "H1-R1", "H1-R2"])
    h2 = ok(["H2-1", "H2-2", "H2-3", "H2-4", "H2-5"])
    h2_quality = ok(["H2-1", "H2-2", "H2-4"])
    h3 = ok(["H3-1", "H3-2"])
    h4_1 = st["H4-1"] == "GEÇTİ"
    lines = [
        (
            f"- H1: {'geçti' if h1 else 'geçmedi'} · H2: {'geçti' if h2 else 'geçmedi'} · "
            f"H3: {'geçti' if h3 else 'geçmedi'} · H4: H4-1 {'geçti' if h4_1 else 'geçmedi'}, H4-2 doğrulamada ölçülür."
        )
    ]
    if not h4_1:
        return lines + [
            "- **H4 geçmedi → başka hiçbir sonuç geçerli sayılmaz; önce determinizm düzeltilir ve her şey yeniden koşturulur.**"
        ]
    lines.append(
        "- Kural H4 koşulu: aşağıdaki çıkarımlar H4-2'nin doğrulamada (VERIFICATION.md) geçmesine bağlıdır; "
        "H4-2 geçmezse başka hiçbir sonuç geçerli sayılmaz."
    )
    if h1 and h2 and h3:
        lines.append(
            "- **H1 ∧ H2 ∧ H3 (∧ H4) geçti → çekirdek veri modeli (01_SPEC.md §1–7) dondurulur; çekirdek aşamasına geçilir.**"
        )
    if not h1:
        lines.append("- **H1 geçmedi → veri modeli/eşleyici yeniden ele alınır; H2/H3 saklanır.**")
    if h1 and not h2:
        if h2_quality and st["H2-5"] == "GEÇTİ" and st["H2-3"] == "KALDI":
            lines.append(
                "- **Kısmi H2: H2 kalite geçti, gecikme kaldı** → gecikme bir sonraki tura mühendislik borcu; "
                "çekirdek aşamasına geçilir, graf arama 'kanıtlanmış, optimize edilmemiş' etiketi alır."
            )
        else:
            lines.append(
                "- **H1 geçti, H2 geçmedi → çekirdek 'versiyonlu md çekirdeği' olarak ilerler; graf arama araştırma "
                "kolu olarak ayrılır, çekirdeğe bağlanmaz.**"
            )
    if not h3:
        lines.append("- **H3 geçmedi → sıcak katman yeniden ele alınır, mimari değişmez.**")
    return lines


def fmt(x, nd=4) -> str:
    if isinstance(x, bool):
        return "eşit" if x else "eşit değil"
    if isinstance(x, int):
        return str(x)
    if isinstance(x, float):
        return f"{x:.{nd}f}"
    return str(x)


def table(header: list[str], rows: list[list]) -> str:
    out = ["| " + " | ".join(header) + " |", "|" + "---|" * len(header)]
    out += [
        "| " + " | ".join(fmt(c) if not isinstance(c, str) else c for c in row) + " |"
        for row in rows
    ]
    return "\n".join(out)


def assumptions_by_hypothesis() -> str:
    text = (PROGRESS / "ASSUMPTIONS.md").read_text(encoding="utf-8")
    groups: dict[str, list[str]] = {}
    for line in text.splitlines():
        m = re.match(r"^- ((?:\[[^\]]+\])+) (.*)$", line)
        if m:
            for tag in re.findall(r"\[([^\]]+)\]", m.group(1)):
                groups.setdefault(tag, []).append(m.group(2))
    out = []
    for tag in ("H1", "H2", "H3", "H4", "-"):
        items = groups.get(tag, [])
        out.append(f"**[{tag}]** ({len(items)} madde)")
        out += [f"- {x}" for x in items]
    return "\n".join(out)


def parked() -> str:
    hits = []
    for p in sorted(PROGRESS.glob("T*.md")):
        if p.name.endswith(".review.md"):
            continue
        for line in p.read_text(encoding="utf-8").splitlines():
            if line.startswith("PARK:"):
                hits.append(f"- {p.stem}: {line}")
    stop = PROGRESS / "STOP.md"
    if stop.exists():
        hits.append(f"- STOP.md: {stop.read_text(encoding='utf-8').strip()}")
    return "Park edilen görev yok." if not hits else "\n".join(hits)


def commentary() -> str:
    if not COMMENTARY.exists():
        raise FileNotFoundError(f"{COMMENTARY} missing")
    text = COMMENTARY.read_text(encoding="utf-8").strip()
    if re.search(r"\d", text):
        raise ValueError("reports/commentary.md must not contain numbers")
    return text


def figures(r: dict, figdir: Path) -> None:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    figdir.mkdir(parents=True, exist_ok=True)
    grid = r["h1s_tuning"]["grid"]
    ths = sorted({g["theta"] for g in grid})
    tss = sorted({g["theta_split"] for g in grid})
    z = np.array(
        [
            [
                next(g["objective"] for g in grid if g["theta"] == t and g["theta_split"] == s)
                for t in ths
            ]
            for s in tss
        ]
    )
    fig, ax = plt.subplots(figsize=(6, 3.2))
    im = ax.imshow(z, cmap="viridis", aspect="auto")
    ax.set_xticks(range(len(ths)), [f"{t:.2f}" for t in ths])
    ax.set_yticks(range(len(tss)), [f"{s:.2f}" for s in tss])
    ax.set_xlabel("THETA")
    ax.set_ylabel("THETA_SPLIT")
    ax.set_title("H1-S tuning objective (S1 + (1−S2) + S3 + S4)")
    fig.colorbar(im)
    fig.tight_layout()
    fig.savefig(figdir / "h1s_grid.png", dpi=120)
    plt.close(fig)

    curve = r["h2_tuning"]["curve"]
    fig, ax = plt.subplots(figsize=(6, 3.2))
    for key, label in (("ndcg_all", "all"), ("ndcg_single", "single"), ("ndcg_multi", "multi")):
        ax.plot([c["alpha"] for c in curve], [c[key] for c in curve], marker="o", label=label)
    ax.axvline(r["h2_tuning"]["selected_alpha"], color="grey", linestyle="--")
    ax.set_xlabel("ALPHA")
    ax.set_ylabel("nDCG@10 (tuning)")
    ax.set_title("graph_hybrid / structural ALPHA sensitivity")
    ax.legend()
    fig.tight_layout()
    fig.savefig(figdir / "h2_alpha.png", dpi=120)
    plt.close(fig)

    cells = r["h2"]["cells"]
    strategies = ["flat", "hybrid", "graph_flat", "graph_hybrid", "hier"]
    groups = [
        ("structural", "single"),
        ("structural", "multi"),
        ("fixed", "single"),
        ("fixed", "multi"),
    ]
    fig, ax = plt.subplots(figsize=(8, 3.6))
    w = 0.16
    for i, s in enumerate(strategies):
        ax.bar(
            [g + i * w for g in range(len(groups))],
            [cells[f"{s}/{c}/{t}"]["ndcg@10"] for c, t in groups],
            w,
            label=s,
        )
    ax.set_xticks([g + 2 * w for g in range(len(groups))], [f"{c}\n{t}" for c, t in groups])
    ax.set_ylabel("nDCG@10 (test)")
    ax.legend(fontsize=8)
    fig.tight_layout()
    fig.savefig(figdir / "h2_ndcg.png", dpi=120)
    plt.close(fig)


def build(run: str) -> str:
    r = load_run(run)
    vals = values(r)
    st = statuses(vals)
    man = r["manifest"]
    phases = man["phases"]
    shas = sorted({p["git_sha"] for p in phases.values()})
    dirty = "temiz" if all(p["worktree_clean"] for p in phases.values()) else "KİRLİ"
    any_phase = phases[min(phases)]
    data_sets = {json.dumps(p["data_manifests"], sort_keys=True) for p in phases.values()}
    if len(data_sets) != 1 or json.loads(data_sets.pop()) != runinfo.data_manifests():
        raise RuntimeError(
            "data manifests differ between run phases or from the committed manifests"
        )
    if len({json.dumps(p["versions"], sort_keys=True) for p in phases.values()}) != 1:
        raise RuntimeError("toolchain versions differ between run phases")
    d2 = json.loads((DATA / "MANIFEST.d2.json").read_text())
    d3 = json.loads((DATA / "MANIFEST.d3.json").read_text())
    h1s, h1t, h1r, h2, h2t, h3, h4 = (
        r["h1s"],
        r["h1s_tuning"],
        r["h1r"],
        r["h2"],
        r["h2_tuning"],
        r["h3"],
        r["h4"],
    )

    summary = table(
        ["Kod", "Ölçüm", "Eşik", "Sonuç", "Durum"],
        [
            [c, m, t, "—" if vals[c][0] is None else fmt(vals[c][0], 6), st[c]]
            for c, m, t, _, _ in THRESHOLDS
        ],
    )
    summary += "\n\nKaynak: " + " · ".join(f"{c} ← `{vals[c][1]}`" for c, *_ in THRESHOLDS)

    grid = table(
        ["THETA", "THETA_SPLIT", "S1", "S2", "S3", "S4", "Amaç"],
        [
            [
                g["theta"],
                g["theta_split"],
                g["H1-S1"],
                g["H1-S2"],
                g["H1-S3"],
                g["H1-S4"],
                g["objective"],
            ]
            for g in h1t["grid"]
        ],
    )
    grid += "\n\nKaynak `h1s_tuning.json:grid`."
    probe = h1t.get("sensitivity_probe")
    if h1t.get("grid_flat") and probe:
        grid += (
            f"\n\nIzgara düz (`h1s_tuning.json:grid_flat`). Izgara dışı tanı noktası (seçilemez) THETA={fmt(probe['theta'], 2)}, "
            f"THETA_SPLIT={fmt(probe['theta_split'], 2)}: S1 {fmt(probe['H1-S1'])}, S2 {fmt(probe['H1-S2'])}, "
            f"S3 {fmt(probe['H1-S3'])}, S4 {fmt(probe['H1-S4'])} (`h1s_tuning.json:sensitivity_probe`)."
        )
    per_op = table(
        ["Op", "Adım", "Koruma", "Yanlış eşleme", "Olay recall"],
        [
            [
                op,
                v["steps"],
                v.get("retention", "—"),
                v.get("false_match", "—"),
                v.get("event_recall", "—"),
            ]
            for op, v in h1s["per_op"].items()
        ],
    )
    det = h1s["H1-S3_detail"]
    per_op += (
        f"\n\nS3 ayrıntı: P {fmt(det['precision'])}, R {fmt(det['recall'])}, doğru {det['tp']} / tahmin {det['n_pred']} / gerçek {det['n_truth']}. "
        f"n: S1 {h1s['n_s1']}, S2 {h1s['n_s2']}, S4 {h1s['n_s4']}; {h1s['n_docs']} doküman, {h1s['n_steps']} adım. Kaynak `h1s.json:per_op`, `h1s.json:H1-S3_detail`."
    )

    curve = table(
        ["ALPHA", "nDCG tümü", "single", "multi"],
        [[c["alpha"], c["ndcg_all"], c["ndcg_single"], c["ndcg_multi"]] for c in h2t["curve"]],
    )
    strategies = ["flat", "hybrid", "graph_flat", "graph_hybrid", "hier"]
    ndcg = table(
        ["Strateji", "structural single", "structural multi", "fixed single", "fixed multi"],
        [
            [s]
            + [
                h2["cells"][f"{s}/{c}/{t}"]["ndcg@10"]
                for c in ("structural", "fixed")
                for t in ("single", "multi")
            ]
            for s in strategies
        ],
    )
    ndcg += f"\n\nKaynak `h2.json:cells.<strateji>/<chunk>/<tür>.ndcg@10`; sorgu sayısı single {h2['n_questions']['single']}, multi {h2['n_questions']['multi']}."
    rec = table(
        ["Strateji", "Tür", "Recall@5", "Recall@10", "MRR", "full_cover@10"],
        [
            [s, t]
            + [
                h2["cells"][f"{s}/structural/{t}"][m]
                for m in ("recall@5", "recall@10", "mrr", "full_cover@10")
            ]
            for s in strategies
            for t in ("single", "multi")
        ],
    )
    rec += "\n\nKaynak `h2.json:cells.<strateji>/structural/<tür>`."
    lat = table(
        ["Strateji", "structural p50", "structural p95", "fixed p50", "fixed p95"],
        [
            [
                s,
                h2["latency_ms"][f"{s}/structural"]["p50"],
                h2["latency_ms"][f"{s}/structural"]["p95"],
                h2["latency_ms"][f"{s}/fixed"]["p50"],
                h2["latency_ms"][f"{s}/fixed"]["p95"],
            ]
            for s in strategies
        ],
    )
    lat += "\n\nKaynak `h2.json:latency_ms`."

    def ci(key):
        b = h2["bootstrap"][key]
        return f"Δ {fmt(b['delta'])} [{fmt(b['ci95'][0])}, {fmt(b['ci95'][1])}] (n={b['n']})"

    lossc = ", ".join(f"{k} {v}" for k, v in h1r["loss_categories"].items())
    other = (
        "; ".join(f"{e['doc']}@{e['commit'][:7]}:{e['version_id']}" for e in h1r["other_examples"])
        or "yok"
    )
    fe = (PROGRESS / "FREEZE_EXCEPTIONS.md").read_text(encoding="utf-8").strip()
    lines = [
        "# mdkernel-v2 — Nihai Rapor",
        "",
        (
            f"RUN: {run} · Commit: {', '.join(s[:12] for s in shas)} ({dirty}) · Rust {any_phase['versions']['rust']} · "
            f"Python {any_phase['versions']['python']} · SEED {man['seed']}"
        ),
        (
            f"Veri: D2 {d2['rev'][:12]} (MANIFEST {any_phase['data_manifests']['MANIFEST.d2.json']}) · D3 {d3['rev'][:12]} (events {d3['events_hash']}) · "
            f"D1 (MANIFEST {any_phase['data_manifests']['MANIFEST.d1.json']}) · D4 single {h2['n_questions']['single']} / multi {h2['n_questions']['multi']} (test)"
        ),
        "",
        f"> `eval/report.py` tarafından yalnız `reports/runs/{run}/` içinden üretildi. Elle sayı yoktur. Her tablo altında kaynak alan yolları.",
        "",
        "## 1. Özet",
        summary,
        "",
        "## 2. Karar kuralı çıktısı",
        *decision(st),
        "",
        "## 3. H1",
        "### 3.1 THETA × THETA_SPLIT ızgarası (ayar)",
        grid,
        "",
        f"Seçilen: THETA={fmt(h1t['selected']['theta'], 2)}, THETA_SPLIT={fmt(h1t['selected']['theta_split'], 2)} · Şekil `figs/h1s_grid.png`",
        "### 3.2 Test, op bazında",
        per_op,
        "### 3.3 Gerçek tarihçe",
        f"Depo {h1r['repo']}@{h1r['rev'][:12]} · commit {h1r['commits']} · çift {h1r['pairs']} · blok {h1r['blocks']} · dokunulmamış {h1r['untouched']}",
        "",
        f"R1 {fmt(h1r['H1-R1'], 6)} · R2 {fmt(h1r['H1-R2'], 6)} · hunk içi (bilgi) {fmt(h1r['touched_retention'], 6)}",
        "",
        f"Kayıp kategorileri: {lossc} · `other` örnekleri: {other} (Kaynak `h1r.json`)",
        "",
        "## 4. H2",
        f"ALPHA seçilen {h2t['selected_alpha']} (kural: {h2t['rule']}) · Duyarlılık eğrisi (ayar, {h2t['n_questions']} soru):",
        "",
        curve,
        "",
        "`figs/h2_alpha.png` · Kaynak `h2_tuning.json:curve`",
        "### 4.1 nDCG@10",
        ndcg,
        "### 4.2 Recall@5/10, MRR, full_cover@10 (structural)",
        rec,
        "### 4.3 Gecikme ms (p50/p95, sorgu embedding'i dahil)",
        lat,
        f"### 4.4 Bootstrap %95 GA (structural, eşleştirilmiş, {h2['bootstrap']['graph_hybrid-hybrid/multi/ndcg@10']['samples']} örnek)",
        f"graph_hybrid−hybrid nDCG: single {ci('graph_hybrid-hybrid/single/ndcg@10')} · multi {ci('graph_hybrid-hybrid/multi/ndcg@10')}",
        "",
        f"graph_hybrid−hybrid full_cover: multi {ci('graph_hybrid-hybrid/multi/full_cover@10')}",
        "",
        f"graph_flat−flat nDCG: single {ci('graph_flat-flat/single/ndcg@10')} · multi {ci('graph_flat-flat/multi/ndcg@10')}",
        "",
        "`figs/h2_ndcg.png` · Kaynak `h2.json:bootstrap`",
        "",
        "## 5. H3",
        (
            f"p50 {fmt(h3['H3-1']['p50_ms'], 2)} ms · p95 {fmt(h3['H3-1']['p95_ms'], 2)} ms (N={h3['H3-1']['n']}) · "
            f"yeniden-chunk kayıp {h3['H3-2']['lost']}/{h3['H3-2']['gained']} · olay hash eşit {fmt(h3['rechunk']['log_equal'])} · "
            f"hit@1 {fmt(h3['latency']['hit_at_1_rate'], 3)}, hit@10 {fmt(h3['latency']['hit_at_10_rate'], 3)} (bilgi) · Kaynak `h3.json`"
        ),
        "",
        "## 6. H4",
        f"H4-1: manifest {h4['H4-1']['manifest_hash']} vs yeniden kurulum {h4['H4-1']['rebuilt_hash']} → {fmt(h4['H4-1']['equal'])} (Kaynak `h4.json`)",
        "",
        "H4-2: doğrulama aşamasında ölçülür (`VERIFICATION.md`).",
        "",
        "## 7. Varsayımlar, park, dondurma istisnaları",
        assumptions_by_hypothesis(),
        "",
        "**Park:** " + parked(),
        "",
        "**Dondurma istisnaları:**",
        "",
        fe,
        "",
        "## 8. Serbest yorum (ajan; sayı içermez)",
        commentary(),
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    text = build(a.run)
    d = runinfo.run_dir(a.run, create=False)
    r = load_run(a.run)
    figures(r, d / "figs")
    (d / "FINAL.md").write_text(text, encoding="utf-8")
    out = ROOT / "reports"
    (out / "FINAL.md").write_text(text, encoding="utf-8")
    if (out / "figs").exists():
        shutil.rmtree(out / "figs")
    shutil.copytree(d / "figs", out / "figs")
    print(f"wrote reports/FINAL.md and reports/runs/{a.run}/FINAL.md")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
