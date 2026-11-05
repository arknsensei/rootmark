"""P2-Q.1/R4 (K-P2Q1.3): P2-Q's Obsidian scene (`p2q.obsidian_scene`,
unchanged) run on the new pre-registration `research/p2q1/scenario.json`:
the scene's one read of `research/p2q/scenario.json` is directed to it;
nothing else differs. Arguments are the scene's own.

    P2Q_PATH_KEY_FILE=<key> python -m p2q1.obsidian_rerun --snapshot … --work … --out … --guide … --guide-home … --rootmark …
"""

from __future__ import annotations

from pathlib import Path

from p2q import obsidian_scene as scene

NEW = Path(__file__).resolve().parent / "scenario.json"
_load = scene.load


def load(path):
    if Path(path).resolve() == (scene.HERE / "scenario.json").resolve():
        return _load(NEW)
    return _load(path)


scene.load = load

if __name__ == "__main__":
    raise SystemExit(scene.main())
