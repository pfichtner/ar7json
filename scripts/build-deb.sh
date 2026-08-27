#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 <target> <version> <workspace>" >&2
    exit 1
fi

TARGET="$1"
VERSION="$2"
WORKSPACE="$3"

VERSION="${VERSION#v}"

case "$TARGET" in
    x86_64-unknown-linux-gnu)      ARCH=amd64 ;;
    aarch64-unknown-linux-gnu)     ARCH=arm64 ;;
    armv7-unknown-linux-gnueabihf) ARCH=armhf ;;
    *)
        echo "Error: Unsupported target for Debian package: $TARGET" >&2
        exit 1
        ;;
esac

PKG_DIR="ar7json_${VERSION}_${ARCH}"
mkdir -p "$PKG_DIR/DEBIAN" "$PKG_DIR/usr/bin"

cp "target/${TARGET}/release/ar7json" "$PKG_DIR/usr/bin/"

cd "$PKG_DIR/usr/bin"
while read -r name; do ln -s ar7json "$name"; done < "${WORKSPACE}/symlinks/symlinks"
cd ../../..

mkdir -p "$PKG_DIR/usr/share/bash-completion/completions"
mkdir -p "$PKG_DIR/usr/share/zsh/vendor-completions"
mkdir -p "$PKG_DIR/usr/share/fish/vendor_completions.d"

cp "${WORKSPACE}/completions/ar7json.bash" "$PKG_DIR/usr/share/bash-completion/completions/ar7json"
cp "${WORKSPACE}/completions/_ar7json" "$PKG_DIR/usr/share/zsh/vendor-completions/_ar7json"
cp "${WORKSPACE}/completions/ar7json.fish" "$PKG_DIR/usr/share/fish/vendor_completions.d/ar7json.fish"

cat > "$PKG_DIR/DEBIAN/control" <<EOF
Package: ar7json
Version: $VERSION
Architecture: $ARCH
Maintainer: Peter Fichtner <1958485+pfichtner@users.noreply.github.com>
Description: Standalone AR7 to JSON converter for AVM FRITZ!Box ar7.cfg configuration files
 Standalone converter between AVM FRITZ!Box ar7.cfg (AR7) format and JSON.
 Parses AVM export files including the header and produces round-trip-safe
 output in both directions.
Section: utils
Priority: optional
EOF

dpkg-deb --build "$PKG_DIR" "ar7json-${VERSION}-${TARGET}.deb"
