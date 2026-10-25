# rootmark for VS Code (skeleton, P2-B)

A second client of `rootmark serve` beside the Obsidian plugin, over the same
HTTP contract (`docs/CLIENT_CONTRACT.md`, version 2) and the same generated
types (`clients/obsidian/src/contract.gen.ts`, one file imported by both
clients, with the client core `api.ts` and the contract checker). It is a
skeleton: it is not published to the Marketplace and has no settings UI.

## What it does
- **Status** (`rootmark: Status`): the vault, versions, index state, policy;
  the status bar shows the connection.
- **Search** (`rootmark: Search`): a query with filters typed inline —
  `prefix:<path>` (repeatable), `from:YYYY-MM-DD`, `to:YYYY-MM-DD`,
  `proposals` — opened at the hit's line when its location is current.
- **Blame** (`rootmark: Blame this note`, also on open and after a save): one
  decoration per layer over each block's lines, from the server's block
  locations (`loc`). `proposal` and `unverified` blocks stand out. The
  decorations are drawn only when the editor's text is exactly the bytes the
  locations belong to (`content_hash`); otherwise the status bar says why —
  `stale` (the file is not the version rootmark ingested), `changed`
  (unsaved, or saved and not yet ingested), `unlocated` (a v1 server) — and no
  line is guessed.
- **History**, **Promote**, **Demote** of the block at the cursor (the human
  channel: `X-Consumer: vscode`; promote and demote need `[identity]
  human_id`).

## Saves are not attested
VS Code saves a note as any editor does: rootmark's watcher sees an
**unattested external change**. This client never calls `attest_session` or
`attest` (contract §2.6). Under `[attestation] unattested = "canonical"` (the
default recipe of a new vault) the change is the human's, as for any editor;
under `unattested = "unverified"` the saved blocks are `unverified` (actor
`unattested_fs`) until a human promotes them — exactly as for a save from a
script or another editor. Attesting VS Code saves (like the Obsidian plugin
does) is the work of a later round. See `docs/HEADLESS_CHANNEL.md` for the
other human channel without attestation.

## Token
The token is read from the vault's `.rootmark/server.token` at each connect
and kept in memory only, for the lifetime of the client object; it is never
written to settings, `globalState`, `SecretStorage`, the workspace or a log.
Like the Obsidian plugin, an extension that can read the token is in the
human channel (`docs/THREAT_MODEL.md`): run agents under an account that
cannot read `.rootmark/`.

## Build and test
```
npm ci
npm test          # vitest: the command layer against a fake server, contract-checked
npm run lint      # tsc --noEmit, eslint
npm run build     # dist/extension.cjs (vscode external, no runtime dependency)
```
To try it, open a vault served by `rootmark serve --http 127.0.0.1:<port>`
(`[server] http_port` in `.rootmark/policy.toml`) and run the extension from
this folder (`code --extensionDevelopmentPath=clients/vscode <vault>`).
