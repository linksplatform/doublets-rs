#!/usr/bin/env bash
# Keep Rust test implementations outside source directories.
set -euo pipefail
cd "$(dirname "$0")/../.."
if rg -n --glob '**/src/**/*.rs' --glob '!**/target/**' \
    '^[[:blank:]]*#\[([[:alnum:]_]+::)*test([[:blank:]]*\]|\()' .; then
    echo 'Move Rust test implementations from src/ to tests/.' >&2
    exit 1
else
    result=$?
    if [[ $result != 1 ]]; then exit "$result"; fi
fi
echo 'All Rust test implementations are outside src/.'
