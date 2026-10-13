SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c
PY := uv run python
MDK := ./target/release/mdk

.PHONY: setup build test test-rust test-py lint d1 d2 d3 freeze-check \
        eval-h1s-tune eval-h1s eval-h1r eval-h2-tune eval-h2 eval-h3 eval-h4 report verify require-run

setup:
	uv sync --frozen
	$(MAKE) build

build:
	cargo build --release
	uv run --frozen maturin develop --release --quiet

test: test-rust test-py

test-rust:
	cargo test --release

test-py: build
	uv run --frozen pytest -q

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	uv run --frozen ruff check .

require-run:
	@if [ -z "$(RUN)" ]; then echo "error: RUN=<run_id> is required" >&2; exit 1; fi

d2:
	$(PY) -m eval.datasets.git_history $(ARGS)

d3:
	$(PY) -m eval.datasets.vault $(ARGS)

d1: d3
	$(PY) -m eval.datasets.synthetic_edits $(ARGS)

freeze-check:
	$(PY) -m eval.freeze_check

eval-h1s-tune: require-run
	$(PY) -m eval.run_h1s --phase tune --run $(RUN)

eval-h1s: require-run
	$(PY) -m eval.run_h1s --phase test --run $(RUN)

eval-h1r: require-run
	$(PY) -m eval.run_h1r --run $(RUN)

eval-h2-tune: require-run
	$(PY) -m eval.run_h2 --phase tune --run $(RUN)

eval-h2: require-run
	$(PY) -m eval.run_h2 --phase test --run $(RUN)

eval-h3: require-run
	$(PY) -m eval.run_h3 --run $(RUN)

eval-h4: require-run
	$(PY) -m eval.run_h4 --run $(RUN)

report: require-run
	$(PY) -m eval.report --run $(RUN)

verify:
	bash scripts/verify.sh
