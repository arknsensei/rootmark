#!/usr/bin/env bash
# Deliberate-violation test for scripts/check_deps.sh (TASKS A1, VERIFY V7):
# in scratch copies of the workspace manifests and sources, add one reverse /
# forbidden dependency at a time and require check_deps.sh to fail; the
# unmodified copy must pass. The working tree is never touched.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch="$(mktemp -d -t rootmark-check-deps.XXXXXX)"
trap 'rm -rf "$scratch"' EXIT

fresh() { # fresh <name> → path of a new copy of the workspace (no target/, no data)
  local d="$scratch/$1"
  mkdir -p "$d"
  (cd "$here" && tar --exclude node_modules -cf - Cargo.toml Cargo.lock rust-toolchain.toml .cargo scripts \
      core index server bindings/python \
      $( [ -d clients ] && echo clients ) \
      $( [ -f docs/CLIENT_CONTRACT.md ] && echo docs/CLIENT_CONTRACT.md ) \
      $( [ -f research/pyproject.toml ] && echo research/pyproject.toml ) \
      $( [ -d research/tests ] && echo research/tests ) ) | tar -xf - -C "$d"
  echo "$d"
}

expect() { # expect pass|fail <copy> <label>
  local want="$1" d="$2" label="$3" rc=0
  CARGO_TARGET_DIR="$scratch/target" bash "$d/scripts/check_deps.sh" "$d" >"$scratch/out" 2>&1 || rc=$?
  if { [ "$want" = pass ] && [ "$rc" -eq 0 ]; } || { [ "$want" = fail ] && [ "$rc" -ne 0 ]; }; then
    echo "ok   [$want] $label"
  else
    echo "FAIL [$want expected, exit $rc] $label" >&2
    cat "$scratch/out" >&2
    exit 1
  fi
}

add_dep() { # add_dep <manifest> <line>  (appended to [dependencies])
  awk -v line="$2" '{print} /^\[dependencies\]/{print line}' "$1" >"$1.new" && mv "$1.new" "$1"
  grep -q "^\[dependencies\]" "$1" || printf '\n[dependencies]\n%s\n' "$2" >>"$1"
}

d="$(fresh clean)"; expect pass "$d" "unmodified workspace"

d="$(fresh core-index)"
add_dep "$d/core/Cargo.toml" 'rootmark-index = { path = "../index" }'
expect fail "$d" "core depends on index"

d="$(fresh index-server)"
add_dep "$d/index/Cargo.toml" 'rootmark-server = { path = "../server" }'
expect fail "$d" "index depends on server"

d="$(fresh core-dev-server)"
printf '\n[dev-dependencies]\nrootmark-server = { path = "../server" }\n' >>"$d/core/Cargo.toml"
expect fail "$d" "core dev-depends on server"

d="$(fresh core-table-form)"
printf '\n[dependencies.rootmark-server]\npath = "../server"\n' >>"$d/core/Cargo.toml"
expect fail "$d" "core depends on server (table form)"

d="$(fresh research-import)"
mkdir -p "$d/research/eval"
printf 'import sys\nsys.path.insert(0, "../server")\n' >"$d/research/eval/bad.py"
expect fail "$d" "research reaches past bindings/python"

d="$(fresh client-import)"
mkdir -p "$d/clients/obsidian/src"
printf "import { parse } from '../../../core/src/parse';\n" >"$d/clients/obsidian/src/bad.ts"
expect fail "$d" "client imports core directly"

d="$(fresh client-import-server)"
mkdir -p "$d/clients/obsidian/src"
printf "import { Kernel } from '../../../server/src/tools';\n" >"$d/clients/obsidian/src/bad.ts"
expect fail "$d" "client imports server directly (P1-E/E0)"

d="$(fresh client-path)"
mkdir -p "$d/clients/obsidian/src"
printf 'export const p = "/v1/demote";\n' >"$d/clients/obsidian/src/bad.ts"
expect fail "$d" "client names an HTTP path outside the contract (P1-E/E0)"

d="$(fresh client-path-ok)"
mkdir -p "$d/clients/obsidian/src"
printf 'export const p = "/v1/promote";\n' >"$d/clients/obsidian/src/ok.ts"
expect pass "$d" "client names a contract endpoint"

echo "test_check_deps: OK"
