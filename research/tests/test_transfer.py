"""D-1/R1 — the transfer tool (research/transfer; DESIGN §1) on a synthetic lab.

A tiny lab repository is built in a temporary directory with its own lists,
an overlay and SOURCES.json; the tool must give the same tree id on two runs
and stop on every deliberate corruption: a changed overlay source, an M rule
that no longer matches, an unclassified file, and each leak class.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

from transfer import build as B
from transfer import classify as C
from transfer import rewrite as R

ALLOW = "core/\nLICENSE\nCHANGELOG.md\n"
STAYS = "research/\nreports/\nproduct/\n"
TRANSFORMS = """# t

## Dönüşümler

| desen | dönüşüm | ürün yolu |
|---|---|---|
| `README.md` | Y — overlay | `=` |
| `core/Cargo.toml` | M — repo | `=` |
| `research/data/ref.json` | K — data | `testdata/ref.json` |

## İstisnalar

| dosya | dizgi | gerekçe |
|---|---|---|
| `core/src/lib.rs` | `ada` | fiction |

## E-posta istisnaları

| dosya | adres | gerekçe |
|---|---|---|
| `testdata/ref.json` | `support@dendron.so` | third-party data |
"""


def sh(repo: Path, *a: str) -> str:
    return subprocess.run(
        ["git", "-C", str(repo), *a], check=True, capture_output=True, env=B.GIT_ENV
    ).stdout.decode()


def blob_of(repo: Path, path: str) -> str:
    return sh(repo, "rev-parse", f"HEAD:{path}").strip()


def write(repo: Path, files: dict[str, str | bytes]) -> None:
    for p, d in files.items():
        f = repo / p
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(d if isinstance(d, bytes) else d.encode())


def commit(repo: Path, msg: str = "c") -> str:
    sh(repo, "add", "-A")
    sh(
        repo,
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@example.com",
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        msg,
    )
    return sh(repo, "rev-parse", "HEAD").strip()


@pytest.fixture
def lab(tmp_path, monkeypatch):
    repo = tmp_path / "lab"
    repo.mkdir()
    sh(repo, "init", "-q", "-b", "main")
    write(
        repo,
        {
            f"{C.LISTS}/allowlist.txt": ALLOW,
            f"{C.LISTS}/stays.txt": STAYS,
            f"{C.LISTS}/transforms.md": TRANSFORMS,
            "core/Cargo.toml": 'repository = "https://github.com/arknsensei/rootmark"\n',
            "core/src/lib.rs": "// see /Users/ada/notes for the example\n",
            "LICENSE": "Apache\n",
            "CHANGELOG.md": "# Changes\n\nSee [README](README.md).\n",
            "README.md": "# lab readme (Turkish)\n",
            "research/data/ref.json": '{"t": "write to support@dendron.so"}\n',
            "research/notes.md": "lab only\n",
            "reports/x.json": "{}\n",
        },
    )
    commit(repo)
    write(
        repo,
        {
            "product/overlay/README.md": "# RootMark\n\nSee [CHANGELOG](CHANGELOG.md).\n",
            "product/overlay/RELEASING.md": "# Releasing\n",
            "product/overlay/SOURCES.json": json.dumps(
                {
                    "README.md": {"source": "README.md", "blob": blob_of(repo, "README.md")},
                    "RELEASING.md": {"source": None},
                }
            ),
        },
    )
    commit(repo)
    monkeypatch.setattr(
        R,
        "RULES",
        {"core/Cargo.toml": [(b"arknsensei/rootmark", b"rootmark-ai/rootmark", 1)]},
    )
    return repo


def run(lab: Path, out: Path, ref: str = "HEAD") -> dict:
    return B.build(lab, ref, out, None)


def test_clean_transfer_twice_same_tree(lab, tmp_path):
    a = run(lab, tmp_path / "A")
    b = run(lab, tmp_path / "B")
    assert a["ok"], (a["classification_errors"], a["leaks"])
    assert a["tree"] == b["tree"]
    tree = tmp_path / "A" / "tree"
    assert sorted(p.relative_to(tree).as_posix() for p in tree.rglob("*") if p.is_file()) == [
        "CHANGELOG.md",
        "LICENSE",
        "README.md",
        "RELEASING.md",
        "TRANSFER.json",
        "core/Cargo.toml",
        "core/src/lib.rs",
        "testdata/ref.json",
    ]
    assert b"rootmark-ai/rootmark" in (tree / "core/Cargo.toml").read_bytes()
    assert (tree / "README.md").read_text() == "# RootMark\n\nSee [CHANGELOG](CHANGELOG.md).\n"
    man = json.loads((tree / "TRANSFER.json").read_text())
    kinds = {r["path"]: (r["transform"], r["origin"].split(":")[0]) for r in man["files"]}
    assert kinds["testdata/ref.json"] == ("K", "lab")
    assert kinds["README.md"] == ("Y", "overlay")
    assert kinds["RELEASING.md"] == ("Y", "overlay")
    assert kinds["core/Cargo.toml"] == ("M", "lab")
    # the tree id is git's: a commit of the bare repository has that tree
    assert sh(tmp_path / "A" / "tree.git", "cat-file", "-t", a["tree"]).strip() == "tree"


def test_executable_mode_is_kept(lab, tmp_path):
    (lab / "core/run.sh").write_text("#!/bin/sh\n")
    (lab / "core/run.sh").chmod(0o755)
    commit(lab)
    a = run(lab, tmp_path / "A")
    ls = sh(tmp_path / "A" / "tree.git", "ls-tree", "-r", a["tree"])
    assert "100755 blob" in next(ln for ln in ls.splitlines() if ln.endswith("core/run.sh"))


def test_changed_overlay_source_stops(lab, tmp_path):
    write(lab, {"README.md": "# lab readme, edited\n"})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert not r["ok"]
    assert any("source README.md changed" in e for e in r["classification_errors"])


def test_overlay_without_sources_entry_stops(lab, tmp_path):
    write(lab, {"product/overlay/EXTRA.md": "x\n"})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert any("EXTRA.md: no SOURCES.json entry" in e for e in r["classification_errors"])


def test_m_rule_that_no_longer_matches_stops(lab, tmp_path):
    write(lab, {"core/Cargo.toml": 'repository = "https://example.org/x"\n'})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert any("matches 0 times, expected 1" in e for e in r["classification_errors"])


def test_unclassified_file_stops(lab, tmp_path):
    write(lab, {"newdir/file.txt": "x\n"})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert r["counts"]["unclassified"] == 1 and not r["ok"]


@pytest.mark.parametrize(
    ("files", "cls"),
    [
        ({"core/reports_link.md": "[x](../reports/x.json)\n"}, "broken_link"),
        ({"core/.rootmark/events.jsonl": "{}\n"}, "rootmark_dir"),
        ({"core/server.token": "t\n"}, "server_token"),
        ({"core/id_ed25519": "k\n"}, "key_pattern"),
        ({"core/a.txt": "token ghp_" + "a" * 36 + "\n"}, "secret_content"),
        ({"core/b.txt": "see /home/someone/notes\n"}, "home_path_unjustified"),
        ({"core/c.txt": "mail me: person@corp.io\n"}, "email_unjustified"),
        ({"core/ROADMAP.md": "# Yol haritası\n"}, "turkish_process"),
    ],
)
def test_each_leak_class_stops(lab, tmp_path, files, cls):
    write(lab, files)
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert r["leaks"]["counts"][cls] >= 1, r["leaks"]["counts"]
    assert not r["ok"]


def test_a_lab_path_in_the_allowlist_is_a_leak(lab, tmp_path):
    write(lab, {f"{C.LISTS}/allowlist.txt": ALLOW + "reports/x.json\n"})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert r["leaks"]["counts"]["lab_prefix"] == 1 and not r["ok"]


def test_host_home_and_private_terms(lab, tmp_path, monkeypatch):
    home = str(Path.home())
    write(lab, {"core/d.txt": f"cache at {home}/x\n"})
    commit(lab)
    r = run(lab, tmp_path / "A")
    assert r["leaks"]["counts"]["host_user"] >= 1
    monkeypatch.setenv("ROOTMARK_LEAK_TERMS", "Apache")
    r2 = run(lab, tmp_path / "B")
    assert r2["leaks"]["counts"]["private_term"] >= 1
    assert "Apache" not in json.dumps(r2)  # the value is never recorded


def test_tag_must_be_the_commit(lab, tmp_path):
    sh(lab, "tag", "v9.9.9", "HEAD~1")
    with pytest.raises(B.TransferError):
        B.build(lab, "HEAD", tmp_path / "A", "v9.9.9")


def _product_checkout(lab: Path, tmp_path: Path) -> tuple[Path, str]:
    r = run(lab, tmp_path / "P")
    prod = tmp_path / "product"
    import shutil

    shutil.copytree(tmp_path / "P" / "tree", prod)
    sh(prod, "init", "-q", "-b", "main")
    commit(prod, "rootmark 0.0.1")
    return prod, r["lab_commit"]


def test_verify_accepts_the_transferred_product(lab, tmp_path):
    from transfer import cli

    prod, lab_commit = _product_checkout(lab, tmp_path)
    args = [
        "verify",
        "--product",
        str(prod),
        "--lab-commit",
        lab_commit,
        "--out",
        str(tmp_path / "V"),
    ]
    assert cli.main([*args, "--repo", str(lab)]) == 0


def test_verify_rejects_a_changed_m_file_and_a_wrong_expected_tree(lab, tmp_path, capsys):
    from transfer import cli

    prod, lab_commit = _product_checkout(lab, tmp_path)
    base = ["verify", "--product", str(prod), "--lab-commit", lab_commit, "--repo", str(lab)]
    assert cli.main([*base, "--out", str(tmp_path / "V0"), "--expect-tree", "0" * 40]) == 1
    # an M output edited by hand in the product, TRANSFER.json left alone
    (prod / "core/Cargo.toml").write_text('repository = "https://example.org/evil"\n')
    commit(prod, "hand edit")
    assert cli.main([*base, "--out", str(tmp_path / "V1")]) == 1
    out = capsys.readouterr().out
    assert "core/Cargo.toml: bytes differ from the rebuilt tree" in out
