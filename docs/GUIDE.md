# Running rootmark — a guide (0.3.1)

This guide takes you from nothing to a vault that people and agents write together, with every block's origin recorded: install, prepare a vault, run the server, connect an agent, install the Obsidian plugin, daily use, collecting data for an evaluation, upgrading, headless use on Linux, and troubleshooting. Every command block is meant to be copied and run as it stands; the outputs shown come from a real run of 0.3.0 (0.3.1 changes only the watcher: §10 and "Known issues").

rootmark is one binary, `rootmark`. It runs beside a folder of Markdown files (a *vault*) and keeps everything it derives in `<vault>/.rootmark/`; it never writes identifiers or metadata into your Markdown. It answers three ways:

| Channel | Who | How |
|---|---|---|
| MCP over stdio | agents (Claude Code, any MCP client) | the agent's client starts `rootmark serve --mcp stdio`; the actor is always `agent:<client name>` |
| HTTP on `127.0.0.1` | people and their editors (the Obsidian plugin, `curl`, a bot) | `Authorization: Bearer <.rootmark/server.token>`; the actor is `human:<[identity] human_id>` |
| command line | you, at a terminal | `rootmark <command> <vault> …` |

What an agent writes with `propose` is a `proposal` and is not returned by a default search until a person promotes it; what a person writes is `canonical`. With `[attestation] unattested = "unverified"` (below) a file that a process writes directly, without a client that vouches for the save, is `unverified` and is not knowledge either.

Contents: [1 Install](#1-install) · [2 Prepare a vault](#2-prepare-a-vault) · [3 Run the server](#3-run-the-server) · [4 Connect an agent](#4-connect-an-agent) · [5 The Obsidian plugin](#5-the-obsidian-plugin) · [6 Daily use](#6-daily-use) · [7 Collecting data for an evaluation](#7-collecting-data-for-an-evaluation) · [8 Upgrading and migrating](#8-upgrading-and-migrating) · [9 Headless use on Linux](#9-headless-use-on-linux) · [10 Troubleshooting](#10-troubleshooting) · [Known issues](#known-issues)

## 1. Install

Release targets: macOS on Apple silicon (`aarch64-apple-darwin`) and Linux x86_64 (`x86_64-unknown-linux-gnu`, glibc 2.38 or newer: Debian 13, Ubuntu 24.04 and later). A Windows x86_64 archive and `install.ps1` are published for information.

**Prerequisites:** `bash`, `curl`, `tar`, `shasum` (macOS) or `sha256sum` (Linux), and `jq` (for the model download). macOS: `brew install jq`. Debian/Ubuntu: `sudo apt-get install -y curl ca-certificates jq`. The GitHub CLI (`gh`) is optional and only checks provenance.

### 1.1 macOS (Apple silicon) and Linux x86_64

Download the installer and the checksums of the release, check them, then run the installer. It downloads the archive for your platform, checks its SHA-256 against `SHA256SUMS`, installs `~/.local/bin/rootmark` and fetches the embedder model into `~/.local/share/rootmark/models` (each file checked against the pinned manifest):

```sh
VER=0.3.1
mkdir -p ~/rootmark-install && cd ~/rootmark-install
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | shasum -a 256 -c      # Linux: sha256sum -c
gh attestation verify install.sh -R arknsensei/rootmark   # optional (GitHub CLI): provenance
bash install.sh --version "$VER"
```

`install.sh --prefix <dir>` installs under `<dir>/bin` and `<dir>/share/rootmark` instead of `~/.local`. Piping `curl … | bash` would skip both checks; download first.

Then put the binary on your `PATH` and tell rootmark where the models are (add both lines to your shell profile, e.g. `~/.zshrc` or `~/.bashrc`):

```sh
export PATH="$HOME/.local/bin:$PATH"
export ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark --version
rootmark --spec-version
rootmark conformance >/dev/null
```

```text
rootmark 0.3.1
2.0.3
rootmark conformance: level 3, 39/39 (spec 2.0.3)
```

`rootmark conformance` runs the specification's conformance suite (Levels 1–3) on fixtures embedded in the binary; the JSON report goes to stdout.

To check every archive's provenance too: `gh attestation verify rootmark-$VER-aarch64-apple-darwin.tar.gz -R arknsensei/rootmark` (download the archive from the release page first).

### 1.2 Turkish and mixed-language vaults: the multilingual model

The defaults (an ASCII lexical tokenizer and an English embedder) suit English notes. For Turkish or Turkish/English notes use the Turkish-folding tokenizer and the multilingual embedder (on a Turkish test set: hybrid nDCG@10 0.88 against 0.80 with the defaults). The release archive carries only the default model's manifest, so fetch the multilingual one from the release tag, then the model (≈ 480 MB):

```sh
curl -fsSL -o "$ROOTMARK_MODELS/MANIFEST.multilingual-minilm-l12-v2.json" \
  "https://raw.githubusercontent.com/arknsensei/rootmark/v0.3.1/models/MANIFEST.multilingual-minilm-l12-v2.json"
bash "$ROOTMARK_MODELS/fetch_model.sh" "$(dirname "$ROOTMARK_MODELS")" MANIFEST.multilingual-minilm-l12-v2.json
```

Then set `tokenizer` and `embedder` in the vault's recipe (§2.2).

### 1.3 Linux in Docker

To try the Linux build on any machine with Docker (on Apple silicon the `linux/amd64` image runs emulated and is slow):

```sh
docker run --rm -it --platform linux/amd64 debian:trixie-slim bash
```

and inside the container:

```sh
apt-get update && apt-get install -y --no-install-recommends curl ca-certificates jq
VER=0.3.1
mkdir -p ~/rootmark-install && cd ~/rootmark-install
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | sha256sum -c
bash install.sh --version "$VER"
export PATH="$HOME/.local/bin:$PATH" ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark conformance >/dev/null
```

### 1.4 Uninstall

```sh
rm -f ~/.local/bin/rootmark
rm -rf ~/.local/share/rootmark
```

A vault's `.rootmark/` stays; delete it to remove everything rootmark derived from that vault (your Markdown is untouched either way).

## 2. Prepare a vault

A vault is any folder of Markdown files. The commands below use a variable for it; set it to your folder (use a copy the first time):

```sh
export VAULT="$HOME/notes"
```

### 2.1 Ingest

`ingest-dir` reads every `*.md` under the folder in byte-wise path order and writes the event log (`.rootmark/events.jsonl`: one line per block event), the block store and the snapshots. Your files are only read.

```sh
rootmark ingest-dir "$VAULT" "$VAULT" | jq '{events_hash, documents: (.ingested | length)}'
```

```json
{
  "events_hash": "30316ce84f3b851001375fb041d40582",
  "documents": 3
}
```

`events_hash` is the blake3 of the event log: the same files give the same hash on every machine.

### 2.2 The recipe: `.rootmark/policy.toml`

The recipe says who may promote, what agents may change, which channel's writes count, and how the index is built. Write it before the first `serve` (a `serve` without a recipe writes a default one in which writes to the files count as the human's and HTTP writes are refused until you set `[identity]`). Every table below is required except those marked optional; a missing table is an error that names it (`missing field version_trigger`).

```sh
mkdir -p "$VAULT/.rootmark"
cat > "$VAULT/.rootmark/policy.toml" <<'EOF'
[promotion]
policy = "human_only"            # only a person (the HTTP channel) promotes by hand
version = "2026-11-03"           # any non-empty string; required when there are rules

# optional: an agent's proposals under a prefix are approved in advance
[[promotion.rules]]
id = "collector"
actor = "agent:collector"        # the agent's MCP client name (see §4.3)
path_prefix = "sources/"         # vault-relative; keep it narrow
to = "canonical"

[edit]
policy = "append_only"           # an agent may not change or delete canonical blocks

[version_trigger]
idle_ms = 30000
min_sim_change = 0.2

[journal]
retention_hours = 168

[server]
http_port = 7357                 # the HTTP port; the Obsidian plugin reads it
token_rotate_days = 30
max_propose_bytes = 1048576

[index]
compact_every_s = 300
# Turkish or mixed vaults (after §1.2):
# tokenizer = "uni_fold_tr"
# embedder = "multilingual-minilm-l12-v2"

[identity]
human_id = "ada"                 # you: HTTP actions are recorded as human:ada

[attestation]
unattested = "unverified"        # a file write no client vouched for is not knowledge
EOF
rootmark policy lint "$VAULT" | jq '{ok, promotion, rules: [.rules[].id]}'
```

```json
{
  "ok": true,
  "promotion": "human_only",
  "rules": [
    "collector"
  ]
}
```

What each table means:

- **`[promotion]`** — `policy = "human_only"`: a lineage moves from `proposal` (or `unverified`) to `canonical` only when a person promotes it over HTTP (or `rootmark promote --actor human:<id>`); `"deny"`: nobody promotes. **`[[promotion.rules]]`** (optional, any number): an agent whose MCP client name is `actor`'s name, writing with `propose` under `path_prefix`, gets its blocks promoted at once; each such promotion is an event naming the rule and the recipe's `version`. The name is the agent's own declaration (anyone who can start that agent can use it), so keep prefixes narrow. A file written directly, and `unverified` content, are never promoted by a rule.
- **`[edit]`** — `append_only` (default): an agent's `propose` may add sections and fix its own proposals, but never change, move or delete a `canonical` block; `unrestricted` allows it (every change still names its actor).
- **`[version_trigger]`**, **`[journal]`** — when edits become a new version of a block and how long the raw journal keeps unversioned keystrokes; the values above are the defaults.
- **`[server]`** — `http_port`: the port `serve --http 127.0.0.1` uses (the Obsidian plugin reads it); `token_rotate_days`: the HTTP token is renewed after that many days; `max_propose_bytes`: the largest `propose`.
- **`[index]`** — `compact_every_s`: how often new chunks move into the main index; `tokenizer` and `embedder`: see §1.2 (changing them later needs `rootmark migrate`); optional `date_from_path = "{yyyy}/{mm}/{dd}"` or `date_field = "date"` enable the search date filter.
- **`[identity]`** — `human_id`: the person behind the HTTP token. Without it every HTTP write (promote, demote, checkpoint, attest) is refused with 403.
- **`[attestation]`** — what a change seen by the file watcher counts as when no human client attested the save. `"unverified"`: a file written by any process that is not an attesting client (the Obsidian plugin, `rootmark attest`) is `unverified`, shown only with `include_proposal`, and promoted only by a person. `"canonical"`: every file change counts as the human's (the default `serve` writes). Use `"unverified"` if an agent can write into the vault folder directly.
- **`[retention]`**, **`[prune]`** (optional) — §6.7.

### 2.3 Build the index

```sh
rootmark index build "$VAULT" | jq '{records, embedder_id}'
rootmark index status "$VAULT" | jq '{cold_records, stale_docs}'
```

```json
{
  "records": 4,
  "embedder_id": "minilm-l6-v2/e73168f9+fixed1"
}
```

The first build embeds every chunk (on an Apple M4 roughly 50 ms per chunk; minutes for a few thousand notes). The vectors do not depend on the processor: the same vault gives the same index on every machine. `serve` builds the index itself if there is none.

## 3. Run the server

`rootmark serve` serves one vault: the HTTP API on `127.0.0.1` (never another address), MCP over stdio if asked, and the file watcher (`--watch`) that takes edits made by editors into the log. It runs until SIGTERM/SIGINT or, with `--mcp stdio`, until the MCP client closes the stream.

**One server per vault.** Two servers on the same vault can both write its log (there is no lock): run either the agent-started server of §4 or a stand-alone one, not both. A second server on the same port stops at once with `bind 127.0.0.1:7357: Address already in use`.

A stand-alone server for people and editors (Obsidian, `curl`), in its own terminal:

```sh
rootmark serve --vault "$VAULT" --http 127.0.0.1 --watch
```

```text
rootmark serve: token Created (/Users/ada/notes/.rootmark/server.token)
rootmark serve: http listening on 127.0.0.1:7357
rootmark serve: watching /Users/ada/notes
```

The HTTP token is in `$VAULT/.rootmark/server.token` (mode 600). Every process that can read it acts as you; an agent must not be able to read it (§9). Stop the server with Ctrl-C (or `kill -TERM <pid>`): it compacts the index and says `stopped`.

To call the HTTP API from a terminal, define a small helper in the shell where you work (every call sends the token and a channel label, `X-Consumer`):

```sh
TOKEN="$(cat "$VAULT/.rootmark/server.token")"
rmk() { curl -s -H "Authorization: Bearer $TOKEN" -H "X-Consumer: terminal" -H 'Content-Type: application/json' "http://127.0.0.1:7357/v1/$1" ${2:+-d "$2"}; echo; }
rmk status | jq '{mdk_version, spec_version, index: .index.state, unattested: .attestation.unattested, human: .server_policy.identity.human_id}'
```

```json
{
  "mdk_version": "0.3.1",
  "spec_version": "2.0.3",
  "index": "ok",
  "unattested": "unverified",
  "human": "ada"
}
```

`status` is `GET`; every other endpoint is `POST /v1/<tool>` with a JSON body (`rmk <tool> '<json>'`). The full list of tools, fields and error codes is [CLIENT_CONTRACT.md](CLIENT_CONTRACT.md).

## 4. Connect an agent

An agent talks MCP over stdio: its client starts `rootmark serve --mcp stdio` itself, so **that** process is the vault's server while the agent's session runs — give it `--http 127.0.0.1 --watch` too, and Obsidian and your terminal use the same server. Stop any stand-alone server (§3) first.

### 4.1 Claude Code

From the project directory where you run Claude Code (`--scope project` writes `.mcp.json` there; `--scope user` makes it available everywhere):

```sh
claude mcp add rootmark --scope project \
  -e ROOTMARK_MODELS="$ROOTMARK_MODELS" \
  -- "$HOME/.local/bin/rootmark" serve --vault "$VAULT" --mcp stdio --http 127.0.0.1 --watch
claude mcp list
```

which writes `.mcp.json`:

```json
{
  "mcpServers": {
    "rootmark": {
      "type": "stdio",
      "command": "/Users/ada/.local/bin/rootmark",
      "args": ["serve", "--vault", "/Users/ada/notes", "--mcp", "stdio", "--http", "127.0.0.1", "--watch"],
      "env": { "ROOTMARK_MODELS": "/Users/ada/.local/share/rootmark/models" }
    }
  }
}
```

Use absolute paths (the client does not read your shell profile). A project-scoped server waits for your approval: `claude mcp list` says `⏸ Pending approval (run claude to approve)` until you start `claude` in that directory and accept the `rootmark` server (once). In a session, `/mcp` lists rootmark's tools: `status`, `search`, `read_block`, `history`, `blame`, `propose`, `promote` (always `denied` for an agent), `checkpoint`, `checkpoints`, `events`, `trace`. Tell the agent to write with `propose` (a new file: `mode: "create"`; adding to a file: `mode: "append"` with content that starts with its own heading, e.g. `## Proposal: …`) and to answer from `search`.

#### Keep the agent out of `.rootmark/` and the vault's files

`propose` is the agent's governed path; nothing stops a desktop agent that runs as you from also reading `.rootmark/server.token` and writing the vault's files directly. A process that reads the token can call the HTTP API as you — it is in the human channel (THREAT_MODEL) and can promote its own writes; a direct file write is ingested by the watcher (as the human's under `unattested = "canonical"`, `unverified` under `"unverified"`). Close both for Claude Code with its permission rules **and** its sandbox — the rules stop the Read/Edit/Write tools and the file commands Claude Code recognizes (`cat`, `>`), the sandbox stops every other process a Bash command starts (`python3 -c 'open(…)'`):

```json
{
  "permissions": {
    "deny": ["Read(//Users/ada/notes/.rootmark/**)", "Edit(//Users/ada/notes/**)"]
  },
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "filesystem": {
      "denyRead": ["/Users/ada/notes/.rootmark"],
      "denyWrite": ["/Users/ada/notes"]
    }
  }
}
```

Put it in `~/.claude/settings.json` (or your organisation's managed settings, which a project cannot loosen), with your vault's absolute path: a permission rule writes it with two leading slashes (`//Users/…`), a sandbox path with one. `allowUnsandboxedCommands: false` keeps the agent from retrying a refused command outside the sandbox; `failIfUnavailable: true` refuses to start rather than run unsandboxed. MCP servers run outside the sandbox, so `rootmark serve --mcp stdio` still reads its token and writes the vault. Checked with Claude Code 2.1.318 on macOS (P2-Q.1, `docs/rounds/P2-Q.1/progress/R3_agent_isolation.md`): with these settings a session refused all five attempts — the Read tool on the token, `cat`, `python3 open()`, `echo > note.md`, the Write tool — while the same session without them did all five. Without such a boundary, treat the agent as able to act as you. Other agents need the equivalent of their own (a separate OS user is the strongest).

### 4.2 Any other MCP client

Any client that starts a stdio server works the same way: command `rootmark`, arguments `serve --vault <absolute vault path> --mcp stdio --http 127.0.0.1 --watch`, environment `ROOTMARK_MODELS=<models dir>`. The protocol version is `2025-06-18`; one JSON-RPC message per line.

### 4.3 The agent's name

The MCP channel records every write as `agent:<clientInfo.name>/<clientInfo.version>` (e.g. `agent:claude-code/2.1.0`); a promotion rule's `actor = "agent:<name>"` matches the name with any version. To see the names the agents of a vault have used (after their first `propose`), read them from the event log:

```sh
jq -r 'select(.meta.provenance.author_kind == "Agent") | .meta.provenance.source' "$VAULT/.rootmark/events.jsonl" | sort -u
```

## 5. The Obsidian plugin

The plugin shows rootmark inside Obsidian: the status bar, a blame gutter, proposals with Promote, search, checkpoints and traces; and it **attests** every save you make in Obsidian, so that with `unattested = "unverified"` your own edits count as yours (`canonical`) while files written by other processes stay `unverified`. It talks only HTTP to a running `rootmark serve` and never starts one; it reads `[server] http_port` and `[identity] human_id` from the vault's recipe and the token from `.rootmark/server.token`. It has no settings page: the recipe is its configuration. It is not in the community store; install it from the release by hand or with BRAT. Desktop only.

### 5.1 Install from the release (by hand)

Download the plugin package and its checksum list (`SHA256SUMS.local` lists the files built with the release, among them the plugin zip), check it, and unpack it into the vault's plugin folder under the plugin's id, `rootmark`:

```sh
VER=0.3.1
cd ~/rootmark-install
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/rootmark-$VER.zip"
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/SHA256SUMS.local"
[ "$(awk -v f="obsidian/rootmark-$VER.zip" '$2 == f {print $1}' SHA256SUMS.local)" = "$(shasum -a 256 "rootmark-$VER.zip" | cut -d' ' -f1)" ] && echo "zip OK"
mkdir -p "$VAULT/.obsidian/plugins"
rm -rf "$VAULT/.obsidian/plugins/rootmark"
unzip -q "rootmark-$VER.zip" -d "$VAULT/.obsidian/plugins"
mv "$VAULT/.obsidian/plugins/rootmark-$VER" "$VAULT/.obsidian/plugins/rootmark"
ls "$VAULT/.obsidian/plugins/rootmark"
```

```text
zip OK
main.js		manifest.json	styles.css
```

(On Linux use `sha256sum` for `shasum -a 256`.) Then open the vault in Obsidian. The first time a vault that has a plugin folder is opened, Obsidian asks **"Do you trust the author of this vault?"** with two buttons:

- **Trust author and enable plugins** — community plugins are turned on for this vault; go on with step 2.
- **Browse vault in Restricted Mode** — plugins stay off; go on with step 1.

1. **Settings → Community plugins → Exit Restricted mode** (once per vault; older Obsidian versions call the button "Turn on community plugins").
2. Under **Installed plugins** press the reload icon if **rootmark** is not listed, then switch **rootmark** on.
3. Make sure the server runs (§3, or the agent's server of §4) and the recipe has `[server] http_port` and `[identity] human_id`. The status bar shows

   ```text
   rootmark 0.3.1 · human:ada
   ```

   and, after your first saves, `attested 3 · unattested 0`. `rootmark: offline` means no server answered on the recipe's port: start it and run the command **rootmark: Reconnect to rootmark serve** (Ctrl/Cmd-P).

### 5.2 Use

Commands (Ctrl/Cmd-P, "rootmark: …"): **Show proposal blocks** (the vault's `proposal` and `unverified` blocks with **Promote**, which asks for a reason; the plugin's **Demote** button is shown disabled in 0.3.x — undo a promotion over HTTP, `rmk demote` in §6.4, or with `rootmark demote`), **Search** (query, mode, `k`, *Include proposals*, *At checkpoint*), **Checkpoints** (name the current state, list), **Traces** (what each answer relied on), **Refresh blame of the active note** (the gutter: every block's layer and author). Each save in Obsidian is attested: the watcher then takes the change as yours. Renaming a note in Obsidian (with *Settings → Files and links → Automatically update internal links* on) is attested as a rename, and the notes whose links Obsidian rewrote as link updates.

### 5.3 Install with BRAT

With the [BRAT](https://github.com/TfTHacker/obsidian42-brat) plugin installed and enabled: BRAT → *Add beta plugin* → `arknsensei/rootmark` (it downloads `main.js`, `manifest.json`, `styles.css` from the latest release), then enable **rootmark** as in step 2 above.

### 5.4 Uninstall

Switch **rootmark** off under *Settings → Community plugins*, then delete its folder:

```sh
rm -rf "$VAULT/.obsidian/plugins/rootmark"
```

## 6. Daily use

The examples use the HTTP helper `rmk` of §3 (the human channel) and the command line. Agents do the same over MCP with the same arguments.

### 6.1 An agent proposes; the file is plain Markdown

Over MCP the agent calls `propose {doc_path, content, reason, mode}`. The command line can do the same write as a named agent, but only while **no server runs** on the vault (stop it with Ctrl-C in its terminal; the CLI and a server would both write the log):

```sh
printf '# Rate decision\n\nThe bank held the rate at 45%%.\n' > /tmp/rate.md
printf '# Idea\n\nTry a coarser grind for the light roast.\n' > /tmp/idea.md
rootmark propose "$VAULT" sources/rates.md /tmp/rate.md --source collector --reason "daily rates" | jq -c '{actor: .actor.id, promoted: (.promoted | length), layers: [.layers[].layer]}'
rootmark propose "$VAULT" agent/idea.md /tmp/idea.md --source helper --reason "an idea" | jq -c '{actor: .actor.id, layers: [.layers[].layer]}'
cat "$VAULT/agent/idea.md"
```

```text
{"actor":"collector","promoted":2,"layers":["canonical","canonical"]}
{"actor":"helper","layers":["proposal","proposal"]}
# Idea

Try a coarser grind for the light roast.
```

The first is under the rule's prefix and becomes `canonical` at once (`promoted` holds one `Promoted` event per lineage, naming the rule); the second stays `proposal`. The file holds exactly the text, nothing else. Start the server again (§3) — and in the shell where you work, read the token again (`TOKEN=…` of §3) — before the next steps.

### 6.2 Who wrote what: `blame` and `history`

```sh
rmk blame '{"doc_path":"agent/idea.md"}' | jq -c '.blocks[] | {layer, author_kind, source, lineage_id}'
```

```json
{"layer":"proposal","author_kind":"Agent","source":"helper","lineage_id":"17fc4a67e346582502eae09a263c011c"}
```

`rmk history '{"lineage_id":"<id>"}'` lists every event of one block lineage with its actor. On the command line: `rootmark blame "$VAULT" agent/idea.md`, `rootmark history "$VAULT" <lineage>`.

### 6.3 Search: unapproved is not knowledge

```sh
rmk search '{"query":"coarser grind","k":5}' | jq -c '[.hits[] | {doc_path, layer}]'
rmk search '{"query":"coarser grind","k":5,"include_proposal":true}' | jq -c '[.hits[] | {doc_path, layer}]'
```

The default answer holds only `canonical` blocks; `include_proposal` adds `proposal` and `unverified` hits after them, labelled and with half the score. Optional fields: `mode` (`hybrid`, `lexical`, `flat`), `path_prefix` (`["projects/2026"]`), `at` (a checkpoint), `date_from`/`date_to`. Every search records a consumption trace (§6.5).

### 6.4 Promote and demote (people only)

```sh
L="$(rmk blame '{"doc_path":"agent/idea.md"}' | jq -r '.blocks[-1].lineage_id')"
rmk promote "{\"lineage_id\":\"$L\",\"reason\":\"checked\"}" | jq '{decision, layer}'
rmk demote  "{\"lineage_id\":\"$L\",\"reason\":\"not yet\"}" | jq '{decision, layer}'
```

A promotion needs a reason (without one: 400, nothing written). To make a search hit visible, promote every lineage of its chunk (`hits[].lineage_ids`). `demote` also undoes a rule's promotion. The command-line form is `rootmark promote "$VAULT" <lineage> --actor human:ada --reason "…"` (with no server running).

### 6.5 What an answer rested on: `trace`

```sh
rmk trace '{"consumer":"terminal"}' | jq -c '.traces[-1] | {trace_id, checkpoint_id, returned: [.returned[].doc_path]}'
```

Every `search` and `read_block` writes one trace line (`.rootmark/traces.jsonl`): the checkpoint it ran at, the query's hash and the blocks it returned. `rootmark trace list "$VAULT"` prints them all.

### 6.6 Checkpoints: yesterday's state, block by block

```sh
rmk checkpoint '{"name":"before-review"}' | jq '{checkpoint_id, name}'
rmk search '{"query":"coarser grind","at":"before-review"}' | jq -c '[.hits[] | .doc_path]'
rmk blame '{"doc_path":"agent/idea.md","at":"before-review"}' | jq -c '[.blocks[].layer]'
```

A checkpoint names the current state (the same state always has the same id). `at` asks `search`, `blame` and `read_block` about that state; nothing is rolled back in your files.

### 6.7 Retention and pruning

rootmark never deletes on its own. To bound `.rootmark/` in a long-lived vault, add to the recipe

```toml
[retention]
keep_checkpoints = 24            # traces and unnamed checkpoints behind the last 24 go to an archive

[prune]
version = "1"

[[prune.rules]]
id = "stale"
layer = "proposal"
idle_checkpoints = 168           # a proposal untouched for 168 checkpoints stops being retrieved
```

then look at the plan and prune only after reading it:

```sh
rmk prune '{"dry_run":true}' | jq '{pruned: (.rules.pruned | length)}'
rmk prune '{}'
```

`canonical` is never pruned; archived files stay in `.rootmark/`. See [Known issues](#known-issues) before pruning.

## 7. Collecting data for an evaluation

To judge how rootmark serves you over weeks, keep its records and summarise them now and then.

**Recorded by rootmark, inside `.rootmark/`:**
- `events.jsonl` — every block event (created, updated, moved, split, merged, deleted), every promotion and demotion with its actor (`human:<id>`, `policy:<rule>`, the agent), the rule and recipe version, every checkpoint. It is append-only and the base of everything else.
- `traces.jsonl` (and `traces/archive/` after a prune) — every search and read: when, at which checkpoint, which blocks it returned.
- `checkpoints/` — the named and unnamed states.
- `proposals.jsonl` — the reason of every `propose`.

**Recorded only if you ask:**
- The attestation log: `rootmark serve … --attest-log <file>` appends one line per attestation (accepted or refused, and why) and per observed file change (attested, `unverified` and why). The file must lie outside the vault.
- The server's own messages go to stderr; keep them: `rootmark serve … 2>>"$HOME/rootmark-logs/serve.log"`. For a server started by an agent's client, wrap the command: `"command": "/bin/sh", "args": ["-c", "exec /Users/ada/.local/bin/rootmark serve --vault /Users/ada/notes --mcp stdio --http 127.0.0.1 --watch --attest-log /Users/ada/rootmark-logs/attest.log 2>>/Users/ada/rootmark-logs/serve.log"]`.

**Not recorded anywhere:** request durations, error counts per endpoint, memory use, index build times. rootmark has no operating telemetry yet (planned: P2-O); measure them from outside if you need them.

**Suggestions.** Name a checkpoint at least once a day (`rmk checkpoint '{"name":"day-2026-11-04"}'`) — layers and searches can then be compared over time. Back up `.rootmark/` with the vault (it is plain files; copy it while no server runs, or accept that the last lines may be partial). Keep `--attest-log` and `serve.log` outside the vault, one pair per vault.

**A summary without content:** `observe.py` reads the vault's records (only reads; it writes nothing into the vault) and prints counts and ratios — writes per channel and agent, promotions by person and by rule, demotions and prunes, layers and the growth of `unverified` at each checkpoint, searches and the most relied-on blocks (named by a hash of their id), attestations accepted and refused by class, the index state. It prints no text, path or query, so the output can be shared. It needs Python 3.10+ only:

```sh
mkdir -p "$HOME/rootmark-logs" && cd "$HOME/rootmark-logs"
curl -fsSLO https://raw.githubusercontent.com/arknsensei/rootmark/v0.3.1/research/p2q/observe.py
rmk status > status.json
python3 observe.py --vault "$VAULT" --status status.json --out "observe-$(date +%F).json"
jq '{events: .events.total, layers: .layers.lineages, promotions: .promotions.lineages_by_actor}' "observe-$(date +%F).json"
```

Add `--attest-log <file>` if the server writes one, and `--since-seq <n>` to count only a period (the `head_seq` of the previous summary). Run it once a day or once a week, at the same time as the named checkpoint.

## 8. Upgrading and migrating

1. Stop the server (and the agent sessions that started one).
2. Install the new version over the old one (§1.1 with the new `VER`); reinstall the Obsidian plugin (§5.1; `rm -rf` of the old folder is part of the commands).
3. Look at the vault's migration plan, then migrate:

```sh
rootmark migrate --dry-run "$VAULT" | jq -c '.plan[] | {migration, required}'
rootmark migrate "$VAULT"
```

**From 0.2.0:** the embedder's arithmetic changed (identical vectors on every processor), so an index built by 0.2.0 is `migration_required` (searches answer `migration_required` until then); `rootmark migrate` re-embeds every chunk (the same time as a first build). Events, ids and `events_hash` do not change. **From 0.1.0:** the recipe has no `[attestation]` table and is read as `unattested = "unverified"`; `migrate` appends `unattested = "canonical"` (the old behaviour) — change it to `"unverified"` if that is what you want. Every migration keeps a rollback record under `.rootmark/migrations/`; `rootmark migrate --rollback <id> "$VAULT"` undoes the newest. Going back to an older rootmark: [CHANGELOG.md](../CHANGELOG.md) "Downgrading".

## 9. Headless use on Linux

On a server without Obsidian, agents write over MCP and a person approves over HTTP — from a terminal or through a bot that relays a remote person's approval. The design and its limits are in [HEADLESS_CHANNEL.md](HEADLESS_CHANNEL.md) and [THREAT_MODEL.md](THREAT_MODEL.md); in short:

- **The token is the human.** Whoever can read `.rootmark/server.token` acts as `human:<id>`. Run the server and the human channel (a bot, your shell) as one OS user (e.g. `rmhuman`) and agents as another (`agent`) that can read neither the vault nor `.rootmark/`. The agent account then cannot write files directly either: it can only `propose`.
- **The agent starts the server through one fixed command** it may run as `rmhuman`, e.g. a `sudoers` line `agent ALL=(rmhuman) NOPASSWD: /usr/local/bin/rootmark serve --vault /srv/vault --mcp stdio --http 127.0.0.1\:7357 --watch` and, in the agent's MCP configuration, the command `sudo -n -u rmhuman /usr/local/bin/rootmark serve --vault /srv/vault --mcp stdio --http 127.0.0.1:7357 --watch`.
- **HTTP stays local** (`127.0.0.1` only): the bot runs on the same machine.

If no agent writes over MCP (people and their tools only), the server can run as a service. An example systemd unit — **not tested in this release**:

```ini
# /etc/systemd/system/rootmark.service
[Unit]
Description=rootmark serve (/srv/vault)
After=network.target

[Service]
User=rmhuman
Environment=ROOTMARK_MODELS=/home/rmhuman/.local/share/rootmark/models
ExecStart=/home/rmhuman/.local/bin/rootmark serve --vault /srv/vault --http 127.0.0.1:7357 --watch --attest-log /var/log/rootmark/attest.log
StandardError=append:/var/log/rootmark/serve.log
Restart=on-failure
KillSignal=SIGTERM
TimeoutStopSec=60

[Install]
WantedBy=multi-user.target
```

Such a service and an agent-started server must not serve the same vault at the same time (§3, one server per vault).

## 10. Troubleshooting

| You see | Meaning | Do |
|---|---|---|
| `error: policy … rejected: … missing field version_trigger` (or another table) | the recipe lacks a required table | use the full recipe of §2.2; `rootmark policy lint "$VAULT"` names the field |
| `error: --http 127.0.0.1: no port given and no [server] http_port in policy.toml` | no port | add `[server] http_port` or pass `--http 127.0.0.1:<port>` |
| `bind 127.0.0.1:7357: Address already in use` | another server (or program) has the port | one server per vault (§3); stop the other one |
| HTTP 401 | missing or old token | read `.rootmark/server.token` again (it rotates) |
| HTTP 403 `denied` on promote/attest/checkpoint | no `[identity] human_id` in the recipe, or the policy refuses | set `human_id`, restart the server |
| `status.index.state` not `ok`, searches answer `migration_required` | the index was built by an older rootmark or with another embedder/tokenizer | `rootmark migrate "$VAULT"` (server stopped) |
| `rootmark: offline` in Obsidian | no server on the recipe's port, or no `http_port` | start the server, then **rootmark: Reconnect to rootmark serve** |
| your Obsidian edits are `unverified` | the save was not attested: the server was not connected when you saved, or an attestation was refused | the status bar's tooltip says why; promote the block, or save again while connected; `--attest-log` records refusals (`code`, `reason`) |
| a change is not in `blame` yet | the watcher takes a change a few seconds after the save | wait; `serve.log` shows `watch: <file> ingested` |
| model files missing | `ROOTMARK_MODELS` unset or wrong | `export ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"`; rerun `fetch_model.sh` |

## Known issues

- **Pruning and shared chunks:** a live `canonical` block that shares a search chunk with a block a prune rule made inactive is not returned by search after that prune (it is still read by `blame`, `read_block` and at earlier checkpoints; nothing is deleted). Look at `prune` with `dry_run` first. To be fixed (P2-C).
- **No lock on a vault:** two servers on the same vault both write its log; run one (§3).
- **Removals are taken a little late, on purpose:** the watcher takes a deleted or moved-away note into the log (its blocks get `BlockDeleted` and leave search; fixed in 0.3.1, P2-Q PQ-18) about `window_s` + 0.5 s after it sees it (2.5 s by default), so that Obsidian's attested rename is still checked against the old path. A note deleted and written again within that time counts as changed, not removed. With `unattested = "unverified"`, a process that deletes a note's file without the plugin removes its `canonical` blocks too — recorded with the actor `unattested_fs`, and still read at earlier checkpoints (§6.6); keep such processes away from the vault's files (§4.1).
- **Claude Code 2.1.318 lists no rootmark tools:** the server connects, but the client's newer request metadata is refused by the MCP library 0.3.x is built with (`Failed to fetch tools: request _meta is missing …` in `claude --debug mcp`; P2-Q.1 PQ1-1). A client that negotiates with `initialize` and sends no such metadata gets the tools (checked by hand); a fix is pending a decision (K-P2Q1-T1).
- **SIGTERM with `--mcp stdio`:** the server logs `stopping` but exits only when the MCP client closes the stream (P2-Q PQ-14); a stand-alone `serve --http … --watch` stops on SIGTERM.
- **Hidden folders are skipped:** `ingest-dir` ignores directories whose name starts with `.` (P2-Q PQ-13); notes there are not ingested.
- **The multilingual model's manifest is not in the release archive** (§1.2 fetches it from the tag).
- **The agent's name is self-declared** (promotion rules, §2.2): any process that starts the agent's client can use its name.
