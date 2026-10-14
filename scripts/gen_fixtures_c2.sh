#!/usr/bin/env bash
# Writes spec/fixtures/C2-*/expected.json from the **current** core (TASKS
# B7) with the evaluator `make conformance` uses (core/tests/conformance/
# eval_c2.rs), then rewrites spec/fixtures/MANIFEST.json with the frozen
# v3-final generator (scripts/gen_fixtures.sh), which re-derives every C1
# expected.json and lists every fixture file. On a clean checkout both steps
# must leave the tree byte-identical (`git status` clean; VERIFY V2).
#
#   scripts/gen_fixtures_c2.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
(cd "$here" && MDK_FIXTURES="$here/spec/fixtures" \
  cargo test -p mdkernel-core --release -q --test conformance_c2_gen -- --ignored --nocapture)
bash "$here/scripts/gen_fixtures.sh"
echo "C2 fixtures written from the current core; MANIFEST.json from scripts/gen_fixtures.sh"
