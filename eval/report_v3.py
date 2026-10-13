"""`make report-v3 RUN=<run_id>`: builds the v3 FINAL.md from one run directory only.

Reads exactly `reports/runs/<RUN>/{manifest,h1s_v3_tuning,h1s_v3,h1r_v3,h3_v3,
h4_v3,h4_v2repro}.json` (any missing file is an error; nothing is searched
for, nothing falls back), plus the committed data manifests,
`docs/progress/v3/ASSUMPTIONS.md`, `docs/progress/FREEZE_EXCEPTIONS.md`, v3
progress records (PARK lines) and the number-free commentary
`reports/v3/commentary.md`. Layout follows `docs/v3/05_REPORT_TEMPLATE.md`;
thresholds are coded verbatim from `docs/v3/00_GOAL.md`. Writes
`reports/runs/<RUN>/FINAL.md` and copies it to `reports/v3/FINAL.md`.

    python -m eval.report_v3 --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import re

from eval import runinfo
from eval.datasets.common import DATA, ROOT, die

REQUIRED = [
    "manifest.json",
    "h1s_v3_tuning.json",
    "h1s_v3.json",
    "h1r_v3.json",
    "h3_v3.json",
    "h4_v3.json",
    "h4_v2repro.json",
]
PROGRESS = ROOT / "docs" / "progress" / "v3"
FREEZE_EXCEPTIONS = ROOT / "docs" / "progress" / "FREEZE_EXCEPTIONS.md"
OUT = ROOT / "reports" / "v3"
COMMENTARY = OUT / "commentary.md"

# Pre-registered thresholds (docs/v3/00_GOAL.md); (code, measurement, set, threshold text, comparator, bound)
THRESHOLDS = [
    ("H1-S1", "Soy koruma (rephrase_*, move)", "D1-easy (K4)", "≥ 0.95", ">=", 0.95),
    ("H1-S2", "Yanlış eşleme (K4 tanımı)", "D1-easy", "≤ 0.02", "<=", 0.02),
    ("H1-S3", "Bölme/birleştirme olay F1", "D1-easy", "≥ 0.80", ">=", 0.80),
    ("H1-S4", "Tekrar-blok koruma (dup_delete)", "D1-easy", "≥ 0.95", ">=", 0.95),
    (
        "H1-H1",
        "Soy koruma (rephrase_*, move, split_edit, merge_edit baskın)",
        "D1-hard",
        "≥ 0.90",
        ">=",
        0.90,
    ),
    ("H1-H2", "Yanlış eşleme", "D1-hard", "≤ 0.03", "<=", 0.03),
    ("H1-H3", "Bölme/birleştirme olay F1", "D1-hard", "≥ 0.70", ">=", 0.70),
    ("H1-H4", "Tekrar-blok koruma (dup_delete_adjacent)", "D1-hard", "≥ 0.90", ">=", 0.90),
    ("H1-R1", "Gerçek tarihçe, dokunulmamış blok koruma", "D2", "≥ 0.999", ">=", 0.999),
    ("H1-R2", "Gerçek tarihçe, genel koruma", "D2", "≥ 0.85", ">=", 0.85),
    ("H1-G", "Izgarada farklı amaç değeri sayısı (geçerlilik)", "D1-hard ayar", "≥ 3", ">=", 3),
    ("H3-1", "Yazma→aranabilir p95 (ms)", "D3", "< 1000", "<", 1000.0),
    ("H3-2", "Yeniden-chunk soy kaybı (kayıp + kazanç)", "D3", "= 0", "==", 0),
    (
        "H4-1",
        "Olay günlüğü hash, D3 yeniden kurulum, eşleyici v3",
        "D3",
        "= manifest (v3)",
        "is",
        True,
    ),
    (
        "H4-V2",
        "Eşleyici v2 bayrağıyla D3 hash ve v2 h1s.json kalite alanları",
        "D3, D1-v2",
        "= eşit",
        "is",
        True,
    ),
    (
        "H4-2",
        "İki temiz koşturmada kalite metrikleri maks. göreli fark",
        "tümü",
        "= 0",
        "verify",
        None,
    ),
]
EASY = ("H1-S1", "H1-S2", "H1-S3", "H1-S4")
HARD = ("H1-H1", "H1-H2", "H1-H3", "H1-H4")
REAL = ("H1-R1", "H1-R2")


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
    h1s, h1t, h1r, h3, h4, v2r = (
        r["h1s_v3"],
        r["h1s_v3_tuning"],
        r["h1r_v3"],
        r["h3_v3"],
        r["h4_v3"],
        r["h4_v2repro"],
    )
    if h1s["H1-G"] != h1t["H1-G"]:
        raise RuntimeError("h1s_v3.json:H1-G differs from h1s_v3_tuning.json:H1-G")
    out = {c: (h1s[c], f"h1s_v3.json:{c}") for c in EASY + HARD}
    out.update({c: (h1r[c], f"h1r_v3.json:{c}") for c in REAL})
    out["H1-G"] = (h1t["H1-G"], "h1s_v3_tuning.json:H1-G")
    out["H3-1"] = (h3["H3-1"]["p95_ms"], "h3_v3.json:H3-1.p95_ms")
    out["H3-2"] = (h3["H3-2"]["lost"] + h3["H3-2"]["gained"], "h3_v3.json:H3-2.lost+gained")
    out["H4-1"] = (h4["H4-1"]["equal"], "h4_v3.json:H4-1.equal")
    out["H4-V2"] = (v2r["H4-V2"]["equal"], "h4_v2repro.json:H4-V2.equal")
    out["H4-2"] = (None, "scripts/verify.sh pair (compare_runs.py)")
    return out


def statuses(vals: dict) -> dict:
    out = {}
    for code, _, _, _, cmp, bound in THRESHOLDS:
        v = vals[code][0]
        out[code] = (
            "doğrulamada" if cmp == "verify" else ("GEÇTİ" if passes(cmp, v, bound) else "KALDI")
        )
    return out


def decision(st: dict) -> list[str]:
    """docs/v3/00_GOAL.md §Karar kuralı, applied mechanically."""
    ok = lambda codes: all(st[c] == "GEÇTİ" for c in codes)
    g = st["H1-G"] == "GEÇTİ"
    h1_codes = EASY + REAL + (HARD if g else ())
    failing = [c for c in h1_codes if st[c] != "GEÇTİ"]
    h3 = ok(["H3-1", "H3-2"])
    lines = [
        (
            f"- H1-easy: {'geçti' if ok(EASY) else 'geçmedi'} · H1-hard: {'geçti' if ok(HARD) else 'geçmedi'}"
            f" ({'kanıt' if g else 'bilgi — H1-G geçmedi'}) · H1-real: {'geçti' if ok(REAL) else 'geçmedi'} · "
            f"H3: {'geçti' if h3 else 'geçmedi'} · H4-1: {st['H4-1']} · H4-V2: {st['H4-V2']} · H4-2 doğrulamada ölçülür."
        )
    ]
    if st["H4-V2"] != "GEÇTİ":
        return lines + [
            "- **H4-V2 geçmedi → v2 davranışı korunamamış; sonuçlar geçersiz, önce bu düzeltilir.**"
        ]
    if st["H4-1"] != "GEÇTİ":
        return lines + ["- **H4-1 geçmedi → determinizm bozuldu; sonuçlar geçersiz.**"]
    lines.append(
        "- Kural H4 koşulu: aşağıdaki çıkarımlar H4-2'nin doğrulamada geçmesine bağlıdır; "
        "H4-2 geçmezse determinizm bozulmuştur ve sonuçlar geçersizdir."
    )
    if not g:
        lines.append(
            "- **H1-G geçmedi → H1-H* bilgi amaçlıdır; karar yalnız H1-S* ve H1-R* ile verilir. "
            "Not: zor set kanıtı eksik.**"
        )
    if not failing and h3:
        lines.append(
            "- **H1-S* ∧ H1-R* ∧ "
            + ("H1-H* " if g else "(H1-G geçmediği için H1-H* hariç) ")
            + "∧ H3 ∧ H4 geçti → `01_SPEC.md` §1–7 (K4 ve v3 eşleyiciyle) dondurulur; çekirdek fazı 1 başlar.**"
        )
    elif not failing:
        lines.append(
            "- H1 geçti ancak H3 geçmedi → karar kuralının dondurma koşulu (H3) sağlanmadı; "
            "`01_SPEC.md` dondurulmaz, H3 regresyonu ele alınır."
        )
    elif set(failing) <= {"H1-S2", "H1-H2"}:
        lines.append(
            f"- **{' ve '.join(failing)} kaldı, diğerleri geçti → eşleyici bir tur daha; diğer sonuçlar saklanır.**"
        )
    else:
        lines.append(
            f"- **H1 alt ölçümü kaldı ({', '.join(failing)}) → veri modeli yeniden ele alınır.**"
        )
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
    for tag in ("H1", "H3", "H4", "-"):
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
        raise ValueError("reports/v3/commentary.md must not contain numbers")
    return text


def metrics_table(m: dict, codes: tuple, label: str) -> str:
    rows = [[c, m[c]] for c in codes]
    return table(["Kod", label], rows)


def per_op_table(m: dict) -> str:
    rows = []
    for op, row in m["per_op"].items():
        rows.append(
            [
                op,
                row["steps"],
                row.get("retention", "—"),
                row.get("false_match", "—"),
                row.get("event_recall", "—"),
            ]
        )
    return table(["Op", "Adım", "Koruma", "Yanlış eşleme", "Olay recall"], rows)


def build(run: str) -> str:
    r = load_run(run)
    vals = values(r)
    st = statuses(vals)
    man = r["manifest"]
    phases = man["phases"]
    v3_phases = {
        k: p
        for k, p in phases.items()
        if k in ("h1s_v3_tune", "h1s_v3_test", "h1r_v3", "h3_v3", "h4_v3", "h4_v2repro")
    }
    if len(v3_phases) != 6:
        raise RuntimeError(f"run {run}: v3 phases missing from manifest.json: {sorted(v3_phases)}")
    shas = sorted({p["git_sha"] for p in v3_phases.values()})
    dirty = "temiz" if all(p["worktree_clean"] for p in v3_phases.values()) else "KİRLİ"
    any_phase = v3_phases["h1s_v3_test"]
    data_sets = {json.dumps(p["data_manifests"], sort_keys=True) for p in v3_phases.values()}
    if len(data_sets) != 1 or json.loads(data_sets.pop()) != runinfo.data_manifests():
        raise RuntimeError(
            "data manifests differ between run phases or from the committed manifests"
        )
    if len({json.dumps(p["versions"], sort_keys=True) for p in v3_phases.values()}) != 1:
        raise RuntimeError("toolchain versions differ between run phases")
    dm = any_phase["data_manifests"]
    d2 = json.loads((DATA / "MANIFEST.d2.json").read_text())
    d3 = json.loads((DATA / "MANIFEST.d3.json").read_text())
    h1s, h1t, h1r, h3, h4, v2r = (
        r["h1s_v3"],
        r["h1s_v3_tuning"],
        r["h1r_v3"],
        r["h3_v3"],
        r["h4_v3"],
        r["h4_v2repro"],
    )
    easy, hard = h1s["easy"], h1s["hard"]

    summary = table(
        ["Kod", "Ölçüm", "Küme", "Eşik", "Sonuç", "Durum"],
        [
            [c, m, k, t, "—" if vals[c][0] is None else fmt(vals[c][0], 6), st[c]]
            for c, m, k, t, _, _ in THRESHOLDS
        ],
    )
    summary += "\n\nKaynak: " + " · ".join(f"{c} ← `{vals[c][1]}`" for c, *_ in THRESHOLDS)

    grid = table(
        ["THETA", "THETA_SPLIT", "H1", "H2", "H3", "H4", "Amaç"],
        [
            [
                g["theta"],
                g["theta_split"],
                g["H1-H1"],
                g["H1-H2"],
                g["H1-H3"],
                g["H1-H4"],
                g["objective"],
            ]
            for g in h1t["grid"]
        ],
    )
    s3e, s3h = easy["H1-S3_detail"], hard["H1-H3_detail"]
    v2h = v2r["H4-V2"]
    fe = FREEZE_EXCEPTIONS.read_text(encoding="utf-8").strip()
    lines = [
        "# mdkernel-v3 — Nihai Rapor",
        "",
        (
            f"RUN: {run} · Commit: {', '.join(s[:12] for s in shas)} ({dirty}) · Rust {any_phase['versions']['rust']} · "
            f"Python {any_phase['versions']['python']} · SEED {man['seed']}"
        ),
        (
            f"Veri: D2 {d2['rev'][:12]} (MANIFEST {dm['MANIFEST.d2.json']}) · D3 {d3['rev'][:12]} "
            f"(v2 {d3['events_hash']} / v3 {d3['events_hash_v3']}) · D1 (MANIFEST {dm['MANIFEST.d1.json']}) · "
            f"D1-easy(K4) (MANIFEST {dm['MANIFEST.d1k4.json']}) · D1-hard (MANIFEST {dm['MANIFEST.d1h.json']})"
        ),
        "",
        "> `eval/report_v3.py` tarafından yalnız `reports/runs/"
        + run
        + "/` içinden üretildi. Elle sayı yoktur. "
        "Her tablo altında kaynak alan yolları. Eşikler `docs/v3/00_GOAL.md`'den aynen kodlanmıştır.",
        "",
        (
            "> **K4 tanım değişikliği (sonuç görüldükten sonra alınmış tasarım kararı):** v3'te bölme/birleştirmede baskın "
            "parça soyu korur (`docs/ARCHITECTURE.md` §10 K4). Bu karar v2 sonuçları görüldükten sonra alınmıştır ve gerçek "
            "tanımını değiştirir; H1-S2/H1-H2 yanlış eşleme paydası buna göre yeniden yazılmıştır (merge sonucu paydada değildir). "
            "v2 tanımlı S2 bilgi amaçlı ayrıca raporlanır; v2 davranışının bayrakla birebir korunduğu H4-V2 ile gösterilir."
        ),
        "",
        "## 1. Özet",
        summary,
        "",
        "## 2. Karar kuralı çıktısı",
        *decision(st),
        "",
        "## 3. H1-easy — D1-easy (K4 gerçek)",
        (
            f"Test bölünmesi: {easy['n_docs']} doküman, {easy['n_steps']} puanlı adım · eşleyici {h1s['matcher']} "
            f"(THETA={h1s['theta']}, THETA_SPLIT={h1s['theta_split']})"
        ),
        "",
        metrics_table(easy, EASY, "Değer"),
        "",
        (
            f"Olay F1 ayrıntısı: P {fmt(s3e['precision'])} · R {fmt(s3e['recall'])} · tp {s3e['tp']} / tahmin {s3e['n_pred']} / gerçek {s3e['n_truth']} · "
            f"payda S1 {easy['n_retention']}, S2 {easy['n_false_match']}, S4 {easy['n_dup']}"
        ),
        "",
        f"v2 tanımlı S2 (bilgi): {fmt(easy['H1-S2_v2def'], 6)} (payda {easy['n_false_match_v2def']})",
        "",
        per_op_table(easy),
        "",
        "Kaynak `h1s_v3.json:easy`",
        "",
        "## 4. H1-hard — D1-hard",
        "### 4.1 THETA × THETA_SPLIT ızgarası (ayar, D1-hard ayar bölünmesi)",
        f"Ayar: {len(h1t['tune_ids'])} doküman · kural: {h1t['rule']}",
        "",
        grid,
        "",
        (
            f"Seçilen: THETA={h1t['selected']['theta']}, THETA_SPLIT={h1t['selected']['theta_split']} · "
            f"H1-G (farklı amaç değeri) {h1t['H1-G']} · Kaynak `h1s_v3_tuning.json`"
        ),
        "### 4.2 Test",
        f"Test bölünmesi: {hard['n_docs']} doküman, {hard['n_steps']} puanlı adım",
        "",
        metrics_table(hard, HARD, "Değer"),
        "",
        (
            f"Olay F1 ayrıntısı: P {fmt(s3h['precision'])} · R {fmt(s3h['recall'])} · tp {s3h['tp']} / tahmin {s3h['n_pred']} / gerçek {s3h['n_truth']} · "
            f"payda H1 {hard['n_retention']}, H2 {hard['n_false_match']}, H4 {hard['n_dup']}"
        ),
        "",
        f"v2 tanımlı S2 (bilgi): {fmt(hard['H1-H2_v2def'], 6)} (payda {hard['n_false_match_v2def']})",
        "",
        per_op_table(hard),
        "",
        "Kaynak `h1s_v3.json:hard`",
        "",
        "## 5. H1-real — D2 gerçek tarihçe (eşleyici v3)",
        (
            f"Depo {h1r['repo']}@{h1r['rev'][:12]} · commit {h1r['commits']} · çift {h1r['pairs']} · blok {h1r['blocks']} · "
            f"dokunulmamış {h1r['untouched']}"
        ),
        "",
        f"R1 {fmt(h1r['H1-R1'], 6)} · R2 {fmt(h1r['H1-R2'], 6)} · hunk içi (bilgi) {fmt(h1r['touched_retention'], 6)}",
        "",
        "Kayıp kategorileri: "
        + ", ".join(f"{k} {v}" for k, v in h1r["loss_categories"].items())
        + f" · `other` sayısı {h1r['other_count']} · Kaynak `h1r_v3.json`",
        "",
        "## 6. H3 (eşleyici v3)",
        (
            f"p50 {fmt(h3['H3-1']['p50_ms'], 2)} ms · p95 {fmt(h3['H3-1']['p95_ms'], 2)} ms (N={h3['H3-1']['n']}) · "
            f"yeniden-chunk kayıp {h3['H3-2']['lost']}/{h3['H3-2']['gained']} · olay hash eşit {fmt(h3['rechunk']['log_equal'])} · "
            f"hit@1 {fmt(h3['latency']['hit_at_1_rate'], 3)}, hit@10 {fmt(h3['latency']['hit_at_10_rate'], 3)} (bilgi) · Kaynak `h3_v3.json`"
        ),
        "",
        "## 7. H4",
        (
            f"H4-1 (v3): manifest `events_hash_v3` {h4['H4-1']['manifest_hash']} vs yeniden kurulum "
            f"{h4['H4-1']['rebuilt_hash']} → {fmt(h4['H4-1']['equal'])} · v2 günlüğü (aynı yeniden kurulum) "
            f"{h4['v2_log']['rebuilt_hash']} → {fmt(h4['v2_log']['equal'])} · Kaynak `h4_v3.json`"
        ),
        "### 7.1 H4-V2 — v2 davranışının yeniden üretimi",
        (
            f"D3 `--matcher v2` `events_hash`: {v2h['d3']['events_hash']} (beklenen {v2h['d3']['expected']}) → "
            f"{fmt(v2h['d3']['equal'])}"
        ),
        "",
        (
            f"v2 D1, v2 metrikleri, `{v2h['h1s']['reference']}` ile alan alan: {v2h['h1s']['n_fields']} alan, "
            f"farklı {v2h['h1s']['n_differing']} → {fmt(v2h['h1s']['equal'])} · Kaynak `h4_v2repro.json`"
        ),
        "",
        "H4-2: doğrulama aşamasında ölçülür (`scripts/verify.sh pair`).",
        "",
        "## 8. Varsayımlar, park, dondurma istisnaları",
        assumptions_by_hypothesis(),
        "",
        "**Park:** " + parked(),
        "",
        "**Dondurma istisnaları:**",
        "",
        fe,
        "",
        "## 9. Serbest yorum (ajan; sayı içermez)",
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
    (d / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "FINAL.md").write_text(text, encoding="utf-8")
    print(f"wrote reports/v3/FINAL.md and reports/runs/{a.run}/FINAL.md")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
