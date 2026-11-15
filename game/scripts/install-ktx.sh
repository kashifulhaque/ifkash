#!/usr/bin/env bash
# Installs the pinned KTX-Software release for `lf-tools assets`, without sudo.
#
# The script downloads the official release archive from
# github.com/KhronosGroup/KTX-Software, checks its SHA-256 digest, and unpacks the `ktx`
# tool and its library into a directory that you own. It doesn't run the system installer.
# See docs/game/decisions/0010-ktx2-textures.md.
#
# Usage: bash game/scripts/install-ktx.sh
#
# Environment:
#   LF_KTX_DIR  The install directory. The default is
#               ${XDG_DATA_HOME:-$HOME/.local/share}/latticefall/ktx-VERSION.
#               `lf-tools assets` reads the same variable.
set -euo pipefail

# Keep the version in sync with KTX_VERSION in game/crates/tools/src/assets.rs.
VERSION="4.4.2"
BASE_URL="https://github.com/KhronosGroup/KTX-Software/releases/download/v${VERSION}"
DEST="${LF_KTX_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/latticefall/ktx-${VERSION}}"

# The digests are the ones GitHub publishes for each release asset.
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    FILE="KTX-Software-${VERSION}-Darwin-arm64.pkg"
    SHA256="500bd8f9d63358c3f3a0d83b724c8574436a72c37dc0e4bad90ec1ca38032c3c" ;;
  Darwin-x86_64)
    FILE="KTX-Software-${VERSION}-Darwin-x86_64.pkg"
    SHA256="efecc685ab891a6e119a9fdc8cbe038e135f9a367eb2f5d8a059553f947f1fea" ;;
  Linux-x86_64)
    FILE="KTX-Software-${VERSION}-Linux-x86_64.tar.bz2"
    SHA256="a8781bad05f9624edbf910b7f258cd0a4ba7d3e63b49ecc0a0ab440bf6a0a245" ;;
  Linux-aarch64 | Linux-arm64)
    FILE="KTX-Software-${VERSION}-Linux-arm64.tar.bz2"
    SHA256="60382e7b842177b8048bd58ccdc770383f8ef65b94452a25d3afdb55f2405c5a" ;;
  *)
    echo "install-ktx: no pinned KTX-Software build for $(uname -s) $(uname -m)." >&2
    echo "install-ktx: install version ${VERSION} from ${BASE_URL%/download/*} and put ktx on the PATH." >&2
    exit 1 ;;
esac

if [ -x "$DEST/bin/ktx" ] && "$DEST/bin/ktx" --version 2>/dev/null | grep -q "v${VERSION}"; then
  echo "install-ktx: ktx ${VERSION} is already in $DEST"
  exit 0
fi

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "install-ktx: downloading $FILE"
curl -fsSL -o "$TMP/$FILE" "$BASE_URL/$FILE"
GOT="$(sha256 "$TMP/$FILE")"
if [ "$GOT" != "$SHA256" ]; then
  echo "install-ktx: checksum mismatch for $FILE" >&2
  echo "install-ktx: expected $SHA256, got $GOT" >&2
  exit 1
fi

mkdir -p "$TMP/tree/bin" "$TMP/tree/lib"
case "$FILE" in
  *.pkg)
    # Expand the installer package instead of running it. The tool finds its library
    # through the @executable_path/../lib run path, so bin/ and lib/ stay side by side.
    pkgutil --expand-full "$TMP/$FILE" "$TMP/pkg"
    cp "$TMP"/pkg/*-tools.pkg/Payload/usr/local/bin/ktx "$TMP/tree/bin/"
    cp -a "$TMP"/pkg/*-library.pkg/Payload/usr/local/lib/. "$TMP/tree/lib/"
    ;;
  *.tar.bz2)
    mkdir "$TMP/tar"
    tar -xjf "$TMP/$FILE" -C "$TMP/tar" --strip-components=1
    cp "$TMP/tar/bin/ktx" "$TMP/tree/bin/"
    cp -a "$TMP/tar/lib/." "$TMP/tree/lib/"
    ;;
esac

"$TMP/tree/bin/ktx" --version | grep -q "v${VERSION}" || {
  echo "install-ktx: the unpacked ktx doesn't run or isn't version ${VERSION}" >&2
  exit 1
}

mkdir -p "$(dirname "$DEST")"
rm -rf "$DEST"
mv "$TMP/tree" "$DEST"
echo "install-ktx: installed $("$DEST/bin/ktx" --version) in $DEST"
