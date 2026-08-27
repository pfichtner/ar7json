#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 <target> <version> <workspace>" >&2
    exit 1
fi

TARGET="$1"
VERSION="$2"
WORKSPACE="$3"

RELEASE_DIR="target/${TARGET}/release"
OUTPUT="ar7json-${VERSION}-${TARGET}.tar.gz"

cd "$RELEASE_DIR"
tar czf "../../../${OUTPUT}" \
    ar7json ar7-to-json json-to-ar7 ar7-check ar7-fmt \
    -C "${WORKSPACE}" completions/
