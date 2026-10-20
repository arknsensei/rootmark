#!/usr/bin/env bash
# Installs a rootmark release (P2-A/A3; docs/RELEASE.md "Kurulum") on Linux
# x86_64 (and macOS arm64): downloads the release archive and SHA256SUMS,
# checks the archive's SHA-256 against SHA256SUMS, installs the binary and
# the embedder model manifest, and fetches the pinned model files (each
# checked against models/MANIFEST.json by fetch_model.sh). Any mismatch or
# missing file is an error; nothing is installed until the archive passed
# every check (a failed model download afterwards is an error too, and leaves
# the installed binary: run fetch_model.sh again).
#
#   install.sh [--version <x.y.z>] [--prefix <dir>] [--from <dir>] [--no-model]
#
#   --version  the release (default: the latest GitHub release)
#   --prefix   install under <dir>/bin and <dir>/share/rootmark (default
#              ~/.local)
#   --from     a directory holding the archive and SHA256SUMS instead of the
#              GitHub release (CI checks the artefacts this way)
#   --no-model do not download the model files (search needs them)
#
# Provenance: `gh attestation verify <archive> -R arknsensei/rootmark`
# checks the archive's build attestation (GitHub artifact attestations).
set -euo pipefail

repo="arknsensei/rootmark"
version=""; prefix="${HOME}/.local"; from=""; model=true
while [ $# -gt 0 ]; do
  case "$1" in
    --version) version="${2:?--version needs a value}"; shift 2 ;;
    --prefix) prefix="${2:?--prefix needs a value}"; shift 2 ;;
    --from) from="${2:?--from needs a value}"; shift 2 ;;
    --no-model) model=false; shift ;;
    -h | --help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "install.sh: unknown argument $1" >&2; exit 2 ;;
  esac
done
die() { echo "install.sh: $*" >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  *) die "no release archive for $(uname -s) $(uname -m) (x86_64 Linux, arm64 macOS; Windows: install.ps1)" ;;
esac
if command -v sha256sum >/dev/null; then sha() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null; then sha() { shasum -a 256 "$1" | cut -d' ' -f1; }
else die "sha256sum or shasum is required"; fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fetch() { # <name> → $tmp/<name>
  if [ -n "$from" ]; then
    [ -f "$from/$1" ] || die "$from/$1 missing"
    cp "$from/$1" "$tmp/$1"
  else
    curl -fsSL --retry 3 -o "$tmp/$1" "https://github.com/$repo/releases/download/v$version/$1" \
      || die "download of $1 (v$version) failed"
  fi
}
if [ -z "$version" ]; then
  if [ -n "$from" ]; then
    a="$(cd "$from" && ls rootmark-*-"$target".tar.gz 2>/dev/null | head -1)"
    [ -n "$a" ] || die "no rootmark-*-$target.tar.gz in $from"
    version="${a#rootmark-}"; version="${version%-"$target".tar.gz}"
  else
    version="$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p' | head -1)"
    [ -n "$version" ] || die "could not read the latest release (use --version)"
  fi
fi
archive="rootmark-$version-$target.tar.gz"
fetch "$archive"
fetch SHA256SUMS
want="$(awk -v f="$archive" '{n = $2; sub(/^\*/, "", n); if (n == f) print $1}' "$tmp/SHA256SUMS")"
[ -n "$want" ] || die "$archive is not listed in SHA256SUMS"
got="$(sha "$tmp/$archive")"
[ "$got" = "$want" ] || die "$archive SHA-256 $got != SHA256SUMS $want"
tar -xzf "$tmp/$archive" -C "$tmp"
d="$tmp/rootmark-$version-$target"
[ -x "$d/rootmark" ] && [ -f "$d/models/MANIFEST.json" ] || die "the archive does not hold rootmark and models/MANIFEST.json"
"$d/rootmark" --version | grep -qx "rootmark $version" || die "the archive's binary is not rootmark $version"

share="$prefix/share/rootmark"
mkdir -p "$prefix/bin" "$share/models"
install -m 0755 "$d/rootmark" "$prefix/bin/rootmark"
cp "$d/models/MANIFEST.json" "$d/models/fetch_model.sh" "$share/models/"
cp "$d/LICENSE" "$share/"
if $model; then
  bash "$share/models/fetch_model.sh" "$share" || die "model download failed (run $share/models/fetch_model.sh $share again)"
fi
echo "install.sh: rootmark $version → $prefix/bin/rootmark (SHA-256 checked against SHA256SUMS)"
echo "install.sh: set ROOTMARK_MODELS=$share/models (search and serve need the model files)"
