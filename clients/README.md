# clients

TypeScript clients only (docs/PACKAGES.md): `clients/obsidian` (phase 1, round P1-E) and `clients/vscode` (phase 2). Clients talk to `server` over MCP/HTTP and never import `core`, `index`, `server` or `bindings` (checked by `scripts/check_deps.sh`). `clients/obsidian` since round P1-E (contract: `docs/CLIENT_CONTRACT.md`).
