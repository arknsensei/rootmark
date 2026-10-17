# Security policy

## Reporting a vulnerability

Please **do not open a public issue** for a security problem. Report it privately through GitHub: the repository's **Security** tab → **Report a vulnerability** (private vulnerability reporting). The report reaches the maintainers only; you get an answer within seven days, and a fix or a decision is coordinated with you before anything is disclosed.

While the repository is private, GitHub does not offer private vulnerability reporting: collaborators report to the repository owner directly. Making the repository public includes enabling private vulnerability reporting (Settings → Code security) and removing this sentence.

Useful in a report: the rootmark version (`rootmark --version`), the channel (MCP stdio, local HTTP, CLI, file watcher, Obsidian client), the steps or a minimal vault that reproduces it, and what you expected instead.

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x | yes |

## Scope and threat model

rootmark 0.1 runs on one machine, for one user's vault: `rootmark serve` listens on MCP stdio and on `127.0.0.1` only. The threat model, the trust boundaries of the channels and the accepted assumptions are in [docs/SECURITY.md](docs/SECURITY.md) and [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) (Turkish). In short:

- the MCP channel is always an **agent**, the HTTP channel with the token in `.rootmark/server.token` is always the **human** of `policy.toml [identity] human_id`;
- **accepted assumption:** any local process that can read the token file counts as the human, and a process that can write the vault's files directly writes as the human while the watcher is on. Run agents in an account or sandbox that cannot read `.rootmark/` and give them MCP only;
- multi-user servers, remote access and encryption at rest are phase 2.

A way around these boundaries that the documents do not already accept (an agent promoting its own proposal, a write that changes a canonical block under `append_only`, a request that escapes the vault, a silent failure) is a vulnerability.
