"""P2-A's sets (TASKS A5; GOAL P2-A A-1, A-2, A-3, A-7): the chain's half
of the round, through the `rootmark` binary only (research reaches the
kernel through the binary and `bindings/python`).

- `migrate_old` (A-2): the six old vaults — P2-0's three `v0.1.0` vaults
  (`research/data/p2_0_old_vaults`), P2-0.1's two `ce4cf04` vaults
  (`p2_0_1_old_vaults`), P2-0.2's `21b16ea` vault (`p2_0_2_old_vaults`) —
  each: `migrate --dry-run` (its report kept; no byte or directory of the
  vault changes), `migrate` (every migration that applies, with its
  records), then `events_hash`, every document's `replay` and `blame`
  equal to the values before the migration and to the vault's committed
  expectations (`expected.json`: `events_hash`, `replay`, `layers`), a
  second `migrate` a no-op, then `--rollback` of every record, newest
  first, and the vault byte for byte (every file and directory) as it was;
  `rootmark conformance` of this binary once (39/39, level 3).
- `fixture` (A-1, through the CLI): the pre-rename `/2` index fixture
  (`server/tests/fixtures/index_v2`) — `rename_store` and `index_rebuild`
  forward and back, byte-equal after the rollbacks.
- `tamper` (A5's deliberate cases): a record whose saved copy no longer
  matches its SHA-256 → rollback refused, vault unchanged; the vault's
  index changed since the migration → refused; the older `/2` index with
  `[index] auto_rebuild = false` behind `rootmark serve --http` →
  `search` answers 503 `migration_required` with a hint, `status` 200 with
  `migrations.artifacts.index.state = "migration_required"`. The case of a
  migration that would change `events_hash` needs a migration this binary
  does not register: it is the server test
  `a_migration_that_would_change_events_hash_stops` (`tests`).
- `tests` (A-1, A-3, A-6, A-7): the round's server tests (`p2a_migrate`,
  `p2a_status`, `p2_a_a0`, `p2_0_2_carry`, `propose_kernel`, `contract`) run
  with `cargo test`; each test's result.

    python -m eval.run_p2a --run <RUN> --rootmark <bin> [--only a,b]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

from eval import runinfo

REPO = Path(__file__).resolve().parents[2]
DATA = REPO / "research/data"
OLD = [
    ("v0.1.0", DATA / "p2_0_old_vaults", ["e2e", "mixed", "scene"]),
    ("ce4cf04", DATA / "p2_0_1_old_vaults", ["attested", "bulk"]),
    ("21b16ea", DATA / "p2_0_2_old_vaults", ["partial"]),
]
FIXTURE = REPO / "server/tests/fixtures/index_v2/vault"
SETS = ["migrate_old", "fixture", "tamper", "tests"]
TESTS = ["p2a_migrate", "p2a_status", "p2_a_a0", "p2_0_2_carry", "propose_kernel", "contract"]


def run(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, check=False, **kw)


def ok_json(cmd: list[str]) -> dict:
    r = run(cmd)
    if r.returncode != 0:
        raise RuntimeError(f"{' '.join(cmd)}: exit {r.returncode}\n{r.stderr[-2000:]}")
    return json.loads(r.stdout)


def tree(d: Path) -> dict:
    """Every file (SHA-256) and every directory under `d`."""
    out = {}
    for p in sorted(d.rglob("*")):
        rel = p.relative_to(d).as_posix()
        out[rel] = "dir" if p.is_dir() else hashlib.sha256(p.read_bytes()).hexdigest()
    return out


def answers(rm: str, v: Path, docs: list[str]) -> dict:
    out = {
        "events_hash": ok_json([rm, "events-hash", str(v)])["events_hash"],
        "replay": {},
        "blame": {},
    }
    for d in docs:
        out["replay"][d] = ok_json([rm, "replay", str(v), d])
        out["blame"][d] = ok_json([rm, "blame", str(v), d])
    return out


def records(v: Path) -> list[str]:
    d = v / ".rootmark/migrations"
    return sorted(x.name for x in d.iterdir()) if d.is_dir() else []


def roll_back_all(rm: str, v: Path) -> list[dict]:
    out = []
    for rid in reversed(records(v)):
        r = run([rm, "migrate", "--rollback", rid, str(v)])
        out.append(
            {
                "id": rid,
                "exit": r.returncode,
                "byte_equal": r.returncode == 0 and json.loads(r.stdout).get("byte_equal") is True,
            }
        )
        if r.returncode != 0:
            break
    return out


def migrate_vault(rm: str, v: Path, docs: list[str], want: dict | None) -> dict:
    t0 = tree(v)
    before = answers(rm, v, docs)
    dry = ok_json([rm, "migrate", "--dry-run", str(v)])
    after_dry = tree(v)
    mig = ok_json([rm, "migrate", str(v)])
    after = answers(rm, v, docs)
    again = ok_json([rm, "migrate", str(v)])
    recs = records(v)
    noop = again["applied"] == [] and len(recs) == len(mig["applied"])
    back = roll_back_all(rm, v)
    t1 = tree(v)
    ok = {
        "dry_run_changes_nothing": after_dry == t0,
        "migrated": bool(mig["applied"]),
        "events_hash_equal": after["events_hash"] == before["events_hash"],
        "replay_equal": after["replay"] == before["replay"],
        "blame_equal": after["blame"] == before["blame"],
        "second_migrate_noop": noop,
        "rollback_all": bool(back) and all(b["exit"] == 0 and b["byte_equal"] for b in back),
        "byte_equal_after_rollback": t1 == t0,
    }
    if want is not None:
        ok["events_hash_expected"] = after["events_hash"] == want["events_hash"]
        ok["replay_expected"] = all(after["replay"][d] == want["replay"][d] for d in want["docs"])
        ok["layers_expected"] = all(
            [[b["lineage_id"], b["layer"]] for b in after["blame"][d]["blocks"]]
            == want["layers"][d]
            for d in want["docs"]
        )
    return {
        "dry_run": {
            "plan": [
                {k: p[k] for k in ("migration", "required", "from", "to", "bytes")}
                for p in dry["plan"]
            ],
            "matrix": dry["matrix"],
        },
        "applied": [a["id"] for a in mig["applied"]],
        "events_hash": after["events_hash"],
        "documents": len(docs),
        "rollbacks": back,
        "checks": ok,
        "pass": all(ok.values()),
    }


def run_migrate_old(rm: str, work: Path) -> dict:
    rows = []
    for written_by, base, names in OLD:
        for name in names:
            src = base / name
            want = json.loads((src / "expected.json").read_text())
            v = work / f"{written_by}-{name}"
            shutil.copytree(src / "vault", v)
            shutil.copytree(src / "store", v / ".rootmark")
            r = migrate_vault(rm, v, want["docs"], want)
            rows.append({"vault": f"{written_by}/{name}", **r})
    conf = run([rm, "conformance"])
    c = json.loads(conf.stdout) if conf.returncode == 0 else {}
    conformance = {
        "exit": conf.returncode,
        "level_claimed": c.get("level_claimed"),
        "passed": c.get("passed"),
        "total": c.get("total"),
        "pass": conf.returncode == 0
        and c.get("level_claimed") == 3
        and c.get("passed") == c.get("total") == 39,
    }
    return {
        "set": "migrate_old (A-2)",
        "vaults": rows,
        "passed": sum(r["pass"] for r in rows),
        "conformance": conformance,
        "pass": len(rows) == 6 and all(r["pass"] for r in rows) and conformance["pass"],
    }


def fixture_copy(work: Path, name: str) -> Path:
    d = work / name
    shutil.copytree(FIXTURE, d / "vault")
    return d / "vault"


def run_fixture(rm: str, work: Path) -> dict:
    v = fixture_copy(work, "fixture")
    t0 = tree(v)
    mig = ok_json([rm, "migrate", str(v)])
    applied = [a["migration"] for a in mig["applied"]]
    idx = json.loads((v / ".rootmark/index/manifest.json").read_text())["format"]
    back = roll_back_all(rm, v)
    t1 = tree(v)
    ok = {
        "applied_in_order": applied
        == ["rename_store", "index_rebuild", "log_readers", "checkpoints"],
        "index_migrated": idx == "mdkernel-index/3",
        "required_empty_after": mig["matrix"]["required"] == [],
        "rollback_all": len(back) == 4 and all(b["exit"] == 0 and b["byte_equal"] for b in back),
        "byte_equal_after_rollback": t1 == t0,
    }
    return {
        "set": "fixture (A-1)",
        "applied": applied,
        "index_format_after": idx,
        "rollbacks": back,
        "checks": ok,
        "pass": all(ok.values()),
    }


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def post(port: int, token: str, tool: str, body: dict) -> tuple[int, dict]:
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/v1/{tool}",
        data=json.dumps(body).encode(),
        headers={
            "Authorization": f"Bearer {token}",
            "X-Consumer": "p2a-tamper",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read())


def run_tamper(rm: str, work: Path) -> dict:
    cases = {}
    # 1. a damaged copy in a record
    v = fixture_copy(work, "t1")
    ok_json([rm, "migrate", str(v)])
    for rid in ["0004-checkpoints", "0003-log_readers"]:
        ok_json([rm, "migrate", "--rollback", rid, str(v)])
    copy = v / ".rootmark/migrations/0002-index_rebuild/files/.rootmark/index/manifest.json"
    good = copy.read_bytes()
    copy.write_bytes(good + b" ")
    t0 = tree(v)
    r = run([rm, "migrate", "--rollback", "0002-index_rebuild", str(v)])
    cases["damaged_record_refused"] = {
        "exit": r.returncode,
        "message": r.stderr.strip().replace(str(v), "<vault>")[-200:],
        "pass": r.returncode != 0 and "SHA-256" in r.stderr and tree(v) == t0,
    }
    copy.write_bytes(good)
    # 2. the index changed since the migration
    m = v / ".rootmark/index/manifest.json"
    mb = m.read_bytes()
    m.write_bytes(mb + b"\n")
    t0 = tree(v)
    r = run([rm, "migrate", "--rollback", "0002-index_rebuild", str(v)])
    cases["changed_vault_refused"] = {
        "exit": r.returncode,
        "message": r.stderr.strip().replace(str(v), "<vault>")[-200:],
        "pass": r.returncode != 0 and "changed since" in r.stderr and tree(v) == t0,
    }
    m.write_bytes(mb)
    # 3. the /2 index, auto_rebuild = false, a served vault
    v = fixture_copy(work, "t3")
    ok_json([rm, "migrate", "--rename-store", str(v)])
    port = free_port()
    # the server's own recipe, written by `rootmark serve` into another,
    # empty vault: started here, the first `serve` would rebuild the old
    # index (`auto_rebuild = true`, K-E0.3) before the case is measured
    e = work / "recipe"
    e.mkdir(parents=True)
    (e / "a.md").write_text("# A\n", encoding="utf-8")
    ok_json([rm, "ingest-dir", str(e), str(e)])
    run([rm, "serve", "--vault", str(e), "--mcp", "stdio"], stdin=subprocess.DEVNULL)
    t = (e / ".rootmark/policy.toml").read_text(encoding="utf-8")
    t = t.replace("[server]\n", f"[server]\nhttp_port = {port}\n").replace(
        "[index]\n", "[index]\nauto_rebuild = false\n"
    )
    t = t.replace('# [identity]\n# human_id = "<your id>"\n', '[identity]\nhuman_id = "editor"\n')
    (v / ".rootmark/policy.toml").write_text(t, encoding="utf-8")
    fmt0 = json.loads((v / ".rootmark/index/manifest.json").read_text())["format"]
    p = subprocess.Popen(
        [rm, "serve", "--vault", str(v), "--http", "127.0.0.1"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        token = None
        for _ in range(300):
            tp = v / ".rootmark/server.token"
            if tp.is_file():
                try:
                    socket.create_connection(("127.0.0.1", port), timeout=0.2).close()
                    token = tp.read_text().strip()
                    break
                except OSError:
                    pass
            time.sleep(0.1)
        if token is None:
            raise RuntimeError("rootmark serve did not listen")
        s_st, s_body = post(port, token, "search", {"query": "extinguisher"})
        st_st, st_body = post(port, token, "status", {})
    finally:
        p.terminate()
        _, err = p.communicate(timeout=60)
    ix = st_body.get("migrations", {}).get("artifacts", {}).get("index", {})
    cases["old_index_migration_required"] = {
        "index_format": fmt0,
        "search_status": s_st,
        "search_code": s_body.get("error", {}).get("code"),
        # the vault's path is this run's temporary directory
        "search_hint": (s_body.get("error", {}).get("hint") or "").replace(str(v), "<vault>"),
        "status_status": st_st,
        "status_index_state": st_body.get("index", {}).get("state"),
        "matrix_index_state": ix.get("state"),
        "serve_log_says_migrate": "rootmark migrate" in err,
        "pass": fmt0 == "mdkernel-index/2"
        and s_st == 503
        and s_body.get("error", {}).get("code") == "migration_required"
        and "rootmark migrate" in (s_body.get("error", {}).get("hint") or "")
        and st_st == 200
        and ix.get("state") == "migration_required"
        and "rootmark migrate" in err,
    }
    return {"set": "tamper (A5)", "cases": cases, "pass": all(c["pass"] for c in cases.values())}


def run_tests() -> dict:
    args = ["cargo", "test", "-p", "rootmark-server", "--release"]
    for t in TESTS:
        args += ["--test", t]
    r = run([*args, "--", "--test-threads=1"], cwd=REPO)
    res = {}
    for line in (r.stdout + r.stderr).splitlines():
        if line.startswith("test ") and line.endswith((" ok", " FAILED")):
            name = line[5:].rsplit(" ... ", 1)[0]
            res[name] = line.endswith(" ok")
    return {
        "set": "tests",
        "exit": r.returncode,
        "tests": res,
        "passed": sum(res.values()),
        "total": len(res),
        "pass": r.returncode == 0 and bool(res) and all(res.values()),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--only", default="")
    a = ap.parse_args(argv)
    rm = str(Path(a.rootmark).resolve())
    only = set(filter(None, a.only.split(",")))
    if only - set(SETS):
        raise ValueError(f"unknown sets {sorted(only - set(SETS))}")
    todo = [s for s in SETS if not only or s in only]
    out = runinfo.run_dir(a.run) / "p2a"
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rootmark-p2a-") as w:
        work = Path(w)
        for s in todo:
            if s == "migrate_old":
                r = run_migrate_old(rm, work / s)
            elif s == "fixture":
                r = run_fixture(rm, work / s)
            elif s == "tamper":
                r = run_tamper(rm, work / s)
            else:
                r = run_tests()
            (out / f"{s}.json").write_text(json.dumps(r, indent=1, ensure_ascii=False) + "\n")
    runinfo.record_phase(
        a.run,
        "p2a" + ("" if not only else "_" + "_".join(sorted(only))),
        {"sets": todo},
        [f"p2a/{s}.json" for s in todo],
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
