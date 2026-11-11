#!/usr/bin/env bash
# D-0/R5 — PQ2-1 probe on Linux x86_64 in Docker (DESIGN §4): the pinned
# image of research/p2q/scenario.json (debian:trixie-slim, linux/amd64), the
# published 0.3.2 Linux binary (checked against SHA256SUMS by the caller),
# the pinned model files (checked here against models/MANIFEST.json).
#
#   research/d0/pq2_1_docker.sh <linux rootmark dir> <models dir> <out.json> [reps]
#
# Mounts are read-only except the output directory; the container is
# removed at the end (--rm); only the container this script names is touched.
set -euo pipefail
bin="$(cd "$1" && pwd)"; models="$(cd "$2" && pwd)"; out="$3"; reps="${4:-10}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["docker"]["image"])' "$root/research/p2q/scenario.json")"
mkdir -p "$(dirname "$out")"; outdir="$(cd "$(dirname "$out")" && pwd)"; outname="$(basename "$out")"
python3 -I - "$root/models/MANIFEST.json" "$models/minilm-l6-v2" <<'PY'
import hashlib, json, sys, pathlib
m = json.load(open(sys.argv[1])); d = pathlib.Path(sys.argv[2])
for name, f in m["files"].items():
    h = hashlib.sha256((d / name).read_bytes()).hexdigest()
    if h != f["sha256"]:
        sys.exit(f"model file {name}: sha256 mismatch")
print("models: manifest ok")
PY
name="d0-pq21-$$"
docker run --rm --name "$name" --platform linux/amd64 \
  -v "$root/research/d0:/d0/src:ro" -v "$bin:/d0/bin:ro" \
  -v "$models:/d0/models:ro" -v "$outdir:/d0/out" \
  "$image" bash -c "
    set -euo pipefail
    apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends python3 >/dev/null
    python3 -I /d0/src/pq2_1_probe.py --rootmark /d0/bin/rootmark --models /d0/models \
      --work /tmp/pq21 --out /d0/out/$outname --platform linux --reps $reps
  "
