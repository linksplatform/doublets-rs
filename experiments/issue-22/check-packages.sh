#!/usr/bin/env bash
# Compile every maintained manifest on stable, including the standalone FFI packages.
set -euo pipefail
cd "$(dirname "$0")/../.."
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$(pwd -W 2>/dev/null || pwd)/target"
log_dir=${ISSUE22_LOG_DIR:-target/issue-22-logs}
mkdir -p "$log_dir"
status=0
checked=0
# Git is available on every CI runner. Capture discovery first so a failed
# command cannot be swallowed by process substitution and pass with zero checks.
manifests=$(git ls-files --cached --others --exclude-standard -- 'Cargo.toml' '**/Cargo.toml')
while IFS= read -r manifest; do
    if ! grep -q '^\[package\]' "$manifest"; then
        continue
    fi
    checked=$((checked + 1))
    package=$(dirname "$manifest")
    name=${package//\//-}
    echo "Checking $package on stable (log: $log_dir/$name.log)"
    if ! cargo check --manifest-path "$manifest" --all-targets --all-features --locked > "$log_dir/$name.log" 2>&1; then
        echo "FAILED: $package"
        cat "$log_dir/$name.log"
        status=1
    fi
done <<< "$manifests"
if [[ $checked -eq 0 ]]; then
    echo 'No package manifests were checked.' >&2
    exit 1
fi
exit "$status"
