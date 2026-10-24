"""L-8 (GOAL P2-L; TASKS L5): three vaults with an index of this format and
the old lexical variant read by this binary after their recipe names the
new one — P2-0's three `v0.1.0` vaults (`data/p2_0_old_vaults/{e2e, mixed,
scene}`, committed), each copied, brought to this rootmark (`rootmark
migrate`: an `mdkernel-index/3` index with `mdk_lower_alnum`), then:

- `[index] tokenizer = "uni_fold_tr"` added to its recipe;
- `rootmark status`-equivalent (`migrate --dry-run`'s matrix):
  `artifacts.index.state = "migration_required"`, `required` names
  `index_rebuild`;
- `rootmark migrate`: `index_rebuild` applied, the index manifest's
  `lexical.tokenizer` is `uni_fold_tr`, the matrix `required` empty;
- `events_hash`, `replay` and `blame` of every document equal before and
  after; the newest record rolled back byte for byte.

Writes `p2l/migrate.json`; exit 0 when the three pass, 3 otherwise (the
record is still written).

    python -m eval.run_p2l_migrate --run <RUN> --rootmark <binary>
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

from eval import runinfo
from eval.run_p2a import DATA, answers, ok_json, records, run, tree

VAULTS = ["e2e", "mixed", "scene"]
SRC = DATA / "p2_0_old_vaults"


def with_tokenizer(v: Path, tok: str) -> None:
    p = v / ".rootmark/policy.toml"
    text = p.read_text(encoding="utf-8") if p.exists() else ""
    if "\n[index]\n" in "\n" + text:
        text = ("\n" + text).replace("\n[index]\n", f'\n[index]\ntokenizer = "{tok}"\n', 1)[1:]
    else:
        text += f'\n[index]\ntokenizer = "{tok}"\n'
    if not p.exists():
        raise RuntimeError(f"{v}: no policy.toml after migrate")
    p.write_text(text, encoding="utf-8")


def one(rm: str, name: str, work: Path) -> dict:
    src = SRC / name
    want = json.loads((src / "expected.json").read_text())
    v = work / name
    shutil.copytree(src / "vault", v)
    shutil.copytree(src / "store", v / ".rootmark")
    docs = list(want["docs"])
    first = ok_json([rm, "migrate", str(v)])
    if not (v / ".rootmark/policy.toml").exists():
        # the recipe `serve` would write (the vault had none)
        r = run([rm, "serve", "--vault", str(v), "--mcp", "stdio"], input="")
        if r.returncode != 0:
            raise RuntimeError(f"{name}: serve could not write the recipe: {r.stderr[-500:]}")
    b = run([rm, "index", "build", str(v)])
    if b.returncode != 0:
        raise RuntimeError(f"{name}: index build: {b.stderr[-500:]}")
    man0 = json.loads((v / ".rootmark/index/manifest.json").read_text())
    before = answers(rm, v, docs)
    t0 = tree(v)
    with_tokenizer(v, "uni_fold_tr")
    dry = ok_json([rm, "migrate", "--dry-run", str(v)])
    mig = ok_json([rm, "migrate", str(v)])
    man1 = json.loads((v / ".rootmark/index/manifest.json").read_text())
    after = answers(rm, v, docs)
    applied = [a["migration"] for a in mig["applied"]]
    newest = records(v)[-1] if records(v) else None
    back = run([rm, "migrate", "--rollback", newest, str(v)]) if newest else None
    back_ok = (
        back is not None
        and back.returncode == 0
        and json.loads(back.stdout).get("byte_equal") is True
    )
    # the rollback restores the old index; the recipe stays as written (and
    # `migrate` leaves no record of the first, recipe-less run's validations
    # apart: they are in both trees)
    t1 = {k: h for k, h in tree(v).items() if k != ".rootmark/policy.toml"}
    t0p = {k: h for k, h in t0.items() if k != ".rootmark/policy.toml"}
    ok = {
        "old_index_this_format": man0["format"] == "mdkernel-index/3"
        and man0["lexical"]["tokenizer"] == "mdk_lower_alnum",
        "migration_required": dry["matrix"]["artifacts"]["index"]["state"] == "migration_required"
        and "index_rebuild" in dry["matrix"]["required"],
        "index_rebuild_applied": "index_rebuild" in applied,
        "new_variant_in_manifest": man1["lexical"]["tokenizer"] == "uni_fold_tr",
        "nothing_required_after": mig["matrix"]["required"] == [],
        "events_hash_equal": before["events_hash"] == after["events_hash"] == want["events_hash"],
        "replay_equal": before["replay"] == after["replay"],
        "blame_equal": before["blame"] == after["blame"],
        "rollback_byte_equal": back_ok and t1 == t0p,
    }
    return {
        "vault": name,
        "first_migrate": [a["migration"] for a in first["applied"]],
        "docs": len(docs),
        "events_hash": before["events_hash"],
        "applied": applied,
        "checks": ok,
        "pass": all(ok.values()),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    a = ap.parse_args(argv)
    rm = str(Path(a.rootmark).resolve())
    with tempfile.TemporaryDirectory(prefix="rootmark-p2l8-") as tmp:
        rows = [one(rm, n, Path(tmp)) for n in VAULTS]
    out = {
        "criterion": "L-8",
        "vaults": rows,
        "passed": sum(r["pass"] for r in rows),
        "total": len(rows),
        "pass": all(r["pass"] for r in rows),
    }
    (runinfo.run_dir(a.run) / "p2l").mkdir(exist_ok=True)
    runinfo.write_json(a.run, "p2l/migrate.json", out)
    print(f"L-8: {out['passed']}/{out['total']} vaults → {'pass' if out['pass'] else 'FAIL'}")
    return 0 if out["pass"] else 3


if __name__ == "__main__":
    sys.exit(main())
