#!/usr/bin/env bash
# Fetches the embedder model files pinned in models/MANIFEST.json from
# HuggingFace (GOAL P1-C "Embedder"; TASKS C0) into models/<name>/ (git-ignored).
#
#   scripts/fetch_model.sh [<repo-root>]
#
# Every file is downloaded from the manifest's `source` repository at its
# pinned `revision`; a file already present is kept only if its sha256
# matches the manifest. A download is checked against the manifest's size and
# sha256 before it is moved into place; any mismatch is an error and nothing
# is overwritten. The blake3 check is `rootmark-index models verify` (Rust), the
# next step of `make models`.
set -euo pipefail

root="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
manifest="$root/models/MANIFEST.json"
[ -f "$manifest" ] || { echo "fetch_model: $manifest missing" >&2; exit 1; }
command -v jq >/dev/null || { echo "fetch_model: jq is required" >&2; exit 1; }

sha256() { shasum -a 256 "$1" | cut -d' ' -f1; }
# jq on Windows ends its lines with CRLF: the `\r` is not part of a value
# (GH-0 STOP 7: `size 90405214 != manifest 90405214`; P2-A/A3)
jqr() { jq "$@" | tr -d '\r'; }

name="$(jqr -er .name "$manifest")"
repo="$(jqr -er .source "$manifest")"
rev="$(jqr -er .revision "$manifest")"
dest="$root/models/$name"
mkdir -p "$dest"
tmp="$(mktemp -d -t rootmark-model.XXXXXX)"
trap 'rm -rf "$tmp"' EXIT

n=0
while IFS=$'\t' read -r path remote want_sha size; do
  n=$((n + 1))
  out="$dest/$path"
  if [ -f "$out" ] && [ "$(sha256 "$out")" = "$want_sha" ]; then
    echo "fetch_model: $path present (sha256 ok)"
    continue
  fi
  url="https://huggingface.co/$repo/resolve/$rev/$remote"
  echo "fetch_model: $url"
  curl -fsSL --retry 3 -o "$tmp/file" "$url"
  got_size="$(wc -c <"$tmp/file" | tr -d ' ')"
  [ "$got_size" = "$size" ] || { echo "fetch_model: $path size $got_size != manifest $size" >&2; exit 1; }
  got_sha="$(sha256 "$tmp/file")"
  [ "$got_sha" = "$want_sha" ] || { echo "fetch_model: $path sha256 $got_sha != manifest $want_sha" >&2; exit 1; }
  mkdir -p "$(dirname "$out")"
  mv "$tmp/file" "$out"
done < <(jqr -er '.files | to_entries[] | [.key, .value.remote_path, .value.sha256, (.value.size | tostring)] | @tsv' "$manifest")
[ "$n" -gt 0 ] || { echo "fetch_model: manifest lists no file" >&2; exit 1; }
echo "fetch_model: $n files in models/$name (sha256 ok; run \`make models\` for the blake3 manifest check)"
