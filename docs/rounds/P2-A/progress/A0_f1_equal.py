"""P2-A/F1 before/after: every changed harness ends with the same effective
policy. Old = the recipe `rootmark serve` wrote at ROUND_START (base +
commented `[attestation]`) with the harness's old transform; new = the
recipe it writes now (base + `[attestation] unattested = "canonical"`)
with the new transform. Both are loaded by `rootmark policy show` (the vault path aside) and
parsed as TOML; both must be equal. Usage: python3 A0_f1_equal.py <rootmark> <workdir>"""

import json
import subprocess
import tomllib
import sys
from pathlib import Path

rm, work = sys.argv[1], Path(sys.argv[2])
OLD_TAIL = (
    "\n# external edits no client attested (spec 2.0.0 §7.1): \"unverified\"\n"
    "# (the default) or \"canonical\" (every file edit counts as the human's)\n"
    "# [attestation]\n# unattested = \"unverified\"\n# window_s = 2.0\n"
)
DEFAULT = '[attestation]\nunattested = "canonical"\n'


def served(v: Path) -> str:
    v.mkdir(parents=True)
    (v / "a.md").write_text("# A\n", encoding="utf-8")
    subprocess.run([rm, "ingest-dir", str(v), str(v)], check=True, capture_output=True)
    subprocess.run([rm, "serve", "--vault", str(v), "--mcp", "stdio"], check=True,
                   capture_output=True, stdin=subprocess.DEVNULL)
    return (v / ".rootmark/policy.toml").read_text(encoding="utf-8")


new = served(work / "v")
base = new[: new.index("\n# external edits")]
old = base + OLD_TAIL
assert DEFAULT in new and new.count("[attestation]") == 1


def show(text: str, name: str) -> str:
    v = work / name
    v.mkdir()
    (v / "a.md").write_text("# A\n", encoding="utf-8")
    subprocess.run([rm, "ingest-dir", str(v), str(v)], check=True, capture_output=True)
    (v / ".rootmark/policy.toml").write_text(text, encoding="utf-8")
    r = subprocess.run([rm, "policy", "show", str(v)], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"{name}: {r.stderr}")
    shown = json.loads(r.stdout)
    shown.pop("source")  # the vault's path
    # what `policy show` prints (the kernel's part) and every table as parsed
    return json.dumps({"show": shown, "toml": tomllib.loads(text)}, sort_keys=True)


def common(t: str) -> str:  # the harnesses' shared steps (port, id)
    t = t.replace("[server]\n", "[server]\nhttp_port = 40123\n")
    return t.replace('# [identity]\n# human_id = "<your id>"\n', '[identity]\nhuman_id = "editor"\n')


UNV = '\n[attestation]\nunattested = "unverified"\nwindow_s = 0.5\n'
cases = {
    # run_p2_0.py, p2_0/sessions.ts, p2_0_1/real.ts, p2_0_2/real.ts, p2_0_2/sessions.ts
    "unverified_harnesses": (common(old) + UNV, common(new).replace(DEFAULT, "") + UNV),
    # serve_watch.rs: default_recipe() → base_recipe(), + canonical
    "serve_watch": (old + '\n[attestation]\nunattested = "canonical"\n',
                    base + "\n" + '\n[attestation]\nunattested = "canonical"\n'),
}
# http_e2e.sh: the sed of the new script, then the same printf
p = work / "http_e2e.toml"
p.write_text(new, encoding="utf-8")
subprocess.run(["sed", "-i.bak", '/^\\[attestation\\]$/{N;/\\nunattested = "canonical"$/d;}', str(p)], check=True)
cases["http_e2e"] = (old + '\n[attestation]\nunattested = "canonical"\n',
                     p.read_text(encoding="utf-8") + '\n[attestation]\nunattested = "canonical"\n')
out = {}
for k, (o, n) in cases.items():
    a, b = show(o, f"{k}-old"), show(n, f"{k}-new")
    out[k] = {"equal": a == b, "policy": json.loads(a)}
    if a != b:
        print(k, a, b, sep="\n")
print(json.dumps({k: v["equal"] for k, v in out.items()}))
sys.exit(0 if all(v["equal"] for v in out.values()) else 1)
