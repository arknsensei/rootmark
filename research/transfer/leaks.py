"""D-1 — the leak check over the product tree (DESIGN §1.5).

Runs on the generated product files (paths and bytes), not on lab sources:
the overlays and the M outputs are checked as they go. Records hold counts,
product paths and the justified exceptions only; the host user's name, home
directory and the private terms are read at run time and never written.
"""

from __future__ import annotations

import fnmatch
import getpass
import os
import re
from pathlib import Path, PurePosixPath

LAB_PREFIXES = ("research/", "reports/", "docs/rounds/", "docs/archive/", "product/")
KEY_GLOBS = (
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "id_rsa*",
    "id_ed25519*",
    ".env",
    ".env.*",
    "*secret*",
)
SECRET_RE = re.compile(
    rb"gh[pousr]_[A-Za-z0-9]{36,}"
    rb"|github_pat_[A-Za-z0-9_]{50,}"
    rb"|dckr_pat_[A-Za-z0-9_-]{20,}"
    rb"|-----BEGIN [A-Z ]*PRIVATE KEY-----"
    rb"|AKIA[0-9A-Z]{16}"
    rb"|sk-ant-[A-Za-z0-9_-]{20,}"
    rb"|xox[baprs]-[A-Za-z0-9-]{10,}"
)
HOME_RE = re.compile(rb"/(?:Users|home)/([A-Za-z0-9._-]+)/")
MAIL_RE = re.compile(
    rb"[A-Za-z0-9._%+-]+@[A-Za-z0-9][A-Za-z0-9-]*(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}"
)
# role and documentation addresses that may appear anywhere
MAIL_OK = re.compile(
    rb"^(?:[A-Za-z0-9._%+-]+@rootmark\.ai|[A-Za-z0-9._%+-]+@example\.(?:com|org|net)|git@github\.com)$"
)
# the lab's Turkish process documents (never a product path, by name)
TURKISH_PROCESS = {
    "NORTH_STAR.md",
    "DECISIONS.md",
    "DEV_PROCESS.md",
    "ROADMAP.md",
    "HANDOFF.md",
    "ARCHITECTURE.md",
    "PACKAGES.md",
    "RELEASE.md",
    "SPEC_EDITS_CRITERION.md",
    "HEADLESS_CHANNEL.md",
}
LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
TERMS_FILE = Path.home() / ".config/rootmark-lab/leak_terms.txt"


def _private_terms() -> list[bytes]:
    """Terms that must never reach the product (e.g. a personal or admin
    address): ~/.config/rootmark-lab/leak_terms.txt or $ROOTMARK_LEAK_TERMS
    (one per line); none in a clean environment. Values never recorded."""
    lines = []
    if TERMS_FILE.is_file():
        lines += TERMS_FILE.read_text(encoding="utf-8").splitlines()
    lines += os.environ.get("ROOTMARK_LEAK_TERMS", "").splitlines()
    return [ln.strip().encode() for ln in lines if ln.strip() and not ln.startswith("#")]


def check(files: dict[str, bytes], home_exc: list[dict], mail_exc: list[dict]) -> dict:
    """files: product path -> bytes. Returns counts and lists (product paths,
    justified exceptions); `ok` only if every unjustified count is 0."""
    out = {
        k: []
        for k in (
            "lab_prefix",
            "rootmark_dir",
            "server_token",
            "key_pattern",
            "turkish_process",
            "secret_content",
            "host_user",
            "home_path_unjustified",
            "email_unjustified",
            "private_term",
            "broken_link",
        )
    }
    justified = {"home_path": [], "email": []}
    me = getpass.getuser().encode()
    home = os.path.expanduser("~").encode()
    me_re = re.compile(rb"(?<![A-Za-z0-9])" + re.escape(me) + rb"(?![A-Za-z0-9])")
    terms = _private_terms()
    home_ok = {(e["file"], e["string"]) for e in home_exc}
    mail_ok = {(e["file"], e["address"]) for e in mail_exc}
    paths = set(files)
    for p in sorted(files):
        pp = PurePosixPath(p)
        if p.startswith(LAB_PREFIXES):
            out["lab_prefix"].append(p)
        if ".rootmark" in pp.parts:
            out["rootmark_dir"].append(p)
        if pp.name == "server.token":
            out["server_token"].append(p)
        if any(fnmatch.fnmatchcase(pp.name, g) for g in KEY_GLOBS):
            out["key_pattern"].append(p)
        if pp.name in TURKISH_PROCESS:
            out["turkish_process"].append(p)
        b = files[p]
        if SECRET_RE.search(b):
            out["secret_content"].append(p)
        # the home directory is always a leak; the user name as a word too,
        # except the CI runner's generic `runner` (a common English word in
        # the tree, not a person) — the runner's home (/home/runner,
        # /Users/runner) is still found, by `home` and by HOME_RE
        if home in b or (me != b"runner" and me_re.search(b)):
            out["host_user"].append(p)
        for m in sorted(set(HOME_RE.findall(b))):
            s = m.decode("utf-8", "replace")
            (justified["home_path"] if (p, s) in home_ok else out["home_path_unjustified"]).append(
                {"file": p, "user": s} if (p, s) in home_ok else p
            )
        for m in sorted(set(MAIL_RE.findall(b))):
            if MAIL_OK.match(m):
                continue
            s = m.decode("utf-8", "replace")
            if (p, s) in mail_ok:
                justified["email"].append({"file": p, "address": s})
            else:
                out["email_unjustified"].append(p)
        if any(t in b for t in terms):
            out["private_term"].append(p)
        # the site's pages link to pages site/build.sh copies in at build
        # time; the built site's every internal link is checked there
        if (
            p.endswith(".md")
            and "/fixtures" not in p
            and "fixtures_variants/" not in p
            and not p.startswith("site/src/")
        ):
            text = b.decode("utf-8", "replace")
            for t in LINK_RE.findall(text):
                if re.match(r"^[a-z]+:", t) or t.startswith("#"):
                    continue
                target = os.path.normpath(os.path.join(os.path.dirname(p), t.split("#", 1)[0]))
                if target in (".", ""):
                    continue
                if target not in paths and not any(
                    x.startswith(target.rstrip("/") + "/") for x in paths
                ):
                    out["broken_link"].append(f"{p} -> {t}")
    res = {k: sorted(set(v)) for k, v in out.items()}
    counts = {k: len(v) for k, v in res.items()}
    return {
        "counts": counts,
        "files": {k: v for k, v in res.items() if v},
        "justified": justified,
        "private_terms_searched": len(terms),
        "ok": not any(counts.values()),
    }
