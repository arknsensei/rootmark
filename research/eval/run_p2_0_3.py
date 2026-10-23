"""P2-0.3 measurements (GOAL P2-0.3 "Önceden kayıtlı eşikler"; TASKS R0, R1):
a thin wrapper of `eval.run_p2_0_2` — no set, threshold, W, generator,
harness or comparator of P2-0.2 changes (ASSUMPTIONS P2-0.3 R0-3).

    python -m eval.run_p2_0_3 test --run <RUN> --rootmark <bin>
        [--obsidian <bin>] [--plugin <dir>] [--only d_b,...]
    python -m eval.run_p2_0_3 checks --run <RUN> [--ref-run <dir>]

`test` writes `<RUN>/p2_0_3/`:

- `d_b.json` (H-P2-0.3b, b′, c, b-yük): **D-B test-4** — `run_p2_0_2.run_d_b`
  as it is, with `p2_0_2_sets.d_b_seed` replaced for that call only by
  `SEED` (BLAKE3("P2-0.3/D-B")[:8], ASSUMPTIONS R0-2): P2-0.2's test-3
  generator and link-form distribution (X0-5..X0-7, X0-15..X0-18) on the
  vault and sessions of the new seed. D-B test-3's sessions are never
  generated here (`_guard`: the generator refuses any other seed);
- `d_a_prime.json`, `d_c_prime.json` and their logs (H-P2-0.3e, d′):
  `run_p2_0_2.run_real` as it is (P2-0.2's sets, unchanged: D-A′ on the
  test-3 vault, X0-8; D-C′ on P2-0.1's ticks).

Each real-Obsidian set runs under `eval.real_retry` (P2-A/A0 e: a hung save
is retried once at set level, recorded in `retries`). The other sets (rules,
D-E, D-A, D-A2, D-C, D-B2) are those `make p2a-checks` already wrote into
`<RUN>/p2_0_2/` earlier in the same chain with P2-0.2's code; they are not
run twice (ASSUMPTIONS R0-4).

`checks` writes `p2_0_3_checks.json`: `run_p2_0_2_checks.checks` (P2-0.2's
— P2-0's — thresholds, applied by P2-0.2's code) on a view of the RUN in
which `p2_0_2/{d_b,d_a_prime,d_c_prime}.json` are this round's, with the
codes renamed H-P2-0.2* → H-P2-0.3*; H-P2-0.3b-yük is the same value
against ≤ 0.02 (the report takes it from both chains of the loaded pair);
H-P2-0.3i is the pair half (`pair_compare.json`, `eval.compare_p2a pair`),
read by the report. Beside them: the seed check, the byte equality of the
D-A, D-A2 and D-C logs with the P2-A official RUN (`--ref-run`; the product
did not change) and the P2-A recorders of the same chain (`p2a_checks.json`:
FAIL summary, regress, deliberate cases).

Fail-fast: a missing file, a seed other than `SEED`, a W other than the
P2-0.2 phase's — an exception, never a partial record.
"""

from __future__ import annotations

import argparse
import json
import tempfile
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

import blake3

from eval import real_retry, runinfo
from eval import run_p2_0 as p0
from eval import run_p2_0_2 as r2
from eval import run_p2_0_2_checks as c2
from eval.datasets import p2_0_2_sets as sets2
from eval.datasets.common import die
from eval.run_p2_0_1 import window

REPO = Path(__file__).resolve().parents[2]
SEED = int(blake3.blake3(b"P2-0.3/D-B").hexdigest()[:8], 16)
# the seeds never used again (GOAL P2-0.3): D-B test-3's, which P2-A's
# information measurement also used (reports/rounds/P2-A/info_d_b_test3.json)
OLD_SEEDS = {int(blake3.blake3(b"P2-0.2/D-B").hexdigest()[:8], 16)}
REF_RUN = REPO / "reports/rounds/P2-A/runs/20261020T230453Z-1a775d3"
LOGS = [f"{s}/{g}" for s in ("d_a", "d_a2", "d_c") for g in ("events.jsonl", "attestations.jsonl")]
SETS = ["d_b", "d_a_prime", "d_c_prime"]
REAL = {
    "d_b": "p2_0_3/d_b.json",
    "d_a_prime": "p2_0_3/d_a_prime.json",
    "d_c_prime": "p2_0_3/d_c_prime.json",
}


@contextmanager
def _seed(seed: int) -> Iterator[None]:
    """`p2_0_2_sets.d_b_seed` returns `seed` inside the block."""
    orig = sets2.d_b_seed
    sets2.d_b_seed = lambda: seed
    try:
        yield
    finally:
        sets2.d_b_seed = orig


@contextmanager
def _guard() -> Iterator[None]:
    """D-B's session generator refuses any seed but `SEED` inside the block."""
    orig = sets2.d_b3

    def d_b3() -> dict:
        if sets2.d_b_seed() != SEED:
            raise RuntimeError(
                f"GOAL P2-0.3: D-B sessions only with the seed {SEED}, not {sets2.d_b_seed()}"
            )
        return orig()

    sets2.d_b3 = d_b3
    try:
        yield
    finally:
        sets2.d_b3 = orig


def run_d_b4(rootmark: str, obsidian: str, plugin: Path, window_s: float, work: Path) -> dict:
    """D-B test-4: P2-0.2's `run_d_b` with the new seed."""
    with _seed(SEED):
        r = r2.run_d_b(rootmark, obsidian, plugin, window_s, work)
    if r.get("skipped"):
        return r
    if r["seed"] != SEED or r["coverage"]["seed"] != SEED:
        raise RuntimeError(f"D-B test-4 ran with the seed {r['seed']}, not {SEED}")
    r["set"] = "D-B test-4"
    r["generator"] = "eval.datasets.p2_0_2_sets (D-B test-3's generator, unchanged; seed replaced)"
    return r


def test(a: argparse.Namespace) -> int:
    window_s = window()
    out = runinfo.run_dir(a.run) / "p2_0_3"
    out.mkdir(parents=True, exist_ok=True)
    rootmark = str(Path(a.rootmark).resolve())
    only = set(filter(None, a.only.split(",")))
    if only - set(SETS):
        raise ValueError(f"unknown sets {sorted(only - set(SETS))}")
    todo = [s for s in SETS if not only or s in only]
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-3-") as w, _guard():
        work = Path(w)
        for s in todo:
            if not Path(a.obsidian).is_file():
                r = {"skipped": True, "reason": f"no Obsidian binary at {a.obsidian}", "pass": None}
            elif s == "d_b":
                r = real_retry.with_retry(
                    lambda wk: run_d_b4(rootmark, a.obsidian, a.plugin, window_s, wk), work / s
                )
            else:
                r = real_retry.with_retry(
                    lambda wk, s=s: r2.run_real(
                        s, rootmark, a.obsidian, a.plugin, window_s, out, wk
                    ),
                    work / s,
                )
            (out / f"{s}.json").write_text(json.dumps(r, indent=1, ensure_ascii=False) + "\n")
    runinfo.record_phase(
        a.run,
        "p2_0_3_test" + ("" if not only else "_" + "_".join(sorted(only))),
        {"window_s": window_s, "sets": todo, "obsidian": a.obsidian, "seed": SEED},
        [f"p2_0_3/{t}.json" for t in todo],
    )
    return 0


def _load(p: Path):
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p2-0-3-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def _view(d: Path, v: Path) -> None:
    """`v`: the RUN `d` as `run_p2_0_2_checks` reads it, with this round's
    real-Obsidian sets in place of the skipped ones."""
    (v / "p2_0_2").mkdir(parents=True)
    for name in ("manifest.json", "conformance_p1d1.json"):
        (v / name).symlink_to(d / name)
    for f in (d / "p2_0_2").iterdir():
        if f.name not in {f"{s}.json" for s in SETS}:
            (v / "p2_0_2" / f.name).symlink_to(f)
    for s, src in REAL.items():
        (v / "p2_0_2" / f"{s}.json").symlink_to(d / src)


def _w(d: Path) -> float:
    phases = _load(d / "manifest.json")["phases"]
    ws = {v["params"]["window_s"] for k, v in phases.items() if k.startswith("p2_0_3_test")}
    if len(ws) != 1:
        raise ValueError(f"the RUN's P2-0.3 phases name {len(ws)} windows")
    return ws.pop()


def _logs(d: Path, ref: Path) -> dict:
    """D-A, D-A2, D-C logs: byte equal to the P2-A official RUN's."""
    rows = {}
    for g in LOGS:
        a, b = d / "p2_0_2" / g, ref / "p2_0_2" / g
        if not a.is_file() or not b.is_file():
            raise FileNotFoundError(f"{a if not a.is_file() else b} missing")
        rows[g] = {
            "blake3": blake3.blake3(a.read_bytes()).hexdigest()[:32],
            "equal": a.read_bytes() == b.read_bytes(),
        }
    return {
        "reference_run": str(ref.relative_to(REPO)),
        "logs": rows,
        "pass": all(x["equal"] for x in rows.values()),
    }


def checks(d: Path, ref: Path) -> dict:
    db = _load(d / "p2_0_3/d_b.json")
    for s in SETS:
        _load(d / REAL[s])
    if not db.get("skipped") and (db["seed"] != SEED or db["seed"] in OLD_SEEDS):
        raise ValueError(f"D-B test-4's seed is {db['seed']}, the pre-registered one {SEED}")
    w0 = c2._w(d)
    if _w(d) != w0:
        raise ValueError(f"W differs: P2-0.3 {_w(d)}, P2-0.2 phase {w0}")
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-3-view-") as t:
        v = Path(t)
        _view(d, v)
        base = c2.checks(v)
    out: dict = {"window": {**base.pop("window"), "p2_0_3_window_s": _w(d)}}
    for k, x in base.items():
        k3 = k.replace("H-P2-0.2", "H-P2-0.3")
        if isinstance(x, dict) and "source" in x:
            x = {**x, "source": _src(x["source"])}
        out[k3] = x
    # P2-0.2's k (the written rules) is not a P2-0.3 hypothesis: recorded only
    out["rules"] = {**out.pop("H-P2-0.3k"), "pass_is_criterion": False}
    b = out["H-P2-0.3b"]
    if b.get("skipped"):
        out["H-P2-0.3b-yük"] = {**c2.NOT_MEASURED, "source": "p2_0_3/d_b.json"}
    else:
        b["set"] = "D-B test-4"
        rate = b["false_unverified_rate"]
        out["H-P2-0.3b-yük"] = {
            "false_unverified_rate": rate,
            "threshold": 0.02,
            "source": "p2_0_3/d_b.json",
            # the criterion is both chains of the loaded pair (the report)
            "pass": rate is not None and rate <= 0.02 and b["sessions"] == 200,
        }
    out["seed"] = {
        "seed": SEED,
        "derivation": 'int(blake3("P2-0.3/D-B")[:8], 16)',
        "recorded": None if db.get("skipped") else db["seed"],
        "not_reused": sorted(OLD_SEEDS),
        "pass": db.get("skipped") is True or db["seed"] == SEED,
    }
    out["logs_vs_p2a"] = _logs(d, ref)
    p2a = _load(d / "p2a_checks.json")
    out["p2a_recorders"] = {
        k: p2a[k] for k in ("fail_summary", "regress", "tamper") if k in p2a
    } | {"source": "p2a_checks.json (make p2a-checks, same chain)"}
    return out


def _src(s: str) -> str:
    for name, path in REAL.items():
        s = s.replace(f"p2_0_2/{name}.json", path)
    return s


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    t = sub.add_parser("test")
    t.add_argument("--run", required=True)
    t.add_argument("--rootmark", required=True)
    t.add_argument("--obsidian", default=p0.OBSIDIAN)
    t.add_argument("--plugin", type=Path, default=REPO / "clients/obsidian")
    t.add_argument("--only", default="", help="comma list of sets for a partial (development) run")
    c = sub.add_parser("checks")
    c.add_argument("--run", required=True)
    c.add_argument("--ref-run", type=Path, default=REF_RUN)
    a = ap.parse_args(argv)
    if a.cmd == "test":
        return test(a)
    d = runinfo.run_dir(a.run, create=False)
    out = checks(d, a.ref_run.resolve())
    runinfo.write_json(a.run, "p2_0_3_checks.json", out)
    runinfo.record_phase(a.run, "p2_0_3_checks", {"seed": SEED}, ["p2_0_3_checks.json"])
    print(
        "; ".join(
            f"{k} {'-' if not isinstance(v, dict) or v.get('pass') is None else 'pass' if v['pass'] else 'FAIL'}"
            for k, v in out.items()
            if k != "window"
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
