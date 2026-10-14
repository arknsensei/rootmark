SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

# Quality gates (CLAUDE.md, docs/DEV_PROCESS.md §3): test, lint, conformance, verify.
# Python lives only in research/ (its own uv project); TypeScript only in clients/.

UVR := cd research && uv run --frozen
PY := cd research && uv run --frozen python
# v2 code paths (frozen eval, v2 runners) never pass a matcher: run them under
# matcher v2 so they reproduce v2 behaviour exactly (prototype ASSUMPTIONS T21)
PY2 := cd research && MDK_MATCHER=v2 uv run --frozen python
# v3-final reference run (docs/rounds/P1-A/progress/SOURCE.md); A-3 compares against it
V3_REF_RUN := 20261013T182004Z-f82dabd
# P1-A official RUN (reports/rounds/P1-A/; GOAL P1-B B-3 compares against it)
P1A_REF_RUN := ../reports/rounds/P1-A/runs/20261013T221822Z-27d8ae8
# B-10 fuzz seeds (VERIFY V6: two different seeds)
FUZZ_SEEDS := 20261014 7
PHASE ?= test
TS_PKGS := $(wildcard clients/*/package.json)

.PHONY: report report-p1b test-tamper check-frozen setup setup-py build require-run d2 d3 d1 d1k4 d1h datasets freeze-check freeze-check-v3 \
        test-freeze-indep eval-h1s-tune eval-h1s eval-h1r eval-h1s-v3 eval-h1r-v3 eval-h3 eval-h4 \
        eval-h4-v2repro d1ho d1sh d1hs eval-holdout eval-split-grid eval-holdout-split p1a-checks p1b-checks test test-rust test-py test-ts lint lint-rust lint-py lint-ts \
        check-deps test-check-deps conformance regress verify

setup: setup-py build

setup-py:
	@# --inexact: keep the maturin-installed mdkernel_core (research is `package = false`)
	cd research && uv sync --frozen --inexact

# Rust workspace + the mdkernel_core extension into research/.venv
build:
	cargo build --workspace --release
	$(UVR) maturin develop --release --quiet
	@# macOS ld can emit a misaligned LINKEDIT string pool that dyld rejects;
	@# fail here, loudly, instead of at first use (prototype ASSUMPTIONS T5)
	$(PY) -c "import mdkernel_core"

test: test-rust test-py test-ts

test-rust:
	cargo test --workspace --release

test-py: setup-py build
	$(UVR) pytest -q

test-ts:
ifeq ($(strip $(TS_PKGS)),)
	@echo "test-ts: no TypeScript client package (clients/*/package.json) in this round; nothing to run"
else
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npx vitest run); done
endif

lint: lint-rust lint-py lint-ts check-deps test-check-deps check-frozen

lint-rust:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --release -- -D warnings

lint-py: setup-py
	$(UVR) ruff check .
	$(UVR) ruff format --check .

lint-ts:
ifeq ($(strip $(TS_PKGS)),)
	@echo "lint-ts: no TypeScript client package (clients/*/package.json) in this round; nothing to run"
else
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npx tsc --noEmit && npx eslint .); done
endif

# Dependency direction core ← index ← server ← clients; research → bindings/python only (A-6)
check-deps:
	bash scripts/check_deps.sh

# Frozen files unchanged since their freeze commit (research/FROZEN.md)
check-frozen:
	bash scripts/check_frozen.sh

# Deliberate corruptions must fail (TASKS A6, VERIFY V8); RUN_DIR = a complete P1-A RUN
test-tamper:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	bash scripts/test_tamper.sh "$(RUN_DIR)"

# Deliberate reverse dependency in a scratch copy must make check_deps.sh fail
test-check-deps:
	bash scripts/test_check_deps.sh

# Levels 1 and 2 (spec/CONFORMANCE.md): spec/fixtures/C1-* and C2-* against
# the current core; writes reports/conformance.json and fails unless 25/25
# pass (level_claimed 2) and the frozen fixture manifest matches. The fixture root is this tree's, resolved at run
# time (MDK_FIXTURES; P1-A verifier recommendation Ö2).
conformance:
	rm -f reports/conformance.json
	MDK_FIXTURES="$(CURDIR)/spec/fixtures" MDK_CONFORMANCE_REPORT="$(CURDIR)/reports/conformance.json" \
	  cargo test -p mdkernel-core --release --test conformance -- --nocapture
	@test -f reports/conformance.json || { echo "error: reports/conformance.json not written" >&2; exit 1; }
	@jq -e --arg root "$$(cd spec/fixtures && pwd -P)" '.level_claimed == 2 and .passed == .total and .total == 25 and .fixtures_root == $$root and (.fixtures_manifest_hash | type) == "string"' reports/conformance.json >/dev/null \
	  || { echo "error: conformance level 2 not reached (reports/conformance.json)" >&2; exit 1; }
	@echo "conformance: level $$(jq -r .level_claimed reports/conformance.json), $$(jq -r '"\(.passed)/\(.total)"' reports/conformance.json) (reports/conformance.json)"

require-run:
	@if [ -z "$(RUN)" ]; then echo "error: RUN=<run_id> is required (cd research && uv run --frozen python -m eval.runinfo)" >&2; exit 1; fi

# ---- research datasets (git-ignored; rebuilt and verified against committed MANIFESTs)
d2:
	$(PY2) -m eval.datasets.git_history $(ARGS)

# D3: frozen v2 builder under matcher v2 + the v3 log hash (`events_hash_v3`)
d3:
	$(PY) -m eval.d3_v3 $(ARGS)

d1: d3
	$(PY2) -m eval.datasets.synthetic_edits $(ARGS)

d1k4: d3
	$(PY) -m eval.datasets.synthetic_edits_v2 --set d1k4 $(ARGS)

d1h: d3
	$(PY) -m eval.datasets.synthetic_edits_v2 --set d1h $(ARGS)

# P1-A holdout (GOAL A-2) and split-threshold difficulty slice (A-7)
d1ho: d3 d1h
	$(PY) -m eval.datasets.synthetic_edits_v3 --set holdout $(ARGS)

d1sh: d3 d1h
	$(PY) -m eval.datasets.synthetic_edits_v3 --set split_hard $(ARGS)

# P1-B split-weighted holdout slice (GOAL Ö3, B-9); bases from the D2 revision
d1hs: d2 d3 d1h d1ho
	$(PY) -m eval.datasets.synthetic_edits_v4 $(ARGS)

datasets: d2 d3 d1 d1k4 d1h d1ho d1sh d1hs

# ---- freeze checkers (research/FROZEN.md)
freeze-check:
	$(PY2) -m eval.freeze_check

freeze-check-v3:
	$(PY) -m eval.freeze_check_v3

# the checkers must not depend on compiled matcher constants (GOAL P1-A)
test-freeze-indep:
	bash scripts/test_freeze_indep.sh

# ---- prototype phases (frozen runners; outputs in research/reports/runs/<RUN>/)
eval-h1s-tune: require-run
	$(PY2) -m eval.run_h1s --phase tune --run $(RUN)

eval-h1s: require-run
	$(PY2) -m eval.run_h1s --phase test --run $(RUN)

eval-h1r: require-run
	$(PY2) -m eval.run_h1r --run $(RUN)

eval-h1s-v3: require-run
	$(PY) -m eval.run_h1s_v3 --phase $(PHASE) --run $(RUN) $(ARGS)

eval-h1r-v3: require-run
	$(PY) -m eval.run_h1r_v3 --run $(RUN)

eval-h3: require-run
	$(PY) -m eval.run_h3_v3 --run $(RUN)

eval-h4: require-run
	$(PY) -m eval.run_h4_v3 --run $(RUN)

eval-h4-v2repro: require-run
	$(PY) -m eval.run_h4_v2repro --run $(RUN)

# P1-A: holdout at the compiled v3 parameters (A-2) and THETA_SPLIT grid (A-7)
eval-holdout: require-run
	$(PY) -m eval.run_holdout --run $(RUN)

eval-split-grid: require-run
	$(PY) -m eval.run_split_grid --run $(RUN)

# P1-B: holdout-split at the compiled v3 parameters (B-9)
eval-holdout-split: require-run
	$(PY) -m eval.run_holdout_split --run $(RUN)

# A-1 / A-5 / A-6 recorded in the RUN (the round report reads one RUN only)
p1a-checks: require-run
	$(MAKE) conformance
	{ bash scripts/check_deps.sh && bash scripts/test_check_deps.sh; } >research/reports/runs/$(RUN)/check_deps.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/check_deps.log >&2; exit 1; }
	$(PY) -m eval.run_p1a_checks --run $(RUN) --conformance ../reports/conformance.json \
	  --deps-log reports/runs/$(RUN)/check_deps.log

# P1-B governance checks into the RUN (B-1, B-2, B-5, B-6, B-10, scenario for
# B-4): conformance level 2, v3-final compat, policy bounds, fuzz with two
# seeds, the deterministic governance scenario, dependency direction.
p1b-checks: require-run
	$(MAKE) conformance
	cp reports/conformance.json research/reports/runs/$(RUN)/conformance.json
	{ bash scripts/check_deps.sh && bash scripts/test_check_deps.sh; } >research/reports/runs/$(RUN)/check_deps.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/check_deps.log >&2; exit 1; }
	cargo build --release -q -p mdkernel-core --bin mdk
	bash scripts/check_v3_compat.sh research/reports/runs/$(RUN)/v3_compat.json >/dev/null
	bash scripts/test_policy_bounds.sh target/release/mdk research/reports/runs/$(RUN)/policy_bounds.json
	@for s in $(FUZZ_SEEDS); do \
	  MDK_FUZZ_SEED=$$s MDK_FUZZ_OUT="$(CURDIR)/research/reports/runs/$(RUN)/fuzz_$$s.json" \
	    cargo test -p mdkernel-core --release -q --test layers_fuzz >/dev/null || exit 1; done
	rm -rf research/reports/runs/$(RUN)/governance
	bash scripts/governance_scenario.sh target/release/mdk research/reports/runs/$(RUN)/governance
	$(PY) -m eval.run_p1b_checks --run $(RUN) --seeds $(FUZZ_SEEDS)

# P1-A round report from one RUN directory (TASKS A7): writes <RUN_DIR>/FINAL.md
# and reports/rounds/P1-A/FINAL.md
report:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1a --run-dir "$(abspath $(RUN_DIR))"

# P1-B round report from one RUN directory (TASKS B9): <RUN_DIR>/FINAL.md and
# reports/rounds/P1-B/FINAL.md; the RUN must hold pair_compare.json (verify pair)
report-p1b:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1b --run-dir "$(abspath $(RUN_DIR))"

# Quality regression (TASKS A4, GOAL A-3): v2 phases + v3 D1-easy/hard, D2, H3,
# H4, H4-V2 under RUN, then every quality field compared with the v3-final
# reference run (compare_runs --set v3; latency fields excluded, non-zero exit
# on any difference). Results stay in the RUN directory (git-ignored): nothing
# is written into the source tree, so later phases still record a clean tree;
# the round's official RUN is committed under reports/rounds/P1-A/ by A7.
#
# P1-B (TASKS B8, GOAL B-3): plus the P1-A holdout and THETA_SPLIT grid,
# compared field by field with the P1-A official RUN (frozen compare_p1a),
# and the d1hs holdout-split (B-9; new, nothing to compare with).
regress: require-run
	$(MAKE) eval-h1s-tune eval-h1s eval-h1r RUN=$(RUN)
	$(MAKE) eval-h1s-v3 RUN=$(RUN) PHASE=tune
	$(MAKE) eval-h1s-v3 RUN=$(RUN) PHASE=test
	$(MAKE) eval-h1r-v3 eval-h3 eval-h4 eval-h4-v2repro RUN=$(RUN)
	$(PY) -m eval.compare_runs reports/runs/$(V3_REF_RUN) reports/runs/$(RUN) --set v3 \
	  --out reports/runs/$(RUN)/regress_vs_v3final.json
	@echo "regress: RUN=$(RUN) equal to v3-final $(V3_REF_RUN) (research/reports/runs/$(RUN)/regress_vs_v3final.json)"
	$(MAKE) eval-holdout eval-split-grid eval-holdout-split RUN=$(RUN)
	$(PY) -m eval.compare_p1a $(P1A_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1a.json >/dev/null
	@echo "regress: RUN=$(RUN) P1-A set equal to the P1-A official RUN (research/reports/runs/$(RUN)/regress_vs_p1a.json)"

# Clean-clone double run (scripts/verify.sh pair; two temp dirs, compared)
verify:
	bash scripts/verify.sh
