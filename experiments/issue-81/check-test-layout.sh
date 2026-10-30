#!/usr/bin/env bash
# Keep Rust test implementations in tests directories.
set -euo pipefail
cd "$(dirname "$0")/../.."
if git --no-pager grep --no-color --untracked --exclude-standard -n -E \
    '^[[:blank:]]*#\[([[:alnum:]_]+::)*test([[:blank:]]*\]|\()' \
    -- '*.rs' ':(exclude,glob)**/tests/**' ':(exclude,glob)**/target/**'; then
    echo 'Move Rust test implementations into tests/.' >&2
    exit 1
else
    result=$?
    if [[ $result != 1 ]]; then exit "$result"; fi
fi
echo 'All Rust test implementations are under tests/.'
