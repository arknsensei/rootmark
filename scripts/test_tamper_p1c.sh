#!/usr/bin/env bash
# P1-C deliberate corruptions (TASKS C9; VERIFY V2, V3): each must fail.
#
#   scripts/test_tamper_p1c.sh
#
# 1. model manifest: a copy of models/ whose MANIFEST.json changes one
#    blake3 (model file linked, not copied) → `mdk-index models verify` fails;
#    a model file one byte longer → fails.
# 2. embedder_id: a built index (mini_vault) whose `embedders.jsonl` mixes
#    two ids (manifest hash kept consistent) → `mdk search` fails; a
#    manifest naming another embedder → fails; the untouched index searches.
# 3. fixtures: a copy of spec/fixtures with one byte of C3-01/expected.json
#    changed → conformance fails (manifest); the same change with the
#    manifest row updated → the Level 3 checker fails (output differs).
# Nothing in the source tree is modified (temporary copies only). Exit 1 if
# any corruption is accepted.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
work="$(mktemp -d -t mdk-tamper-p1c.XXXXXX)"
trap 'rm -rf "$work"' EXIT
cargo build -q --release -p mdkernel-core --bin mdk -p mdkernel-index --bin mdk-index
mdk="$root/target/release/mdk"
mdki="$root/target/release/mdk-index"
n=0 rejected=0
expect_fail() { # expect_fail <label> <cmd...>
  local label="$1"; shift
  n=$((n + 1))
  if "$@" >"$work/out.$n" 2>&1; then
    echo "tamper P1-C: ACCEPTED (must fail): $label" >&2
  else
    rejected=$((rejected + 1))
    echo "tamper P1-C: rejected: $label — $(grep -m1 -iE 'error|panicked|fail' "$work/out.$n" | cut -c1-160)"
  fi
}
b3() { # b3 <file> → full blake3 hex (research venv)
  (cd research && uv run --frozen python -c 'import blake3,sys; print(blake3.blake3(open(sys.argv[1],"rb").read()).hexdigest())' "$1")
}

# ---- 1. model manifest
src="$(cd models && pwd)"
name="$(jq -r .name models/MANIFEST.json)"
mk_models() { # mk_models <dir>: MANIFEST copy, files linked
  mkdir -p "$1/$name/1_Pooling"
  cp models/MANIFEST.json "$1/"
  for f in $(jq -r '.files | keys[]' models/MANIFEST.json); do
    ln -s "$src/$name/$f" "$1/$name/$f"
  done
}
mk_models "$work/m0"
MDK_MODELS="$work/m0" "$mdki" models verify >/dev/null || { echo "tamper P1-C: the untouched model copy does not verify" >&2; exit 1; }
mk_models "$work/m1"
old="$(jq -r '.files["tokenizer.json"].blake3' models/MANIFEST.json)"
new="0${old:1}"; [ "$new" != "$old" ] || new="1${old:1}"
sed -i.bak "s/$old/$new/" "$work/m1/MANIFEST.json"
expect_fail "model manifest blake3 changed" env MDK_MODELS="$work/m1" "$mdki" models verify
mk_models "$work/m2"
rm "$work/m2/$name/sentence_bert_config.json"
{ cat "$src/$name/sentence_bert_config.json"; printf ' '; } >"$work/m2/$name/sentence_bert_config.json"
expect_fail "model file changed" env MDK_MODELS="$work/m2" "$mdki" models verify

# ---- 2. embedder_id
v="$work/vault"
"$mdk" ingest-dir "$v" spec/fixtures/mini_vault >/dev/null
"$mdk" checkpoint create "$v" --name t0 >/dev/null
"$mdk" index build "$v" >/dev/null
"$mdk" search "$v" espresso --at t0 >/dev/null || { echo "tamper P1-C: the untouched index does not search" >&2; exit 1; }
idx="$v/.mdkernel/index"
cp -R "$idx" "$work/index.orig"
first="$(head -1 "$idx/vectors/embedders.jsonl")"
id="$(jq -r .embedder_id <<<"$first")"
{ jq -c '.embedder_id = "other/00000000"' <<<"$first"; tail -n +2 "$idx/vectors/embedders.jsonl"; } >"$work/emb"
mv "$work/emb" "$idx/vectors/embedders.jsonl"
h="$(b3 "$idx/vectors/embedders.jsonl")"
jq --arg h "$h" '.embedders_blake3 = $h' "$idx/manifest.json" >"$work/man" && mv "$work/man" "$idx/manifest.json"
expect_fail "embedder_id mixed in embedders.jsonl" "$mdk" search "$v" espresso --at t0
rm -rf "$idx" && cp -R "$work/index.orig" "$idx"
jq --arg id "other/00000000" '.embedder_id = $id' "$idx/manifest.json" >"$work/man" && mv "$work/man" "$idx/manifest.json"
expect_fail "index manifest names another embedder" "$mdk" search "$v" espresso --at t0
rm -rf "$idx" && cp -R "$work/index.orig" "$idx"
"$mdk" search "$v" espresso --at t0 >/dev/null || { echo "tamper P1-C: restored index does not search" >&2; exit 1; }

# ---- 3. fixtures
conf() { # conf <fixture root>: levels 1–3 against <root>, report in $work
  local r="$work/report.json"
  rm -f "$r"
  MDK_FIXTURES="$1" MDK_CONFORMANCE_REPORT="$r" cargo test -q -p mdkernel-core --release --test conformance >/dev/null 2>&1 \
    || { echo "error: level 1-2 checker failed (manifest $(jq -r .fixtures_manifest "$r" 2>/dev/null))"; return 1; }
  MDK_FIXTURES="$1" MDK_CONFORMANCE_REPORT="$r" cargo test -q -p mdkernel-index --release --test conformance_c3 >/dev/null 2>&1 \
    || { echo "error: level 3 checker failed ($(jq -r '[.tests[] | select(.result != "pass") | .id] | join(",")' "$r" 2>/dev/null))"; return 1; }
  jq -e '.level_claimed == 3 and .passed == .total and .total == 30' "$r" >/dev/null
}
cp -R spec/fixtures "$work/fx0"
conf "$work/fx0" || { echo "tamper P1-C: untouched fixture copy fails conformance" >&2; exit 1; }
cp -R spec/fixtures "$work/fx1"
f="$work/fx1/C3-01/expected.json"
python3 - "$f" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
i = s.index('"agent/zanzibar.md"')
open(p, "w", encoding="utf-8").write(s[:i] + '"agent/zanzibaR.md"' + s[i + len('"agent/zanzibar.md"'):])
PY
expect_fail "C3 fixture byte changed (manifest mismatch)" conf "$work/fx1"
nh="$(b3 "$f" | cut -c1-32)"
jq --arg h "$nh" '.files["C3-01/expected.json"] = $h' "$work/fx1/MANIFEST.json" >"$work/m.json" && mv "$work/m.json" "$work/fx1/MANIFEST.json"
expect_fail "C3 expected ranking changed (manifest updated)" conf "$work/fx1"

echo "tamper P1-C: $rejected/$n corruptions rejected"
[ "$rejected" -eq "$n" ] || exit 1
