#!/usr/bin/env bash
# P2-V.1/E2 (GOAL P2-V.1 Kapsam 3, V1-2): the processor-independence proof
# of the embedder, one runner's half. The fixed corpus is the designated
# subset of the P1-C corpus D3 (`make d3`, verified against
# research/data/MANIFEST.d3.json): every tenth document of
# research/data/d3/docs in bytewise name order (index 0, 10, 20, …; 102
# documents). For each manifest model (`minilm-l6-v2`, the default, and
# `multilingual-minilm-l12-v2` by `[index] embedder`) a vault of those
# documents is ingested, checkpointed (`t0`) and indexed; the vector files
# (`vectors/vectors.f32`, `vectors/embedders.jsonl`), the index manifest and
# a flat-mode search record (the scores are cosines of the stored vectors)
# for fixed queries are kept, with their blake2b. The processor model goes
# into cpu.json. `research/p2v1/e2_compare.py` compares the runners.
#
#   scripts/embed_cpu.sh <rootmark binary> <out dir> <label>
set -euo pipefail

bin="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="$2"
label="$3"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docs="$root/research/data/d3/docs"
[ -x "$bin" ] || { echo "embed_cpu: $bin is not executable" >&2; exit 1; }
[ -d "$docs" ] || { echo "embed_cpu: $docs missing (make d3)" >&2; exit 1; }
mkdir -p "$out"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

if [ "$(uname)" = Darwin ]; then
  cpu="$(sysctl -n machdep.cpu.brand_string)"
else
  cpu="$(lscpu | sed -n 's/^Model name:[[:space:]]*//p' | head -1)"
fi
jq -n --arg label "$label" --arg arch "$(uname -m)" --arg os "$(uname -s)" --arg cpu "$cpu" \
  '{label: $label, arch: $arch, os: $os, cpu_model: $cpu}' >"$out/cpu.json"

# the designated subset: every tenth document, bytewise name order
mkdir -p "$work/corpus"
LC_ALL=C ls "$docs" | LC_ALL=C sort | awk 'NR % 10 == 1' >"$out/corpus.txt"
while IFS= read -r f; do cp "$docs/$f" "$work/corpus/$f"; done <"$out/corpus.txt"
[ "$(wc -l <"$out/corpus.txt")" -eq 102 ] || { echo "embed_cpu: corpus is not 102 documents" >&2; exit 1; }

queries=(
  "how do I install the command line tool"
  "pricing for teams"
  "release notes for the latest version"
  "keyboard shortcuts"
  "privacy and data retention"
  "integrations with other apps"
  "Türkçe arama nasıl çalışır"
  "careers and open positions"
)

for model in minilm-l6-v2 multilingual-minilm-l12-v2; do
  v="$work/$model/vault"
  mkdir -p "$v"
  "$bin" ingest-dir "$v" "$work/corpus" >/dev/null
  if [ "$model" != minilm-l6-v2 ]; then
    # the recipe of the P2-L recorder (`research/eval/run_p2l.py` RECIPE)
    printf '[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 168\n\n[index]\ntokenizer = "mdk_lower_alnum"\nembedder = "%s"\n' "$model" \
      >"$v/.rootmark/policy.toml"
  fi
  "$bin" checkpoint create --name t0 "$v" >/dev/null
  "$bin" index build "$v" >"$out/$model.build.json"
  mkdir -p "$out/$model"
  cp "$v/.rootmark/index/vectors/vectors.f32" "$v/.rootmark/index/vectors/embedders.jsonl" \
    "$v/.rootmark/index/manifest.json" "$out/$model/"
  : >"$out/$model/flat.jsonl"
  for q in "${queries[@]}"; do
    "$bin" search --mode flat --k 10 --at t0 --consumer embed-cpu "$v" "$q" \
      | jq -c --arg q "$q" '{query: $q, hits: [.hits[] | {doc_path, chunk_id, score}]}' >>"$out/$model/flat.jsonl"
  done
done

# blake2b of every kept file (the comparison reads these and the files)
python3 - "$out" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1])
files = sorted(p for p in out.rglob("*") if p.is_file() and p.name not in ("hashes.json",))
h = {str(p.relative_to(out)): hashlib.blake2b(p.read_bytes()).hexdigest() for p in files}
(out / "hashes.json").write_text(json.dumps(h, indent=1, sort_keys=True) + "\n")
PY
echo "embed_cpu: $label ($cpu) done"
