"""P2-Q Turkish query pre-registration (K-P2Q.3): writes
`research/p2q/scenario_tr.json`. The words come from general Turkish
vocabulary, not from any corpus; nothing here reads a corpus.

    python -m p2q.make_scenario_tr [--check]
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

OUT = Path(__file__).with_name("scenario_tr.json")

# (lower case, Turkish upper case, title case)
FOLD = [
    ("istanbul", "İSTANBUL", "İstanbul"),
    ("ışık", "IŞIK", "Işık"),
    ("şirket", "ŞİRKET", "Şirket"),
    ("değişiklik", "DEĞİŞİKLİK", "Değişiklik"),
    ("güncelleme", "GÜNCELLEME", "Güncelleme"),
    ("çalışma", "ÇALIŞMA", "Çalışma"),
]
# ASCII capital I is the dotless ı's capital in Turkish folding
TRAP = [("ISTANBUL", "ıstanbul"), ("ILISKI", "ılıskı")]
# accentless spelling of an accented word: (accentless, accented)
ACCENTLESS = [
    ("sirket", "şirket"),
    ("degisiklik", "değişiklik"),
    ("guncelleme", "güncelleme"),
    ("calisma", "çalışma"),
    ("isik", "ışık"),
    ("ogrenme", "öğrenme"),
]
MIXED = [
    "şirket company",
    "güncelleme update notes",
    "faiz rate decision",
    "toplantı meeting notes",
]
SEMANTIC = [
    "proje planındaki değişiklikler",
    "veri toplama yöntemi",
    "kullanıcı geri bildirimi",
    "haftalık rapor özeti",
]


def build() -> dict:
    qs = []
    n = 0

    def add(**kw):
        nonlocal n
        n += 1
        qs.append({"id": f"tr{n:02d}", **kw})

    for low, up, title in FOLD:
        add(group="fold", mode="lexical", query=low, base=low, criterion="TR-2")
        add(group="fold", mode="lexical", query=up, base=low, criterion="TR-1,TR-2")
        add(group="fold", mode="lexical", query=title, base=low, criterion="TR-1,TR-2")
    for q, folded in TRAP:
        add(group="ascii_i", mode="lexical", query=q, folded=folded, criterion="TR-2")
    for plain, accented in ACCENTLESS:
        add(group="accentless", mode="lexical", query=plain, accented=accented, criterion="TR-2")
    for plain, accented in ACCENTLESS[:2]:
        add(
            group="accentless_hybrid",
            mode="hybrid",
            query=plain,
            accented=accented,
            criterion="report",
        )
    for q in MIXED:
        add(group="mixed", mode="hybrid", query=q, criterion="report")
    for q in SEMANTIC:
        add(group="semantic", mode="hybrid", query=q, criterion="report")
    return {
        "version": "p2q-scenario-tr/1",
        "decision": "K-P2Q.3",
        "corpora": ["obsidian", "qorbit"],
        "recipe": "tr",
        "k": {"lexical": 1000, "hybrid": 10},
        "folding_rule": "uni_fold_tr: NFC; İ→i, I→ı; then Unicode lower case; tokens are maximal "
        "runs of alphabetic/numeric characters; no diacritic folding",
        "pass": [
            "fresh copy",
            "ingest-dir",
            "recipe tr",
            "index build",
            "serve --http 127.0.0.1:0 (no MCP, no watch)",
            "queries (default search)",
            "SIGTERM",
        ],
        "criteria": {
            "TR-1": "upper-case and title-case variants: lexical hits (document, chunk, order) "
            "equal to the lower-case form's",
            "TR-2": "every lexical query's document set equals the documents of the copy that "
            "contain its folded token (oracle: the same rule applied locally to each "
            "document's text)",
            "report": "accentless vs accented: documents lost (lexical) and top-10 overlap "
            "(hybrid); mixed and semantic: hit counts and overlaps; no threshold",
        },
        "reading": "TR-1/TR-2 failure is a finding (FINDINGS.md, target P2-C); GOAL Q-1..Q-11 "
        "thresholds unchanged",
        "queries": qs,
    }


def main(argv=None) -> int:
    text = json.dumps(build(), ensure_ascii=False, indent=1, sort_keys=True) + "\n"
    if argv and "--check" in argv or "--check" in sys.argv[1:]:
        same = OUT.read_text(encoding="utf-8") == text
        print("make_scenario_tr: " + ("same" if same else "DIFFERENT"))
        return 0 if same else 1
    OUT.write_text(text, encoding="utf-8")
    print(f"make_scenario_tr: wrote {OUT.name} ({len(build()['queries'])} queries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
