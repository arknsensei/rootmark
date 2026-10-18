SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

# Quality gates (CLAUDE.md, docs/DEV_PROCESS.md §3): test, lint, conformance, verify.
# Python lives only in research/ (its own uv project); TypeScript only in clients/.

UVR := cd research && uv run --frozen
PY := cd research && uv run --frozen python
# v2 code paths (frozen eval, v2 runners) never pass a matcher: run them under
# matcher v2 so they reproduce v2 behaviour exactly (prototype ASSUMPTIONS T21)
PY2 := cd research && ROOTMARK_MATCHER=v2 uv run --frozen python
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
        eval-h4-v2repro d1ho d1sh d1hs eval-holdout eval-split-grid eval-holdout-split p1a-checks p1b-checks p1b1-checks report-p1b1 test test-rust test-py test-ts lint lint-rust lint-py lint-ts \
        check-deps test-check-deps check-compare-fields check-client-types conformance regress verify models c3-quality p1c-checks report-p1c \
        c1-variant p1c1-checks report-p1c1 p1d-checks report-p1d p1d1-checks report-p1d1 p1e-checks report-p1e p1e1-checks report-p1e1 \
        p2-0-harness p2-0-tune p2-0-test

setup: setup-py build

setup-py:
	@# --inexact: keep the maturin-installed rootmark_core (research is `package = false`)
	cd research && uv sync --frozen --inexact

# Rust workspace + the rootmark_core extension into research/.venv
build:
	cargo build --workspace --release
	$(UVR) maturin develop --release --quiet
	@# macOS ld can emit a misaligned LINKEDIT string pool that dyld rejects;
	@# fail here, loudly, instead of at first use (prototype ASSUMPTIONS T5)
	$(PY) -c "import rootmark_core"

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

lint: lint-rust lint-py lint-ts check-deps test-check-deps check-frozen check-compare-fields check-client-types

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

# K-E0.2 (P1-E/E0): research/eval/DETERMINISTIC_FIELDS.md matches every
# run comparator's compared files and excluded fields, both ways
check-compare-fields: setup-py
	bash scripts/check_compare_fields.sh >/dev/null

# P1-E/E1: clients/obsidian/src/contract.gen.ts matches the live tools/list
# and docs/CLIENT_CONTRACT.md (regenerate: scripts/gen_client_types.sh)
check-client-types:
	bash scripts/gen_client_types.sh --check

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

# Levels 1, 2 and 3 (spec/CONFORMANCE.md): spec/fixtures/C1-*, C2-* against
# the current core (frozen core/tests/conformance.rs writes
# reports/conformance.json: 27 entries since P1-D.1 (C2-10), level_claimed 2; C2-09 since
# P1-C/C0, K-C0.2), then spec/fixtures/C3-* against the current index
# (index/tests/conformance_c3.rs extends the report: 31 entries since
# P1-C.1 (C3-05, K-C1.3); 34 since P1-D.1 (C3-06, C3-07, K-D1.1, K-D1.8; C2-10,
# K-D1.2; spec 1.0.3), level_claimed 3; P1-C/C7). Fails unless 34/34 pass
# (level_claimed 3) and
# the frozen fixture manifest matches. The fixture root is this tree's,
# resolved at run time (ROOTMARK_FIXTURES; P1-A verifier recommendation Ö2).
# Level 3 needs the model files (`make models`).
conformance:
	rm -f reports/conformance.json
	ROOTMARK_FIXTURES="$(CURDIR)/spec/fixtures" ROOTMARK_CONFORMANCE_REPORT="$(CURDIR)/reports/conformance.json" \
	  cargo test -p rootmark-core --release --test conformance -- --nocapture
	@test -f reports/conformance.json || { echo "error: reports/conformance.json not written" >&2; exit 1; }
	ROOTMARK_FIXTURES="$(CURDIR)/spec/fixtures" ROOTMARK_CONFORMANCE_REPORT="$(CURDIR)/reports/conformance.json" \
	  cargo test -p rootmark-index --release --test conformance_c3 -- --nocapture
	@jq -e --arg root "$$(cd spec/fixtures && pwd -P)" '.level_claimed == 3 and .passed == .total and .total == 37 and .fixtures_root == $$root and (.fixtures_manifest_hash | type) == "string" and ([.tests[] | select(.level == 3)] | length) == 8' reports/conformance.json >/dev/null \
	  || { echo "error: conformance level 3 not reached (reports/conformance.json)" >&2; exit 1; }
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

research/reports/runs/%/holdout.json:
	$(MAKE) eval-holdout RUN=$*

research/reports/runs/%/split_grid.json:
	$(MAKE) eval-split-grid RUN=$*

research/reports/runs/%/holdout_split.json:
	$(MAKE) eval-holdout-split RUN=$*

# P1-B: holdout-split at the compiled v3 parameters (B-9)
eval-holdout-split: require-run
	$(PY) -m eval.run_holdout_split --run $(RUN)

# A-1 / A-5 / A-6 recorded in the RUN (the round report reads one RUN only)
# Since P1-B the conformance report holds levels 1 and 2; A-1 is level 1, so
# the frozen recorder reads the level-1 view of it (same 17 entries).
p1a-checks: require-run
	$(MAKE) conformance
	jq '.tests |= map(select(.level == 1)) | .total = 17 | .passed = ([.tests[] | select(.result == "pass")] | length)' \
	  reports/conformance.json >reports/conformance_l1.json
	{ bash scripts/check_deps.sh && bash scripts/test_check_deps.sh; } >research/reports/runs/$(RUN)/check_deps.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/check_deps.log >&2; exit 1; }
	$(PY) -m eval.run_p1a_checks --run $(RUN) --conformance ../reports/conformance_l1.json \
	  --deps-log reports/runs/$(RUN)/check_deps.log

# P1-B governance checks into the RUN (B-1, B-2, B-5, B-6, B-10, scenario for
# B-4): conformance level 2, v3-final compat, policy bounds, fuzz with two
# seeds, the deterministic governance scenario, dependency direction.
# P1-B.1: v3_compat_mixed.json — mixed-author v3-final vaults (B1-1).
p1b-checks: require-run
	$(MAKE) conformance
	@# since P1-C the report holds levels 1–3 and C2-09 (K-C0.2); the frozen
	@# P1-B/P1-B.1 recorders read the P1-B view of it (C1-01..17, C2-01..08:
	@# 25 entries, level 2 iff all 25 pass with an intact manifest), as
	@# p1a-checks reads the level-1 view; the full report is kept beside it
	@# (P1-D.1: C2-10 is outside the P1-B view too)
	cp reports/conformance.json research/reports/runs/$(RUN)/conformance_full.json
	jq '.tests |= map(select(.level <= 2 and .id != "C2-09" and .id != "C2-10")) | .total = 25 | .passed = ([.tests[] | select(.result == "pass")] | length) | .level_claimed = (if .passed == 25 and .fixtures_manifest == "ok" then 2 else 0 end)' \
	  reports/conformance.json >research/reports/runs/$(RUN)/conformance.json
	{ bash scripts/check_deps.sh && bash scripts/test_check_deps.sh; } >research/reports/runs/$(RUN)/check_deps.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/check_deps.log >&2; exit 1; }
	cargo build --release -q -p rootmark-server --bin rootmark
	bash scripts/check_v3_compat.sh research/reports/runs/$(RUN)/v3_compat.json \
	  research/reports/runs/$(RUN)/v3_compat_mixed.json >/dev/null
	bash scripts/test_policy_bounds.sh target/release/rootmark research/reports/runs/$(RUN)/policy_bounds.json
	@for s in $(FUZZ_SEEDS); do \
	  ROOTMARK_FUZZ_SEED=$$s ROOTMARK_FUZZ_OUT="$(CURDIR)/research/reports/runs/$(RUN)/fuzz_$$s.json" \
	    cargo test -p rootmark-core --release -q --test layers_fuzz >/dev/null || exit 1; done
	rm -rf research/reports/runs/$(RUN)/governance
	bash scripts/governance_scenario.sh target/release/rootmark research/reports/runs/$(RUN)/governance
	$(PY) -m eval.run_p1b_checks --run $(RUN) --seeds $(FUZZ_SEEDS)

# P1-B.1 checks into the RUN (GOAL B1-1..B1-6): the compat and trace_id tests
# (log), the P1-B official RUN comparison (B1-6), then p1b1_checks.json.
# Needs a RUN that `make p1b-checks` completed (conformance, v3_compat_mixed).
P1B_REF_RUN := ../reports/rounds/P1-B/runs/20261014T135400Z-7735a07
p1b1-checks: require-run
	cargo test -p rootmark-core --release --test compat_v3final --test trace_id_v101 -- --test-threads=1 \
	  >research/reports/runs/$(RUN)/p1b1_tests.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/p1b1_tests.log >&2; exit 1; }
	$(PY) -m eval.compare_p1b1 $(P1B_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1b.json >/dev/null
	$(PY) -m eval.run_p1b1_checks --run $(RUN)

# P1-B.1 round report from one RUN directory (TASKS B1.4): <RUN_DIR>/FINAL.md
# and reports/rounds/P1-B.1/FINAL.md; the RUN must hold pair_compare.json
report-p1b1:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1b1 --run-dir "$(abspath $(RUN_DIR))"

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
	@# file targets: outputs a chain already produced are not recomputed
	$(MAKE) research/reports/runs/$(RUN)/holdout.json research/reports/runs/$(RUN)/split_grid.json \
	  research/reports/runs/$(RUN)/holdout_split.json RUN=$(RUN)
	$(PY) -m eval.compare_p1a $(P1A_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1a.json >/dev/null
	@echo "regress: RUN=$(RUN) P1-A set equal to the P1-A official RUN (research/reports/runs/$(RUN)/regress_vs_p1a.json)"

# Embedder model files (P1-C/C0): fetched from HuggingFace at the pinned
# revision (scripts/fetch_model.sh) and checked against models/MANIFEST.json
# (size, blake3) by the Rust verifier; any mismatch fails.
models:
	bash scripts/fetch_model.sh
	cargo run -q --release -p rootmark-index --bin rootmark-index -- models verify

# P1-C C-3 (TASKS C8): D3 copy → checkpoint → index build → D4 test split,
# hybrid and flat (structural, k 10) vs the v2-final hybrid cells; writes
# c3_quality.json into the RUN (needs `make models`, `make d3`).
c3-quality: require-run
	$(PY) -m eval.run_c3_quality --run $(RUN)

# P1-C criteria into the RUN (TASKS C9; GOAL C-1..C-13): model manifest,
# conformance levels 1–3, the C-13 compat tests (log), the C-7 property,
# C-10 embedding and C-8 latency tests (JSON; C-8 single-threaded), C-3
# (c3-quality), C-11 (compare_p1c regress against the P1-B.1 official RUN;
# needs p1b-checks and p1b1-checks in the same RUN), then run_p1c_checks
# (C-4, C-5, C-6 inputs, C-9 on a D3 copy under P1C_WORK) → p1c_checks.json.
P1B1_REF_RUN := ../reports/rounds/P1-B.1/runs/20261014T212243Z-226b511
P1C_WORK ?= $(CURDIR)/research/reports/p1c_work
p1c-checks: require-run
	cargo run -q --release -p rootmark-index --bin rootmark-index -- models verify >research/reports/runs/$(RUN)/models_verify.json
	$(MAKE) conformance
	@# since P1-C.1 the report holds C3-05 (K-C1.3, 31 entries, spec 1.0.2);
	@# the frozen P1-C recorder reads the P1-C view of it (C1, C2, C3-01..04:
	@# 30 entries, level 3 iff all 30 pass with an intact manifest; K-C1.6),
	@# as p1b-checks reads the P1-B view; the full report is kept beside it.
	@# P1-D.1: the report holds C2-10, C3-06, C3-07 too (34 entries, spec
	@# 1.0.3); conformance_p1c1.json is the P1-C.1 view (31 entries, level 3
	@# iff all 31 pass with an intact manifest) and conformance_p1d1.json the
	@# full report
	cp reports/conformance.json research/reports/runs/$(RUN)/conformance_p1d1.json
	jq '.tests |= map(select(.id != "C2-10" and .id != "C3-06" and .id != "C3-07")) | .total = 31 | .passed = ([.tests[] | select(.result == "pass")] | length) | .level_claimed = (if .passed == 31 and .fixtures_manifest == "ok" then 3 else .level_claimed end)' \
	  reports/conformance.json >research/reports/runs/$(RUN)/conformance_p1c1.json
	jq '.tests |= map(select(.id != "C3-05" and .id != "C2-10" and .id != "C3-06" and .id != "C3-07")) | .total = 30 | .passed = ([.tests[] | select(.result == "pass")] | length) | .level_claimed = (if .passed == 30 and .fixtures_manifest == "ok" then 3 else .level_claimed end)' \
	  reports/conformance.json >research/reports/runs/$(RUN)/conformance_full.json
	cargo test -p rootmark-core --release --test compat_v3final kc01 -- --test-threads=1 \
	  >research/reports/runs/$(RUN)/c13_tests.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/c13_tests.log >&2; exit 1; }
	ROOTMARK_C7_OUT="$(CURDIR)/research/reports/runs/$(RUN)/c7_property.json" \
	  cargo test -q -p rootmark-index --release --test governance c7_property -- --test-threads=1
	ROOTMARK_C10_OUT="$(CURDIR)/research/reports/runs/$(RUN)/c10_embedding.json" \
	  cargo test -q -p rootmark-index --release --test embed c10 -- --test-threads=1
	ROOTMARK_C8_OUT="$(CURDIR)/research/reports/runs/$(RUN)/c8_latency.json" \
	  cargo test -q -p rootmark-index --release --test hot c8 -- --test-threads=1
	$(MAKE) c3-quality RUN=$(RUN)
	@# exit 3 (a difference) is recorded, not fatal, since P1-C.1: the spec
	@# version 1.0.2 differs from the frozen B1-3 record by decision (K-C1.6);
	@# C1-8 (compare_p1c1 regress) decides; a missing file/field (1) fails
	rc=0; $(PY) -m eval.compare_p1c regress $(P1B1_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1b1.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1c_checks --run $(RUN) --work "$(P1C_WORK)/$(RUN)"

# P1-C round report from one RUN directory (TASKS C9): <RUN_DIR>/FINAL.md and
# reports/rounds/P1-C/FINAL.md; the RUN must hold pair_compare.json
report-p1c:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1c --run-dir "$(abspath $(RUN_DIR))"

# P1-C.1 C1-1 (K-C1.2): the prototype pipeline on D3 with the Lucene/tantivy
# BM25 variant (eval.bm25_lucene) vs the Rust cells of the P1-C official RUN;
# writes c1_variant.json into the RUN (needs `make d3`; the prototype index
# cache data/d3_index/structural is built if missing).
c1-variant: require-run
	$(PY) -m eval.run_c1_variant --run $(RUN)

# P1-C.1 criteria into the RUN (GOAL C1-1..C1-9): C1-1 (c1-variant), the
# C1-2..C1-6 Rust tests (index/tests/p1c1.rs → p1c1/*.json), the CLI
# deliberate tests (scripts/test_p1c1.sh → p1c1_cli.json), C1-8 regress
# against the P1-C official RUN (eval.compare_p1c1, K-C1.6 exclusions;
# exit 3 recorded, not fatal), then run_p1c1_checks → p1c1_checks.json (C1-7
# from conformance_p1c1.json, C1-9 against data/quality_ref_rust.json).
# Needs a RUN that `make p1c-checks` completed.
P1C_REF_RUN := ../reports/rounds/P1-C/runs/20261015T014417Z-9aa326b
p1c1-checks: require-run
	$(MAKE) c1-variant RUN=$(RUN)
	mkdir -p research/reports/runs/$(RUN)/p1c1
	ROOTMARK_P1C1_OUT="$(CURDIR)/research/reports/runs/$(RUN)/p1c1" \
	  cargo test -q -p rootmark-index --release --test p1c1 -- --test-threads=1
	bash scripts/test_p1c1.sh research/reports/runs/$(RUN)/p1c1_cli.json \
	  >research/reports/runs/$(RUN)/p1c1_cli.log 2>&1 \
	  || { cat research/reports/runs/$(RUN)/p1c1_cli.log >&2; exit 1; }
	rc=0; $(PY) -m eval.compare_p1c1 regress $(P1C_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1c.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1c1_checks --run $(RUN)

# P1-C.1 round report from one RUN directory (TASKS C1.4): <RUN_DIR>/FINAL.md
# and reports/rounds/P1-C.1/FINAL.md; the RUN must hold pair_compare.json
report-p1c1:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1c1 --run-dir "$(abspath $(RUN_DIR))"

# P1-D criteria into the RUN (GOAL D-1..D-11; TASKS D7), under
# research/reports/runs/<RUN>/p1d/: the official MCP Inspector (D-1; needs
# Node.js and network), the MCP e2e (D-2..D-6, D-8 half, D-11), the HTTP
# and service tests (D-7), the watcher latency and SIGTERM tests (D-10), the
# agent scene (TASKS D6), the P1-D deliberate corruptions; then D-9
# (eval.compare_p1d regress against the P1-C.1 official RUN) and
# run_p1d_checks → p1d_checks.json. A criterion that fails is recorded, not
# fatal (its script's JSON must exist); a missing result fails.
P1C1_REF_RUN := ../reports/rounds/P1-C.1/runs/20261015T094412Z-b9f7622
P1D_DIR = research/reports/runs/$(RUN)/p1d
p1d-checks: require-run
	rm -rf $(P1D_DIR)
	mkdir -p $(P1D_DIR)
	bash scripts/mcp_inspector.sh $(P1D_DIR)/inspector >$(P1D_DIR)/inspector.log 2>&1 \
	  || echo "p1d-checks: mcp_inspector.sh exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/inspector/inspector.json || { cat $(P1D_DIR)/inspector.log >&2; exit 1; }
	bash scripts/mcp_e2e.sh $(P1D_DIR)/mcp_e2e >$(P1D_DIR)/mcp_e2e.log 2>&1 \
	  || echo "p1d-checks: mcp_e2e.sh exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/mcp_e2e/mcp_e2e.json || { tail -40 $(P1D_DIR)/mcp_e2e.log >&2; exit 1; }
	bash scripts/http_e2e.sh $(P1D_DIR)/http >$(P1D_DIR)/http_e2e.log 2>&1 \
	  || echo "p1d-checks: http_e2e.sh exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/http/http_e2e.json || { tail -40 $(P1D_DIR)/http_e2e.log >&2; exit 1; }
	ROOTMARK_D10_OUT="$(CURDIR)/$(P1D_DIR)/d10.json" ROOTMARK_SIGTERM_OUT="$(CURDIR)/$(P1D_DIR)/sigterm.json" \
	  cargo test -q -p rootmark-server --release --test serve_watch -- --test-threads=1 >$(P1D_DIR)/serve_watch.log 2>&1 \
	  || echo "p1d-checks: serve_watch exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/d10.json && test -f $(P1D_DIR)/sigterm.json || { tail -40 $(P1D_DIR)/serve_watch.log >&2; exit 1; }
	bash scripts/scene_agent.sh $(P1D_DIR)/scene >$(P1D_DIR)/scene.log 2>&1 \
	  || echo "p1d-checks: scene_agent.sh exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/scene/scene.json || { cat $(P1D_DIR)/scene.log >&2; exit 1; }
	bash scripts/test_tamper_p1d.sh $(P1D_DIR)/inspector/tools_list.json $(P1D_DIR)/tamper_p1d.json >$(P1D_DIR)/tamper_p1d.log 2>&1 \
	  || echo "p1d-checks: test_tamper_p1d.sh exit $$? (recorded)" >&2
	@test -f $(P1D_DIR)/tamper_p1d.json || { cat $(P1D_DIR)/tamper_p1d.log >&2; exit 1; }
	@# exit 3 (a difference) is recorded, not fatal; a missing file/field (1) fails
	rc=0; $(PY) -m eval.compare_p1d regress $(P1C1_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1c1.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1d_checks --run $(RUN)

# P1-D round report from one RUN directory (TASKS D8): <RUN_DIR>/FINAL.md and
# reports/rounds/P1-D/FINAL.md; the RUN must hold pair_compare.json
report-p1d:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1d --run-dir "$(abspath $(RUN_DIR))"

# P1-D.1 criteria into the RUN (GOAL D1-1..D1-13; TASKS D1.7), under
# research/reports/runs/<RUN>/p1d1/: scripts/test_p1d1.sh (D1-1..D1-9 by
# server/tests/p1d1_criteria.rs, D1-11 old vaults with the p1d-final tree,
# D1-13 contract check), the P1-D.1 deliberate corruptions, then D1-12's
# regress half (eval.compare_p1d1 regress against the P1-D official RUN,
# decision exclusions only) and run_p1d1_checks → p1d1_checks.json. Needs
# p1c1-checks (conformance_p1d1.json) and p1d-checks in the same RUN. A
# criterion that fails is recorded, not fatal; a missing result fails.
P1D_REF_RUN := ../reports/rounds/P1-D/runs/20261015T210419Z-a6b52fe
P1D1_DIR = research/reports/runs/$(RUN)/p1d1
p1d1-checks: require-run
	rm -rf $(P1D1_DIR)
	mkdir -p $(P1D1_DIR)
	bash scripts/test_p1d1.sh $(P1D1_DIR) >$(P1D1_DIR)/test_p1d1.log 2>&1 \
	  || { tail -40 $(P1D1_DIR)/test_p1d1.log >&2; exit 1; }
	@test -f $(P1D1_DIR)/p1d1.json || { tail -40 $(P1D1_DIR)/test_p1d1.log >&2; exit 1; }
	bash scripts/test_tamper_p1d1.sh $(P1D1_DIR)/tamper_p1d1.json >$(P1D1_DIR)/tamper_p1d1.log 2>&1 \
	  || echo "p1d1-checks: test_tamper_p1d1.sh exit $$? (recorded)" >&2
	@test -f $(P1D1_DIR)/tamper_p1d1.json || { cat $(P1D1_DIR)/tamper_p1d1.log >&2; exit 1; }
	@# exit 3 (a difference) is recorded, not fatal; a missing file/field (1) fails
	rc=0; $(PY) -m eval.compare_p1d1 regress $(P1D_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1d.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1d1_checks --run $(RUN)

# P1-D.1 round report from one RUN directory (TASKS D1.7): <RUN_DIR>/FINAL.md
# and reports/rounds/P1-D.1/FINAL.md; the RUN must hold pair_compare.json
report-p1d1:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1d1 --run-dir "$(abspath $(RUN_DIR))"

# P1-E criteria into the RUN (GOAL E-1..E-10, ADDENDUM E-12..E-16; TASKS
# E7), under research/reports/runs/<RUN>/p1e/: the E0 tests (E-14, E-15,
# ROOTMARK_P1E_OUT), the scene (E-1, E-3, E-5, E-6), the plugin e2e (E-2..E-6),
# the client gates (E-10), the local release binary (E-7, chain half), the
# P1-E deliberate corruptions, the deterministic-field audit (E-13), then
# E-9 (eval.compare_p1e regress against the P1-D.1 official RUN; exit 3
# recorded, not fatal) and run_p1e_checks → p1e_checks.json. Needs
# p1d1-checks in the same RUN. A criterion that fails is recorded, not
# fatal (its JSON must exist); a missing result fails.
P1D1_REF_RUN := ../reports/rounds/P1-D.1/runs/20261016T124520Z-1527365
P1E_DIR = research/reports/runs/$(RUN)/p1e
p1e-checks: require-run
	rm -rf $(P1E_DIR)
	mkdir -p $(P1E_DIR)/e0
	ROOTMARK_P1E_OUT="$(CURDIR)/$(P1E_DIR)/e0" cargo test -q -p rootmark-server --release --test p1e_e0 -- --test-threads=1 \
	  >$(P1E_DIR)/e0.log 2>&1 || echo "p1e-checks: p1e_e0 exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/e0/e14.json && test -f $(P1E_DIR)/e0/e15.json || { tail -40 $(P1E_DIR)/e0.log >&2; exit 1; }
	bash scripts/scene.sh $(P1E_DIR)/scene >$(P1E_DIR)/scene.log 2>&1 || echo "p1e-checks: scene.sh exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/scene/scene.json || { tail -40 $(P1E_DIR)/scene.log >&2; exit 1; }
	bash scripts/plugin_e2e.sh $(P1E_DIR)/plugin_e2e >$(P1E_DIR)/plugin_e2e.log 2>&1 || echo "p1e-checks: plugin_e2e.sh exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/plugin_e2e/plugin_e2e.json || { tail -40 $(P1E_DIR)/plugin_e2e.log >&2; exit 1; }
	bash scripts/client_checks.sh $(P1E_DIR)/client.json >$(P1E_DIR)/client.log 2>&1 || echo "p1e-checks: client_checks.sh exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/client.json || { tail -40 $(P1E_DIR)/client.log >&2; exit 1; }
	bash scripts/release_local.sh $(P1E_DIR)/release >$(P1E_DIR)/release.log 2>&1 || echo "p1e-checks: release_local.sh exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/release/release_local.json || { tail -40 $(P1E_DIR)/release.log >&2; exit 1; }
	bash scripts/test_tamper_p1e.sh $(P1E_DIR)/tamper_p1e.json >$(P1E_DIR)/tamper_p1e.log 2>&1 || echo "p1e-checks: test_tamper_p1e.sh exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/tamper_p1e.json || { tail -40 $(P1E_DIR)/tamper_p1e.log >&2; exit 1; }
	bash scripts/check_compare_fields.sh --out $(P1E_DIR)/check_compare_fields.json >/dev/null || echo "p1e-checks: check_compare_fields exit $$? (recorded)" >&2
	@test -f $(P1E_DIR)/check_compare_fields.json || exit 1
	@# exit 3 (a difference) is recorded, not fatal; a missing file/field (1) fails
	rc=0; $(PY) -m eval.compare_p1e regress $(P1D1_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1d1.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1e_checks --run $(RUN)

# P1-E.1 (GOAL P1-E.1 E1-1..E1-9; TASKS E1.5): the status tests on the
# p1d-final `/2` index fixture (E1-1, E1-2), the contract check with its
# status step (E1-8), the deliberate cases, the release package of this
# commit with the real-Obsidian e2e (E1-3, E1-9) and its checklist (E1-4,
# E1-5; Linux not attempted, the package outside the RUN), E1-6 against the
# P1-E official RUN, then p1e1_checks.json and the historical FAIL labels of
# every recorder (K-E1.4) → research/reports/runs/<RUN>/p1e1/
P1E_REF_RUN := ../reports/rounds/P1-E/runs/20261017T024341Z-632f29e
P1E1_DIR = research/reports/runs/$(RUN)/p1e1
P1E1_DIST ?= $(CURDIR)/target/p1e1-dist
p1e1-checks: require-run
	rm -rf $(P1E1_DIR)
	mkdir -p $(P1E1_DIR)/status $(P1E1_DIR)/release
	ROOTMARK_P1E1_OUT="$(CURDIR)/$(P1E1_DIR)/status" cargo test -q -p rootmark-server --release --test p1e1_status -- --test-threads=1 \
	  >$(P1E1_DIR)/status.log 2>&1 || echo "p1e1-checks: p1e1_status exit $$? (recorded)" >&2
	@test -f $(P1E1_DIR)/status/e1_1.json && test -f $(P1E1_DIR)/status/e1_2.json || { tail -40 $(P1E1_DIR)/status.log >&2; exit 1; }
	bash scripts/contract_check.sh $(P1E1_DIR)/contract.json >$(P1E1_DIR)/contract.log 2>&1 || echo "p1e1-checks: contract_check.sh exit $$? (recorded)" >&2
	@test -f $(P1E1_DIR)/contract.json || { tail -40 $(P1E1_DIR)/contract.log >&2; exit 1; }
	bash scripts/test_tamper_p1e1.sh $(P1E1_DIR)/tamper_p1e1.json >$(P1E1_DIR)/tamper_p1e1.log 2>&1 || echo "p1e1-checks: test_tamper_p1e1.sh exit $$? (recorded)" >&2
	@test -f $(P1E1_DIR)/tamper_p1e1.json || { tail -40 $(P1E1_DIR)/tamper_p1e1.log >&2; exit 1; }
	ROOTMARK_RELEASE_LINUX=0 bash scripts/release.sh $(P1E1_DIST) >$(P1E1_DIR)/release.log 2>&1 || echo "p1e1-checks: release.sh exit $$? (recorded)" >&2
	@test -f $(P1E1_DIST)/release.json && test -f $(P1E1_DIST)/plugin_e2e_real/plugin_e2e_real.json || { tail -40 $(P1E1_DIR)/release.log >&2; exit 1; }
	bash scripts/release_check.sh $(P1E1_DIST) >$(P1E1_DIR)/release_check.log 2>&1 || echo "p1e1-checks: release_check.sh exit $$? (recorded)" >&2
	@test -f $(P1E1_DIST)/release_check.json || { tail -40 $(P1E1_DIR)/release_check.log >&2; exit 1; }
	cp $(P1E1_DIST)/release.json $(P1E1_DIST)/release_check.json $(P1E1_DIR)/release/
	cp $(P1E1_DIST)/plugin_e2e_real/plugin_e2e_real.json $(P1E1_DIR)/release/plugin_e2e_real.json
	@if [ -f $(P1E1_DIST)/plugin_e2e_real/access_log.json ]; then cp $(P1E1_DIST)/plugin_e2e_real/access_log.json $(P1E1_DIR)/release/access_log_real.json; \
	  else echo '[]' >$(P1E1_DIR)/release/access_log_real.json; fi
	bash scripts/test_tamper_p1e1_e2e.sh $(P1E1_DIR)/tamper_p1e1_e2e.json >$(P1E1_DIR)/tamper_p1e1_e2e.log 2>&1 || echo "p1e1-checks: test_tamper_p1e1_e2e.sh exit $$? (recorded)" >&2
	@test -f $(P1E1_DIR)/tamper_p1e1_e2e.json || { tail -40 $(P1E1_DIR)/tamper_p1e1_e2e.log >&2; exit 1; }
	bash scripts/check_compare_fields.sh --out $(P1E1_DIR)/check_compare_fields.json >/dev/null || echo "p1e1-checks: check_compare_fields exit $$? (recorded)" >&2
	@test -f $(P1E1_DIR)/check_compare_fields.json || exit 1
	@# exit 3 (a difference) is recorded, not fatal; a missing file/field (1) fails
	rc=0; $(PY) -m eval.compare_p1e1 regress $(P1E_REF_RUN) reports/runs/$(RUN) \
	  --out reports/runs/$(RUN)/regress_vs_p1e.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]
	$(PY) -m eval.run_p1e1_checks --run $(RUN)
	rc=0; $(PY) -m eval.historical --run-dir reports/runs/$(RUN) --out reports/runs/$(RUN)/p1e1/historical.json >/dev/null || rc=$$?; \
	  [ $$rc -eq 0 ] || [ $$rc -eq 3 ]

# P1-E.1 round report from one RUN directory: <RUN_DIR>/FINAL.md and
# reports/rounds/P1-E.1/FINAL.md
report-p1e1:
	@if [ -z "$(RUN_DIR)" ]; then echo "error: RUN_DIR=<research/reports/runs/RUN> is required" >&2; exit 1; fi
	$(PY) -m eval.report_p1e1 --run-dir "$(abspath $(RUN_DIR))"

# P2-0 (TASKS Z5): the real-Obsidian session harness (D-B), the plugin
# bundle it installs, then the window tuning on the D-B tuning set
# (p2_0/tuning.json) and the test sets with the chosen W (D-E, D-A, D-C,
# D-B test) → research/reports/runs/<RUN>/p2_0/
p2-0-harness:
	cd clients/obsidian && npm run build && npx esbuild e2e/p2_0/sessions.ts --bundle --format=esm \
	  --platform=node --target=node22 --outfile=e2e/p2_0/.build/sessions.mjs

p2-0-tune: require-run build p2-0-harness
	$(PY) -m eval.run_p2_0 --phase tune --run $(RUN) --rootmark ../target/release/rootmark

p2-0-test: require-run build p2-0-harness
	$(PY) -m eval.run_p2_0 --phase test --run $(RUN) --rootmark ../target/release/rootmark

# Clean-clone double run (scripts/verify.sh pair; two temp dirs, compared)
verify:
	bash scripts/verify.sh
