"""The agent of D-A2 (GOAL P2-0.1 T3; ASSUMPTIONS Y0-7, Y3-1): writes a
vault's files through the other file-system paths an agent with a shell
has — a Python process, `sed -i`, `git apply`, `cp`/`rsync` over the file,
`cat >>`, and an atomic rename — never through MCP or HTTP, never with an
attestation, never under `.rootmark/`. P2-0's `eval.agent_fs_writer` (one
`write` call, D-A and D-C) is left unchanged; this module adds the paths.

A step is `{"op": <path>, "path": <doc>, "text": <the document's full new
text>, "kind": <append_human|replace_section|overwrite|new_file>, "tool"?:
"cp"|"rsync"}`; how each path realises it (Y3-1):

- `python`: a separate `python3` process; `append_human` opens the file in
  append mode and writes the added suffix, the others write the whole text.
- `sed`: BSD/GNU `sed -i` (a temporary file renamed over the target):
  `append_human` appends the added lines (`$a`), the others replace every
  line with the new text (`1,$c`); `new_file` (sed edits existing files
  only) is `sed -e '' <prepared file> > <target>`.
- `git_apply`: a unified diff of the current and the new text, made outside
  the vault, applied in the vault with `git apply` (new files from
  `/dev/null`).
- `cp_rsync`: the new text prepared outside the vault, then `cp` or
  `rsync` over the target.
- `cat`: `cat >> <target>` with the added suffix for `append_human`;
  `cat > <target>` for the others (`new_file` included).
- `atomic_rename`: the new text written to a hidden temporary file next to
  the target, then `mv` over it.

The prepared files live in a work directory outside the vault. Fail-fast:
a tool that exits non-zero, or a file that does not end with the intended
text, is an error.
"""

from __future__ import annotations

import difflib
import shutil
import subprocess
import sys
from pathlib import Path

PATHS = ["python", "sed", "git_apply", "cp_rsync", "cat", "atomic_rename"]


def _target(vault: Path, rel: str) -> Path:
    parts = Path(rel).parts
    if Path(rel).is_absolute() or ".." in parts or any(p == ".rootmark" for p in parts):
        raise ValueError(f"agent step outside the vault's documents: {rel!r}")
    return vault / rel


def _run(cmd: list[str], **kw) -> None:
    r = subprocess.run(cmd, check=False, capture_output=True, **kw)
    if r.returncode != 0:
        raise RuntimeError(f"{cmd[0]}: exit {r.returncode}: {r.stderr[-400:]!r}")


def _sed_text(text: str) -> str:
    """`text` as the argument of sed's `a`/`c` command: each line but the
    last ends with a backslash; backslashes escaped."""
    lines = text.split("\n")
    return "\\\n".join(line.replace("\\", "\\\\") for line in lines)


def _sed_inplace(script: str, target: Path) -> None:
    # GNU sed knows --version, BSD sed needs an empty suffix after -i
    gnu = subprocess.run(["sed", "--version"], check=False, capture_output=True).returncode == 0
    cmd = ["sed", "-i"] if gnu else ["sed", "-i", ""]
    _run([*cmd, "-e", script, str(target)])


def apply_step(vault: Path, step: dict, work: Path) -> None:
    """Applies one D-A2 step to `vault` with the step's path."""
    op, kind, text = step["op"], step["kind"], step["text"]
    target = _target(vault, step["path"])
    target.parent.mkdir(parents=True, exist_ok=True)
    work.mkdir(parents=True, exist_ok=True)
    old = target.read_text(encoding="utf-8") if target.exists() else None
    if kind == "append_human":
        if old is None or not text.startswith(old.rstrip("\n")):
            raise ValueError(f"{step['path']}: an append must extend the current text")
        suffix = text[len(old.rstrip("\n")) :]
        if not old.endswith("\n"):
            raise ValueError(f"{step['path']}: append to a file without a final newline")
        suffix = suffix.removeprefix("\n")  # old ends with "\n"
    prep = work / "prepared.md"
    prep.write_bytes(text.encode("utf-8"))
    if op == "python":
        if kind == "append_human":
            code = "import sys; open(sys.argv[1], 'a', encoding='utf-8').write(sys.stdin.read())"
            _run([sys.executable, "-c", code, str(target)], input=suffix.encode())
        else:
            code = "import sys; open(sys.argv[1], 'w', encoding='utf-8').write(sys.stdin.read())"
            _run([sys.executable, "-c", code, str(target)], input=text.encode())
    elif op == "sed":
        if kind == "new_file":
            with open(target, "wb") as f:
                r = subprocess.run(["sed", "-e", "", str(prep)], check=False, stdout=f)
            if r.returncode != 0:
                raise RuntimeError(f"sed: exit {r.returncode}")
        elif kind == "append_human":
            _sed_inplace(f"$a\\\n{_sed_text(suffix.rstrip(chr(10)))}", target)
        else:
            _sed_inplace(f"1,$c\\\n{_sed_text(text.rstrip(chr(10)))}", target)
    elif op == "git_apply":
        a = [] if old is None else old.splitlines(keepends=True)
        b = text.splitlines(keepends=True)
        rel = step["path"]
        diff = "".join(
            difflib.unified_diff(
                a,
                b,
                fromfile="/dev/null" if old is None else f"a/{rel}",
                tofile=f"b/{rel}",
            )
        )
        patch = work / "change.patch"
        patch.write_text(diff, encoding="utf-8")
        _run(["git", "apply", "-p1", str(patch)], cwd=vault)
    elif op == "cp_rsync":
        tool = step["tool"]
        if tool == "cp":
            _run(["cp", str(prep), str(target)])
        else:
            _run(["rsync", str(prep), str(target)])
    elif op == "cat":
        if kind == "append_human":
            (work / "suffix.md").write_bytes(suffix.encode("utf-8"))
            _run(["sh", "-c", 'cat "$1" >> "$2"', "sh", str(work / "suffix.md"), str(target)])
        else:
            _run(["sh", "-c", 'cat "$1" > "$2"', "sh", str(prep), str(target)])
    elif op == "atomic_rename":
        tmp = target.parent / f".{target.name}.agtmp"
        tmp.write_bytes(text.encode("utf-8"))
        _run(["mv", str(tmp), str(target)])
    else:
        raise ValueError(f"unknown D-A2 path {op!r}")
    got = target.read_text(encoding="utf-8")
    if got.rstrip("\n") != text.rstrip("\n"):
        raise RuntimeError(f"{op}: {step['path']} does not hold the intended text")


def clean(work: Path) -> None:
    shutil.rmtree(work, ignore_errors=True)
