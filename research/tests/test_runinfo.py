"""Run manifest: the clean-tree flag must see untracked, staged and modified files."""

import subprocess

import pytest

from eval import runinfo


@pytest.fixture
def repo(tmp_path, monkeypatch):
    def git(*a):
        subprocess.run(["git", *a], cwd=tmp_path, check=True, capture_output=True)

    git("init", "-q")
    git("config", "user.email", "t@t")
    git("config", "user.name", "t")
    (tmp_path / ".gitignore").write_text("data/\n")
    (tmp_path / "a.py").write_text("x = 1\n")
    git("add", "-A")
    git("commit", "-qm", "init")
    monkeypatch.setattr(runinfo, "ROOT", tmp_path)
    return tmp_path, git


def test_clean_tree(repo):
    assert runinfo.worktree_clean()


def test_untracked_file_is_dirty(repo):
    root, _ = repo
    (root / "new.py").write_text("y = 2\n")
    assert not runinfo.worktree_clean()


def test_staged_and_modified_are_dirty(repo):
    root, git = repo
    (root / "a.py").write_text("x = 2\n")
    assert not runinfo.worktree_clean()
    git("add", "a.py")
    assert not runinfo.worktree_clean()


def test_ignored_paths_do_not_count(repo):
    root, _ = repo
    (root / "data").mkdir()
    (root / "data" / "big.bin").write_text("z")
    assert runinfo.worktree_clean()


def test_run_id_format():
    assert runinfo.RUN_RE.match("20261013T141540Z-94ac155")
    with pytest.raises(ValueError):
        runinfo.run_dir("latest")
