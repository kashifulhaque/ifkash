#!/usr/bin/env bash
# Downloads the CC0 Poly Haven textures that build_cave.py reads, into
# ~/cinderheart-art/cave-textures/ (or the folder given as the first argument), and checks
# each file's MD5 against textures.json. Files already present with the right MD5 are kept.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
dest="${1:-$HOME/cinderheart-art/cave-textures}"
mkdir -p "$dest"

md5_of() {
  if command -v md5 >/dev/null 2>&1; then md5 -q "$1"; else md5sum "$1" | cut -d' ' -f1; fi
}

python3 -I -c 'import json, sys; [print(k, v) for k, v in json.load(open(sys.argv[1]))["files"].items()]' \
  "$here/textures.json" |
  while read -r name sum; do
    file="$dest/$name"
    if [[ -f "$file" && "$(md5_of "$file")" == "$sum" ]]; then continue; fi
    asset="${name%_*_1k.jpg}"
    asset="${asset%_nor}"
    url="https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/$asset/$name"
    echo "fetching $name"
    curl -fsSL -A "cave-texture-fetch" "$url" -o "$file.part"
    if [[ "$(md5_of "$file.part")" != "$sum" ]]; then
      echo "error: $name has the wrong MD5; Poly Haven may have updated it" >&2
      rm -f "$file.part"
      exit 1
    fi
    mv "$file.part" "$file"
  done
echo "textures are in $dest"
