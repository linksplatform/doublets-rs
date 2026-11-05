#!/usr/bin/env bash
# Negative control for `integration/tests/fusion.rs`.
#
# The fusion test asserts that a nine-layer decorator stack emits exactly the same
# machine code as the bare store. This script proves the test is not vacuous: it
# rewrites the `#[inline]` attributes of the forwarding macros to `#[inline(never)]`,
# re-runs the test (which must fail), then restores the file.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
macros="$root/doublets/src/decorators/macros.rs"
backup="$(mktemp)"
trap 'cp "$backup" "$macros"; rm -f "$backup"' EXIT

cp "$macros" "$backup"

echo "== baseline: the stack must fuse"
cargo test -p integration --test fusion

echo
echo "== negative control: forwarding methods marked #[inline(never)]"
sed -i 's/^        #\[inline\]$/        #[inline(never)]/' "$macros"
if cargo test -p integration --test fusion; then
    echo "FAIL: the fusion test passed even with the decorators kept out of line" >&2
    exit 1
fi

echo
echo "OK: the fusion test detects a surviving decorator layer"
