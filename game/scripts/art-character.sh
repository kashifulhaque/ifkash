#!/usr/bin/env bash
# Turns a Mixamo character and its clips into the game's GLB, then compresses it.
#
#   bash game/scripts/art-character.sh NAME [--source DIR]
#
# NAME picks the config `game/art/characters/NAME.json`, which maps the Mixamo file names
# to game clip ids. The source files stay outside the repository (by default in
# `~/cinderheart-art/NAME/`); only the baked GLB and its JSON sidecar are written here.
# Runs headless Blender (set BLENDER to use another binary), then `lf-tools assets`, which
# needs the `ktx` tool (`mise run game:ktx`) for the textures.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
name="${1:-}"
if [[ -z "$name" ]]; then
  echo "usage: art-character.sh NAME [--source DIR]" >&2
  echo "configs: $(cd "$root/game/art/characters" && /bin/ls *.json | sed 's/\.json$//' | tr '\n' ' ')" >&2
  exit 2
fi
shift
config="$root/game/art/characters/$name.json"
[[ -f "$config" ]] || { echo "art: no config at $config" >&2; exit 2; }

blender="${BLENDER:-$(command -v blender || echo /opt/homebrew/bin/blender)}"
[[ -x "$blender" ]] || { echo "art: Blender not found; install it (brew install --cask blender) or set BLENDER" >&2; exit 2; }

log="$(mktemp -t lf-art)"
trap 'rm -f "$log"' EXIT
status=0
"$blender" --background --factory-startup --python-exit-code 1 \
  --python "$root/game/art/blender/bake_character.py" -- "$config" "$@" >"$log" 2>&1 || status=$?
# Blender is chatty: show the pipeline's own lines and any Python error.
grep -E '^art:|Error|Traceback|^  File|^    ' "$log" || true
if [[ "$status" -ne 0 ]]; then
  echo "art: Blender failed (exit $status); full log: rerun with BLENDER_LOG=1" >&2
  [[ -n "${BLENDER_LOG:-}" ]] && cat "$log" >&2
  exit "$status"
fi

cd "$root/game"
RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.99.1}" cargo run -q -p lf-tools -- assets
echo "art: done. Try /game?dev=1&play=1&art=$name"
