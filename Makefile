SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c
PY := uv run python
# v2 code paths (frozen eval, v2 runners) never pass a matcher: run them under
# matcher v2 so they reproduce v2 behaviour exactly (docs/progress/v3/ASSUMPTIONS T21)
PY2 := MDK_MATCHER=v2 uv run python
MDK := ./target/release/mdk

.PHONY: setup build test test-rust test-py lint d1 d2 d3 d1k4 d1h freeze-check freeze-check-v3 \
        eval-h1s-tune eval-h1s eval-h1r eval-h2-tune eval-h2 eval-h3 eval-h4 report verify require-run

setup:
	uv sync --frozen
	$(MAKE) build

build:
	cargo build --release
	uv run --frozen maturin develop --release --quiet
	@# macOS ld-27037 can emit an 8-byte-misaligned LINKEDIT string pool that dyld
	@# rejects; fail here, loudly, instead of at first use (see ASSUMPTIONS T5).
	uv run --frozen python -c "import mdkernel_core"

test: test-rust test-py

test-rust:
	cargo test --release

test-py: build
	uv run --frozen pytest -q

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	uv run --frozen ruff check .
	uv run --frozen ruff format --check .

require-run:
	@if [ -z "$(RUN)" ]; then echo "error: RUN=<run_id> is required" >&2; exit 1; fi

d2:
	$(PY2) -m eval.datasets.git_history $(ARGS)

d3:
	$(PY2) -m eval.datasets.vault $(ARGS)

d1: d3
	$(PY2) -m eval.datasets.synthetic_edits $(ARGS)

d1k4: d3
	$(PY) -m eval.datasets.synthetic_edits_v2 --set d1k4 $(ARGS)

d1h: d3
	$(PY) -m eval.datasets.synthetic_edits_v2 --set d1h $(ARGS)

freeze-check:
	$(PY2) -m eval.freeze_check

freeze-check-v3:
	$(PY) -m eval.freeze_check_v3

eval-h1s-tune: require-run
	$(PY2) -m eval.run_h1s --phase tune --run $(RUN)

eval-h1s: require-run
	$(PY2) -m eval.run_h1s --phase test --run $(RUN)

eval-h1r: require-run
	$(PY2) -m eval.run_h1r --run $(RUN)

eval-h2-tune: require-run
	$(PY2) -m eval.run_h2 --phase tune --run $(RUN)

eval-h2: require-run
	$(PY2) -m eval.run_h2 --phase test --run $(RUN)

eval-h3: require-run
	$(PY2) -m eval.run_h3 --run $(RUN)

eval-h4: require-run
	$(PY2) -m eval.run_h4 --run $(RUN)

report: require-run
	$(PY2) -m eval.report --run $(RUN)

verify:
	bash scripts/verify.sh
