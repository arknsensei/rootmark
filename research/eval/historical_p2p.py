"""Historical FAILs of P2-P (K-E1.4 carried to P2-P; TASKS P5): the round's
classifier, which `scripts/verify.sh` prints as its FAIL summary, against
the **P2-L official RUN** (`reports/rounds/P2-L/runs/20261024T184609Z-ced61f9`).

The labels and their rules are P2-A's (`eval.historical_p2a.classify`,
unchanged): `historical` (listed in `historical_fails_p2p.json`, the record
equal to the reference's but the declared `changes`, each with its
decision), `migration_required` (listed in `migration_required_fails_p2p.json`,
every listed path that differs holding its expected new value), a listed
`not_measured` set, and `fail` — anything else, never hidden. The listings
start from P2-L's (the same FAILs, now against P2-L's RUN, which holds
them) and are completed from a development RUN before the official RUN
(P5.md); this round's own record (`p2p_checks.json`) is absent from the
reference and counts if it fails.

    python -m eval.historical_p2p --run-dir <RUN_DIR> [--out <json>] [--summary]

Exit 0 when the counted FAILs are 0, 3 otherwise (a recorded result), 1 on
an error.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import historical_p2a as H
from eval.historical import load

LIST = Path(__file__).with_name("historical_fails_p2p.json")
MIGRATION = Path(__file__).with_name("migration_required_fails_p2p.json")


def excluded() -> list:
    from eval import compare_p2p

    return compare_p2p.EXCLUDED


def classify(run_dir: Path) -> dict:
    return H.classify(run_dir, load(LIST), load(MIGRATION), excluded())


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run-dir", type=Path, required=True)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--summary", action="store_true", help="print as the chain's FAIL summary")
    a = ap.parse_args(argv)
    res = classify(a.run_dir)
    text = json.dumps(res, indent=1, ensure_ascii=False) + "\n"
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    tag = "[verify summary]" if a.summary else "historical_p2p:"
    for r in res["fails"]:
        print(f"{tag} {H.label(r)}")
    print(
        f"{tag} {res['historical']} historical and {res['migration_required']} "
        f"migration-required FAIL(s) not counted; counted FAILs {res['counted_fails']} "
        f"(eval.historical_p2p against {res['reference_run']})"
    )
    return 0 if res["counted_fails"] == 0 else 3


if __name__ == "__main__":
    sys.exit(main())
