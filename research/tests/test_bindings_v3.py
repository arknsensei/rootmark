"""Python binding, matcher v3 surface (SPEC_DELTA §7): `matcher=` argument,
`MDK_MATCHER` default, per-vault version lock, mini_vault hashes equal to the CLI."""

import json
import subprocess
from pathlib import Path

import mdkernel_core as mk
import pytest

ROOT = Path(__file__).resolve().parents[1]
MINI = ROOT.parent / "spec" / "fixtures" / "mini_vault"
MDK = ROOT.parent / "target" / "release" / "mdk"
V2_MINI_HASH = "8914f9b418ae360d49c85c951050f432"

A = (
    "The roaster controls heat, airflow and drum speed for every batch of green coffee, "
    "and the operator writes each change into the roast log so that the next batch can be compared."
)
B = "Cupping follows within one day of roasting."


def doc(*ps):
    return "\n\n".join(ps) + "\n"


def cli_hash(tmp, matcher):
    out = subprocess.run(
        [str(MDK), "--matcher", matcher, "ingest-dir", str(tmp), str(MINI)],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(out.stdout)["events_hash"]


def test_default_is_v3_and_env_and_argument(tmp_path, monkeypatch):
    monkeypatch.delenv("MDK_MATCHER", raising=False)
    assert mk.Vault(str(tmp_path / "a")).matcher() == "v3"
    monkeypatch.setenv("MDK_MATCHER", "v2")
    assert mk.Vault(str(tmp_path / "b")).matcher() == "v2"
    assert mk.Vault(str(tmp_path / "c"), matcher="v3").matcher() == "v3"
    assert mk.matcher_defaults() == mk.matcher_defaults("v2")
    with pytest.raises(RuntimeError):
        mk.Vault(str(tmp_path / "d"), matcher="v4")


def test_matcher_defaults_per_version(tmp_path):
    assert mk.matcher_defaults("v2") == (0.60, 0.70)
    assert mk.Vault(str(tmp_path), matcher="v3").params() == mk.matcher_defaults("v3")
    assert mk.Vault(str(tmp_path), matcher="v2").params() == (0.60, 0.70)


def test_mini_vault_hashes_python_equals_cli(tmp_path):
    hashes = {}
    for m in ("v2", "v3"):
        v = mk.Vault(str(tmp_path / f"py-{m}"), matcher=m)
        v.ingest_dir(str(MINI))
        assert v.matcher_lock() == m
        hashes[m] = v.events_hash()
        assert cli_hash(tmp_path / f"cli-{m}", m) == hashes[m]
    assert hashes["v2"] == V2_MINI_HASH
    assert hashes["v3"] != V2_MINI_HASH


def test_mixed_versions_raise(tmp_path):
    v = mk.Vault(str(tmp_path), matcher="v2")
    v.ingest("d.md", doc(A))
    w = mk.Vault(str(tmp_path), matcher="v3")
    assert w.matcher_lock() == "v2"
    with pytest.raises(RuntimeError, match="locked to matcher v2"):
        w.ingest("d.md", doc(A, B))


def test_k4_split_through_binding(tmp_path):
    v = mk.Vault(str(tmp_path), matcher="v3")
    v.ingest("d.md", doc(f"{A} {B}"))
    (old,) = [lin for lin, _ in v.snapshot("d.md")["blocks"]]
    r = v.ingest("d.md", doc(A, B))
    new = [lin for lin, _ in v.snapshot("d.md")["blocks"]]
    assert new[0] == old and new[1] != old
    (ev,) = [e for e in r["events"] if e["kind"] == "BlockSplit"]
    assert ev["meta"]["dominant"] == old
    assert ev["meta"]["matcher_version"] == "v3"
