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

mkdir -p completions

"$BINARY" completions bash > completions/ar7json.bash
"$BINARY" completions zsh > completions/_ar7json
"$BINARY" completions fish > completions/ar7json.fish
"$BINARY" completions powershell > completions/ar7json.powershell
"$BINARY" completions elvish > completions/ar7json.elvish
