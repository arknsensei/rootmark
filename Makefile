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
PHASE ?= test
TS_PKGS := $(wildcard clients/*/package.json)

.PHONY: test-tamper check-frozen setup setup-py build require-run d2 d3 d1 d1k4 d1h datasets freeze-check freeze-check-v3 \
        test-freeze-indep eval-h1s-tune eval-h1s eval-h1r eval-h1s-v3 eval-h1r-v3 eval-h3 eval-h4 \
        eval-h4-v2repro d1ho d1sh eval-holdout eval-split-grid p1a-checks test test-rust test-py test-ts lint lint-rust lint-py lint-ts \
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

# Level 1 (spec/CONFORMANCE.md): spec/fixtures/C1-* against the current core;
# writes reports/conformance.json and fails unless 17/17 pass and the frozen
# fixture manifest matches.
conformance:
	rm -f reports/conformance.json
	MDK_CONFORMANCE_REPORT="$(CURDIR)/reports/conformance.json" \
	  cargo test -p mdkernel-core --release --test conformance -- --nocapture
	@test -f reports/conformance.json || { echo "error: reports/conformance.json not written" >&2; exit 1; }
	@jq -e '.level_claimed == 1 and .passed == .total and .total == 17' reports/conformance.json >/dev/null \
	  || { echo "error: conformance level 1 not reached (reports/conformance.json)" >&2; exit 1; }
	@echo "conformance: level 1, $$(jq -r '"\(.passed)/\(.total)"' reports/conformance.json) (reports/conformance.json)"

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

datasets: d2 d3 d1 d1k4 d1h d1ho d1sh

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

# A-1 / A-5 / A-6 recorded in the RUN (the round report reads one RUN only)
p1a-checks: require-run
	$(MAKE) conformance
	{ bash scripts/check_deps.sh && bash scripts/test_check_deps.sh; } >research/reports/runs/$(RUN)/check_deps.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/check_deps.log >&2; exit 1; }
	$(PY) -m eval.run_p1a_checks --run $(RUN) --conformance ../reports/conformance.json \
	  --deps-log reports/runs/$(RUN)/check_deps.log

# Quality regression (TASKS A4, GOAL A-3): v2 phases + v3 D1-easy/hard, D2, H3,
# H4, H4-V2 under RUN, then every quality field compared with the v3-final
# reference run (compare_runs --set v3; latency fields excluded, non-zero exit
# on any difference). Results are copied to reports/rounds/P1-A/runs/<RUN>/.
regress: require-run
	$(MAKE) eval-h1s-tune eval-h1s eval-h1r RUN=$(RUN)
	$(MAKE) eval-h1s-v3 RUN=$(RUN) PHASE=tune
	$(MAKE) eval-h1s-v3 RUN=$(RUN) PHASE=test
	$(MAKE) eval-h1r-v3 eval-h3 eval-h4 eval-h4-v2repro RUN=$(RUN)
	$(PY) -m eval.compare_runs reports/runs/$(V3_REF_RUN) reports/runs/$(RUN) --set v3 \
	  --out reports/runs/$(RUN)/regress_vs_v3final.json
	mkdir -p reports/rounds/P1-A/runs/$(RUN)
	cp research/reports/runs/$(RUN)/*.json reports/rounds/P1-A/runs/$(RUN)/
	@echo "regress: RUN=$(RUN) equal to v3-final $(V3_REF_RUN) (reports/rounds/P1-A/runs/$(RUN)/regress_vs_v3final.json)"

# Clean-clone double run (scripts/verify.sh pair; two temp dirs, compared)
verify:
	bash scripts/verify.sh
