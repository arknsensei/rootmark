# Releasing RootMark

A release of this repository is made from a **transfer commit** — the tree the development repository's tool produces from its release tag (README, "How this repository is made"). Nothing here is edited by hand. Every step that writes outside a workflow run (a tag, a merge, a release, a registry, DNS) is a maintainer's step.

## Before the transfer (development repository)
1. The release passes the development repository's release gate (`scripts/release_check.sh`, items R1–R21; R21 is a run of real, current Claude Code against the release binary — tools listed and a `propose` — recorded at most 7 days before the release).
2. A maintainer tags the development repository's merged `main`: `vX.Y.Z`.

## Transfer
3. A maintainer runs the development repository's `transfer` workflow with the tag. It builds the product tree twice (equal tree ids required), commits it on `transfer/vX.Y.Z` here (message lines `Lab-Commit`, `Lab-Tag`, `Transfer-Tree`) and opens nothing else.
4. A maintainer opens the pull request `transfer/vX.Y.Z → main`. `ci` (macOS and Linux: test, lint, conformance 39/39; dependency direction and age) and `transfer-verify` (the tree rebuilt from the lab tag: identical tree id, every file byte-equal, `TRANSFER.json` checked, leak check clean) must be green.
5. A maintainer merges it **fast-forward / rebase** (the tree must not change: the merged commit keeps `Transfer-Tree`), then tags it: `git tag -a vX.Y.Z -m "RootMark X.Y.Z" <merged commit>` and pushes the tag.

## Release
6. A maintainer runs the `release` workflow with the tag. It builds every asset from the tag and lists all of them in one `SHA256SUMS`: the archives (`aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`; Windows for information), `install.sh`, `install.ps1`, the wheel, the Obsidian plugin (`rootmark-obsidian-X.Y.Z.zip` and the BRAT files `main.js`, `manifest.json`, `styles.css`) and `THIRD_PARTY_LICENSES`. Each archive is installed with its installer and checked (`--version`, `rootmark conformance` 39/39). Every asset gets a build provenance attestation. The GitHub release is created with the CHANGELOG section as its notes. Then:
   - `image`: the image from the release's Linux archive, pushed once to `ghcr.io/rootmark-ai/rootmark` and `rootmark/rootmark` (`X.Y.Z`, `X.Y`, `latest`; one digest, checked in both registries), with provenance attestations (secret `DOCKERHUB_TOKEN`, variable `DOCKERHUB_USERNAME`);
   - `tap`: the formula from the published `SHA256SUMS`, as a pull request to `rootmark-ai/homebrew-tap` (secret `HOMEBREW_TAP_TOKEN`: that repository only).
7. A maintainer merges the tap pull request. The first time only: make the `ghcr.io` package public (package settings).
8. The `pages` workflow publishes https://docs.rootmark.ai on every push to `main` (the first time: enable Pages with "GitHub Actions" as the source, the custom domain `docs.rootmark.ai`, HTTPS).

## Checks after the release (anyone can run them)
```sh
VER=X.Y.Z
gh release download "v$VER" -R rootmark-ai/rootmark -D rel && cd rel
sha256sum -c SHA256SUMS                                  # macOS: shasum -a 256 -c SHA256SUMS
for f in *; do [ "$f" = SHA256SUMS ] || gh attestation verify "$f" -R rootmark-ai/rootmark; done
gh attestation verify "oci://ghcr.io/rootmark-ai/rootmark:$VER" -R rootmark-ai/rootmark
docker buildx imagetools inspect "ghcr.io/rootmark-ai/rootmark:$VER" | grep Digest
docker buildx imagetools inspect "rootmark/rootmark:$VER" | grep Digest   # the same digest
```

Signing (Apple notarization, Windows Authenticode) is not done; provenance is shown by the attestations.
