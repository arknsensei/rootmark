#!/usr/bin/env bash
# Freeze checkers must not depend on compiled, tunable matcher constants
# (GOAL P1-A "denetleyiciler derlenmiş sabitlere bağlı olamaz"; TASKS A4).
#
# In a scratch copy of the working tree (tracked + untracked, not ignored):
#   1. every compiled THETA constant in core/src/lineage.rs is changed;
#   2. the extension is rebuilt into the copy's own research/.venv;
#   3. control: `matcher_defaults()` reports the changed values, and the
#      v3-final `freeze_check.py` (which read the compiled defaults) FAILS;
#   4. `make freeze-check` and `make freeze-check-v3` still pass with the
#      digests of the unmodified tree.
# The working tree is never touched.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch="$(mktemp -d -t mdk-freeze-indep.XXXXXX)"
trap 'rm -rf "$scratch"' EXIT
log() { printf '[freeze-indep] %s\n' "$*" >&2; }

copy="$scratch/repo"
mkdir -p "$copy"
(cd "$here" && git ls-files -co --exclude-standard -z | xargs -0 tar -cf -) | tar -xf - -C "$copy"
# git metadata for runinfo-free checkers is not needed; v3-final checker comes from history
git -C "$here" show v3-final:eval/freeze_check.py >"$copy/research/eval/freeze_check_v3final.py"

lin="$copy/core/src/lineage.rs"
for c in THETA THETA_SPLIT THETA_V3 THETA_SPLIT_V3; do
  grep -Eq "^pub const $c: f64 = [0-9.]+;" "$lin" || { echo "error: constant $c not found" >&2; exit 1; }
done
sed -E -i.bak \
  -e 's/^pub const THETA: f64 = [0-9.]+;/pub const THETA: f64 = 0.91;/' \
  -e 's/^pub const THETA_SPLIT: f64 = [0-9.]+;/pub const THETA_SPLIT: f64 = 0.92;/' \
  -e 's/^pub const THETA_V3: f64 = [0-9.]+;/pub const THETA_V3: f64 = 0.93;/' \
  -e 's/^pub const THETA_SPLIT_V3: f64 = [0-9.]+;/pub const THETA_SPLIT_V3: f64 = 0.94;/' "$lin"
rm -f "$lin.bak"
grep -E '^pub const THETA' "$lin" >&2

log "build modified core + extension in the copy"
export CARGO_TARGET_DIR="$scratch/target"
(cd "$copy" && cargo build --release -q -p mdkernel-bindings)
(cd "$copy/research" && uv sync --frozen -q && uv run --frozen maturin develop --release --quiet)

py() { (cd "$copy/research" && uv run --frozen python "$@"); }

log "control 1: compiled defaults changed"
got="$(py -c 'import mdkernel_core as m; print(m.matcher_defaults("v2"), m.matcher_defaults("v3"))')"
[ "$got" = "(0.91, 0.92) (0.93, 0.94)" ] || { echo "error: constants not in effect: $got" >&2; exit 1; }

log "control 2: the v3-final checker (compiled defaults) must fail"
if (cd "$copy/research" && MDK_MATCHER=v2 uv run --frozen python -m eval.freeze_check_v3final) >"$scratch/old.log" 2>&1; then
  echo "error: v3-final freeze_check passed with changed constants; the test has no power" >&2
  exit 1
fi
grep -q "FAILED" "$scratch/old.log" || { tail -5 "$scratch/old.log" >&2; echo "error: v3-final checker failed for another reason" >&2; exit 1; }

log "freeze checkers with changed constants"
out2="$(cd "$copy/research" && MDK_MATCHER=v2 uv run --frozen python -m eval.freeze_check 2>/dev/null | tail -1)"
out3="$(cd "$copy/research" && uv run --frozen python -m eval.freeze_check_v3 2>/dev/null | tail -1)"
echo "$out2" >&2
echo "$out3" >&2
[ "$out2" = "freeze-check OK (d489a0ed44b15d3aeb272e16cdd4a5af)" ] || { echo "error: freeze-check depends on compiled constants" >&2; exit 1; }
[ "$out3" = "freeze-check-v3 OK (d138cddcde1d0978a020a6fe8df2f610)" ] || { echo "error: freeze-check-v3 depends on compiled constants" >&2; exit 1; }
echo "test_freeze_indep: OK (checkers unchanged under THETA 0.91/0.92/0.93/0.94; v3-final checker fails)"
