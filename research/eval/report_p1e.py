"""P1-E round report (TASKS E8): `FINAL.md` from one RUN directory only.

Every number is read from a file of the RUN (named next to it): the GOAL
E-1..E-10 table (E-11 is the independent verifier's and stays open here),
the ADDENDUM E-12..E-16 table, the deliberate corruptions, and the
decision rule of docs/rounds/P1-E/GOAL.md, derived from the rows:

- E-1..E-11 all pass → phase 1 complete: the 0.1.0 tag (by a human,
  docs/RELEASE.md), spec §7–§11 "Validated" confirmed, NORTH_STAR updated,
  phase 2 planning starts.
- E-3, E-4, E-5 or E-6 fails → client governance; no release.
- E-7 fails → the release is deferred, phase 2 planning starts.
- E-2 fails while E-1 passes → the scene is right in the kernel, the UI is
  missing: release "binary + spec", the plugin labelled "not ready".
- any other failure → undefined failure (K-D1.9): the round fails, a
  decision request.
- ADDENDUM: E-12 fails → E1 does not start (a decision request).

Branches are applied in that order of precedence (governance first);
E-11 open means every conclusion holds only if the independent
verification confirms the measured rows. E-8 = `pair_compare.json`
(`eval.compare_p1e pair`, A ↔ B). Writes `<RUN_DIR>/FINAL.md` and
`reports/rounds/P1-E/FINAL.md`.

    python -m eval.report_p1e --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P1-E" / "FINAL.md"
GOVERNANCE = ("E-3", "E-4", "E-5", "E-6")
GOAL_IDS = tuple(f"E-{n}" for n in range(1, 11))
ADDENDUM_IDS = tuple(f"E-{n}" for n in range(12, 17))


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool | None) -> str:
    return "açık (doğrulayıcı)" if b is None else ("geçti" if b else "**KALDI**")


def rows(d: Path) -> list[dict]:
    c = _load(d, "p1e_checks.json")
    pair = _load(d, "pair_compare.json")
    p1e_bytes = {k: v for k, v in pair["bytes"].items() if k.startswith("p1e/")}
    e7 = c["E-7"]
    reg = c["E-9"]["regress_vs_p1d1"]
    return [
        {
            "id": "E-1",
            "what": "`scripts/scene.sh` temiz vault'ta 11 adım (ajan MCP, insan HTTP)",
            "value": c["E-1"]["steps"],
            "threshold": "11/11",
            "pass": c["E-1"]["pass"],
            "source": "p1e/scene/scene.json",
        },
        {
            "id": "E-2",
            "what": "Plugin e2e aynı 11 adım (komut katmanı gerçek `rootmark serve`'e karşı; Obsidian başsız koşmadı, ASSUMPTIONS E4-1)",
            "value": c["E-2"]["steps"],
            "threshold": "11/11",
            "pass": c["E-2"]["pass"],
            "source": "p1e/plugin_e2e/plugin_e2e.json",
        },
        {
            "id": "E-3",
            "what": "md sha256'ları yalnız `propose` adımında değişir (sahne ve e2e, her adım)",
            "value": f"sahne {c['E-3']['scene_md_changes_only_at_propose']}, e2e {c['E-3']['plugin_e2e_md_changes_only_at_propose']}",
            "threshold": "doğru",
            "pass": c["E-3"]["pass"],
            "source": "p1e/scene/scene.json, p1e/plugin_e2e/plugin_e2e.json",
        },
        {
            "id": "E-4",
            "what": "Plugin yalnız sözleşme uçlarını çağırır (kayıt tutan vekil ↔ sözleşme)",
            "value": f"{c['E-4']['requests']} istek, sözleşme dışı {c['E-4']['outside_contract']}; {', '.join(c['E-4']['endpoints_used'])}",
            "threshold": "sözleşme dışı 0",
            "pass": c["E-4"]["pass"],
            "source": "p1e/plugin_e2e/plugin_e2e.json, access_log.json",
        },
        {
            "id": "E-5",
            "what": "Plugin'den Promote → `Promoted` actor `human:<human_id>`; gerekçesiz → 400",
            "value": f"aktörler {c['E-5']['plugin_promote']['actors']}; gerekçesiz {c['E-5']['plugin_promote']['without_reason']}",
            "threshold": "2/2",
            "pass": c["E-5"]["pass"],
            "source": "p1e/plugin_e2e/plugin_e2e.json, p1e/scene/scene.json",
        },
        {
            "id": "E-6",
            "what": "Ajanın canonical değiştirme denemesi → `denied`; blame ve arama değişmez",
            "value": f"e2e {c['E-6']['plugin']['attempt']['replace_section']}/{c['E-6']['plugin']['attempt']['append']}, değişmedi {c['E-6']['plugin']['unchanged']}; sahne {c['E-6']['scene']['attempt']['replace_section']}",
            "threshold": "1/1",
            "pass": c["E-6"]["pass"],
            "source": "p1e/plugin_e2e/plugin_e2e.json, p1e/scene/scene.json",
        },
        {
            "id": "E-7",
            "what": "4 platform ikilisi derlenir, `--version`/`--spec-version` doğru, her birinde `rootmark conformance` 34/34",
            "value": f"{e7['four_targets']['ok']} ({'; '.join(f'{k}: {v}' for k, v in e7['four_targets']['targets'].items())}); zincirin yerel hedefi {e7['chain_local_target']['target']} {e7['chain_local_target']['conformance']['passed']}/34, `make conformance`'a eşit {e7['chain_local_target']['equal_to_make_conformance']}",
            "threshold": "4/4",
            "pass": c["E-7"]["pass"],
            "source": "p1e/release/release_local.json, docs/rounds/P1-E/progress/E5_release.json",
        },
        {
            "id": "E-8",
            "what": "Çift koşturma: sahne ve e2e vault `events.jsonl`/`traces.jsonl` bayt eşit; e2e çıktıları eşit",
            "value": f"{pair['n_fields']} alan, fark {len(pair['differing_fields'])}; P1-E yapıtları bayt eşit {all(v['equal'] for v in p1e_bytes.values())} ({len(p1e_bytes)} dosya)",
            "threshold": "eşit",
            "pass": pair["equal"] and all(v["equal"] for v in p1e_bytes.values()),
            "source": "pair_compare.json",
        },
        {
            "id": "E-9",
            "what": "Regresyon: P1-D.1 kalite alanları (karar gereği hariçler ölçümden önce); uyumluluk 34/34",
            "value": f"{reg['n_fields']} alan, fark {len(reg['differing_fields'])}, hariç tutulan önek {len(reg['excluded_differences'])}, kullanılmayan {len(reg['exclusions_without_difference'])}; uyumluluk {c['E-9']['conformance']['passed']}/{c['E-9']['conformance']['total']}",
            "threshold": "fark 0",
            "pass": c["E-9"]["pass"],
            "source": "regress_vs_p1d1.json, conformance_p1d1.json",
        },
        {
            "id": "E-10",
            "what": "`tsc --strict`, `eslint`, `vitest` temiz; bundle < 500 KB; Obsidian API dışında çalışma zamanı bağımlılığı 0",
            "value": f"çıkışlar {c['E-10']['tsc_eslint_vitest_exit']}, vitest {c['E-10']['vitest']['passed']}/{c['E-10']['vitest']['tests']}, {c['E-10']['bundle_bytes']} B, bağımlılık {c['E-10']['runtime_dependencies']}, paketin istekleri {c['E-10']['bundle_requires']}",
            "threshold": "geçer",
            "pass": c["E-10"]["pass"],
            "source": "p1e/client.json",
        },
        {
            "id": "E-11",
            "what": "Bağımsız doğrulayıcı sahneyi (E-1) ve plugin e2e'yi (E-2) kendi ortamında koşturur",
            "value": "doğrulama bekliyor",
            "threshold": "geçer",
            "pass": None,
            "source": "reports/rounds/P1-E/VERIFICATION.md (yazılacak)",
        },
    ]


def addendum_rows(d: Path) -> list[dict]:
    c = _load(d, "p1e_checks.json")
    return [
        {
            "id": "E-12",
            "what": "P1-D.1 `verify.sh pair` yeniden koşturma (temiz klonlar)",
            "value": f"çıkış {c['E-12']['exit_code']}, {c['E-12']['n_fields']} alan, fark {c['E-12']['differing_fields']}",
            "threshold": "çıkış 0, fark 0",
            "pass": c["E-12"]["pass"],
            "source": c["E-12"]["record"],
        },
        {
            "id": "E-13",
            "what": "`check_compare_fields` tüm `compare_*` için",
            "value": f"eşleşmeyen {c['E-13']['unmatched']}",
            "threshold": "0",
            "pass": c["E-13"]["pass"],
            "source": "p1e/check_compare_fields.json",
        },
        {
            "id": "E-14",
            "what": "`index_stale`: eski indeks → 503; `auto_rebuild` → 200",
            "value": c["E-14"]["passed"],
            "threshold": "2/2",
            "pass": c["E-14"]["pass"],
            "source": "p1e/e0/e14.json",
        },
        {
            "id": "E-15",
            "what": "Başlıksız `append` → `policy_violation` + hint; başlıklı → kabul",
            "value": c["E-15"]["passed"],
            "threshold": "2/2",
            "pass": c["E-15"]["pass"],
            "source": "p1e/e0/e15.json",
        },
        {
            "id": "E-16",
            "what": "`contract_check.sh` HTTP adımı; kasıtlı uç kaldırma → hata",
            "value": f"HTTP adımı {c['E-16']['http_step_pass']}, uç kaldırma reddedildi {c['E-16']['endpoint_removed_rejected']}",
            "threshold": "2/2",
            "pass": c["E-16"]["pass"],
            "source": "p1d1/d1_13.json, p1e/tamper_p1e.json",
        },
    ]


def decide(r: list[dict], ad: list[dict]) -> tuple[str, list[str]]:
    failed = [x["id"] for x in r if x["pass"] is False]
    ad_failed = [x["id"] for x in ad if not x["pass"]]
    cond = " (E-11 açık: sonuç bağımsız doğrulamanın ölçülen satırları teyit etmesine bağlıdır)"
    if "E-12" in ad_failed:
        return "E-12 kaldı → **E1 başlamazdı; karar talebi** (ADDENDUM).", failed + ad_failed
    if ad_failed:
        return "ADDENDUM kriteri kaldı (" + ", ".join(
            ad_failed
        ) + ") → **tanımlanmamış kalış: tur kaldı, karar talebi** (K-D1.9).", failed + ad_failed
    if not failed:
        return (
            "E-1..E-10 geçti"
            + cond
            + ' → E-11 de geçerse **Faz 1 tamamlandı**: `rootmark 0.1.0` etiketi (insan, docs/RELEASE.md), spec §7–§11 "Validated" teyit, NORTH_STAR güncellenir, faz 2 planlaması başlar.',
            failed,
        )
    if any(f in GOVERNANCE for f in failed):
        return "E-3/E-4/E-5/E-6 kaldı → **istemci yönetişimi; yayım yok.**", failed
    if failed == ["E-7"]:
        return (
            "Yalnız E-7 kaldı"
            + cond
            + " → **yayım ertelenir, faz 2 planlaması başlar** (K-E5-T). `rootmark 0.1.0` etiketi atılmaz.",
            failed,
        )
    if failed == ["E-2"] and "E-1" not in failed:
        return (
            'E-2 kaldı, E-1 geçti → **sahne çekirdekte doğru, UI eksik; yayım "ikili + spec", plugin "hazır değil".**',
            failed,
        )
    if set(failed) == {"E-2", "E-7"}:
        return (
            "E-2 ve E-7 kaldı → **tanımlanmamış birleşik kalış: tur kaldı, karar talebi** (K-D1.9).",
            failed,
        )
    return "Tanımlanmamış kalış → **tur kaldı, karar talebi** (K-D1.9).", failed


def _table(rs: list[dict]) -> list[str]:
    out = ["| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |", "|---|---|---|---|---|---|"]
    for x in rs:
        out.append(
            f"| {x['id']} | {x['what']} | {str(x['value']).replace('|', '\\|')} | {x['threshold']} | {_ok(x['pass'])} | `{x['source']}` |"
        )
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p1e_checks.json")
    r = rows(d)
    ad = addendum_rows(d)
    assert tuple(x["id"] for x in r) == (*GOAL_IDS, "E-11")
    assert tuple(x["id"] for x in ad) == ADDENDUM_IDS
    rule, failed = decide(r, ad)
    t = c["tamper"]
    lines = [
        "# P1-E — FINAL",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer); E-7'nin dört hedefi `scripts/release.sh` kaydı, E-12 P1-D.1 yeniden koşturma kaydı.",
        "",
        "## GOAL çıkış kriterleri",
        "",
        *_table(r),
        "",
        "## ADDENDUM (E0) kriterleri",
        "",
        *_table(ad),
        "",
        f"Kasıtlı bozmalar (`p1e/tamper_p1e.json`): {t['rejected']} reddedildi, kontroller {t['controls']} → {_ok(t['pass'])}.",
        "",
        "## Karar kuralı",
        "",
        f"Kalan kriterler: {', '.join(failed) if failed else 'yok'}; açık: E-11.",
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
