"""P2-0.2 round report (TASKS X6): `FINAL.md` from the RUN directory of the
unloaded pair's chain A, with the loaded pair's comparison beside it.

Every number is read from a file (named next to it): the GOAL hypothesis
table H-P2-0.2a..k (H-P2-0.2i = both pair comparisons, `eval.compare_p2_0_2
pair`: the unloaded `make verify`'s `pair_compare.json` in the RUN and the
`VERIFY_LOAD=1` pair's, given by `--load-pair`), the sets by kind, the
per-format report of the rename sessions (b′), the deliberate cases, the
regress against the P2-0.1 official RUN, the FAILs that the frozen
recorders of earlier rounds record (`p2_0_2/historical.json`,
`eval.historical_p2_0_2`, also the chain's printed summary), and the
decision rule of docs/rounds/P2-0.2/GOAL.md, applied in this order:

- 0.2a, 0.2a′ or 0.2e failed → a design flaw: the round fails, a decision
  request; P2-A does not start;
- 0.2d, 0.2h, 0.2i, 0.2j or 0.2k failed → an infrastructure flaw: the round
  fails, a decision request;
- any other failure (0.2c, regress, deliberate cases) → undefined failure:
  the round fails, a decision request;
- 0.2b or 0.2f failed (the rest passed) → GOAL P2-0's 0b branch:
  `unverified` is **not** the default, the policy ships `canonical`, the
  promise narrowed by "with an attesting client" (a decision record); P2-A
  starts;
- all pass → the "Validated" stamp of spec §7.1/§8 follows the independent
  verification (DEV_PROCESS §5, K-P2-02.3: not placed here); `unverified`
  stays the default; a NORTH_STAR proposal (a human decision); P2-A starts
  — with "şu koşullarla", the stamp waits and the conditions close in
  P2-A's opening task.

A set recorded as skipped (no Obsidian binary) or a missing loaded pair
leaves its hypotheses not measured and the rule waits.

Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P2-0.2/FINAL.md`.

    python -m eval.report_p2_0_2 --run-dir <RUN_DIR> [--load-pair <pair_compare.json>]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import historical_p2_0_2
from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P2-0.2" / "FINAL.md"
IDS = (
    "H-P2-0.2a",
    "H-P2-0.2a′",
    "H-P2-0.2b",
    "H-P2-0.2b′",
    "H-P2-0.2c",
    "H-P2-0.2d",
    "H-P2-0.2e",
    "H-P2-0.2f",
    "H-P2-0.2h",
    "H-P2-0.2i",
    "H-P2-0.2j",
    "H-P2-0.2k",
)


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool | str | None) -> str:
    if b == "info":
        return "raporlandı (bilgi)"
    return "geçti" if b is True else "ölçülmedi" if b is None else "kaldı"


def _f(x, n: int = 4) -> str:
    return "—" if x is None else f"{x:.{n}f}"


def _pair(p: dict | None) -> str:
    if p is None:
        return "yok"
    g = p["h_p2_0_2i"]
    return f"günlükler eşit {g['equal']}; {p['n_fields']} alan, fark {len(p['differing_fields'])}; yapıtlar bayt eşit {p['bytes_equal']}"


def rows(d: Path, load_pair: dict | None) -> list[dict]:
    c = _load(d, "p2_0_2_checks.json")
    da = _load(d, "p2_0_2/d_a.json")
    pair = _load(d, "pair_compare.json")
    a, b, bp, cc = c["H-P2-0.2a"], c["H-P2-0.2b"], c["H-P2-0.2b′"], c["H-P2-0.2c"]
    dd, e, f = c["H-P2-0.2d"], c["H-P2-0.2e"], c["H-P2-0.2f"]
    dc, dcp = dd["d_c"], dd["d_c_prime"]
    skip_b, skip_dp, skip_e = (x.get("skipped") is True for x in (b, dcp, e))
    pair_ok = pair["h_p2_0_2i"]["equal"] and pair["equal"]
    load_ok = None if load_pair is None else load_pair["h_p2_0_2i"]["equal"] and load_pair["equal"]
    k = c["H-P2-0.2k"]
    return [
        {
            "id": "H-P2-0.2a",
            "what": "Canonical sızıntısı (D-A, D-A2)",
            "value": f"D-A {a['d_a']['canonical_leak']}/{a['d_a']['new_blocks']} ({a['d_a']['scenarios']} senaryo); D-A2 {a['d_a2']['canonical_leak']}/{a['d_a2']['new_blocks']} ({a['d_a2']['scenarios']} senaryo)",
            "threshold": "= 0",
            "pass": a["pass"],
            "source": "p2_0_2/d_a.json, p2_0_2/d_a2.json",
        },
        {
            "id": "H-P2-0.2a′",
            "what": "Canonical içerik korunumu (D-A)",
            "value": f"{c['H-P2-0.2a′']['scenarios']} senaryo; canonical soyu değiştiren senaryo {da['scenarios_with_canonical_changed']}, değişen canonical soy {c['H-P2-0.2a′']['canonical_changed']}",
            "threshold": "200/200",
            "pass": c["H-P2-0.2a′"]["pass"],
            "source": "p2_0_2/d_a.json",
        },
        {
            "id": "H-P2-0.2b",
            "what": "Yanlış `unverified`, W sonunda (D-B test-3, gerçek Obsidian, karışık bağlantı biçimleri)",
            "value": "atlandı (Obsidian yok)"
            if skip_b
            else f"{b['unverified_after_window']}/{b['new_blocks']} = {_f(b['false_unverified_rate'])} ({b['sessions']} oturum, {b['saves']} kayıt, yan not {b['side_notes']}; tohum {b['seed']})",
            "threshold": "≤ 0.01",
            "pass": b["pass"],
            "source": "p2_0_2/d_b.json",
        },
        {
            "id": "H-P2-0.2b′",
            "what": "Yalnız yeniden adlandırma oturumları; biçim başına (bilgi)",
            "value": "atlandı"
            if skip_b
            else f"{bp['unverified_after_window']}/{bp['new_blocks']} ({bp['sessions']} oturum, {bp['records']} kayıt, yan notlar dahil); biçim tablosu aşağıda",
            "threshold": "raporlanır",
            "pass": None if skip_b else "info",
            "source": "p2_0_2/d_b.json",
        },
        {
            "id": "H-P2-0.2c",
            "what": "Tasdik gecikmesi p95 (D-B test-3)",
            "value": "atlandı"
            if skip_b
            else f"{cc['latency_p95_ms']} ms ({cc['latency_measured']} ölçüm; `canonical` olmayan {cc['latency_unsettled']}; yazımsız kayıt {cc['no_write']})",
            "threshold": "< 500 ms",
            "pass": cc["pass"],
            "source": "p2_0_2/d_b.json",
        },
        {
            "id": "H-P2-0.2d",
            "what": "Karışık katman doğruluğu (D-C, T3); gerçek Obsidian çakışma (D-C′)",
            "value": f"D-C {dc['right']}/{dc['blocks']} = {_f(dc['accuracy'])} ({dc['ticks']} tik; ajan `canonical` {dc['agent_canonical']}; `partial` satırı {dc['partial_lines']}); D-C′ "
            + (
                "atlandı (Obsidian yok)"
                if skip_dp
                else f"{dcp['human_canonical']}/{dcp['human_blocks']} = {_f(dcp['human_accuracy'])}, ajan `canonical` {dcp['agent_canonical']} ({dcp['ticks']} tik)"
            ),
            "threshold": "≥ 0.99; ≥ 0.95 ve ajan `canonical` = 0",
            "pass": dd["pass"],
            "source": "p2_0_2/d_c.json, p2_0_2/d_c_prime.json",
        },
        {
            "id": "H-P2-0.2e",
            "what": "Aklama: ajan bloğu `canonical`; sahte ret; katman taşımada ajan bloğu `canonical` (D-A′)",
            "value": "atlandı (Obsidian yok)"
            if skip_e
            else f"ajan `canonical` {e['agent_canonical']}/{e['agent_blocks']} ({e['scenarios']} senaryo; bağlantı yeniden yazıldı {e['links_rewritten']}); sahte ret {e['fakes_denied']}/{e['fakes']}; taşıma: ajan `canonical` {e['carry_agent_canonical']}/{e['carry_agent_blocks']} ({e['carry_scenarios']} senaryo; insan blokları `canonical` {e['carry_human_canonical_before']} → {e['carry_human_canonical_after']})",
            "threshold": "= 0; 10/10; = 0",
            "pass": e["pass"],
            "source": "p2_0_2/d_a_prime.json",
        },
        {
            "id": "H-P2-0.2f",
            "what": "Obsidian dışı insan: yanlış `unverified`; gecikme p95 (D-B2)",
            "value": f"{f['unverified_after_window']}/{f['new_blocks']} = {_f(f['false_unverified_rate'])}; p95 {f['latency_p95_ms']} ms ({f['sessions']} oturum; `canonical` olmayan {f['latency_unsettled']})",
            "threshold": "≤ 0.01; < 500 ms",
            "pass": f["pass"],
            "source": "p2_0_2/d_b2.json",
        },
        {
            "id": "H-P2-0.2h",
            "what": "Eski vault'lar (3 `v0.1.0` + 2 `ce4cf04` + 1 `21b16ea`): açılır, replay, `events_hash`, `attestations.jsonl` değişmez; 2.0.1 günlüğü yeni okuyucuda",
            "value": c["H-P2-0.2h"]["vaults"],
            "threshold": "6/6",
            "pass": c["H-P2-0.2h"]["pass"],
            "source": "p2_0_2/d_e.json",
        },
        {
            "id": "H-P2-0.2i",
            "what": "Determinizm: çift koşturma yüksüz ve yük altında (D-A, D-A2, D-C günlükleri bayt eşit; bütün çift kümesi)",
            "value": f"yüksüz: {_pair(pair)}; yüklü: {_pair(load_pair)}",
            "threshold": "ikisi de eşit",
            "pass": None if load_ok is None else bool(pair_ok and load_ok),
            "source": "pair_compare.json; --load-pair",
        },
        {
            "id": "H-P2-0.2j",
            "what": "Uyumluluk: mevcut 38 + C2-14 (T1/T2)",
            "value": f"{c['H-P2-0.2j']['passed']}/{c['H-P2-0.2j']['total']}, seviye {c['H-P2-0.2j']['level_claimed']}, spec {c['H-P2-0.2j']['spec_version']}; C2-14 {c['H-P2-0.2j']['new']['C2-14']}",
            "threshold": "hepsi",
            "pass": c["H-P2-0.2j"]["pass"],
            "source": "conformance_p1d1.json",
        },
        {
            "id": "H-P2-0.2k",
            "what": "Damga kuralı (DEV_PROCESS §5), CRITERION netleştirmesi, §8 okuyucu kuralı yazılı; başvuru okuyucusu: bilinmeyen alanlı satır okunur, bilinmeyen değer reddedilir",
            "value": f"yazılı {sum(k['written'].values())}/{len(k['written'])}; ileri uyumluluk {sum(x['pass'] for x in k['forward_compatibility'].values())}/{len(k['forward_compatibility'])}",
            "threshold": "yazılı ve testli",
            "pass": k["pass"],
            "source": "p2_0_2/rules.json",
        },
    ]


def decide(r: list[dict], c: dict) -> tuple[str, list[str]]:
    by = {x["id"]: x["pass"] for x in r}
    graded = [i for i in IDS if i != "H-P2-0.2b′"]
    failed = [i for i in graded if by[i] is False]
    unmeasured = [i for i in graded if by[i] is None]
    extra = [k for k in ("regress", "tamper") if c[k]["pass"] is not True]
    if unmeasured:
        return (
            f"**{unmeasured} ölçülmedi → karar kuralı bekler**: tablo tamamlanana dek karar verilmez. Diğer kalanlar: {failed or 'yok'}.",
            failed,
        )
    design = [i for i in failed if i in ("H-P2-0.2a", "H-P2-0.2a′", "H-P2-0.2e")]
    infra = [
        i for i in failed if i in ("H-P2-0.2d", "H-P2-0.2h", "H-P2-0.2i", "H-P2-0.2j", "H-P2-0.2k")
    ]
    bf = [i for i in failed if i in ("H-P2-0.2b", "H-P2-0.2f")]
    other = [i for i in failed if i not in design + infra + bf]
    if not failed and not extra:
        return (
            (
                "**Hepsi geçti → P2-0 hipotezi geniş bağlantı biçimleriyle de kanıtlandı**: spec "
                '§7.1/§8 "Validated" damgası bağımsız doğrulamanın hükmüne bağlıdır (DEV_PROCESS §5, '
                'K-P2-02.3; bu RUN damga koymaz) — "güvenilir" ise damga konur, "şu koşullarla" ise '
                "damga bekler ve koşullar P2-A'nın açılış görevinde kapatılır; `unverified` varsayılan "
                "kalır; NORTH_STAR önerisi aşağıda (insan kararı); **P2-A başlar**."
            ),
            failed,
        )
    if design:
        return (
            f"**{design} kaldı → tasarım kusuru; tur kaldı, karar talebi; P2-A başlamaz.**",
            failed,
        )
    if infra:
        return (f"**{infra} kaldı → altyapı kusuru; tur kaldı, karar talebi.**", failed)
    if other or extra:
        return (
            f"**Tanımlanmamış kalış → tur kaldı, karar talebi**: kalan {failed}; regress/kasıtlı vakalar {extra or 'geçti'}.",
            failed,
        )
    return (
        (
            f"**{bf} kaldı → GOAL P2-0'ın 0b dalı**: `unverified` varsayılan **olmaz**, politika "
            '`canonical` ile yayımlanır, vaat "tasdikli istemci ile" ibaresiyle daraltılır (karar '
            "kaydıyla; öneri aşağıda); **P2-A başlar**."
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


def _kinds(c: dict) -> list[str]:
    out: list[str] = []
    b, bp = c["H-P2-0.2b"], c["H-P2-0.2b′"]
    if not b.get("skipped"):
        out += [
            "**D-B test-3 (gerçek Obsidian), oturum türlerine göre** (`p2_0_2/d_b.json`):",
            "",
            "| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |",
            "|---|---|---|---|",
            *[
                f"| {k} | {v['saves']} | {v['new_blocks']} | {v['unverified']} |"
                for k, v in sorted(b["by_kind"].items())
            ],
            "",
            (
                "**H-P2-0.2b′ — yeniden adlandırma oturumlarının yeni blokları, bağlantı biçimine göre** "
                "(`p2_0_2/d_b.json` `renames.by_class`; sınıflar ASSUMPTIONS X0-7, X0-18; bir blok "
                "birden çok sınıfta sayılabilir):"
            ),
            "",
            "| Biçim | yeni blok | W sonunda `unverified` |",
            "|---|---|---|",
            *[f"| {k} | {v['blocks']} | {v['unverified']} |" for k, v in bp["by_class"].items()],
            "",
            f"Bağlantı ayarları (X0-15): `{json.dumps(b.get('link_settings'))}`.",
            "",
        ]
    f = c["H-P2-0.2f"]
    out += [
        "**D-B2 (Obsidian dışı insan), oturum türlerine göre** (`p2_0_2/d_b2.json`):",
        "",
        "| Tür | oturum | yeni blok | W sonunda `unverified` |",
        "|---|---|---|---|",
        *[
            f"| {k} | {v['sessions']} | {v['new_blocks']} | {v['unverified_after_window']} |"
            for k, v in f["by_kind"].items()
        ],
        "",
        "**D-A2 (Obsidian dışı ajan), yollara göre** (`p2_0_2/d_a2.json`):",
        "",
        "| Yol | senaryo | yeni blok | `canonical` sızıntısı |",
        "|---|---|---|---|",
        *[
            f"| {k} | {v['scenarios']} | {v['new_blocks']} | {v['canonical_leak']} |"
            for k, v in c["H-P2-0.2a"]["d_a2"]["by_path"].items()
        ],
        "",
    ]
    dcp = c["H-P2-0.2d"]["d_c_prime"]
    if not dcp.get("skipped"):
        out += [
            "**D-C′ (gerçek Obsidian çakışma), gecikmeye göre** (`p2_0_2/d_c_prime.json`):",
            "",
            "| Gecikme (ms) | tik | insan bloğu | insan `canonical` | ajan `canonical` |",
            "|---|---|---|---|---|",
            *[
                f"| {k} | {v['ticks']} | {v['human_blocks']} | {v['human_canonical']} | {v['agent_canonical']} |"
                for k, v in dcp["by_delay"].items()
            ],
            "",
        ]
    e = c["H-P2-0.2e"]
    if not e.get("skipped"):
        out += [
            "**D-A′ (aklama, gerçek Obsidian), türlere göre** (`p2_0_2/d_a_prime.json`):",
            "",
            "| Tür | senaryo | ajan `canonical` |",
            "|---|---|---|",
            *[
                f"| {k} | {v['scenarios']} | {v['agent_canonical']} |"
                for k, v in e["by_kind"].items()
            ],
            "",
        ]
    return out


NORTH_STAR = [
    "NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına ve bağımsız doğrulamanın hükmüne göre:",
    "",
    '- Hepsi geçti ve doğrulama "güvenilir" (ya da koşulları kapandı): vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir; insanın yaygın Obsidian bağlantı biçimleri ve yeniden adlandırmaları bunu bozmaz (tasdikli istemciyle; insan kanalına giremeyen ajana karşı — token\'ı okuyan, insan istemcisini başlatıp süren ya da eklenti dosyalarını yazan süreç kapsam dışı)." eklenir.',
    '- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical`, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).',
    "- Kaldı dallarında: vaat değişmez; öneri insanın karar talebine verdiği cevaba bağlıdır.",
]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--load-pair", type=Path)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p2_0_2_checks.json")
    hist = _load(d, "p2_0_2/historical.json")
    lp = None
    if a.load_pair is not None:
        if not a.load_pair.is_file():
            raise FileNotFoundError(f"{a.load_pair} missing")
        lp = json.loads(a.load_pair.read_text(encoding="utf-8"))
    r = rows(d, lp)
    assert tuple(x["id"] for x in r) == IDS
    rule, failed = decide(r, c)
    reg, t, fs = c["regress"], c["tamper"], c["fail_summary"]
    lines = [
        "# P2-0.2 — FINAL (bağlantı biçimleri ve katman taşıma, yama turu)",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar ve yük altındaki çiftin karşılaştırması (her satırda adı geçer). Eşikler GOAL P2-0.2'ninkiler (P2-0'ınkiler aynen); `W` = 0.5 s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı). Spec §7.1/§8 durumu: Draft (damga bağımsız doğrulamadan sonra; DEV_PROCESS §5).",
        "",
        "## Hipotezler",
        "",
        *_table(r),
        "",
        "## Kümeler",
        "",
        *_kinds(c),
        "## Regresyon ve kasıtlı vakalar",
        "",
        f"- P2-0.1 resmi RUN'ına karşı (`regress_vs_p2_0_1.json`, referans `{reg['reference_run']}`): {reg['n_fields']} alan, hariç dışı fark {len(reg['differing_fields'])}, karar gereği {len(reg['excluded_differences'])} alt ağaç, kullanılmayan hariç {len(reg['exclusions_without_difference'])}; uyumluluk eski testler eşit {not reg['conformance']['differing_tests']}, yeni test {reg['conformance']['new_tests_passed']}; P2-0.1'in yeniden koşan kümeleri (D-A, D-A2, D-E beş satır, D-C T3/X2-1 eşdeğerliği) eşit {reg['p2_0_1_sets']['equal']} → {_ok(reg['pass'])}.",
        f"- Kasıtlı vakalar (`p2_0_2/tamper_p2_0_2.json`): {t['rejected']} reddedildi, kontroller {t['controls']} → {_ok(t['pass'])}.",
        "",
        "## Önceki turların kaydedicileri",
        "",
        f"`p2_0_2/historical.json` (`eval.historical_p2_0_2`, referans `{hist['reference_run']}`; `make verify`'ın FAIL özeti bu sınıflandırıcının çıktısıdır): {hist['historical']} tarihsel FAIL sayılmadı; sayılan FAIL {hist['counted_fails']} → {_ok(fs['pass'])}.",
        "",
        *[f"- {historical_p2_0_2.label(row)}" for row in hist["fails"]],
        "",
        "## Karar kuralı",
        "",
        f"Kalan hipotezler: {', '.join(failed) if failed else 'yok'}; bağımsız doğrulama açık.",
        "",
        rule,
        "",
        "## NORTH_STAR önerisi",
        "",
        *NORTH_STAR,
        "",
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
