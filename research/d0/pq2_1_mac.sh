#!/usr/bin/env bash
# D-0/R5 — PQ2-1 probe on macOS arm64 (DESIGN §4): the published 0.3.2
# binary (checked against SHA256SUMS by the caller), the pinned model files
# (checked here against models/MANIFEST.json), a synthetic vault in <work>.
#
#   research/d0/pq2_1_mac.sh <mac rootmark dir> <models dir> <work dir> <out.json> [reps]
set -euo pipefail
bin="$(cd "$1" && pwd)"; models="$(cd "$2" && pwd)"; work="$3"; out="$4"; reps="${5:-10}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
python3 -I - "$root/models/MANIFEST.json" "$models/minilm-l6-v2" <<'PY'
import hashlib, json, sys, pathlib
m = json.load(open(sys.argv[1])); d = pathlib.Path(sys.argv[2])
for name, f in m["files"].items():
    if hashlib.sha256((d / name).read_bytes()).hexdigest() != f["sha256"]:
        sys.exit(f"model file {name}: sha256 mismatch")
print("models: manifest ok")
PY
python3 -I "$root/research/d0/pq2_1_probe.py" --rootmark "$bin/rootmark" --models "$models" \
  --work "$work" --out "$out" --platform macos --reps "$reps"
