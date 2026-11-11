#!/usr/bin/env bash
# D-0/R5 supplement — offline_probe.py on Linux x86_64 in Docker (same image,
# binary and model checks as pq2_1_docker.sh).
#
#   research/d0/offline_docker.sh <linux rootmark dir> <models dir> <out.json> [reps]
set -euo pipefail
bin="$(cd "$1" && pwd)"; models="$(cd "$2" && pwd)"; out="$3"; reps="${4:-3}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["docker"]["image"])' "$root/research/p2q/scenario.json")"
mkdir -p "$(dirname "$out")"; outdir="$(cd "$(dirname "$out")" && pwd)"; outname="$(basename "$out")"
docker run --rm --name "d0-offline-$$" --platform linux/amd64 \
  -v "$root/research/d0:/d0/pkg/d0:ro" -v "$bin:/d0/bin:ro" \
  -v "$models:/d0/models:ro" -v "$outdir:/d0/out" \
  "$image" bash -c "
    set -euo pipefail
    apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends python3 >/dev/null
    cd /d0/pkg && python3 -E -s -m d0.offline_probe --rootmark /d0/bin/rootmark --models /d0/models \
      --work /tmp/off --out /d0/out/$outname --platform linux --reps $reps
  "
