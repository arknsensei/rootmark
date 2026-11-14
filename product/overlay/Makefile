SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

# RootMark — the product's build and quality gates: build, test, lint,
# conformance, models, the clients. Outputs go to out/ (git-ignored).

TS_PKGS := $(wildcard clients/*/package.json)

.PHONY: build test test-rust test-ts lint lint-rust lint-ts check-deps check-dep-age check-client-types \
        conformance models models-multilingual clients scene

build:
	cargo build --release --locked -p rootmark-server --bin rootmark

test: test-rust test-ts

# Two recorders of older releases (p1e1_status.rs E1-1, p1e_e0.rs E-14)
# expect `index_stale` for an older index that is now `migration_required`
# (since 0.2.0); the new behaviour is tested by server/tests/p2a_status.rs.
test-rust:
	cargo test --workspace --release --locked -- --skip e1_1_real_old_index_status_and_reads \
	  --skip e14_stale_index_answers_index_stale_or_is_rebuilt

test-ts:
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npx vitest run); done

lint: lint-rust lint-ts check-deps check-client-types

lint-rust:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --release --locked -- -D warnings

lint-ts:
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npx tsc --noEmit && npx eslint .); done

# dependency direction core ← index ← server ← clients
check-deps:
	bash scripts/check_deps.sh

# every pinned dependency (crates, the npm lock files with their transitive
# packages, workflow actions) published at least 14 days before adoption
check-dep-age:
	bash scripts/check_dep_age.sh

# clients/obsidian/src/contract.gen.ts matches the live tools/list and
# docs/CLIENT_CONTRACT.md (regenerate: scripts/gen_client_types.sh)
check-client-types:
	bash scripts/gen_client_types.sh --check

# Levels 1–3 of spec/CONFORMANCE.md on spec/fixtures: 39/39 (needs `make models`)
conformance:
	rm -f out/conformance.json
	ROOTMARK_FIXTURES="$(CURDIR)/spec/fixtures" ROOTMARK_CONFORMANCE_REPORT="$(CURDIR)/out/conformance.json" \
	  cargo test -p rootmark-core --release --locked --test conformance -- --nocapture
	@test -f out/conformance.json || { echo "error: out/conformance.json not written" >&2; exit 1; }
	ROOTMARK_FIXTURES="$(CURDIR)/spec/fixtures" ROOTMARK_CONFORMANCE_REPORT="$(CURDIR)/out/conformance.json" \
	  cargo test -p rootmark-index --release --locked --test conformance_c3 -- --nocapture
	@jq -e --arg root "$$(cd spec/fixtures && pwd -P)" '.level_claimed == 3 and .passed == .total and .total == 39 and .fixtures_root == $$root and (.fixtures_manifest_hash | type) == "string" and ([.tests[] | select(.level == 3)] | length) == 8' out/conformance.json >/dev/null \
	  || { echo "error: conformance level 3 not reached (out/conformance.json)" >&2; exit 1; }
	@echo "conformance: level $$(jq -r .level_claimed out/conformance.json), $$(jq -r '"\(.passed)/\(.total)"' out/conformance.json) (out/conformance.json)"

models:
	bash scripts/fetch_model.sh
	cargo run -q --release --locked -p rootmark-index --bin rootmark-index -- models verify

models-multilingual:
	bash scripts/fetch_model.sh . MANIFEST.multilingual-minilm-l12-v2.json
	cargo run -q --release --locked -p rootmark-index --bin rootmark-index -- models verify --embedder multilingual-minilm-l12-v2

clients:
	@for p in $(dir $(TS_PKGS)); do (cd "$$p" && npm ci && npm run build); done

# the eleven-step product scene on a clean copy of spec/fixtures/mini_vault
scene: build
	bash scripts/scene.sh out/scene
