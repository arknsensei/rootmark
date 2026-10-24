#!/usr/bin/env bash
# Independent end-to-end reproduction (docs/DEV_PROCESS.md §5,
# docs/rounds/P1-B/VERIFY.md; P1-B TASKS B9; P1-C TASKS C9).
#
# P1-C (re-freeze P1-C/F8): after `make setup` the chain fetches and checks
# the embedder model (`make models`); conformance is level 3 (30 tests);
# after p1b-checks it records `make p1b1-checks` (P1-B.1 set, C-11 input)
# and `make p1c-checks` (C-1..C-13 into p1c_checks.json; the D3
# determinism vault in <workdir>/p1c_work/<RUN>/.rootmark/index/) and runs
# the P1-C deliberate corruptions (`scripts/test_tamper_p1c.sh`); `pair`
# compares A ↔ B with `eval.compare_p1c pair` (the P1-B set plus the P1-B.1
# and P1-C files, the 200 rankings and every index file hash, the
# governance vault bytes) and `compare-p1b` is kept unchanged.
#
# P1-C.1 (re-freeze P1-C.1/F6): after the P1-C corruptions the chain records
# `make p1c1-checks` (C1-1 Python variant, C1-2..C1-6 tests, the CLI
# deliberate tests `scripts/test_p1c1.sh`, C1-8 regress against the P1-C
# official RUN, C1-9 → p1c1_checks.json); `pair` compares A ↔ B with
# `eval.compare_p1c1 pair` (the P1-C pair set plus c1_variant.json and
# p1c1_checks.json, no exclusion); `compare-p1c` is kept unchanged and
# `compare-p1c1` is new.
#
# P1-D (re-freeze P1-D/F4): after the P1-C.1 checks the chain records `make
# p1d-checks` (D-1..D-11: the official MCP Inspector — Node.js and network
# needed —, the MCP e2e, the HTTP/service tests, the watcher latency and
# SIGTERM tests, the agent scene, the P1-D deliberate corruptions, D-9
# against the P1-C.1 official RUN → p1d_checks.json); `pair` compares A ↔ B
# with `eval.compare_p1d pair` (the P1-C.1 pair set plus the P1-D files and
# the P1-D artefacts byte for byte: MCP transcripts, the e2e vault's
# events.jsonl/traces.jsonl/proposals.jsonl and Markdown, the scene, the
# Inspector's tool list); `compare-p1c1` is kept unchanged and `compare-p1d`
# is new.
#
# P1-D.1 (re-freeze P1-D.1/F12): after the P1-D checks the chain records
# `make p1d1-checks` (D1-1..D1-13: the criteria tests, the p1d-final old
# vaults, the contract check, the P1-D.1 deliberate corruptions, D1-12's
# regress against the P1-D official RUN → p1d1_checks.json); `pair` compares
# A ↔ B with `eval.compare_p1d1 pair` (the P1-D pair set plus the P1-D.1
# files and artefacts); `compare-p1d` is kept unchanged and `compare-p1d1`
# is new.
#
# P1-E (re-freeze P1-E/F7): after the P1-D.1 checks the chain records `make
# p1e-checks` (E-1..E-10 and E-12..E-16: the E0 tests, the 11-step scene,
# the plugin e2e against a real `rootmark serve`, the client gates, the local
# release binary and `rootmark conformance`, the P1-E deliberate corruptions, the
# deterministic-field audit, E-9 against the P1-D.1 official RUN →
# p1e_checks.json); `pair` compares A ↔ B with `eval.compare_p1e pair` (the
# P1-D.1 pair set plus the P1-E files and artefacts byte for byte: the
# scene's and the plugin e2e's vault logs, the plugin's access log);
# `compare-p1d1` is kept unchanged and `compare-p1e` is new.
#
# P1-E.1 (re-freeze P1-E.1/F3 summary, F4 chain and pair): after the P1-E checks the chain records `make
# p1e1-checks` (E1-1..E1-9: the status tests on the p1d-final `/2` index, the
# contract check's status step, the deliberate cases, the release package of
# the commit with the real-Obsidian e2e — recorded as skipped without an
# Obsidian binary — and its checklist, E1-6 against the P1-E official RUN,
# the historical FAIL labels → p1e1_checks.json, p1e1/historical.json) and
# ends with a summary of every recorder's FAILs (K-E1.4: historical ones
# shown with their decisions, not counted; the summary never changes the
# exit code — criteria are recorded, not fatal); `pair` compares A ↔ B with
# `eval.compare_p1e1 pair` (the P1-E pair set plus the P1-E.1 files and
# artefacts); `compare-p1e` is kept unchanged and `compare-p1e1` is new.
# P2-0 (re-freeze P2-0/F12): after the P1-E.1 checks the chain records `make
# p2-0-checks` (the window W chosen on the D-B tuning set, committed; D-E,
# D-A, D-C and the D-B test sessions in the real Obsidian; the deliberate
# attestation cases; the regress against the P1-E.1 official RUN →
# p2_0_checks.json); `pair` compares A ↔ B with `eval.compare_p2_0 pair`
# (the P1-E.1 pair set plus the P2-0 files, the D-A and D-C vault logs and
# attestation logs byte for byte, H-P2-0g); `compare-p2-0` is new.
# P2-0.1 (re-freeze P2-0.1/F4, P2-0.1/F5): the chain records `make
# p2-0-1-checks` instead of `make p2-0-checks` (that would run P2-0's D-B
# test set again; P2-0's hypotheses are re-measured with P2-0's W on D-B
# test-2 and the new sets → p2_0_1_checks.json); the FAIL summary is the
# round's classifier (`eval.historical_p2_0_1` against the P2-0 official
# RUN), so its counted FAILs are the round's; `pair` compares A ↔ B with
# `eval.compare_p2_0_1 pair` (the P1-E.1 pair set plus the P2-0.1 files,
# the D-A, D-A2 and D-C vault and attestation logs byte for byte,
# H-P2-0.1i); `compare-p2-0` is kept and `compare-p2-0-1` is new.
#
# P2-0.2 (re-freeze P2-0.2/F1, P2-0.2/F5): `pair` with VERIFY_LOAD=1 runs both
# chains under an artificial CPU load (as many `yes` as logical CPUs); the
# chain records `make p2-0-2-checks` instead of `make p2-0-1-checks` (that
# would run P2-0.1's D-B test set again; P2-0's hypotheses are re-measured
# with P2-0's W on D-B test-3 and the new sets → p2_0_2_checks.json); the
# FAIL summary is the round's classifier (`eval.historical_p2_0_2` against
# the P2-0.1 official RUN); `pair` compares A ↔ B with `eval.compare_p2_0_2
# pair` (the P1-E.1 pair set plus the P2-0.2 files, the D-A, D-A2 and D-C
# vault and attestation logs byte for byte, H-P2-0.2i); `compare-p2-0-1` is
# kept and `compare-p2-0-2` is new.
# P2-A (re-freeze P2-A/F5): the chain records `make p2a-checks` instead of
# `make p2-0-2-checks` (p2a-checks runs P2-0.2's checks again without the
# real-Obsidian sets, K-P2.3, then P2-A's sets → p2a_checks.json); the FAIL
# summary is the round's classifier (`eval.historical_p2a` against the
# P2-0.2 official RUN, with its "migration required" category, K-P2A.2);
# `pair` compares A ↔ B with `eval.compare_p2a pair` (the P2-0.2 pair set
# plus the P2-A files); `compare-p2-0-2` is kept and `compare-p2a` is new.
# P2-0.3 (re-freeze P2-0.3/F1): after `make p2a-checks` the chain records
# `make p2-0-3-checks` (P2-0.2's real-Obsidian sets with P2-0.2's code: D-B
# test-4 with the round's pre-registered seed, D-A′, D-C′ → <RUN>/p2_0_3/;
# p2_0_3_checks.json). Nothing else changes: the steps, the classifier,
# `pair`, `compare_p2a` and VERIFY_LOAD are as in P2-A.
# P2-L (re-freeze P2-L/F6): after `make p2-0-3-checks` the chain records
# `make p2l-checks` (the round's tests by criterion → p2l_checks.json) and
# `make p2l-quality` (D-TR in 12 cells, the English reference with both
# lexical variants, L-8's three vaults → p2l_quality.json, p2l/); the FAIL
# summary is the round's classifier (`eval.historical_p2l` against the
# P2-0.3 official RUN); `pair` compares A ↔ B with `eval.compare_p2l pair`
# (the P2-A pair set plus the P2-L files); `compare-p2a` is kept and
# `compare-p2l` is new.
# P2-P (re-freeze P2-P/F2): after `make p2l-quality` the chain records
# `make p2p-checks` (the round's tests by criterion → p2p_checks.json: P-1's
# three vaults, P-2..P-5 and S1–S7, P-7, the collector scene P-3/P-6); the
# FAIL summary is the round's classifier (`eval.historical_p2p` against the
# P2-L official RUN); `pair` and the fast pair compare with
# `eval.compare_p2p` (the P2-L sets plus p2p_checks.json); `compare-p2l` is
# kept and `compare-p2p` is new.
#
# P2-L (re-freeze P2-L/F3; K-P2.4 (3)): a fast mode for the rounds' own
# quality gate. `FAST=1 scripts/verify.sh` (`make verify FAST=1`) runs two
# fast chains side by side (`chain-fast`: a clean clone, `make setup`,
# models and D3, check_frozen and check_deps, `make conformance` and
# `rootmark conformance`, C-3 quality, `make fast-regress` — the English
# quality reference's 618 values and the conformance report by test id
# against the last official RUN —, then the active round's checks,
# `make $(ROUND_CHECKS)`) and compares them with `eval.compare_p2l fast`;
# it writes the elapsed time beside the comparison. Without FAST the full
# chain below runs, unchanged.
#
#   scripts/verify.sh chain <source-repo> <workdir>
#       clone <source-repo> into <workdir>/repo, `make setup`, rebuild and
#       verify every research dataset against its committed manifest (d1hs
#       included), run both freeze checkers and `scripts/check_frozen.sh`,
#       `make conformance` (level 2), then under one RUN id: `make regress`
#       (v2 phases, v3 D1-easy/hard, D2, H3, H4, H4-V2 compared with the
#       v3-final reference run; P1-A holdout and THETA_SPLIT grid compared
#       with the P1-A official RUN; d1hs holdout-split), `make p1b-checks`
#       (conformance, v3-final compat, policy bounds, governance fuzz with two
#       seeds, the deterministic governance scenario, dependency direction
#       → p1b_checks.json) and the deliberate corruptions
#       (`scripts/test_tamper.sh`: fixtures, holdout data, RUN files, moved
#       target/ B-7, FROZEN.md rows B-8; its log goes into the RUN); prints
#       "RUN=<id>" and "RUN_DIR=<path>".
#   scripts/verify.sh pair <source-repo> <workdir-A> <workdir-B>
#       two independent chains, then `compare` A ↔ B.
#   scripts/verify.sh compare <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       research/eval/compare_p1a.py (P1-A set: v3 + holdout + split_grid;
#       unchanged P1-A semantics) run inside <repo>/research.
#   scripts/verify.sh compare-p1b <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       research/eval/compare_p1b.py (P1-A set + holdout_split +
#       p1b_checks + governance summary, and the governance vault's
#       events.jsonl, traces.jsonl and checkpoints/ byte for byte) run inside
#       <repo>/research; exits non-zero when any quality field or artefact
#       differs (B-4) or a file/field is missing.
#   scripts/verify.sh            (what `make verify` runs)
#       pair mode with <source-repo> = this repository and two fresh temp dirs.
#
# Nothing here refers to the main working directory except as the clone
# source; every path is derived from the arguments. Any failing step fails
# the whole run (set -e; each chain is its own process).
set -euo pipefail

log() { printf '[verify %s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }

chain() {
  local src="$1" work="$2"
  # stdout carries only the RUN=/RUN_DIR= result lines; everything else → stderr
  exec 3>&1 1>&2
  mkdir -p "$work"
  work="$(cd "$work" && pwd)"
  if [ -e "$work/repo" ]; then
    echo "error: $work/repo already exists; use an empty directory" >&2
    exit 1
  fi
  local t0=$SECONDS
  if [ -n "$(git -C "$src" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    log "warning: $src has uncommitted changes; only its committed HEAD is verified"
  fi
  log "clone $src -> $work/repo"
  git clone --quiet "$src" "$work/repo"
  cd "$work/repo"
  log "commit $(git rev-parse HEAD)"
  log "setup"
  make setup >"$work/setup.log" 2>&1 || { tail -40 "$work/setup.log" >&2; exit 1; }
  log "models (fetch + manifest check)"
  make models >"$work/models.log" 2>&1 || { tail -20 "$work/models.log" >&2; exit 1; }
  log "datasets (d2 d3 d1 d1k4 d1h d1ho d1sh d1hs, manifests verified)"
  make datasets
  log "freeze-check (v2), freeze-check-v3, check_frozen"
  make freeze-check
  make freeze-check-v3
  bash scripts/check_frozen.sh
  log "conformance (level 3)"
  make conformance
  local run
  run="$(cd research && uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  log "holdout (P1-A A-2) and THETA_SPLIT grid (A-7)"
  make eval-holdout RUN="$run"
  make eval-split-grid RUN="$run"
  log "regress (v2 + v3 phases vs v3-final; holdout + split grid vs P1-A; holdout-split)"
  make regress RUN="$run"
  log "A-1 / A-5 / A-6 into the RUN (level-1 view)"
  make p1a-checks RUN="$run"
  log "governance: conformance, v3-final compat, policy bounds, fuzz, scenario (p1b_checks)"
  make p1b-checks RUN="$run"
  log "P1-B.1 checks (B1-1..B1-6 recorded again; C-11 input)"
  make p1b1-checks RUN="$run"
  log "P1-C checks (C-1..C-13; D3 determinism vault under $work/p1c_work)"
  make p1c-checks RUN="$run" P1C_WORK="$work/p1c_work"
  log "P1-C deliberate corruptions (model manifest, embedder_id, fixtures)"
  bash scripts/test_tamper_p1c.sh >"research/reports/runs/$run/tamper_p1c.log" 2>&1 \
    || { cat "research/reports/runs/$run/tamper_p1c.log" >&2; exit 1; }
  tail -1 "research/reports/runs/$run/tamper_p1c.log" >&2
  log "P1-C.1 checks (C1-1..C1-9; CLI deliberate tests: lock file, empty query, manifest fields)"
  make p1c1-checks RUN="$run"
  tail -1 "research/reports/runs/$run/p1c1_cli.log" >&2
  log "P1-D checks (D-1..D-11: Inspector, MCP e2e, HTTP/service, watch, scene, tamper; D-9 vs P1-C.1)"
  make p1d-checks RUN="$run"
  log "P1-D.1 checks (D1-1..D1-13: criteria tests, p1d-final vaults, contract, tamper; D1-12 vs P1-D)"
  make p1d1-checks RUN="$run"
  log "P1-E checks (E-1..E-10, E-12..E-16: E0 tests, scene, plugin e2e, client, local release, tamper, field audit; E-9 vs P1-D.1)"
  make p1e-checks RUN="$run"
  log "P1-E.1 checks (E1-1..E1-9: status on the /2 index, contract status step, tamper, release package + real Obsidian e2e, E1-6 vs P1-E)"
  make p1e1-checks RUN="$run"
  # P2-0.1 (P2-0.1/F5): P2-0's own checks would run P2-0's D-B test set again
  # (GOAL P2-0.1: never read); the round's checks re-measure P2-0's
  # hypotheses with P2-0's W and thresholds on D-B test-2 and the new sets
  # P2-0.2 (P2-0.2/F5): P2-0.1's own checks would run P2-0.1's D-B test set
  # again (GOAL P2-0.2: never read); the round's checks re-measure P2-0's
  # hypotheses with P2-0's W and thresholds on D-B test-3 and the new sets
  # P2-A (P2-A/F5): P2-0.2's checks without the real-Obsidian sets, then
  # P2-A's sets (migration of six old vaults, deliberate cases, tests),
  # regress vs P2-0.2 and the round classifier
  log "P2-A checks (P2-0.2 sets without real Obsidian; migrate 6 old vaults, /2 fixture, tamper, tests; regress vs P2-0.2; round classifier)"
  make p2a-checks RUN="$run"
  log "P2-0.3 checks (real Obsidian: D-B test-4 with the new seed, D-A′, D-C′; p2_0_3_checks)"
  make p2-0-3-checks RUN="$run"
  log "P2-L checks (round tests by criterion; D-TR 12 cells, English reference, L-8 three vaults)"
  make p2l-checks RUN="$run"
  make p2l-quality RUN="$run"
  log "P2-P checks (round tests by criterion: P-1 three vaults, rules, S1–S7, channel label, collector scene)"
  make p2p-checks RUN="$run"
  log "deliberate corruptions (B-7, B-8 and the P1-A cases)"
  bash scripts/test_tamper.sh "research/reports/runs/$run" >"research/reports/runs/$run/tamper.log" 2>&1 \
    || { cat "research/reports/runs/$run/tamper.log" >&2; exit 1; }
  tail -1 "research/reports/runs/$run/tamper.log" >&2
  # every phase must have run on the clean committed tree (nothing written into it)
  local dirty
  dirty="$(jq -r '.phases | to_entries[] | select(.value.worktree_clean != true) | .key' \
    "research/reports/runs/$run/manifest.json")"
  if [ -n "$dirty" ]; then
    echo "error: phases ran on a dirty work tree: $(echo $dirty)" >&2
    git status --porcelain --untracked-files=all >&2
    exit 1
  fi
  # K-E1.4 (re-freeze P1-E.1/F3): every recorder's FAILs; historical ones
  # are shown with their decisions and not counted; the summary never
  # changes the exit code (criteria are recorded, not fatal). Since P2-0.1
  # (P2-0.1/F4; P2-0 verification recommendation 3) the summary is the
  # round's own classifier against the previous official RUN, so its
  # counted FAILs are the round's (eval.historical_p2_0_1; since P2-0.2,
  # P2-0.2/F5, eval.historical_p2_0_2 against the P2-0.1 official RUN)
  log "FAIL summary (K-E1.4, round classifier eval.historical_p2p)"
  (cd research && uv run --frozen python -m eval.historical_p2p --run-dir "reports/runs/$run" --summary || true) >&2
  log "chain finished in $((SECONDS - t0)) s"
  echo "RUN=$run" >&3
  echo "RUN_DIR=$work/repo/research/reports/runs/$run" >&3
}

# P2-L/F3: the fast chain (K-P2.4 (3)); the same clone, setup and RUN as
# `chain`, then only what the fast gate checks
chain_fast() {
  local src="$1" work="$2"
  exec 3>&1 1>&2
  mkdir -p "$work"
  work="$(cd "$work" && pwd)"
  if [ -e "$work/repo" ]; then
    echo "error: $work/repo already exists; use an empty directory" >&2
    exit 1
  fi
  local t0=$SECONDS
  if [ -n "$(git -C "$src" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    log "warning: $src has uncommitted changes; only its committed HEAD is verified"
  fi
  log "fast: clone $src -> $work/repo"
  git clone --quiet "$src" "$work/repo"
  cd "$work/repo"
  log "fast: commit $(git rev-parse HEAD)"
  make setup >"$work/setup.log" 2>&1 || { tail -40 "$work/setup.log" >&2; exit 1; }
  make models >"$work/models.log" 2>&1 || { tail -20 "$work/models.log" >&2; exit 1; }
  log "fast: D3 (manifest verified)"
  make d3 >"$work/d3.log" 2>&1 || { tail -20 "$work/d3.log" >&2; exit 1; }
  log "fast: check_frozen, check_deps"
  bash scripts/check_frozen.sh
  bash scripts/check_deps.sh
  local run
  run="$(cd research && uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  local fd="research/reports/runs/$run/fast"
  mkdir -p "$fd"
  log "fast: conformance (make conformance, rootmark conformance)"
  make conformance
  cp reports/conformance.json "$fd/conformance.json"
  ROOTMARK_MODELS="$work/repo/models" target/release/rootmark conformance >"$fd/conformance_rootmark.json"
  local erc=0
  bash scripts/conformance_equiv.sh reports/conformance.json "$fd/conformance_rootmark.json" >"$fd/conformance_equiv.log" 2>&1 || erc=$?
  jq -n --argjson e "$erc" '{exit: $e}' >"$fd/conformance_equiv.json"
  log "fast: C-3 quality (D3 + D4 test split)"
  make c3-quality RUN="$run" >"$fd/c3_quality.log" 2>&1 || { tail -20 "$fd/c3_quality.log" >&2; exit 1; }
  log "fast: regress (quality reference, conformance vs the last official RUN)"
  make fast-regress RUN="$run"
  log "fast: round checks (make $(make -s print-round-checks))"
  make round-checks RUN="$run"
  local dirty
  dirty="$(jq -r '.phases | to_entries[] | select(.value.worktree_clean != true) | .key' \
    "research/reports/runs/$run/manifest.json")"
  if [ -n "$dirty" ] || [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "error: the fast chain ran on a dirty work tree: $(echo $dirty)" >&2
    git status --porcelain --untracked-files=all >&2
    exit 1
  fi
  log "fast chain finished in $((SECONDS - t0)) s"
  echo "RUN=$run" >&3
  echo "RUN_DIR=$work/repo/research/reports/runs/$run" >&3
}

# P2-L/F3: two fast chains side by side, then `compare_p2l fast`
pair_fast() {
  local src="$1" a="$2" b="$3"
  local self res_a res_b dir_a dir_b pa pb rca=0 rcb=0 t0=$SECONDS
  self="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  res_a="$(mktemp -t rootmark-verify-fa.XXXXXX)"
  res_b="$(mktemp -t rootmark-verify-fb.XXXXXX)"
  bash "$self" chain-fast "$src" "$a" >"$res_a" 2>"$res_a.log" & pa=$!
  bash "$self" chain-fast "$src" "$b" >"$res_b" 2>"$res_b.log" & pb=$!
  wait "$pa" || rca=$?
  wait "$pb" || rcb=$?
  sed 's/^/[A] /' "$res_a.log" >&2
  sed 's/^/[B] /' "$res_b.log" >&2
  if [ "$rca" -ne 0 ] || [ "$rcb" -ne 0 ]; then
    echo "error: a fast chain failed (A exit $rca, B exit $rcb)" >&2
    exit 1
  fi
  dir_a="$(sed -n 's/^RUN_DIR=//p' "$res_a")"
  dir_b="$(sed -n 's/^RUN_DIR=//p' "$res_b")"
  for d in "$dir_a" "$dir_b"; do
    if [ -z "$d" ] || [ ! -f "$d/manifest.json" ]; then
      echo "error: chain produced no run directory ($d)" >&2
      exit 1
    fi
  done
  cat "$res_a" "$res_b"
  local repo rc=0
  repo="$(cd "$a" && pwd)/repo"
  log "compare (P2-P fast) $dir_a $dir_b"
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2p fast "$dir_a" "$dir_b" \
    --out "$dir_a/fast_pair_compare.json") >/dev/null || rc=$?
  jq -n --argjson s "$((SECONDS - t0))" '{elapsed_s: $s}' >"$dir_a/fast_pair_time.json"
  if [ "$rc" -ne 0 ]; then
    echo "error: fast comparison failed (compare_p2p fast exit $rc; $dir_a/fast_pair_compare.json)" >&2
    exit "$rc"
  fi
  log "fast pair equal ($(jq -r .n_fields "$dir_a/fast_pair_compare.json") fields) in $((SECONDS - t0)) s"
}

pair() {
  local src="$1" a="$2" b="$3"
  local self res_a res_b dir_a dir_b
  self="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  res_a="$(mktemp -t rootmark-verify-a.XXXXXX)"
  res_b="$(mktemp -t rootmark-verify-b.XXXXXX)"
  # P2-0.2 (P2-0.2/F1, TASKS X2): VERIFY_LOAD=1 runs both chains under an
  # artificial CPU load — as many `yes >/dev/null` as logical CPUs (`nproc`;
  # macOS `sysctl -n hw.ncpu`), stopped when the chains end; the comparison
  # is the same
  load_pids=()
  if [ "${VERIFY_LOAD:-}" = "1" ]; then
    local n
    n="$(nproc 2>/dev/null || sysctl -n hw.ncpu)"
    log "VERIFY_LOAD=1: $n x yes >/dev/null for the two chains"
    for _ in $(seq 1 "$n"); do yes >/dev/null & load_pids+=("$!"); done
    trap 'kill "${load_pids[@]}" 2>/dev/null || true' EXIT
  fi
  # each chain is its own process: a failing step fails the whole pair
  # (errexit is not inherited inside $(...) on bash 3.2)
  bash "$self" chain "$src" "$a" >"$res_a"
  bash "$self" chain "$src" "$b" >"$res_b"
  if [ "${#load_pids[@]}" -gt 0 ]; then
    kill "${load_pids[@]}" 2>/dev/null || true
    wait "${load_pids[@]}" 2>/dev/null || true
    # the PIDs are reaped: no exit trap may signal them again (P2-0.2/F5)
    trap - EXIT
    log "VERIFY_LOAD=1: load stopped"
  fi
  dir_a="$(sed -n 's/^RUN_DIR=//p' "$res_a")"
  dir_b="$(sed -n 's/^RUN_DIR=//p' "$res_b")"
  for d in "$dir_a" "$dir_b"; do
    if [ -z "$d" ] || [ ! -f "$d/manifest.json" ]; then
      echo "error: chain produced no run directory ($d)" >&2
      exit 1
    fi
  done
  cat "$res_a" "$res_b"
  compare_p2p "$(cd "$a" && pwd)/repo" "$dir_a" "$dir_b" "$dir_a/pair_compare.json"
}

compare_p2p() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-P) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2p pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2p pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P2-A pair set, P2-L and P2-P results equal; D-A, D-A2 and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare_p2l() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-L) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2l pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2l pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P2-A pair set and P2-L results equal; D-A, D-A2 and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare_p2a() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-A) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2a pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2a pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P2-0.2 pair set and P2-A results equal; D-A, D-A2 and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1a "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1a exit $rc)" >&2
    exit "$rc"
  fi
  log "max relative difference of quality fields = 0"
}

compare_p1c() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-C) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1c pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1c pair exit $rc)" >&2
    exit "$rc"
  fi
  log "quality fields, rankings and index file hashes equal; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

compare_p1c1() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-C.1) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1c1 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1c1 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "quality fields, rankings, index file hashes, P1-C.1 checks equal; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

compare_p1d() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-D) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1d pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1d pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-C.1 set and P1-D results equal; MCP transcripts, e2e vault, scene, Inspector list byte-equal"
}

compare_p1d1() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-D.1) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1d1 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1d1 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-D set and P1-D.1 results equal; P1-D and P1-D.1 artefacts byte-equal"
}

compare_p1e() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-E) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1e pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1e pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-D.1 set and P1-E results equal; scene and plugin e2e vault logs, access log byte-equal"
}

compare_p1e1() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-E.1) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1e1 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1e1 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-E set and P1-E.1 results equal; real-Obsidian e2e record and access log byte-equal"
}

compare_p2_0() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-0) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2_0 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2_0 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-E.1 set and P2-0 results equal; D-A and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare_p2_0_1() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-0.1) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2_0_1 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2_0_1 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-E.1 set and P2-0.1 results equal; D-A, D-A2 and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare_p2_0_2() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P2-0.2) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p2_0_2 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p2_0_2 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-E.1 set and P2-0.2 results equal; D-A, D-A2 and D-C events.jsonl and attestations.jsonl byte-equal"
}

compare_p1b() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1b "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1b exit $rc)" >&2
    exit "$rc"
  fi
  log "max relative difference of quality fields = 0; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

case "${1:-}" in
  chain) chain "${2:?source repo}" "${3:?workdir}" ;;
  chain-fast) chain_fast "${2:?source repo}" "${3:?workdir}" ;;
  pair-fast) pair_fast "${2:?source repo}" "${3:?workdir A}" "${4:?workdir B}" ;;
  pair) pair "${2:?source repo}" "${3:?workdir A}" "${4:?workdir B}" ;;
  compare) compare "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1b) compare_p1b "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1c) compare_p1c "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1c1) compare_p1c1 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1d) compare_p1d "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1d1) compare_p1d1 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1e) compare_p1e "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1e1) compare_p1e1 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2-0) compare_p2_0 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2-0-1) compare_p2_0_1 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2-0-2) compare_p2_0_2 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2a) compare_p2a "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2l) compare_p2l "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p2p) compare_p2p "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  "")
    src="$(git rev-parse --show-toplevel)"
    base="$(mktemp -d -t rootmark-verify.XXXXXX)"
    log "workdirs under $base"
    if [ "${FAST:-}" = "1" ]; then
      pair_fast "$src" "$base/A" "$base/B"
    else
      pair "$src" "$base/A" "$base/B"
    fi
    ;;
  *)
    echo "usage: [FAST=1] $0 [chain|chain-fast <src> <dir> | pair|pair-fast <src> <dirA> <dirB> | compare|compare-p1b|compare-p1c|compare-p1c1|compare-p1d|compare-p1d1|compare-p1e|compare-p1e1|compare-p2-0|compare-p2-0-1|compare-p2-0-2|compare-p2a|compare-p2l|compare-p2p <repo> <runA> <runB> [out]]" >&2
    exit 2
    ;;
esac
