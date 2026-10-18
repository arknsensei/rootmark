"""P2-0 round report (TASKS Z8): `FINAL.md` from one RUN directory.

Every number is read from a file of the RUN (named next to it): the GOAL
hypothesis table H-P2-0a..h (H-P2-0g = `pair_compare.json`,
`eval.compare_p2_0 pair`, A ↔ B: the D-A and D-C logs byte for byte), the
chosen window and its tuning rows, the deliberate cases, the regress
against the P1-E.1 official RUN, the FAILs that the frozen recorders of
earlier rounds record (`p2_0/historical.json`, `eval.historical_p2_0`), and the
decision rule of docs/rounds/P2-0/GOAL.md:

- 0a ∧ 0a′ ∧ 0b ∧ 0c ∧ 0d ∧ 0f ∧ 0g ∧ 0h → the hypothesis holds: spec §7
  "Validated", `unverified` the default, the threat model updated (stage 1
  or 2 by 0e), P2-A starts;
- 0a or 0a′ failed → a design flaw: the round fails, a decision request;
- 0b failed, 0a passed → W or the client's attestation flow; a patch round;
  `unverified` is **not** the default, the policy ships `canonical`, the
  NORTH_STAR promise narrowed by "with an attesting client";
- 0e failed (or not measured) → the threat model stays "stage 1";
- any other failure → undefined failure: the round fails, a decision
  request.

The report also carries the threat-model draft and the NORTH_STAR
proposal (a proposal only: NORTH_STAR is a human decision).

Writes `<RUN_DIR>/FINAL.md` and `reports/rounds/P2-0/FINAL.md`.

    python -m eval.report_p2_0 --run-dir <RUN_DIR>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import historical_p2_0
from eval.datasets.common import ROOT, die
from eval.report_p1b import commit_of

OUT = ROOT.parent / "reports" / "rounds" / "P2-0" / "FINAL.md"
IDS = ("H-P2-0a", "H-P2-0a′", "H-P2-0b", "H-P2-0b′", "H-P2-0c", "H-P2-0d", "H-P2-0e")
IDS += ("H-P2-0f", "H-P2-0g", "H-P2-0h")


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _ok(b: bool | None) -> str:
    return "geçti" if b is True else "ölçülmedi" if b is None else "kaldı"


def rows(d: Path) -> list[dict]:
    c = _load(d, "p2_0_checks.json")
    pair = _load(d, "pair_compare.json")
    g = pair["h_p2_0g"]
    b, bp, cc = c["H-P2-0b"], c["H-P2-0b′"], c["H-P2-0c"]
    db_skip = b.get("skipped") is True
    return [
        {
            "id": "H-P2-0a",
            "what": "Canonical sızıntısı: ajan yazımından sonra `canonical` olan yeni/değişen blok",
            "value": f"{c['H-P2-0a']['canonical_leak']} / {c['H-P2-0a']['new_blocks']} yeni blok, {c['H-P2-0a']['scenarios']} senaryo",
            "threshold": "= 0",
            "pass": c["H-P2-0a"]["pass"],
            "source": "p2_0/d_a.json",
        },
        {
            "id": "H-P2-0a′",
            "what": "Canonical içerik korunumu: eski sürüm checkpoint'ten okunur, `history` silen/değiştiren aktörü (`unattested_fs`) gösterir",
            "value": f"{c['H-P2-0a′']['scenarios']} senaryo; değişen canonical soy {c['H-P2-0a′']['canonical_changed']}",
            "threshold": "200/200",
            "pass": c["H-P2-0a′"]["pass"],
            "source": "p2_0/d_a.json",
        },
        {
            "id": "H-P2-0b",
            "what": "Yanlış `unverified`: gerçek insan düzenlemesinde W sonunda `unverified` kalan blok oranı",
            "value": "atlandı (Obsidian yok)"
            if db_skip
            else f"{b['unverified_after_window']}/{b['new_blocks']} = {b['false_unverified_rate']:.4f} ({b['sessions']} oturum, {b['saves']} kayıt)",
            "threshold": "≤ 0.01",
            "pass": b["pass"],
            "source": "p2_0/d_b.json",
        },
        {
            "id": "H-P2-0b′",
            "what": "Geç tasdikle düzelen kayıt (bilgi)",
            "value": "atlandı" if db_skip else f"{bp['raised_late']}/{bp['saves']} kayıt",
            "threshold": "raporlanır",
            "pass": None if db_skip else True,
            "source": "p2_0/d_b.json",
        },
        {
            "id": "H-P2-0c",
            "what": "Tasdik gecikmesi: kayıt → `canonical` p95",
            "value": "atlandı"
            if db_skip
            else f"{cc['latency_p95_ms']} ms ({cc['latency_measured']} ölçüm; yazımsız kayıt {cc['no_write']})",
            "threshold": "< 500 ms",
            "pass": cc["pass"],
            "source": "p2_0/d_b.json",
        },
        {
            "id": "H-P2-0d",
            "what": "Karışık (D-C): katman doğruluğu",
            "value": f"{c['H-P2-0d']['right']}/{c['H-P2-0d']['blocks']} = {c['H-P2-0d']['accuracy']:.4f} ({c['H-P2-0d']['ticks']} tik)",
            "threshold": "≥ 0.99",
            "pass": c["H-P2-0d"]["pass"],
            "source": "p2_0/d_c.json",
        },
        {
            "id": "H-P2-0e",
            "what": "Anahtar zinciri: terminal sürecinin gizi okuyamadığı deneme oranı (alt-hipotez)",
            "value": "ölçülmedi: " + c["H-P2-0e"]["reason"],
            "threshold": "20/20",
            "pass": c["H-P2-0e"]["pass"],
            "source": "p2_0_checks.json (Z6)",
        },
        {
            "id": "H-P2-0f",
            "what": "Eski vault'lar (`v0.1.0`): açılır, replay eşit, `events_hash` değişmez, katmanlar korunur",
            "value": c["H-P2-0f"]["vaults"],
            "threshold": "3/3",
            "pass": c["H-P2-0f"]["pass"],
            "source": "p2_0/d_e.json",
        },
        {
            "id": "H-P2-0g",
            "what": "Determinizm: çift koşturma, D-A ve D-C `events.jsonl` + `attestations.jsonl` bayt eşit; bütün çift kümesi",
            "value": f"günlükler eşit {g['equal']}; {pair['n_fields']} alan, fark {len(pair['differing_fields'])}; yapıtlar bayt eşit {pair['bytes_equal']}",
            "threshold": "eşit",
            "pass": g["equal"] and pair["equal"],
            "source": "pair_compare.json",
        },
        {
            "id": "H-P2-0h",
            "what": "Uyumluluk: C2-11, C2-12, C3-08 + mevcut 34",
            "value": f"{c['H-P2-0h']['passed']}/{c['H-P2-0h']['total']}, seviye {c['H-P2-0h']['level_claimed']}, spec {c['H-P2-0h']['spec_version']}",
            "threshold": "37/37",
            "pass": c["H-P2-0h"]["pass"],
            "source": "conformance_p1d1.json",
        },
    ]


def decide(r: list[dict], c: dict, hist: dict) -> tuple[str, list[str]]:
    by = {x["id"]: x["pass"] for x in r}
    core = ["H-P2-0a", "H-P2-0a′", "H-P2-0b", "H-P2-0c", "H-P2-0d", "H-P2-0f", "H-P2-0g"]
    core.append("H-P2-0h")
    failed = [i for i in core if by[i] is not True]
    stage = "aşama 2" if by["H-P2-0e"] is True else "aşama 1"
    extra = [k for k in ("regress", "tamper") if c[k]["pass"] is not True]
    counted = hist["counted"]
    if not failed and not extra and not counted:
        return (
            (
                "**Hipotez kanıtlandı** (0a ∧ 0a′ ∧ 0b ∧ 0c ∧ 0d ∧ 0f ∧ 0g ∧ 0h): spec §7 "
                f'"Validated"; `unverified` varsayılan olur; tehdit modeli **{stage}** '
                "(0e geçmedi ya da ölçülmedi → aşama 1: token okuyan ajan tasdik üretebilir); "
                "P2-A başlar. Bağımsız doğrulama bu RUN'ın parçası değildir."
            ),
            failed,
        )
    if by["H-P2-0a"] is not True or by["H-P2-0a′"] is not True:
        return (
            "**0a veya 0a′ kaldı → tasarım kusuru; tur kaldı, karar talebi.**",
            failed,
        )
    if failed == ["H-P2-0b"] or set(failed) <= {"H-P2-0b", "H-P2-0c"} and "H-P2-0b" in failed:
        only_b = failed == ["H-P2-0b"]
        return (
            (
                "**0b kaldı, 0a geçti → W ayarı ya da istemci tasdik akışı; bir yama turu.** "
                "`unverified` varsayılan **olmaz**, politika `canonical` ile yayımlanır, "
                'NORTH_STAR vaadi "tasdikli istemci ile" ibaresiyle daraltılır (öneri, insan kararı). '
                f"Tehdit modeli {stage}."
                + (
                    ""
                    if only_b
                    else f" Ayrıca kalan: {[i for i in failed if i != 'H-P2-0b']} (karar kuralı 0b dalını uygular; 0c bu dalda yama turunun konusudur)."
                )
                + (f" Kayıt dışı: {extra}." if extra else "")
                + (f" Sayılan önceki-tur FAIL'leri: {counted}." if counted else "")
            ),
            failed,
        )
    return (
        (
            "**Tanımlanmamış kalış → tur kaldı, karar talebi**: "
            f"kalan hipotezler {failed or 'yok'}; regress/tamper {extra or 'geçti'}; "
            f"sayılan önceki-tur FAIL'leri {counted or 'yok'}."
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


def _window(d: Path, c: dict) -> list[str]:
    t = _load(d, "p2_0/tuning.json")
    out = [
        f"Seçilen `W` = **{c['window']['window_s']} s** ({c['window']['chosen_by']}; ayar seti BLAKE3 `{c['window']['tune_set_blake3']}`; kayıt `docs/rounds/P2-0/progress/Z5_tuning.json`).",
        "",
        "| W (s) | kayıt | yeni blok | W sonunda `unverified` | oran | geç yükselme | p95 (ms) |",
        "|---|---|---|---|---|---|---|",
    ]
    for w in t.get("windows", []):
        out.append(
            f"| {w['window_s']} | {w['saves']} | {w['new_blocks']} | {w['unverified_after_window']} | "
            f"{w['false_unverified_rate']:.4f} | {w['raised_late']} | {w['latency_p95_ms']} |"
        )
    return out


def _by_kind(c: dict) -> list[str]:
    b = c["H-P2-0b"]
    if b.get("skipped"):
        return ["D-B atlandı."]
    out = [
        "| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |",
        "|---|---|---|---|",
    ]
    for k, v in sorted(b["by_kind"].items()):
        out.append(f"| {k} | {v['saves']} | {v['new_blocks']} | {v['unverified']} |")
    return out


THREAT = [
    "- **Aşama 1 (etkin, `status.attestation.stage = 1`):** tasdik, token'la açılan bir oturum anahtarının MAC'idir (HTTP-only; MCP'de tasdik aracı yoktur — kasıtlı vakalar). İnsan kanalı Obsidian sürecinden geçen yazımlardır (eklentinin `adapter.write/append/rename` gözlemi; aynı süreçteki başka eklentiler de bu kanaldadır). **`.rootmark/server.token`'ı okuyabilen bir ajan oturum açıp tasdik üretebilir**; aşama 1 dikkatsiz ajana (dosyaya doğrudan yazan) karşı korur, kasıtlı ajana karşı korumaz.",
    '- **Tasdiksiz yazım** (`unverified`, aktör `unattested_fs`): aramada varsayılan görünmez, `include_proposal` ile etiketli; insan "Mine"/`promote` ile yükseltir. Toplu tasdik (`rootmark attest --all`) yalnız günlüğe yazar, hiçbir alımı yükseltmez.',
    "- **Bilinen boşluklar:** Obsidian'ın bağlantı güncellemeleri `adapter.write` dışından yazılır ve tasdik edilmez (ASSUMPTIONS Z5-5; yeniden adlandırmada yanlış `unverified`); ajanın insanın kaydedilmemiş değişikliği olan nota yazması ve Obsidian'ın birleştirip kaydetmesi ajan bloklarını tasdikli yapabilir (Z3 bulgu 12).",
    "- **Aşama 2 (ölçülmedi, Z6):** eklentinin tek anahtar zinciri ilkeli Electron `safeStorage`; açık sorunlar: sunucu tarafında sırrın ajandan uzak saklanması, açık anahtar kaydının insana özgü bir onaya bağlanması, `.obsidian/plugins/**` yazımıyla eklenti enjeksiyonu, ve terminalin Obsidian'ı `--remote-debugging-port` ile başlatıp CDP üzerinden `safeStorage`'ı çağırması (istem yok; anahtar zinciri ACL'si bunu kapatmaz). Sonda `scripts/keychain_probe.sh` insanın koşturması için hazır.",
]

NORTH_STAR = [
    "NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına göre:",
    "",
    '- Hipotez kanıtlandıysa: vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir (aşama 1: tasdikli istemci ile; token okuyan kasıtlı ajan kapsam dışı)." eklenir.',
    '- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical` kalır, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).',
]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    a = ap.parse_args(argv)
    d = a.run_dir
    man = _load(d, "manifest.json")
    c = _load(d, "p2_0_checks.json")
    hist = _load(d, "p2_0/historical.json")
    r = rows(d)
    assert tuple(x["id"] for x in r) == IDS
    rule, failed = decide(r, c, hist)
    reg, t = c["regress"], c["tamper"]
    lines = [
        "# P2-0 — FINAL (tasdikli yazma)",
        "",
        f"- RUN: `{d.name}`; commit `{commit_of(man)}`",
        "- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer).",
        "",
        "## Hipotezler",
        "",
        *_table(r),
        "",
        "## Pencere (W)",
        "",
        *_window(d, c),
        "",
        "## D-B oturum türlerine göre (test)",
        "",
        *_by_kind(c),
        "",
        "## Regresyon ve kasıtlı vakalar",
        "",
        f"- P1-E.1 resmi RUN'ına karşı (`regress_vs_p1e1.json`, referans `{reg['reference_run']}`): {reg['n_fields']} alan, hariç dışı fark {len(reg['differing_fields'])}, karar gereği {len(reg['excluded_differences'])} alt ağaç, kullanılmayan hariç {len(reg['exclusions_without_difference'])}; uyumluluk eski testler eşit {not reg['conformance']['differing_tests']}, yeni testler {reg['conformance']['new_tests_passed']} → {_ok(reg['pass'])}.",
        f"- Kasıtlı vakalar (`p2_0/tamper_p2_0.json`): {t['rejected']} reddedildi, kontroller {t['controls']} → {_ok(t['pass'])}.",
        "",
        "## Önceki turların kaydedicileri",
        "",
        f"`p2_0/historical.json` (`eval.historical_p2_0`, referans: `{hist['reference_run']}`): {hist['historical']} tarihsel FAIL sayılmadı; sayılan FAIL {hist['counted_fails']}.",
        "",
        *[f"- {historical_p2_0.label(row)}" for row in hist["fails"]],
        "",
        "## Karar kuralı",
        "",
        f"Kalan hipotezler: {', '.join(failed) if failed else 'yok'}; bağımsız doğrulama açık.",
        "",
        rule,
        "",
        "## Tehdit modeli (taslak)",
        "",
        *THREAT,
        "",
        "## NORTH_STAR önerisi",
        "",
        *NORTH_STAR,
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
