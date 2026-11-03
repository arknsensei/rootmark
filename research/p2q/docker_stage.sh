#!/usr/bin/env bash
# P2-Q Linux stage in Docker (GOAL Q3 (a); scenario.json `docker`): a fresh
# linux/amd64 container of the pre-registered image; inside it the guide's
# Linux-in-Docker block (docs/GUIDE.md §1.3, block 1.3#2; for qorbit also
# §1.2) installs the published release as written, then the headless scene
# runs with the same driver and pre-registration as on the Mac.
#
#   research/p2q/docker_stage.sh <corpus> <snapshot dir> <out dir> [<key file>]
#
# The snapshot (and, for a personal corpus, the path key) is mounted
# read-only and copied inside the container by the driver; nothing of it is
# written outside <out>, which receives only the driver's records (counts,
# hashes; personal paths as k2). The container is removed at the end
# (--rm); only containers this script names are touched. Python 3 is the
# harness's own prerequisite (installed before the guide's block).
set -euo pipefail
corpus="$1"; snap="$(cd "$2" && pwd)"; out="$3"; key="${4:-}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["docker"]["image"])' "$root/research/p2q/scenario.json")"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
blocks='1.3#2'
[ "$corpus" = qorbit ] && blocks='1.3#2,1.2#1'
keyargs=()
if [ -n "$key" ]; then keyargs=(-v "$(cd "$(dirname "$key")" && pwd)/$(basename "$key"):/p2q/key:ro" -e P2Q_PATH_KEY_FILE=/p2q/key); fi
name="p2q-docker-$corpus-$$"
t0=$(date +%s)
docker run --rm --name "$name" --platform linux/amd64 \
  -v "$root/research/p2q:/p2q/src/research/p2q:ro" \
  -v "$root/research/data/d_notes:/p2q/src/research/data/d_notes:ro" \
  -v "$root/docs/GUIDE.md:/p2q/src/docs/GUIDE.md:ro" \
  -v "$snap:/p2q/snapshot:ro" -v "$out:/p2q/out" "${keyargs[@]}" \
  "$image" bash -c "
    set -euo pipefail
    apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends python3 >/dev/null
    cd /p2q/src/research
    python3 -m p2q.guide_run --guide ../docs/GUIDE.md --work /p2q/work/guide --mode release --only '$blocks'
    cp /p2q/work/guide/guide_run.json /p2q/out/guide_run.json
    export ROOTMARK_MODELS=/p2q/work/guide/home/.local/share/rootmark/models
    python3 -m p2q.driver --corpus '$corpus' --snapshot /p2q/snapshot --work /p2q/work/scene \
      --out /p2q/out --rootmark /p2q/work/guide/home/.local/bin/rootmark
    grep -m1 'model name' /proc/cpuinfo | sed 's/.*: //' > /p2q/out/cpu.txt
  "
echo "docker_stage: $corpus done in $(( $(date +%s) - t0 )) s (emulated: information)"
