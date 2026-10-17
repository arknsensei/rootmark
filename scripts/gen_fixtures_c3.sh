#!/usr/bin/env bash
# Writes spec/fixtures/C3-*/expected.json from the **current** index (TASKS
# C7) with the evaluator `make conformance` uses (index/tests/conformance/
# eval_c3.rs), then rewrites spec/fixtures/MANIFEST.json with the frozen
# v3-final generator (scripts/gen_fixtures.sh), which re-derives every C1
# expected.json and lists every fixture file. Needs the model files (`make
# models`). On a clean checkout both steps must leave the tree byte-identical
# (`git status` clean; VERIFY V3).
#
#   scripts/gen_fixtures_c3.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
(cd "$here" && ROOTMARK_FIXTURES="$here/spec/fixtures" \
  cargo test -p rootmark-index --release -q --test conformance_c3_gen -- --ignored --nocapture)
bash "$here/scripts/gen_fixtures.sh"
echo "C3 fixtures written from the current index; MANIFEST.json from scripts/gen_fixtures.sh"
