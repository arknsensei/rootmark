"""run_h3 helpers."""

from eval import run_h3 as R


def test_query_sentence_takes_first_long_sentence():
    assert (
        R.query_sentence("Short one. This sentence has more than five words.")
        == "This sentence has more than five words."
    )
    assert R.query_sentence("tiny text") == "tiny text"
