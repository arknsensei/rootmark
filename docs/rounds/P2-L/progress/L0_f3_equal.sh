#!/usr/bin/env bash
# P2-L/F3, F4 before/after: the full chain's code is byte-identical to main's
# (every function of scripts/verify.sh but the new chain_fast and pair_fast,
# and the dispatcher's lines but the new ones), and check_compare_fields.py
# only gained the compare_p2l registration lines.
set -euo pipefail
base="${1:-main}"
fns() { awk '/^[a-z_0-9]+\(\) \{$/{name=$1} name && name !~ /^(chain_fast|pair_fast)\(\)$/ {print} /^}$/{name=""}'; }
a="$(git show "$base:scripts/verify.sh" | fns | shasum -a 256 | cut -d' ' -f1)"
b="$(fns <scripts/verify.sh | shasum -a 256 | cut -d' ' -f1)"
echo "verify.sh functions but chain_fast/pair_fast: $base $a, tree $b, equal $([ "$a" = "$b" ] && echo true || echo false)"
echo "verify.sh lines removed since $base: $(git diff "$base" -- scripts/verify.sh | grep -c '^-[^-]' || true) (the dispatcher's default branch and usage line)"
git diff "$base" -- scripts/verify.sh | grep '^-[^-]' || true
echo "check_compare_fields.py lines removed since $base: $(git diff "$base" -- research/eval/check_compare_fields.py | grep -c '^-[^-]' || true)"
git diff "$base" -- research/eval/check_compare_fields.py | grep '^+[^+]'
