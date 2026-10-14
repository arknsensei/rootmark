#!/usr/bin/env bash
# Deliberate corruption tests (P1-A TASKS A6, VERIFY V8; P1-B TASKS B0: B-7,
# B-8). Every corruption is made in a scratch copy; the working tree is never
# touched. Each case must fail loudly with the named error.
#
#   scripts/test_tamper.sh <run-dir>
#
# <run-dir>: a complete P1-A RUN directory (research/reports/runs/<RUN>), used
# for the missing/changed result file cases. Requires `make setup` and the
# research datasets (research/data/{d3,d1h,d1ho}) in this checkout.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
run_dir="$(cd "${1:?usage: test_tamper.sh <run-dir>}" && pwd)"
[ -f "$run_dir/manifest.json" ] || { echo "error: $run_dir is not a RUN directory" >&2; exit 2; }
scratch="$(mktemp -d -t mdk-tamper.XXXXXX)"
trap 'rm -rf "$scratch"' EXIT
py="$here/research/.venv/bin/python"
fake_run="20261013T000000Z-0000000"
n=0

expect_fail() { # expect_fail <label> <pattern> <command...>
  local label="$1" pat="$2"; shift 2
  local rc=0
  "$@" >"$scratch/out" 2>&1 || rc=$?
  if [ "$rc" -eq 0 ]; then
    echo "FAIL $label: exit 0 (corruption not detected)" >&2; tail -5 "$scratch/out" >&2; exit 1
  fi
  if ! grep -Eq "$pat" "$scratch/out"; then
    echo "FAIL $label: exit $rc but no /$pat/ in output" >&2; tail -8 "$scratch/out" >&2; exit 1
  fi
  echo "ok   $label (exit $rc: $(grep -Eo "$pat" "$scratch/out" | head -1))"
  n=$((n + 1))
}

copy_tree() { # copy_tree <dest> — tracked + untracked (not ignored) files
  mkdir -p "$1"
  (cd "$here" && git ls-files -co --exclude-standard -z | xargs -0 tar -cf -) | tar -xf - -C "$1"
}

# --- 1. conformance fixtures (spec/fixtures is frozen and hash-listed)
conf="$scratch/conf"
copy_tree "$conf"
export CARGO_TARGET_DIR="$scratch/target"
flip_vid() { # flip one character of the first version_id in a JSON file
  "$py" - "$1" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
i = s.index('"version_id": "') + len('"version_id": "')
open(p, "w", encoding="utf-8").write(s[:i] + ("1" if s[i] == "0" else "0") + s[i + 1 :])
PY
}
flip_vid "$conf/spec/fixtures/C1-04/expected.json"
expect_fail "fixture expected value changed -> make conformance" "spec/fixtures changed|conformance failures" \
  make -C "$conf" conformance
# same corruption with MANIFEST.json updated: the case comparison still fails
"$py" - "$conf/spec/fixtures" <<'PY'
import json, sys
from pathlib import Path
import blake3
root = Path(sys.argv[1])
m = json.loads((root / "MANIFEST.json").read_text())
m["files"]["C1-04/expected.json"] = blake3.blake3((root / "C1-04/expected.json").read_bytes()).hexdigest()[:32]
(root / "MANIFEST.json").write_text(json.dumps(m, indent=2) + "\n")
PY
expect_fail "fixture expected value + manifest changed -> case mismatch" "C1-04: actual output differs" \
  make -C "$conf" conformance

# --- 2. holdout data (research/data/d1ho, MANIFEST.d1ho.json)
res="$scratch/res/research"
mkdir -p "$res"
(cd "$here/research" && git ls-files -co --exclude-standard -z | xargs -0 tar -cf -) | tar -xf - -C "$res"
mkdir -p "$res/data"
for d in d3 d1h d1ho; do cp -cR "$here/research/data/$d" "$res/data/" 2>/dev/null || cp -R "$here/research/data/$d" "$res/data/"; done
cp "$res/data/MANIFEST.d1ho.json" "$scratch/MANIFEST.d1ho.orig"
truth="docs/050/truth.json"

"$py" - "$res/data/MANIFEST.d1ho.json" "$truth" <<'PY'
import json, sys
p, key = sys.argv[1], sys.argv[2]
m = json.loads(open(p).read())
h = m["files"][key]
m["files"][key] = ("1" if h[0] == "0" else "0") + h[1:]
open(p, "w").write(json.dumps(m, indent=1, sort_keys=True) + "\n")
PY
expect_fail "holdout manifest hash changed -> run_holdout" "does not match MANIFEST.d1ho.json" \
  bash -c "cd '$res' && '$py' -m eval.run_holdout --run $fake_run"

cp "$scratch/MANIFEST.d1ho.orig" "$res/data/MANIFEST.d1ho.json"
"$py" - "$res/data" "$truth" <<'PY'
import json, sys
from pathlib import Path
import blake3
data, key = Path(sys.argv[1]), sys.argv[2]
tp = data / "d1ho" / key
t = json.loads(tp.read_text())
step = next(s for s in t["steps"] if s["scored"] and s["mapping"][0][0] is not None)
step["mapping"][0][0]["version_id"] = "0" * 32
tp.write_text(json.dumps(t, indent=1, sort_keys=True) + "\n")
mp = data / "MANIFEST.d1ho.json"
m = json.loads(mp.read_text())
m["files"][key] = blake3.blake3(tp.read_bytes()).hexdigest()[:32]
mp.write_text(json.dumps(m, indent=1, sort_keys=True) + "\n")
PY
expect_fail "holdout truth version_id zeroed (+ manifest) -> TruthError" "TruthError" \
  bash -c "cd '$res' && '$py' -m eval.run_holdout --run $fake_run"

# --- 3. RUN result files (pair comparison, A-4)
for name in holdout.json split_grid.json h1s_v3.json; do
  b="$scratch/run-missing-$name"
  cp -R "$run_dir" "$b"
  rm "$b/$name"
  expect_fail "RUN file $name missing -> compare_p1a" "missing" \
    bash -c "cd '$here/research' && '$py' -m eval.compare_p1a '$run_dir' '$b'"
done
b="$scratch/run-changed"
cp -R "$run_dir" "$b"
"$py" - "$b/holdout.json" <<'PY'
import json, sys
p = sys.argv[1]
d = json.loads(open(p).read())
d["H1-H1"] = d["H1-H1"] * (1 - 1e-12)
open(p, "w").write(json.dumps(d))
PY
expect_fail "holdout H1-H1 changed by 1e-12 relative -> compare_p1a" "A-4 FAILED" \
  bash -c "cd '$here/research' && '$py' -m eval.compare_p1a '$run_dir' '$b'"

# --- 4. moved target/ + corrupted fixture (P1-B B-7, P1-A recommendation Ö2):
# the copy keeps the built target/ (mtimes preserved, so cargo may reuse the
# test binary); the fixture root must still be the copy's, not a compiled-in
# path of this tree.
moved="$scratch/moved"
copy_tree "$moved"
cp -pR "$here/target" "$moved/target"
moved="$(cd "$moved" && pwd -P)"
flip_vid "$moved/spec/fixtures/C1-04/expected.json"
expect_fail "moved target/ + fixture changed -> make conformance (B-7)" "spec/fixtures changed|conformance failures" \
  env -u CARGO_TARGET_DIR make -C "$moved" conformance
got_root="$(jq -r .fixtures_root "$moved/reports/conformance.json")"
if [ "$got_root" != "$moved/spec/fixtures" ]; then
  echo "FAIL moved target/: conformance.json fixtures_root=$got_root, expected $moved/spec/fixtures" >&2
  exit 1
fi
echo "ok   moved target/: conformance.json fixtures_root is the copy's ($got_root)"

# --- 5. FROZEN.md frozen-table rows (P1-B B-8, P1-A recommendation Ö1): a
# clone of the committed HEAD, clean first, then rows removed or relaxed
fz="$scratch/frozen"
git clone --quiet "$here" "$fz"
bash "$fz/scripts/check_frozen.sh" "$fz" >"$scratch/out" 2>&1 \
  || { echo "FAIL check_frozen on a clean clone: exit $?" >&2; tail -8 "$scratch/out" >&2; exit 1; }
echo "ok   check_frozen clean clone -> exit 0"
row="$(grep -m1 -E '^\| `research/eval/\*\*`' "$fz/research/FROZEN.md")"
grep -vxF "$row" "$fz/research/FROZEN.md" >"$scratch/FROZEN.md" && cp "$scratch/FROZEN.md" "$fz/research/FROZEN.md"
expect_fail "FROZEN.md row deleted (uncommitted) -> check_frozen (B-8)" "rows removed or rewritten" \
  bash "$fz/scripts/check_frozen.sh" "$fz"
git -C "$fz" -c user.name=tamper -c user.email=tamper@invalid commit --quiet -am "tamper: drop a frozen row"
expect_fail "FROZEN.md row deleted (committed) -> check_frozen (B-8)" "rows removed or rewritten" \
  bash "$fz/scripts/check_frozen.sh" "$fz"
git -C "$fz" reset --quiet --hard HEAD~1
sed -i.bak 's/^| `scripts\/verify.sh` | `P1-A\/A6:`/| `scripts\/verify.sh` | `P1-Z\/never:`/' "$fz/research/FROZEN.md"
rm -f "$fz/research/FROZEN.md.bak"
git -C "$fz" -c user.name=tamper -c user.email=tamper@invalid commit --quiet -am "tamper: relax a frozen row"
expect_fail "FROZEN.md row relaxed (committed) -> check_frozen (B-8)" "rows removed or rewritten" \
  bash "$fz/scripts/check_frozen.sh" "$fz"

echo "test_tamper: OK ($n corruptions detected)"
