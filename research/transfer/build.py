"""D-1 — the product tree of a lab commit (DESIGN §1).

Reads only the commit's objects (`git ls-tree`, `git cat-file`), never the
work tree: the lists, the overlays and SOURCES.json come from the same
commit as the files. Writes:
  <out>/tree/                the product files
  <out>/tree.git             a bare repository holding the tree's objects
  <out>/transfer_result.json tree id, counts, leak check (no host value)
and exits non-zero on any unclassified or doubly classified file, product
path collision, dead pattern, missing M rule or failing rule, overlay whose
source changed, overlay without a SOURCES entry (or the reverse), symlink,
or unjustified leak.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

from transfer import classify as C
from transfer import leaks as L
from transfer import rewrite as R

OVERLAY = "product/overlay/"
SOURCES = OVERLAY + "SOURCES.json"
LAB_REPO = "arknsensei/rootmark"
GIT_ENV = {
    **{k: v for k, v in os.environ.items() if not k.startswith("GIT_")},
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "LC_ALL": "C",
}


class TransferError(Exception):
    pass


def git(repo: Path, *args, data: bytes | None = None, env=None) -> bytes:
    p = subprocess.run(
        ["git", "-C", str(repo), *args],
        input=data,
        capture_output=True,
        env=env or GIT_ENV,
        check=False,
    )
    if p.returncode != 0:
        raise TransferError(f"git {' '.join(args[:3])}: {p.stderr.decode(errors='replace')[-400:]}")
    return p.stdout


class Commit:
    """The objects of one lab commit."""

    def __init__(self, repo: Path, ref: str):
        self.repo = repo
        self.sha = git(repo, "rev-parse", "--verify", f"{ref}^{{commit}}").decode().strip()
        self.entries = {}  # path -> (mode, blob)
        raw = git(repo, "ls-tree", "-r", "-z", "--full-tree", self.sha)
        for rec in raw.split(b"\0"):
            if not rec:
                continue
            meta, path = rec.split(b"\t", 1)
            mode, typ, obj = meta.decode().split()
            p = path.decode("utf-8")
            if typ != "blob":
                raise TransferError(f"{p}: {typ} entries are not transferred (submodule?)")
            self.entries[p] = (mode, obj)
        self._cache: dict[str, bytes] = {}

    def read(self, path: str) -> bytes:
        if path not in self.entries:
            raise TransferError(f"{path}: not in lab commit {self.sha[:12]}")
        obj = self.entries[path][1]
        if obj not in self._cache:
            self._cache[obj] = git(self.repo, "cat-file", "blob", obj)
        return self._cache[obj]

    def committer_date(self) -> str:
        return git(self.repo, "show", "-s", "--format=%cI", self.sha).decode().strip()


def product_files(commit: Commit, lab_tag: str | None, cfg: Commit | None = None):
    """-> (files: path -> dict(mode, data, origin, transform), report dict).

    cfg: the commit the lists, overlays and SOURCES.json are read from (the
    same commit by default; another one only for the standalone check of an
    older tree, R1 — never in transfer.yml or transfer-verify)."""
    cfg = cfg or commit
    lists = C.LISTS
    allow = C.parse_list(cfg.read(f"{lists}/allowlist.txt").decode("utf-8"), "allowlist.txt")
    stays = C.parse_list(cfg.read(f"{lists}/stays.txt").decode("utf-8"), "stays.txt")
    trans, home_exc, mail_exc = C.parse_transforms(
        cfg.read(f"{lists}/transforms.md").decode("utf-8")
    )
    cl = C.classify(sorted(commit.entries), allow, stays, trans)
    errors = []
    # with another commit's lists (standalone check of an older tree) a
    # pattern for a file that came later is dead by design: reported only
    checked = ("unclassified", "double", "collisions") + (
        () if cfg is not commit else ("dead_patterns",)
    )
    for k in checked:
        if cl[k]:
            errors.append(f"{k}: {len(cl[k])} (first: {cl[k][:5]})")
    # overlays and their sources
    overlays = {
        p[len(OVERLAY) :]: p for p in cfg.entries if p.startswith(OVERLAY) and p != SOURCES
    }
    try:
        sources = json.loads(cfg.read(SOURCES))
    except (TransferError, ValueError) as e:
        raise TransferError(f"{SOURCES}: {e}") from e
    for dest in sorted(set(overlays) ^ set(sources)):
        errors.append(
            f"overlay {dest}: "
            + (
                "no SOURCES.json entry"
                if dest in overlays
                else "SOURCES.json entry without an overlay file"
            )
        )
    files = {}
    y_dests = {}
    skipped_sources = []
    for f, info in sorted(cl["files"].items()):
        if info["class"] == "stays":
            continue
        mode, blob = commit.entries[f]
        if mode not in ("100644", "100755"):
            errors.append(f"{f}: mode {mode} (only regular files go)")
            continue
        dest, kind = info["dest"], info["kind"]
        origin = f"lab:{f}@{blob}"
        if kind in ("=", "K"):
            data = commit.read(f)
        elif kind == "M":
            try:
                data = R.rewrite(f, commit.read(f))
            except R.RewriteError as e:
                errors.append(str(e))
                continue
        else:  # Y
            y_dests[dest] = f
            continue  # filled from the overlay below
        files[dest] = {"mode": mode, "data": data, "origin": origin, "transform": kind}
    for dest, op in sorted(overlays.items()):
        src = sources.get(dest, {}).get("source") if isinstance(sources.get(dest), dict) else None
        lab_src = y_dests.get(dest)
        if lab_src is not None:
            if src != lab_src:
                errors.append(
                    f"overlay {dest}: SOURCES.json source {src!r}, transforms.md says {lab_src!r}"
                )
                continue
            want = sources[dest].get("blob")
            have = commit.entries[lab_src][1]
            # an older tree checked with today's overlays (cfg is another
            # commit; standalone check only): its sources are older by design
            if want != have and cfg is not commit:
                skipped_sources.append(dest)
            elif want != have:
                errors.append(
                    f"overlay {dest}: source {lab_src} changed ({str(want)[:12]} -> {have[:12]}); "
                    "update the overlay and its SOURCES.json blob"
                )
                continue
            kind = "Y"
        else:
            if src is not None:
                errors.append(
                    f"overlay {dest}: SOURCES.json names source {src!r} but no Y row of transforms.md does"
                )
                continue
            if dest in files:
                errors.append(f"overlay {dest}: collides with a lab file's product path")
                continue
            kind = "Y"
        mode = cfg.entries[op][0]
        if mode not in ("100644", "100755"):
            errors.append(f"{op}: mode {mode}")
            continue
        files[dest] = {
            "mode": mode,
            "data": cfg.read(op),
            "origin": f"overlay:{op}@{cfg.entries[op][1]}",
            "transform": kind,
        }
    for dest, f in sorted(y_dests.items()):
        if dest not in overlays:
            errors.append(f"{f}: transform Y but no overlay product/overlay/{dest}")
    if "TRANSFER.json" in files:
        errors.append("TRANSFER.json: a product file of that name would be overwritten")
    counts = {
        "lab_files": len(commit.entries),
        "goes": sum(1 for v in cl["files"].values() if v["class"] == "allow"),
        "stays": sum(1 for v in cl["files"].values() if v["class"] == "stays"),
        "transforms": {
            k: sum(1 for v in cl["files"].values() if v.get("kind") == k) for k in ("K", "M", "Y")
        },
        "overlays_with_source": sum(1 for d in overlays if d in y_dests),
        "overlays_product_only": sum(1 for d in overlays if d not in y_dests),
        "unclassified": len(cl["unclassified"]),
        "double_classified": len(cl["double"]),
        "product_path_collisions": len(cl["collisions"]),
        "dead_patterns": len(cl["dead_patterns"]),
        "overlay_sources_not_checked": skipped_sources,
    }
    manifest = {
        "format": 1,
        "lab_repo": LAB_REPO,
        "lab_commit": commit.sha,
        "lab_tag": lab_tag,
        "files": [
            {
                "path": p,
                "mode": files[p]["mode"],
                "sha256": hashlib.sha256(files[p]["data"]).hexdigest(),
                "origin": files[p]["origin"],
                "transform": files[p]["transform"],
            }
            for p in sorted(files)
        ],
    }
    files["TRANSFER.json"] = {
        "mode": "100644",
        "data": (json.dumps(manifest, indent=1, ensure_ascii=False) + "\n").encode("utf-8"),
        "origin": None,
        "transform": None,
    }
    lk = L.check({p: v["data"] for p, v in files.items()}, home_exc, mail_exc)
    report = {
        "counts": counts,
        "classification_errors": errors,
        "unclassified_files": cl["unclassified"][:50],
        "double_files": cl["double"][:50],
        "leaks": lk,
    }
    return files, report


def write_tree(files: dict, out: Path) -> str:
    """Materialise the files and compute the git tree id (modes as given,
    bytes as they are: no filter, no line-ending conversion)."""
    tree = out / "tree"
    bare = out / "tree.git"
    for d in (tree, bare):
        if d.exists():
            shutil.rmtree(d)
    tree.mkdir(parents=True)
    git(out, "init", "-q", "--bare", str(bare))
    paths = sorted(files)
    for p in paths:
        t = tree / p
        t.parent.mkdir(parents=True, exist_ok=True)
        t.write_bytes(files[p]["data"])
        os.chmod(t, 0o755 if files[p]["mode"] == "100755" else 0o644)
    objs = (
        git(
            bare,
            "hash-object",
            "-w",
            "--no-filters",
            "--stdin-paths",
            data=("\n".join(str(tree / p) for p in paths) + "\n").encode("utf-8"),
        )
        .decode()
        .split()
    )
    if len(objs) != len(paths):
        raise TransferError("hash-object: object count differs from file count")
    env = {**GIT_ENV, "GIT_INDEX_FILE": str(out / "transfer.index"), "GIT_DIR": str(bare)}
    if (out / "transfer.index").exists():
        (out / "transfer.index").unlink()
    info = "".join(f"{files[p]['mode']} {o}\t{p}\n" for p, o in zip(paths, objs, strict=True))
    subprocess.run(
        ["git", "update-index", "--add", "--index-info"],
        input=info.encode("utf-8"),
        env=env,
        check=True,
    )
    t = (
        subprocess.run(["git", "write-tree"], env=env, capture_output=True, check=True)
        .stdout.decode()
        .strip()
    )
    (out / "transfer.index").unlink()
    return t


def build(
    repo: Path, ref: str, out: Path, lab_tag: str | None, config_ref: str | None = None
) -> dict:
    out.mkdir(parents=True, exist_ok=True)
    commit = Commit(repo, ref)
    if lab_tag is not None:
        tagc = git(repo, "rev-parse", "--verify", f"{lab_tag}^{{commit}}").decode().strip()
        if tagc != commit.sha:
            raise TransferError(f"tag {lab_tag} is {tagc[:12]}, not {commit.sha[:12]}")
    cfg = Commit(repo, config_ref) if config_ref else None
    files, report = product_files(commit, lab_tag, cfg)
    tree = write_tree(files, out)
    result = {
        "transfer": "D-1 research/transfer (docs/rounds/D-1/progress/DESIGN.md §1)",
        "lab_commit": commit.sha,
        "lab_tag": lab_tag,
        "lab_commit_date": commit.committer_date(),
        "config_commit": cfg.sha if cfg else commit.sha,
        "tree": tree,
        "product_files": len(files),
        **report,
    }
    result["ok"] = not report["classification_errors"] and report["leaks"]["ok"]
    (out / "transfer_result.json").write_text(
        json.dumps(result, indent=1, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return result
