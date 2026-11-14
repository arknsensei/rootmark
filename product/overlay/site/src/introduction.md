# RootMark

**Block-level lineage, provenance and a deterministic derived index for Markdown written by people and agents together.**

![Conformance](https://img.shields.io/endpoint?url=https%3A%2F%2Fdocs.rootmark.ai%2Fspec%2Fbadge.json) — measured by this site's build from [`conformance.json`](spec/conformance.json), never typed by hand.

RootMark is one binary, `rootmark`, that runs beside a folder of Markdown files (a *vault*). For every block of every note it keeps **lineage** (which block it came from, across edits, splits, merges and moves), **provenance** (who wrote it — a person, an agent by name, a rule a person wrote — and through which channel), a **layer** (`canonical` approved knowledge, `proposal` an agent's suggestion, optionally `unverified`) and a **deterministic index** (lexical and vector search over approved blocks; the same files give the same index on every machine).

Agents talk to it over **MCP** (stdio); people and their editors over a **local HTTP API** (127.0.0.1 only) and the **CLI**; an **Obsidian plugin** shows blame, proposals with *Promote*, search, checkpoints and traces. Your Markdown files are never changed by RootMark: everything derived lives in `<vault>/.rootmark/`.

What is measured, and under which conditions — in particular for agents that write files directly instead of through MCP — is on the page [How RootMark differs from git](git.md), with a link to the record of every claim.

- **Start:** [Install](install.md), then the [Guide](guide.md).
- **Build a client:** the [client contract](contract.md) and the [specification](spec/index.md).
- **Source and releases:** <https://github.com/rootmark-ai/rootmark>.
