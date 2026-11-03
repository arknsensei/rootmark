"""P2-Q guide runner (GOAL Q-4; TASKS Q1 (b), Q2 (a): `progress/GUIDE_RUN.md`).

Runs the `sh` blocks of docs/GUIDE.md as they are written, in order, in one
bash session (variables, functions and `cd` carry over, as in a reader's
terminal), with HOME pointing at a sandbox home so that every `~` path of
the guide lands in the round's work directory. Each block's commands run
under an ERR trap: a failing command is recorded with its line, the block
goes on (a reader would see the error and go on too) and the block is a
deviation.

Only what a terminal cannot do is arranged around the blocks (a plan per
mode, below): the foreground `rootmark serve` of §3 runs in the background;
the server is stopped where the guide says to stop it (§4, §6.1, §8) and
started again after; §1.4 (uninstall) runs last; Docker blocks belong to the
Docker run; Obsidian's own clicks (§5.1 steps 1–3) belong to the Obsidian
harness. Nothing in a block's text is changed.

    python -m p2q.guide_run --guide docs/GUIDE.md --work <dir> --mode release|candidate
        [--binary <rootmark>] [--models <dir>] [--observe <observe.py>]

`candidate` (Q1, before the release exists): the install blocks that
download the release are skipped and the candidate binary and models are
put where install.sh would put them. `release` (Q2): every block runs.
Writes <work>/guide_run.json (per block: id, heading, first command, exit
status, failing commands) and <work>/guide_run.log (the full transcript,
which stays outside the repository).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

FENCE = re.compile(r"^```(\w*)\s*$")
HEAD = re.compile(r"^(#{2,3}) (\d+(?:\.\d+)?)\.? ")

# blocks a terminal on this machine does not run, by mode
SKIP = {
    "1.3#1": "Docker: run by the Linux (Docker) stage, Q3",
    "1.3#2": "inside the container: run by the Linux (Docker) stage, Q3",
}
SKIP_CANDIDATE = {
    "1.1#1": "downloads the release (not published yet): the candidate binary is put in place",
    "1.2#1": "downloads from the release tag (not published yet)",
    "5.1#1": "downloads the release's plugin zip (not published yet)",
}
DEFER = {"1.4#1"}  # uninstall: last
STOP_BEFORE = {"4.1#1", "6.1#1", "8#1"}  # the guide stops the server here
START_AFTER = {"4.1#1", "6.1#1"}  # and the reader starts it again


def blocks(guide: str) -> list[dict]:
    out, sec, n, cur = [], "0", {}, None
    for line in guide.splitlines():
        m = HEAD.match(line)
        if m and cur is None:
            sec = m.group(2)
            continue
        f = FENCE.match(line)
        if f and cur is None:
            cur = {"lang": f.group(1), "lines": []}
        elif line.startswith("```") and cur is not None:
            if cur["lang"] == "sh":
                n[sec] = n.get(sec, 0) + 1
                out.append(
                    {"id": f"{sec}#{n[sec]}", "section": sec, "text": "\n".join(cur["lines"])}
                )
            cur = None
        elif cur is not None:
            cur["lines"].append(line)
    return out


START = """
__start_server() {
  : > "$__W/serve.err"
  ( cd "$__CWD_SERVE" && exec rootmark serve --vault "$VAULT" --http 127.0.0.1 --watch \
      2>>"$__W/serve.err" </dev/null ) &
  __SERVE_PID=$!
  for __i in $(seq 1 120); do grep -q 'http listening' "$__W/serve.err" && break; sleep 0.5; done
  echo "@@HOOK server started"
}
__stop_server() {
  if [ -n "${__SERVE_PID:-}" ]; then kill -TERM "$__SERVE_PID"; wait "$__SERVE_PID"; echo "@@HOOK server stopped rc=$?"; __SERVE_PID=; fi
}
"""


def script(bl: list[dict], mode: str, work: Path) -> str:
    skip = dict(SKIP)
    if mode == "candidate":
        skip.update(SKIP_CANDIDATE)
    parts = ["set -o pipefail", START, '__CWD_SERVE="$HOME"']
    order = [b for b in bl if b["id"] not in DEFER] + [b for b in bl if b["id"] in DEFER]
    for b in order:
        bid = b["id"]
        if bid in skip:
            parts.append(f"echo '@@BLOCK {bid} SKIP'")
            continue
        if bid in STOP_BEFORE:
            parts.append("__stop_server")
        text = b["text"]
        serve = text.lstrip().startswith("rootmark serve ")
        parts.append(f"echo '@@BLOCK {bid} START'")
        parts.append("__fails=0; trap '__fails=$((__fails+1)); echo \"@@ERR $LINENO\"' ERR")
        if serve:
            parts.append("__start_server")
        else:
            parts.append(text)
        parts.append('trap - ERR; echo "@@BLOCK ' + bid + ' END fails=$__fails"')
        if bid in START_AFTER:
            parts.append("__start_server")
    parts.append("__stop_server")
    return "\n".join(parts) + "\n"


def parse(log: str, bl: list[dict]) -> list[dict]:
    res = {
        b["id"]: {
            "id": b["id"],
            "section": b["section"],
            "first_command": b["text"].strip().splitlines()[0] if b["text"].strip() else "",
            "commands": sum(
                1 for ln in b["text"].splitlines() if ln.strip() and not ln.startswith((" ", "\t"))
            ),
            "status": "not run",
        }
        for b in bl
    }
    cur = None
    for line in log.splitlines():
        m = re.match(r"@@BLOCK (\S+) (START|SKIP|END)(?: fails=(\d+))?", line)
        if m:
            bid, what = m.group(1), m.group(2)
            if what == "SKIP":
                res[bid]["status"] = "skipped"
            elif what == "START":
                cur = bid
                res[bid]["err_lines"] = []
            else:
                fails = int(m.group(3))
                res[bid]["status"] = "ok" if fails == 0 else "failed"
                res[bid]["fails"] = fails
                cur = None
        elif cur and line.startswith("@@ERR"):
            res[cur]["err_lines"].append(line.split()[1])
    return [res[b["id"]] for b in bl]


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--guide", type=Path, required=True)
    ap.add_argument("--work", type=Path, required=True)
    ap.add_argument("--mode", choices=["release", "candidate"], required=True)
    ap.add_argument("--binary", type=Path)
    ap.add_argument("--models", type=Path)
    ap.add_argument("--observe", type=Path)
    ap.add_argument("--vault-seed", type=Path, help="a folder of Markdown copied to ~/notes")
    a = ap.parse_args(argv)
    work = a.work.resolve()
    home = work / "home"
    if home.exists():
        shutil.rmtree(home)
    home.mkdir(parents=True)
    if a.vault_seed:
        shutil.copytree(a.vault_seed, home / "notes")
    if a.mode == "candidate":
        (home / ".local/bin").mkdir(parents=True)
        shutil.copy2(a.binary, home / ".local/bin/rootmark")
        share = home / ".local/share/rootmark"
        share.mkdir(parents=True)
        os.symlink(a.models.resolve(), share / "models")
        (home / "rootmark-install").mkdir()
    bl = blocks(a.guide.read_text(encoding="utf-8"))
    sh = script(bl, a.mode, work)
    if a.mode == "candidate" and a.observe:
        # §7 downloads observe.py from the release tag: the candidate's copy
        sh = sh.replace(
            "curl -fsSLO https://raw.githubusercontent.com/arknsensei/rootmark/v0.3.0/research/p2q/observe.py",
            f"cp {a.observe.resolve()} .",
        )
    (work / "guide_run.sh").write_text(sh, encoding="utf-8")
    env = dict(os.environ)
    real_home = env.get("HOME", "")
    env.update({"HOME": str(home), "__W": str(work), "GH_CONFIG_DIR": f"{real_home}/.config/gh"})
    env.pop("ROOTMARK_MODELS", None)
    r = subprocess.run(
        ["bash", str(work / "guide_run.sh")],
        cwd=home,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    log = r.stdout + "\n@@STDERR\n" + r.stderr
    (work / "guide_run.log").write_text(log, encoding="utf-8")
    res = parse(r.stdout, bl)
    rec = {
        "mode": a.mode,
        "blocks": res,
        "ok": sum(x["status"] == "ok" for x in res),
        "failed": [x["id"] for x in res if x["status"] == "failed"],
        "skipped": [x["id"] for x in res if x["status"] == "skipped"],
        "not_run": [x["id"] for x in res if x["status"] == "not run"],
    }
    (work / "guide_run.json").write_text(json.dumps(rec, indent=1, ensure_ascii=False) + "\n")
    print(json.dumps({k: rec[k] for k in ("mode", "ok", "failed", "skipped", "not_run")}))
    return 0 if not rec["failed"] and not rec["not_run"] else 1


if __name__ == "__main__":
    sys.exit(main())
