#!/usr/bin/env bash
# D-0/R3 — does the outgoing set build on its own? Exports the files that go
# (allowlist) and the sources of the transformed files at their product paths
# (transforms NOT applied: a lower bound of D-1's tree) from <commit> into
# <dir>, then `cargo build --release --locked` of the workspace and
# `npm ci && npm run build` of both clients. Writes <dir>/export_check.json.
#
#   research/d0/export_check.sh <dir> [<commit>]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
dir="$1"; commit="${2:-$(cat "$root/docs/rounds/D-0/progress/ROUND_START")}"
rm -rf "$dir"; mkdir -p "$dir/tree"
python3 -I - "$root" "$commit" "$dir/tree" <<'PY'
import json, subprocess, sys, pathlib
root, commit, out = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
sys.path.insert(0, root + "/research/d0")
d = json.load(open(root + "/reports/rounds/D-0/transfer/classification.json"))
assert d["commit"] == commit, "classification.json is for another commit"
import classify  # noqa: E402  (same pattern logic)
files = [x for x in subprocess.run(["git", "-C", root, "ls-tree", "-r", "-z", "--name-only", commit], capture_output=True).stdout.decode().split("\0") if x]
TR = classify.TR
pats = [(p, "allow", None) for p, _ in classify.read_list(TR / "allowlist.txt")] + \
       [(p, "stays", None) for p, _ in classify.read_list(TR / "stays.txt")] + \
       [(p, "transform", dest) for p, _, _, dest in classify.read_transforms(TR / "transforms.md")[0]]
n = 0
for f in files:
    hits = [(len(p), c, p, x) for p, c, x in pats if classify.matches(p, f)]
    _, c, p, x = max(hits)
    if c == "stays":
        continue
    dst = f if c == "allow" else classify.dest_of(f, p, x)
    blob = subprocess.run(["git", "-C", root, "show", f"{commit}:{f}"], capture_output=True).stdout
    t = out / dst
    t.parent.mkdir(parents=True, exist_ok=True)
    t.write_bytes(blob)
    n += 1
print(f"exported {n} files")
PY
cd "$dir/tree"
res=()
t0=$(date +%s); if CARGO_TARGET_DIR="$dir/target" cargo build --release --locked --workspace >"$dir/cargo.log" 2>&1; then res+=("\"cargo_build_release\": \"ok\""); else res+=("\"cargo_build_release\": \"failed\""); fi
res+=("\"cargo_s\": $(( $(date +%s) - t0 ))")
for c in obsidian vscode; do
  if (cd "clients/$c" && npm ci --ignore-scripts >"$dir/npm_$c.log" 2>&1 && npm run build >>"$dir/npm_$c.log" 2>&1); then res+=("\"client_$c\": \"ok\""); else res+=("\"client_$c\": \"failed\""); fi
done
( IFS=,; echo "{\"commit\": \"$commit\", ${res[*]}}" ) | tee "$dir/export_check.json"
