#!/usr/bin/env bash
# Builds lf-wasm into src/lib/latticefall/wasm/: cargo build, wasm-bindgen, wasm-opt -O3.
# Run it through `mise run game:wasm`. The output is committed until ADR 0003 is approved,
# so the build must be deterministic: re-running it leaves `git status` clean.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/src/lib/latticefall/wasm"
NAME="lf_wasm"
CACHE="$HOME/Library/Caches/worker-build"

export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.99.1}"

# Prefer tools on PATH (CI installs them); fall back to the worker-build cache.
find_tool() {
  local name="$1" fallback="$2"
  if command -v "$name" >/dev/null 2>&1; then
    command -v "$name"
  elif [ -x "$fallback" ]; then
    echo "$fallback"
  else
    echo "error: $name not found on PATH or at $fallback" >&2
    return 1
  fi
}
WASM_BINDGEN="$(find_tool wasm-bindgen "$CACHE/wasm-bindgen-aarch64-apple-darwin-0.2.106/wasm-bindgen")"
WASM_OPT="$(find_tool wasm-opt "$CACHE/wasm-opt-arm64-macos-126/bin/wasm-opt")"

# The CLI must match the wasm-bindgen crate version in Cargo.lock, or the glue breaks.
LOCK_VERSION="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[version = "]/, ""); print; exit }' "$ROOT/game/Cargo.lock")"
CLI_VERSION="$("$WASM_BINDGEN" --version | awk '{print $2}')"
if [ "$LOCK_VERSION" != "$CLI_VERSION" ]; then
  echo "error: wasm-bindgen CLI is $CLI_VERSION but game/Cargo.lock pins $LOCK_VERSION" >&2
  exit 1
fi

# Keep build paths out of the binary so the output is the same on every machine.
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$ROOT=/lf --remap-path-prefix=$CARGO_HOME_DIR=/cargo"
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=abort
export CARGO_PROFILE_RELEASE_STRIP=debuginfo

cd "$ROOT/game"
cargo build --release --target wasm32-unknown-unknown -p lf-wasm

rm -rf "$OUT"
mkdir -p "$OUT"
"$WASM_BINDGEN" --target web --out-dir "$OUT" --out-name "$NAME" \
  "$ROOT/game/target/wasm32-unknown-unknown/release/$NAME.wasm"

"$WASM_OPT" -O3 \
  --enable-bulk-memory --enable-bulk-memory-opt --enable-nontrapping-float-to-int \
  --enable-sign-ext --enable-mutable-globals --enable-multivalue --enable-reference-types \
  "$OUT/${NAME}_bg.wasm" -o "$OUT/${NAME}_bg.wasm"

RAW=$(wc -c < "$OUT/${NAME}_bg.wasm" | tr -d ' ')
GZ=$(gzip -9 -c "$OUT/${NAME}_bg.wasm" | wc -c | tr -d ' ')
echo "game:wasm: ${NAME}_bg.wasm raw $RAW bytes, gzip $GZ bytes (budget 1500000 gzipped)"
