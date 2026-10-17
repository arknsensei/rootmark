"""P1-E.1 round report (GOAL P1-E.1 E1.5): `FINAL.md` from one RUN directory.

Every number is read from a file of the RUN (named next to it): the GOAL
E1-1..E1-9 table (E1-7 = `pair_compare.json`, `eval.compare_p1e1 pair`,
A ↔ B), the deliberate cases, the FAILs that the frozen recorders of
earlier rounds record — each labelled "tarihsel, karar K-… ile kapandı"
or counted (K-E1.4, `p1e1/historical.json`, `eval.historical`) — and the
decision rule of docs/rounds/P1-E.1/GOAL.md:

- E1-1..E1-9 pass ∧ the independent verification says "Güvenilir" → the
  human steps: `git tag v0.1.0`, the `release.yml` dispatch, the
  docs/RELEASE.md checklist; phase 1 closes. The verification is not part
  of this RUN: the report says what holds if it confirms.
- any other outcome (a criterion fails, or a counted FAIL of an earlier
  recorder) → undefined failure (K-D1.9): the round fails, a decision
  request.

Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P1-E.1/FINAL.md`.

    python -m eval.report_p1e1 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import historical
from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-E.1" / "FINAL.md"
IDS = tuple(f"E1-{n}" for n in range(1, 10))


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool | None) -> str:
    return "geçti" if b is True else "kaldı"


def rows(d: Path) -> list[dict]:
    c = _load(d, "p1e1_checks.json")
    pair = _load(d, "pair_compare.json")
    real3 = c["E1-3"]
    e6 = c["E1-6"]["regress_vs_p1e"]
    r = [
        {
            "id": "E1-1",
            "what": "Gerçek `/2` indeks + `auto_rebuild=false`: `status` GET/POST/MCP 200 `stale`; `search` 503; `blame/checkpoints/trace` 200; `auto_rebuild=true` → hepsi 200",
            "value": c["E1-1"]["passed"],
            "threshold": "8/8",
            "pass": c["E1-1"]["pass"],
            "source": "p1e1/status/e1_1.json",
        },
        {
            "id": "E1-2",
            "what": "Okunamayan manifest: `status` 200 `damaged`; `search` 503",
            "value": c["E1-2"]["passed"],
            "threshold": "2/2",
            "pass": c["E1-2"]["pass"],
            "source": "p1e1/status/e1_2.json",
        },
        {
            "id": "E1-3",
            "what": "Gerçek Obsidian e2e harness 11 adım (yerel)",
            "value": "atlandı (Obsidian yok)"
            if real3.get("skipped")
            else f"{real3['steps']}; E-3 {real3['e3_md_changes_only_at_propose']}; sözleşme dışı {real3['e4_outside_contract']}/{real3['e4_requests']}",
            "threshold": "11/11",
            "pass": real3["pass"],
            "source": "p1e1/release/plugin_e2e_real.json",
        },
        {
            "id": "E1-4",
            "what": "`release_check.sh` R7 SIGPIPE düzeltmesi; kontrol listesinin bütün maddeleri betikle",
            "value": f"R7 {c['E1-4']['r7']}; {len(c['E1-4']['release_md_items'])} madde, betikte {len(c['E1-4']['script_items'])}; release_check {c['E1-4']['release_check']}",
            "threshold": "geçer",
            "pass": c["E1-4"]["pass"],
            "source": "p1e1/release/release_check.json",
        },
        {
            "id": "E1-5",
            "what": "`release.yml` insan tetiklemeli; `aarch64-apple-darwin` kaydı (hash, `--version`, `--spec-version`, 34/34); diğerleri kayıt ya da atlandı",
            "value": f"{c['E1-5']['required']['target']} {c['E1-5']['required']['version']} spec {c['E1-5']['required']['spec_version']} conformance {(c['E1-5']['required']['conformance'] or {}).get('passed')}/{(c['E1-5']['required']['conformance'] or {}).get('total')}, sha256 {c['E1-5']['sha256_present']}; diğer: {c['E1-5']['other_targets']}; R13 {c['E1-5']['workflow_r13']}",
            "threshold": "1/1 zorunlu",
            "pass": c["E1-5"]["pass"],
            "source": "p1e1/release/release.json",
        },
        {
            "id": "E1-6",
            "what": "Uyumluluk 34/34; P1-E kalite alanları regresyon (P1-E resmi RUN)",
            "value": f"{e6['n_fields']} alan, hariç dışı fark {len(e6['differing_fields'])}, karar gereği {len(e6['excluded_differences'])} alt ağaç, kullanılmayan hariç {len(e6['exclusions_without_difference'])}; uyumluluk {c['E1-6']['conformance']['passed']}/{c['E1-6']['conformance']['total']}",
            "threshold": "fark 0",
            "pass": c["E1-6"]["pass"],
            "source": "regress_vs_p1e.json",
        },
        {
            "id": "E1-7",
            "what": "Çift koşturma (A ↔ B)",
            "value": f"{pair['n_fields']} alan, fark {len(pair['differing_fields'])}; yapıtlar bayt eşit {pair['bytes_equal']}",
            "threshold": "fark 0",
            "pass": pair["equal"],
            "source": "pair_compare.json",
        },
        {
            "id": "E1-8",
            "what": "`contract_check.sh` HTTP adımı `status`'un yeni alanını ve 503 istisnasının `status`'u kapsamadığını denetler",
            "value": f"durumlar {[s['state'] for s in c['E1-8']['status_index'] if s['ok']]}; kasıtlı {sum(x['as_expected'] for x in c['E1-8']['deliberate'])}/{len(c['E1-8']['deliberate'])}",
            "threshold": "geçer",
            "pass": c["E1-8"]["pass"],
            "source": "p1e1/contract.json",
        },
        {
            "id": "E1-9",
            "what": "Sözleşme §3 güncel; eklenti bağlantı ekranı `stale/damaged` durumunu ve yeniden kurma yolunu gösteriyor (e2e adımı)",
            "value": "atlandı (Obsidian yok)"
            if c["E1-9"]["skipped"]
            else f"durum çubuğu `{(c['E1-9']['connection_step'] or {}).get('status_bar')}`; yeniden kurma sonrası `{(c['E1-9']['connection_step'] or {}).get('status_bar_after_rebuild')}`",
            "threshold": "1/1",
            "pass": c["E1-9"]["pass"],
            "source": "p1e1/release/plugin_e2e_real.json, p1e1/contract.json",
        },
    ]
    return r


def decide(r: list[dict], hist: dict) -> tuple[str, list[str]]:
    failed = [x["id"] for x in r if x["pass"] is not True]
    counted = hist["counted"]
    if not failed and not counted:
        return (
            (
                '**E1-1..E1-9 geçti → bağımsız doğrulama "Güvenilir" derse insan adımı:** '
                "`git tag v0.1.0`, `release.yml` tetikleme (workflow_dispatch), `docs/RELEASE.md` "
                "kontrol listesi; faz 1 kapanır. Doğrulama bu RUN'ın parçası değildir; etiket ve "
                "tetikleme bu turda yapılmadı."
            ),
            failed,
        )
    return (
        (
            "**Tanımlanmamış kalış → tur kaldı, karar talebi** (K-D1.9): "
            f"kalan kriterler {failed or 'yok'}; sayılan önceki-tur FAIL'leri {counted or 'yok'}."
        ),
        failed,
    )


def _table(rs: list[dict]) -> list[str]:
    out = ["| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |", "|---|---|---|---|---|---|"]
    for x in rs:
        val = str(x["value"]).replace("|", "\\|")
        out.append(
            f"| {x['id']} | {x['what']} | {val} | {x['threshold']} | {_ok(x['pass'])} | `{x['source']}` |"
        )
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p1e1_checks.json")
    hist = _load(d, "p1e1/historical.json")
    r = rows(d)
    assert tuple(x["id"] for x in r) == IDS
    rule, failed = decide(r, hist)
    t = c["tamper"]
    lines = [
        "# P1-E.1 — FINAL",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).",
        "",
        "## Çıkış kriterleri",
        "",
        *_table(r),
        "",
        f"Kasıtlı vakalar (`p1e1/tamper_p1e1.json`, `p1e1/tamper_p1e1_e2e.json`): {t['rejected']} reddedildi, kontroller {t['controls']}, gerçek Obsidian vakaları koştu: {t['real_cases_run']} → {_ok(t['pass'])}.",
        "",
        "## Önceki turların kaydedicileri (K-E1.4)",
        "",
        f"`p1e1/historical.json` (referans: `{hist['reference_run']}`): {hist['historical']} tarihsel FAIL sayılmadı; sayılan FAIL {hist['counted_fails']}.",
        "",
        *[f"- {historical.label(row)}" for row in hist["fails"]],
        "",
        "## Karar kuralı",
        "",
        f"Kalan kriterler: {', '.join(failed) if failed else 'yok'}; bağımsız doğrulama açık.",
        "",
        rule,
        "",
    ]
    text = "\n".join(lines)
    (d / "FINAL.md").write_text(text, encoding="utf-8")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    print(rule)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
