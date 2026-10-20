"""P2-A/A0 d: the information measurement after the (A) fix (GOAL P2-A
A-7, K-P2-02.5 (2)) — D-B test-3 (P2-0.2's pre-registered set and session
harness, unchanged) on one chain, real Obsidian, with the current binary.

Information only: no threshold, no decision (the value is P2-0.3's prior;
`unverified` becoming the default again needs that hypothesis round).
The harness's own `pass` (P2-0.2's threshold) is not carried over; the
value is recorded as `value_0_2b` with the rename sessions' share and the
per-format report. A hung save is retried at most once at set level
(`eval.real_retry`, A0 e).

  python -m eval.info_p2a --rootmark ../target/release/rootmark \
      --out ../reports/rounds/P2-A/info_d_b_test3.json
"""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path

from eval import real_retry
from eval import run_p2_0 as p0
from eval import run_p2_0_2 as r2
from eval.run_p2_0_1 import window

REPO = Path(__file__).resolve().parents[2]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--obsidian", default=p0.OBSIDIAN)
    ap.add_argument("--plugin", type=Path, default=REPO / "clients/obsidian")
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    if not Path(a.obsidian).is_file():
        raise FileNotFoundError(f"no Obsidian binary at {a.obsidian}: the measurement is not made")
    w = window()
    version = subprocess.run(
        [rootmark, "--version"], capture_output=True, text=True, check=True
    ).stdout.strip()
    commit = subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True, check=True
    ).stdout.strip()
    dirty = bool(
        subprocess.run(
            ["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=no"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    )
    with tempfile.TemporaryDirectory(prefix="rootmark-p2a-info-") as t:
        r = real_retry.with_retry(
            lambda wk: r2.run_d_b(rootmark, a.obsidian, a.plugin, w, wk), Path(t)
        )
    harness_pass = r.pop("pass", None)
    out = {
        "kind": "information (not a decision criterion; GOAL P2-A A-7, K-P2-02.5 (2))",
        "set": r.get("set"),
        "seed": r.get("seed"),
        "commit": commit,
        "worktree_dirty": dirty,
        "rootmark_version": version,
        "window_s": w,
        "value_0_2b": r.get("false_unverified_rate"),
        "new_blocks": r.get("new_blocks"),
        "unverified_after_window": r.get("unverified_after_window"),
        "latency_p95_ms": r.get("latency_p95_ms"),
        "sessions": r.get("sessions"),
        "renames": r.get("renames"),
        "retries": r.get("retries"),
        "reference": {
            "p2_0_2_official": "116/2743 = 0.0423 (RUN 20261019T212951Z-72fd78f, FINAL note 2)",
            "p2_0_2_threshold_not_applied": "≤ 0.01 (H-P2-0.2b); recorded only",
            "harness_pass_under_p2_0_2_threshold": harness_pass,
        },
        "harness": r,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(
        f"0.2b (information) = {out['value_0_2b']}  ({out['unverified_after_window']}/{out['new_blocks']})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
