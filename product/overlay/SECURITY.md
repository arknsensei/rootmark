# Security policy

## Reporting a vulnerability

Please **do not open a public issue** for a security problem. Report it privately: the repository's **Security** tab → **Report a vulnerability**, or e-mail **security@rootmark.ai**. You get an answer within seven days; a fix or a decision is coordinated with you before anything is disclosed.

Useful in a report: the version (`rootmark --version`), how you installed it (release archive, Homebrew, container image), the channel (MCP stdio, local HTTP, CLI, file watcher, Obsidian plugin), the steps or a minimal vault that reproduces it, and what you expected instead.

## Supported versions

| Version | Supported |
|---|---|
| 0.4.x | yes |
| older | no (upgrade: `rootmark migrate`, see the guide) |

## Scope and threat model

RootMark runs on one machine, for one user's vault: `rootmark serve` listens on MCP stdio and on `127.0.0.1` only (in the container image: MCP stdio only). The trust boundaries of the channels and the accepted assumptions are in [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md). In short:

- the MCP channel is always an **agent**; the HTTP channel with the token in `.rootmark/server.token` is always the **human** named in `policy.toml [identity] human_id`;
- **accepted assumption:** any local process that can read the token file counts as the human, and a process that can write the vault's files directly writes as the human while the watcher is on (with the default policy). Run agents in an account or sandbox that cannot read `.rootmark/`, and give them MCP only;
- multi-user servers, remote access and encryption at rest are not in scope yet.

A way around these boundaries that the threat model does not already accept (an agent promoting its own proposal, a write that changes a canonical block under `append_only`, a request that escapes the vault, a silent failure) is a vulnerability.
