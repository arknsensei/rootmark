"""H1-R definitions: touched lines (adjacency included), retention, loss categories."""

import pytest

from eval import metrics as M


def test_touched_lines_include_adjacent():
    # -5,2 covers old lines 5-6; 4 and 7 border the hunk
    assert M.touched_lines([{"old": [5, 2], "new": [5, 3]}]) == {4, 5, 6, 7}
    # pure insertion after old line 10 touches 10 and 11
    assert M.touched_lines([{"old": [10, 0], "new": [11, 2]}]) == {10, 11}
    assert M.is_touched(1, 3, {4}) is False
    assert M.is_touched(1, 4, {4}) is True


def blk(lin, vid, a, b, typ="Paragraph", text=None):
    return {
        "lineage_id": lin,
        "version_id": vid,
        "block_type": typ,
        "normalized_text": text or vid,
        "line_start": a,
        "line_end": b,
    }


def test_pair_record_and_categories():
    old = [
        blk("L1", "v1", 1, 1),  # untouched, kept
        blk("L2", "v2", 3, 3),  # untouched, lost, next to a touched block → hunk_adjacent
        blk("L3", "v3", 5, 6),  # touched (hunk 6), lost
        blk("L4", "dup", 9, 9),  # untouched, lost, duplicated content → dup_content
        blk("L5", "dup", 11, 11),  # untouched, kept
        blk(
            "L6", "v6", 13, 13, text="x"
        ),  # untouched, lost, same text now a ListItem → type_change
        blk("L7", "v7", 20, 20),  # untouched, lost, far away → other
    ]
    hunks = [{"old": [6, 1], "new": [6, 1]}]
    new = [{"version_id": "v6b", "block_type": "ListItem", "normalized_text": "x"}]
    r = M.h1r_pair_record(old, hunks, ["L1", "L5"], new)
    assert (r["untouched"], r["untouched_kept"], r["touched"], r["touched_kept"]) == (6, 2, 1, 0)
    cats = [loss["category"] for loss in r["losses"]]
    assert cats == ["hunk_adjacent", "dup_content", "type_change", "other"]
    agg = M.h1r_aggregate([r], [{"commit": "c", "path": "p.md"}])
    assert agg["H1-R1"] == pytest.approx(2 / 6)
    assert agg["H1-R2"] == pytest.approx(2 / 7)
    assert agg["touched_retention"] == 0.0
    assert agg["loss_categories"] == {
        "dup_content": 1,
        "hunk_adjacent": 1,
        "type_change": 1,
        "other": 1,
    }
    assert agg["other_examples"] == [{"commit": "c", "doc": "p.md", "version_id": "v7"}]


def test_empty_denominator_is_error():
    r = M.h1r_pair_record([], [], [], [])
    with pytest.raises(M.MetricError):
        M.h1r_aggregate([r], [{"commit": "c", "path": "p"}])
