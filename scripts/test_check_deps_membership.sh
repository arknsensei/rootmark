#!/usr/bin/env bash
# P2-Q.3/R3 test of scripts/check_deps.sh's endpoint membership (step 3b;
# docs/rounds/P2-Q.3/progress/R3_check_deps.md): in scratch copies (the
# working tree is never touched),
#   1. a `grep -qx attest_session` that answers "no" (standing in for the
#      false answer seen on macOS CI, run 43241761070) → check_deps passes,
#      with a NOTE line naming the pipeline's exit statuses, no VIOLATION;
#   2. a client naming a path outside the contract → check_deps fails, with
#      the diagnosis lines (endpoints read, files naming the path);
#   3. the unmodified copy → passes with no NOTE.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch="$(mktemp -d -t rootmark-check-deps-m.XXXXXX)"
trap 'rm -rf "$scratch"' EXIT

fresh() { # fresh <name> → a copy of what check_deps.sh reads
  local d="$scratch/$1"
  mkdir -p "$d"
  (cd "$here" && tar --exclude node_modules -cf - Cargo.toml Cargo.lock rust-toolchain.toml .cargo scripts \
      core index server bindings/python clients docs/CLIENT_CONTRACT.md \
      $( [ -f research/pyproject.toml ] && echo research/pyproject.toml ) \
      $( [ -d research/tests ] && echo research/tests ) ) | tar -xf - -C "$d"
  echo "$d"
}

fail() { echo "FAIL $*" >&2; cat "$scratch/err" >&2; exit 1; }

check() { # check <copy> [PATH prefix] → rc; stderr in $scratch/err
  local d="$1" pre="${2:-}" rc=0
  if [ -n "$pre" ]; then
    PATH="$pre:$PATH" CARGO_TARGET_DIR="$scratch/target" bash "$d/scripts/check_deps.sh" "$d" >"$scratch/out" 2>"$scratch/err" || rc=$?
  else
    CARGO_TARGET_DIR="$scratch/target" bash "$d/scripts/check_deps.sh" "$d" >"$scratch/out" 2>"$scratch/err" || rc=$?
  fi
  echo "$rc"
}

# 1. a false "no" from `grep -qx`
real="$(command -v grep)"
mkdir -p "$scratch/fake"
cat >"$scratch/fake/grep" <<EOF
#!/usr/bin/env bash
if [ "\${1:-}" = "-qx" ] && [ "\${2:-}" = "attest_session" ]; then cat >/dev/null; exit 1; fi
exec "$real" "\$@"
EOF
chmod +x "$scratch/fake/grep"
d="$(fresh one)"
rc="$(check "$d" "$scratch/fake")"
[ "$rc" -eq 0 ] || fail "[pass expected, exit $rc] a false grep -qx answer is not a violation"
grep -q "check_deps: NOTE: the pipeline 'echo | grep -qx attest_session' failed (exit statuses: 0 1)" "$scratch/err" ||
  fail "the NOTE line with the exit statuses is missing"
! grep -q "VIOLATION" "$scratch/err" || fail "a VIOLATION for a listed endpoint"
echo "ok   [pass] a false grep -qx answer: NOTE with exit statuses, no violation"

# 2. a path outside the contract: a violation with its diagnosis
d="$(fresh two)"
printf 'export const p = "/v1/delete";\n' >"$d/clients/obsidian/src/bad.ts"
rc="$(check "$d")"
[ "$rc" -ne 0 ] || fail "[fail expected, exit 0] a path outside the contract"
grep -q "check_deps: VIOLATION: clients/ names /v1/delete, not an endpoint" "$scratch/err" || fail "the violation line is missing"
grep -q "check_deps: diagnosis: pipeline exit statuses: 0 1; endpoints read from docs/CLIENT_CONTRACT.md" "$scratch/err" ||
  fail "the endpoints diagnosis is missing"
grep -q "check_deps: diagnosis: files naming /v1/delete: clients/obsidian/src/bad.ts" "$scratch/err" ||
  fail "the files diagnosis is missing"
echo "ok   [fail] a path outside the contract: violation with its diagnosis"

# 3. unmodified: no NOTE
d="$(fresh three)"
rc="$(check "$d")"
[ "$rc" -eq 0 ] || fail "[pass expected, exit $rc] unmodified copy"
! grep -q "NOTE" "$scratch/err" || fail "a NOTE on an unmodified copy"
echo "ok   [pass] unmodified copy, no NOTE"
echo "test_check_deps_membership: OK"
