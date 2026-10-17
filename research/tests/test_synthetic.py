"""D1 generator: per-op consistency, op ratios, truth format, truth resolution."""

import json
import random
import re
from collections import Counter

import pytest
import rootmark_core as mk

from eval import truth as T
from eval.datasets import synthetic_edits as se

WORDS = [
    "coffee",
    "roast",
    "batch",
    "grinder",
    "espresso",
    "water",
    "cup",
    "filter",
    "bean",
    "origin",
    "aroma",
    "acidity",
    "body",
    "sweet",
    "bitter",
    "crack",
    "drum",
    "heat",
    "airflow",
    "profile",
    "washed",
    "natural",
    "blend",
    "rest",
    "dose",
    "yield",
]


def source_doc(seed: int) -> str:
    rng = random.Random(seed)

    def sentence():
        n = rng.randint(6, 14)
        return " ".join(rng.choice(WORDS) for _ in range(n)).capitalize() + "."

    parts = ["# Title"]
    for s in range(3):
        parts.append(f"## Section {s}")
        for _ in range(rng.randint(3, 4)):
            parts.append(" ".join(sentence() for _ in range(rng.randint(1, 4))))
        parts.append("- " + sentence())
    return "\n\n".join(parts) + "\n"


def pool():
    return se.parse_blocks(
        "\n\n".join(
            " ".join(random.Random(i).choice(WORDS) for _ in range(12)) + f" pool{i}."
            for i in range(40)
        )
    )[0]


def blocks_of(text):
    return se.parse_blocks(text)[0]


def at(blocks, ref):
    hits = [i for i, b in enumerate(blocks) if b.version_id == ref["version_id"]]
    return hits[ref["k"]]


@pytest.fixture(scope="module")
def generated(tmp_path_factory):
    out = tmp_path_factory.mktemp("d1")
    p = pool()
    truths = {}
    for i in range(100):
        name = f"doc{i:03d}.md"
        truths[name] = (se.generate_doc(name, source_doc(i), p, out / name), out / name)
    return truths


def test_ratios_within_3pct_over_1000_steps(generated):
    ops = Counter(s["op"] for t, _ in generated.values() for s in t["steps"] if s["scored"])
    total = sum(ops.values())
    assert total == 1000
    for name, p in se.OPS:
        assert abs(ops[name] / total - p) <= 0.03, (name, ops[name] / total, p)
    preps = sum(1 for t, _ in generated.values() for s in t["steps"] if s["op"] == "dup_prepare")
    assert preps == ops["dup_delete"]


def test_every_op_is_consistent(generated):
    seen = set()
    for t, d in generated.values():
        versions = [blocks_of((d / "v00.md").read_text())]
        for s in t["steps"]:
            versions.append(blocks_of((d / s["file"]).read_text()))
        for i, s in enumerate(t["steps"]):
            old, new = versions[i], versions[i + 1]
            op, m = s["op"], s["mapping"]
            seen.add(op)
            olds = [at(old, o) for o, _ in m if o is not None]
            news = [[at(new, n) for n in ns] for _, ns in m]
            if op in ("rephrase_light", "rephrase_heavy"):
                lo, hi = (0.05, 0.15) if op == "rephrase_light" else (0.15, 0.30)
                a, b = old[olds[0]].text.split(), new[news[0][0]].text.split()
                assert len(a) == len(b)
                changed = sum(x != y for x, y in zip(a, b))
                assert max(1, round(lo * len(a))) <= changed <= max(1, round(hi * len(a)))
                assert len(new) == len(old)
            elif op == "split":
                a, b = news[0]
                assert b == a + 1 and len(new) == len(old) + 1
                assert mk.normalize(new[a].text + " " + new[b].text) == mk.normalize(
                    old[olds[0]].text
                )
            elif op == "merge":
                assert news[0] == news[1] and olds[1] == olds[0] + 1 and len(new) == len(old) - 1
                assert mk.normalize(old[olds[0]].text + " " + old[olds[1]].text) == mk.normalize(
                    new[news[0][0]].text
                )
            elif op == "move":
                assert (
                    old[olds[0]].version_id == new[news[0][0]].version_id and olds[0] != news[0][0]
                )
                assert sorted(b.version_id for b in old) == sorted(b.version_id for b in new)
            elif op == "insert":
                assert m[0][0] is None and len(new) == len(old) + 1
                assert new[news[0][0]].version_id not in {b.version_id for b in old}
            elif op == "delete":
                assert m[0][1] == [] and len(new) == len(old) - 1
            elif op == "dup_prepare":
                vids = {new[n[0]].version_id for n in news}
                assert len(vids) == 1 and len(news) == 3
                pos = sorted(n[0] for n in news)
                assert pos[1] - pos[0] >= 2 and pos[2] - pos[1] >= 2
            elif op == "dup_delete":
                assert len(new) == len(old) - 1
                assert m[2][1] == [] and len(m) == 3
                # the first copy is deleted; the remaining copies shift k by one
                assert [o["k"] for o, _ in m] == [1, 2, 0]
                assert [ns[0]["k"] for _, ns in m[:2]] == [0, 1]
                assert olds[2] < olds[0] < olds[1]
                vid = old[olds[2]].version_id
                assert sum(b.version_id == vid for b in old) == 3
                assert sum(b.version_id == vid for b in new) == 2
            else:
                raise AssertionError(op)
    assert seen == {n for n, _ in se.OPS} | {"dup_prepare"}


def test_truth_has_only_content_refs(generated):
    ulid = re.compile(r'"[0-9A-HJKMNP-TV-Z]{26}"')
    for t, d in generated.values():
        raw = (d / "truth.json").read_text()
        assert not ulid.search(raw)
        for s in t["steps"]:
            for o, ns in s["mapping"]:
                for r in ([o] if o else []) + ns:
                    assert set(r) == {"version_id", "k"}
                    T.check_ref(r)


def test_truth_resolves_through_vault(generated, tmp_path):
    t, d = generated["doc007.md"]
    v = mk.Vault(str(tmp_path))
    v.ingest("doc.md", (d / "v00.md").read_text())
    for s in t["steps"]:
        for o, _ in s["mapping"]:
            if o is not None:
                T.resolve(v, o, "doc.md")
        v.ingest("doc.md", (d / s["file"]).read_text())
        for _, ns in s["mapping"]:
            for n in ns:
                T.resolve(v, n, "doc.md")
    with pytest.raises(T.TruthError):
        T.resolve(v, {"version_id": "0" * 32, "k": 0}, "doc.md")
    with pytest.raises(T.TruthError):
        T.resolve(v, {"version_id": "x", "k": 0}, "doc.md")


def test_generation_is_deterministic(tmp_path):
    a = se.generate_doc("same.md", source_doc(3), pool(), tmp_path / "a")
    b = se.generate_doc("same.md", source_doc(3), pool(), tmp_path / "b")
    assert a == b
    for f in sorted((tmp_path / "a").iterdir()):
        assert f.read_bytes() == (tmp_path / "b" / f.name).read_bytes()
    assert json.loads((tmp_path / "a" / "truth.json").read_text()) == a


BORDERLINE = (
    "# Doc\n\nOne paragraph with several words here. And a second sentence follows it.\n\n"
    "Another paragraph that has enough words in it to qualify for edits.\n\n"
    "- a list item that is long enough to be rephrased by the generator\n\n"
    "## Part\n\n```\ncode\n```\n\nThird paragraph stands alone with a handful of words.\n\n"
    "| a | b |\n|---|---|\n| 1 | 2 |\n\n> quote\n"
)


def test_borderline_document_never_crashes(tmp_path):
    assert se.eligible(BORDERLINE)
    redrawn = 0
    for i in range(60):
        t = se.generate_doc(f"edge{i}.md", BORDERLINE, pool(), tmp_path / str(i))
        assert sum(s["scored"] for s in t["steps"]) == se.STEPS
        redrawn += len(t["redraws"])
    assert redrawn > 0, "the borderline document should exercise op redraws"


def test_truth_never_contains_lineage_ids(generated, tmp_path):
    for name in sorted(generated)[:20]:
        t, d = generated[name]
        v = mk.Vault(str(tmp_path / name))
        r0 = v.ingest(t["doc_path"], (d / "v00.md").read_text())
        seen = {l for e in r0["events"] for l in e["lineage_ids"]}
        for s in t["steps"]:
            r = v.ingest(t["doc_path"], (d / s["file"]).read_text())
            seen.update(l for e in r["events"] for l in e["lineage_ids"])
        raw = (d / "truth.json").read_text()
        assert seen and not any(l in raw for l in seen)
