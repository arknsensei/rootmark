"""P2-0.3 round report (TASKS R2): `FINAL.md` from the four chains' RUN
directories — the unloaded pair (`make verify`; chain A is the official RUN)
and the loaded pair (`VERIFY_LOAD=1 make verify`) — in the form of
`eval.report_p2_0_2` (its table and helpers, unchanged).

Every number is read from a file (named next to it): the GOAL P2-0.3
hypothesis table H-P2-0.3a..j from the official RUN's `p2_0_3_checks.json`
(`eval.run_p2_0_3 checks`), H-P2-0.3b-yük from both loaded chains',
H-P2-0.3i from both pairs' `pair_compare.json` (`eval.compare_p2a pair`);
D-B test-4 in the four chains, by session kind and by link format (b′),
beside P2-0.2's official RUN (test-3) and P2-A's information measurement
(test-3 after the (A) fix); and the decision rule of
docs/rounds/P2-0.3/GOAL.md, applied in this order:

- a chain measured D-B with a seed other than the pre-registered one → the
  round is invalid (GOAL "Tasarım"); whether the product changed is the
  `git diff` of the round (FINAL), and the D-A, D-A2, D-C logs against the
  P2-A official RUN are reported beside it, not a branch of the rule (a
  timing race in one chain is H-P2-0.3i's, ASSUMPTIONS R0-6);
- a hypothesis not measured (no Obsidian, a missing chain) → the rule waits;
- 0.3a, 0.3a′ or 0.3e failed → a design flaw: the round fails, a decision
  request; P2-L does not start;
- 0.3d, 0.3f, 0.3h, 0.3i or 0.3j failed → an infrastructure/regression
  flaw: the round fails, a decision request;
- 0.3b failed and nothing else outside {0.3b, 0.3b-yük} → `canonical` stays
  the default for good; the P2-0 series closes with its documented bound;
  P2-L starts;
- only 0.3b-yük failed → proven with a documented bound ("under load false
  `unverified` can exceed 2 %"); `unverified` becomes the default; P2-L;
- all pass → proven; with a "güvenilir" independent verification (or its
  conditions closed) `unverified` becomes the default again (the next
  round's opening task), the spec §7.1/§8 stamp, a NORTH_STAR note (a human
  decision); the P2-0 series closes; P2-L starts;
- any other failure (0.3c among them) → undefined failure: the round fails,
  a decision request.

Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P2-0.3/FINAL.md`.

    python -m eval.report_p2_0_3 --run-dir <A> --chain-b <B> \
        --load-a <loaded A> --load-b <loaded B> [--load-pair <pair_compare.json>]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import run_p2_0_2_checks as c2
from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of
from eval.report_p2_0_2 import _f, _load, _ok, _table

REPO = ROOT.parent
OUT = REPO / "reports" / "rounds" / "P2-0.3" / "FINAL.md"
P2_0_2_RUN = REPO / "reports/rounds/P2-0.2/runs/20261019T212951Z-72fd78f"
P2A_INFO = REPO / "reports/rounds/P2-A/info_d_b_test3.json"
IDS = (
    "H-P2-0.3a",
    "H-P2-0.3a′",
    "H-P2-0.3b",
    "H-P2-0.3b-yük",
    "H-P2-0.3b′",
    "H-P2-0.3c",
    "H-P2-0.3d",
    "H-P2-0.3e",
    "H-P2-0.3f",
    "H-P2-0.3h",
    "H-P2-0.3i",
    "H-P2-0.3j",
)
DESIGN = {"H-P2-0.3a", "H-P2-0.3a′", "H-P2-0.3e"}
INFRA = {"H-P2-0.3d", "H-P2-0.3f", "H-P2-0.3h", "H-P2-0.3i", "H-P2-0.3j"}


def _b(c: dict) -> str:
    b = c["H-P2-0.3b"]
    if b.get("skipped"):
        return "atlandı"
    return f"{b['unverified_after_window']}/{b['new_blocks']} = {_f(b['false_unverified_rate'])}"


def _pair(p: dict) -> str:
    g = p["h_p2_0_2i"]
    return f"günlükler eşit {g['equal']}; {p['n_fields']} alan, fark {len(p['differing_fields'])}; yapıtlar bayt eşit {p['bytes_equal']}"


def rows(chains: dict[str, dict], pairs: dict[str, dict]) -> list[dict]:
    c = chains["A"]
    a, b, bp, cc = c["H-P2-0.3a"], c["H-P2-0.3b"], c["H-P2-0.3b′"], c["H-P2-0.3c"]
    dd, e, f = c["H-P2-0.3d"], c["H-P2-0.3e"], c["H-P2-0.3f"]
    dc, dcp = dd["d_c"], dd["d_c_prime"]
    skip_b, skip_dp, skip_e = (x.get("skipped") is True for x in (b, dcp, e))
    loads = [chains["load A"]["H-P2-0.3b-yük"], chains["load B"]["H-P2-0.3b-yük"]]
    load_pass = None if any(x["pass"] is None for x in loads) else all(x["pass"] for x in loads)
    pass_i = {k: p["h_p2_0_2i"]["equal"] and p["equal"] for k, p in pairs.items()}
    h = c["H-P2-0.3h"]
    j = c["H-P2-0.3j"]
    return [
        {
            "id": "H-P2-0.3a",
            "what": "Canonical sızıntısı (D-A, D-A2)",
            "value": f"D-A {a['d_a']['canonical_leak']}/{a['d_a']['new_blocks']} ({a['d_a']['scenarios']} senaryo); D-A2 {a['d_a2']['canonical_leak']}/{a['d_a2']['new_blocks']} ({a['d_a2']['scenarios']} senaryo)",
            "threshold": "= 0",
            "pass": a["pass"],
            "source": "p2_0_2/d_a.json, p2_0_2/d_a2.json",
        },
        {
            "id": "H-P2-0.3a′",
            "what": "Canonical içerik korunumu (D-A)",
            "value": f"{c['H-P2-0.3a′']['scenarios']} senaryo; değişen canonical soy {c['H-P2-0.3a′']['canonical_changed']}",
            "threshold": "200/200",
            "pass": c["H-P2-0.3a′"]["pass"],
            "source": "p2_0_2/d_a.json",
        },
        {
            "id": "H-P2-0.3b",
            "what": "Yanlış `unverified`, W sonunda, yüksüz resmi zincir (D-B test-4, gerçek Obsidian)",
            "value": "atlandı (Obsidian yok)"
            if skip_b
            else f"{_b(c)} ({b['sessions']} oturum, {b['saves']} kayıt, yan not {b['side_notes']}; tohum {b['seed']})",
            "threshold": "≤ 0.01",
            "pass": b["pass"],
            "source": "p2_0_3/d_b.json",
        },
        {
            "id": "H-P2-0.3b-yük",
            "what": "Aynı ölçüm, yüklü çiftin her iki zinciri",
            "value": f"yüklü A {_b(chains['load A'])}; yüklü B {_b(chains['load B'])}",
            "threshold": "≤ 0.02 (ikisi de)",
            "pass": load_pass,
            "source": "load A, load B: p2_0_3/d_b.json",
        },
        {
            "id": "H-P2-0.3b′",
            "what": "Biçim başına ve oturum türüne göre (bilgi); P2-0.2 ile karşılaştırma",
            "value": "atlandı"
            if skip_b
            else f"yeniden adlandırma oturumları {bp['unverified_after_window']}/{bp['new_blocks']} ({bp['records']} kayıt); tablolar aşağıda",
            "threshold": "raporlanır",
            "pass": None if skip_b else "info",
            "source": "p2_0_3/d_b.json (renames)",
        },
        {
            "id": "H-P2-0.3c",
            "what": "Tasdik gecikmesi p95 (D-B test-4)",
            "value": "atlandı"
            if skip_b
            else f"{_f(cc['latency_p95_ms'], 0)} ms ({cc['latency_measured']} ölçüm; `canonical` olmayan {cc['latency_unsettled']}; yazımsız kayıt {cc['no_write']})",
            "threshold": "< 500 ms",
            "pass": cc["pass"],
            "source": "p2_0_3/d_b.json",
        },
        {
            "id": "H-P2-0.3d",
            "what": "Karışık katman doğruluğu (D-C); gerçek Obsidian çakışma (D-C′)",
            "value": f"D-C {dc['right']}/{dc['blocks']} = {_f(dc['accuracy'])} ({dc['ticks']} tik; ajan `canonical` {dc['agent_canonical']}); "
            + (
                "D-C′ atlandı"
                if skip_dp
                else f"D-C′ {dcp['human_canonical']}/{dcp['human_blocks']} = {_f(dcp['human_accuracy'])}, ajan `canonical` {dcp['agent_canonical']} ({dcp['ticks']} tik)"
            ),
            "threshold": "≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0",
            "pass": dd["pass"],
            "source": "p2_0_2/d_c.json, p2_0_3/d_c_prime.json",
        },
        {
            "id": "H-P2-0.3e",
            "what": "Aklama: bağlantı güncellemesi, sahte, katman taşıma (D-A′)",
            "value": "atlandı"
            if skip_e
            else f"ajan `canonical` {e['agent_canonical']}/{e['agent_blocks']} ({e['scenarios']} senaryo); sahte ret {e['fakes_denied']}/{e['fakes']}; taşıma: ajan `canonical` {e['carry_agent_canonical']}/{e['carry_agent_blocks']} ({e['carry_scenarios']} senaryo)",
            "threshold": "ajan `canonical` = 0; sahte 10/10",
            "pass": e["pass"],
            "source": "p2_0_3/d_a_prime.json",
        },
        {
            "id": "H-P2-0.3f",
            "what": "Obsidian dışı insan (D-B2)",
            "value": f"{f['unverified_after_window']}/{f['new_blocks']} = {_f(f['false_unverified_rate'])}; p95 {_f(f['latency_p95_ms'], 1)} ms ({f['sessions']} oturum)",
            "threshold": "≤ 0.01; p95 < 500 ms",
            "pass": f["pass"],
            "source": "p2_0_2/d_b2.json",
        },
        {
            "id": "H-P2-0.3h",
            "what": "Eski vault'lar (D-E)",
            "value": h["vaults"],
            "threshold": "6/6",
            "pass": h["pass"],
            "source": "p2_0_2/d_e.json",
        },
        {
            "id": "H-P2-0.3i",
            "what": "Determinizm: yüksüz ve yüklü çift (D-A, D-A2, D-C günlükleri bayt eşit; bütün çift kümesi)",
            "value": f"yüksüz: {_pair(pairs['unloaded'])}; yüklü: {_pair(pairs['loaded'])}",
            "threshold": "ikisi de eşit",
            "pass": pass_i["unloaded"] and pass_i["loaded"],
            "source": "pair_compare.json; yüklü çiftin pair_compare.json'u",
        },
        {
            "id": "H-P2-0.3j",
            "what": "Uyumluluk",
            "value": f"{j['passed']}/{j['total']}, seviye {j['level_claimed']}, spec {j['spec_version']}",
            "threshold": "39/39",
            "pass": j["pass"],
            "source": "conformance_p1d1.json",
        },
    ]


def decide(r: list[dict], chains: dict[str, dict]) -> tuple[str, list[str]]:
    invalid = [k for k, c in chains.items() if not c["seed"]["pass"]]
    if invalid:
        return (
            f'**Tur geçersiz** (GOAL "Tasarım"): {", ".join(invalid)} zincirinde D-B tohumu ön-kayıtlı değil.',
            [],
        )
    failed = [x["id"] for x in r if x["pass"] is False]
    waiting = [x["id"] for x in r if x["pass"] is None]
    if waiting:
        return f"**Karar bekler**: ölçülmedi {', '.join(waiting)}.", failed
    fs = set(failed)
    if fs & DESIGN:
        rule = "**Tasarım kusuru** → tur kaldı, karar talebi; P2-L başlamaz."
    elif fs & INFRA:
        rule = "**Altyapı/regresyon kusuru** → tur kaldı, karar talebi."
    elif "H-P2-0.3b" in fs and fs <= {"H-P2-0.3b", "H-P2-0.3b-yük"}:
        rule = "**0.3b kaldı** → varsayılan `canonical` **kesin** kalır; P2-0 dizisi belgelenmiş sınırla kapanır, başka yama turu açılmaz; NORTH_STAR notu olduğu gibi; **P2-L başlar**."
    elif fs == {"H-P2-0.3b-yük"}:
        rule = '**Yalnız 0.3b-yük kaldı** → hipotez kanıtlandı, belgelenmiş sınırla ("yük altında yanlış `unverified` %2\'yi aşabilir"); `unverified` varsayılan olur (sonraki turun açılış görevi), sınır THREAT_MODEL ve NORTH_STAR notunda; **P2-L başlar**.'
    elif not fs:
        rule = '**Hepsi geçti** → hipotez kanıtlandı. Bağımsız doğrulama "güvenilir" (ya da koşulları kapanmış) ise: `unverified` yeniden varsayılan (`rootmark serve`\'ün tarifi `unattested = "unverified"`; K-P2-02.5 (1) geri alınır; mevcut `policy.toml`\'lar değişmez; uygulama sonraki turun açılış görevinde), spec §7.1/§8 "Validated" (DEV_PROCESS §5), NORTH_STAR notu yeniden yazılır (insan kararı), P2-0 dizisi kesin kapanır; **P2-L başlar**.'
    else:
        rule = "**Tanımlanmamış kalış** → tur kaldı, karar talebi."
    return rule, failed


def _kind_rows(cs: list[dict | None]) -> list[str]:
    kinds = sorted({k for c in cs if c for k in c})
    out = []
    for k in kinds:
        cells = [
            "—" if c is None or k not in c else f"{c[k]['unverified']}/{c[k]['new_blocks']}"
            for c in cs
        ]
        out.append(f"| {k} | " + " | ".join(cells) + " |")
    return out


def _class_rows(cs: list[dict | None]) -> list[str]:
    classes = sorted({k for c in cs if c for k in c})
    out = []
    for k in classes:
        cells = [
            "—" if c is None or k not in c else f"{c[k]['unverified']}/{c[k]['blocks']}" for c in cs
        ]
        out.append(f"| {k} | " + " | ".join(cells) + " |")
    return out


def tables(chains: dict[str, dict]) -> list[str]:
    p2 = _load(P2_0_2_RUN, "p2_0_2_checks.json")
    info = json.loads(P2A_INFO.read_text(encoding="utf-8"))
    names = ["A (resmi)", "B", "yüklü A", "yüklü B"]
    bs = [chains[k]["H-P2-0.3b"] for k in ("A", "B", "load A", "load B")]
    bps = [chains[k]["H-P2-0.3b′"] for k in ("A", "B", "load A", "load B")]
    cs = [chains[k]["H-P2-0.3c"] for k in ("A", "B", "load A", "load B")]
    ok = [not b.get("skipped") for b in bs]
    head = (
        "| | " + " | ".join(names) + " | P2-0.2 resmi (test-3) | P2-A bilgi (test-3, (A) sonrası) |"
    )
    sep = "|---" * (len(names) + 3) + "|"
    info_kind = c2._by_kind(info["harness"])
    return [
        "**D-B test-4, dört zincir** (her zincirin `p2_0_3_checks.json`'u; karşılaştırma: P2-0.2 resmi RUN `p2_0_2_checks.json`, P2-A `info_d_b_test3.json`):",
        "",
        head,
        sep,
        "| yanlış `unverified` | "
        + " | ".join(
            f"{b['unverified_after_window']}/{b['new_blocks']} = {_f(b['false_unverified_rate'])}"
            if o
            else "atlandı"
            for b, o in zip(bs, ok, strict=True)
        )
        + f" | {p2['H-P2-0.2b']['unverified_after_window']}/{p2['H-P2-0.2b']['new_blocks']} = {_f(p2['H-P2-0.2b']['false_unverified_rate'])}"
        + f" | {info['unverified_after_window']}/{info['new_blocks']} = {_f(info['value_0_2b'])} |",
        "| p95 (ms) | "
        + " | ".join(_f(c["latency_p95_ms"], 0) if o else "—" for c, o in zip(cs, ok, strict=True))
        + f" | {_f(p2['H-P2-0.2c']['latency_p95_ms'], 0)} | {_f(info['latency_p95_ms'], 0)} |",
        "| yeniden adlandırma oturumları | "
        + " | ".join(
            f"{bp['unverified_after_window']}/{bp['new_blocks']}" if o else "—"
            for bp, o in zip(bps, ok, strict=True)
        )
        + f" | {p2['H-P2-0.2b′']['unverified_after_window']}/{p2['H-P2-0.2b′']['new_blocks']}"
        + f" | {info['renames']['unverified_after_window']}/{info['renames']['new_blocks']} |",
        "| tohum | "
        + " | ".join(str(b["seed"]) if o else "—" for b, o in zip(bs, ok, strict=True))
        + f" | {p2['H-P2-0.2b']['seed']} | {info['seed']} |",
        "",
        "**Oturum türüne göre** (W sonunda `unverified` / yeni blok; test-4 ile test-3'ün oturumları farklıdır, sütunlar yalnız yan yana konur):",
        "",
        "| Oturum türü | " + " | ".join(names) + " | P2-0.2 resmi | P2-A bilgi |",
        sep,
        *_kind_rows(
            [b.get("by_kind") if o else None for b, o in zip(bs, ok, strict=True)]
            + [p2["H-P2-0.2b"]["by_kind"], info_kind]
        ),
        "",
        "**H-P2-0.3b′ — yeniden adlandırma oturumlarının yeni blokları, bağlantı biçimine göre** (`renames.by_class`; sınıflar P2-0.2 ASSUMPTIONS X0-7, X0-18; bir blok birden çok sınıfta sayılabilir):",
        "",
        "| Biçim | " + " | ".join(names) + " | P2-0.2 resmi | P2-A bilgi |",
        sep,
        *_class_rows(
            [bp.get("by_class") if o else None for bp, o in zip(bps, ok, strict=True)]
            + [p2["H-P2-0.2b′"]["by_class"], info["renames"]["by_class"]]
        ),
        "",
        f"Bağlantı ayarları (X0-15, resmi zincir): `{json.dumps(bs[0].get('link_settings'))}`.",
        "",
    ]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--chain-b", type=Path, required=True)
    ap.add_argument("--load-a", type=Path, required=True)
    ap.add_argument("--load-b", type=Path, required=True)
    ap.add_argument("--load-pair", type=Path)
    a = ap.parse_args(argv)
    dirs = {"A": a.run_dir, "B": a.chain_b, "load A": a.load_a, "load B": a.load_b}
    chains = {k: _load(d, "p2_0_3_checks.json") for k, d in dirs.items()}
    lp = a.load_pair or a.load_a / "pair_compare.json"
    if not lp.is_file():
        raise FileNotFoundError(f"{lp} missing")
    pairs = {
        "unloaded": _load(a.run_dir, "pair_compare.json"),
        "loaded": json.loads(lp.read_text(encoding="utf-8")),
    }
    man = {k: _load(d, "manifest.json") for k, d in dirs.items()}
    commits = {k: commit_of(m) for k, m in man.items()}
    if len(set(commits.values())) != 1:
        raise ValueError(f"the four chains measured different commits: {commits}")
    r = rows(chains, pairs)
    assert tuple(x["id"] for x in r) == IDS
    rule, failed = decide(r, chains)
    c = chains["A"]
    s, lg, p2a = c["seed"], c["logs_vs_p2a"], c["p2a_recorders"]
    fs, reg, t = p2a.get("fail_summary", {}), p2a.get("regress", {}), p2a.get("tamper", {})
    lines = [
        "# P2-0.3 — FINAL (tasdikli yazma: yeniden ölçüm, hipotez turu)",
        "",
        f"- RUN: `{a.run_dir.name}` (resmi, yüksüz çiftin A zinciri); commit `{commits['A']}`",
        f"- Zincirler: yüksüz B `{a.chain_b.name}`; yüklü A `{a.load_a.name}`, yüklü B `{a.load_b.name}` (dördü aynı commit).",
        f"- Eşikler GOAL P2-0.3'ünkiler (P2-0'ınkiler aynen); `W` = {c['window']['window_s']} s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı). Spec §7.1/§8 durumu: Draft (damga bağımsız doğrulamadan sonra; DEV_PROCESS §5).",
        f"- Tohum (D-B test-4): `{s['seed']}` = {s['derivation']} (R0'da ön-kayıtlı); kayıtlı {s['recorded']}; yeniden kullanılmayan {s['not_reused']} → {_ok(s['pass'])}.",
        f"- Ürün değişmedi: D-A, D-A2, D-C günlükleri P2-A resmi RUN'ıyla (`{lg['reference_run']}`) bayt eşit → {_ok(lg['pass'])} (dört zincirde: "
        + ", ".join(f"{k} {chains[k]['logs_vs_p2a']['pass']}" for k in chains)
        + ").",
        "",
        "## Hipotezler",
        "",
        *_table(r),
        "",
        "## D-B test-4",
        "",
        *tables(chains),
        "## Aynı zincirin P2-A kaydedicileri (bilgi)",
        "",
        f"- FAIL özeti (`p2a/historical.json`, referans `{fs.get('reference_run')}`): sayılan FAIL {fs.get('counted_fails')}, tarihsel {fs.get('historical')}, göç gerekli {fs.get('migration_required')}.",
        f"- Regress (`regress_vs_p2_0_2.json`): {reg.get('n_fields')} alan, dışlama dışı fark {len(reg.get('differing_fields') or {})} → {_ok(reg.get('pass'))}.",
        f"- Kasıtlı vakalar (`p2a/tamper.json`): {_ok(t.get('pass'))}.",
        "",
        "## Karar kuralı",
        "",
        f"Kalan hipotezler: {', '.join(failed) if failed else 'yok'}; bağımsız doğrulama açık.",
        "",
        rule,
        "",
        "",
    ]
    text = "\n".join(lines)
    (a.run_dir / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(rule)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
