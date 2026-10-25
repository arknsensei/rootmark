#!/usr/bin/env bash
# Dependency direction check (docs/PACKAGES.md, GOAL A-6):
#   core ← index ← server ← clients;  research → bindings/python only.
#
#   scripts/check_deps.sh [<repo-root>]
#
# 1. Static scan of every workspace crate's Cargo.toml: internal (rootmark-*)
#    dependencies in any section must be in the allow list below.
# 2. `cargo tree` per crate (normal, build and dev edges, depth 1): same rule
#    on the resolved graph. A cargo failure (e.g. a cycle) is an error.
# 3. Import scanner: research/ Python may reach the kernel only through the
#    `rootmark_core` extension module; clients/ TypeScript/JavaScript may not
#    import core, index, bindings or research (only server's MCP/HTTP contract).
# Exit 0 = no reverse dependency; exit 1 = violations (listed on stderr).
set -euo pipefail

root="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
cd "$root"

# crate dir | package name | allowed internal dependencies
CRATES="core|rootmark-core|
index|rootmark-index|rootmark-core
server|rootmark-server|rootmark-core rootmark-index
bindings/python|rootmark-bindings|rootmark-core rootmark-index"

fail=0
violation() { echo "check_deps: VIOLATION: $*" >&2; fail=1; }

allowed() { # allowed <dep> <allow-list...>
  local d="$1"; shift
  for a in "$@"; do [ "$a" = "$d" ] && return 0; done
  return 1
}

# 0. every workspace member must be listed above (no silently unchecked crate)
members="$(grep -E '^members[[:space:]]*=' Cargo.toml | grep -Eo '"[^"]+"' | tr -d '"')"
for m in $members; do
  echo "$CRATES" | cut -d'|' -f1 | grep -qx "$m" || violation "workspace member $m is not in check_deps.sh's crate list"
done

# 1. static Cargo.toml scan
while IFS='|' read -r dir pkg allow; do
  manifest="$dir/Cargo.toml"
  [ -f "$manifest" ] || { violation "$manifest missing"; continue; }
  # dependency keys `rootmark-x = ...` and renamed deps `package = "rootmark-x"`
  deps="$( { grep -Eo '^[[:space:]]*rootmark-[a-z-]+[[:space:]]*=' "$manifest" | tr -d ' =' || true;
             grep -Eo 'package[[:space:]]*=[[:space:]]*"rootmark-[a-z-]+"' "$manifest" |
               grep -Eo 'rootmark-[a-z-]+' || true;
             grep -Eo '^\[[a-z.-]*dependencies\.rootmark-[a-z-]+\]' "$manifest" |
               grep -Eo 'rootmark-[a-z-]+' || true;
             grep -Eo '^[[:space:]]*rootmark-[a-z-]+\.workspace' "$manifest" |
               grep -Eo 'rootmark-[a-z-]+' || true; } | sort -u)"
  for d in $deps; do
    # shellcheck disable=SC2086
    allowed "$d" $allow || violation "$pkg ($manifest) depends on $d; allowed: ${allow:-none}"
  done
done <<<"$CRATES"

# 2. resolved graph
while IFS='|' read -r dir pkg allow; do
  if ! out="$(cargo tree -q -p "$pkg" -e normal,build,dev --prefix none --depth 1 \
               --format '{p}' 2>&1)"; then
    violation "cargo tree -p $pkg failed: $(echo "$out" | head -3)"
    continue
  fi
  for d in $(echo "$out" | awk '{print $1}' | { grep -E '^rootmark-' || true; } | sort -u); do
    [ "$d" = "$pkg" ] && continue
    # shellcheck disable=SC2086
    allowed "$d" $allow || violation "$pkg resolves a dependency on $d (cargo tree); allowed: ${allow:-none}"
  done
done <<<"$CRATES"

# 3a. research: only `rootmark_core` from the kernel side
if [ -d research ]; then
  if grep -Eq '^manifest-path' research/pyproject.toml 2>/dev/null &&
     ! grep -Eq '^manifest-path[[:space:]]*=[[:space:]]*"\.\./bindings/python/Cargo.toml"' research/pyproject.toml; then
    violation "research/pyproject.toml builds an extension other than ../bindings/python"
  fi
  hits="$(grep -RInE \
      -e '^[[:space:]]*(from|import)[[:space:]]+(rootmark_index|rootmark_server|prototype|core|index|server|bindings|clients)([[:space:].]|$)' \
      -e 'sys\.path\.(insert|append)' \
      -e '\.\./(core|index|server|clients|prototype)/' \
      --include='*.py' research 2>/dev/null | grep -v '/\.venv/' | grep -v '^research/data/' || true)"
  [ -z "$hits" ] || violation "research/ reaches past bindings/python:
$hits"
fi

# 3b. clients: no direct import of core, index, server, bindings, research
#     (P1-E/E0: `server` added — a client reaches the server only over the
#     contract); and every HTTP path a client source names (`/v1/<x>`) is an
#     endpoint of docs/CLIENT_CONTRACT.md §1 (`POST /v1/<§2 tool>`, `GET
#     /v1/status`)
if [ -d clients ]; then
  hits="$(grep -RInE \
      -e "(from|require\(|import\()[[:space:]]*['\"](\.\./)+(core|index|server|bindings|research|prototype)(/|['\"])" \
      -e "(from|require\(|import\()[[:space:]]*['\"]@rootmark/(core|index|server)['\"]" \
      --include='*.ts' --include='*.tsx' --include='*.js' --include='*.mjs' --include='*.cjs' \
      clients 2>/dev/null | grep -v '/node_modules/' || true)"
  [ -z "$hits" ] || violation "clients/ imports a kernel package directly:
$hits"
  if [ ! -f docs/CLIENT_CONTRACT.md ]; then
    violation "clients/ exists but docs/CLIENT_CONTRACT.md does not (the client's only dependency)"
  else
    # a row names the endpoint (`attest`) or its HTTP path (`POST /v1/demote`, §2.7; P2-B)
    endpoints="$(sed -n '/^## 2\. /,/^## 3\. /p' docs/CLIENT_CONTRACT.md | grep -E '^\| `(POST /v1/)?[a-z_]+` \|' | sed -E 's#^\| `(POST /v1/)?([a-z_]+)`.*#\2#' | sort -u)"
    used="$(grep -RhoE --include='*.ts' --include='*.tsx' --include='*.js' --include='*.mjs' \
        --exclude-dir=node_modules --exclude=main.js "/v1/[A-Za-z0-9_]+" clients 2>/dev/null | sed 's#^/v1/##' | sort -u || true)"
    for u in $used; do
      if ! echo "$endpoints" | grep -qx "$u"; then
        violation "clients/ names /v1/$u, not an endpoint of docs/CLIENT_CONTRACT.md"
      fi
    done
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "check_deps: FAILED (reverse or forbidden dependencies above)" >&2
  exit 1
fi
echo "check_deps: OK (core ← index ← server ← clients; research → bindings/python)"
