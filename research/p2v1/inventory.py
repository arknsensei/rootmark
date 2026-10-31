"""P2-V.1/E3 (TASKS E3; GOAL P2-V.1 İlkeler, Kapsam 4): the round's
inventory, `reports/rounds/P2-V.1/inventory.json`, written once before the
round's tool changes and never changed after (pre-registration).

The P2-V inventory (`reports/rounds/P2-V/inventory.json`, pre-registered in
P2-V/V1) is carried over entry for entry; this round adds, by name and
without a pattern:

- **R14** (`commit_bound`): the release checklist's R14 `detail` names the
  newest round's `ROUND_START` (K-P2V-T1);
- **B6**: the entries P2-V named by array position (O13 `items›15`, H1
  `items›17`, C2 `items›3`) name their checklist item by its id (`item`:
  R15, R17, R4a), resolved to the position in each record by `eval.tier2`;
  the two real-Obsidian records are one O record each (P2-V listed them
  twice, also as D byte artefacts): their files stay byte-compared and
  set aside by name (`byte_exclusions`), as their fields are (O1, O2);
- **B3** (`derived_criteria`): the deterministic half of P1-E.1's
  `tamper` — `p1e1/tamper_p1e1.json`'s `pass` — is a criterion of its own,
  `p1e1_checks.json:tamper·det`, counted by tier 2 (the criterion `tamper`
  stays O: its real-Obsidian half is the Mac set's);
- **B2** (`criterion_fields`): a Z/O criterion of `tier2_set_aside_criteria`
  sets aside only the fields named for it here — for `C1-8` its regress
  entries of the E1 fields below and of its frozen reference's
  platform-bound values — not its whole record (the rankings' ids and order
  inside it stay compared);
- **E1's expected difference families** (`e1_fields`, `e1_bytes`;
  `reports/rounds/P2-V.1/e1/expected_families.md`, committed in E1 before
  any tier-2 run of the round): the fields whose value comes from the
  vectors' numerics or names the embedder — every flat-mode score leaf of
  `p1c_index.json`'s rankings (position 3 of each hit of the `flat` mode
  of `rankings_200`, `rankings_after_compact›p1c` and `…›p1c-hot`), the
  index files' digests, C-10's `min_cosine`, D1-3's digests, D-TR's vector
  and embedder digests, and every string leaf of a compared record that
  holds an `embedder_id`. Enumerated from the record's structure (all such
  leaves, differing or not), not from observed differences. They are D in
  the pair (two chains must agree on them) and named only for the
  comparisons against records of the ONNX Runtime numerics: the old chain
  (`eval.tier2 equiv`, V1-6), the frozen reference RUNs embedded in the
  regresses (tier 2's derived-FAIL rule, B1), the Mac set's C1-8 reading;
- **CPU** (`cpu_fields`): each chain's processor model in `host.json`
  (information; the comparison job writes both and whether they differ);
- **Mac set** (`mac_reading`, E4's reading pre-registered here, K-P2V.3 T2
  (b)+(c)): deterministic O criteria run once; criteria closed by decision
  (E-7, E1-4, H-P2-0.3b) are reported as `tarihsel` by record equality.

    python -m p2v1.inventory --run <a new-chain RUN> --out ../reports/rounds/P2-V.1/inventory.json

(from `research/`, `PYTHONPATH=.`). The RUN is read for the structure of
the records (which leaves exist), never for their values.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "reports/rounds/P2-V/inventory.json"
OLD_IDS = ("minilm-l6-v2/e73168f9", "multilingual-minilm-l12-v2/ca55a866")
TAG = "+fixed1"


def key(path: tuple) -> str:
    """The comparator's field key (`compare_runs._key`: dots as `·`)."""
    return "›".join(str(p).replace(".", "·") for p in path)


def leaves(x, p=()):
    if isinstance(x, dict):
        for k, v in x.items():
            yield from leaves(v, (*p, k))
    elif isinstance(x, list):
        for i, v in enumerate(x):
            yield from leaves(v, (*p, i))
    else:
        yield p, x


def holds_embedder_id(v) -> bool:
    return isinstance(v, str) and (any(i in v for i in OLD_IDS) or TAG in v)


def e1_fields(run: Path, compared: list[str]) -> list[dict]:
    out: list[dict] = []

    def add(family: str, file: str, path: tuple, why: str) -> None:
        out.append({"family": family, "file": file, "field": key(path), "why": why})

    idx = json.loads((run / "p1c_index.json").read_text(encoding="utf-8"))
    struct = [
        (("rankings_200", "flat"), idx["rankings_200"]["flat"]),
        (("rankings_after_compact", "p1c", "flat"), idx["rankings_after_compact"]["p1c"]["flat"]),
        (
            ("rankings_after_compact", "p1c-hot", "flat"),
            idx["rankings_after_compact"]["p1c-hot"]["flat"],
        ),
    ]
    for base, queries in struct:
        for q, hits in enumerate(queries):
            for r, hit in enumerate(hits):
                if len(hit) != 4 or not isinstance(hit[3], float):
                    raise SystemExit(
                        f"p1c_index.json {base} {q} {r}: not [doc, chunk, layer, score]"
                    )
                add("E1-S", "p1c_index.json", (*base, q, r, 3), "düz kipin kosinüs skoru")
    for group in ("index_files_build", "index_files_compact", "index_files_final"):
        for f in sorted(idx[group]):
            if f.endswith(("vectors.f32", "embedders.jsonl")) or f == "manifest.json":
                add(
                    "E1-V",
                    "p1c_index.json",
                    (group, f),
                    "vektör dosyasının ya da onu özetleyen manifest'in blake3'ü",
                )
    add("E1-C10", "p1c_checks.json", ("C-10", "min_cosine"), "C-10'un en küçük kosinüsü")
    for h in ("before_hash", "after_hash"):
        add("E1-D13", "p1d1/d1_3.json", (h,), "D1-3'ün arama sonuçlarının (skorlarıyla) özeti")
    dtr = json.loads((run / "p2l/d_tr.json").read_text(encoding="utf-8"))
    for b in sorted(dtr["builds"]):
        for f in ("vectors_blake3", "embedders_blake3"):
            add(
                "E1-DTR",
                "p2l/d_tr.json",
                ("builds", b, "manifest", f),
                "D-TR kurulumunun vektör / kimlik dosyası özeti",
            )
    # E1-R: the regresses' counters of excluded differences of a regress
    # whose differences are E1 fields (`excluded_differences›<entry>›leaves`)
    carriers = (
        "p1c1_checks.json:C1-8",
        "p1d_checks.json:D-9",
        "p1d1_checks.json:D1-12",
        "p1e_checks.json:E-9",
        "p1e1_checks.json:E1-6",
    )
    for name in compared:
        p = run / name
        if not name.endswith("_checks.json") or not p.is_file():
            continue
        doc = json.loads(p.read_text(encoding="utf-8"))
        for path, _v in leaves(doc):
            if path[-1:] != ("leaves",) or "excluded_differences" not in path:
                continue
            i = len(path) - 1 - path[::-1].index("excluded_differences")
            entry = str(path[i + 1]) if i + 1 < len(path) else ""
            # the entry is the carrier regress itself (`<criterion>` or
            # `<criterion>/regress…`), not a path inside it
            own = any(
                entry == c or (entry.startswith(c + "/regress") and "/" not in entry[len(c) + 1 :])
                for c in carriers
            )
            if "differing_fields" in path[:i] or not own:
                continue
            add(
                "E1-R",
                name,
                path,
                "kenara alınan farkların yaprak sayısı; kenara alınan girdi E1 alanlarını taşıyan bir regress'tir",
            )
    # every string leaf of a compared record that holds an embedder id
    for name in compared:
        p = run / name
        if not p.is_file() or not name.endswith(".json"):
            continue
        try:
            doc = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        for path, v in leaves(doc):
            if holds_embedder_id(v):
                add("E1-E", name, path, "`embedder_id` dizgesi (`+fixed1`, ASSUMPTIONS E1-1)")
    seen, uniq = set(), []
    for e in out:
        k = (e["file"], e["field"])
        if k not in seen:
            seen.add(k)
            uniq.append(e)
    for i, e in enumerate(uniq, 1):
        e["id"] = f"E1F{i}"
    return uniq


def e1_bytes(run: Path, byte_paths: list[str]) -> list[dict]:
    out = []
    for path in byte_paths:
        p = run / path
        if not p.is_file():
            continue
        text = p.read_text(encoding="utf-8", errors="replace")
        fam = None
        if path == "p1d1/d1_3.json":
            fam = ("E1-D13", "D1-3'ün arama sonuçları skorlarıyla")
        elif any(i in text for i in OLD_IDS) or TAG in text:
            fam = ("E1-E", "`embedder_id` dizgesini taşır")
        if fam:
            out.append({"family": fam[0], "path": path, "why": fam[1]})
    for i, e in enumerate(out, 1):
        e["id"] = f"E1B{i}"
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", type=Path, required=True, help="a new-chain RUN (structure only)")
    ap.add_argument(
        "--compared",
        type=Path,
        required=True,
        help="a compare_p2r pair result (its file and byte lists)",
    )
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args()
    base = json.loads(BASE.read_text(encoding="utf-8"))
    cmp_ = json.loads(a.compared.read_text(encoding="utf-8"))
    inv = json.loads(json.dumps(base))  # every P2-V entry, as it is
    inv["round"] = "P2-V.1"
    inv["task"] = "P2-V.1/E3"
    inv["round_start"] = "749a4a5"
    inv["base"] = {
        "file": "reports/rounds/P2-V/inventory.json",
        "rule": "P2-V'nin envanteri aynen taşınır (her girdi, kimliği ve gerekçesiyle); bu dosya yalnız `p2v1_changes`'te adı geçen değişiklikleri ve yeni bölümleri ekler",
    }
    # B6: by item id, not array position
    named = {"O13": "R15", "H1": "R17", "C2": "R4a"}
    for group in ("pair_exclusions", "host_fields", "commit_bound"):
        for e in inv[group]:
            if e.get("id") in named:
                rest = e["field"].split("›", 2)
                if rest[:1] != ["items"]:
                    raise SystemExit(f"{e['id']}: {e['field']} is not an items entry")
                e["item"] = named[e["id"]]
                e["field_in_item"] = rest[2] if len(rest) > 2 else None
                e["p2v_field"] = e.pop("field")
                e["field"] = None
                if e["id"] == "C2":
                    e["why"] = e["why"].replace("R4 maddesinin", "R4a maddesinin")
    inv["commit_bound"].append(
        {
            "id": "C3",
            "file": "p1e1/release/release_check.json",
            "field": None,
            "item": "R14",
            "field_in_item": "detail",
            "why": "R14'ün `detail`'i en yeni turun `ROUND_START`'ını adlandırır (`newest round start <sha>`): girdisi depodaki tur kayıtlarıdır (K-P2V-T1; P2-V'de ön-kayıtta yoktu, V-3'ün B tarafındaki tek adı geçmeyen fark)",
        }
    )
    # B6: the real-Obsidian records once (O), bytes set aside by name
    inv["records"] = [
        r
        for r in inv["records"]
        if not (
            r.get("bytes")
            and r["file"]
            in ("p1e1/release/plugin_e2e_real.json", "p1e1/release/access_log_real.json")
        )
    ]
    for i, path in enumerate(
        ("p1e1/release/plugin_e2e_real.json", "p1e1/release/access_log_real.json"), 14
    ):
        if not any(e["path"] == path for e in inv["byte_exclusions"]):
            inv["byte_exclusions"].append(
                {
                    "id": f"O{i}",
                    "path": path,
                    "class": "O",
                    "set": "O-REL",
                    "why": "gerçek Obsidian kaydının baytları (O1/O2'nin dosyası; P2-V'de ayrıca D bayt artefaktı olarak da listelenmişti, B6)",
                }
            )
    # B3
    inv["derived_criteria"] = [
        {
            "id": "p1e1_checks.json:tamper·det",
            "file": "p1e1/tamper_p1e1.json",
            "field": "pass",
            "class": "D",
            "of": "p1e1_checks.json:tamper",
            "why": "P1-E.1 `tamper` = `tamper_p1e1.json` `pass` ∧ `tamper_p1e1_e2e.json` `pass` (`run_p1e1_checks.py`); ikinci yarı gerçek Obsidian (O, Mac kümesi), birinci yarı deterministik: kademe 2 onu kendi kimliğiyle sayar (gözden geçirici B3)",
        }
    ]
    # E1 families
    fields = e1_fields(a.run, list(cmp_["files"]))
    inv["e1_fields"] = fields
    inv["e1_bytes"] = e1_bytes(a.run, sorted(cmp_["bytes"]))
    inv["e1_families"] = {
        "source": "reports/rounds/P2-V.1/e1/expected_families.md",
        "use": "pair'de D (iki zincir eşit olmalı); yalnız ONNX Runtime sayısalının kayıtlarına karşı karşılaştırmalarda adlı: eski zincir (`tier2 equiv`, V1-6), regress'lerin gömdüğü dondurulmuş referans RUN'lar (türev FAIL kuralı, B1), Mac kümesinin C1-8 okuması",
        "counts": {
            f: sum(e["family"] == f for e in fields) for f in sorted({e["family"] for e in fields})
        },
    }
    inv["observed_outside_e1"] = [
        {
            "file": "p1e1/release/release_check.json",
            "item": "R7",
            "field_in_item": "detail",
            "observed": "kademe 2 gözlem koşusu 41215091092 (E1 commit'i f25cbaa) ↔ P2-V V0m 40613392932: `rootmark_bindings-0.2.0-cp311-abi3-linux_x86_64.whl` → `…-manylinux_2_34_x86_64.whl`",
            "mechanism": "E1 `ort`'u (ONNX Runtime, C++ ve libstdc++ bağlantısı) çıkardı; maturin uzantıyı artık manylinux 2.34 politikasıyla etiketleyebiliyor. İki zincir arasında aynı (D).",
            "status": "E1'in ön-kayıtlı ailelerinde (`expected_families.md`) YOK: araçlar bu alanı kenara almaz ve adlandırılmış saymaz; V1-6 onu adı geçmeyen fark olarak okur (bilgi kaydıdır, ad değildir)",
        }
    ]
    # B2: a Z/O criterion sets aside only its named fields
    plat = [
        ("rankings_after_compact", "p1c-hot", "lexical", 47, 4, 3),
        ("rankings_after_compact", "p1c-hot", "lexical", 47, 5, 3),
        ("rankings_after_compact", "p1c-hot", "lexical", 47, 8, 3),
        ("rankings_after_compact", "p1c-hot", "lexical", 120, 9, 3),
    ]
    inv["criterion_fields"] = {
        "p1c1_checks.json:C1-8": {
            "why": "C1-8'in regress'i P1-C resmî RUN'ına (ORT, yayım Mac'i) karşıdır. Kenara alınan yalnız: E1 alanlarının (`e1_fields`) ve P2-V'nin Z/O alanlarının gömülü girdileri, aşağıdaki `fields` (C1-8'in kendi hükmü ve yayım Mac'ine bağlı dört sözcüksel skor). Sıralamaların kimlik ve sırası karşılaştırmada kalır (gözden geçirici B2). Yalnız kademe 2'nin türev FAIL kuralında kullanılır; Mac kümesinin C1-8 okuması yalnız `e1_fields`'ı kullanır (Mac'te sözcüksel skorlar referansla aynıdır).",
            "fields": [
                {
                    "file": "p1c1_checks.json",
                    "field": "C1-8›pass",
                    "why": "C1-8'in hükmü: referansa karşı farkların fonksiyonu",
                },
                {
                    "file": "p1c1_checks.json",
                    "field": "C1-8›regress›equal",
                    "why": "C1-8'in regress'inin eşitlik özeti",
                },
                *[
                    {
                        "file": "p1c_index.json",
                        "field": key(p),
                        "why": "sıcak katmanın sözcüksel BM25 skoru, son ULP'de platform `libm`'ine bağlı (macOS ↔ glibc; Linux koşucuları arasında eşit). P2-V'den beri: eski zincir V0m 40613392932'de ve E1'in gözlem koşusu 41215091092'de aynı dört yaprak; E1'den bağımsız (yayım Mac'ine bağlı O değeri, P2-V envanterinin O sınıfı tanımı)",
                    }
                    for p in plat
                ],
            ],
        }
    }
    inv["cpu_fields"] = [
        {
            "file": "host.json",
            "field": "cpu_model",
            "class": "bilgi",
            "why": "her zincirin işlemci modeli (`lscpu` Model name); karşılaştırma işi ikisini ve aynı/farklı olduğunu `tier2.json`'a hükümle yazar (K-P2V-T1 (c) köprüsü); RUN'ın kaydı değildir, karşılaştırmaya girmez",
        }
    ]
    inv["mac_reading"] = {
        "single_run": [
            {
                "criterion": "p1c1_checks.json:C1-8",
                "why": "deterministik; 3 tekrar bilgi katmaz (K-P2V.3 T2 (b)); okuma: farkları ⊆ `criterion_fields` C1-8",
            },
            {"criterion": "p1e_checks.json:E-7", "why": "deterministik (yerel yayım kaydı)"},
            {"criterion": "p1e1_checks.json:E1-4", "why": "deterministik (yayım kontrol listesi)"},
        ],
        "historical": [
            {
                "criterion": "p1e_checks.json:E-7",
                "decisions": ["K-E5-T", "K-E1.1"],
                "reference": "docs/rounds/P1-E/progress/E5_release.json",
                "reading": "kayıt eşitliği: hedef başına sonuç referansla aynı (2/4)",
            },
            {
                "criterion": "p1e1_checks.json:E1-4",
                "decisions": ["K-E1.4"],
                "reading": "kayıt eşitliği: kontrol listesinin maddelerinin `ok`'u, tarihsel maddeler (R4, R15, R17) dışında geçer; tarihsel maddeler referansla aynı",
            },
            {
                "criterion": "p2_0_3_checks.json:H-P2-0.3b",
                "decisions": ["K-P2-03.1"],
                "reading": "kalış korunuyor (`pass` false, referansla aynı); değeri raporlanır (gerçek Obsidian, yarışa bağlı: kayıt baytları tekrar eşit olamaz)",
            },
        ],
        "rule": "tarihsel kriter ham eşikle değil kayıt eşitliğiyle raporlanır ve kümenin hükmüne girmez; deterministik O kriterleri bir kez koşar ve geçmelidir; Z kümeleri 3 tekrar, değerleriyle (eşikleri kademe hükmüne girmez, K-P2V.3)",
    }
    inv["p2v1_changes"] = [
        "R14: `commit_bound` C3",
        "B6: O13/H1/C2 madde kimliğiyle (`item`), gerçek Obsidian kayıtları tek O kaydı; baytları `byte_exclusions` O14/O15",
        "B3: `derived_criteria` `p1e1_checks.json:tamper·det`",
        "B2: `criterion_fields` (C1-8)",
        "E1: `e1_fields`, `e1_bytes`, `e1_families`",
        "CPU: `cpu_fields`",
        "Mac: `mac_reading` (E4'ün okuması)",
        "B1 bir kural değişikliğidir (türev koruması kriterin tam fark listesine bakar), envanterde girdi değildir: `rules` ek satırı",
    ]
    inv["rules"] = [
        *inv["rules"],
        "P2-V.1 (B1): türev koruması sınıflandırıcının kırpılmış listesine değil kriterin kendi kaydının, sınıflandırıcının referans RUN'ına karşı tam fark listesine (aynı izin listesiyle) bakar; her fark gömülü kuralla kenara alınan bir Z/O alanı, bir E1 alanı ya da kenara alınan bir kriterin `criterion_fields`'ı değilse FAIL sayılır.",
        "P2-V.1 (E1): `e1_fields`/`e1_bytes` iki zincir arasında D'dir (kenara alınmaz); yalnız ONNX Runtime sayısalının kayıtlarına karşı karşılaştırmalarda adlıdır.",
    ]
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(inv, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(
        f"inventory: {len(fields)} e1_fields {inv['e1_families']['counts']}, {len(inv['e1_bytes'])} e1_bytes"
    )


if __name__ == "__main__":
    main()
