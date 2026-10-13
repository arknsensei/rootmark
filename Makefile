SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

# Quality gates (CLAUDE.md, docs/DEV_PROCESS.md §3): test, lint, conformance, verify.
# Python lives only in research/ (its own uv project); TypeScript only in clients/.

UVR := cd research && uv run --frozen
TS_PKGS := $(wildcard clients/*/package.json)

.PHONY: setup setup-py test test-rust test-py test-ts lint lint-rust lint-py lint-ts \
        check-deps test-check-deps conformance regress verify

setup: setup-py
	cargo build --workspace --release

setup-py:
	cd research && uv sync --frozen

test: test-rust test-py test-ts

test-rust:
	cargo test --workspace --release

test-py: setup-py
	$(UVR) pytest -q

test-ts:
ifeq ($(strip $(TS_PKGS)),)
	@echo "test-ts: no TypeScript client package (clients/*/package.json) in this round; nothing to run"
else
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npx vitest run); done
endif

lint: lint-rust lint-py lint-ts check-deps test-check-deps

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

regress:
	@echo "error: make regress is implemented in P1-A/A4" >&2; exit 1

verify:
	@echo "error: make verify is implemented in P1-A/A6" >&2; exit 1
