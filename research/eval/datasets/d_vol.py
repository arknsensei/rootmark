"""D-VOL (P2-R/R3; GOAL P2-R 4): the synthetic documents of the volume
measurement. Deterministic from `SEED`: no file is committed, every byte
comes from this module.

- 200 human notes `notes/n<i>.md`: a heading and three paragraphs of words
  drawn from a fixed vocabulary;
- an edit of note `i` in round `r`: its paragraph `r % 3` replaced by a new
  sentence (the rest of the note as it was);
- proposal `j` of round `r`: `agents/r<r>/p<j>.md`, a heading and one
  paragraph;
- queries: two or three vocabulary words.

    from eval.datasets import d_vol
"""

from __future__ import annotations

import random

from eval.datasets.common import SEED

NOTES = 200
VOCAB = [
    f"{a}{b}"
    for a in ("ka", "lo", "mi", "ne", "pu", "ra", "si", "to", "vu", "ze")
    for b in ("bar", "cen", "dol", "fim", "gur", "hes", "jok", "lan", "mor", "nup")
]


class Gen:
    """The D-VOL generator: one random stream from `SEED`, consumed in call
    order (the measurer calls it in a fixed order)."""

    def __init__(self, seed: int = SEED) -> None:
        self.r = random.Random(seed * 7 + 2026)
        self.notes: dict[str, list[str]] = {}

    def sentence(self, n: int = 12) -> str:
        w = [self.r.choice(VOCAB) for _ in range(n)]
        return w[0].capitalize() + " " + " ".join(w[1:]) + "."

    def note_path(self, i: int) -> str:
        return f"notes/n{i:03d}.md"

    @staticmethod
    def render(title: str, paras: list[str]) -> str:
        return f"# {title}\n\n" + "\n\n".join(paras) + "\n"

    def initial(self) -> dict[str, str]:
        """The 200 notes, path → text."""
        out = {}
        for i in range(NOTES):
            p = self.note_path(i)
            self.notes[p] = [self.sentence() for _ in range(3)]
            out[p] = self.render(f"Note {i}", self.notes[p])
        return out

    def edit(self, i: int, rnd: int) -> tuple[str, str]:
        """Note `i`'s text after its edit in round `rnd`."""
        p = self.note_path(i)
        self.notes[p][rnd % 3] = self.sentence()
        return p, self.render(f"Note {i}", self.notes[p])

    def edited(self, rnd: int, n: int) -> list[int]:
        """The `n` notes edited in round `rnd` (distinct, sorted)."""
        return sorted(self.r.sample(range(NOTES), n))

    def proposal(self, rnd: int, j: int) -> tuple[str, str]:
        return f"agents/r{rnd:02d}/p{j}.md", self.render(f"Proposal {rnd}-{j}", [self.sentence(16)])

    def query(self) -> str:
        return " ".join(self.r.choice(VOCAB) for _ in range(self.r.choice((2, 3))))

    def pick(self, n: int) -> int:
        return self.r.randrange(n)
