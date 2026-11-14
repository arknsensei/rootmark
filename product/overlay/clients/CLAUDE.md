# clients — agent notes
Responsibility: the TypeScript clients (`obsidian`; `vscode`, a skeleton).
Dependency direction: clients depend only on the server's MCP/HTTP contract; `core`/`index`/`server`/`bindings` are never imported, only the endpoints of `docs/CLIENT_CONTRACT.md` are called (`scripts/check_deps.sh`).
Language and tools: TypeScript, `tsc`, `eslint`, `vitest` (`make test-ts`, `make lint-ts`).
Frozen surfaces: none.
Test commands: `make test-ts`.
Not in this package: runtime logic (TypeScript is client only); dependencies on the sources of `core`/`index`/`server`. Package notes: `clients/obsidian/CLAUDE.md`.
