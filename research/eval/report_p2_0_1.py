"""P2-0.1 round report (TASKS Y7): `FINAL.md` from one RUN directory.

Every number is read from a file of the RUN (named next to it): the GOAL
hypothesis table H-P2-0.1a..k (H-P2-0.1i = `pair_compare.json`,
`eval.compare_p2_0_1 pair`, A ↔ B), the sets by kind, the deliberate cases,
the regress against the P2-0 official RUN, the FAILs that the frozen
recorders of earlier rounds record (`p2_0_1/historical.json`,
`eval.historical_p2_0_1`, which is also the chain's printed summary), and
the decision rule of docs/rounds/P2-0.1/GOAL.md, applied in its order:

- all pass → P2-0's hypothesis is proven: spec §7.1 "Validated",
  `unverified` stays the default, the threat model with the T4 text, a
  NORTH_STAR proposal, P2-A starts;
- only 0.1d and/or 0.1d′ failed, every wrong block from collision ticks,
  and no agent block `canonical` → proven **with a documented limit**;
  otherwise as above, the limit written into THREAT_MODEL and §7.1;
- 0.1b or 0.1f failed (the rest passed or within the limit above) → P2-0's
  0b branch: `unverified` is **not** the default, the policy ships
  `canonical`, the NORTH_STAR promise narrowed by "with an attesting
  client"; P2-A starts;
- 0.1a, 0.1a′, 0.1e or 0.1g failed → a design flaw: the round fails, a
  decision request; P2-A does not start;
- 0.1h, 0.1i, 0.1j or 0.1k failed → an infrastructure flaw: the round
  fails, a decision request;
- any other failure (0.1c, regress, deliberate cases) → undefined failure:
  the round fails, a decision request.

A real-Obsidian set recorded as skipped leaves its hypotheses not measured
and the rule waits. The report carries the threat-model text and the
NORTH_STAR proposal (a proposal only: NORTH_STAR is a human decision).

Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P2-0.1/FINAL.md`.

    python -m eval.report_p2_0_1 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import historical_p2_0_1
from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P2-0.1" / "FINAL.md"
IDS = (
    "H-P2-0.1a",
    "H-P2-0.1a′",
    "H-P2-0.1b",
    "H-P2-0.1b′",
    "H-P2-0.1c",
    "H-P2-0.1d",
    "H-P2-0.1d′",
    "H-P2-0.1e",
    "H-P2-0.1f",
    "H-P2-0.1g",
    "H-P2-0.1h",
    "H-P2-0.1i",
    "H-P2-0.1j",
    "H-P2-0.1k",
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


def rows(d: Path) -> list[dict]:
    c = _load(d, "p2_0_1_checks.json")
    da = _load(d, "p2_0_1/d_a.json")
    pair = _load(d, "pair_compare.json")
    g = pair["h_p2_0_1i"]
    b, bp, cc = c["H-P2-0.1b"], c["H-P2-0.1b′"], c["H-P2-0.1c"]
    dp, e, f = c["H-P2-0.1d′"], c["H-P2-0.1e"], c["H-P2-0.1f"]
    skip_b, skip_dp, skip_e = (x.get("skipped") is True for x in (b, dp, e))
    return [
        {
            "id": "H-P2-0.1a",
            "what": "Canonical sızıntısı (P2-0 0a)",
            "value": f"{c['H-P2-0.1a']['canonical_leak']} / {c['H-P2-0.1a']['new_blocks']} yeni blok, {c['H-P2-0.1a']['scenarios']} senaryo",
            "threshold": "= 0",
            "pass": c["H-P2-0.1a"]["pass"],
            "source": "p2_0_1/d_a.json",
        },
        {
            "id": "H-P2-0.1a′",
            "what": "Canonical içerik korunumu (P2-0 0a′)",
            "value": f"{c['H-P2-0.1a′']['scenarios']} senaryo; canonical soyu değiştiren senaryo {da['scenarios_with_canonical_changed']}, değişen canonical soy {c['H-P2-0.1a′']['canonical_changed']}",
            "threshold": "200/200",
            "pass": c["H-P2-0.1a′"]["pass"],
            "source": "p2_0_1/d_a.json",
        },
        {
            "id": "H-P2-0.1b",
            "what": "Yanlış `unverified`, W sonunda (P2-0 0b; D-B test-2, gerçek Obsidian)",
            "value": "atlandı (Obsidian yok)"
            if skip_b
            else f"{b['unverified_after_window']}/{b['new_blocks']} = {_f(b['false_unverified_rate'])} ({b['sessions']} oturum, {b['saves']} kayıt, yan not {b['side_notes']})",
            "threshold": "≤ 0.01",
            "pass": b["pass"],
            "source": "p2_0_1/d_b.json",
        },
        {
            "id": "H-P2-0.1b′",
            "what": "Yanlış `unverified`, yalnız yeniden adlandırma oturumları (bilgi)",
            "value": "atlandı"
            if skip_b
            else f"{bp['unverified_after_window']}/{bp['new_blocks']} ({bp['rename_sessions']} oturum, {bp['rename_saves']} kayıt, yan notlar dahil)",
            "threshold": "raporlanır",
            "pass": None if skip_b else "info",
            "source": "p2_0_1/d_b.json",
        },
        {
            "id": "H-P2-0.1c",
            "what": "Tasdik gecikmesi p95 (P2-0 0c; D-B test-2)",
            "value": "atlandı"
            if skip_b
            else f"{cc['latency_p95_ms']} ms ({cc['latency_measured']} ölçüm; `canonical` olmayan {cc['latency_unsettled']}; yazımsız kayıt {cc['no_write']})",
            "threshold": "< 500 ms",
            "pass": cc["pass"],
            "source": "p2_0_1/d_b.json",
        },
        {
            "id": "H-P2-0.1d",
            "what": "Karışık katman doğruluğu (P2-0 0d; D-C, insan `content` gönderir)",
            "value": f"{c['H-P2-0.1d']['right']}/{c['H-P2-0.1d']['blocks']} = {_f(c['H-P2-0.1d']['accuracy'])} ({c['H-P2-0.1d']['ticks']} tik; ajan `canonical` {c['H-P2-0.1d']['agent_canonical']}; `partial` satırı {c['H-P2-0.1d']['partial_lines']})",
            "threshold": "≥ 0.99",
            "pass": c["H-P2-0.1d"]["pass"],
            "source": "p2_0_1/d_c.json",
        },
        {
            "id": "H-P2-0.1d′",
            "what": "Gerçek Obsidian çakışma: insan bloğu doğruluğu; ajan bloğu `canonical` (D-C′)",
            "value": "atlandı (Obsidian yok)"
            if skip_dp
            else f"{dp['human_canonical']}/{dp['human_blocks']} = {_f(dp['human_accuracy'])}; ajan `canonical` {dp['agent_canonical']}/{dp['agent_blocks']} ({dp['ticks']} tik)",
            "threshold": "≥ 0.95; = 0",
            "pass": dp["pass"],
            "source": "p2_0_1/d_c_prime.json",
        },
        {
            "id": "H-P2-0.1e",
            "what": "Aklama: ajan bloğu `canonical`; sahte bağlantı güncellemesi reddi (D-A′)",
            "value": "atlandı (Obsidian yok)"
            if skip_e
            else f"ajan `canonical` {e['agent_canonical']}/{e['agent_blocks']} ({e['scenarios']} senaryo; bağlantı yeniden yazıldı {e['links_rewritten']}); sahte ret {e['fakes_denied']}/{e['fakes']}",
            "threshold": "= 0; 10/10",
            "pass": e["pass"],
            "source": "p2_0_1/d_a_prime.json",
        },
        {
            "id": "H-P2-0.1f",
            "what": "Obsidian dışı insan: yanlış `unverified`; gecikme p95 (D-B2)",
            "value": f"{f['unverified_after_window']}/{f['new_blocks']} = {_f(f['false_unverified_rate'])}; p95 {f['latency_p95_ms']} ms ({f['sessions']} oturum; `canonical` olmayan {f['latency_unsettled']})",
            "threshold": "≤ 0.01; < 500 ms",
            "pass": f["pass"],
            "source": "p2_0_1/d_b2.json",
        },
        {
            "id": "H-P2-0.1g",
            "what": "Obsidian dışı ajan: canonical sızıntısı (D-A2)",
            "value": f"{c['H-P2-0.1g']['canonical_leak']} / {c['H-P2-0.1g']['new_blocks']} yeni blok, {c['H-P2-0.1g']['scenarios']} senaryo",
            "threshold": "= 0",
            "pass": c["H-P2-0.1g"]["pass"],
            "source": "p2_0_1/d_a2.json",
        },
        {
            "id": "H-P2-0.1h",
            "what": "Eski vault'lar (3 `v0.1.0` + 2 `ce4cf04`): açılır, replay eşit, `events_hash` ve `attestations.jsonl` değişmez",
            "value": c["H-P2-0.1h"]["vaults"],
            "threshold": "5/5",
            "pass": c["H-P2-0.1h"]["pass"],
            "source": "p2_0_1/d_e.json",
        },
        {
            "id": "H-P2-0.1i",
            "what": "Determinizm: çift koşturma (D-A, D-A2, D-C, günlükler bayt eşit); bütün çift kümesi",
            "value": f"günlükler eşit {g['equal']}; {pair['n_fields']} alan, fark {len(pair['differing_fields'])}; yapıtlar bayt eşit {pair['bytes_equal']}",
            "threshold": "eşit",
            "pass": g["equal"] and pair["equal"],
            "source": "pair_compare.json",
        },
        {
            "id": "H-P2-0.1j",
            "what": "Uyumluluk: mevcut 37 + C2-13",
            "value": f"{c['H-P2-0.1j']['passed']}/{c['H-P2-0.1j']['total']}, seviye {c['H-P2-0.1j']['level_claimed']}, spec {c['H-P2-0.1j']['spec_version']}",
            "threshold": "38/38",
            "pass": c["H-P2-0.1j"]["pass"],
            "source": "conformance_p1d1.json",
        },
        {
            "id": "H-P2-0.1k",
            "what": "`make verify` özetinin sayılan FAIL'i = turun sınıflandırıcısı",
            "value": f"sayılan {c['H-P2-0.1k']['counted_fails']} (özet = `eval.historical_p2_0_1`; tarihsel {c['H-P2-0.1k']['historical']})",
            "threshold": "eşit (0)",
            "pass": c["H-P2-0.1k"]["pass"],
            "source": "p2_0_1/historical.json",
        },
    ]


def _collisions_only(d: Path) -> bool:
    """0.1d/0.1d′'s wrong blocks all from collision ticks, no agent block
    `canonical` (the documented-limit branch)."""
    dc = _load(d, "p2_0_1/d_c.json")
    wrong_ok = all(w["tick"] for w in dc["wrong"]) and all(
        mode == "collide" for mode in _modes_of(d, dc["wrong"])
    )
    dcp = _load(d, "p2_0_1/d_c_prime.json")
    agent_dcp = 0 if dcp.get("skipped") else dcp["agent_canonical"]
    return wrong_ok and dc["agent_canonical"] == 0 and agent_dcp == 0


def _modes_of(d: Path, wrong: list[dict]) -> list[str]:
    from eval.datasets import p2_0_sets

    mode = {t["id"]: t["mode"] for t in p2_0_sets.d_c()["ticks"]}
    return [mode[w["tick"]] for w in wrong]


def decide(d: Path, r: list[dict], c: dict, hist: dict) -> tuple[str, list[str]]:
    by = {x["id"]: x["pass"] for x in r}
    graded = [i for i in IDS if i != "H-P2-0.1b′"]
    failed = [i for i in graded if by[i] is False]
    unmeasured = [i for i in graded if by[i] is None]
    extra = [k for k in ("regress", "tamper") if c[k]["pass"] is not True]
    if unmeasured:
        return (
            (
                f"**Gerçek Obsidian kümesi ölçülmedi → {unmeasured} ölçülmedi; karar kuralı "
                "bekler**: kümeler gerçek Obsidian'la koşulup tablo tamamlanana dek karar verilmez. "
                f"Diğer kalanlar: {failed or 'yok'}."
            ),
            failed,
        )
    design = [i for i in failed if i in ("H-P2-0.1a", "H-P2-0.1a′", "H-P2-0.1e", "H-P2-0.1g")]
    infra = [i for i in failed if i in ("H-P2-0.1h", "H-P2-0.1i", "H-P2-0.1j", "H-P2-0.1k")]
    collide = [i for i in failed if i in ("H-P2-0.1d", "H-P2-0.1d′")]
    bf = [i for i in failed if i in ("H-P2-0.1b", "H-P2-0.1f")]
    other = [i for i in failed if i not in design + infra + collide + bf]
    if not failed and not extra:
        return (
            (
                '**Hepsi geçti → P2-0 hipotezi kanıtlandı**: spec §7.1 "Validated"; '
                "`unverified` varsayılan kalır; tehdit modeli T4 metniyle; NORTH_STAR önerisi "
                "aşağıda (insan kararı); **P2-A başlar**. Bağımsız doğrulama bu RUN'ın parçası değildir."
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
            (
                "**Tanımlanmamış kalış → tur kaldı, karar talebi**: "
                f"kalan {failed}; regress/kasıtlı vakalar {extra or 'geçti'}."
            ),
            failed,
        )
    limit_ok = not collide or _collisions_only(d)
    if collide and not limit_ok:
        return (
            (
                "**Tanımlanmamış kalış → tur kaldı, karar talebi**: "
                f"{collide} kaldı ve yanlışların hepsi çakışma tiklerinden değil ya da bir ajan bloğu `canonical`."
            ),
            failed,
        )
    if bf:
        return (
            (
                f"**{bf} kaldı → GOAL P2-0'ın 0b dalı**: `unverified` varsayılan **olmaz**, "
                'politika `canonical` ile yayımlanır, NORTH_STAR vaadi "tasdikli istemci ile" '
                "ibaresiyle daraltılır (karar kaydıyla; öneri aşağıda); **P2-A başlar**."
                + (
                    f" Ayrıca {collide} belgelenmiş sınır koşulunu sağlıyor (yanlışların hepsi çakışma tiklerinden, ajan bloğu `canonical` 0)."
                    if collide
                    else ""
                )
            ),
            failed,
        )
    return (
        (
            f"**Yalnız {collide} kaldı, yanlışların tamamı çakışma tiklerinden ve ajan bloğu "
            '`canonical` = 0 → hipotez belgelenmiş sınırla kanıtlandı** ("eşzamanlı yazımda '
            'insan içeriği güvenli tarafta `unverified` kalabilir; insan terfi eder"); spec §7.1 '
            '"Validated"; `unverified` varsayılan kalır; sınır THREAT_MODEL ve spec §7.1 notuna '
            "yazılır; **P2-A başlar**."
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
    b = c["H-P2-0.1b"]
    if not b.get("skipped"):
        out += [
            "**D-B test-2 (gerçek Obsidian), oturum türlerine göre** (`p2_0_1/d_b.json`):",
            "",
            "| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |",
            "|---|---|---|---|",
            *[
                f"| {k} | {v['saves']} | {v['new_blocks']} | {v['unverified']} |"
                for k, v in sorted(b["by_kind"].items())
            ],
            "",
        ]
    f = c["H-P2-0.1f"]
    out += [
        "**D-B2 (Obsidian dışı insan), oturum türlerine göre** (`p2_0_1/d_b2.json`):",
        "",
        "| Tür | oturum | yeni blok | W sonunda `unverified` |",
        "|---|---|---|---|",
        *[
            f"| {k} | {v['sessions']} | {v['new_blocks']} | {v['unverified_after_window']} |"
            for k, v in f["by_kind"].items()
        ],
        "",
        "**D-A2 (Obsidian dışı ajan), yollara göre** (`p2_0_1/d_a2.json`):",
        "",
        "| Yol | senaryo | yeni blok | `canonical` sızıntısı |",
        "|---|---|---|---|",
        *[
            f"| {k} | {v['scenarios']} | {v['new_blocks']} | {v['canonical_leak']} |"
            for k, v in c["H-P2-0.1g"]["by_path"].items()
        ],
        "",
    ]
    dp = c["H-P2-0.1d′"]
    if not dp.get("skipped"):
        out += [
            "**D-C′ (gerçek Obsidian çakışma), gecikmeye göre** (`p2_0_1/d_c_prime.json`):",
            "",
            "| Gecikme (ms) | tik | insan bloğu | insan `canonical` | ajan `canonical` |",
            "|---|---|---|---|---|",
            *[
                f"| {k} | {v['ticks']} | {v['human_blocks']} | {v['human_canonical']} | {v['agent_canonical']} |"
                for k, v in dp["by_delay"].items()
            ],
            "",
        ]
    e = c["H-P2-0.1e"]
    if not e.get("skipped"):
        out += [
            "**D-A′ (aklama, gerçek Obsidian), türlere göre** (`p2_0_1/d_a_prime.json`):",
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
    "NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına göre:",
    "",
    '- Hipotez kanıtlandıysa (belgelenmiş sınırla ya da sınırsız): vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir (tasdikli istemciyle; insan kanalına giremeyen ajana karşı — token\'ı okuyan, insan istemcisini başlatıp süren ya da eklenti dosyalarını yazan süreç kapsam dışı)." eklenir; belgelenmiş sınır dalında: "eşzamanlı yazımda insan içeriği güvenli tarafta `unverified` kalabilir".',
    '- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical`, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).',
    "- Kaldı dallarında: vaat değişmez; öneri insanın karar talebine verdiği cevaba bağlıdır.",
]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p2_0_1_checks.json")
    hist = _load(d, "p2_0_1/historical.json")
    r = rows(d)
    assert tuple(x["id"] for x in r) == IDS
    rule, failed = decide(d, r, c, hist)
    reg, t = c["regress"], c["tamper"]
    lines = [
        "# P2-0.1 — FINAL (tasdik yolları, yama turu)",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer). Eşikler GOAL P2-0.1'inkiler (P2-0'ınkiler aynen); `W` = 0.5 s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı).",
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
        f"- P2-0 resmi RUN'ına karşı (`regress_vs_p2_0.json`, referans `{reg['reference_run']}`): {reg['n_fields']} alan, hariç dışı fark {len(reg['differing_fields'])}, karar gereği {len(reg['excluded_differences'])} alt ağaç, kullanılmayan hariç {len(reg['exclusions_without_difference'])}; uyumluluk eski testler eşit {not reg['conformance']['differing_tests']}, yeni test {reg['conformance']['new_tests_passed']}; P2-0'ın yeniden koşan kümeleri (D-A, D-E `v0.1.0`) eşit {reg['p2_0_sets']['equal']} → {_ok(reg['pass'])}.",
        f"- Kasıtlı vakalar (`p2_0_1/tamper_p2_0_1.json`): {t['rejected']} reddedildi, kontroller {t['controls']} → {_ok(t['pass'])}.",
        "",
        "## Önceki turların kaydedicileri",
        "",
        f"`p2_0_1/historical.json` (`eval.historical_p2_0_1`, referans `{hist['reference_run']}`; `make verify`'ın FAIL özeti bu sınıflandırıcının çıktısıdır): {hist['historical']} tarihsel FAIL sayılmadı; sayılan FAIL {hist['counted_fails']}.",
        "",
        *[f"- {historical_p2_0_1.label(row)}" for row in hist["fails"]],
        "",
        "## Karar kuralı",
        "",
        f"Kalan hipotezler: {', '.join(failed) if failed else 'yok'}; bağımsız doğrulama açık.",
        "",
        rule,
        "",
        "## Tehdit modeli",
        "",
        "`docs/THREAT_MODEL.md` / `docs/SECURITY.md` \"Tasdikli yazma ve insan kanalı\" ve spec §7.1 bilgilendirici notu (K-P2-01.4): insan kanalı Obsidian sürecinden geçen yazım ve `rootmark attest`'i çalıştıranın beyanıdır; bu süreci başlatabilen, hata ayıklayıcıyla (CDP) sürebilen ya da eklenti dosyalarını yazabilen her yerel süreç ve aşama 1'de token'ı okuyabilen her süreç insan kanalındadır (CDP yolu P2-0 doğrulamasında gösterildi); aşama 2 bu yolları kapatmaz; kasıtlı ajana karşı ayrım OS düzeyindedir ve kapsam dışıdır.",
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
