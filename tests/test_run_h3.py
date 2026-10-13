"""run_h3 helpers: probe texts are unique and never collide with each other."""

from eval import run_h3 as R


def test_probe_texts_unique_and_distinct():
    texts = [R.probe_text(i) for i in range(100)]
    assert len(set(texts)) == 100
    assert all("marmot" in t for t in texts)
