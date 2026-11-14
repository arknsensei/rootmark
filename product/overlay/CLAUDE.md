# RootMark — rules for coding agents

This repository is generated from the development repository (one commit per release; see README, "How this repository is made"). A change you make here is carried into the development repository by a person and comes back with the next release; keep it small and self-contained.

## Read first
1. `README.md`, then `docs/CLIENT_CONTRACT.md` (the contract between the server and every client).
2. `spec/SPEC.md` for the part you touch, and `spec/CONFORMANCE.md`.
3. The package's own `CLAUDE.md` (`index/`, `server/`, `bindings/python/`, `clients/`, `clients/obsidian/`).

## Fixed rules
- Runtime code is Rust only (`core`, `index`, `server`, `bindings`). TypeScript only in `clients/`. No runtime code in another language.
- Dependency direction: `core ← index ← server ← clients`. Never the reverse (`scripts/check_deps.sh`). Clients reach the server only through the contract (MCP / `POST /v1/<tool>` / `GET /v1/status`).
- Never write identifiers or metadata into Markdown files: everything derived lives in `<vault>/.rootmark/` and can be rebuilt from the files.
- No timestamps or randomness in identifiers; the same input gives the same identifiers, event log and index on every machine.
- No silent failure: an unresolved reference, a missing file or a manifest mismatch is an error.
- `spec/SPEC.md` §1–6 and the conformance fixtures do not change without a specification version change.
- Do not change a test to make it pass.
- Dependencies are pinned to exact versions published at least 14 days before they are adopted (`scripts/check_dep_age.sh`).

## Quality gates
`make test`, `make lint`, `make conformance` (39/39). The CI runs the same on macOS and Linux.
