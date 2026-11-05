"""P2-Q.1/R4 (K-P2Q1.3): a new pre-registration of P2-Q's Obsidian scene.

P2-Q's personal-path key is lost (K-P2Q1-T2), so the pre-registered
selections of `research/p2q/scenario.json` (`obsidian`: `k2:` refs) cannot
be resolved. This writes `research/p2q1/scenario.json`: P2-Q's scenario with
only the Obsidian corpus' snapshot summary and the `obsidian` section made
again — by P2-Q's own generator functions, unchanged (`p2q.make_scenario.
obsidian`, the same seed and step definitions O01–O14), over today's
snapshot's sorted path list and a new key (`P2Q_PATH_KEY_FILE`, 32 bytes,
outside the repository). Everything else is copied from P2-Q's file.
Written before the scene runs (registration).

    P2Q_PATH_KEY_FILE=<key> python -m p2q1.make_obsidian --obsidian <snapshot> [--check]
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from p2q import make_scenario as ms
from p2q.common import corpus_summary, k2, list_md, path_key

HERE = Path(__file__).resolve().parent
P2Q = HERE.parent / "p2q" / "scenario.json"


def build(obs: Path) -> dict:
    scen = json.loads(P2Q.read_text(encoding="utf-8"))
    key = path_key()
    paths = list_md(obs)
    linked = ms.wikilink_targets(obs, paths)
    ob, *_ = ms.obsidian(ms.SEED, paths, linked, lambda p: k2(p, key))
    scen["registered"] = (
        "P2-Q.1/R4 (K-P2Q1.3): P2-Q's Obsidian scene, new selection with a new key "
        "over today's snapshot, before the scene runs; other corpora as in P2-Q"
    )
    scen["corpora"]["obsidian"]["snapshot"] = corpus_summary(obs)
    scen["corpora"]["obsidian"]["linked_notes"] = len(linked)
    scen["obsidian"] = {k: v for k, v in ob.items() if k != "steps"}
    return scen


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--obsidian", required=True, type=Path)
    ap.add_argument("--out", type=Path, default=HERE / "scenario.json")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    text = json.dumps(build(a.obsidian), ensure_ascii=False, indent=1, sort_keys=True) + "\n"
    if a.check:
        same = a.out.read_text(encoding="utf-8") == text
        print("same" if same else "different")
        return 0 if same else 1
    a.out.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
