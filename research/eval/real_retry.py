"""A bounded set-level retry for the real-Obsidian sets (P2-A/A0 e; P2-0.2
verification recommendation 3, ASSUMPTIONS P2-A A0-6).

A real-Obsidian set (D-B, D-A′, D-C′) is one harness process; one hung save
(CDP evaluate timeout, Obsidian not writing a saved file within 2 s) used
to end the whole chain. `with_retry` runs the set again, from a fresh work
directory, at most `attempts - 1` more times, and only for those two
failures; any other failure is raised at once. Every retry is recorded in
the result (`retries`: attempt, the failure's last 500 characters), so a
retried set is visible in the RUN, and the measured result is the last
attempt's as a whole (no record of a failed attempt is kept).

Nothing here changes a set, a threshold or a measurement.
"""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path

# the harnesses' messages for a hung save (clients/obsidian/e2e/real/cdp.ts,
# clients/obsidian/e2e/p2_0_2/real.ts)
HUNG = ("CDP evaluate timed out", "was not written by Obsidian within 2 s of the save")
ATTEMPTS = 2


def hung(message: str) -> bool:
    """Whether a harness failure is a hung save."""
    return any(m in message for m in HUNG)


def with_retry(run: Callable[[Path], dict], work: Path, attempts: int = ATTEMPTS) -> dict:
    """`run(work / "try<n>")` until it returns, retrying a hung save at most
    `attempts - 1` times; the result carries `retries`."""
    if attempts < 1:
        raise ValueError("attempts must be at least 1")
    retries: list[dict] = []
    for n in range(1, attempts + 1):
        try:
            r = run(work / f"try{n}")
        except RuntimeError as e:
            msg = str(e)
            if not hung(msg) or n == attempts:
                raise
            retries.append({"attempt": n, "error": msg[-500:]})
            continue
        r["retries"] = retries
        return r
    raise AssertionError("unreachable")
