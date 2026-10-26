"""P2-R determinism record (GOAL P2-R R-6; `make p2r-checks`): a prune's
outputs and files written whole into the RUN, so that the pair comparison
(`eval.compare_p2r`) sees two chains' bytes.

A vault built through the CLI only: 12 human notes, 4 rounds of an edit
(`rootmark ingest`), an agent's proposal (`rootmark propose`), two reads
(`rootmark read`, one trace each) and a checkpoint; the second checkpoint
named. Recipe: `[retention] keep_checkpoints = 2`, `[[prune.rules]]`
`proposal` `idle_checkpoints = 2`. Then `rootmark prune --dry-run` (and the
sha256 of every file under `.rootmark/` before and after it: equal), then
`rootmark prune` twice (the second finds nothing new), `--now-ms` fixed.
Recorded: the three outputs whole, the sha256 of every live and archive file
(`traces.jsonl`, `traces/`, `checkpoints/`, `loc/`, `events.jsonl`,
`trace_filters.jsonl`), the archived traces (`rootmark trace list
--archived`) and `events_hash`.

    python -m eval.run_p2r_det --run <RUN> --rootmark <binary>

Writes `p2r/det.json` (no time, no path). Exit 0, 1 on an error.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

from eval import runinfo
from eval.datasets.common import dump_json

NOW_MS = 1_800_000_000_000
RECIPE = (
    '[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n'
    "[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 24\n\n"
    '[identity]\nhuman_id = "det"\n\n[attestation]\nunattested = "canonical"\n\n'
    "[retention]\nkeep_checkpoints = 2\n\n"
    '[prune]\nversion = "det"\n\n'
    '[[prune.rules]]\nid = "stale"\nlayer = "proposal"\nidle_checkpoints = 2\n'
)


def run(cmd: list[str]) -> dict:
    r = subprocess.run(cmd, capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(f"run_p2r_det: {' '.join(cmd[1:3])} failed: {r.stderr.decode()}")
    return json.loads(r.stdout)


def hashes(v: Path) -> dict:
    m = v / ".rootmark"
    out = {}
    for p in sorted(m.rglob("*")):
        rel = p.relative_to(m).as_posix()
        if p.is_file() and not rel.startswith("index/"):
            out[rel] = hashlib.sha256(p.read_bytes()).hexdigest()
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    a = ap.parse_args(argv)
    rm = str(Path(a.rootmark).resolve())
    out_dir = runinfo.run_dir(a.run, create=False) / "p2r"
    out_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rootmark-p2r-det-") as tmp:
        v = Path(tmp) / "vault"
        v.mkdir()
        for i in range(12):
            (v / f"n{i:02d}.md").write_text(
                f"# Note {i}\n\nA human note number {i} about espresso and tea.\n", encoding="utf-8"
            )
        run([rm, "ingest-dir", str(v), str(v)])
        (v / ".rootmark" / "policy.toml").write_text(RECIPE, encoding="utf-8")
        for r in range(4):
            (v / "n00.md").write_text(
                f"# Note 0\n\nRound {r} of the first note.\n", encoding="utf-8"
            )
            run([rm, "ingest", str(v), str(v / "n00.md")])
            f = Path(tmp) / "p.md"
            f.write_text(f"# Proposal {r}\n\nAn agent proposal of round {r}.\n", encoding="utf-8")
            run([rm, "propose", str(v), f"agents/p{r}.md", str(f), "--source", "det-agent"])
            cp = run([rm, "checkpoint", "create", str(v)])["checkpoint_id"]
            for doc in ("n00.md", f"agents/p{r}.md"):
                lid = run([rm, "blame", str(v), doc])["blocks"][-1]["lineage_id"]
                run([rm, "read", str(v), "--at", cp, "--consumer", "det", lid])
            if r == 1:
                run([rm, "checkpoint", "create", str(v), "--name", "kept"])
        before = hashes(v)
        dry = run([rm, "prune", str(v), "--dry-run", "--now-ms", str(NOW_MS)])
        dry_unchanged = hashes(v) == before
        first = run([rm, "prune", str(v), "--now-ms", str(NOW_MS)])
        second = run([rm, "prune", str(v), "--now-ms", str(NOW_MS)])
        res = {
            "dry_run": dry,
            "dry_run_changed_nothing": dry_unchanged,
            "prune": first,
            "prune_again": second,
            "files_before": before,
            "files_after": hashes(v),
            "archived_traces": run([rm, "trace", "list", str(v), "--archived"]),
            "checkpoints": run([rm, "checkpoint", "list", str(v)]),
            "events_hash": run([rm, "events-hash", str(v)])["events_hash"],
        }
    res["pass"] = (
        dry_unchanged
        and len(first["rules"]["pruned"]) > 0
        and first["retention"]["traces"]["lines"] > 0
        and second["rules"]["pruned"] == []
        and second["retention"]["traces"]["lines"] == 0
    )
    (out_dir / "det.json").write_text(dump_json(res), encoding="utf-8")
    print(
        f"p2r-det: pruned {len(first['rules']['pruned'])} lineages, "
        f"{first['retention']['traces']['lines']} trace lines, "
        f"{len(first['retention']['checkpoints'])} checkpoints archived; dry run changed nothing "
        f"{dry_unchanged}; pass {res['pass']}"
    )
    return 0 if res["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
