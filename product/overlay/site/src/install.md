# Install

RootMark 0.4.0 runs on **macOS on Apple silicon** and **Linux x86_64** (glibc 2.38 or newer), natively or in a **container image** (`linux/amd64`). Every release asset is listed in one `SHA256SUMS` and has a build provenance attestation: `gh attestation verify <file> -R rootmark-ai/rootmark`.

## macOS and Linux: the install script

```sh
VER=0.4.0
curl -fsSLO "https://github.com/rootmark-ai/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/rootmark-ai/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | shasum -a 256 -c      # Linux: sha256sum -c
gh attestation verify install.sh -R rootmark-ai/rootmark   # optional: provenance
bash install.sh --version "$VER"                       # needs curl, tar, shasum/sha256sum, jq
export PATH="$HOME/.local/bin:$PATH" ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark --version            # rootmark 0.4.0
rootmark conformance >/dev/null   # level 3, 39/39
```

`install.sh` downloads the archive for your platform and `SHA256SUMS`, checks the archive, installs `~/.local/bin/rootmark` and the default embedder model under `~/.local/share/rootmark/models` (checked against its manifest). **`ROOTMARK_MODELS` must point there** for search and the server: the binary does not look for the model by itself.

## Homebrew

```sh
brew install rootmark-ai/tap/rootmark
"$(brew --prefix rootmark)/share/rootmark/models/fetch_model.sh" "$(brew --prefix rootmark)/share/rootmark"
export ROOTMARK_MODELS="$(brew --prefix rootmark)/share/rootmark/models"
rootmark conformance >/dev/null   # level 3, 39/39
```

The formula installs the binary and the model manifest; the model files are fetched once with the command above (the formula's caveats say the same).

## Docker

The image has the binary of the release's Linux archive, the default model (with its licence under `/usr/share/rootmark/models/minilm-l6-v2/`) and `ROOTMARK_MODELS` set. It runs as a non-root user (uid 10001); give it your own uid so that it can write `.rootmark/` in the mounted vault.

```sh
IMG=ghcr.io/rootmark-ai/rootmark:0.4.0          # the same image: rootmark/rootmark:0.4.0 (Docker Hub)
docker run --rm "$IMG" conformance >/dev/null                     # level 3, 39/39
docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" ingest-dir /vault /vault
docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" index build /vault
```

The image's default command is the MCP server over stdio **with the watcher**: `serve --vault /vault --mcp stdio --watch`. An agent starts it itself; for Claude Code, in the project's `.mcp.json`:

```json
{
  "mcpServers": {
    "rootmark": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "--user", "501:20", "-v", "/Users/ada/notes:/vault",
               "ghcr.io/rootmark-ai/rootmark:0.4.0"]
    }
  }
}
```

(`501:20` and `/Users/ada/notes` are an example: use `id -u`, `id -g` and your vault's path.) The HTTP API listens on 127.0.0.1 **inside** the container and cannot be published with `-p`; the image is for agents over MCP. People use the HTTP API and the Obsidian plugin with a native install. The image is `linux/amd64`; on Apple silicon Docker Desktop runs it under emulation (enable Rosetta in Docker Desktop's settings for usable speed).

Verify the image: `gh attestation verify oci://ghcr.io/rootmark-ai/rootmark:0.4.0 -R rootmark-ai/rootmark`; `docker buildx imagetools inspect` shows the same digest on both registries.

## The Obsidian plugin

Not in the community store. Install it with [BRAT](https://github.com/TfTHacker/obsidian42-brat) (`rootmark-ai/rootmark`: BRAT reads the release's `main.js`, `manifest.json` and `styles.css`), or by hand from the release's `rootmark-obsidian-0.4.0.zip` (checked with `grep ' rootmark-obsidian-0.4.0.zip$' SHA256SUMS | shasum -a 256 -c`) into `<vault>/.obsidian/plugins/rootmark/`. See the [Guide](guide.md), §5.

## Your first vault

```sh
export VAULT="$HOME/notes"          # any folder of Markdown files; use a copy the first time
rootmark ingest-dir "$VAULT" "$VAULT"
rootmark index build "$VAULT"
rootmark serve --vault "$VAULT" --http 127.0.0.1:7357 --watch   # the HTTP API for people and editors
```

Keep `--watch` on whenever a server runs for the vault, also in an agent's MCP configuration: only a server started with `--watch` takes in, when it starts, the changes made while no server was running. Then connect an agent and install the plugin: [Guide](guide.md), §3–§5.

## From source

Rust 1.92.0 (`rust-toolchain.toml`): `cargo build --release --locked -p rootmark-server --bin rootmark`, then `make models` and `make conformance`.
