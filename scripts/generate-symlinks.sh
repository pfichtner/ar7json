#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <binary-path>" >&2
    exit 1
fi

BINARY="$1"

if [[ ! -x "$BINARY" ]]; then
    echo "Error: Binary not found or not executable: $BINARY" >&2
    exit 1
fi

mkdir -p symlinks
"$BINARY" symlinks > symlinks/symlinks
