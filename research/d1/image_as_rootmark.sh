#!/usr/bin/env bash
# D-1/R6 (D1-7): the published image standing in for a `rootmark` binary, so
# research/p2q2/claude_probe.py can start real Claude Code against the image's
# MCP stdio exactly as an agent starts it (`docker run -i --rm … IMG`, the
# image's default command `serve --vault /vault --mcp stdio --watch`).
#   image_as_rootmark.sh --version
#   image_as_rootmark.sh serve --vault <host vault> --mcp stdio
# The image is ROOTMARK_IMAGE (default ghcr.io/rootmark-ai/rootmark:0.4.0).
set -euo pipefail
img="${ROOTMARK_IMAGE:-ghcr.io/rootmark-ai/rootmark:0.4.0}"
run=(docker run --rm --platform linux/amd64)
case "${1:-}" in
  --version) exec "${run[@]}" "$img" --version ;;
  serve)
    [ "${2:-}" = --vault ] && [ -n "${3:-}" ] && [ "${4:-}" = --mcp ] && [ "${5:-}" = stdio ] && [ $# -eq 5 ] \
      || { echo "image_as_rootmark: only 'serve --vault <dir> --mcp stdio'" >&2; exit 2; }
    exec "${run[@]}" -i --user "$(id -u):$(id -g)" -v "$3:/vault" "$img" ;;
  *) echo "image_as_rootmark: only --version or serve" >&2; exit 2 ;;
esac
