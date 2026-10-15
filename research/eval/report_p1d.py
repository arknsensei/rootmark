"""P1-D round report (TASKS D8): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
D-1..D-11 table, the scene and the deliberate corruptions, and the decision
rule of docs/rounds/P1-D/GOAL.md, derived from the rows:

- D-1..D-11 all pass → P1-E starts.
- D-3, D-4 or D-5 fails → governance violation; P1-E does not start.
- D-8 fails → determinism; P1-E does not start.
- D-1 or D-7 fails (and none of the above) → P1-E starts, 0.1.0 waits.
- any other failure → the rule names no branch: reported as such, no branch
  is assumed (decision request).

D-1: GOAL says 9 tools, its table lists 10 (ASSUMPTIONS D5-1); every listed
tool is measured. D-8 = the e2e's double run inside each chain and
`pair_compare.json` (`eval.compare_p1d pair`, A ↔ B). D-9 =
`regress_vs_p1c1.json`. Writes `<RUN_DIR>/FINAL.md` and
`reports/rounds/P1-D/FINAL.md`.

    python -m eval.report_p1d --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-D" / "FINAL.md"
GOVERNANCE = ("D-3", "D-4", "D-5")
DETERMINISM = ("D-8",)
RELEASE = ("D-1", "D-7")


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool) -> str:
    return "geçti" if b else "**KALDI**"


def _b(b: bool) -> str:
    return "evet" if b else "**hayır**"


def evaluate(d: Path) -> list[dict]:
    c = _load(d, "p1d_checks.json")
    pair = _load(d, "pair_compare.json")
    p1d_bytes = {k: v for k, v in pair["bytes"].items() if k.startswith("p1d/")}
    d10 = c["D-10"]["measured"]
    return [
        {
            "id": "D-1",
            "what": "Resmi MCP Inspector `tools/list`, şema doğrulaması (JSON Schema 2020-12), araç başına Inspector çağrısı",
            "value": f"{c['D-1']['inspector']}: şema {c['D-1']['schemas_valid']}, çağrı {c['D-1']['inspector_calls_ok']}",
            "threshold": f"{c['D-1']['goal_tools']}/{c['D-1']['goal_tools']} (tablo 10 araç; ASSUMPTIONS D5-1)",
            "pass": c["D-1"]["pass"],
            "source": "p1d/inspector/inspector.json",
        },
        {
            "id": "D-2",
            "what": "araç başına betikli uçtan uca test (`scripts/mcp_e2e.sh`, stdio JSON-RPC)",
            "value": f"{c['D-2']['tools_passed']} araç; el sıkışması {_b(c['D-2']['handshake'])}",
            "threshold": "9/9 (10 araç)",
            "pass": c["D-2"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json",
        },
        {
            "id": "D-3",
            "what": "`propose` sonrası: düz md (regex `lineage\\|version_id\\|mdkernel` 0), bloklar `proposal`, varsayılan `search` bulmuyor, `include_proposal` etiketli buluyor",
            "value": c["D-3"]["passed"],
            "threshold": "4/4",
            "pass": c["D-3"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json",
        },
        {
            "id": "D-4",
            "what": "MCP `promote` → `denied/human_only`; CLI `promote --actor human:<id>` → `Promoted`; ardından `search` buluyor",
            "value": c["D-4"]["passed"],
            "threshold": "3/3",
            "pass": c["D-4"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json",
        },
        {
            "id": "D-5",
            "what": "her `search`/`read_block` için tam bir iz, `consumer` = MCP istemci adı/sürümü",
            "value": f"{c['D-5']['passed']} denetim (200 çağrı → 200 iz)",
            "threshold": "200 → 200",
            "pass": c["D-5"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json",
        },
        {
            "id": "D-6",
            "what": "`checkpoint` → `propose` + CLI terfi → `search --at <cp>` eski, `search` yeni",
            "value": c["D-6"]["passed"],
            "threshold": "2/2",
            "pass": c["D-6"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json",
        },
        {
            "id": "D-7",
            "what": "HTTP: token'sız 401, yanlış token 401, doğru 200, `X-Consumer` yok 400, dış arayüze bağlanma reddi",
            "value": f"{c['D-7']['passed']} (diğer HTTP/servis vakaları {c['D-7']['other_http_cases']})",
            "threshold": "5/5",
            "pass": c["D-7"]["pass"],
            "source": "p1d/http/http_e2e.json",
        },
        {
            "id": "D-8",
            "what": "çift koşturma: MCP e2e çıktıları, vault `events.jsonl`/`traces.jsonl` (zaman damgası/token yok)",
            "value": f"zincir içi iki koşturma eşit {_b(c['D-8']['e2e_two_runs_equal'])}; A↔B {pair['n_fields']} alan, "
            f"farklı {len(pair['differing_fields'])}; P1-D baytları {sum(v['equal'] for v in p1d_bytes.values())}/{len(p1d_bytes)} eşit, "
            f"bütün baytlar eşit {_b(pair['bytes_equal'])}",
            "threshold": "eşit",
            "pass": c["D-8"]["pass"] and pair["equal"],
            "source": "p1d/mcp_e2e/mcp_e2e.json, pair_compare.json",
        },
        {
            "id": "D-9",
            "what": "C1/C2/C3 uyumluluk değişmedi; P1-C.1 resmi RUN'ı ile P1-C.1 çift kümesi (kalite regresyonu dahil)",
            "value": f"{c['D-9']['n_fields']} alan, farklı {len(c['D-9']['differing_fields'])}; uyumluluk "
            f"C1 {c['D-9']['conformance']['by_level']['1']}, C2 {c['D-9']['conformance']['by_level']['2']}, "
            f"C3 {c['D-9']['conformance']['by_level']['3']}, farklı test {len(c['D-9']['conformance']['differing_tests'])}",
            "threshold": "fark 0",
            "pass": c["D-9"]["pass"],
            "source": "regress_vs_p1c1.json",
        },
        {
            "id": "D-10",
            "what": "`serve --watch`: dışarıdan insan düzenlemesi → `search` (sıcak) buluyor; `canonical`",
            "value": f"{d10['edits']} düzenleme, p95 {d10['p95_ms']:.1f} ms, en çok {d10['max_ms']:.1f} ms; "
            f"canonical/Human {_b(c['D-10']['layers_canonical_human'])}; SIGTERM compact {_b(c['D-10']['sigterm_compacts'])}, "
            f"compact sırasında kaybolmuyor {_b(c['D-10']['sigterm_never_lost'])}",
            "threshold": f"p95 < {c['D-10']['threshold_p95_ms']} ms, katman doğru",
            "pass": c["D-10"]["pass"],
            "source": "p1d/d10.json, p1d/sigterm.json",
        },
        {
            "id": "D-11",
            "what": "kasıtlı: `propose` `../`, vault dışı yol, 10 MB üstü, olmayan `replace_section` başlığı",
            "value": f"MCP e2e {c['D-11']['mcp']['passed']}; tamper {sum(c['D-11']['tamper'].values())}/{len(c['D-11']['tamper'])}",
            "threshold": "4/4 hata",
            "pass": c["D-11"]["pass"],
            "source": "p1d/mcp_e2e/mcp_e2e.json, p1d/tamper_p1d.json",
        },
    ]


def decision(rows: list[dict]) -> list[str]:
    ok = {r["id"]: r["pass"] for r in rows}
    bad = [k for k, v in ok.items() if not v]
    if not bad:
        return ["D-1..D-11 geçti → **P1-E başlar**."]
    out = []
    gov = [k for k in GOVERNANCE if k in bad]
    det = [k for k in DETERMINISM if k in bad]
    rel = [k for k in RELEASE if k in bad]
    other = [k for k in bad if k not in (*GOVERNANCE, *DETERMINISM, *RELEASE)]
    if gov:
        out.append(f"{', '.join(gov)} kaldı → **yönetişim ihlali; P1-E başlamaz**.")
    if det:
        out.append("D-8 kaldı → **determinizm; P1-E başlamaz**.")
    if rel and not gov and not det:
        out.append(f"{', '.join(rel)} kaldı → **P1-E başlar, 0.1.0 bekler**.")
    elif rel:
        out.append(f"{', '.join(rel)} da kaldı (0.1.0 bekler).")
    if other:
        out.append(
            f"{', '.join(other)} kaldı → karar kuralı bu durumu adlandırmaz; dal varsayılmaz, karar talebi gerekir."
        )
    return out


def render(d: Path) -> str:
    man = _load(d, "manifest.json")
    rows = evaluate(d)
    c = _load(d, "p1d_checks.json")
    scene = _load(d, "p1d/scene/scene.json")
    tamper = _load(d, "p1d/tamper_p1d.json")
    lines = [
        "# P1-D — Sonuç raporu",
        "",
        f"- RUN: `{d.name}` · ölçülen commit `{commit_of(man)}`",
        "- Üretim: `python -m eval.report_p1d --run-dir <RUN_DIR>` (her sayı bu RUN'ın dosyalarından).",
        "",
        "## Çıkış kriterleri (GOAL P1-D)",
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
        "## Ürün sahnesi, ajan tarafı (`p1d/scene/scene.json`, TASKS D6)",
        "",
        f"{scene['passed']}/{scene['total']} adım ({scene['consumer']}).",
        "",
        "| Adım | Araç | Gösterdiği | Sonuç |",
        "|---|---|---|---|",
    ]
    for s in scene["steps"]:
        lines.append(
            f"| {s['step']} | {s['tool']} | `{json.dumps(s['shows'], ensure_ascii=False)}` | {_ok(s['pass'])} |"
        )
    lines += [
        "",
        "## Kasıtlı bozmalar (`p1d/tamper_p1d.json`)",
        "",
        (
            f"{tamper['rejected']}/{tamper['rejected_total']} bozma reddedildi, "
            f"{tamper['controls_ok']}/{tamper['controls_total']} kontrol geçti."
        ),
        "",
        *[
            f"- [{x['group']}] {x['case']}: {'beklendiği gibi' if x['as_expected'] else '**BEKLENMEDİK**'}"
            for x in tamper["cases"]
        ],
        "",
        "## D-9 farkları (`regress_vs_p1c1.json`)",
        "",
        *([f"- `{k}`" for k in sorted(c["D-9"]["differing_fields"])] or ["- (fark yok)"]),
        "",
        "## Notlar",
        "",
        '- D-1/D-2: GOAL "9 araç" der; GOAL tablosu 10 araç listeler (`status, search, read_block, history, blame, propose, promote, checkpoint, checkpoints, trace`). Hepsi ölçüldü (ASSUMPTIONS D5-1).',
        "- D-4/D-6: insan terfisi CLI'dan (`mdk promote --actor human:<id>`); bir chunk bloklarının en düşük katmanını alır (spec §9), bu yüzden dokümanın bütün soyları terfi ettirildi (ASSUMPTIONS D5-2).",
        "- Açık karar talepleri: K-D4-T (`replace_section` insan bloklarını terfisiz siler; CLI `mdk propose`; TOCTOU sınırı) — `docs/DECISIONS.md`.",
        "",
        "## Karar kuralı (GOAL P1-D)",
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
