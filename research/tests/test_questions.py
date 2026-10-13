"""D4: leakage and file-name filters; the committed set resolves completely."""

import pytest

from eval.datasets import questions as Q


def test_leak_detects_eight_word_runs_only():
    p = "To publish your notes you run the publish command from the command palette in VS Code."
    assert Q.leaks("Can I run the publish command from the command palette in VS Code?", p)
    assert not Q.leaks("How do I publish notes from VS Code?", p)  # 7-word overlap at most
    assert Q.leaks("x " + " ".join(Q.words(p)[:8]), p)
    assert not Q.leaks("x " + " ".join(Q.words(p)[:7]), p)


def test_file_name_filter():
    assert Q.name_stem("notes/my-cool_note.md") == "my cool note"
    assert Q.mentions_name("What does My Cool  Note say?", ["notes/my-cool_note.md"])
    assert not Q.mentions_name("What does the note say?", ["notes/my-cool_note.md"])


def test_prompts_are_fixed_english_strings():
    assert Q.PROMPT_SINGLE.startswith(
        "Based only on the passage below, write one natural user question"
    )
    assert Q.PROMPT_MULTI.startswith(
        "Two passages are given. Write one question that requires BOTH passages"
    )
    assert Q.MODEL == "claude-sonnet-4-6" and Q.TEMPERATURE == 0


@pytest.mark.skipif(
    not (Q.D3 / ".mdkernel" / "events.jsonl").exists() or not (Q.D4 / "single.jsonl").exists(),
    reason="needs data/d3 (make d3) and data/d4",
)
def test_committed_question_set_resolves_completely():
    Q.check()  # raises on any unresolvable truth, leak, file name or split problem
