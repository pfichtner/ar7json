#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <version> <artifacts-dir>" >&2
    exit 1
fi

VERSION="$1"
ARTIFACTS_DIR="$2"

if [[ ! -d "$ARTIFACTS_DIR" ]]; then
    echo "Error: Artifacts directory not found: $ARTIFACTS_DIR" >&2
    exit 1
fi

cd "$ARTIFACTS_DIR"
for f in *; do
    sha256sum "$f" >> "${VERSION}-SHA256SUMS.txt"
done
